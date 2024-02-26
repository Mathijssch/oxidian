use chrono::{DateTime, NaiveDateTime, Utc};
use std::io::Read;
use std::process::Command;
use std::{
    fs::File,
    path::{Path, PathBuf},
    time::SystemTime,
};

use crate::utils::constants::TAG_DIR;
use crate::components::link::Dimensions;
use crate::core::html::HtmlTag;
use crate::exporting::wrap_pulldown_cmark::MarkdownParser;
use crate::obsidian::tags::Tag;
use pulldown_cmark::html;

use crate::exporting::config::ExportConfig;
use figment::Error;
use figment::{
    providers::{Format, Serialized, Toml},
    Figment,
};

use log::{info, debug};

use super::errors::PathInputToOutputError;

/// Return the contents of a file at path `path` as a String.
pub fn read_file_to_str<T: AsRef<Path>>(path: T) -> Result<String, std::io::Error> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

pub fn byte_to_kb(bytes: usize) -> f64 {
    bytes as f64 / 1024.
}

///Get the first char in a string if there is one, else return space.
///The default is an arbitrary value which we don't expect in practice.
pub fn initial<T: AsRef<str>>(text: T) -> char {
    let mut chars = text.as_ref().chars();
    while let Some(initial) = chars.next() {
        if initial.is_alphabetic() {
            return initial;
        }
    }
    ' '
}

/// Convert a [std::time::SystemTime] into a [chrono::NaiveDateTime]
pub fn to_datetime(date: SystemTime) -> NaiveDateTime {
    let datetime_utc: DateTime<Utc> = date.into();
    datetime_utc.naive_utc()
}

///Capitalize the first character in the given string.
pub fn capitalize_first(input: &str) -> String {
    if let Some(first_char) = input.chars().next() {
        let mut result = String::with_capacity(input.len());
        result.push_str(&first_char.to_uppercase().collect::<String>());
        result.push_str(&input[1..].to_lowercase());
        result
    } else {
        String::new()
    }
}

///Render a html link to the page of a fully-specified tag (with subtags, separated by `/`.)
pub fn render_full_tag_link(tag: &str, tag_dir: &Path) -> String {
    let mut path = tag_dir.to_owned();
    let mut tag_name = tag;
    let mut components = tag.split("/").peekable();
    while let Some(component) = components.next() {
        if let Some(parent_comp) = components.peek() {
            path.push(parent_comp);
        }
        tag_name = component;
        path.push(generate_tag_page_name(&component));
    }
    HtmlTag::a(path.to_str().unwrap()).wrap(&capitalize_first(tag_name))
}

///Get the time at which the file at the given path was added to a git repository.
///If it fails to do so for whatever reason, return None.
pub fn get_git_creation_time<T: AsRef<Path>>(path: T) -> Option<NaiveDateTime> {
    // Run the git command
    let git_output = Command::new("git")
        .args(&["log", "-1", "--format=%ai", "--reverse"])
        .arg(path.as_ref())
        .output();

    info!("Git output: {:?}", git_output);

    // Early escapes
    let output = match git_output {
        Ok(out) => out,
        Err(e) => {
            info!("Git command failed. Got error: {:?}", e);
            return None;
        }
    };

    if !output.status.success() {
        info!("Git command failed. Got status: {:?}", output.status);
        return None;
    }

    // Got a valid response from git.
    let git_date_str = match String::from_utf8(output.stdout) {
        Ok(date_str) => date_str,
        Err(e) => {
            info!(
                "Could not convert git output to a string. Got error {:?}",
                e
            );
            return None;
        }
    };
    // Trim the newline characters and remove the last 6 characters (timezone offset)
    //
    if git_date_str.len() < 6 {
        //info!("Git returned empty output");
        return None;
    }

    let trimmed_git_date_str = git_date_str.trim_end().get(..git_date_str.len() - 6);

    // Parse the git date string into a DateTime
    if let Ok(git_date_parsed) =
        NaiveDateTime::parse_from_str(trimmed_git_date_str.unwrap_or(""), "%Y-%m-%d %H:%M:%S")
    {
        return Some(git_date_parsed);
    }
    println!("Could not parse git date '{:?}'", trimmed_git_date_str);
    None
}

///Create a new path that relates to `new_ref` like `path` does to `original`.
///
///Example
///-------
///```ignore
///# use std::path::Path;
///let path = Path::new("indir/subdir/file.txt");
///let base_path = Path::new("indir");
///let new_path = Path::new("outdir");
///
///let moved_path = move_to(path, base_path, new_path);
///assert_eq!(moved_path, Ok(Path::new("outdir/subdir/file.txt")));
///
///```
pub fn move_to(
    path: &Path,
    original: &Path,
    new_ref: &Path,
) -> Result<PathBuf, PathInputToOutputError> {
    //let path = path.canonicalize()?;
    //let original = original.canonicalize()?;
    //debug!("path: `{}`. Original input: `{}`", path.to_string_lossy(),
    //    original.to_string_lossy());
    let relative_path = path.strip_prefix(original)?;
    Ok(new_ref.join(relative_path))
}

/// Convert a given string containing Markdown content to a html representation.
pub fn markdown_to_html(markdown: &str) -> String {
    // Set up options and parser. Strikethroughs are not part of the CommonMark standard
    // and we therefore must enable it explicitly.
    let mut options = pulldown_cmark::Options::empty();
    options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
    options.insert(pulldown_cmark::Options::ENABLE_TABLES);
    options.insert(pulldown_cmark::Options::ENABLE_FOOTNOTES);
    options.insert(pulldown_cmark::Options::ENABLE_TASKLISTS);
    options.insert(pulldown_cmark::Options::ENABLE_HEADING_ATTRIBUTES);

    let basic_parser = pulldown_cmark::Parser::new_ext(&markdown, options);
    let wrapper = MarkdownParser::new(basic_parser);
    let mut html_output = String::new();
    html::push_html(&mut html_output, wrapper);
    html_output
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
    PathBuf::from(format!("tag-{}.html", name))
}

pub fn format_tag_path(tag: &Tag) -> String {
    let mut components: Vec<&str> = tag.tag_path.split('/').collect();
    if let Some(last) = components.pop() {
        let mut result = "/".to_string();
        result.push_str(TAG_DIR);
        result.push('/');
        for comp in components {
            result.push_str(comp);
            result.push('/');
        }
        result.push_str(&generate_tag_page_name(last).to_string_lossy());
        result
    } else {
        tag.tag_path.clone()
    }
}

/// Parse dimensions, given in the form
/// - \d: width in pixels
/// - \d\s?x\s?\d: width times height in pixels.
///
/// If the given string is not in the correct form, then return None
pub fn parse_dims<T: AsRef<str>>(alias: T) -> Option<Dimensions> {
    parse_dims_inner(alias.as_ref())
}

/// See [[parse_dims]].
fn parse_dims_inner(alias: &str) -> Option<Dimensions> {
    let dimensions_raw: Vec<&str> = alias.split('x').collect();
    if let Some(width) = dimensions_raw.get(0) {
        if let Some(w) = width.parse::<u32>().ok() {
            if let Some(h) = dimensions_raw.get(1) {
                return Some(Dimensions {
                    width: w,
                    height: h.parse::<u32>().ok(),
                });
            }
            return Some(Dimensions::new(w));
        }
    }
    None
}
