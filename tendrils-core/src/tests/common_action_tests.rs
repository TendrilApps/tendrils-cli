//! Contains tests that are common to some or all
//! of the tendril actions.
//! See also:
//! - [`crate::tests::link_tendril_tests`]
//! - [`crate::tests::pull_tendril_tests`]
//! - [`crate::tests::push_tendril_tests`]

use crate::path_ext::PathExt;
use crate::test_utils::{
    global_cfg_dir,
    global_cfg_file,
    home_dir,
    symlink_expose,
    Setup,
};
use crate::{
    link_tendril,
    pull_tendril,
    push_tendril,
    ActionLog,
    ActionMode,
    FsoType,
    Location,
    Tendril,
    TendrilActionError,
    TendrilActionSuccess,
    TendrilsActor,
    TendrilsApi,
    TendrilMode,
    UniPath,
};
use crate::tests::tendril_action_tests::compatible_action_and_tendril_modes;
use rstest::rstest;
use rstest_reuse::{self, apply, template};
use serial_test::serial;
use std::fs::{create_dir_all, read_to_string, remove_dir, remove_file, write};
use std::path::PathBuf;

impl ActionMode {
    fn call(&self, t: &Tendril, dry_run: bool, force: bool) -> ActionLog {
        match (&self, t.mode) {
            (ActionMode::Push, TendrilMode::Link) => link_tendril(t, dry_run, force),
            (ActionMode::Push, _) => push_tendril(t, dry_run, force),
            (ActionMode::Pull, _) => pull_tendril(t, dry_run, force),
        }
    }
}

#[apply(compatible_action_and_tendril_modes)]
fn remote_is_sibling_to_given_td_repo_proceeds_normally(
    #[case] action: ActionMode,
    #[case] tendril_mode: TendrilMode,
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let setup = Setup::new();
    setup.make_local_dir();
    setup.make_target_nested_file();
    assert_eq!(
        // Check they are siblings
        setup.remote_dir.parent().unwrap(),
        setup.td_repo.parent().unwrap()
    );

    let exp_remote_type;
    let mut tendril = setup.dir_tendril();
    tendril.mode = tendril_mode;
    if tendril.mode.requires_symlink() {
        symlink_expose(&setup.remote_dir, &setup.target_dir, false, true)
            .unwrap();
        exp_remote_type = Some(FsoType::SymDir);
    }
    else {
        setup.make_remote_dir();
        exp_remote_type = Some(FsoType::Dir);
    }

    let actual = action.call(&tendril, dry_run, force);

    let exp_result;
    if dry_run {
        exp_result = Ok(TendrilActionSuccess::OverwriteSkipped);
    }
    else {
        exp_result = Ok(TendrilActionSuccess::Overwrite);
    }
    assert_eq!(
        actual,
        ActionLog::new(
            Some(FsoType::Dir),
            exp_remote_type,
            setup.remote_dir,
            exp_result,
        )
    );
}

#[apply(compatible_action_and_tendril_modes)]
fn remote_is_another_td_repo_proceeds_normally(
    #[case] action: ActionMode,
    #[case] tendril_mode: TendrilMode,
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let api = TendrilsActor {};
    let setup = Setup::new();
    setup.make_local_nested_file();
    setup.make_remote_nested_file();
    create_dir_all(setup.remote_dir.join(".tendrils")).unwrap();
    write(&setup.remote_dir.join(".tendrils/tendrils.json"), "").unwrap();
    assert!(api.is_tendrils_repo(&UniPath::from(&setup.remote_dir)));

    let mut tendril = setup.dir_tendril();
    tendril.mode = tendril_mode;

    let actual = action.call(&tendril, dry_run, force);

    let exp_result;
    if tendril_mode.requires_symlink() && !force {
        exp_result = Err(TendrilActionError::TypeMismatch {
            loc: Location::Dest,
            mistype: FsoType::Dir,
        })
    }
    else if dry_run {
        exp_result = Ok(TendrilActionSuccess::OverwriteSkipped);
    }
    else {
        exp_result = Ok(TendrilActionSuccess::Overwrite);
    }
    assert_eq!(
        actual,
        ActionLog::new(
            Some(FsoType::Dir),
            Some(FsoType::Dir),
            setup.remote_dir,
            exp_result,
        )
    );
}

#[apply(compatible_action_and_tendril_modes)]
#[serial(SERIAL_MUT_ENV_VARS)]
fn remote_is_global_config_dir_proceeds_normally(
    #[case] action: ActionMode,
    #[case] tendril_mode: TendrilMode,
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let mut setup = Setup::new();
    setup.local_dir = setup.td_repo.join("SomeApp/.tendrils");
    setup.make_global_cfg_file("Global Config Contents".to_string());
    setup.make_local_dir();
    setup.remote_dir = global_cfg_dir();

    let mut tendril = Tendril::new_expose(
        setup.uni_td_repo(),
        "SomeApp/.tendrils".into(),
        home_dir().join(".tendrils").into(),
        TendrilMode::CopyOverwrite,
    ).unwrap();
    tendril.mode = tendril_mode;

    let actual = action.call(&tendril, dry_run, force);

    let exp_result;
    if tendril.mode.requires_symlink() && !force {
        exp_result = Err(TendrilActionError::TypeMismatch {
            loc: Location::Dest,
            mistype: FsoType::Dir,
        })
    }
    else if dry_run {
        exp_result = Ok(TendrilActionSuccess::OverwriteSkipped);
    }
    else {
        exp_result = Ok(TendrilActionSuccess::Overwrite);
    }
    assert_eq!(
        actual,
        ActionLog::new(
            Some(FsoType::Dir),
            Some(FsoType::Dir),
            setup.remote_dir,
            exp_result,
        )
    );
}

