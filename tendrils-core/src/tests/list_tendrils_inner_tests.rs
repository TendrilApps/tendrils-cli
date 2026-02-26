//! Contains tests specific to list actions.
//! See also [`crate::tests::common_action_tests`].

use crate::test_utils::{
    set_ra,
    symlink_expose,
    Setup,
};
use crate::{
    FsoType, InvalidTendrilError, ListLog, RawTendril, TendrilMode, TendrilReport, UniPath, list_tendrils_inner
};
use rstest::rstest;
use core::assert_eq;
use core::convert::From;
use std::fs::{remove_file, write};
use std::path::PathBuf;

#[test]
fn empty_tendrils_list_returns_empty_logs() {
    let td_repo = UniPath::from(PathBuf::from("test"));
    let given = vec![];

    let actual = list_tendrils_inner(&td_repo, given);
    assert_eq!(actual, vec![]);
}

#[rstest]
#[case(TendrilMode::Link)]
#[case(TendrilMode::DirMerge)]
#[case(TendrilMode::DirOverwrite)]
fn returns_fso_types_and_resolved_paths_for_all_in_given_order(
    #[case] mode: TendrilMode,
) {
    let setup = Setup::new();
    setup.make_local_file();
    setup.make_local_nested_file();
    setup.make_remote_file();
    setup.make_remote_nested_file();
    setup.make_target_file();
    symlink_expose(&setup.parent_dir.join("misc_link.txt"), &setup.local_file, false, true).unwrap();
    symlink_expose(&setup.parent_dir.join("misc_link"), &setup.local_dir, false, true).unwrap();
    symlink_expose(&setup.parent_dir.join("wrong_link.txt"), &setup.target_file, false, true).unwrap();
    let file_to_del = &setup.td_repo.join("I don't exist");
    write(file_to_del, "").unwrap();
    symlink_expose(&setup.parent_dir.join("missing_link.txt"), &file_to_del, false, true).unwrap();
    remove_file(file_to_del).unwrap();

    let raw_file_tendril = RawTendril {
        local: "SomeApp/misc.txt".to_string(),
        remote: setup.remote_file.to_string_lossy().into(),
        mode,
        profiles: vec![],
    };
    let raw_dir_tendril = RawTendril {
        local: "SomeApp/misc".to_string(),
        remote: setup.remote_dir.to_string_lossy().into(),
        mode,
        profiles: vec!["p3".to_string()],
    };
    let raw_file_link_tendril = RawTendril {
        local: "SomeApp/misc.txt".to_string(),
        remote: setup.parent_dir.join("misc_link.txt").to_string_lossy().into(),
        mode,
        profiles: vec![],
    };
    let raw_dir_link_tendril = RawTendril {
        local: "SomeApp/misc".to_string(),
        remote: setup.parent_dir.join("misc_link").to_string_lossy().into(),
        mode,
        profiles: vec![],
    };
    let raw_wrong_link_tendril = RawTendril {
        local: "SomeApp/misc.txt".to_string(),
        remote: setup.parent_dir.join("wrong_link.txt").to_string_lossy().into(),
        mode,
        profiles: vec![],
    };
    let raw_missing_link_tendril = RawTendril {
        local: "I don't exist".to_string(),
        remote: setup.parent_dir.join("missing_link.txt").to_string_lossy().into(),
        mode,
        profiles: vec![],
    };
    let raw_dne_tendril = RawTendril {
        local: "I don't exist".to_string(),
        remote: setup.parent_dir.join("I don't exist").to_string_lossy().into(),
        mode,
        profiles: vec!["p1".to_string(), "p3".to_string()],
    };
    let raw_invalid_tendril = RawTendril {
        local: "".to_string(),
        remote: setup.parent_dir.join("I don't exist").to_string_lossy().into(),
        mode,
        profiles: vec![],
    };
    let given = vec![
        raw_file_tendril.clone(),
        raw_dir_tendril.clone(),
        raw_file_link_tendril.clone(),
        raw_dir_link_tendril.clone(),
        raw_wrong_link_tendril.clone(),
        raw_missing_link_tendril.clone(),
        raw_dne_tendril.clone(),
        raw_invalid_tendril.clone(),
    ];

    let actual = list_tendrils_inner(&setup.td_repo.clone().into(), given);

    let exp = vec![
        TendrilReport {
            raw_tendril: raw_file_tendril,
            log: Ok(ListLog::new(
                Some(FsoType::File),
                Some(FsoType::File),
                setup.remote_file.clone(),
            )),
        },
        TendrilReport {
            raw_tendril: raw_dir_tendril,
            log: Ok(ListLog::new(
                Some(FsoType::Dir),
                Some(FsoType::Dir),
                setup.remote_dir.clone(),
            )),
        },
        TendrilReport {
            raw_tendril: raw_file_link_tendril,
            log: Ok(ListLog::new(
                Some(FsoType::File),
                Some(FsoType::SymFile),
                setup.parent_dir.join("misc_link.txt").clone(),
            )),
        },
        TendrilReport {
            raw_tendril: raw_dir_link_tendril,
            log: Ok(ListLog::new(
                Some(FsoType::Dir),
                Some(FsoType::SymDir),
                setup.parent_dir.join("misc_link").clone(),
            )),
        },
        TendrilReport {
            raw_tendril: raw_wrong_link_tendril,
            log: Ok(ListLog::new(
                Some(FsoType::File),
                Some(FsoType::SymFile),
                setup.parent_dir.join("wrong_link.txt").clone(),
            )),
        },
        TendrilReport {
            raw_tendril: raw_missing_link_tendril,
            log: Ok(ListLog::new(
                None,
                Some(FsoType::BrokenSym),
                setup.parent_dir.join("missing_link.txt").clone(),
            )),
        },
        TendrilReport {
            raw_tendril: raw_dne_tendril,
            log: Ok(ListLog::new(
                None,
                None,
                setup.parent_dir.join("I don't exist").clone(),
            )),
        },
        TendrilReport {
            raw_tendril: raw_invalid_tendril,
            log: Err(InvalidTendrilError::InvalidLocal),
        },
    ];

    assert_eq!(actual, exp);
    assert_eq!(setup.local_file_contents(), "Local file contents");
    assert_eq!(setup.local_nested_file_contents(), "Local nested file contents");
    assert_eq!(setup.remote_file_contents(), "Remote file contents");
    assert_eq!(setup.remote_nested_file_contents(), "Remote nested file contents");
    assert_eq!(setup.target_file_contents(), "Target file contents");
}

