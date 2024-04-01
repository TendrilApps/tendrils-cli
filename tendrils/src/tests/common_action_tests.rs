//! Contains tests that are common to some or all
//! of the tendril actions.
//! See also:
//! - [`crate::tests::link_tendril_tests`]
//! - [`crate::tests::pull_tendril_tests`]
//! - [`crate::tests::push_tendril_tests`]

use crate::{
    is_tendrils_dir,
    link_tendril,
    pull_tendril,
    push_tendril,
};
use crate::enums::{TendrilActionError, TendrilActionSuccess, TendrilMode};
use crate::tendril::Tendril;
use crate::test_utils::{get_disposable_dir, symlink_expose, Setup};
use rstest::rstest;
use rstest_reuse::{self, apply, template};
use serial_test::serial;
use std::fs::{create_dir_all, read_to_string, remove_dir, remove_file, write};
use std::path::{Path, PathBuf};
use tempdir::TempDir;

#[rstest]
#[case(link_tendril)]
#[case(pull_tendril)]
#[case(push_tendril)]
fn remote_is_given_td_dir_returns_recursion_error(
    #[case] action: fn(&Path, &Tendril, bool, bool)
        -> Result<TendrilActionSuccess, TendrilActionError>,

    #[values(true, false)]
    dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();

    let mut tendril = Tendril::new(
        "SomeApp",
        "TendrilsDir",
        setup.td_dir.parent().unwrap().to_path_buf(),
        TendrilMode::DirOverwrite,
    ).unwrap();
    if action == link_tendril {
        tendril.mode = TendrilMode::Link;
    }

    let actual = action(&setup.td_dir, &tendril, dry_run, force);

    assert!(matches!(actual, Err(TendrilActionError::Recursion)));
}

#[rstest]
#[case(link_tendril)]
#[case(pull_tendril)]
#[case(push_tendril)]
fn remote_is_ancestor_to_given_td_dir_returns_recursion_error(
    #[case] action: fn(&Path, &Tendril, bool, bool)
        -> Result<TendrilActionSuccess, TendrilActionError>,

    #[values(true, false)]
    dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let mut setup = Setup::new();
    setup.td_dir = setup.parent_dir  
        .join("Nested1")
        .join("Nested2")
        .join("Nested3")
        .join("TendrilsDir");

    let mut tendril = Tendril::new(
        "SomeApp",
        "Nested1",
        setup.parent_dir,
        TendrilMode::DirOverwrite,
    ).unwrap();
    if action == link_tendril {
        tendril.mode = TendrilMode::Link;
    }

    let actual = action(&setup.td_dir, &tendril, dry_run, force);

    assert!(matches!(actual, Err(TendrilActionError::Recursion)));
}

#[rstest]
#[case(link_tendril)]
#[case(pull_tendril)]
#[case(push_tendril)]
fn remote_is_direct_child_of_given_td_dir_returns_recursion_error(
    #[case] action: fn(&Path, &Tendril, bool, bool)
        -> Result<TendrilActionSuccess, TendrilActionError>,

    #[values(true, false)]
    dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let temp_td_dir = TempDir::new_in(
        get_disposable_dir(),
        "TendrilsDir"
    ).unwrap();
    let td_dir = temp_td_dir.path().to_path_buf();
    let parent_dir = td_dir.clone();
    let remote_file = parent_dir.join("misc.txt");
    write(&remote_file, "Remote file contents").unwrap();

    let mut tendril = Tendril::new(
        "SomeApp",
        "misc.txt",
        parent_dir,
        TendrilMode::DirOverwrite,
    ).unwrap();
    if action == link_tendril {
        tendril.mode = TendrilMode::Link;
    }

    let actual = action( &td_dir, &tendril, dry_run, force);

    assert!(matches!(actual, Err(TendrilActionError::Recursion)));
}

