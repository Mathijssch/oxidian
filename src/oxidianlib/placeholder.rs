use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher}
};

#[derive(Hash,Debug,PartialEq,Clone)]
pub struct Sanitization { 
    pub original: String, 
    pub replacement: String,
    pub before_markdown: bool  // Safe to replace before markdown conversion
}

impl Sanitization {
    pub fn get_placeholder(&self) -> String {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        return format!("{}", hasher.finish());
    }

    pub fn from<T: Into<String> + Clone >(string: T) -> Self {
        let replacement = string.clone();
        Sanitization { original: string.into(), replacement: replacement.into(), before_markdown: true }
    }

    pub fn new<T: Into<String>, V: Into<String>>(original: T, replacement: V, before: bool) -> Self {
        Sanitization { original: original.into(), replacement: replacement.into(), before_markdown: before }
    }

}
