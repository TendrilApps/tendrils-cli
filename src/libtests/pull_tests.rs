use crate::resolved_tendril::TendrilMode;
use crate::{pull, PushPullError};
use crate::resolved_tendril::ResolvedTendril;
use crate::test_utils::{get_disposable_folder, is_empty};
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
    let given_parent_folder = temp_parent_folder.path().to_path_buf();
    let source_app1_file = given_parent_folder.join("misc1.txt");
    let source_app2_file = given_parent_folder.join("misc2.txt");
    let source_app1_folder = given_parent_folder.join("App1 Folder");
    let nested_app1_file = source_app1_folder.join("nested1.txt");
    create_dir_all(source_app1_folder).unwrap();
    write(&source_app1_file, "App 1 file contents").unwrap();
    write(&source_app2_file, "App 2 file contents").unwrap();
    write(&nested_app1_file, "Nested 1 file contents").unwrap();

    let given = [
        ResolvedTendril::new(
            "App1".to_string(), "misc1.txt".to_string(),
            given_parent_folder.clone(), TendrilMode::FolderOverwrite,
        ).unwrap(),
        ResolvedTendril::new(
            "App2".to_string(), "misc2.txt".to_string(),
            given_parent_folder.clone(), TendrilMode::FolderOverwrite,
        ).unwrap(),
        ResolvedTendril::new(
            "App1".to_string(), "App1 Folder".to_string(),
            given_parent_folder.clone(), TendrilMode::FolderOverwrite,
        ).unwrap(),
        ResolvedTendril::new(
            "App2".to_string(), "I don't exist".to_string(),
            given_parent_folder.clone(), TendrilMode::FolderOverwrite,
        ).unwrap(),
    ];
    let io_not_found_err = std::io::Error::from(std::io::ErrorKind::NotFound);
    let expected: Vec<(&ResolvedTendril, Result<(), PushPullError>)> = vec![
        (&given[0], Ok(())),
        (&given[1], Ok(())),
        (&given[2], Ok(())),
        (&given[3], Err(PushPullError::IoError(io_not_found_err))),
    ];

    let actual = pull(&given_tendrils_folder, &given);

    assert_eq!(actual.len(), expected.len());
    for (i, exp) in expected.iter().enumerate() {
        assert_eq!(exp.0, actual[i].0);
    }

    // Could not get the error matching working in a loop - manually checking instead
    assert!(matches!(actual[0].1, Ok(())));
    assert!(matches!(actual[1].1, Ok(())));
    assert!(matches!(actual[2].1, Ok(())));
    match &actual[3].1 {
        Err(PushPullError::IoError(e)) => {
            assert_eq!(e.kind(), std::io::ErrorKind::NotFound)
        },
        _ => panic!()
    }

    let dest_app1_file_contents = read_to_string(source_app1_file).unwrap();
    let dest_app2_file_contents = read_to_string(source_app2_file).unwrap();
    let dest_app1_nested_file_contents = read_to_string(given_tendrils_folder
        .join("App1")
        .join("App1 Folder")
        .join("nested1.txt")
    ).unwrap();
    assert_eq!(dest_app1_file_contents, "App 1 file contents");
    assert_eq!(dest_app2_file_contents, "App 2 file contents");
    assert_eq!(dest_app1_nested_file_contents, "Nested 1 file contents");
}
