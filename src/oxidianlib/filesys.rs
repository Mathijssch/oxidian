use super::errors::GetAgeError;
use super::utils::prepend_slash;
use super::{constants::NOTE_EXT, errors::NotePathError};
use log::{debug, error, info, warn};
use slugify::slugify;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::{ffi::OsStr, fs, io};
use walkdir::{WalkDir, DirEntry};
///Functions to interact with the file system

pub enum ResolvedPath {
    Broken,
    Unchanged,
    Updated(PathBuf),
}


/// Several options
/// 1. `path` is empty. Do nothing.
/// 2. `path` is an absolute path.
///     - Check if the path exists relative to `base_dir`.
///     - If not, flag it as broken.
/// 3. `path` is not absolute and has at least one parent
///     - Try to find it relative to `relative_to`.
///     - If it cannot be found, then mark as broken.
/// 4. `path` is not absolute and has no parents
///     - Try to find it in `relative_to`. If found, do nothing.
///     - If not found, and if `full_search`, then try to find it elsewhere in `base_dir`. If found, return full path.
///     Otherwise, do nothing.
pub fn resolve_path<'a>(
    path: &Path,
    relative_to: &Path,
    base_dir: &Path,
    full_search: bool,
    ignore: &'a Vec<PathBuf>
) -> ResolvedPath {
    if path.components().next().is_none() {
        return ResolvedPath::Unchanged;
    } // Empty path
      // corresponds to an
      // internal link.
      // Don't look for a
      // file
    if path.is_absolute() {
        let stripped = path.strip_prefix("/").unwrap_or(path);
        if base_dir.join(stripped).exists() {
            return ResolvedPath::Unchanged;
        } else {
            return ResolvedPath::Broken;
        }
    }

    let full = relative_to.join(path);
    if base_dir.join(&full).exists() {
        return ResolvedPath::Updated(prepend_slash(full));
    }

    if base_dir.join(path).exists() {
        return ResolvedPath::Updated(prepend_slash(path));
    }

    //info!("Full search is {}, parent is {:?}, links is {:?}", full_search, path.parent(), path);
    if full_search && just_filename(path) {
        if let Some(full_path) = find_recursive(base_dir, path, ignore) {
            return ResolvedPath::Updated(prepend_slash(&full_path));
        }
    }
    ResolvedPath::Broken
}

fn just_filename(path: &Path) -> bool { path.parent() == Some(Path::new("")) }

/// Create directory if it doesn't exist yet.
pub fn create_dir_if_not_exists(path: &Path) -> Result<(), std::io::Error> {
    //let path = std::path::Path::new(dir_path);
    //
    if path.components().count() == 0 {
        warn!("Trying to create empty path {:?}", path);
        return Ok(());
    };
    if !path.exists() {
        match fs::create_dir_all(path) {
            Ok(_) => {
                info!("Directory '{:?}' created successfully.", path);
                Ok(())
            }
            Err(err) => {
                error!("Error creating directory '{:?}': {:?}", path, err);
                Err(err)
            }
        }
    } else {
        debug!("Directory '{:?}' already exists. Skipping.", path);
        Ok(())
    }
}

///Find `path` in `base_dir` or a subdirectory thereof. 
///If no match is found, None is returned.
pub fn find_recursive<'a>(base_dir: &Path, path: &Path, ignore: &'a Vec<PathBuf>) -> Option<PathBuf> {
    let filename = match path.file_name() {
        Some(name) => name,
        None => {
            return None;
        }
    };
    
    let entries = walk_ignoring(base_dir, ignore);
    for entry in entries {
        if entry.file_name() == filename {
            return Some(relative_to(entry.path(), base_dir).to_path_buf());
        }
    }
    None
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

///Return the given path `path`, but expressed relative to path `relative_to`.
///If `relative_to` is not a prefix of `path`, then a copy of `path` is returned.
///
///This function also returns whether this was the case or not.
///This could be useful to determine whether the extracted piece has the be joined to `relative_to` again later.
pub fn relative_to_with_info<T: AsRef<Path>, U: AsRef<Path>>(
    path: T,
    relative_to: U,
) -> (PathBuf, bool) {
    let mut has_prefix = true;
    let path_ref = path.as_ref();
    let internal_path = path_ref
        .strip_prefix(relative_to.as_ref())
        .unwrap_or_else(|_| {
            has_prefix = false;
            return path_ref;
        });
    (internal_path.to_owned(), has_prefix)
}

///Return the given path `path`, but expressed relative to path `relative_to`.
///If `relative_to` is not a prefix of `path`, then a copy of `path` is returned.
pub fn relative_to<T: AsRef<Path>, U: AsRef<Path>>(path: T, relative_to: U) -> PathBuf {
    let path_ref = path.as_ref();
    path_ref
        .strip_prefix(relative_to.as_ref())
        .unwrap_or_else(|_| path_ref)
        .to_owned()
}

pub fn walk_ignoring<'a>(dir: &Path, ignoring: &'a Vec<PathBuf>) -> impl Iterator<Item=DirEntry> + 'a {
    WalkDir::new(dir)
        .into_iter()
        .filter_entry(|entry| {
            let result = !ignoring.iter().any(|ignore_dir| entry.path() == ignore_dir);
            //if !result {
            //    info!("Ignoring {:?}", entry);
            //}
            return result;
        })
        .filter_map(Result::ok)
}

