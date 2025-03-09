use crate::{
    FsoType,
    InvalidTendrilError,
    RawTendril,
    TendrilActionError,
    TendrilActionSuccess,
};
use std::marker::PhantomData;
use std::path::PathBuf;

/// Generic report format for any operation on a [`RawTendril`]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TendrilReport<T: TendrilLog> {
    /// The original tendril bundle that this tendril was expanded from.
    pub raw_tendril: RawTendril,

    /// Result containing the log from the operation, provided
    /// the tendril was valid.
    /// Otherwise, it contains the [`InvalidTendrilError`].
    pub log: Result<T, InvalidTendrilError>,
}

/// Generic log information for any operation on a tendril
pub trait TendrilLog {
    /// The type of the file system object in the Tendrils repo.
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
        ActionLog { local_type, remote_type, resolved_path, result }
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

/// Contains various updater functions for live feedback during a tendrils
/// command.
pub trait UpdateHandler<L>
where
    L: TendrilLog
{
    /// Accepts `value` indicating the total count
    /// of all raw tendrils that will be processed.
    fn count(&mut self, value: i32);

    /// Accepts `raw` indicating the tendril that will be processed.
    /// This is called *before* processing.
    fn before(&mut self, raw: RawTendril);

    /// Accepts `report` containing the result of the operation.
    /// This is called *after* processing.
    fn after(&mut self, report: TendrilReport<L>);
}

/// Accepts callbacks to be called by the [`UpdateHandler`] methods.
pub struct CallbackUpdater<A, B, C, L>
where
    A: FnMut(TendrilReport<L>),
    B: FnMut(RawTendril),
    C: FnMut(i32),
    L: TendrilLog,
{
    pub count: C,
    pub before: B,
    pub after: A,
    _marker: PhantomData<L>,
}

impl<A, B, C, L> CallbackUpdater<A, B, C, L>
where
    A: FnMut(TendrilReport<L>),
    B: FnMut(RawTendril),
    C: FnMut(i32),
    L: TendrilLog,
{
    pub fn new(count: C, before: B, after: A) -> CallbackUpdater<A, B, C, L> {
        CallbackUpdater {
            count,
            before,
            after,
            _marker: PhantomData,
        }
    }

    #[cfg(test)]
    pub fn default() -> CallbackUpdater<impl FnMut(TendrilReport<L>), impl FnMut(RawTendril), impl FnMut(i32), L> {
        CallbackUpdater {
            count: |_| {},
            before: |_| {},
            after: |_| {},
            _marker: PhantomData,
        }
    }
}

impl<A, B, C, L> UpdateHandler<L> for CallbackUpdater<A, B, C, L>
where
    A: FnMut(TendrilReport<L>),
    B: FnMut(RawTendril),
    C: FnMut(i32),
    L: TendrilLog,
{
    fn count(&mut self, total: i32) {
        (self.count)(total)
    }

    fn before(&mut self, raw: RawTendril) {
        (self.before)(raw)
    }

    fn after(&mut self, report: TendrilReport<L>) {
        (self.after)(report)
    }
}
