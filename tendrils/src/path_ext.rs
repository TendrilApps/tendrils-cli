use crate::enums::FsoType;
use crate::env_ext::get_home_dir;
use std::path::{Path, PathBuf};

#[cfg(test)]
mod tests;

pub(crate) trait PathExt {
    /// Returns the type of the file system object that
    /// the path points to, or returns `None` if the FSO
    /// does not exist.
    fn get_type(&self) -> Option<FsoType>;

    /// Replaces all directory separators with those of the current platform
    /// (i.e. `\\` on Windows and `/` on all others).
    fn replace_dir_seps(&self) -> PathBuf;
}

impl PathExt for Path {
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

    fn replace_dir_seps(&self) -> PathBuf {
        use std::path::MAIN_SEPARATOR;
        #[cfg(windows)]
        let sep_to_remove = '/' as u8;

        #[cfg(not(windows))]
        let sep_to_remove = '\\' as u8;

        let mut bytes = Vec::from(self.as_os_str().as_encoded_bytes());

        for b in bytes.iter_mut() {
            if *b == sep_to_remove {
                *b = MAIN_SEPARATOR as u8;
            }
        }

        // All bytes were originally from an OsString, or are the known path
        // separators so this call is safe.
        bytes_to_os_string(bytes).into()
    }
}

fn bytes_to_os_string(bytes: Vec<u8>) -> std::ffi::OsString {
    unsafe {
        std::ffi::OsString::from_encoded_bytes_unchecked(bytes)
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