#[rstest]
#[case(TendrilMode::Link)]
#[case(TendrilMode::DirMerge)]
#[case(TendrilMode::DirOverwrite)]
fn no_read_access_from_local_or_remote_file_returns_proper_fso_type(
    #[case] mode: TendrilMode,
) {
    let setup = Setup::new();
    setup.make_td_repo_dir();
    setup.make_remote_nra_file();
    setup.make_local_nra_file();

    let raw_tendril = RawTendril {
        local: "SomeApp/nra.txt".to_string(),
        remote: setup.remote_nra_file.to_string_lossy().into(),
        mode,
        profiles: vec![],
    };
    let given = vec![raw_tendril.clone()];

    let actual = list_tendrils_inner(&setup.td_repo.into(), given);

    let exp = vec![
        TendrilReport {
            raw_tendril: raw_tendril,
            log: Ok(ListLog::new(
                Some(FsoType::File),
                Some(FsoType::File),
                setup.remote_nra_file,
            )),
        },
    ];

    assert_eq!(actual, exp);
}

#[rstest]
#[case(TendrilMode::Link)]
#[case(TendrilMode::DirMerge)]
#[case(TendrilMode::DirOverwrite)]
fn no_read_access_from_local_or_remote_dir_returns_proper_fso_type(
    #[case] mode: TendrilMode,
) {
    let setup = Setup::new();
    setup.make_td_repo_dir();
    setup.make_remote_nra_dir();
    setup.make_local_nra_dir();

    let raw_tendril = RawTendril {
        local: "SomeApp/nra".to_string(),
        remote: setup.remote_nra_dir.to_string_lossy().into(),
        mode,
        profiles: vec![],
    };
    let given = vec![raw_tendril.clone()];

    let actual = list_tendrils_inner(&setup.td_repo.into(), given);
    set_ra(&setup.remote_nra_dir, true);
    set_ra(&setup.local_nra_dir, true);

    let exp = vec![
        TendrilReport {
            raw_tendril: raw_tendril,
            log: Ok(ListLog::new(
                Some(FsoType::Dir),
                Some(FsoType::Dir),
                setup.remote_nra_dir,
            )),
        },
    ];

    assert_eq!(actual, exp);
}