#[apply(compatible_action_and_tendril_modes)]
#[serial(SERIAL_MUT_ENV_VARS)]
fn remote_is_in_global_config_dir_proceeds_normally(
    #[case] action: ActionMode,
    #[case] tendril_mode: TendrilMode,
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    // Using global config file here as example
    let mut setup = Setup::new();
    setup.local_file = setup.td_repo.join("SomeApp/global-config.json");
    setup.make_global_cfg_file("Global Config Contents".to_string());
    setup.make_local_file();
    setup.remote_file = global_cfg_file();

    let mut tendril = Tendril::new_expose(
        setup.uni_td_repo(),
        "SomeApp/global-config.json".into(),
        global_cfg_dir().join("global-config.json").into(),
        TendrilMode::CopyOverwrite,
    ).unwrap();
    tendril.mode = tendril_mode;

    let actual = action.call(&tendril, dry_run, force);

    let exp_result;
    if tendril.mode.requires_symlink() && !force {
        exp_result = Err(TendrilActionError::TypeMismatch {
            loc: Location::Dest,
            mistype: FsoType::File,
        })
    }
    else if dry_run {
        exp_result = Ok(TendrilActionSuccess::OverwriteSkipped);
    }
    else {
        exp_result = Ok(TendrilActionSuccess::Overwrite);
    }
    assert_eq!(
        actual,
        ActionLog::new(
            Some(FsoType::File),
            Some(FsoType::File),
            setup.remote_file,
            exp_result,
        )
    );
}

#[apply(compatible_action_and_tendril_modes)]
#[serial(SERIAL_MUT_ENV_VARS)]
fn repo_is_global_cfg_dir_and_config_file_exists_proceeds_normally(
    #[case] action: ActionMode,
    #[case] tendril_mode: TendrilMode,
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let mut setup = Setup::new();
    setup.make_global_cfg_file("Global Config Contents".to_string());
    setup.td_repo = global_cfg_dir();
    setup.group_dir = setup.td_repo.join("SomeApp");
    setup.local_file = setup.group_dir.join("misc.txt");
    setup.make_local_file();
    setup.make_remote_file();

    let mut tendril = setup.file_tendril();
    tendril.mode = tendril_mode;

    let actual = action.call(&tendril, dry_run, force);

    let exp_result;
    if tendril.mode.requires_symlink() && !force {
        exp_result = Err(TendrilActionError::TypeMismatch {
            loc: Location::Dest,
            mistype: FsoType::File,
        })
    }
    else if dry_run {
        exp_result = Ok(TendrilActionSuccess::OverwriteSkipped);
    }
    else {
        exp_result = Ok(TendrilActionSuccess::Overwrite);
    }
    assert_eq!(
        actual,
        ActionLog::new(
            Some(FsoType::File),
            Some(FsoType::File),
            setup.remote_file,
            exp_result,
        )
    );
}

#[apply(compatible_action_and_tendril_modes)]
#[serial(SERIAL_MUT_ENV_VARS)]
fn repo_is_in_global_cfg_dir_and_config_file_exists_proceeds_normally(
    #[case] action: ActionMode,
    #[case] tendril_mode: TendrilMode,
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let mut setup = Setup::new();
    setup.make_global_cfg_file("Global Config Contents".to_string());
    setup.td_repo = global_cfg_dir().join("TendrilsRepo");
    setup.group_dir = setup.td_repo.join("SomeApp");
    setup.local_file = setup.group_dir.join("misc.txt");
    setup.make_local_file();
    setup.make_remote_file();

    let mut tendril = setup.file_tendril();
    tendril.mode = tendril_mode;

    let actual = action.call(&tendril, dry_run, force);

    let exp_result;
    if tendril.mode.requires_symlink() && !force {
        exp_result = Err(TendrilActionError::TypeMismatch {
            loc: Location::Dest,
            mistype: FsoType::File,
        })
    }
    else if dry_run {
        exp_result = Ok(TendrilActionSuccess::OverwriteSkipped);
    }
    else {
        exp_result = Ok(TendrilActionSuccess::Overwrite);
    }
    assert_eq!(
        actual,
        ActionLog::new(
            Some(FsoType::File),
            Some(FsoType::File),
            setup.remote_file,
            exp_result,
        )
    );
}

#[apply(compatible_action_and_tendril_modes)]
#[cfg_attr(windows, ignore)] // These are invalid paths on Windows
#[serial(SERIAL_MUT_ENV_VARS)]
fn var_in_local_uses_raw_path_even_if_var_exists(
    #[values("<mut-testing>", "<I_DO_NOT_EXIST>")] local: &str,
    #[case] action: ActionMode,
    #[case] tendril_mode: TendrilMode,
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let mut setup = Setup::new();
    setup.local_file = setup.td_repo.join(local);
    setup.make_local_file();
    setup.make_target_file();
    std::env::set_var("mut-testing", "NON-EXISTENT PATH");

    let mut tendril = Tendril::new_expose(
        setup.uni_td_repo(),
        local.into(),
        UniPath::from(&setup.remote_file),
        TendrilMode::CopyOverwrite,
    )
    .unwrap();
    tendril.mode = tendril_mode;
    let exp_remote_type;
    if tendril.mode.requires_symlink() {
        symlink_expose(&setup.remote_file, &setup.target_file, false, true)
            .unwrap();
        exp_remote_type = Some(FsoType::SymFile);
    }
    else {
        setup.make_remote_file();
        exp_remote_type = Some(FsoType::File);
    }

    let actual = action.call(&tendril, dry_run, force);

    let exp_result;
    if dry_run {
        exp_result = Ok(TendrilActionSuccess::OverwriteSkipped);
    }
    else {
        exp_result = Ok(TendrilActionSuccess::Overwrite);
    }
    assert_eq!(
        actual,
        ActionLog::new(
            Some(FsoType::File),
            exp_remote_type,
            tendril.remote().inner().to_path_buf(),
            exp_result,
        )
    );
}

