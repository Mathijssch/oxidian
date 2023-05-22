use std::path::Path;

pub struct Link<'a> {
    target: &'a Path, 
    alias: String, 
    link_type: LinkType
}

pub enum LinkType {
    External,
    Post,
    Image, 
    Video, 
    Document
}
