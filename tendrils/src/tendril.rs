use crate::enums::{InvalidTendrilError, TendrilMode};
use std::path::PathBuf;

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
            || Tendril::is_path(&group)
            || group.to_lowercase() == "tendrils.json"
            || group.to_lowercase() == ".git"
            || group.contains('\n')
            || group.contains('\r') {
            return Err(InvalidTendrilError::InvalidGroup);
        }

        if name.is_empty()
            || Tendril::is_path(&name)
            || name.contains('\n')
            || name.contains('\r') {
            return Err(InvalidTendrilError::InvalidName);
        }

        let parent_str = parent.to_string_lossy();
        if parent_str.is_empty()
            || parent_str.contains('\n')
            || parent_str.contains('\r') {
            return Err(InvalidTendrilError::InvalidParent);
        }

        Ok(Tendril {
            group,
            name,
            parent,
            mode,
        })
    }

    pub fn group(&self) -> &str {
        &self.group
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn full_path(&self) -> PathBuf {
        #[cfg(not(windows))]
        let platform_dir_sep = "/";
        #[cfg(windows)]
        let platform_dir_sep = "\\";

        let parent_str = String::from(self.parent.to_string_lossy());

        if parent_str.ends_with('\\')
            || parent_str.ends_with('/')
            || parent_str.is_empty() {
            PathBuf::from(parent_str + &self.name)
        }
        else if parent_str.contains('\\') {
            if parent_str.contains('/') {
                // Mixed separators - fall back to the
                // platform's default separator
                PathBuf::from(parent_str + platform_dir_sep + &self.name)
            }
            else {
                PathBuf::from(parent_str + "\\" + &self.name)
            }
        }
        else if parent_str.contains("/") {
            PathBuf::from(parent_str + "/" + &self.name)
        }
        else {
            // No separators - fall back to the
            // platform's default separator
            PathBuf::from(parent_str + platform_dir_sep + &self.name)
        }
    }

    fn is_path(x: &str) -> bool {
        x.contains('/') || x.contains('\\')
    }
}
