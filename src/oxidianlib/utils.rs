use std::{path::Path, fs::File};
use std::io::Read;

pub fn find_all_occurrences(text: &str, pattern: &str) -> Vec<usize> {
    let mut indices = Vec::new();
    let mut start = 0;

    while let Some(index) = text[start..].find(pattern) {
        let absolute_index = start + index;
        indices.push(absolute_index);
        start = absolute_index + pattern.len();
    }

    indices
}

pub fn read_note_from_file<T: AsRef<Path>>(path: T) -> Result<String, std::io::Error> {
    let content = String::new();
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents) 
}
