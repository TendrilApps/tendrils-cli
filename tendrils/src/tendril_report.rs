use crate::{
    FsoType,
    InvalidTendrilError,
    TendrilActionError,
    TendrilActionSuccess,
    TendrilBundle,
};
use std::path::PathBuf;

/// Generic report format for any operation on a [`TendrilBundle`]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TendrilReport<'a, T> where T: TendrilLog {
    /// The original tendril bundle that this tendril was expanded from
    pub orig_tendril: &'a TendrilBundle,
    /// The name of the tendril that was expanded. If this `orig_tendril`
    /// contains many names, this indicate which was used
    pub name: &'a str,
    /// Result containing the log from the operation, provided
    /// the tendril was valid.
    /// Otherwise, it contains the [`InvalidTendrilError`]
    pub log: Result<T, InvalidTendrilError>
}

/// Generic log information for any operation on a tendril
pub trait TendrilLog {
    /// The type of the file system object in the Tendrils folder.
    /// `None` if it does not exist.
    fn local_type(&self) -> &Option<FsoType>;

    /// The type of the file system object at its device-specific
    /// location.
    /// `None` if it does not exist.
    fn remote_type(&self) -> &Option<FsoType>;

    /// The full path to the remote. This shows the result after resolving
    /// all environment/other variables in the path.
    fn resolved_path(&self) -> &PathBuf;
}

/// Contains the metadata from a single tendrils action.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionLog {
    local_type: Option<FsoType>,
    remote_type: Option<FsoType>,
    resolved_path: PathBuf,
    /// Result of this individual action.
    pub result: Result<TendrilActionSuccess, TendrilActionError>,
}

impl ActionLog {
    pub fn new(
        local_type: Option<FsoType>,
        remote_type: Option<FsoType>,
        resolved_path: PathBuf,
        result: Result<TendrilActionSuccess, TendrilActionError>,
    ) -> ActionLog {
        ActionLog {local_type, remote_type, resolved_path, result}
    }
}

impl TendrilLog for ActionLog {
    fn local_type(&self) -> &Option<FsoType> {
        &self.local_type
    }
    fn remote_type(&self) -> &Option<FsoType> {
        &self.remote_type
    }
    fn resolved_path(&self) -> &PathBuf {
        &self.resolved_path
    }
}