#[rstest]
#[case(link_tendril)]
#[case(pull_tendril)]
#[case(push_tendril)]
fn remote_is_nested_child_of_given_td_dir_returns_recursion_error(
    #[case] action: fn(&Path, &Tendril, bool, bool)
        -> Result<TendrilActionSuccess, TendrilActionError>,

    #[values(true, false)]
    dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let temp_td_dir = TempDir::new_in(
        get_disposable_dir(),
        "TendrilsDir"
    ).unwrap();
    let td_dir = temp_td_dir.path().to_path_buf();
    let parent_dir = td_dir
        .join("Nested1")
        .join("Nested2")
        .join("Nested3");
    let remote_file = parent_dir
        .join("misc.txt");
    create_dir_all(&remote_file.parent().unwrap()).unwrap();
    write(&remote_file, "Remote file contents").unwrap();

    let mut tendril = Tendril::new(
        "SomeApp",
        "misc.txt",
        parent_dir,
        TendrilMode::DirOverwrite,
    ).unwrap();
    if action == link_tendril {
        tendril.mode = TendrilMode::Link;
    }

    let actual = action(&td_dir, &tendril, dry_run, force);

    assert!(matches!(actual, Err(TendrilActionError::Recursion)));
}

#[rstest]
#[case(link_tendril)]
#[case(pull_tendril)]
#[case(push_tendril)]
fn remote_is_sibling_to_given_td_dir_proceeds_normally(
    #[case] action: fn(&Path, &Tendril, bool, bool)
        -> Result<TendrilActionSuccess, TendrilActionError>,

    #[values(true, false)]
    dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();
    setup.make_local_dir();
    setup.make_target_nested_file();
    assert_eq!( // Check they are siblings
        setup.remote_dir.parent().unwrap(),
        setup.td_dir.parent().unwrap()
    );

    let mut tendril = setup.dir_tendril();
    if action == link_tendril {
        tendril.mode = TendrilMode::Link;
        symlink_expose(&setup.remote_dir, &setup.target_dir, false, true)
            .unwrap();
    }
    else {
        setup.make_remote_dir();
    }

    let actual = action(&setup.td_dir, &tendril, dry_run, force);

    if dry_run {
        assert!(matches!(actual, Ok(TendrilActionSuccess::OverwriteSkipped)));
    }
    else {
        assert!(matches!(actual, Ok(TendrilActionSuccess::Overwrite)));
    }
}

#[rstest]
#[case(link_tendril)]
#[case(pull_tendril)]
#[case(push_tendril)]
fn remote_is_another_td_dir_proceeds_normally(
    #[case] action: fn(&Path, &Tendril, bool, bool)
        -> Result<TendrilActionSuccess, TendrilActionError>,

    #[values(true, false)]
    dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();
    setup.make_local_nested_file();
    setup.make_remote_nested_file();
    write(&setup.remote_dir.join("tendrils.json"), "").unwrap();
    assert!(is_tendrils_dir(&setup.remote_dir));

    let mut tendril = setup.dir_tendril();
    if action == link_tendril {
        tendril.mode = TendrilMode::Link;
    }

    let actual = action(&setup.td_dir, &tendril, dry_run, force);

    if action == link_tendril && !force {
        assert!(matches!(actual, Err(TendrilActionError::TypeMismatch)))
    }
    else if dry_run {
        assert!(matches!(actual, Ok(TendrilActionSuccess::OverwriteSkipped)));
    }
    else {
        assert!(matches!(actual, Ok(TendrilActionSuccess::Overwrite)));
    }
}

