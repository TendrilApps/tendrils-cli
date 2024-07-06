use serde::Deserialize;

/// Indicates the tendril action to be performed.
#[derive(Clone, Debug, Eq, PartialEq)]
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

    /// Perform all outward bound actions (link & push)
    Out,
}

/// Indicates an error while initializing a new
/// *Tendrils* folder.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InitError {
    /// A general file system error
    IoError { kind: std::io::ErrorKind },

    /// The folder to initialize is already a
    /// *Tendrils* folder
    AlreadyInitialized,

    /// The folder to initialize is not empty.
    NotEmpty,
}

impl From<std::io::Error> for InitError {
    fn from(err: std::io::Error) -> Self {
        InitError::IoError { kind: err.kind() }
    }
}

/// Indicates an error while reading/parsing a
/// configuration file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GetConfigError {
    /// A general file system error while reading the
    /// file.
    IoError { kind: std::io::ErrorKind },

    /// An error while parsing the json from the file.
    ParseError(String),
}

impl From<std::io::Error> for GetConfigError {
    fn from(err: std::io::Error) -> Self {
        GetConfigError::IoError { kind: err.kind() }
    }
}

impl From<serde_json::Error> for GetConfigError {
    fn from(err: serde_json::Error) -> Self {
        GetConfigError::ParseError(err.to_string())
    }
}

/// Indicates a successful tendril action.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TendrilActionSuccess {
    // To keep the memory size of this enum to a minimum, the new and
    // overwrite variations are separated as their own invariants. If
    // needed in the future this could become a nested enum, although this
    // would increase the memory size
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

impl ToString for TendrilActionSuccess {
    fn to_string(&self) -> String {
        match self {
            TendrilActionSuccess::New => String::from("Created"),
            TendrilActionSuccess::NewSkipped => String::from("Skipped creation"),
            TendrilActionSuccess::Overwrite => String::from("Overwritten"),
            TendrilActionSuccess::OverwriteSkipped => {
                String::from("Skipped overwrite")
            }
        }
    }
}

/// Indicates an unsuccessful tendril action.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TendrilActionError {
    /// General file system errors
    /// `loc` indicates which side of the action had the unexpected type,
    /// as indicated by `mistype`
    IoError {
        /// The type of error that occured
        kind: std::io::ErrorKind,
        /// Where the error occured
        loc: Location,
    },

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
    TypeMismatch {
        /// The unexpected type that was found
        mistype: FsoType,
        /// Where the unexpected type was found
        loc: Location,
    },
}

impl From<std::io::Error> for TendrilActionError {
    fn from(err: std::io::Error) -> Self {
        TendrilActionError::IoError { kind: err.kind(), loc: Location::Unknown }
    }
}

impl From<std::io::ErrorKind> for TendrilActionError {
    fn from(e_kind: std::io::ErrorKind) -> Self {
        TendrilActionError::IoError { kind: e_kind, loc: Location::Unknown }
    }
}

impl ToString for TendrilActionError {
    fn to_string(&self) -> String {
        use std::io::ErrorKind::NotFound;
        use FsoType::{Dir, File, SymDir, SymFile};
        use Location::{Dest, Source, Unknown};
        match self {
            TendrilActionError::IoError { kind: NotFound, loc: Source } => {
                String::from("Source not found")
            }
            TendrilActionError::IoError { kind: NotFound, loc: Dest } => {
                String::from("Destination not found")
            }
            TendrilActionError::IoError { kind: NotFound, loc: Unknown } => {
                String::from("Not found")
            }
            TendrilActionError::IoError { kind: e_kind, loc: Source } => {
                format!("{:?} error at source", e_kind)
            }
            TendrilActionError::IoError { kind: e_kind, loc: Dest } => {
                format!("{:?} error at destination", e_kind)
            }
            TendrilActionError::IoError { kind: e_kind, loc: Unknown } => {
                format!("{:?} error", e_kind)
            }
            TendrilActionError::ModeMismatch => {
                String::from("Wrong tendril style")
            }
            TendrilActionError::Recursion => String::from("Recursive tendril"),
            TendrilActionError::TypeMismatch { loc: Source, mistype: File } => {
                String::from("Unexpected file at source")
            }
            TendrilActionError::TypeMismatch { loc: Source, mistype: Dir } => {
                String::from("Unexpected directory at source")
            }
            TendrilActionError::TypeMismatch {
                loc: Source,
                mistype: SymFile | SymDir,
            } => String::from("Unexpected symlink at source"),
            TendrilActionError::TypeMismatch { loc: Dest, mistype: File } => {
                String::from("Unexpected file at destination")
            }
            TendrilActionError::TypeMismatch { loc: Dest, mistype: Dir } => {
                String::from("Unexpected directory at destination")
            }
            TendrilActionError::TypeMismatch {
                loc: Dest,
                mistype: SymFile | SymDir,
            } => String::from("Unexpected symlink at destination"),
            TendrilActionError::TypeMismatch { loc: Unknown, mistype: _ } => {
                String::from("Unexpected file system object")
            }
        }
    }
}

/// Indicates a side of a file system transaction
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Location {
    Source,
    Dest,
    Unknown,
}

/// Indicates a type of file system object
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FsoType {
    /// A standard file
    File,
    /// A standard directory
    Dir,
    /// A symlink to a file
    SymFile,
    /// A symlink to a directory
    SymDir,
}

/// Indicates the behaviour of this tendril, and determines whether it is
/// a push/pull style, or a link style tendril.
#[derive(Clone, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
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
            _ => OneOrMany::Vec(from),
        }
    }
}
