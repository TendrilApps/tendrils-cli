use crate::enums::InvalidTendrilError;
use std::path::PathBuf;

/// A Tendril that is prepared for use with Tendril operations
/// and always exists in a valid state.
/// Note: This does *not* guarantee that the path
/// exists or is valid.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedTendril {
    group: String,
    name: String,
    parent: PathBuf,
    pub mode: TendrilMode,
}

impl ResolvedTendril {
    pub fn new(
        group: String,
        name: String,
        parent: PathBuf,
        mode: TendrilMode,
    ) -> Result<ResolvedTendril, InvalidTendrilError> {
        if group.is_empty()
            || ResolvedTendril::is_path(&group)
            || group.to_lowercase() == ".git"
            || group.contains('\n')
            || group.contains('\r') {
            return Err(InvalidTendrilError::InvalidGroup);
        }

        if name.is_empty()
            || ResolvedTendril::is_path(&name)
            || name.contains('\n')
            || name.contains('\r') {
            return Err(InvalidTendrilError::InvalidName);
        }

        let parent_str = parent.to_string_lossy();
        if parent_str.contains('\n') || parent_str.contains('\r') {
            return Err(InvalidTendrilError::InvalidParent);
        }

        Ok(ResolvedTendril {
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

        let parent_str = self.parent
            .to_string_lossy()
            .to_string();

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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TendrilMode {
    DirMerge,
    DirOverwrite,
    Link,
}