#[rstest]
#[case("<mut-testing>", "TendrilsDir", "SomeApp", "misc.txt")]
#[case("Parent", "<mut-testing>", "SomeApp", "misc.txt")]
#[case("Parent", "TendrilsDir", "<mut-testing>", "misc.txt")]
#[case("Parent", "TendrilsDir", "SomeApp", "<mut-testing>")]
#[case("<I_DO_NOT_EXIST>", "TendrilsDir", "SomeApp", "misc.txt")]
#[case("Parent", "<I_DO_NOT_EXIST>", "SomeApp", "misc.txt")]
#[case("Parent", "TendrilsDir", "<I_DO_NOT_EXIST>", "misc.txt")]
#[case("Parent", "TendrilsDir", "SomeApp", "<I_DO_NOT_EXIST>")]
#[cfg_attr(windows, ignore)] // These are invalid paths on Windows
#[serial("mut-env-var-testing")]
fn var_in_any_field_exists_uses_raw_path_even_if_var_exists(
    #[case] parent: &str,
    #[case] td_dir: &str,
    #[case] group: &str,
    #[case] name: &str,

    #[values(link_tendril, pull_tendril, push_tendril)]
    action: fn(&Path, &Tendril, bool, bool)
        -> Result<TendrilActionSuccess, TendrilActionError>,

    #[values(true, false)]
    dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    // Any variables should have been resolved at this point
    let mut setup = Setup::new();
    setup.parent_dir = setup.temp_dir.path().join(parent);
    setup.td_dir = setup.temp_dir.path().join(td_dir);
    setup.group_dir = setup.td_dir.join(group);
    setup.remote_file = setup.parent_dir.join(name);
    setup.local_file = setup.group_dir.join(name);
    setup.make_parent_dir();
    setup.make_local_file();
    setup.make_target_file();
    std::env::set_var("mut-testing", "NON-EXISTENT PATH");

    let mut tendril = Tendril::new(
        group,
        name,
        setup.parent_dir.clone(),
        TendrilMode::DirOverwrite,
    ).unwrap();
    if action == link_tendril {
        tendril.mode = TendrilMode::Link;
        symlink_expose(&setup.remote_file, &setup.target_file, false, true)
            .unwrap();
    }
    else {
        setup.make_remote_file();
    }

    let actual = action(&setup.td_dir, &tendril, dry_run, force);

    if dry_run {
        assert!(matches!(actual, Ok(TendrilActionSuccess::OverwriteSkipped)));
    }
    else {
        assert!(matches!(actual, Ok(TendrilActionSuccess::Overwrite)));
    }
}

#[rstest]
#[case(link_tendril)]
#[case(pull_tendril)]
#[case(push_tendril)]
fn other_tendrils_in_same_group_dir_are_unchanged(
    #[case] action: fn(&Path, &Tendril, bool, bool)
        -> Result<TendrilActionSuccess, TendrilActionError>,

    #[values(true, false)]
    dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();
    setup.make_local_file();
    setup.make_local_dir();
    setup.make_target_file();
    setup.make_target_dir();
    let some_other_local_file= &setup.group_dir.join("other.txt");
    let some_other_local_dir= &setup.group_dir.join("other");
    let some_other_local_nested= &setup.group_dir.join("nested.txt");
    create_dir_all(some_other_local_dir).unwrap();
    write(some_other_local_file, "Another tendril from the same group").unwrap();
    write(some_other_local_nested, "Another nested from the same group").unwrap();

    let mut file_tendril = setup.file_tendril();
    let mut dir_tendril = setup.dir_tendril();
    if action == link_tendril {
        file_tendril.mode = TendrilMode::Link;
        dir_tendril.mode = TendrilMode::Link;
        symlink_expose(&setup.remote_file, &setup.target_file, false, true)
            .unwrap();
        symlink_expose(&setup.remote_dir, &setup.target_dir, false, true)
            .unwrap();
    }
    else {
        setup.make_remote_file();
        setup.make_remote_dir();
    }

    let file_actual = action(&setup.td_dir, &file_tendril, dry_run, force);
    let dir_actual = action(&setup.td_dir, &dir_tendril, dry_run, force);

    // Check that other tendril is unchanged
    let some_other_local_file_contents = read_to_string(some_other_local_file).unwrap();
    let some_other_local_nested_contents = read_to_string(some_other_local_nested).unwrap();
    assert_eq!(some_other_local_file_contents, "Another tendril from the same group");
    assert_eq!(some_other_local_nested_contents, "Another nested from the same group");
    if dry_run {
        assert!(matches!(file_actual, Ok(TendrilActionSuccess::OverwriteSkipped)));
        assert!(matches!(dir_actual, Ok(TendrilActionSuccess::OverwriteSkipped)));
    }
    else {
        assert!(matches!(file_actual, Ok(TendrilActionSuccess::Overwrite)));
        assert!(matches!(dir_actual, Ok(TendrilActionSuccess::Overwrite)));
    }
}

