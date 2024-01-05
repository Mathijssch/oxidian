use std::fs::File;
use std::io::{self, Error, Write};
use std::path::{Path, PathBuf};
use log::debug;

//use super::formatting::link_to_md;
use super::frontmatter::{extract_yaml_frontmatter, parse_frontmatter};
use super::link::Link;
use super::load_static::HTML_TEMPLATE;
use super::placeholder::Sanitization;
use super::obs_headers::HeaderParser;
use super::utils::{markdown_to_html, read_note_from_file, self};
use super::{filesys, html, obs_tags};
use super::{formatting, obs_admonitions, obs_comments, obs_links, obs_placeholders};
use yaml_rust::Yaml;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Note<'a> {
    pub path: PathBuf,
    pub links: Vec<Link>,
    pub frontmatter: Option<Yaml>,
    pub tags: Vec<obs_tags::Tag>,
    content: String,
    placeholders: Vec<Sanitization>,
    pub title: String,
    pub backlinks: Vec<&'a Link>,
}


impl<'a> AsRef<Note<'a>> for Note<'a> {
    fn as_ref(&self) -> &Note<'a> {
        &self
    }
}

impl<'a> AsMut<Note<'a>> for Note<'a> {
    fn as_mut(&mut self) -> &mut Note<'a> {
        self
    }
}

impl<'a> Note<'a> {
    fn get_author_prefix(frontmatter: &Yaml) -> Option<String> {
        if let Some(author) = frontmatter["authors"][0].as_str() {
            if let Some(year) = frontmatter["year"].as_str() {
                Some(format!("{} ({}) -", author, year))
            } else {
                Some(format!("{} -", author))
            };
        };
        None
    }

    fn get_title(filename: &Path, frontmatter: Option<&Yaml>) -> String {
        let base_title = match frontmatter.and_then(|fm| fm["title"].as_str()) {
            Some(title) => title,
            None => filename
                .file_stem()
                .and_then(|f| f.to_str())
                .unwrap_or("Note"),
        };
        let prefix = frontmatter
            .and_then(|fm| Self::get_author_prefix(fm))
            .unwrap_or_else(|| String::from(""));
        return prefix + base_title;
    }

    fn process_links(&self, mut content: String) -> String {
        for link in &self.links {
            content = content.replace(&link.source_string, &formatting::link_to_md(link));
        }
        return content;
    }

    fn process_tags(&self, mut content: String) -> String {
        for tag in &self.tags {
            content = content.replace(&tag.source, &formatting::tag_to_md(&tag));
        }
        content
    }

    fn process_headers(content: String) -> String { 
        let mut parser = HeaderParser::new();

        // Add a bit of buffer capacity for the newlines that we'll add.
        let mut updated = String::with_capacity(content.len() + 10);
        for line in content.lines() {
            if let Some(updated_line) = parser.process_line(line) {
                updated.push_str(&updated_line);
            } else 
            {
                updated.push_str(line);
            }
            updated.push('\n');
        }
        return updated;
    }
    
    // Get a raw version of the notes, not meant for postprocessing, just for extraction of
    // information.
    pub fn new_raw(path: PathBuf) -> Result<Self, std::io::Error> {
        let mut content = Self::sanitize(&read_note_from_file(&path)?);

        let frontmatter = match extract_yaml_frontmatter(&content) {
            Some(fm_content) => {
                let fm_count = fm_content.lines().count() + 2; // +2 for the surrounding "---"
                                                               // lines
                //debug!("Found {} lines of frontmatter in {:?}", fm_count, path);
                content = utils::remove_first_n_lines(&content, fm_count);
                parse_frontmatter(&fm_content).ok()
            }, 
            None => None
        };

        let links = Self::find_obsidian_links(&content);
        let title = Self::get_title(&path, frontmatter.as_ref());

        Ok(Note {
            path,
            links,
            content,
            title,
            frontmatter,
            placeholders: vec![],
            tags: vec![],
            backlinks: vec![],
        })

    }

