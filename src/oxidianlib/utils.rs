use std::{path::{Path, PathBuf}, fs::File};
use std::io::Read;

use pulldown_cmark::html;

use super::exporter::ExportConfig;
use figment::{Figment, providers::{Serialized, Format, Toml}};
use figment::Error;

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

/// Return the contents of a file at path `path` as a String.
pub fn read_note_from_file<T: AsRef<Path>>(path: T) -> Result<String, std::io::Error> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents) 
}


///Get the first char in a string if there is one, else return space.
///The default is an arbitrary value which we don't expect in practice. 
pub fn initial<T: AsRef<str>>(text: T) -> char {
    let mut chars = text.as_ref().chars();
    while let Some(initial) = chars.next() {
        if initial.is_alphabetic() { return initial }
    }
    ' '
}

///Create a new path that relates to `new_ref` like `path` does to `original`. 
///
///Example 
///-------
///```
///let path = Path::from("indir/subdir/file.txt"); 
///let base_path = Path::from("indir"); 
///let new_path = Path::from("outdir");
///
///let moved_path = move_to(path, base_path, new_path);
///assert_eq!(moved_path, Ok(Path::from("outdir/subdir/file.txt")));
///
///```
pub fn move_to(path: &Path, original: &Path, new_ref: &Path) -> Result<PathBuf, std::path::StripPrefixError> {
    let relative_path = path.strip_prefix(original)?;
    Ok(new_ref.join(relative_path))
}

//pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
//where P: AsRef<Path>, {
//    let file = File::open(filename)?;
//    Ok(io::BufReader::new(file).lines())
//}

/// Convert a given string containing Markdown content to a html representation.
pub fn markdown_to_html(markdown: &str) -> String {  
    // Set up options and parser. Strikethroughs are not part of the CommonMark standard
    // and we therefore must enable it explicitly.
    let mut options = pulldown_cmark::Options::empty();
    options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
    options.insert(pulldown_cmark::Options::ENABLE_TABLES);
    options.insert(pulldown_cmark::Options::ENABLE_FOOTNOTES);
    options.insert(pulldown_cmark::Options::ENABLE_TASKLISTS);

    let parser = pulldown_cmark::Parser::new_ext(&markdown, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    return html_output;
}


/// Prepend a slash in front of a path, making it absolute.
pub fn prepend_slash<T: AsRef<Path>>(path: T) -> PathBuf {
    let slash = Path::new("/");
    slash.join(path.as_ref())
}

/// Read the configuration of the application from a file at the given location. 
/// The values from `ExportConfig::default()` is used for the fields that weren't 
/// specified in the given file.
pub fn read_config_from_file(config_path: &Path) -> Result<ExportConfig, Error> {
    let configuration: ExportConfig = Figment::from(Serialized::defaults(ExportConfig::default()))
    .merge(Toml::file(config_path))
    .extract()?;
    Ok(configuration)

    //let mut file = File::open(config_path).map_err(|_err| ReadConfigError::NoSuchFile(config_path.to_path_buf()))?;
    //let mut buffer = String::new();
    //file.read_to_string(&mut buffer).map_err(|_| ReadConfigError::ReadToString)?;
    //toml::from_str(&buffer).map_err(|_| ReadConfigError::InvalidToml(config_path.to_path_buf()))
}


/// Remove the first `n` lines from a string. Return a new owned string with the first `n` lines
/// of `input` removed.
pub fn remove_first_n_lines(input: &str, n: usize) -> String {
    let mut offset = 0;
    let mut lines = input.lines();

    for _ in 0..n {
        if let Some(line) = lines.next() {
            offset += line.len() + 1; // Add 1 to account for the newline character
        } else {
            // If there are fewer lines than n, return an empty string
            return String::new();
        }
    }
    input[offset..].to_string()
}


pub fn generate_tag_page_name(name: &str) -> PathBuf { 
    return PathBuf::from(format!("tag-{}.html", name));
}


