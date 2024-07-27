use crate::enums::{InvalidTendrilError, TendrilMode};
use std::path::{Path, PathBuf};

/// A Tendril that is prepared for use with Tendril actions
/// and always exists in a valid state.
/// Note: This does *not* guarantee that the path
/// exists or is valid.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tendril<'a> {
    group: &'a str,
    name: &'a str,
    parent: PathBuf,
    pub mode: TendrilMode,
}

impl<'a> Tendril<'a> {
    pub fn new(
        group: &'a str,
        name: &'a str,
        parent: PathBuf,
        mode: TendrilMode,
    ) -> Result<Tendril<'a>, InvalidTendrilError> {
        if group.is_empty()
            || Tendril::is_path(group)
            || group.to_lowercase() == ".tendrils"
            || group.to_lowercase() == ".git"
            || group.contains('\n')
            || group.contains('\r')
        {
            return Err(InvalidTendrilError::InvalidGroup);
        }

        if name.is_empty() || name.contains('\n') || name.contains('\r') {
            return Err(InvalidTendrilError::InvalidName);
        }

        let parent_str = parent.to_string_lossy();
        if parent_str.is_empty()
            || parent_str.contains('\n')
            || parent_str.contains('\r')
        {
            return Err(InvalidTendrilError::InvalidParent);
        }

        Ok(Tendril { group, name, parent, mode })
    }

    pub fn group(&self) -> &str {
        self.group
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn parent(&self) -> &Path {
        &self.parent
    }

    pub fn full_path(&self) -> PathBuf {
        use std::path::{MAIN_SEPARATOR, MAIN_SEPARATOR_STR};

        let mut full_path_str = String::from(self.parent.to_string_lossy());
        if !full_path_str.ends_with('/') && !full_path_str.ends_with('\\') {
            full_path_str.push(MAIN_SEPARATOR);
        }

        if self.name.starts_with('/') || self.name.starts_with('\\') {
            full_path_str.push_str(&self.name[1..]);
        }
        else {
            full_path_str.push_str(self.name);
        }

        #[cfg(windows)]
        return PathBuf::from(full_path_str.replace('/', MAIN_SEPARATOR_STR));

        #[cfg(not(windows))]
        return PathBuf::from(full_path_str.replace('\\', MAIN_SEPARATOR_STR));
    }

    fn is_path(x: &str) -> bool {
        x.contains('/') || x.contains('\\')
    }
}
