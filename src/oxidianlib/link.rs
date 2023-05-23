use std::path::{Path, PathBuf};

use regex::Regex;

lazy_static! {
    static ref OBSIDIAN_NOTE_LINK_RE: Regex =
        Regex::new(r"^(?P<file>[^#|]+)??(#(?P<section>.+?))??(\|(?P<label>.+?))??$").unwrap();
}

use super::errors;

#[derive(Debug)]
pub struct Link {
    pub target: PathBuf,
    pub subtarget: Option<String>,
    pub alias: Option<String>,
    source_string: String,
    is_attachment: bool,
}

pub enum LinkType {
    External,
    Note,
    Internal(InternalType),
    Attachment(FileType),
}

pub enum FileType {
    Pdf,
    Image,
    Video,
    Audio,
    Misc,
}

pub enum InternalType {
    Header,
    Blockref,
}

type InvalidLink = errors::InvalidObsidianLink<String, String>;

// TODO -- make smaller by using references.
struct Capture {
    file: String,
    internal_ref: Option<String>,
    alias: Option<String>,
}

fn attachment_type_from_file(file: &Path) -> FileType {
    let ext_own = match file.extension() {
        Some(e) => {
            e.to_string_lossy().to_lowercase() 
        },
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

    if super::constants::IMG_EXT.contains(&ext) {
        return FileType::Image;
    }

    if super::constants::VIDEO_EXT.contains(&ext) {
        return FileType::Video;
    }

    if super::constants::AUDIO_EXT.contains(&ext) {
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

        return LinkType::Note;
    }

    ///Construct a new [Link] from an Obsidian-styled reference
    pub fn from_obsidian_link(
        obs_link: &str,
        is_attachment: bool,
    ) -> Result<Link, errors::InvalidObsidianLink<String, String>> {
        let captures = OBSIDIAN_NOTE_LINK_RE
            .captures(obs_link)
            .ok_or_else(|| errors::InvalidObsidianLink::ParseError(obs_link.to_string()))?;
        let target = captures
            .name("file")
            .ok_or_else(|| errors::InvalidObsidianLink::MissingMatchGroup {
                link: obs_link.to_string(),
                group: "file".to_string(),
            })?
            .as_str();
        let alias = captures.name("label").map(|v| v.as_str().to_string());
        let subtarget = captures.name("section").map(|v| v.as_str().to_string());

        Ok(Link {
            target: PathBuf::from(target),
            subtarget,
            alias,
            is_attachment,
            source_string: obs_link.to_string(),
        })
    }
}
