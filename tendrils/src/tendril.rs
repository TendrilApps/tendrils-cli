use crate::enums::{InvalidTendrilError, OneOrMany, TendrilMode};
use crate::path_ext::{PathExt, UniPath};
use serde::{de, Deserialize, Deserializer, Serialize};
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

/// Represents a bundle of file system objects that are controlled
/// by Tendrils.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TendrilBundle {
    /// The path to the master file/folder relative to the root of the
    /// Tendrils repo.
    pub local: String,

    /// A list of absolute paths to the various locations on the machine at
    /// which to recreate the [`Self::local`]. Each `local` and
    /// `remote` pair forms a tendril.
    #[serde(deserialize_with = "one_or_many_to_vec")]
    pub remotes: Vec<String>,

    /// `true` indicates that each tendril will have
    /// [`crate::TendrilMode::DirMerge`]. `false` indicates
    /// [`crate::TendrilMode::DirOverwrite`]. Note: this field
    /// may be overriden depending on the value of `link`.
    #[serde(rename = "dir-merge")]
    #[serde(default)]
    pub dir_merge: bool,

    /// `true` indicates that each tendril will have
    /// [`crate::TendrilMode::Link`], regardless of what the `dir_merge`
    /// setting is. `false` indicates that the `dir_merge` setting will be
    /// used.
    #[serde(default)]
    pub link: bool,

    /// A list of profiles to which this tendril belongs. If empty,
    /// this tendril is considered to be included in *all* profiles.
    #[serde(default)]
    #[serde(deserialize_with = "one_or_many_to_vec")]
    pub profiles: Vec<String>,
}

impl TendrilBundle {
    #[cfg(any(test, feature = "_test_utils"))]
    pub fn new(local: &str) -> TendrilBundle {
        TendrilBundle {
            local: String::from(local),
            remotes: vec![],
            dir_merge: false,
            link: false,
            profiles: vec![],
        }
    }

    pub(crate) fn resolve_tendrils<'a>(
        &'a self,
        td_repo: &'a UniPath,
        first_only: bool,
    ) -> Vec<Result<Tendril, InvalidTendrilError>> {
        let mode = match (self.dir_merge, self.link) {
            (true, false) => TendrilMode::DirMerge,
            (false, false) => TendrilMode::DirOverwrite,
            (_, true) => TendrilMode::Link,
        };

        let remotes = match (first_only, self.remotes.is_empty()) {
            (true, false) => vec![self.remotes[0].clone()],
            (false, false) => self.remotes.clone(),
            (_, true) => vec![],
        };

        let mut resolve_results = Vec::with_capacity(remotes.len());

        for remote in remotes.into_iter() {
            resolve_results.push(Tendril::new(
                td_repo,
                PathBuf::from(&self.local),
                UniPath::from(PathBuf::from(remote)),
                mode.clone(),
            ));
        }

        resolve_results
    }
}

fn one_or_many_to_vec<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<String>, D::Error> {
    let one_or_many: OneOrMany<String> =
        de::Deserialize::deserialize(deserializer)?;
    Ok(one_or_many.into())
}