#[rstest]
#[case(link_tendril)]
#[case(pull_tendril)]
#[case(push_tendril)]
fn remote_parent_doesnt_exist_returns_io_error_not_found(
    #[case] action: fn(&Path, &Tendril, bool, bool)
        -> Result<TendrilActionSuccess, TendrilActionError>,

    #[values(true, false)]
    dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let mut setup = Setup::new();
    setup.parent_dir = setup.parent_dir.join("IDoNotExist");
    setup.make_local_file();
    setup.make_local_nested_file();

    let mut file_tendril = setup.file_tendril();
    let mut dir_tendril = setup.dir_tendril();
    if action == link_tendril {
        file_tendril.mode = TendrilMode::Link;
        dir_tendril.mode = TendrilMode::Link;
    }
    assert!(!file_tendril.full_path().parent().unwrap().exists());
    assert!(!dir_tendril.full_path().parent().unwrap().exists());

    let file_actual = action(&setup.td_dir, &file_tendril, dry_run, force);
    let dir_actual = action(&setup.td_dir, &dir_tendril, dry_run, force);

    match file_actual {
        Err(TendrilActionError::IoError(e)) => {
            assert_eq!(e.kind(), std::io::ErrorKind::NotFound);
        },
        _ => panic!("Actual error: {:?}", file_actual),
    }
    match dir_actual {
        Err(TendrilActionError::IoError(e)) => {
            assert_eq!(e.kind(), std::io::ErrorKind::NotFound);
        },
        _ => panic!("Actual error: {:?}", dir_actual),
    }
    assert_eq!(setup.local_file_contents(), "Local file contents");
    assert_eq!(setup.local_nested_file_contents(), "Local nested file contents");
}

#[rstest]
#[case(link_tendril)]
#[case(pull_tendril)]
#[case(push_tendril)]
fn td_dir_doesnt_exist_returns_io_error_not_found(
    #[case] action: fn(&Path, &Tendril, bool, bool)
        -> Result<TendrilActionSuccess, TendrilActionError>,

    #[values(true, false)]
    dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();
    let mut file_tendril = setup.file_tendril();
    let mut dir_tendril = setup.dir_tendril();
    if action == link_tendril {
        file_tendril.mode = TendrilMode::Link;
        dir_tendril.mode = TendrilMode::Link;
        setup.make_target_file();
        setup.make_target_dir();
        symlink_expose(&setup.remote_file, &setup.target_file, false, true)
            .unwrap();
        symlink_expose(&setup.remote_file, &setup.target_file, false, true)
            .unwrap();
    }
    else {
        setup.make_remote_file();
        setup.make_remote_dir();
    }
    assert!(!setup.td_dir.exists());

    let actual = action(&setup.td_dir, &file_tendril, dry_run, force);

    match actual {
        Err(TendrilActionError::IoError(e)) => assert_eq!(e.kind(), std::io::ErrorKind::NotFound),
        _ => panic!("Actual error: {:?}", actual),
    }
    assert!(!setup.td_dir.exists());
}

#[rstest]
#[case(pull_tendril)]
#[case(push_tendril)]
fn link_mode_tendril_returns_mode_mismatch_error(
    #[case] action: fn(&Path, &Tendril, bool, bool)
        -> Result<TendrilActionSuccess, TendrilActionError>,

    #[values(true, false)]
    dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();
    setup.make_remote_file();
    setup.make_local_file();

    let mut tendril = setup.file_tendril();
    tendril.mode = TendrilMode::Link;

    let actual = action(&setup.td_dir, &tendril, dry_run, force);

    assert!(matches!(actual, Err(TendrilActionError::ModeMismatch)));
    assert_eq!(&setup.remote_file_contents(), "Remote file contents");
    assert_eq!(&setup.local_file_contents(), "Local file contents");
}

#[template]
#[rstest]
#[case(link_tendril, true)]
#[case(link_tendril, false)]
#[case(pull_tendril, true)] // Only applies to pull in a dry run
#[case(push_tendril, true)]
#[case(push_tendril, false)]
fn cases_that_do_not_modify_local(
    #[case] action: fn(&Path, &ResolvedTendril, bool, bool)
        -> Result<TendrilActionSuccess, TendrilActionError>,

    #[case] dry_run: bool,
) { }

