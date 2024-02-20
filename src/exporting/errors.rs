use thiserror::Error;
use crate::utils::errors::FileWriteError;

#[derive(Error, Debug)]
pub enum PreambleError {
    #[error(transparent)]
    PreambleReadError(#[from] std::io::Error),
    #[error(transparent)]
    FileWriteError(#[from] FileWriteError),
}
