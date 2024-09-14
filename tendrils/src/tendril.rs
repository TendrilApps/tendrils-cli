use crate::enums::{InvalidTendrilError, OneOrMany, TendrilMode};
use crate::path_ext::{PathExt, UniPath};
use serde::{de, Deserialize, Deserializer, Serialize};
use std::path::{Path, PathBuf};

#[cfg(test)]
pub(crate) mod tests;

/// A tendril that is prepared for use with tendril actions
/// and always exists in a valid state.
/// Note: This does *not* guarantee that the path
/// exists or is valid.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Tendril<'a> {
    /// Given group.
    group: &'a str,
    name: &'a str,
    parent: UniPath,
    local: PathBuf,
    remote: PathBuf,
    pub mode: TendrilMode,
}

impl<'a> Tendril<'a> {
    fn new(
        td_repo: impl AsRef<UniPath>,
        group: &'a str,
        name: &'a str,
        parent: UniPath,
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

        let parent_bytes = parent.inner().as_os_str().as_encoded_bytes();
        if parent_bytes.is_empty()
            || parent_bytes.contains(&('\n' as u8))
            || parent_bytes.contains(&('\r' as u8))
        {
            return Err(InvalidTendrilError::InvalidParent);
        }

        #[cfg(not(windows))]
        let remote =
            parent.inner().join_raw(&PathBuf::from(name));

        #[cfg(windows)]
        let remote =
            parent.inner().join_raw(&PathBuf::from(name)).replace_dir_seps();

        if Self::is_recursive(td_repo.as_ref(), &remote) {
            return Err(InvalidTendrilError::Recursion)
        }

        #[cfg(not(windows))]
        let local = td_repo
            .as_ref()
            .inner()
            .join_raw(&Path::new(group))
            .join_raw(&Path::new(name))
            .into();
        #[cfg(windows)]
        let local = td_repo
            .as_ref()
            .inner()
            .join_raw(&Path::new(group))
            .join_raw(&Path::new(name))
            .replace_dir_seps()
            .into();

        Ok(Tendril { group, name, parent, local, remote, mode })
    }

    #[cfg(any(test, feature = "_test_utils"))]
    pub fn new_expose(
        td_repo: impl AsRef<UniPath>,
        group: &'a str,
        name: &'a str,
        parent: UniPath,
        mode: TendrilMode,
    ) -> Result<Tendril<'a>, InvalidTendrilError> {
        Tendril::new(td_repo.as_ref(), group, name, parent, mode)
    }

    /// Name as given.
    #[cfg(test)]
    pub fn name(&self) -> &str {
        self.name
    }

    /// Sanitized parent path.
    pub fn parent(&self) -> &UniPath {
        &self.parent
    }

    /// The resolved path to this file system object inside the Tendrils repo.
    /// The combination of the given Tendrils repo, [`Self::group`], and
    /// [`Self::name`]
    pub fn local(&self) -> &PathBuf {
        &self.local
    }

    /// The resolved path to this file system object specific to this machine,
    /// outside of the Tendrils repo. The combination of [`Self::parent`] and
    /// [`Self::name`].
    pub fn remote(&self) -> &PathBuf {
        &self.remote
    }

    fn is_path(x: &str) -> bool {
        x.contains('/') || x.contains('\\')
    }

    fn is_recursive(td_repo: &UniPath, remote: &Path) -> bool {
        let repo_inner = td_repo.inner();
        repo_inner == remote
            || repo_inner.ancestors().any(|p| p == remote)
            || remote.ancestors().any(|p| p == repo_inner)
    }
}

/// Represents a bundle of file system objects that are controlled
/// by Tendrils.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TendrilBundle {
    /// The group by which this tendril will be sorted in
    /// the Tendrils repo.
    pub group: String,

    /// A list of file/folder names, each one belonging to each of the
    /// `parents`.
    #[serde(deserialize_with = "one_or_many_to_vec")]
    pub names: Vec<String>,

    /// A list of parent folders containing the files/folders in `names`.
    /// Each parent will be combined with each name to expand to individual
    /// tendrils.
    #[serde(deserialize_with = "one_or_many_to_vec")]
    pub parents: Vec<String>,

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
    pub fn new(group: &str, names: Vec<&str>) -> TendrilBundle {
        TendrilBundle {
            group: String::from(group),
            names: names.into_iter().map(|n: &str| String::from(n)).collect(),
            parents: vec![],
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

        let raw_paths = match (first_only, self.parents.is_empty()) {
            (true, false) => vec![self.parents[0].clone()],
            (false, false) => self.parents.clone(),
            (_, true) => vec![],
        };

        let mut resolve_results =
            Vec::with_capacity(self.names.len() * self.parents.len());

        // Resolve parents early to prevent doing this on
        // each iteration
        let resolved_parents: Vec<UniPath> = raw_paths
            .iter()
            .map(|p| UniPath::from(PathBuf::from(p)))
            .collect();

        for name in self.names.iter() {
            for resolved_parent in resolved_parents.iter() {
                resolve_results.push(Tendril::new(
                    td_repo,
                    &self.group,
                    name,
                    resolved_parent.clone(),
                    mode.clone(),
                ));
            }
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
