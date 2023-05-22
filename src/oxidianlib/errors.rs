use core::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("Output directory `{0:?}` already exists.")]
pub struct DirExistsError<T: fmt::Debug>(pub T);

#[derive(Error, Debug)]
#[error("Missing an index file `{0:?}`.")]
pub struct MissingIndexError<T: fmt::Debug>(pub T);

#[derive(Error, Debug)]
#[error("The directory `{0:?}` does not exist.")]
pub struct MissingDirectoryError<T: fmt::Debug>(pub T);


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
pub enum IndexError {
    #[error("Could not open the index file.")]
    IndexOpenError,
    #[error("Could not read index file.")]
    IndexReadError
}

#[derive(Error, Debug)]
pub enum InvalidObsidianLink<T: fmt::Debug, U: fmt::Debug> {
    #[error("Could not parse the given Obsidian-style link: {0:?}")]
    ParseError(T),
    #[error("Did not find match group {group:?} in link {link:?}.")]
    MissingMatchGroup{link: T, group: U}
}
