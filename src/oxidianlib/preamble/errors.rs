use core::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SyntaxError<T: fmt::Debug, U: fmt::Debug> {
    #[error("The declaration of a new math operator does not accept any arguments.")]
    NoArguments,
    #[error("Expected `{0:?}`, but got `{1:?}`.")]
    UnexpectedToken(T, U),
    #[error("Cannot `{0:?}` to a TexCommand")]
    InvalidCommand(T),
    #[error("Cannot parse number of arguments `{0:?}`")]
    InvalidNumber(T),
    #[error("The file ended before finishing command parsing")]
    PrematureEnd,
}
