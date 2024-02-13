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
    pub parent: PathBuf,
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
            || group.to_lowercase() == ".git"
            || ResolvedTendril::is_path(&group) {
            return Err(InvalidTendrilError::InvalidGroup);
        }
        if name.is_empty() || ResolvedTendril::is_path(&name) {
            return Err(InvalidTendrilError::InvalidName);
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
       self.parent.join(&self.name)
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
