use crate::filtering::filter_by_mode;
use crate::{ActionMode, RawTendril, TendrilMode};
use rstest::rstest;

#[rstest]
#[case(ActionMode::Link)]
#[case(ActionMode::Push)]
#[case(ActionMode::Pull)]
#[case(ActionMode::Out)]
fn empty_tendril_list_returns_empty(#[case] action_mode: ActionMode) {
    let tendrils = vec![];

    let actual = filter_by_mode(tendrils, action_mode);

    assert!(actual.is_empty())
}

#[test]
fn link_action_only_includes_tendrils_with_link_true() {
    let mut t1 = RawTendril::new("SomeLocal");
    t1.mode = TendrilMode::Merge;
    let mut t2 = RawTendril::new("SomeLocal");
    t2.mode = TendrilMode::Link;
    let mut t3 = RawTendril::new("SomeLocal");
    t3.mode = TendrilMode::Overwrite;
    let tendrils = vec![t1.clone(), t2.clone(), t3.clone()];

    let actual = filter_by_mode(tendrils, ActionMode::Link);

    assert_eq!(actual, vec![t2]);
}

#[rstest]
#[case(ActionMode::Push)]
#[case(ActionMode::Pull)]
fn non_link_action_only_includes_tendrils_with_link_false(
    #[case] action_mode: ActionMode,
) {
    let mut t1 = RawTendril::new("SomeLocal");
    t1.mode = TendrilMode::Merge;
    let mut t2 = RawTendril::new("SomeLocal");
    t2.mode = TendrilMode::Link;
    let mut t3 = RawTendril::new("SomeLocal");
    t3.mode = TendrilMode::Overwrite;
    let tendrils = vec![t1.clone(), t2.clone(), t3.clone()];

    let actual = filter_by_mode(tendrils, action_mode);

    assert_eq!(actual, vec![t1, t3]);
}

#[test]
fn out_action_includes_all() {
    let mut t1 = RawTendril::new("SomeLocal");
    t1.mode = TendrilMode::Merge;
    let mut t2 = RawTendril::new("SomeLocal");
    t2.mode = TendrilMode::Link;
    let mut t3 = RawTendril::new("SomeLocal");
    t3.mode = TendrilMode::Overwrite;
    let tendrils = vec![t1.clone(), t2.clone(), t3.clone()];

    let actual = filter_by_mode(tendrils, ActionMode::Out);

    assert_eq!(actual, vec![t1, t2, t3]);
}