#[apply(cases_that_do_not_modify_local)]
pub(crate) fn local_is_unchanged(
    #[case] action: fn(&Path, &Tendril, bool, bool)
        -> Result<TendrilActionSuccess, TendrilActionError>,

    #[case] dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();
    setup.make_local_file();
    setup.make_local_nested_file();
    setup.make_target_file();
    setup.make_target_nested_file();

    let mut file_tendril = setup.file_tendril();
    let mut dir_tendril = setup.dir_tendril();
    if action == link_tendril {
        file_tendril.mode = TendrilMode::Link;
        dir_tendril.mode = TendrilMode::Link;
        symlink_expose(&setup.remote_file, &setup.target_file, false, true)
            .unwrap();
        symlink_expose(&setup.remote_dir, &setup.target_dir, false, true)
            .unwrap();
    }
    else {
        setup.make_remote_file();
        setup.make_remote_nested_file();
    }

    let file_actual = action(&setup.td_dir, &file_tendril, dry_run, force);
    let dir_actual = action(&setup.td_dir, &dir_tendril, dry_run, force);

    assert_eq!(setup.local_file_contents(), "Local file contents");
    assert_eq!(setup.local_nested_file_contents(), "Local nested file contents");
    if dry_run {
        assert!(matches!(file_actual, Ok(TendrilActionSuccess::OverwriteSkipped)));
        assert!(matches!(dir_actual, Ok(TendrilActionSuccess::OverwriteSkipped)));
    }
    else {
        assert!(matches!(file_actual, Ok(TendrilActionSuccess::Overwrite)));
        assert!(matches!(dir_actual, Ok(TendrilActionSuccess::Overwrite)));
    }
}

#[apply(cases_that_do_not_modify_local)]
pub(crate) fn local_symlink_is_unchanged(
    #[case] action: fn(&Path, &Tendril, bool, bool)
        -> Result<TendrilActionSuccess, TendrilActionError>,

    #[case] dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();
    setup.make_remote_file();
    setup.make_remote_nested_file();
    setup.make_target_file();
    setup.make_target_nested_file();
    setup.make_group_dir();
    symlink_expose(&setup.local_file, &setup.target_file, false, true).unwrap();
    symlink_expose(&setup.local_dir, &setup.target_dir, false, true).unwrap();

    let mut file_tendril = setup.file_tendril();
    let mut dir_tendril = setup.dir_tendril();
    if action == link_tendril {
        file_tendril.mode = TendrilMode::Link;
        dir_tendril.mode = TendrilMode::Link;
    }

    let file_actual = action(&setup.td_dir, &file_tendril, dry_run, force);
    let dir_actual = action(&setup.td_dir, &dir_tendril, dry_run, force);

    assert!(setup.local_file.is_symlink());
    assert!(setup.local_dir.is_symlink());
    assert_eq!(setup.local_file_contents(), "Target file contents");
    assert_eq!(setup.local_nested_file_contents(), "Target nested file contents");
    if !force {
        assert!(matches!(file_actual, Err(TendrilActionError::TypeMismatch)));
        assert!(matches!(dir_actual, Err(TendrilActionError::TypeMismatch)));
    }
    else if dry_run {
        assert!(matches!(file_actual, Ok(TendrilActionSuccess::OverwriteSkipped)));
        assert!(matches!(dir_actual, Ok(TendrilActionSuccess::OverwriteSkipped)));
    }
    else {
        assert!(matches!(file_actual, Ok(TendrilActionSuccess::Overwrite)));
        assert!(matches!(dir_actual, Ok(TendrilActionSuccess::Overwrite)));
    }
}

#[template]
#[rstest]
#[case(link_tendril, true)] // Only applies to link in a dry run
#[case(pull_tendril, true)] 
#[case(pull_tendril, false)] 
#[case(push_tendril, true)] // Only applies to push in a dry run
fn cases_that_do_not_modify_remote(
    #[case] action: fn(&Path, &ResolvedTendril, bool, bool)
        -> Result<TendrilActionSuccess, TendrilActionError>,

    #[case] dry_run: bool,
) { }

