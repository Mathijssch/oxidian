use std::{path::Path, borrow::Cow};
use super::note::Note;
use serde_derive::{Serialize,Deserialize}; 

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchEntry<'a> {
    pub path: &'a Path,
    pub title: &'a str,
    pub content: String
}


impl<'a> SearchEntry<'a> {

    pub fn new<U, T> ( note: &'a Note<'a>, stopwords: T, max_len: Option<usize>) -> Self 
    where 
        U: AsRef<str>, 
        T: Iterator<Item=U>
    {
        let mut content = match max_len { 
            Some(l) => note.content.chars().take(l).collect(),
            None => note.content.clone()
        };

        for stopword in stopwords {
            content = content.replace(stopword.as_ref(), " ");
        };

        SearchEntry {
            path: &note.path, 
            title: &note.title, 
            content,
        }
    }

}



//impl<'a> From<&'a Note<'a>> for SearchEntry<'a> {
    
//    fn from (note: &'a Note<'a>) -> Self {
//        SearchEntry {
//            path: &note.path,
//            title: &note.title, 
//            content: &note.content
//        }
//    }

//}

