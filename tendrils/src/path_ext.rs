use crate::enums::FsoType;
use crate::env_ext::get_home_dir;
use std::ffi::OsString;
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

    /// Replaces the first instance of `~` with the `HOME` variable
    /// and returns the replaced string. If `HOME` doesn't exist,
    /// `HOMEDRIVE` and `HOMEPATH` will be combined provided they both exist,
    /// otherwise it returns `self`.
    fn resolve_tilde(&self) -> PathBuf;
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

    fn resolve_tilde(&self) -> PathBuf {
        let path_bytes = self.as_os_str().as_encoded_bytes();

        if path_bytes == &['~' as u8]
            || path_bytes.starts_with(&['~' as u8, '/' as u8])
            || path_bytes.starts_with(&['~' as u8, '\\' as u8]) {
            // Continue
        }
        else {
            return PathBuf::from(self);
        }

        match get_home_dir() {
            Some(mut v) => {
                let trimmed_str;
                unsafe {
                    // All bytes were originally from an OsString so this call
                    // is safe.
                    trimmed_str = OsString::from_encoded_bytes_unchecked(
                        path_bytes[1..].to_vec()
                    );
                }

                v.push(trimmed_str);
                PathBuf::from(v)
            }
            None => PathBuf::from(self),
        }
    }
}

fn bytes_to_os_string(bytes: Vec<u8>) -> std::ffi::OsString {
    unsafe {
        std::ffi::OsString::from_encoded_bytes_unchecked(bytes)
    }
}