pub fn get_all_notes_exclude<'a>(
    path: &Path,
    ignore: &'a Vec<PathBuf>,
) -> impl Iterator<Item = io::Result<PathBuf>> + 'a {
    let entries = walk_ignoring(path, ignore);

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
pub fn slugify_path<'a>(
    path: &'a Path,
    extension: Option<&str>,
) -> Result<PathBuf, NotePathError<&'a Path>> {
    let ext = match extension {
        Some(e) => Some(OsStr::new(e)),
        None => path.extension(),
    };

    let mut output_path = PathBuf::new();

    for component in path.with_extension("").components() {
        match component {
            std::path::Component::Normal(os_str) => {
                output_path.push(slugify!(&os_str.to_string_lossy().as_ref()));
            }, 
            std::path::Component::RootDir => {
                output_path.push("/");
            },
            _ => {} // Ignore other components like Prefix, etc.
        }
    }

    if let Some(ext) = ext {
        output_path = output_path.with_extension(ext);
    }
    Ok(output_path)
}

/// Copy a directory from `src` to `dst`
pub fn copy_directory<U: AsRef<Path>, T: AsRef<Path>>(src: U, dest: T) -> io::Result<()> {
    copy_directory_aux(src.as_ref(), dest.as_ref())
}

/// Return true if the modification date of `candidate` occurs before that of `reference`.
///
/// If either `candidate` or `reference` does not exist, an error is returned.
pub fn is_older<'a>(candidate: &'a Path, reference: &'a Path) -> Result<bool, GetAgeError<&'a Path>> {
    if !candidate.is_file() {
        return Err(GetAgeError::MissingFileError(candidate));
    }
    if !reference.is_file() {
        return Err(GetAgeError::MissingFileError(reference));
    }
    let candidate_time = get_modification_time(candidate)?;
    let reference_time = get_modification_time(reference)?;
    Ok(candidate_time <= reference_time)
}

/// Get the time that the given path was last modified.
pub fn get_modification_time(path: &Path) -> Result<SystemTime, std::io::Error> {
    let metadata = std::fs::metadata(path)?;
    metadata.modified()
}

///Write the given string to a file at the given path.
pub fn write_to_file(path: &Path, content: &str) {
    if let Some(parent_dir) = path.parent() {
        create_dir_if_not_exists(parent_dir).expect("Failed to create parent dir for search index.");
    }
    let file = File::create(&path).expect("Could not create file!");
    let mut writer = std::io::BufWriter::new(file);
    writer
        .write_all(content.as_bytes())
        .expect("Could not write content to file");
}

///Recursively copy directory `src` to `dest`.
fn copy_directory_aux(src: &Path, dest: &Path) -> io::Result<()> {
    // Create the destination directory if it doesn't exist
    create_dir_if_not_exists(dest)?;

    // Iterate through the entries in the source directory
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let entry_type = entry.file_type()?;

        let entry_path = entry.path();
        let dest_path = dest.join(entry.path().file_name().unwrap());

        if entry_type.is_dir() {
            // Recursively copy subdirectories
            copy_directory_aux(&entry_path, &dest_path)?;
        } else {
            // Copy files
            fs::copy(&entry_path, &dest_path)?;
        }
    }

    Ok(())
}

/// Unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{DirBuilder, File};
    use tempfile::tempdir;

    struct TestCase {
        dir: tempfile::TempDir,
        valid_files: Vec<PathBuf>,
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
        TestCase { dir, valid_files }
    }

    /// Test case with a leveled directory structure
    fn create_nested_test_case() -> TestCase {
        let dir = tempdir().expect("Failed to create temporary directory");
        let dir_path = dir.path();

        let subdir_path = dir_path.join("notes");
        let dir_builder = DirBuilder::new();
        let _subdir = dir_builder
            .create(&subdir_path)
            .expect("Failed to create subdir");

        let md_file_path = dir_path.join("sample.md");
        let html_file_path = dir_path.join("sample.html");
        let md_file_subdir_path = subdir_path.join("sample2.md");

        // Create sample files in the temporary directory
        File::create(&md_file_path).expect("Failed to create .md file");
        File::create(&html_file_path).expect("Failed to create .html file");
        File::create(&md_file_subdir_path).expect("Failed to create nested .md file");

        let valid_files = vec![md_file_path, html_file_path, md_file_subdir_path];
        TestCase { dir, valid_files }
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
