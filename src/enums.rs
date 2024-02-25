use serde::Deserialize;

#[derive(Debug)]
pub enum GetTendrilsError {
    IoError(std::io::Error),
    ParseError(serde_json::Error),
}

impl From<std::io::Error> for GetTendrilsError {
    fn from(err: std::io::Error) -> Self {
        GetTendrilsError::IoError(err)
    }
}

impl From<serde_json::Error> for GetTendrilsError {
    fn from(err: serde_json::Error) -> Self {
        GetTendrilsError::ParseError(err)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum TendrilActionSuccess {
    Ok,
    /// Occurs when a command is executed as a dry-run
    Skipped,
}

#[derive(Debug)]
pub enum TendrilActionError {
    IoError(std::io::Error),
    /// Occurs when a tendril action does not match its
    /// mode (such as trying to pull a link tendril)
    ModeMismatch,
    InvalidTendrilError(InvalidTendrilError),
    Recursion,
    TypeMismatch,
}

impl From<InvalidTendrilError> for TendrilActionError {
    fn from(err: InvalidTendrilError) -> Self {
        TendrilActionError::InvalidTendrilError(err)
    }
}

impl From<std::io::Error> for TendrilActionError {
    fn from(err: std::io::Error) -> Self {
        TendrilActionError::IoError(err)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InvalidTendrilError {
    InvalidGroup,
    InvalidName,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum OneOrMany<T> {
    // https://github.com/Mingun/ksc-rs/blob/8532f701e660b07b6d2c74963fdc0490be4fae4b/src/parser.rs#L29pub
    /// Single value
    One(T),
    /// Array of values
    Vec(Vec<T>),
}

impl<T> From<OneOrMany<T>> for Vec<T> {
    fn from(from: OneOrMany<T>) -> Self {
        match from {
            OneOrMany::One(val) => vec![val],
            OneOrMany::Vec(vec) => vec,
        }
    }
}

impl<T> From<Vec<T>> for OneOrMany<T> {
    fn from(mut from: Vec<T>) -> Self {
        match from.len() {
            1 => OneOrMany::One(from.remove(0)),
            _ => OneOrMany::Vec(from)
        }
    }
}
