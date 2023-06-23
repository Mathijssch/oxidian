use std::{path::Path, fs::File};
use std::fs;
use super::constants::NOTE_EXT;
use walkdir::{WalkDir, DirEntry};
use std::io;
//use std::io::{Read, self, BufRead};
use std::path::PathBuf;
use std::io::Read;

use pulldown_cmark::html;

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


pub fn filter_markdown_html_files(path: &str) -> impl Iterator<Item = io::Result<PathBuf>> {
    //let entries = fs::read_dir(path).unwrap();
    let entries = WalkDir::new(path).into_iter().filter_map(Result::ok);
   
    entries.filter_map(|entry| {
        let path = entry.into_path();
        let extension = path.extension()?.to_str()?.to_lowercase();
        
        let contains = NOTE_EXT.iter().any(|ext| **ext == extension);
        if contains {
            Some(Ok(path))
        } else {
            None
        }
    })
}
