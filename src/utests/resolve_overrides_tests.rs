use std::path::PathBuf;
use crate::{
    resolve_overrides, 
    Tendril
};
use crate::utests::common::set_all_platform_paths;
use crate::utests::sample_tendrils::SampleTendrils;

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
    set_all_platform_paths(
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
    misc_override.app = "I don't exist".to_string();
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
    set_all_platform_paths(
        &mut override_tendril,
        &[PathBuf::from("Some").join("override").join("path")]
    );
    override_tendril.folder_merge = !globals[0].folder_merge;
    let overrides = [override_tendril.clone()].to_vec();

    let expected = [override_tendril, SampleTendrils::tendril_2()].to_vec();

    let actual = resolve_overrides(&globals, &overrides);

    assert_eq!(actual, expected);
}