#[apply(compatible_action_and_tendril_modes)]
fn other_tendrils_in_same_group_dir_are_unchanged(
    #[case] action: ActionMode,
    #[case] tendril_mode: TendrilMode,
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let setup = Setup::new();
    setup.make_local_file();
    setup.make_local_dir();
    setup.make_target_file();
    setup.make_target_dir();
    let some_other_local_file = &setup.group_dir.join("other.txt");
    let some_other_local_dir = &setup.group_dir.join("other");
    let some_other_local_nested = &setup.group_dir.join("nested.txt");
    create_dir_all(some_other_local_dir).unwrap();
    write(some_other_local_file, "Another tendril from the same group")
        .unwrap();
    write(some_other_local_nested, "Another nested from the same group")
        .unwrap();

    let mut file_tendril = setup.file_tendril();
    let mut dir_tendril = setup.dir_tendril();

    let exp_remote_type_file;
    let exp_remote_type_dir;
    file_tendril.mode = tendril_mode;
    dir_tendril.mode = tendril_mode;
    if tendril_mode.requires_symlink() {
        symlink_expose(&setup.remote_file, &setup.target_file, false, true)
            .unwrap();
        symlink_expose(&setup.remote_dir, &setup.target_dir, false, true)
            .unwrap();
        exp_remote_type_file = Some(FsoType::SymFile);
        exp_remote_type_dir = Some(FsoType::SymDir);
    }
    else {
        setup.make_remote_file();
        setup.make_remote_dir();
        exp_remote_type_file = Some(FsoType::File);
        exp_remote_type_dir = Some(FsoType::Dir);
    }

    let file_actual = action.call(&file_tendril, dry_run, force);
    let dir_actual = action.call(&dir_tendril, dry_run, force);

    // Check that other tendril is unchanged
    let some_other_local_file_contents =
        read_to_string(some_other_local_file).unwrap();
    let some_other_local_nested_contents =
        read_to_string(some_other_local_nested).unwrap();
    assert_eq!(
        some_other_local_file_contents,
        "Another tendril from the same group"
    );
    assert_eq!(
        some_other_local_nested_contents,
        "Another nested from the same group"
    );
    let exp_result;
    if dry_run {
        exp_result = Ok(TendrilActionSuccess::OverwriteSkipped);
    }
    else {
        exp_result = Ok(TendrilActionSuccess::Overwrite);
    }
    assert_eq!(
        file_actual,
        ActionLog::new(
            Some(FsoType::File),
            exp_remote_type_file,
            setup.remote_file,
            exp_result.clone(),
        )
    );
    assert_eq!(
        dir_actual,
        ActionLog::new(
            Some(FsoType::Dir),
            exp_remote_type_dir,
            setup.remote_dir,
            exp_result,
        )
    );
}

#[apply(compatible_action_and_tendril_modes)]
fn other_files_in_subdir_are_unchanged(
    #[case] action: ActionMode,
    #[case] tendril_mode: TendrilMode,
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let setup = Setup::new();
    let other_local_file =
        setup.local_subdir_file.parent().unwrap().join("other.txt");
    let other_remote_file =
        setup.remote_subdir_file.parent().unwrap().join("other.txt");
    setup.make_local_file();
    setup.make_local_dir();
    setup.make_local_subdir_file();
    setup.make_local_subdir_nested_file();
    create_dir_all(setup.parent_dir.join("SubDir")).unwrap();
    write(&other_local_file, "Other local file contents").unwrap();
    write(&other_remote_file, "Other remote file contents").unwrap();

    let mut subdir_file_tendril = setup.subdir_file_tendril();
    let mut subdir_dir_tendril = setup.subdir_dir_tendril();
    subdir_file_tendril.mode = tendril_mode;
    subdir_dir_tendril.mode = tendril_mode;
    if !tendril_mode.requires_symlink() {
        setup.make_remote_subdir_file();
        setup.make_remote_subdir_nested_file();
    }
    let subdir_file_actual =
        action.call(&subdir_file_tendril, dry_run, force);
    let subdir_dir_actual =
        action.call(&subdir_dir_tendril, dry_run, force);

    let exp_result;
    let mut exp_remote_type_file = Some(FsoType::File);
    let mut exp_remote_type_dir = Some(FsoType::Dir);
    if tendril_mode.requires_symlink() {
        if dry_run {
            exp_result = Ok(TendrilActionSuccess::NewSkipped);
        }
        else {
            exp_result = Ok(TendrilActionSuccess::New);
        }
        exp_remote_type_file = None;
        exp_remote_type_dir = None;
    }
    else if dry_run {
        exp_result = Ok(TendrilActionSuccess::OverwriteSkipped);
    }
    else {
        exp_result = Ok(TendrilActionSuccess::Overwrite);
    }

    assert_eq!(
        subdir_file_actual,
        ActionLog::new(
            Some(FsoType::File),
            exp_remote_type_file,
            setup.remote_subdir_file,
            exp_result.clone(),
        )
    );
    assert_eq!(
        subdir_dir_actual,
        ActionLog::new(
            Some(FsoType::Dir),
            exp_remote_type_dir,
            setup.remote_subdir_dir,
            exp_result,
        )
    );
    let other_local_file_contents = read_to_string(other_local_file).unwrap();
    let other_remote_file_contents = read_to_string(other_remote_file).unwrap();
    assert_eq!(other_local_file_contents, "Other local file contents");
    assert_eq!(other_remote_file_contents, "Other remote file contents");
}

