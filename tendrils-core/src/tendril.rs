use crate::enums::{InvalidTendrilError, TendrilMode};
use crate::path_ext::{PathExt, UniPath};
use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};

#[cfg(test)]
pub(crate) mod tests;

/// A tendril that is prepared for use with tendril actions
/// and always exists in a valid state.
/// Note: This does *not* guarantee that the path
/// exists or is valid.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Tendril {
    /// Path to the local file relative to the root of the Tendrils repo
    local: PathBuf,
    local_abs: PathBuf,
    remote: UniPath,
    pub mode: TendrilMode,
}

impl Tendril {
    fn new(
        td_repo: impl AsRef<UniPath>,
        local: PathBuf,
        remote: UniPath,
        mode: TendrilMode,
    ) -> Result<Tendril, InvalidTendrilError> {
        if local.components().any(|c| c == Component::ParentDir) {
            return Err(InvalidTendrilError::InvalidLocal);
        }

        let mut local_sub_comps = local.components();
        if match local_sub_comps.next() {
            Some(Component::Normal(c))
                if Self::is_forbidden_dir(c) => true,
            Some(Component::RootDir | Component::CurDir) => match local_sub_comps.next() {
                Some(Component::Normal(c)) if Self::is_forbidden_dir(c) => true,
                None => true,
                _ => false,
            }
            None => true,
            _ => false,
        }
        {
            return Err(InvalidTendrilError::InvalidLocal);
        }

        if Self::is_recursive(td_repo.as_ref().inner(), remote.inner()) {
            return Err(InvalidTendrilError::Recursion)
        }

        #[cfg(not(windows))]
        let local_abs = td_repo
            .as_ref()
            .inner()
            .join_raw(&local)
            .into();
        #[cfg(windows)]
        let local_abs = td_repo
            .as_ref()
            .inner()
            .join_raw(&local)
            .replace_dir_seps()
            .into();

        Ok(Tendril { local, local_abs, remote, mode })
    }

    #[cfg(any(test, feature = "_test_utils"))]
    pub fn new_expose(
        td_repo: impl AsRef<UniPath>,
        local: PathBuf,
        remote: UniPath,
        mode: TendrilMode,
    ) -> Result<Tendril, InvalidTendrilError> {
        Tendril::new(td_repo.as_ref(), local, remote, mode)
    }

    /// The absolute path to this file system object inside the Tendrils repo.
    /// The combination of the given Tendrils repo and its `local`.
    pub fn local_abs(&self) -> &Path {
        &self.local_abs
    }

    /// The resolved path to this file system object specific to this machine,
    /// outside of the Tendrils repo.
    pub fn remote(&self) -> &UniPath {
        &self.remote
    }

    fn is_forbidden_dir(path_comp: &OsStr) -> bool {
        match path_comp.to_string_lossy().to_lowercase().trim() {
            ".tendrils" => true,
            #[cfg(windows)] // Trailing dots are ignored on Windows
            ".tendrils." => true,
            "" => true,
            _ => false,
        }
    }

    fn is_recursive(td_repo: &Path, remote: &Path) -> bool {
        td_repo == remote
            || td_repo.ancestors().any(|p| p == remote)
            || remote.ancestors().any(|p| p == td_repo)
    }
}

/// Contains the unresolved, unvalidated information to define a single
/// tendril.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawTendril {
    /// The path to the master file/folder relative to the root of the
    /// Tendrils repo.
    pub local: String,

    /// A list of absolute paths to the various locations on the machine at
    /// which to recreate the [`Self::local`]. Each `local` and
    /// `remote` pair forms a tendril.
    pub remote: String,
    pub mode: TendrilMode,

    /// A list of profiles to which this tendril belongs. If empty,
    /// this tendril is considered to be included in *all* profiles.
    pub profiles: Vec<String>,
}

impl RawTendril {
    #[cfg(any(test, feature = "_test_utils"))]
    pub fn new(local: &str) -> RawTendril {
        RawTendril {
            local: local.to_string(),
            remote: "".to_string(),
            mode: TendrilMode::DirOverwrite,
            profiles: vec![],
        }
    }

    pub(crate) fn resolve<'a>(
        &'a self,
        td_repo: &'a UniPath,
    ) -> Result<Tendril, InvalidTendrilError> {
        Tendril::new(
            td_repo,
            PathBuf::from(&self.local),
            UniPath::from(PathBuf::from(&self.remote)),
            self.mode.clone(),
        )
    }
}
