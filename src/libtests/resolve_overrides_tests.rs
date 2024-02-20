use crate::{
    resolve_overrides, 
    Tendril
};
use crate::libtests::sample_tendrils::SampleTendrils;
use crate::test_utils::set_parents;
use std::path::PathBuf;

#[test]
fn empty_overrides_returns_globals() {
    let globals = [
        SampleTendrils::tendril_1(),
        SampleTendrils::tendril_1()
    ].to_vec();
    let overrides = [].to_vec();

    let actual = resolve_overrides(&globals, &overrides);

    assert_eq!(actual, globals);
}

#[test]
fn empty_globals_returns_empty() {
    let globals = [].to_vec();

    let mut override_tendril = SampleTendrils::tendril_1();
    set_parents(
        &mut override_tendril,
        &[PathBuf::from("Some").join("override").join("path")]
    );
    let overrides = [override_tendril.clone()].to_vec();

    let actual = resolve_overrides(&globals, &overrides);

    assert!(actual.is_empty());
}

#[test]
fn both_empty_returns_empty() {
    let globals = [].to_vec();
    let overrides = [].to_vec();

    let actual = resolve_overrides(&globals, &overrides);

    assert!(actual.is_empty());
}

#[test]
fn both_equal_returns_globals() {
    let globals = [SampleTendrils::tendril_1()].to_vec();
    let overrides = &globals;

    let actual = resolve_overrides(&globals, &overrides);

    assert_eq!(actual, globals);
}

#[test]
fn overrides_not_matching_globals_are_ignored() {
    let globals = [SampleTendrils::tendril_1()].to_vec();
    let mut misc_override = SampleTendrils::tendril_1();
    misc_override.group = "I don't exist".to_string();
    misc_override.name = "Me neither".to_string();
    let overrides = [misc_override].to_vec();

    let actual = resolve_overrides(&globals, &overrides);

    assert_eq!(actual, globals);
}

#[test]
fn overrides_matching_globals_override_globals() {
    let globals:Vec<Tendril> = [
        SampleTendrils::tendril_1(),
        SampleTendrils::tendril_2(),
    ].to_vec();

    let mut override_tendril = globals[0].clone();
    set_parents(
        &mut override_tendril,
        &[PathBuf::from("Some").join("override").join("path")]
    );
    override_tendril.dir_merge = !globals[0].dir_merge;
    override_tendril.link = true;
    let overrides = [override_tendril.clone()].to_vec();

    let expected = [override_tendril, SampleTendrils::tendril_2()].to_vec();

    let actual = resolve_overrides(&globals, &overrides);

    assert_eq!(actual, expected);
}
