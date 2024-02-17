use super::filesys::{relative_to, slugify_path};
use super::{note::Note, utils::prepend_slash};
use std::path::{Path, PathBuf};

use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchEntry<'a> {
    pub path: PathBuf,
    pub title: &'a str,
    pub content: String,
}

impl<'a> SearchEntry<'a> {
    pub fn new<U, T>(
        note: &'a Note<'a>,
        stopwords: T,
        max_len: Option<usize>,
        input_dir: &Path,
    ) -> Self
    where
        U: AsRef<str>,
        T: Iterator<Item = U>,
    {
        let mut content = match max_len {
            Some(l) => note.content.chars().take(l).collect(),
            None => note.content.clone(),
        };

        for stopword in stopwords {
            content = content.replace(stopword.as_ref(), " ");
        }

        SearchEntry {
            path: slugify_path(
                &prepend_slash(&relative_to(&note.path, input_dir)),
                Some("html"),
            )
            .expect(&format!(
                "Failed to sluggify note path for note {}.",
                note.title
            )),
            title: &note.title,
            content,
        }
    }
}