#[rstest]
#[case(TendrilMode::Link)]
#[case(TendrilMode::CopyMerge)]
#[case(TendrilMode::CopyOverwrite)]
fn remote_parent_doesnt_exist_creates_anyways(
    #[case] tendril_mode: TendrilMode,
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let mut setup = Setup::new();
    setup.remote_file = setup.parent_dir.join("IDoNotExist1/misc.txt");
    setup.remote_dir = setup.parent_dir.join("IDoNotExist2/misc");
    setup.remote_nested_file = setup.remote_dir.join("nested.txt");
    setup.remote_subdir_file =
        setup.parent_dir.join("IDoNotExist3/SubDir/misc.txt");
    setup.remote_subdir_dir =
        setup.parent_dir.join("IDoNotExist4/SubDir/misc");
    setup.remote_subdir_nested_file =
        setup.remote_subdir_dir.join("nested.txt");
    setup.make_local_file();
    setup.make_local_nested_file();
    setup.make_local_subdir_file();
    setup.make_local_subdir_nested_file();

    let mut file_tendril = Tendril::new_expose(
        &setup.uni_td_repo(),
        "SomeApp/misc.txt".into(),
        UniPath::from(&setup.remote_file),
        TendrilMode::CopyOverwrite,
    ).unwrap();
    let mut dir_tendril = Tendril::new_expose(
        &setup.uni_td_repo(),
        "SomeApp/misc".into(),
        UniPath::from(&setup.remote_dir),
        TendrilMode::CopyOverwrite,
    ).unwrap();
    let mut subdir_file_tendril = Tendril::new_expose(
        &setup.uni_td_repo(),
        "SomeApp/SubDir/misc.txt".into(),
        UniPath::from(&setup.remote_subdir_file),
        TendrilMode::CopyOverwrite,
    ).unwrap();
    let mut subdir_dir_tendril = Tendril::new_expose(
        &setup.uni_td_repo(),
        "SomeApp/SubDir/misc".into(),
        UniPath::from(&setup.remote_subdir_dir),
        TendrilMode::CopyOverwrite,
    ).unwrap();
    file_tendril.mode = tendril_mode;
    dir_tendril.mode = tendril_mode;
    subdir_file_tendril.mode = tendril_mode;
    subdir_dir_tendril.mode = tendril_mode;
    assert!(!file_tendril.remote().inner().parent().unwrap().exists());
    assert!(!dir_tendril.remote().inner().parent().unwrap().exists());
    assert!(!subdir_file_tendril.remote().inner().parent().unwrap().exists());
    assert!(!subdir_dir_tendril.remote().inner().parent().unwrap().exists());

    let action = ActionMode::Push;
    let file_actual = action.call(&file_tendril, dry_run, force);
    let dir_actual = action.call(&dir_tendril, dry_run, force);
    let subdir_file_actual =
        action.call(&subdir_file_tendril, dry_run, force);
    let subdir_dir_actual =
        action.call(&subdir_dir_tendril, dry_run, force);

    let exp_result;
    if dry_run {
        exp_result = Ok(TendrilActionSuccess::NewSkipped);
    }
    else {
        exp_result = Ok(TendrilActionSuccess::New);
    }
    assert_eq!(
        file_actual,
        ActionLog::new(
            Some(FsoType::File),
            None,
            setup.remote_file.clone(),
            exp_result.clone(),
        )
    );
    assert_eq!(
        dir_actual,
        ActionLog::new(
            Some(FsoType::Dir),
            None,
            setup.remote_dir.clone(),
            exp_result.clone(),
        )
    );
    assert_eq!(
        subdir_file_actual,
        ActionLog::new(
            Some(FsoType::File),
            None,
            setup.remote_subdir_file.clone(),
            exp_result.clone(),
        )
    );
    assert_eq!(
        subdir_dir_actual,
        ActionLog::new(
            Some(FsoType::Dir),
            None,
            setup.remote_subdir_dir.clone(),
            exp_result,
        )
    );
    assert_eq!(setup.local_file_contents(), "Local file contents");
    assert_eq!(
        setup.local_nested_file_contents(),
        "Local nested file contents"
    );
    assert_eq!(
        setup.local_subdir_file_contents(),
        "Local subdir file contents"
    );
    assert_eq!(
        setup.local_subdir_nested_file_contents(),
        "Local subdir nested file contents"
    );
    if dry_run {
        assert!(!setup.remote_file.exists());
        assert!(!setup.remote_dir.exists());
        assert!(!setup.remote_subdir_file.exists());
        assert!(!setup.remote_subdir_dir.exists());
    }
    else {
        assert_eq!(setup.remote_file_contents(), "Local file contents");
        assert_eq!(
            setup.remote_nested_file_contents(),
            "Local nested file contents"
        );
        assert_eq!(
            setup.remote_subdir_file_contents(),
            "Local subdir file contents"
        );
        assert_eq!(
            setup.remote_subdir_nested_file_contents(),
            "Local subdir nested file contents"
        );
    }
}

#[rstest]
#[case(TendrilMode::Link)]
#[case(TendrilMode::CopyMerge)]
#[case(TendrilMode::CopyOverwrite)]
fn remote_direct_parent_doesnt_exist_but_parent_does_should_create_subdirs_then_succeed(
    #[case] tendril_mode: TendrilMode,
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let subdir_file_setup = Setup::new();
    let subdir_dir_setup = Setup::new();
    subdir_file_setup.make_local_subdir_file();
    subdir_dir_setup.make_local_subdir_nested_file();

    let mut subdir_file_tendril = subdir_file_setup.subdir_file_tendril();
    let mut subdir_dir_tendril = subdir_dir_setup.subdir_dir_tendril();
    subdir_file_tendril.mode = tendril_mode;
    subdir_dir_tendril.mode = tendril_mode;
    assert!(!subdir_file_tendril.remote().inner().parent().unwrap().exists());
    assert!(!subdir_dir_tendril.remote().inner().parent().unwrap().exists());

    let action = ActionMode::Push;
    let subdir_file_actual = action.call(&subdir_file_tendril, dry_run, force);
    let subdir_dir_actual = action.call(&subdir_dir_tendril, dry_run, force);

    let exp_result;
    if action == ActionMode::Pull {
        exp_result = Err(TendrilActionError::IoError {
            kind: std::io::ErrorKind::NotFound,
            loc: Location::Source,
        });
    }
    else if dry_run {
        exp_result = Ok(TendrilActionSuccess::NewSkipped);
        assert_eq!(
            subdir_file_setup.local_subdir_file_contents(),
            "Local subdir file contents"
        );
        assert_eq!(
            subdir_dir_setup.local_subdir_nested_file_contents(),
            "Local subdir nested file contents"
        );
        assert!(!subdir_file_setup.remote_subdir_file.exists());
        assert!(!subdir_dir_setup.remote_subdir_dir.exists());
    }
    else {
        exp_result = Ok(TendrilActionSuccess::New);
        assert_eq!(
            subdir_file_setup.local_subdir_file_contents(),
            "Local subdir file contents"
        );
        assert_eq!(
            subdir_dir_setup.local_subdir_nested_file_contents(),
            "Local subdir nested file contents"
        );
        assert_eq!(
            subdir_file_setup.remote_subdir_file_contents(),
            "Local subdir file contents"
        );
        assert_eq!(
            subdir_dir_setup.remote_subdir_nested_file_contents(),
            "Local subdir nested file contents"
        );
    }
    assert_eq!(
        subdir_file_actual,
        ActionLog::new(
            Some(FsoType::File),
            None,
            subdir_file_setup.remote_subdir_file,
            exp_result.clone(),
        )
    );
    assert_eq!(
        subdir_dir_actual,
        ActionLog::new(
            Some(FsoType::Dir),
            None,
            subdir_dir_setup.remote_subdir_dir,
            exp_result,
        )
    );
}

