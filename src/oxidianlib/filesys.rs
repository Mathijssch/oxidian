use super::constants::NOTE_EXT;
use std::io;
use std::path::PathBuf;
use walkdir::WalkDir;
///Functions to interact with the file system

///Return an iterator over notes in the given directory.
pub fn get_all_notes(path: &str) -> impl Iterator<Item = io::Result<PathBuf>> {
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

/// Unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_filter_markdown_html_files() {
        let dir = tempdir().expect("Failed to create temporary directory");
        let dir_path = dir.path();

        let md_file_path = dir_path.join("sample.md");
        let html_file_path = dir_path.join("sample.html");
        let txt_file_path = dir_path.join("sample.txt");

        // Create sample files in the temporary directory
        File::create(&md_file_path).expect("Failed to create .md file");
        File::create(&html_file_path).expect("Failed to create .html file");
        File::create(&txt_file_path).expect("Failed to create .txt file");

        // Call the filter_markdown_html_files function
        let result: Vec<_> = get_all_notes(dir_path.to_str().unwrap())
            .map(|entry| entry.map(|path| path.file_name().unwrap().to_owned()))
            .collect::<Result<_, _>>()
            .expect("Failed to filter files");

        // Check if the function found the correct files
        assert_eq!(result.len(), 2);
        assert!(result.contains(&md_file_path.file_name().unwrap().to_owned()));
        assert!(result.contains(&html_file_path.file_name().unwrap().to_owned()));
    }
}
