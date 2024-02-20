use std::path::{Path, PathBuf};
use clap::builder::OsStr;
use crate::utils::{utils, constants::NOTE_EXT, filesys::relative_to};
use regex::Regex;
use super::errors;

lazy_static! {
    static ref OBSIDIAN_NOTE_LINK_RE: Regex = Regex::new(
        r"^(?P<file>[^#\^|]*)??([#](?P<block>\^)??(?P<section>[^\^\]\[\#]+?))??(\|(?P<label>.+?))??$"
    )
    .unwrap();
}

use super::note::Note;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Dimensions {
    pub width: u32, 
    pub height: Option<u32>
}

impl Dimensions {
    pub fn new(width: u32) -> Self {
        Dimensions { width, height: None }
    }

    pub fn new_with_details(width: u32, height: u32) -> Self { 
        Dimensions { width, height: Some(height) }
    }
}


#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct Link {
    pub target: PathBuf,
    pub subtarget: Option<String>,
    pub alias: Option<String>,
    pub source_string: String,
    pub is_attachment: bool,
    pub broken: bool,
}

#[derive(Debug, PartialEq)]
pub enum LinkType {
    External,
    Note,
    Internal,
    Attachment(FileType),
}

#[derive(Debug, PartialEq)]
pub enum FileType {
    Pdf,
    Image,
    Video,
    Audio,
    Misc,
}


fn attachment_type_from_file(file: &Path) -> FileType {
    let ext_own = match file.extension() {
        Some(e) => e.to_string_lossy().to_lowercase(),
        None => {
            return FileType::Misc;
        }
    };
    let ext = ext_own.as_str();
    //TODO -- Something weird is going on here: double reference??
    if ext.len() >= 5 {
        // very long file extension, probably a false positive.
        return FileType::Misc;
    }

    if crate::utils::constants::IMG_EXT.contains(&ext) {
        return FileType::Image;
    }

    if crate::utils::constants::VIDEO_EXT.contains(&ext) {
        return FileType::Video;
    }

    if crate::utils::constants::AUDIO_EXT.contains(&ext) {
        return FileType::Audio;
    }
    // TODO: Try to find the extension in a more rigorous way.
    match ext {
        "pdf" => FileType::Pdf,
        _ => FileType::Misc,
    }
}

impl Link {
    pub fn link_type(&self) -> LinkType {
        if self.is_attachment {
            let attach_type = attachment_type_from_file(&self.target);
            return LinkType::Attachment(attach_type);
        };

        if &self.target.starts_with("http://") | &self.target.starts_with("https://") {
            return LinkType::External;
        };

        if self.target.file_name().is_none() {
            return LinkType::Internal;
        };

        LinkType::Note
    }

    pub fn set_broken(&mut self, is_broken: bool) { self.broken = is_broken; }
    pub fn set_target<T: Into<PathBuf>>(&mut self, target: T) { self.target = target.into(); }

    ///Express the target of the link relative to the given directory.
    ///If the link is not in the given directory, then the target is not changed.
    pub fn set_relative(mut self, dir: &Path) -> Self { 
        let relative_path = relative_to(&self.target, dir);
        self.target = utils::prepend_slash(relative_path);
        self
    }

    pub fn new<N: Into<String>, T: Into<PathBuf>>(name: N, target: T) -> Self {
        Self {
            target: target.into(),
            subtarget: None,
            alias: Some(name.into()),
            source_string: "".to_string(),
            is_attachment: false,
            broken: false, 
        }
    }

    pub fn from_note(note: &Note) -> Self {
        Link {
            target: note.path.clone(),
            subtarget: None,
            alias: Some(note.title.clone()),
            source_string: "".to_string(),
            is_attachment: false,
            broken: false,
        }
    }