#[template]
#[rstest]
#[case(ActionMode::Pull, TendrilMode::CopyMerge, true)]
#[case(ActionMode::Pull, TendrilMode::CopyOverwrite, true)]
#[case(ActionMode::Push, TendrilMode::CopyMerge, true)]
#[case(ActionMode::Push, TendrilMode::CopyMerge, false)]
#[case(ActionMode::Push, TendrilMode::CopyOverwrite, true)]
#[case(ActionMode::Push, TendrilMode::CopyOverwrite, false)]
#[case(ActionMode::Push, TendrilMode::Link, true)]
#[case(ActionMode::Push, TendrilMode::Link, false)]
fn cases_that_do_not_modify_local(
    #[case] action: ActionMode,
    #[case] tendril_mode: TendrilMode,
    #[case] dry_run: bool,
) {
}

#[apply(cases_that_do_not_modify_local)]
fn local_is_unchanged(
    #[case] action: ActionMode,
    #[case] tendril_mode: TendrilMode,
    #[case] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let setup = Setup::new();
    setup.make_local_file();
    setup.make_local_nested_file();
    setup.make_target_file();
    setup.make_target_nested_file();

    let mut file_tendril = setup.file_tendril();
    let mut dir_tendril = setup.dir_tendril();

    let exp_remote_type_file;
    let exp_remote_type_dir;
    file_tendril.mode = tendril_mode;
    dir_tendril.mode = tendril_mode;
    if tendril_mode.requires_symlink() {
        symlink_expose(&setup.remote_file, &setup.target_file, false, true)
            .unwrap();
        symlink_expose(&setup.remote_dir, &setup.target_dir, false, true)
            .unwrap();
        exp_remote_type_file = Some(FsoType::SymFile);
        exp_remote_type_dir = Some(FsoType::SymDir);
    }
    else {
        setup.make_remote_file();
        setup.make_remote_nested_file();
        exp_remote_type_file = Some(FsoType::File);
        exp_remote_type_dir = Some(FsoType::Dir);
    }

    let file_actual = action.call(&file_tendril, dry_run, force);
    let dir_actual = action.call(&dir_tendril, dry_run, force);

    let exp_result;
    if dry_run {
        exp_result = Ok(TendrilActionSuccess::OverwriteSkipped);
    }
    else {
        exp_result = Ok(TendrilActionSuccess::Overwrite);
    }
    assert_eq!(
        file_actual,
        ActionLog::new(
            Some(FsoType::File),
            exp_remote_type_file,
            setup.remote_file.clone(),
            exp_result.clone(),
        )
    );
    assert_eq!(
        dir_actual,
        ActionLog::new(
            Some(FsoType::Dir),
            exp_remote_type_dir,
            setup.remote_dir.clone(),
            exp_result,
        )
    );
    assert_eq!(setup.local_file_contents(), "Local file contents");
    assert_eq!(
        setup.local_nested_file_contents(),
        "Local nested file contents"
    );
}

#[apply(cases_that_do_not_modify_local)]
fn local_symlink_is_unchanged(
    #[case] action: ActionMode,
    #[case] tendril_mode: TendrilMode,
    #[case] dry_run: bool,
    #[values(true, false)] force: bool,
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

    let exp_remote_type_file;
    let exp_remote_type_dir;
    file_tendril.mode = tendril_mode;
    dir_tendril.mode = tendril_mode;
    if tendril_mode.requires_symlink() {
        // Setup symlinks at remote to prevent unintended
        // type mismatch here during links
        symlink_expose(&setup.remote_file, &setup.target_file, false, true)
            .unwrap();
        symlink_expose(&setup.remote_dir, &setup.target_dir, false, true)
            .unwrap();
        exp_remote_type_file = Some(FsoType::SymFile);
        exp_remote_type_dir = Some(FsoType::SymDir);
    }
    else {
        exp_remote_type_file = Some(FsoType::File);
        exp_remote_type_dir = Some(FsoType::Dir);
    }

    let file_actual = action.call(&file_tendril, dry_run, force);
    let dir_actual = action.call(&dir_tendril, dry_run, force);

    let exp_loc = match action == ActionMode::Pull {
        true => Location::Dest,
        false => Location::Source,
    };
    let exp_file_result;
    let exp_dir_result;
    if !force {
        exp_file_result = Err(TendrilActionError::TypeMismatch {
            loc: exp_loc.clone(),
            mistype: FsoType::SymFile,
        });
        exp_dir_result = Err(TendrilActionError::TypeMismatch {
            loc: exp_loc,
            mistype: FsoType::SymDir,
        });
    }
    else if dry_run {
        exp_file_result = Ok(TendrilActionSuccess::OverwriteSkipped);
        exp_dir_result = Ok(TendrilActionSuccess::OverwriteSkipped);
    }
    else {
        exp_file_result = Ok(TendrilActionSuccess::Overwrite);
        exp_dir_result = Ok(TendrilActionSuccess::Overwrite);
    }
    assert_eq!(
        file_actual,
        ActionLog::new(
            Some(FsoType::SymFile),
            exp_remote_type_file,
            setup.remote_file.clone(),
            exp_file_result,
        )
    );
    assert_eq!(
        dir_actual,
        ActionLog::new(
            Some(FsoType::SymDir),
            exp_remote_type_dir,
            setup.remote_dir.clone(),
            exp_dir_result,
        )
    );
    assert!(setup.local_file.is_symlink());
    assert!(setup.local_dir.is_symlink());
    assert_eq!(setup.local_file_contents(), "Target file contents");
    assert_eq!(
        setup.local_nested_file_contents(),
        "Target nested file contents"
    );
}