#[apply(cases_that_do_not_modify_remote)]
pub(crate) fn remote_is_unchanged(
    #[case] action: fn(&Path, &Tendril, bool, bool)
        -> Result<TendrilActionSuccess, TendrilActionError>,

    #[case] dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();
    setup.make_local_file();
    setup.make_local_nested_file();
    setup.make_remote_file();
    setup.make_remote_nested_file();

    let mut file_tendril = setup.file_tendril();
    let mut dir_tendril = setup.dir_tendril();
    if action == link_tendril {
        file_tendril.mode = TendrilMode::Link;
        dir_tendril.mode = TendrilMode::Link;
    }

    let file_actual = action(&setup.td_dir, &file_tendril, dry_run, force);
    let dir_actual = action(&setup.td_dir, &dir_tendril, dry_run, force);

    assert_eq!(setup.remote_file_contents(), "Remote file contents");
    assert_eq!(setup.remote_nested_file_contents(), "Remote nested file contents");
    if !force && action == link_tendril {
        assert!(matches!(file_actual, Err(TendrilActionError::TypeMismatch)));
        assert!(matches!(dir_actual,  Err(TendrilActionError::TypeMismatch)));
    }
    else if dry_run {
        assert!(matches!(file_actual, Ok(TendrilActionSuccess::OverwriteSkipped)));
        assert!(matches!(dir_actual, Ok(TendrilActionSuccess::OverwriteSkipped)));
    }
    else {
        assert!(matches!(file_actual, Ok(TendrilActionSuccess::Overwrite)));
        assert!(matches!(dir_actual, Ok(TendrilActionSuccess::Overwrite)));
    }
}

#[apply(cases_that_do_not_modify_remote)]
pub(crate) fn remote_symlink_is_unchanged(
    #[case] action: fn(&Path, &Tendril, bool, bool)
        -> Result<TendrilActionSuccess, TendrilActionError>,

    #[case] dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();
    setup.make_local_file();
    setup.make_local_nested_file();
    setup.make_target_file();
    setup.make_target_nested_file();
    symlink_expose(&setup.remote_file, &setup.target_file, false, true)
        .unwrap();
    symlink_expose(&setup.remote_dir, &setup.target_dir, false, true)
        .unwrap();

    let mut file_tendril = setup.file_tendril();
    let mut dir_tendril = setup.dir_tendril();
    if action == link_tendril {
        file_tendril.mode = TendrilMode::Link;
        dir_tendril.mode = TendrilMode::Link;
    }

    let file_actual = action(&setup.td_dir, &file_tendril, dry_run, force);
    let dir_actual = action(&setup.td_dir, &dir_tendril, dry_run, force);

    assert!(setup.remote_file.is_symlink());
    assert!(setup.remote_dir.is_symlink());
    assert_eq!(setup.remote_file_contents(), "Target file contents");
    assert_eq!(setup.remote_nested_file_contents(), "Target nested file contents");
    if !force && action != link_tendril {
        assert!(matches!(file_actual, Err(TendrilActionError::TypeMismatch)));
        assert!(matches!(dir_actual,  Err(TendrilActionError::TypeMismatch)));
    }
    else if dry_run {
        assert!(matches!(file_actual, Ok(TendrilActionSuccess::OverwriteSkipped)));
        assert!(matches!(dir_actual, Ok(TendrilActionSuccess::OverwriteSkipped)));
    }
    else {
        assert!(matches!(file_actual, Ok(TendrilActionSuccess::Overwrite)));
        assert!(matches!(dir_actual, Ok(TendrilActionSuccess::Overwrite)));
    }
}

