use std::fs::File;
use std::io::{self, Error, Write};
use std::path::{Path, PathBuf};

//use super::formatting::link_to_md;
use super::{filesys, html};
use super::frontmatter::{extract_yaml_frontmatter, parse_frontmatter};
use super::link::Link;
use super::obs_placeholders::Sanitization;
use super::utils::{markdown_to_html, read_note_from_file};
use super::{obs_admonitions, obs_comments, obs_links, obs_placeholders, formatting};
use yaml_rust::Yaml;
use super::load_static::HTML_TEMPLATE;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Note<'a> {
    pub path: PathBuf,
    pub links: Vec<Link>,
    pub frontmatter: Option<Yaml>,
    content: String,
    placeholders: Vec<Sanitization>,
    pub title: String,
    pub backlinks: Vec<&'a Link>,
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

    pub fn new(path: PathBuf) -> Result<Self, std::io::Error> {
        let content = Self::sanitize(&read_note_from_file(&path)?);
        let frontmatter =
            extract_yaml_frontmatter(&content).and_then(|fm| parse_frontmatter(&fm).ok());
        let (content, placeholders) = Self::remove_protected_elems(content);
        let links = Self::find_obsidian_links(&content);
        let title = Self::get_title(&path, frontmatter.as_ref());
        Ok(Note {
            path,
            links,
            content,
            title,
            frontmatter,
            placeholders,
            backlinks: vec![],
        })
    }

    fn find_obsidian_links(content: &str) -> Vec<Link> {
        obs_links::find_obsidian_links(content)
    }

    fn remove_protected_elems(content: String) -> (String, Vec<Sanitization>) {
        // Remove math elements
        return obs_placeholders::disambiguate_protected(&content);
    }



    fn sanitize(content: &str) -> String {
        return format_admonitions(&strip_comments(content));
    }

    ///Export the current note to a html file at the specified path.
    pub fn to_html(&self, path: &Path) -> Result<(), Error> {
        if let Some(parent_dir) = path.parent() {
            filesys::create_dir_if_not_exists(&parent_dir).unwrap();
        }
        let file = File::create(path)?;
        let mut writer = io::BufWriter::new(file);

        let mut content = self.content.to_owned();
        for placeholder in &self.placeholders {
           content = content.replace(&placeholder.get_placeholder(), &placeholder.0);
        }

        content = self.process_links(content);

 
        let html_content = markdown_to_html(&content);

        let template_content = HTML_TEMPLATE; 

        let backlinks: Vec<String> = self.backlinks.iter()
            .map(|link| 
                html::link(
                    &filesys::convert_path(&link.target, Some("html")).unwrap(),
                    &link.alias.clone().unwrap(), ""
                )
            ).collect();
        
        template_content.lines()
            .map(|line| {
                line.replace(r"{{content}}", &html_content)
            })
            .map(|line| 
                line.replace(r"{{backlinks}}", &html::ul(backlinks.iter(), "class=\"backlinks\"")))
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

fn format_admonitions(note: &str) -> String {
    let mut admonitions = obs_admonitions::AdmonitionParser::new();
    let mut output = String::with_capacity(note.len());
    for line in note.lines() {
        if let Some(new) = admonitions.process_line(line) {
            output.push_str(new.as_str());
        } else {
            output.push_str(line);
        }
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