#[template]
#[rstest]
#[case(ActionMode::Pull, TendrilMode::CopyMerge, true)]
#[case(ActionMode::Pull, TendrilMode::CopyMerge, false)]
#[case(ActionMode::Pull, TendrilMode::CopyOverwrite, true)]
#[case(ActionMode::Pull, TendrilMode::CopyOverwrite, false)]
#[case(ActionMode::Push, TendrilMode::CopyMerge, true)]
#[case(ActionMode::Push, TendrilMode::CopyOverwrite, true)]
#[case(ActionMode::Push, TendrilMode::Link, true)]
fn cases_that_do_not_modify_remote(
    #[case] action: ActionMode,
    #[case] tendril_mode: TendrilMode,
    #[case] dry_run: bool,
) {
}

#[apply(cases_that_do_not_modify_remote)]
fn remote_is_unchanged(
    #[case] action: ActionMode,
    #[case] tendril_mode: TendrilMode,
    #[case] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let setup = Setup::new();
    setup.make_local_file();
    setup.make_local_nested_file();
    setup.make_remote_file();
    setup.make_remote_nested_file();

    let mut file_tendril = setup.file_tendril();
    let mut dir_tendril = setup.dir_tendril();
    file_tendril.mode = tendril_mode;
    dir_tendril.mode = tendril_mode;

    let file_actual = action.call(&file_tendril, dry_run, force);
    let dir_actual = action.call(&dir_tendril, dry_run, force);

    let exp_file_result;
    let exp_dir_result;
    if !force && tendril_mode.requires_symlink() {
        exp_file_result = Err(TendrilActionError::TypeMismatch {
            loc: Location::Dest,
            mistype: FsoType::File,
        });
        exp_dir_result = Err(TendrilActionError::TypeMismatch {
            loc: Location::Dest,
            mistype: FsoType::Dir,
        });
    }
    else if dry_run {
        exp_file_result = Ok(TendrilActionSuccess::OverwriteSkipped);
        exp_dir_result = Ok(TendrilActionSuccess::OverwriteSkipped);
    }
    else {
        exp_file_result = Ok(TendrilActionSuccess::Overwrite);
        exp_dir_result = Ok(TendrilActionSuccess::Overwrite);
    }
    assert_eq!(
        file_actual,
        ActionLog::new(
            Some(FsoType::File),
            Some(FsoType::File),
            setup.remote_file.clone(),
            exp_file_result,
        )
    );
    assert_eq!(
        dir_actual,
        ActionLog::new(
            Some(FsoType::Dir),
            Some(FsoType::Dir),
            setup.remote_dir.clone(),
            exp_dir_result,
        )
    );
    assert_eq!(setup.remote_file_contents(), "Remote file contents");
    assert_eq!(
        setup.remote_nested_file_contents(),
        "Remote nested file contents"
    );
}

#[apply(cases_that_do_not_modify_remote)]
fn remote_symlink_is_unchanged(
    #[case] action: ActionMode,
    #[case] tendril_mode: TendrilMode,
    #[case] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let setup = Setup::new();
    setup.make_local_file();
    setup.make_local_nested_file();
    setup.make_target_file();
    setup.make_target_nested_file();
    symlink_expose(&setup.remote_file, &setup.target_file, false, true)
        .unwrap();
    symlink_expose(&setup.remote_dir, &setup.target_dir, false, true).unwrap();

    let mut file_tendril = setup.file_tendril();
    let mut dir_tendril = setup.dir_tendril();
    file_tendril.mode = tendril_mode;
    dir_tendril.mode = tendril_mode;

    let file_actual = action.call(&file_tendril, dry_run, force);
    let dir_actual = action.call(&dir_tendril, dry_run, force);

    let exp_file_result;
    let exp_dir_result;
    if !force && !tendril_mode.requires_symlink() {
        let exp_loc = match action == ActionMode::Pull {
            true => Location::Source,
            false => Location::Dest,
        };
        exp_file_result = Err(TendrilActionError::TypeMismatch {
            loc: exp_loc.clone(),
            mistype: FsoType::SymFile,
        });
        exp_dir_result = Err(TendrilActionError::TypeMismatch {
            loc: exp_loc.clone(),
            mistype: FsoType::SymDir,
        });
    }
    else if dry_run {
        exp_file_result = Ok(TendrilActionSuccess::OverwriteSkipped);
        exp_dir_result = Ok(TendrilActionSuccess::OverwriteSkipped);
    }
    else {
        exp_file_result = Ok(TendrilActionSuccess::Overwrite);
        exp_dir_result = Ok(TendrilActionSuccess::Overwrite);
    }
    assert_eq!(
        file_actual,
        ActionLog::new(
            Some(FsoType::File),
            Some(FsoType::SymFile),
            setup.remote_file.clone(),
            exp_file_result,
        )
    );
    assert_eq!(
        dir_actual,
        ActionLog::new(
            Some(FsoType::Dir),
            Some(FsoType::SymDir),
            setup.remote_dir.clone(),
            exp_dir_result,
        )
    );
    assert!(setup.remote_file.is_symlink());
    assert!(setup.remote_dir.is_symlink());
    assert_eq!(setup.remote_file_contents(), "Target file contents");
    assert_eq!(
        setup.remote_nested_file_contents(),
        "Target nested file contents"
    );
}