    pub fn link_text(&self) -> String {
        match &self.alias {
            Some(alias) => alias.clone(),
            None => {
                match self.link_type() {
                    LinkType::Internal => { 
                        if let Some(subtgt) = &self.subtarget { subtgt.to_string() }
                        else { String::from(".") }
                    },
                    LinkType::Note =>
                    {
                        if let Some(filename) = self.target.with_extension("").file_name() {
                            filename.to_string_lossy().to_string()
                        } else { 
                            self.target.with_extension("").to_string_lossy().to_string()
                        }
                    }
                    _ => { self.target.to_string_lossy().to_string() }
                }
            }
        }
    }

    ///Construct a new [Link] from a markdown-styled reference
    pub fn from_md_link<T, L, S>(
        md_link: L,
        target: T,
        alias: Option<S>,
        is_attachment: bool,
    ) -> Self
    where
        T: Into<String>,
        L: Into<String> + std::fmt::Debug,
        S: Into<String>,
    {
        let raw_target: String = target.into();
        let (target, subtarget) = &raw_target.split_once("#")
            .map_or_else(
                || (PathBuf::from(&raw_target), None),
                |(t, s)| (PathBuf::from(t), Some(s.into()))
            ); 
        Link {
            target: target.to_owned(),
            subtarget: subtarget.to_owned(),
            alias: alias.map(|s| s.into()),
            source_string: md_link.into(),
            is_attachment,
            broken: false,
        }
    }

    fn needs_md_ext(path: &str, is_attachment: bool) -> bool {
        if is_attachment { return false; }
        if path.len() == 0 { return false; } // Internal link 

        let target_path = Path::new(path);
        // Add markdown extension for notes.
        if let Some(ext) = target_path.extension() {
            if !NOTE_EXT.iter().any(|note_ext| OsStr::from(note_ext) == ext ) {
                return true;
            }
        } else { return true; } 
        false
    }

    pub fn parse_dims(&self) -> Option<Dimensions> {
        if !self.is_attachment { return None; }
        let alias = match &self.alias {
            None => { return None; },
            Some(a) => a
        };
        utils::parse_dims(alias)
    }


    ///Construct a new [Link] from an Obsidian-styled reference
    pub fn from_obsidian_link(
        obs_link: &str,
        is_attachment: bool,
    ) -> Result<Link, errors::InvalidObsidianLink<String, String>> {
        let captures = OBSIDIAN_NOTE_LINK_RE
            .captures(obs_link)
            .ok_or_else(|| errors::InvalidObsidianLink::ParseError(obs_link.to_string()))?;

        let target = match captures.name("file") {
            Some(filename) => filename.as_str().trim(),
            None => "",
        };
        
        let target_path = match Self::needs_md_ext(target, is_attachment) {
            true => PathBuf::from(target.to_owned() + ".md"),
            false => PathBuf::from(target)
        };

        let alias = captures.name("label")
                    .map(|v| v.as_str().to_string());
        let subtarget = captures
            .name("section")
            .map(|v| v.as_str().to_string())
            .and_then(|name| Some(name.trim().to_owned()));

        let exclamation = match is_attachment {
            true => "!", false => ""
        };
        let source_str = format!("{}[[{}]]", exclamation, obs_link).to_string();
        Ok(Link {
            target: target_path,
            subtarget,
            alias,
            is_attachment,
            source_string: source_str,
            broken: false, 
        })
    }
}

impl PartialOrd for Link {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Link {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.link_text().cmp(&other.link_text())
    }
}

