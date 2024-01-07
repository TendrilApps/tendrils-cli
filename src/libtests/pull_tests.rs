use crate::{pull, PushPullError};
use crate::tendril::Tendril;
use crate::libtests::test_utils::{
    get_disposable_folder,
    is_empty,
    set_all_platform_paths
};
use fs_extra::file::read_to_string;
use std::fs::{create_dir_all, write};
use tempdir::TempDir;

#[test]
fn given_empty_list_returns_empty() {
    let temp_parent_folder = TempDir::new_in(
        get_disposable_folder(),
        "ParentFolder"
    ).unwrap();
    let given_tendrils_folder = temp_parent_folder.path().join("TendrilsFolder");

    let actual = pull(&given_tendrils_folder, &[]);

    assert!(actual.is_empty());
    assert!(is_empty(&given_tendrils_folder))
}

#[test]
fn returns_tendril_and_result_for_each_given() {
    let temp_parent_folder = TempDir::new_in(
        get_disposable_folder(),
        "ParentFolder"
    ).unwrap();
    let given_tendrils_folder = temp_parent_folder.path().join("TendrilsFolder");
    let source_app1_file = temp_parent_folder.path().join("misc1.txt");
    let source_app2_file = temp_parent_folder.path().join("misc2.txt");
    let source_app1_folder = temp_parent_folder.path().join("App1 Folder");
    let nested_app1_file = source_app1_folder.join("nested1.txt");
    create_dir_all(source_app1_folder).unwrap();
    write(source_app1_file, "App 1 file contents").unwrap();
    write(source_app2_file, "App 2 file contents").unwrap();
    write(nested_app1_file, "Nested 1 file contents").unwrap();

    let mut given = [
        Tendril::new("App1", "skip_me.txt"),
        Tendril::new("App1", "misc1.txt"),
        Tendril::new("App2", "misc2.txt"),
        Tendril::new("App1", "App1 Folder"),
        Tendril::new("App2", "I don't exist"),
    ];
    for t in given[1..].iter_mut() {
        set_all_platform_paths(
            t,
            &[temp_parent_folder.path().to_path_buf().clone()]
        );
    }
    let io_not_found_err = std::io::Error::from(std::io::ErrorKind::NotFound);
    let expected: Vec<(&Tendril, Result<(), PushPullError>)> = vec![
        (&given[0], Err(PushPullError::Skipped)),
        (&given[1], Ok(())),
        (&given[2], Ok(())),
        (&given[3], Ok(())),
        (&given[4], Err(PushPullError::IoError(io_not_found_err))),
    ];

    let actual = pull(&given_tendrils_folder, &given);

    assert_eq!(actual.len(), expected.len());
    for (i, exp) in expected.iter().enumerate() {
        assert_eq!(exp.0, actual[i].0);
    }

    // Could not get the error matching working in a loop - manually checking instead
    assert!(matches!(actual[0].1, Err(PushPullError::Skipped)));
    assert!(matches!(actual[1].1, Ok(())));
    assert!(matches!(actual[2].1, Ok(())));
    assert!(matches!(actual[3].1, Ok(())));
    match &actual[4].1 {
        Err(PushPullError::IoError(e)) => {
            assert_eq!(e.kind(), std::io::ErrorKind::NotFound)
        },
        _ => panic!()
    }

    let dest_app1_file_contents = read_to_string(given_tendrils_folder
        .join("App1").join("misc1.txt")).unwrap();
    let dest_app2_file_contents = read_to_string(given_tendrils_folder
        .join("App2").join("misc2.txt")).unwrap();
    let dest_app1_nested_file_contents = read_to_string(given_tendrils_folder
        .join("App1")
        .join("App1 Folder")
        .join("nested1.txt")
    ).unwrap();
    assert_eq!(dest_app1_file_contents, "App 1 file contents");
    assert_eq!(dest_app2_file_contents, "App 2 file contents");
    assert_eq!(dest_app1_nested_file_contents, "Nested 1 file contents");
}

#[test]
fn duplicate_tendrils_returns_duplicate_error_for_second_occurence_onward() {
    let temp_grandparent_folder = TempDir::new_in(
        get_disposable_folder(),
        "GrandparentFolder"
    ).unwrap();
    let given_tendrils_folder = temp_grandparent_folder.path().join("TendrilsFolder");
    let source1 = temp_grandparent_folder.path().join("Parent1").join("misc.txt");
    let source2 = temp_grandparent_folder.path().join("Parent2").join("misc.txt");
    create_dir_all(temp_grandparent_folder.path().join("Parent1")).unwrap();
    create_dir_all(temp_grandparent_folder.path().join("Parent2")).unwrap();
    write(source1, "Source 1 file contents").unwrap();
    write(source2, "Source 2 file contents").unwrap();

    let mut tendril1 = Tendril::new("SomeApp", "misc.txt");
    let mut tendril2 = Tendril::new("SomeApp", "misc.txt");
    let mut tendril3 = Tendril::new("SomeApp", "misc.txt");
    set_all_platform_paths(&mut tendril1, &[temp_grandparent_folder.path().join("Parent1")]);
    set_all_platform_paths(&mut tendril2, &[temp_grandparent_folder.path().join("Parent2")]);
    set_all_platform_paths(&mut tendril3, &[temp_grandparent_folder.path().join("I don't exist")]);
    let given = [tendril1, tendril2, tendril3];

    let actual = pull(&given_tendrils_folder, &given);

    assert_eq!(actual[0].0, &given[0]);
    assert_eq!(actual[1].0, &given[1]);
    assert_eq!(actual[2].0, &given[2]);
    assert!(matches!(actual[0].1, Ok(())));
    assert!(matches!(actual[1].1, Err(PushPullError::Duplicate)));
    assert!(matches!(actual[2].1, Err(PushPullError::Duplicate)));

    let dest_contents = read_to_string(given_tendrils_folder
        .join("SomeApp").join("misc.txt")).unwrap();
    assert_eq!(dest_contents, "Source 1 file contents");
    assert_eq!(given_tendrils_folder.join("SomeApp").read_dir().iter().count(), 1)
}
