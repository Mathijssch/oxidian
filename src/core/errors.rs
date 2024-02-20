use thiserror::Error;
use std::fmt;

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
pub enum MathFindError<T: fmt::Debug> {
    #[error("Already in math mode. Cannot open a new math environment")]
    NestedMathMode(T),
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
