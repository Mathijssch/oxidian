use super::{constants::NOTE_EXT, errors::NotePathError};
use std::{io, fs, ffi::OsStr};
use slugify::slugify;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
///Functions to interact with the file system

/// Create directory if it doesn't exist yet.
pub fn create_dir_if_not_exists(path: &Path) -> Result<(), std::io::Error> {
    //let path = std::path::Path::new(dir_path);
    //
    if !path.exists() {
        match fs::create_dir(path) {
            Ok(_) => {
                println!("Directory '{:?}' created successfully.", path);
                Ok(())
            }
            Err(err) => {
                eprintln!("Error creating directory '{:?}': {:?}", path, err);
                Err(err)
            }
        }
    } else {
        println!("Directory '{:?}' already exists.", path);
        Ok(())
    }
}

///Return an iterator over notes in the given directory.
pub fn get_all_notes(path: &Path) -> impl Iterator<Item = io::Result<PathBuf>> {
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


/// Convert the path to a markdown file to a slugified version of the path with either
/// * The given extension, 
/// * The original extension
///
pub fn convert_path<'a> (path: &'a Path, extension: Option<&str>) -> Result<PathBuf, NotePathError<&'a Path>> {
    let ext = match extension {
        Some(e) => Some(OsStr::new(e)),
        None => {path.extension()}
    };
    //let extension = path.extension().ok_or_else(err)
    let stem = path.file_stem().ok_or_else(| | NotePathError::NoStem(path))?
        .to_str()
        .ok_or_else(|| NotePathError::InvalidUTF8(path))?;

    let mut slug = slugify!(stem);
    
    if let Some(ext) = ext {
        slug.push_str(".");
        slug.push_str(&ext.to_string_lossy());
    }

    if let Some(directory) = path.parent() {
        let output_path = directory.join(slug).to_path_buf();
        return Ok(output_path);
    }
    Ok(PathBuf::from(slug))
}


/// Unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{DirBuilder,File};
    use tempfile::tempdir;

    
    struct TestCase {
        dir: tempfile::TempDir,
        valid_files: Vec<PathBuf>
    }


    /// Test case with a flat directory structure
    fn create_flat_test_case() -> TestCase {
        let dir = tempdir().expect("Failed to create temporary directory");
        let dir_path = dir.path();

        let md_file_path = dir_path.join("sample.md");
        let html_file_path = dir_path.join("sample.html");
        let txt_file_path = dir_path.join("sample.txt");

        // Create sample files in the temporary directory
        File::create(&md_file_path).expect("Failed to create .md file");
        File::create(&html_file_path).expect("Failed to create .html file");
        File::create(&txt_file_path).expect("Failed to create .txt file");


        let valid_files = vec![md_file_path, html_file_path];
        TestCase {
            dir, valid_files
        }
    }

    /// Test case with a leveled directory structure
    fn create_nested_test_case() -> TestCase {
        let dir = tempdir().expect("Failed to create temporary directory");
        let dir_path = dir.path();

        let subdir_path = dir_path.join("notes");
        let dir_builder = DirBuilder::new();
        let _subdir = dir_builder.create(&subdir_path).expect("Failed to create subdir");

        let md_file_path = dir_path.join("sample.md");
        let html_file_path = dir_path.join("sample.html");
        let md_file_subdir_path = subdir_path.join("sample2.md");

        // Create sample files in the temporary directory
        File::create(&md_file_path).expect("Failed to create .md file");
        File::create(&html_file_path).expect("Failed to create .html file");
        File::create(&md_file_subdir_path).expect("Failed to create nested .md file");

        let valid_files = vec![md_file_path, html_file_path, md_file_subdir_path];
        TestCase {
            dir, valid_files
        }
    }

    fn run_test(test_case: TestCase) {
        let dir_path = test_case.dir.path();
        // Call the filter_markdown_html_files function
        let result: Vec<_> = get_all_notes(dir_path)
            .map(|entry| entry.map(|path| path.file_name().unwrap().to_owned()))
            .collect::<Result<_, _>>()
            .expect("Failed to filter files");

        // Check if the function found the correct files
        assert_eq!(result.len(), test_case.valid_files.len()); 
        for valid_file in test_case.valid_files {
            assert!(result.contains(&valid_file.file_name().unwrap().to_owned()));
        }
    }


    #[test]
    fn test_filter_markdown_html_files() {
        run_test(create_flat_test_case());
    }
    
    #[test]
    fn test_nested_directory() {
        run_test(create_nested_test_case());
    }
}
