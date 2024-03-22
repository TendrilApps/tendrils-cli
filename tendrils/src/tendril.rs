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
        use std::path::MAIN_SEPARATOR_STR;

        let parent_str = String::from(self.parent.to_string_lossy());

        PathBuf::from(parent_str
            .replace('\\', MAIN_SEPARATOR_STR)
            .replace('/', MAIN_SEPARATOR_STR)
        ).join(&self.name)
    }

    fn is_path(x: &str) -> bool {
        x.contains('/') || x.contains('\\')
    }
}
