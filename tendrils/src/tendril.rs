use crate::enums::{InvalidTendrilError, OneOrMany, TendrilMode};
use crate::path_ext::resolve_tilde;
use serde::{de, Deserialize, Deserializer, Serialize};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

#[cfg(test)]
mod tests;

/// A Tendril that is prepared for use with Tendril actions
/// and always exists in a valid state.
/// Note: This does *not* guarantee that the path
/// exists or is valid.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Tendril<'a> {
    group: &'a str,
    name: &'a str,
    parent: PathBuf,
    pub mode: TendrilMode,
}

impl<'a> Tendril<'a> {
    fn new(
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

    #[cfg(any(test, feature = "_test_utils"))]
    pub fn new_expose(
        group: &'a str,
        name: &'a str,
        parent: PathBuf,
        mode: TendrilMode,
    ) -> Result<Tendril<'a>, InvalidTendrilError> {
        Tendril::new(group, name, parent, mode)
    }

    #[cfg(test)]
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

    pub fn local_path(&self, td_repo: &Path) -> PathBuf {
        td_repo.join(self.group).join(self.name)
    }

    fn is_path(x: &str) -> bool {
        x.contains('/') || x.contains('\\')
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

    pub(crate) fn resolve_tendrils(
        &self,
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
        let resolved_parents: Vec<PathBuf> = raw_paths
            .iter()
            .map(|p| resolve_path_variables(String::from(p)))
            .map(|p| PathBuf::from(p))
            .collect();

        for name in self.names.iter() {
            for resolved_parent in resolved_parents.iter() {
                resolve_results.push(Tendril::new(
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

/// Replaces all environment variables in the format `<varname>` in the
/// given path with their values. If the variable is not found, the
/// `<varname>` is left as-is in the path.
///
/// The common tilde (`~`) symbol can also be used as a prefix to the path
/// and corresponds to the `HOME` environment variable on Unix/Windows.
/// If `HOME` doesn't exist, it will fallback to a combination of `HOMEDRIVE`
/// and `HOMEPATH` provided they both exist (otherwise the `~` is left as is).
/// This fallback is mainly a Windows specific issue, but is supported on all
/// platforms either way.
///
/// Any non UTF-8 characters in a variable's value or in the tilde value
/// are replaced with the U+FFFD replacement character.
///
/// # Limitations
/// If the path contains the `<pattern>` and the pattern corresponds to
/// an environment variable, there is no way to escape the brackets
/// to force it to use the raw path. This should only be an issue
/// on Unix (as Windows doesn't allow `<` or `>` in paths anyways),
/// and only when the variable exists (otherwise it uses the raw
/// path). In the future, an escape character such as `|` could be
/// implemented, but this added complexity was avoided for now.
fn resolve_path_variables(mut path: String) -> PathBuf {
    let path_temp = path.clone();
    let vars = parse_env_variables(&path_temp);

    for var in vars {
        let var_no_brkts = &var[1..var.len() - 1];
        let os_value =
            std::env::var_os(var_no_brkts).unwrap_or(OsString::from(var));
        let value = os_value.to_string_lossy();
        path = path.replace(var, &value);
    }

    if path.starts_with('~') {
        path = resolve_tilde(&path);
    }

    PathBuf::from(path)
}

/// Extracts all variable names in the given string that
/// are of the form `<varname>`. The surrounding brackets
/// are also returned.
fn parse_env_variables(input: &str) -> Vec<&str> {
    let mut vars = vec![];
    let mut depth = 0;
    let mut start_index = 0;

    for (index, ch) in input.chars().enumerate() {
        if ch == '<' {
            start_index = index;
            depth += 1;
        }
        else if ch == '>' && depth > 0 {
            if depth > 0 {
                vars.push(&input[start_index..=index]);
            }
            depth -= 1;
        }
    }

    vars
}

fn one_or_many_to_vec<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<String>, D::Error> {
    let one_or_many: OneOrMany<String> =
        de::Deserialize::deserialize(deserializer)?;
    Ok(one_or_many.into())
}
