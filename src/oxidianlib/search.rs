use std::path::Path;
use super::note::Note;
use serde_derive::{Serialize,Deserialize}; 

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchEntry<'a> {
    pub path: &'a Path,
    pub title: &'a str,
    pub content: &'a str
}


impl<'a> From<&'a Note<'a>> for SearchEntry<'a> {

    fn from (note: &'a Note<'a>) -> Self {
        SearchEntry {
            path: &note.path,
            title: &note.title, 
            content: &note.content
        }
    }

}

