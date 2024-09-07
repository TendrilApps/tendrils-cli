use crate::enums::FsoType;
use crate::env_ext::get_home_dir;
use std::ffi::OsString;
use std::path::{Path, PathBuf, MAIN_SEPARATOR_STR};

#[cfg(test)]
mod tests;

pub(crate) trait PathExt {
    /// Appends the given `path` to `self`, regardless of whether the given
    /// path is absolute or relative. Any directory separators at the
    /// end of `self` or start of `path` are preserved. If neither `self` ends
    /// with, or `path` starts with a directory separator, one is added.
    fn join_raw(&self, path: &Path) -> PathBuf;

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
    /// otherwise it returns `self`. This fallback is mainly a Windows specific
    /// issue, but is supported on all platforms either way.
    fn resolve_tilde(&self) -> PathBuf;

    /// Replaces all environment variables in the format `<varname>` in the
    /// given path with their values. If the variable is not found, the
    /// `<varname>` is left as-is in the path.
    ///
    /// # Limitations
    /// If the path contains the `<pattern>` and the pattern corresponds to
    /// an environment variable, there is no way to escape the brackets
    /// to force it to use the raw path. This should only be an issue
    /// on Unix (as Windows doesn't allow `<` or `>` in paths anyways),
    /// and only when the variable exists (otherwise it uses the raw
    /// path). A work-around is to set the variable value to `<pattern>`.
    /// In the future, an escape character such as `|` could be
    /// implemented, but this added complexity was avoided for now.
    fn resolve_env_variables(&self) -> PathBuf;

    /// Converts a relative path to absolute by simply prepending a directory
    /// separator. This does *not* take into account the current directory.
    /// Returns `self` if the path is already absolute. What counts as an
    /// absolute path varies by platform - for example `C:\Path` and `\Path`
    /// are absolute on Windows but not on Unix. On Unix, these would be
    /// converted to `/C:\Path` and `/\Path`. Although not technically an
    /// absolute path, paths starting with / or \ on Windows are considered
    /// absolute here (as they are absolute within the current drive). This
    /// blind conversion may cause unintuitive behaviour in paths with relative
    /// Windows prefixes. For example C:SomePath is converted to \C:SomePath
    /// rather than the C:\SomePath that may have been expected.
    fn to_absolute(&self) -> PathBuf;
}

impl PathExt for Path {
    fn join_raw(&self, path: &Path) -> PathBuf {
        let parent_bytes = self.as_os_str().as_encoded_bytes();
        let child_bytes = path.as_os_str().as_encoded_bytes();
        let mut raw_str = std::ffi::OsString::from(&self);

        if parent_bytes.ends_with(&['/' as u8])
            || parent_bytes.ends_with(&['\\' as u8])
            || child_bytes.starts_with(&['/' as u8])
            || child_bytes.starts_with(&['\\' as u8]) {
            raw_str.push(path);
        }
        else {
            raw_str.push(std::path::MAIN_SEPARATOR_STR);
            raw_str.push(path);
        }

        PathBuf::from(raw_str)
    }

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
        const SEP_TO_REMOVE: u8 = '/' as u8;

        #[cfg(not(windows))]
        const SEP_TO_REMOVE: u8 = '\\' as u8;

        let mut bytes = Vec::from(self.as_os_str().as_encoded_bytes());

        for b in bytes.iter_mut() {
            if *b == SEP_TO_REMOVE {
                *b = MAIN_SEPARATOR as u8;
            }
        }

        unsafe {
            // All bytes were originally from an OsString, or are the known path
            // separators so this call is safe.
            OsString::from_encoded_bytes_unchecked(bytes)
        }.into()
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

    fn resolve_env_variables(&self) -> PathBuf {
        let given_bytes = self.as_os_str().as_encoded_bytes();
        let mut search_start_idx = 0;
        let mut resolved_bytes: Vec<u8> = vec![];

        while let Some(next) = next_env_var(given_bytes, search_start_idx) {
            let var_no_brkts = &given_bytes[next.0 + 1..next.1];
            let var_name_no_brkts = unsafe {
                // All bytes were originally from an OsString so this call
                // is safe.
                OsString::from_encoded_bytes_unchecked(var_no_brkts.to_vec())
            };
            if let Some(v) = std::env::var_os(var_name_no_brkts) {
                resolved_bytes.extend(&given_bytes[search_start_idx..next.0]);
                resolved_bytes.extend(v.as_encoded_bytes());
            }
            else {
                resolved_bytes.extend(&given_bytes[search_start_idx..next.1 + 1]);
            }
            search_start_idx = next.1 + 1;
        }

        if search_start_idx == 0 {
            return PathBuf::from(self);
        }
        else {
            resolved_bytes.extend(&given_bytes[search_start_idx..]);

            let resolved_str = unsafe {
                OsString::from_encoded_bytes_unchecked(resolved_bytes)
            };
            PathBuf::from(resolved_str)
        }
    }

    fn to_absolute(&self) -> PathBuf {
        if self.has_root() {
            PathBuf::from(self)
        }
        else {
            PathBuf::from(MAIN_SEPARATOR_STR).join_raw(self)
        }
    }
}

/// Returns the `(start index, end index)` of the next environment variable
/// name, including the surrounding brackets, starting the search from the
/// `search_start_idx`. Returns `None` if no variables remain at or after the
/// start index.
fn next_env_var(bytes: &[u8], search_start_idx: usize) -> Option<(usize, usize)> {
    let mut var_start = 0;
    let mut has_start = false;

    for (i, b) in bytes[search_start_idx..].iter().enumerate() {
        if *b == '<' as u8 {
            var_start = i;
            has_start = true;
        }
        else if *b == '>' as u8 && has_start {
            return Some((search_start_idx + var_start, search_start_idx + i))
        }
    }

    None
}

#[cfg(test)]
pub fn contains_env_var(input: &Path) -> bool {
    next_env_var(input.as_os_str().as_encoded_bytes(), 0).is_some()
}

/// A [`PathBuf`] wrapper that guarantees that any tilde values have been
/// [resolved](PathExt::resolve_tilde), any environment variables have been
/// [resolved](PathExt::resolve_env_variables), Unix style path separators (`/`)
/// have been [replaced](PathExt::replace_dir_seps) with `\` (Windows only),
/// and that the path has been converted [to absolute](PathExt::to_absolute),
/// in that order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UniPath(PathBuf);

impl UniPath {
    /// The wrapped [`PathBuf`] that has been sanitized.
    pub fn inner(&self) -> &Path {
        &self.0
    }
}

impl From<&Path> for UniPath {
    fn from(value: &Path) -> Self {
        #[cfg(windows)]
        return UniPath(
            value
                .resolve_tilde()
                .resolve_env_variables()
                .replace_dir_seps()
                .to_absolute()
        );

        #[cfg(not(windows))]
        return UniPath(
            value
                .resolve_tilde()
                .resolve_env_variables()
                .to_absolute()
        );
    }
}

impl From<&PathBuf> for UniPath {
    fn from(value: &PathBuf) -> Self {
        Self::from(value.as_path())
    }
}

impl From<PathBuf> for UniPath {
    fn from(value: PathBuf) -> Self {
        Self::from(value.as_path())
    }
}
