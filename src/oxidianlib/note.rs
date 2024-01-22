use std::collections::HashSet;
use std::fs::File;
use std::io::{self, Error, Write};
use std::path::{Path, PathBuf};
use log::{debug,info};
use chrono::NaiveDate;

//use super::formatting::link_to_md;
use super::frontmatter::{extract_yaml_frontmatter, parse_frontmatter};
use super::link::Link;
use super::placeholder::Sanitization;
use super::obs_headers::HeaderParser;
use super::obs_highlights::replace_obs_highlights;
use super::utils::{markdown_to_html, read_note_from_file, self};
use super::{filesys, html, obs_tags};
use super::{formatting, obs_admonitions, obs_comments, obs_links, obs_placeholders};
use yaml_rust::Yaml;


#[allow(dead_code)]
#[derive(Debug,Clone)]
pub struct Note<'a> {
    pub path: PathBuf,
    pub links: Vec<Link>,
    pub frontmatter: Option<Yaml>,
    pub tags: Vec<obs_tags::Tag>,
    content: String,
    placeholders: Vec<Sanitization>,
    pub title: String,
    pub backlinks: HashSet<&'a Link>,
    creation_date: Option<NaiveDate> 
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
            backlinks: HashSet::new(),
            creation_date: None
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
        // Get the labels of block-refs
        let blockref_labels = Self::find_blockref_labels(&content);
        content = Self::replace_blockrefs_by_placeholders(content, &mut placeholders, &blockref_labels);
        // Extract the links
        let mut links = Self::find_obsidian_links(&path, base_dir, &content);
        // Replace links by placeholders, since they may also contain protected symbols with
        // special meaning, like `^` and `#`.
        let mut markdown_links = Self::find_markdown_links(&path, &base_dir, &content);
        links.append(&mut markdown_links);
        content = Self::replace_links_by_placeholders(content, &mut placeholders, &links);
        //content = Self::replace_links_by_placeholders(content, &mut placeholders, &markdown_links);
        let tags = Self::find_tags(&content);
        // Replace admonitions by placeholders, so they are not recognized as quotes by the
        // markdown processor 
        content = Self::replace_admonitions_by_placeholders(content, &mut placeholders);
        let title = Self::get_title(&path, frontmatter.as_ref());

        //let creation_date = Self::compute_creation_date(&frontmatter, &path).unwrap();
        Ok(Note {
            path,
            links,
            content,
            title,
            frontmatter,
            placeholders,
            tags,
            backlinks: HashSet::new(),
            creation_date: None
        })
    }

    pub fn cache_creation_time(&mut self, use_git: bool) {
        if self.creation_date.is_some() { return }

        if let Ok(date) = Self::compute_creation_date(&self.frontmatter, &self.path, use_git) {
            self.creation_date = Some(date);
        }
    }

    ///Get the creation date of the note in a given path.
    ///
    pub fn get_modification_time(&self) -> Result<std::time::SystemTime, std::io::Error> {
        filesys::get_modification_time(&self.path)
    }

    fn compute_creation_date(
        frontmatter: &Option<Yaml>, 
        path: &Path,
        use_git: bool
    ) -> Result<NaiveDate, std::io::Error>  {
        // Try frontmatter 
        if let Some(fm) = frontmatter {
            if let Some(date) = fm["date_created"].as_str() {
                debug!("Reading creation date from frontmatter.");
                if let Ok(parsed) = NaiveDate::parse_from_str(date, "%d-%m-%Y") { 
                    return Ok(parsed);
                };
                info!("Failed to read creation date from frontmatter");
            }
        }

        // Try to read the creation date from git.
        if use_git { 
            if let Some(time) = utils::get_git_creation_time(path) {
                return Ok(time.date());
            } else { 
                info!("Failed to read creation date from git");
            };
        };

        // Try to work from the system time 
        let modified_time = filesys::get_modification_time(path)?;
        Ok(utils::to_datetime(modified_time).date())
    }

    pub fn get_creation_date(&self) -> Option<NaiveDate> {
        self.creation_date
    }

    ///Get the creation date of the note in a given path.
    ///
    //pub fn compute_and_cache_creation(&mut self) -> Result<NaiveDate, std::io::Error> {
    //    info!("Getting creation date of {}", self.title);
    //    if let Some(date) = self.creation_date {
    //        return Ok(date);
    //    }
    //    let creation_date = Self::compute_creation_date()?;
    //    self.creation_date = Some(creation_date);
    //    Ok(creation_date)
    //}

    fn replace_links_by_placeholders(
        content: String,
        placeholders: &mut Vec<Sanitization>,
        links: &Vec<Link>,
    ) -> String {
        let mut content = content;
        for link in links {
            let link_ph = Sanitization::from(link.source_string.to_string());
            //if link.source_string.contains("diff") {
            //    info!("Replacing {} with {}", link_ph.original, link_ph.get_placeholder());
            //}
            content = content.replace(&link_ph.original, &link_ph.get_placeholder());
            placeholders.push(link_ph);
        }
        content
    }

    fn replace_blockrefs_by_placeholders(
        content: String,
        placeholders: &mut Vec<Sanitization>,
        labels: &Vec<obs_labels::BlockLabel>,
    ) -> String {
        let mut content = content;
        for label in labels {
            let link_ph = Sanitization::new(label.source.to_string(),
                                            html::HtmlTag::span().with_id(&label.label).wrap(""),
                                            false);
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

    fn find_markdown_links(content: &str) -> Vec<Link> {
        obs_links::find_markdown_links(content)
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

    /// Add a backlink to `self`s set of backlinks. 
    /// 
    /// The provided [Link] should be a link to the note that refers to [self]. 
    pub fn add_backlink(&mut self, link: &'a Link) { self.backlinks.insert(link); }

    ///Export the current note to a html file at the specified path.
    pub fn to_html<U: AsRef<str>>(&self, path: &Path, template_content: U) -> Result<(), Error> {
        self.to_html_inner(path, template_content.as_ref())?;
        Ok(())
    }

    fn process_highlights(content: &str) -> String { replace_obs_highlights(content) }

    fn to_html_inner(&self, path: &Path, template_content: &str) -> Result<(), Error> {
        if let Some(parent_dir) = path.parent() {
            filesys::create_dir_if_not_exists(&parent_dir).unwrap();
        }
        let file = File::create(path)?;
        let mut writer = io::BufWriter::new(file);

        let mut content = self.content.to_owned();

        // Needs to be done before replacing the highlights placeholders back 
        // to avoid false matches.
        content = Self::process_highlights(&content);

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

        let backlinks: Vec<String> = self
            .backlinks
            .iter()
            .map(|link| {
                html::link(
                    &filesys::slugify_path(&link.target, Some("html")).unwrap(),
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