#[rstest]
#[case(link_tendril)]
#[case(pull_tendril)]
#[case(push_tendril)]
fn current_dir_is_unchanged(
    #[case] action: fn(&Path, &Tendril, bool, bool)
        -> Result<TendrilActionSuccess, TendrilActionError>,

    #[values(true, false)]
    dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();

    let orig_cd = std::env::current_dir().unwrap();

    let mut file_tendril = setup.file_tendril();
    let mut dir_tendril = setup.dir_tendril();
    if action == link_tendril {
        file_tendril.mode = TendrilMode::Link;
        dir_tendril.mode = TendrilMode::Link;
    }

    let file_actual = action(&setup.td_dir, &file_tendril, dry_run, force);
    let dir_actual = action(&setup.td_dir, &dir_tendril, dry_run, force);

    assert_eq!(std::env::current_dir().unwrap(), orig_cd);
    match file_actual {
        Err(TendrilActionError::IoError(e)) => {
            assert_eq!(e.kind(), std::io::ErrorKind::NotFound);
        },
        _ => panic!("Actual error: {:?}", file_actual),
    }
    match dir_actual {
        Err(TendrilActionError::IoError(e)) => {
            assert_eq!(e.kind(), std::io::ErrorKind::NotFound);
        },
        _ => panic!("Actual error: {:?}", dir_actual),
    }
}


#[rstest]
#[case(link_tendril)]
#[case(pull_tendril)]
#[case(push_tendril)]
#[cfg_attr(not(windows), ignore)]
#[serial("root")]
fn windows_platform_parent_is_root_returns_permission_error_unless_dry_run_or_dir(
    #[case] action: fn(&Path, &Tendril, bool, bool)
        -> Result<TendrilActionSuccess, TendrilActionError>,

    #[values(true, false)]
    dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let mut setup = Setup::new();
    setup.parent_dir = PathBuf::from(
        std::env::var("HOME_DRIVE").unwrap_or("C:\\".to_string())
    );
    assert_eq!(setup.parent_dir.parent(), None);
    setup.remote_file = setup.parent_dir.join("tendrils_test_file.txt");
    setup.remote_dir = setup.parent_dir.join("tendrils_test_dir");
    setup.remote_nested_file = setup.remote_dir.join("nested.txt");
    setup.local_file = setup.group_dir.join("tendrils_test_file.txt");
    setup.local_dir = setup.group_dir.join("tendrils_test_dir");
    setup.local_nested_file = setup.local_dir.join("nested.txt");
    setup.make_local_file();
    setup.make_local_nested_file();
    let mut mode = TendrilMode::DirOverwrite;
    if action == link_tendril {
        mode = TendrilMode::Link;
    }
    else if action == pull_tendril {
        setup.make_remote_nested_file();
    }

    let file_tendril = Tendril::new(
        "SomeApp",
        "tendrils_test_file.txt",
        setup.parent_dir.clone(),
        mode
    ).unwrap();
    let dir_tendril = Tendril::new(
        "SomeApp",
        "tendrils_test_dir",
        setup.parent_dir.clone(),
        mode
    ).unwrap();

    let file_actual = action(&setup.td_dir, &file_tendril, dry_run, force);
    let dir_actual = action(&setup.td_dir, &dir_tendril, dry_run, force);

    if action == pull_tendril {
        // No way to create a file at the root for the test. In theory, if a tendril
        // pointed to an existing file at the root this would return successfully
        match file_actual {
            Err(TendrilActionError::IoError(e)) => {
                assert_eq!(e.kind(), std::io::ErrorKind::NotFound);
            },
            _ => panic!("Actual error: {:?}", file_actual),
        }
        if dry_run {
            assert!(matches!(dir_actual, Ok(TendrilActionSuccess::OverwriteSkipped)));
            assert_eq!(setup.local_nested_file_contents(), "Local nested file contents");
        }
        else {
            assert!(matches!(dir_actual, Ok(TendrilActionSuccess::Overwrite)));
            assert_eq!(setup.local_nested_file_contents(), "Remote nested file contents");
        }
    }
    else if dry_run {
        assert!(matches!(file_actual, Ok(TendrilActionSuccess::NewSkipped)));
        assert!(matches!(dir_actual, Ok(TendrilActionSuccess::NewSkipped)));
        assert!(!setup.remote_file.exists());
        assert!(!setup.remote_dir.exists());
    }
    else {
        // File creation at root fails but directory creation is successful
        match file_actual {
            Err(TendrilActionError::IoError(e)) => {
                assert_eq!(e.kind(), std::io::ErrorKind::PermissionDenied);
            },
            _ => panic!("Actual error: {:?}", file_actual),
        }
        assert!(matches!(dir_actual, Ok(TendrilActionSuccess::New)));
        assert!(!setup.remote_file.exists());
        assert_eq!(setup.remote_nested_file_contents(), "Local nested file contents");
    }

    // Cleanup
    remove_file(&setup.remote_file).ok();
    remove_file(&setup.remote_nested_file).ok();
    remove_dir(&setup.remote_dir).ok();
}