impl<'a> From<Note<'a>> for Link {
    fn from(note: Note<'a>) -> Self {
        Link {
            target: note.path.clone(),
            subtarget: None,
            alias: Some(note.title.clone()),
            source_string: "".to_string(),
            is_attachment: false,
            broken: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Link, LinkType};
    use std::{assert_eq, path::PathBuf};

    #[test]
    fn test_from_obsidian_standard() {
        let test_string = "link_to_note";
        let expected_link = Link {
            target: PathBuf::from("link_to_note.md"),
            subtarget: None,
            alias: None,
            is_attachment: false,
            source_string: format!("[[{}]]", test_string).to_string(),
            broken: false, 
        };
        let got_link = Link::from_obsidian_link(test_string, false).unwrap();
        assert_eq!(expected_link, got_link);
    }

    #[test]
    fn test_from_obsidian_blockref() {
        let test_string = "link_to_note#^someblock";
        let expected_link = Link {
            target: PathBuf::from("link_to_note.md"),
            subtarget: Some(String::from("someblock")),
            alias: None,
            is_attachment: false,
            source_string: format!("[[{}]]", test_string).to_string(),
            broken: false,
        };
        let got_link = Link::from_obsidian_link(test_string, false).unwrap();
        assert_eq!(expected_link, got_link);
        assert_eq!(got_link.link_type(), LinkType::Note)
    }

    #[test]
    fn test_from_obsidian_header() {
        let test_string = "link_to_note#someblock";
        let expected_link = Link {
            target: PathBuf::from("link_to_note.md"),
            subtarget: Some(String::from("someblock")),
            alias: None,
            is_attachment: false,
            source_string: format!("[[{}]]", test_string).to_string(),
            broken: false, 
        };
        let got_link = Link::from_obsidian_link(test_string, false).unwrap();
        assert_eq!(expected_link, got_link);
        assert_eq!(got_link.link_type(), LinkType::Note)
    }
    #[test]
    fn test_from_obsidian_with_spaces() {
        let test_string = "link to note";
        let expected_link = Link {
            target: PathBuf::from("link to note.md"),
            subtarget: None,
            alias: None,
            is_attachment: false,
            source_string: format!("[[{}]]", test_string).to_string(),
            broken: false, 
        };
        let got_link = Link::from_obsidian_link(test_string, false).unwrap();
        assert_eq!(expected_link, got_link);
    }

    #[test]
    fn test_from_obsidian_with_leading_spaces() {
        let test_string = " link to note";
        let expected_link = Link {
            target: PathBuf::from("link to note.md"),
            subtarget: None,
            alias: None,
            is_attachment: false,
            source_string: format!("[[{}]]", test_string).to_string(),
            broken: false,
        };
        let got_link = Link::from_obsidian_link(test_string, false).unwrap();
        assert_eq!(expected_link, got_link);
    }

    #[test]
    fn test_from_obsidian_no_file() {
        let test_string = "#internal_id";
        let expected_link = Link {
            target: PathBuf::from(""),
            subtarget: Some("internal_id".to_string()),
            alias: None,
            is_attachment: false,
            source_string: format!("[[{}]]", test_string).to_string(),
            broken: false,
        };
        let got_link = Link::from_obsidian_link(test_string, false).unwrap();
        assert_eq!(expected_link, got_link);
    }

    #[test]
    fn test_from_obsidian_with_trailing_spaces() {
        let test_string = "link to note ";
        let expected_link = Link {
            target: PathBuf::from("link to note.md"),
            subtarget: None,
            alias: None,
            is_attachment: false,
            source_string: format!("[[{}]]", test_string).to_string(),
            broken: false, 
        };
        let got_link = Link::from_obsidian_link(test_string, false).unwrap();
        assert_eq!(expected_link, got_link);
    }

    #[test]
    fn test_from_obsidian_with_alias() {
        let test_string = "link to note|the note I want to mention";
        let expected_link = Link {
            target: PathBuf::from("link to note.md"),
            subtarget: None,
            alias: Some("the note I want to mention".to_string()),
            is_attachment: false,
            source_string: format!("[[{}]]", test_string).to_string(),
            broken:false,
        };
        let got_link = Link::from_obsidian_link(test_string, false).unwrap();
        assert_eq!(expected_link, got_link);
    }
    #[test]
    fn test_from_obsidian_with_subtarget() {
        let test_string = "link to note#header1|the note I want to mention";
        let expected_link = Link {
            target: PathBuf::from("link to note.md"),
            subtarget: Some("header1".to_string()),
            alias: Some("the note I want to mention".to_string()),
            is_attachment: false,
            source_string: format!("[[{}]]", test_string).to_string(),
            broken:false, 
        };
        let got_link = Link::from_obsidian_link(test_string, false).unwrap();
        assert_eq!(expected_link, got_link);
    }
    // More tests...
}
