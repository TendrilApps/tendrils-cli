use serde::Deserialize;

/// Indicates the tendril action to be performed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

/// Indicates an error while initializing a new
/// *Tendrils* folder.
#[derive(Debug)]
pub enum InitError {
    /// A general file system error
    IoError(std::io::Error),

    /// The folder to initialize is already a
    /// *Tendrils* folder
    AlreadyInitialized,

    /// The folder to initialize is not empty.
    NotEmpty,
}

impl From<std::io::Error> for InitError {
    fn from(err: std::io::Error) -> Self {
        InitError::IoError(err)
    }
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
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TendrilActionSuccess {
    // To keep the memory size of this enum to a minimum, the new and overwrite
    // variations are separated as their own invariants. If needed in the
    // future this could become a nested enum, although this would
    // increase the memory size

    /// A successful action that created a new file system object at the
    /// destination.
    New,

    /// A successful action that overwrote a file system object at the
    /// destination.
    Overwrite,

    /// An action that was expected to succeed in creating a new file system
    /// object at the destination but was skipped due to a dry-run.
    NewSkipped,

    /// An action that was expected to succeed in overwriting a file system
    /// object at the destination but was skipped due to a dry-run.
    OverwriteSkipped,
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

/// Indicates the behaviour of this tendril, and determines whether it is
/// a push/pull style, or a link style tendril.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TendrilMode {
    /// Overwrite any files/folders that are present in both the source and
    /// destination, but keep anything in the destination folder that is not
    /// in the source folder. This only applies to folder tendrils.
    /// Tendrils with this mode are considered push/pull.
    DirMerge,

    /// Completely overwrite the destination folder with the contents of
    /// the source folder. This only applies to folder tendrils.
    /// Tendrils with this mode are considered push/pull.
    DirOverwrite,

    /// Create a symlink at the remote location that points to local
    /// file/folder.
    Link,
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