#[apply(compatible_action_and_tendril_modes)]
fn remote_is_broken_symlink_treats_as_if_it_doesnt_exist_if_forced_except_for_pull(
    #[case] action: ActionMode,
    #[case] tendril_mode: TendrilMode,
    #[values (true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let setup = Setup::new();
    setup.make_local_file();
    setup.make_local_nested_file();
    setup.make_target_file();
    setup.make_target_dir();
    symlink_expose(&setup.remote_file, &setup.target_file, false, true)
        .unwrap();
    symlink_expose(&setup.remote_dir, &setup.target_dir, false, true).unwrap();
    remove_file(&setup.target_file).unwrap();
    remove_dir(&setup.target_dir).unwrap();
    assert_eq!(&setup.remote_file.get_type(), &Some(FsoType::BrokenSym));
    assert_eq!(&setup.remote_dir.get_type(), &Some(FsoType::BrokenSym));

    let mut file_tendril = setup.file_tendril();
    let mut dir_tendril = setup.dir_tendril();
    file_tendril.mode = tendril_mode;
    dir_tendril.mode = tendril_mode;

    let file_actual = action.call(&file_tendril, dry_run, force);
    let dir_actual = action.call(&dir_tendril, dry_run, force);

    let exp_file_result;
    let exp_dir_result;
    if !force && action == ActionMode::Push && !tendril_mode.requires_symlink() {
        exp_file_result = Err(TendrilActionError::TypeMismatch {
            loc: Location::Dest,
            mistype: FsoType::BrokenSym,
        });
        exp_dir_result = Err(TendrilActionError::TypeMismatch {
            loc: Location::Dest,
            mistype: FsoType::BrokenSym,
        });
    }
    else if action == ActionMode::Pull {
        exp_file_result = Err(TendrilActionError::IoError {
            kind: std::io::ErrorKind::NotFound,
            loc: Location::Source,
        });
        exp_dir_result = Err(TendrilActionError::IoError {
            kind: std::io::ErrorKind::NotFound,
            loc: Location::Source,
        });
    }
    else if dry_run {
        exp_file_result = Ok(TendrilActionSuccess::OverwriteSkipped);
        exp_dir_result = Ok(TendrilActionSuccess::OverwriteSkipped);
        assert_eq!(&setup.remote_file.get_type(), &Some(FsoType::BrokenSym));
        assert_eq!(&setup.remote_dir.get_type(), &Some(FsoType::BrokenSym));
    }
    else {
        exp_file_result = Ok(TendrilActionSuccess::Overwrite);
        exp_dir_result = Ok(TendrilActionSuccess::Overwrite);
        assert_eq!(setup.remote_file_contents(), "Local file contents");
        assert_eq!(setup.remote_nested_file_contents(), "Local nested file contents");
    }
    assert_eq!(
        file_actual,
        ActionLog::new(
            Some(FsoType::File),
            Some(FsoType::BrokenSym),
            setup.remote_file.clone(),
            exp_file_result,
        )
    );
    assert_eq!(
        dir_actual,
        ActionLog::new(
            Some(FsoType::Dir),
            Some(FsoType::BrokenSym),
            setup.remote_dir.clone(),
            exp_dir_result,
        )
    );
}

#[apply(compatible_action_and_tendril_modes)]
fn local_is_broken_symlink_treats_as_if_it_doesnt_exist_if_forced_except_for_copy_style_push(
    #[case] action: ActionMode,
    #[case] tendril_mode: TendrilMode,
    #[values (true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let setup = Setup::new();
    setup.make_remote_file();
    setup.make_remote_nested_file();
    setup.make_target_file();
    setup.make_target_dir();
    symlink_expose(&setup.local_file, &setup.target_file, false, true)
        .unwrap();
    symlink_expose(&setup.local_dir, &setup.target_dir, false, true).unwrap();
    remove_file(&setup.target_file).unwrap();
    remove_dir(&setup.target_dir).unwrap();
    assert_eq!(&setup.local_file.get_type(), &Some(FsoType::BrokenSym));
    assert_eq!(&setup.local_dir.get_type(), &Some(FsoType::BrokenSym));

    let mut file_tendril = setup.file_tendril();
    let mut dir_tendril = setup.dir_tendril();
    file_tendril.mode = tendril_mode;
    dir_tendril.mode = tendril_mode;

    let file_actual = action.call(&file_tendril, dry_run, force);
    let dir_actual = action.call(&dir_tendril, dry_run, force);

    let exp_file_result;
    let exp_dir_result;
    if !force && (action == ActionMode::Pull || tendril_mode.requires_symlink()) {
        let exp_loc;
        if tendril_mode.requires_symlink() {
            exp_loc = Location::Source;
        }
        else {
            exp_loc = Location::Dest;
        }
        exp_file_result = Err(TendrilActionError::TypeMismatch {
            loc: exp_loc.clone(),
            mistype: FsoType::BrokenSym,
        });
        exp_dir_result = Err(TendrilActionError::TypeMismatch {
            loc: exp_loc,
            mistype: FsoType::BrokenSym,
        });
    }
    else if action == ActionMode::Push && !tendril_mode.requires_symlink() {
        exp_file_result = Err(TendrilActionError::IoError {
            kind: std::io::ErrorKind::NotFound,
            loc: Location::Source,
        });
        exp_dir_result = Err(TendrilActionError::IoError {
            kind: std::io::ErrorKind::NotFound,
            loc: Location::Source,
        });
    }
    else if dry_run {
        exp_file_result = Ok(TendrilActionSuccess::OverwriteSkipped);
        exp_dir_result = Ok(TendrilActionSuccess::OverwriteSkipped);
        assert_eq!(&setup.local_file.get_type(), &Some(FsoType::BrokenSym));
        assert_eq!(&setup.local_dir.get_type(), &Some(FsoType::BrokenSym));
    }
    else {
        exp_file_result = Ok(TendrilActionSuccess::Overwrite);
        exp_dir_result = Ok(TendrilActionSuccess::Overwrite);
        assert_eq!(setup.local_file_contents(), "Remote file contents");
        assert_eq!(setup.local_nested_file_contents(), "Remote nested file contents");
    }
    assert_eq!(
        file_actual,
        ActionLog::new(
            Some(FsoType::BrokenSym),
            Some(FsoType::File),
            setup.remote_file.clone(),
            exp_file_result,
        )
    );
    assert_eq!(
        dir_actual,
        ActionLog::new(
            Some(FsoType::BrokenSym),
            Some(FsoType::Dir),
            setup.remote_dir.clone(),
            exp_dir_result,
        )
    );
}

#[apply(compatible_action_and_tendril_modes)]
fn current_dir_is_unchanged(
    #[case] action: ActionMode,
    #[case] tendril_mode: TendrilMode,
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let setup = Setup::new();
    setup.make_td_repo_dir();

    let orig_cd = std::env::current_dir().unwrap();

    let mut file_tendril = setup.file_tendril();
    let mut dir_tendril = setup.dir_tendril();
    file_tendril.mode = tendril_mode;
    dir_tendril.mode = tendril_mode;

    let file_actual = action.call(&file_tendril, dry_run, force);
    let dir_actual = action.call(&dir_tendril, dry_run, force);

    let exp_result = Err(TendrilActionError::IoError {
        kind: std::io::ErrorKind::NotFound,
        loc: Location::Source,
    });
    assert_eq!(
        file_actual,
        ActionLog::new(
            None,
            None,
            setup.remote_file.clone(),
            exp_result.clone(),
        )
    );
    assert_eq!(
        dir_actual,
        ActionLog::new(None, None, setup.remote_dir.clone(), exp_result,)
    );
    assert_eq!(std::env::current_dir().unwrap(), orig_cd);
}

