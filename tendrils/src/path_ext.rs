use crate::enums::FsoType;
use crate::env_ext::get_home_dir;
use std::path::Path;

pub(crate) trait Fso {
    /// Returns the type of the file system object that
    /// the path points to, or returns `None` if the FSO
    /// does not exist.
    fn get_type(&self) -> Option<FsoType>;
}

impl Fso for Path {
    fn get_type(&self) -> Option<FsoType> {
        if self.is_file() {
            if self.is_symlink() {
                Some(FsoType::SymFile)
            }
            else {
                Some(FsoType::File)
            }
        }
        else if self.is_dir() {
            if self.is_symlink() {
                Some(FsoType::SymDir)
            }
            else {
                Some(FsoType::Dir)
            }
        }
        else {
            None
        }
    }
}

/// Replaces the first instance of `~` with the `HOME` variable
/// and returns the replaced string. If `HOME` doesn't exist,
/// `HOMEDRIVE` and `HOMEPATH` will be combined provided they both exist,
/// otherwise it returns the given string.
///
/// Note: This does *not* check that the tilde is the leading character (it
/// could be anywhere in the string) - this check should be done prior to
/// calling this.
pub(crate) fn resolve_tilde(path: &str) -> String {
    match get_home_dir() {
        Some(v) => path.replacen('~', &v, 1),
        None => String::from(path),
    }
}
