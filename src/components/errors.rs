use thiserror::Error;
use std::fmt;

#[derive(Error, Debug)]
pub enum InvalidObsidianLink<T: fmt::Debug, U: fmt::Debug> {
    #[error("Could not parse the given Obsidian-style link: {0:?}")]
    ParseError(T),
    #[error("Did not find match group {group:?} in link {link:?}.")]
    MissingMatchGroup{link: T, group: U}
}

