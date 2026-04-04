use crate::filtering::filter_by_modes;
use crate::{RawTendril, TendrilMode};
use rstest::rstest;

#[rstest]
#[case(vec![TendrilMode::CopyMerge])]
#[case(vec![TendrilMode::CopyOverwrite])]
#[case(vec![TendrilMode::Link])]
fn empty_tendril_list_returns_empty(#[case] modes: Vec<TendrilMode>) {
    let tendrils = vec![];

    let actual = filter_by_modes(tendrils, &modes);

    assert!(actual.is_empty())
}

#[test]
fn tendril_matches_if_mode_matches_any() {
    let mut t1 = RawTendril::new("SomeLocal");
    t1.mode = TendrilMode::CopyMerge;
    let mut t2 = RawTendril::new("SomeLocal");
    t2.mode = TendrilMode::Link;
    let mut t3 = RawTendril::new("SomeLocal");
    t3.mode = TendrilMode::CopyOverwrite;
    let mut t4 = RawTendril::new("SomeLocal");
    t4.mode = TendrilMode::Link;
    let tendrils = vec![t1.clone(), t2.clone(), t3.clone(), t4.clone()];

    let actual = filter_by_modes(tendrils, &[
        TendrilMode::CopyOverwrite,
        TendrilMode::Link,
    ]);

    assert_eq!(actual, vec![t2, t3, t4]);
}

#[test]
fn duplicate_filter_only_returns_tendril_once() {
    let mut t1 = RawTendril::new("SomeLocal");
    t1.mode = TendrilMode::CopyMerge;
    let mut t2 = RawTendril::new("SomeLocal");
    t2.mode = TendrilMode::Link;
    let tendrils = vec![t1.clone(), t2.clone()];
    let modes = [TendrilMode::Link, TendrilMode::Link, TendrilMode::Link];

    let actual = filter_by_modes(tendrils, &modes);

    assert_eq!(actual, vec![t2]);
}

#[test]
fn empty_filters_list_returns_all_tendrils() {
    let mut t1 = RawTendril::new("SomeLocal");
    t1.mode = TendrilMode::CopyMerge;
    let mut t2 = RawTendril::new("SomeLocal");
    t2.mode = TendrilMode::Link;
    let mut t3 = RawTendril::new("SomeLocal");
    t3.mode = TendrilMode::CopyOverwrite;
    let tendrils = vec![t1.clone(), t2.clone(), t3.clone()];

    let actual = filter_by_modes(tendrils, &[]);

    assert_eq!(actual, vec![t1, t2, t3]);
}