#[rstest]
#[case(link_tendril)]
#[case(pull_tendril)]
#[case(push_tendril)]
#[cfg_attr(windows, ignore)]
#[serial]
#[serial("root")]
fn non_windows_platform_parent_is_root_returns_permission_error_unless_dry_run(
    #[case] action: fn(&Path, &Tendril, bool, bool)
        -> Result<TendrilActionSuccess, TendrilActionError>,

    #[values(true, false)]
    dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let mut setup = Setup::new();
    setup.parent_dir = PathBuf::from("/");
    assert_eq!(setup.parent_dir.parent(), None);
    setup.remote_file = setup.parent_dir.join("tendrils_test_file.txt");
    setup.remote_dir = setup.parent_dir.join("tendrils_test_dir");
    setup.remote_nested_file = setup.remote_dir.join("nested.txt");
    setup.local_file = setup.group_dir.join("tendrils_test_file.txt");
    setup.local_dir = setup.group_dir.join("tendrils_test_dir");
    setup.local_nested_file = setup.local_dir.join("nested.txt");
    setup.make_local_file();
    setup.make_local_nested_file();
    let mut mode = TendrilMode::DirOverwrite;
    if action == link_tendril {
        mode = TendrilMode::Link;
    }

    let file_tendril = Tendril::new(
        "SomeApp",
        "tendrils_test_file.txt",
        setup.parent_dir.clone(),
        mode
    ).unwrap();
    let dir_tendril = Tendril::new(
        "SomeApp",
        "tendrils_test_dir",
        setup.parent_dir.clone(),
        mode
    ).unwrap();

    let file_actual = action(&setup.td_dir, &file_tendril, dry_run, force);
    let dir_actual = action(&setup.td_dir, &dir_tendril, dry_run, force);

    if action == pull_tendril {
        match file_actual {
            Err(TendrilActionError::IoError(e)) => {
                assert_eq!(e.kind(), std::io::ErrorKind::NotFound);
            },
            _ => panic!("Actual error: {:?}", file_actual),
        }
        match dir_actual {
            Err(TendrilActionError::IoError(e)) => {
                assert_eq!(e.kind(), std::io::ErrorKind::NotFound);
            },
            _ => panic!("Actual error: {:?}", dir_actual),
        }
        assert_eq!(setup.local_file_contents(), "Local file contents");
        assert_eq!(setup.local_nested_file_contents(), "Local nested file contents");
    }
    else if dry_run {
        assert!(matches!(file_actual, Ok(TendrilActionSuccess::NewSkipped)));
        assert!(matches!(dir_actual, Ok(TendrilActionSuccess::NewSkipped)));
        assert!(!setup.remote_file.exists());
        assert!(!setup.remote_dir.exists());
    }
    else {
        match file_actual {
            Err(TendrilActionError::IoError(e)) => {
                // Possible bug where the std::io::ErrorKind::ReadOnlyFilesystem
                // is only available in nightly but is being returned on Mac
                assert!(format!("{:?}", e).contains("ReadOnlyFilesystem"));
            },
            _ => panic!("Actual error: {:?}", file_actual),
        }
        match dir_actual {
            Err(TendrilActionError::IoError(e)) => {
                assert!(format!("{:?}", e).contains("ReadOnlyFilesystem"));
            },
            _ => panic!("Actual error: {:?}", dir_actual),
        }
        assert!(!setup.remote_file.exists());
        assert!(!setup.remote_dir.exists());
    }

    // Cleanup
    remove_file(&setup.remote_file).ok();
    remove_file(&setup.remote_nested_file).ok();
    remove_dir(&setup.remote_dir).ok();
}

// TODO: Test when path is invalid and a copy is attempted with both a folder and a file (Windows only?)