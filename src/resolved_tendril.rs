use crate::errors::InvalidTendrilError;
use std::path::PathBuf;

/// A Tendril that is prepared for use with Tendril operations
/// and always exists in a valid state.
/// Note: This does *not* guarantee that the path
/// exists or is valid.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedTendril {
    app: String,
    name: String,
    pub parent: PathBuf,
    pub mode: TendrilMode,
}

impl ResolvedTendril {
    pub fn new(
        app: String,
        name: String,
        parent: PathBuf,
        mode: TendrilMode,
    ) -> Result<ResolvedTendril, InvalidTendrilError> {
        if app.is_empty()
            || app.to_lowercase() == ".git"
            || ResolvedTendril::is_path(&app) {
            return Err(InvalidTendrilError::InvalidApp);
        }
        if name.is_empty() || ResolvedTendril::is_path(&name) {
            return Err(InvalidTendrilError::InvalidName);
        }

        Ok(ResolvedTendril {
            app,
            name,
            parent,
            mode,
        })
    }

    pub fn app(&self) -> &str {
        &self.app
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
    FolderMerge,
    FolderOverwrite,
    Link,
}
