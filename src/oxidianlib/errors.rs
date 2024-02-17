use core::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("Output directory `{0:?}` already exists.")]
pub struct DirExistsError<T: fmt::Debug>(pub T);

#[derive(Error, Debug)]
#[error("Missing an index file named `{0:?}` in `{1:?}`.")]
pub struct MissingIndexError<T: fmt::Debug>(pub T, pub T);

#[derive(Error, Debug)]
#[error("The directory `{0:?}` does not exist.")]
pub struct MissingDirectoryError<T: fmt::Debug>(pub T);

#[derive(Error, Debug)]
pub enum GetAgeError<T: fmt::Debug> {
    #[error("Candidate file {0:?} does not exist")]
    MissingFileError(T),
    #[error(transparent)]
    ModificationTimeError(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum InitializationError<T: fmt::Debug> {
    #[error(transparent)]
    OutputDirExists(#[from] DirExistsError<T>),
    #[error(transparent)]
    MissingDirectory(#[from] MissingDirectoryError<T>),
    #[error(transparent)]
    MissingIndexError(#[from] MissingIndexError<T>),
}

#[derive(Error, Debug)]
pub enum FileWriteError {
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}


#[derive(Error, Debug)]
pub enum IndexError {
    #[error("Could not open the index file.")]
    IndexOpenError,
    #[error("Could not read index file.")]
    IndexReadError
}

#[derive(Error, Debug)]
pub enum PreambleError {
    #[error(transparent)]
    PreambleReadError(#[from] std::io::Error),
    #[error(transparent)]
    FileWriteError(#[from] FileWriteError),
}

#[derive(Error, Debug)]
pub enum InvalidObsidianLink<T: fmt::Debug, U: fmt::Debug> {
    #[error("Could not parse the given Obsidian-style link: {0:?}")]
    ParseError(T),
    #[error("Did not find match group {group:?} in link {link:?}.")]
    MissingMatchGroup{link: T, group: U}
}

#[derive(Error, Debug)]
pub enum InvalidMarkdownLink<T: fmt::Debug, U: fmt::Debug> {
    #[error("Could not parse the given Markdown-style link: {0:?}")]
    ParseError(T),
    #[error("Did not find match group {group:?} in link {link:?}.")]
    MissingMatchGroup{link: T, group: U}
}

#[derive(Error, Debug)]
pub enum MathFindError<T: fmt::Debug> {
    #[error("Already in math mode. Cannot open a new math environment")]
    NestedMathMode(T),
}

#[derive(Error, Debug)]
pub enum NotePathError<T: fmt::Debug> {
    #[error("The given path to a Note {0:?} has an empty stem!")]
    NoStem(T), 
    #[error("The given path to a Note {0:?} cannot be represented as valid UTF-8!")]
    InvalidUTF8(T)
}


#[derive(Error, Debug)]
pub enum ReadConfigError<T: fmt::Debug> {
    #[error("Error reading config file at path {0:?}.")]
    NoSuchFile(T), 
    #[error("Error reading config file to string")]
    ReadToString,
    #[error("Invalid toml file in path {0:?}. Could not load file into configuration object.")]
    InvalidToml(T),
}
