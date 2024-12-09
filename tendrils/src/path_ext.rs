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
    /// with, or `path` starts with a directory separator, one is added. On
    /// Unix, only `/` is considered a directory separator, but on Windows both
    /// `/` and `\` are.
    fn join_raw(&self, path: &Path) -> PathBuf;

    /// Returns the type of the file system object that
    /// the path points to, or returns `None` if the FSO
    /// does not exist.
    fn get_type(&self) -> Option<FsoType>;

    #[cfg(windows)]
    /// Replaces all forward slashes (`/`) with backslashes (`\`).
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

    /// Converts a non-rooted path to rooted by prepending it with the given
    /// `root`. If the given `root` is not rooted either, then the default root
    /// of `/` on Unix or `\` on Windows is used. Returns `self` if the path is
    /// already rooted. What counts as rooted varies by platform - for example
    /// `C:\Path` and `\\MyServer\Share\Path` are rooted on Windows but not on
    /// Unix. This function does *not* take the current directory into account.
    fn root(&self, root: &Path) -> PathBuf;
}

impl PathExt for Path {
    fn join_raw(&self, path: &Path) -> PathBuf {
        let parent_bytes = self.as_os_str().as_encoded_bytes();
        let child_bytes = path.as_os_str().as_encoded_bytes();
        let mut raw_str = std::ffi::OsString::from(&self);

        #[cfg(not(windows))]
        if parent_bytes.ends_with(&['/' as u8])
            || child_bytes.starts_with(&['/' as u8]) {
            raw_str.push(path);
        }
        else {
            raw_str.push(std::path::MAIN_SEPARATOR_STR);
            raw_str.push(path);
        }

        #[cfg(windows)]
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
        else if self.is_symlink() {
            Some(FsoType::BrokenSym)
        }
        else {
            None
        }
    }

    #[cfg(windows)]
    fn replace_dir_seps(&self) -> PathBuf {
        let mut bytes = Vec::from(self.as_os_str().as_encoded_bytes());

        for b in bytes.iter_mut() {
            if *b == '/' as u8 {
                *b = std::path::MAIN_SEPARATOR as u8;
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

    fn root(&self, root: &Path) -> PathBuf {
        if self.has_root() {
            PathBuf::from(self)
        }
        else if root.has_root() {
            root.join_raw(self)
        }
        else {
            Path::new(MAIN_SEPARATOR_STR).join_raw(self)
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

/// A [`PathBuf`] wrapper that guarantees that the path has been resolved in
/// this particular order:
///     1. Any environment variables have been resolved
///     2. A leading tilde has been resolved
///     3. A non-rooted path has been rooted. The default conversion to rooted
/// occurs by prepending `/` on Unix or `\` on Windows. A different root can be
/// provided by constructing the [`UniPath`] using [`UniPath::new_with_root`].
///     4. Unix style path separators (`/`) have been replaced with `\` (Windows
/// only)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UniPath(PathBuf);

impl UniPath {
    /// Converts the given `path` to a [`UniPath`] using the standard rules, but
    /// converts relative paths to absolute by appending them to the given
    /// `root`. If the given `root` is not absolute either, it will default to
    /// using `/` on Unix and `\` on Windows.
    pub fn new_with_root(path: &Path, root: &Path) -> Self {
        #[cfg(windows)]
        return UniPath(
            path
                .resolve_env_variables()
                .resolve_tilde()
                .root(root)
                .replace_dir_seps()
        );

        #[cfg(not(windows))]
        return UniPath(
            path
                .resolve_env_variables()
                .resolve_tilde()
                .root(root)
        );
    }

    /// The wrapped [`PathBuf`] that has been sanitized.
    pub fn inner(&self) -> &Path {
        &self.0
    }
}

impl From<&Path> for UniPath {
    fn from(value: &Path) -> Self {
        Self::new_with_root(value, &Path::new(MAIN_SEPARATOR_STR))
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

impl AsRef<UniPath> for UniPath {
    fn as_ref(&self) -> &UniPath {
        &self
    }
}
