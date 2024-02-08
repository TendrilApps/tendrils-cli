use crate::{push_tendril, TendrilMode};
use crate::test_utils::Setup;
use crate::errors::TendrilActionError;
use rstest::rstest;

#[rstest]
fn given_link_mode_tendril_returns_mode_mismatch_error(
    #[values(true, false)]
    dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();
    setup.make_ctrl_file();

    let mut tendril = setup.resolved_file_tendril();
    tendril.mode = TendrilMode::Link;

    let actual = push_tendril(&setup.td_dir, &tendril, dry_run, force);

    assert!(matches!(actual, Err(TendrilActionError::ModeMismatch)));
    assert!(!setup.local_file.exists());
}
