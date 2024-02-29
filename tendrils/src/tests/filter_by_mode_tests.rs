use crate::{action_mode::ActionMode, filter_by_mode, Tendril};
use rstest::rstest;

#[rstest]
#[case(ActionMode::Link)]
#[case(ActionMode::Push)]
#[case(ActionMode::Pull)]
fn empty_tendril_list_returns_empty(
    #[case] action_mode: ActionMode,
) {
    let tendrils = vec![];

    let actual = filter_by_mode(tendrils, action_mode);

    assert!(actual.is_empty())
}

#[test]
fn link_action_only_includes_tendrils_with_link_true() {
    let mut t1 = Tendril::new("SomeApp", vec!["misc.txt"]);
    t1.link = false;
    let mut t2 = Tendril::new("SomeApp", vec!["misc2.txt"]);
    t2.link = true;
    let mut t3 = Tendril::new("SomeApp", vec!["misc3.txt"]);
    t3.link = false;
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
    let mut t1 = Tendril::new("SomeApp", vec!["misc.txt"]);
    t1.link = false;
    let mut t2 = Tendril::new("SomeApp", vec!["misc2.txt"]);
    t2.link = true;
    let mut t3 = Tendril::new("SomeApp", vec!["misc3.txt"]);
    t3.link = false;
    let tendrils = vec![t1.clone(), t2.clone(), t3.clone()];

    let actual = filter_by_mode(tendrils, action_mode);

    assert_eq!(actual, vec![t1, t3]);
}
