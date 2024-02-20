use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GetAgeError<T: fmt::Debug> {
    #[error("Candidate file {0:?} does not exist")]
    MissingFileError(T),
    #[error(transparent)]
    ModificationTimeError(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum FileWriteError {
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum NotePathError<T: fmt::Debug> {
    #[error("The given path to a Note {0:?} has an empty stem!")]
    NoStem(T),
    #[error("The given path to a Note {0:?} cannot be represented as valid UTF-8!")]
    InvalidUTF8(T),
}