    pub fn new(path: PathBuf) -> Result<Self, std::io::Error> {
        let mut content = Self::sanitize(&read_note_from_file(&path)?);

        let frontmatter = match extract_yaml_frontmatter(&content) {
            Some(fm_content) => {
                let fm_count = fm_content.lines().count() + 2; // +2 for the surrounding "---"
                                                               // lines
                //debug!("Found {} lines of frontmatter in {:?}", fm_count, path);
                content = utils::remove_first_n_lines(&content, fm_count);
                parse_frontmatter(&fm_content).ok()
            }, 
            None => None
        };

        // Remove code blocks, and math.
        let (mut content, mut placeholders) = Self::remove_protected_elems(content);
        // Extract the links
        let links = Self::find_obsidian_links(&content);
        // Replace links by placeholders, since they may also contain protected symbols with
        // special meaning, like `^` and `#`.
        content = Self::replace_links_by_placeholders(content, &mut placeholders, &links);
        let tags = Self::find_tags(&content);
        // Replace admonitions by placeholders, so they are not recognized as quotes by the
        // markdown processor 
        content = Self::replace_admonitions_by_placeholders(content, &mut placeholders);
        let title = Self::get_title(&path, frontmatter.as_ref());

        Ok(Note {
            path,
            links,
            content,
            title,
            frontmatter,
            placeholders,
            tags,
            backlinks: vec![],
        })
    }

    fn replace_links_by_placeholders(
        content: String,
        placeholders: &mut Vec<Sanitization>,
        links: &Vec<Link>,
    ) -> String {
        let mut content = content;
        for link in links {
            let link_ph = Sanitization::from(link.source_string.to_string());
            content = content.replace(&link_ph.original, &link_ph.get_placeholder());
            placeholders.push(link_ph);
        }
        content
    }

    fn replace_admonitions_by_placeholders(
        content: String, 
        placeholders: &mut Vec<Sanitization>
        ) -> String {
        let mut admonitions = obs_admonitions::AdmonitionParser::new();
        let mut output = String::with_capacity(content.len());
        for line in content.lines() {
            match admonitions.process_line(line) {
                obs_admonitions::ParseOutput::Placeholder { replacement, placeholder } => {
                    output.push_str(&replacement);
                    if let Some(ph) = placeholder { 
                        placeholders.push(ph);
                    }
                }, 
                obs_admonitions::ParseOutput::None => {output.push_str(line)}
            }
            output.push('\n');
        }
        return output;
    }

    fn find_obsidian_links(content: &str) -> Vec<Link> {
        obs_links::find_obsidian_links(content)
    }
    
    fn find_tags(content: &str) -> Vec<obs_tags::Tag> {
        // TODO: Extract tags from the frontmatter
        obs_tags::find_tags(content)
    }

    fn remove_protected_elems(content: String) -> (String, Vec<Sanitization>) {
        // Remove math elements and code
        obs_placeholders::disambiguate_protected(&content)
    }

    fn sanitize(content: &str) -> String {
        return strip_comments(content);
    }

    ///Export the current note to a html file at the specified path.
    pub fn to_html(&self, path: &Path) -> Result<(), Error> {
        if let Some(parent_dir) = path.parent() {
            filesys::create_dir_if_not_exists(&parent_dir).unwrap();
        }
        let file = File::create(path)?;
        let mut writer = io::BufWriter::new(file);

        let mut content = self.content.to_owned();

        for placeholder in self.placeholders.iter().filter(|p| p.before_markdown) {
            content = content.replace(&placeholder.get_placeholder(), &placeholder.replacement);
        }

        content = self.process_links(content);
        content = self.process_tags(content);
        content = Self::process_headers(content);

        let mut html_content = markdown_to_html(&content);

        for placeholder in self.placeholders.iter().filter(|p| !p.before_markdown) {
            html_content = html_content.replace(&placeholder.get_placeholder(), &placeholder.replacement);
        }

        let template_content = HTML_TEMPLATE;

        let backlinks: Vec<String> = self
            .backlinks
            .iter()
            .map(|link| {
                html::link(
                    &filesys::convert_path(&link.target, Some("html")).unwrap(),
                    &link.alias.clone().unwrap(),
                    "",
                )
            })
            .collect();

        debug!("Note {} has {} backlinks", self.title, backlinks.len());

        template_content
            .lines()
            .map(|line| line.replace(r"{{content}}", &html_content))
            .map(|line| line.replace(r"{{title}}", &self.title))
            .map(|line| {
                line.replace(
                    r"{{backlinks}}",
                    &html::ul(backlinks.iter(), "class=\"backlinks\""),
                )
            })
            .for_each(|line| {
                writer.write_all(line.as_bytes()).unwrap();
                writer.write_all(b"\n").unwrap();
            });
        Ok(())
    }
}

fn strip_comments(note: &str) -> String {
    let mut output = String::with_capacity(note.len());
    for line in note.lines() {
        output.push_str(obs_comments::process_line(line));
        output.push('\n');
    }
    return output;
}

pub fn create_note(path: &str) -> Note {
    let the_path = PathBuf::from(path);
    Note::new(the_path).unwrap()
}

//pub fn print_text_event(e: &Event) {
//    println!("{:?}", e);
//}