// Only run these tests in a container as they require a specific setup and may
// affect locations outside this repo
mod admin_container {
    use super::*;

    fn assert_admin_container() {
        // See Dockerfile for building test env
        if std::env::var("TENDRILS_TEST_CONTAINER") != Ok("true".to_string()) {
            panic!("This test should only run in a specific test container");
        }
    }

    #[apply(compatible_action_and_tendril_modes)]
    #[serial(SERIAL_ROOT)]
    #[cfg_attr(any(not(target_os = "linux"), not(feature = "_admin_tests")), ignore)]
    pub fn remote_parent_is_root_returns_success_if_admin(
        #[case] action: ActionMode,
        #[case] tendril_mode: TendrilMode,
        #[values(true, false)] dry_run: bool,
        #[values(true, false)] force: bool,
    ) {
        assert_admin_container();

        let mut setup = Setup::new();
        setup.remote_file = PathBuf::from("/tendrils_test_file.txt");
        setup.remote_dir = PathBuf::from("/tendrils_test_dir");
        setup.remote_nested_file = setup.remote_dir.join("nested.txt");
        setup.local_file = setup.group_dir.join("tendrils_test_file.txt");
        setup.local_dir = setup.group_dir.join("tendrils_test_dir");
        setup.local_nested_file = setup.local_dir.join("nested.txt");
        setup.make_local_file();
        setup.make_local_nested_file();
        let mut exp_remote_type_file = Some(FsoType::File);
        let mut exp_remote_type_dir = Some(FsoType::Dir);

        if tendril_mode.requires_symlink() {
            setup.make_target_file();
            setup.make_target_nested_file();

            exp_remote_type_file = Some(FsoType::SymFile);
            exp_remote_type_dir = Some(FsoType::SymDir);
            symlink_expose(&setup.remote_file, &setup.target_file, false, true).unwrap();
            symlink_expose(&setup.remote_dir, &setup.target_dir, false, true).unwrap();
        }
        else {
            setup.make_remote_file();
            setup.make_remote_nested_file();
        }

        let file_tendril = Tendril::new_expose(
            setup.uni_td_repo(),
            "SomeApp/tendrils_test_file.txt".into(),
            UniPath::from(&setup.remote_file),
            tendril_mode,
        )
        .unwrap();
        let dir_tendril = Tendril::new_expose(
            setup.uni_td_repo(),
            "SomeApp/tendrils_test_dir".into(),
            UniPath::from(&setup.remote_dir),
            tendril_mode,
        )
        .unwrap();

        let file_actual = action.call(&file_tendril, dry_run, force);
        let dir_actual = action.call(&dir_tendril, dry_run, force);

        let exp_file_result;
        let exp_dir_result;
        if dry_run {
            exp_file_result = Ok(TendrilActionSuccess::OverwriteSkipped);
            exp_dir_result = Ok(TendrilActionSuccess::OverwriteSkipped);
        }
        else {
            exp_file_result = Ok(TendrilActionSuccess::Overwrite);
            exp_dir_result = Ok(TendrilActionSuccess::Overwrite);
        }

        if action == ActionMode::Pull {
            if dry_run {
                assert_eq!(
                    setup.local_file_contents(),
                    "Local file contents"
                );
                assert_eq!(
                    setup.local_nested_file_contents(),
                    "Local nested file contents"
                );
            }
            else {
                assert_eq!(
                    setup.local_file_contents(),
                    "Remote file contents"
                );
                assert_eq!(
                    setup.local_nested_file_contents(),
                    "Remote nested file contents"
                );
            }
        }
        if tendril_mode.requires_symlink() {
            if dry_run {
                assert_eq!(
                    setup.remote_file_contents(),
                    "Target file contents"
                );
                assert_eq!(
                    setup.remote_nested_file_contents(),
                    "Target nested file contents"
                );
            }
            else {
                assert_eq!(
                    setup.remote_file_contents(),
                    "Local file contents"
                );
                assert_eq!(
                    setup.remote_nested_file_contents(),
                    "Local nested file contents"
                );
            }
        }
        if action == ActionMode::Push {
            if dry_run {
                assert_eq!(
                    setup.remote_file_contents(),
                    "Remote file contents"
                );
                assert_eq!(
                    setup.remote_nested_file_contents(),
                    "Remote nested file contents"
                );
            }
            else {
                assert_eq!(
                    setup.remote_file_contents(),
                    "Local file contents"
                );
                assert_eq!(
                    setup.remote_nested_file_contents(),
                    "Local nested file contents"
                );
            }
        }

        assert_eq!(
            file_actual,
            ActionLog::new(
                Some(FsoType::File),
                exp_remote_type_file,
                PathBuf::from("/tendrils_test_file.txt"),
                exp_file_result,
            )
        );
        assert_eq!(
            dir_actual,
            ActionLog::new(
                Some(FsoType::Dir),
                exp_remote_type_dir,
                PathBuf::from("/tendrils_test_dir"),
                exp_dir_result,
            )
        );

        // Cleanup
        remove_file(&setup.remote_file).unwrap();
        remove_file(&setup.remote_nested_file).unwrap();
        if tendril_mode.requires_symlink() && std::env::consts::OS == "linux" {
            // Linux treats symlinks to directories as files
            remove_file(&setup.remote_dir).unwrap();
        }
        else {
            remove_dir(&setup.remote_dir).unwrap();
        }
    }

    // TODO: Remote IS root
}

// TODO: Test when path is invalid and a copy is attempted with both a folder
// and a file (Windows only?)
// TODO: Test when td_repo is in the ~/.tendrils folder and global config dir exists
// TODO: Test when td_repo is the ~/.tendrils folder and global config dir exists
