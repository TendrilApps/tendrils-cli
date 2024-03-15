use serde::Deserialize;

/// Indicates the tendril action to be performed.
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum ActionMode {
    /// Copy tendrils from the *Tendrils* folder to their various locations
    /// on the computer.
    Push,
    /// Copy tendrils from their various locations on the computer to the
    /// *Tendrils* folder.
    Pull,
    /// Create symlinks at the various locations on the computer to the
    /// tendrils in the *Tendrils* folder.
    Link,
}

/// Indicates an error while reading/parsing the
/// tendrils from file.
#[derive(Debug)]
pub enum GetTendrilsError {
    /// A general file system error while reading the
    /// `tendrils.json` file.
    IoError(std::io::Error),
    /// An error while parsing the json from the file.
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

/// Indicates a successful tendril action.
#[derive(Debug, Eq, PartialEq)]
pub enum TendrilActionSuccess {
    /// A successful action
    Ok,
    /// An action that is expected to succeed but was skipped due to a dry-run
    Skipped,
}

/// Indicates an unsuccessful tendril action.
#[derive(Debug)]
pub enum TendrilActionError {
    /// General file system errors
    IoError(std::io::Error),
    /// The tendril mode does not match the attempted action, such as:
    /// - Attempting to pull a link tendril
    /// - Attempting to link a push/pull tendril
    ModeMismatch,
    /// The tendril action would result in recursive copying/linking, such as:
    /// - Including the *Tendrils* folder as a tendril
    /// - A folder tendril that is an ancestor to the *Tendrils* folder
    /// - A tendril that is inside the *Tendrils* folder
    Recursion,
    /// The type of the remote and local file system objects do not match, or
    /// do not match the expected types, such as:
    /// - The source is a file but the destination is a folder
    /// - The local or remote are symlinks (during a push/pull action)
    /// - The remote is *not* a symlink (during a link action)
    TypeMismatch,
}

impl From<std::io::Error> for TendrilActionError {
    fn from(err: std::io::Error) -> Self {
        TendrilActionError::IoError(err)
    }
}

/// Indicates an invalid tendril field.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InvalidTendrilError {
    InvalidGroup,
    InvalidName,
    InvalidParent,
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
