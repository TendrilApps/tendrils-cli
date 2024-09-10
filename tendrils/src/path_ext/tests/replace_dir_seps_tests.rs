use crate::path_ext::PathExt;
use crate::test_utils::non_utf_8_text;
use rstest::rstest;
use std::ffi::OsString;
use std::path::PathBuf;

#[rstest]
#[case("")]
#[case(" ")]
#[case("PathWithoutSlashes")]
fn path_without_slashes_returns_given(#[case] given_str: &str) {
    let actual = PathBuf::from(given_str).replace_dir_seps();

    assert_eq!(actual.to_string_lossy(), given_str);
}

#[rstest]
#[case("\\Path\\With\\Matching\\Slashes\\",  "/Path/With/Matching/Slashes/")]
#[case("/Path/With\\Mixed/Slashes\\",  "/Path/With/Mixed/Slashes/")]
#[case("\\",  "/")]
#[cfg_attr(windows, ignore)]
fn backslashes_replaced_with_forward_slash(
    #[case] given: PathBuf,
    #[case] expected_str: &str,
) {
    let actual = given.replace_dir_seps();

    assert_eq!(actual.to_string_lossy(), expected_str);
}

#[rstest]
#[case("/Path/With/Matching/Slashes/",  "\\Path\\With\\Matching\\Slashes\\")]
#[case("/Path/With\\Mixed/Slashes\\",  "\\Path\\With\\Mixed\\Slashes\\")]
#[case("/",  "\\")]
#[cfg_attr(not(windows), ignore)]
fn forward_slashes_replaced_with_backslashes(
    #[case] given: PathBuf,
    #[case] expected_str: &str,
) {
    let actual = given.replace_dir_seps();

    assert_eq!(actual.to_string_lossy(), expected_str);
}

#[test]
fn non_utf8_paths_are_preserved() {
    #[cfg(windows)]
    let platform_dir_sep = '\\';
    #[cfg(windows)]
    let other_dir_sep = '/';

    #[cfg(not(windows))]
    let platform_dir_sep = '/';
    #[cfg(not(windows))]
    let other_dir_sep = '\\';

    let mut given_str = OsString::from(other_dir_sep.to_string());
    given_str.push(non_utf_8_text());
    given_str.push(other_dir_sep.to_string());

    let actual = PathBuf::from(given_str).replace_dir_seps();

    let expected_bytes = &mut [platform_dir_sep as u8].to_vec();
    expected_bytes.append(&mut non_utf_8_text().as_encoded_bytes().to_vec());
    expected_bytes.push(platform_dir_sep as u8);

    let expected;
    unsafe {
        expected = OsString::from_encoded_bytes_unchecked(
            expected_bytes.clone()
        );
    }

    assert_eq!(expected, actual.as_os_str());
}
