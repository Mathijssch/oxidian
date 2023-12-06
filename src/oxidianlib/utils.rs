use std::{path::{Path, PathBuf}, fs::File};
use std::io::Read;
use regex::Regex;

use pulldown_cmark::html;

use super::{exporter::ExportConfig, errors::ReadConfigError};

lazy_static! {
    static ref OBSIDIAN_TAG_RE: Regex =
        Regex::new(r"(?<!\\)#([a-zA-Z-_/]+)(?![^[[]*]])").unwrap();
}

//pub fn find_all_occurrences(text: &str, pattern: &str) -> Vec<usize> {
//    let mut indices = Vec::new();
//    let mut start = 0;

//    while let Some(index) = text[start..].find(pattern) {
//        let absolute_index = start + index;
//        indices.push(absolute_index);
//        start = absolute_index + pattern.len();
//    }

//    indices
//}

pub fn read_note_from_file<T: AsRef<Path>>(path: T) -> Result<String, std::io::Error> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents) 
}

pub fn move_to(path: &Path, original: &Path, new_ref: &Path) -> Result<PathBuf, std::path::StripPrefixError> {
    let relative_path = path.strip_prefix(original)?;
    Ok(new_ref.join(relative_path))
}

//pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
//where P: AsRef<Path>, {
//    let file = File::open(filename)?;
//    Ok(io::BufReader::new(file).lines())
//}


pub fn markdown_to_html(markdown: &str) -> String {  
    let parser = pulldown_cmark::Parser::new(&markdown);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    return html_output;
}


// Prepend a slash in front of a path, making it absolute.
pub fn prepend_slash(path: &Path) -> PathBuf {
    let slash = Path::new("/");
    slash.join(&path)
}

pub fn read_config_from_file(config_path: &Path) -> Result<ExportConfig, ReadConfigError<PathBuf>> {
    let mut file = File::open(config_path).map_err(|_err| ReadConfigError::NoSuchFile(config_path.to_path_buf()))?;
    let mut buffer = String::new();
    file.read_to_string(&mut buffer).map_err(|_| ReadConfigError::ReadToString)?;
    toml::from_str(&buffer).map_err(|_| ReadConfigError::InvalidToml(config_path.to_path_buf()))
}


pub fn find_tags(content: &str) {

}


