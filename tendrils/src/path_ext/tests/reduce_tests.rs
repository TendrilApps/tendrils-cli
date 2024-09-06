use crate::path_ext::PathExt;
use crate::test_utils::{get_disposable_dir, non_utf_8_text, symlink_expose};
use rstest::rstest;
use std::ffi::OsString;
use std::path::{MAIN_SEPARATOR_STR as SEP, PathBuf};
use tempdir::TempDir;

#[rstest]
#[case("//", SEP)]
#[cfg_attr(windows, case("\\\\", SEP))]
#[cfg_attr(not(windows), case("\\\\", "\\\\"))]
#[cfg_attr(windows, case("/\\", SEP))]
#[cfg_attr(not(windows), case("/\\", "/\\"))]
#[cfg_attr(windows, case("\\/", SEP))]
#[cfg_attr(not(windows), case("\\/", "\\"))]
#[case("/.", SEP)]
#[cfg_attr(windows, case("\\.", SEP))]
#[cfg_attr(not(windows), case("\\.", "\\."))]
#[case("/./", SEP)]
#[cfg_attr(windows, case("\\.\\", SEP))]
#[cfg_attr(not(windows), case("\\.\\", "\\.\\"))]
#[case("/..", SEP)]
#[cfg_attr(windows, case("\\..", SEP))]
#[cfg_attr(not(windows), case("\\..", "\\.."))]
#[case("/../", SEP)]
#[cfg_attr(windows, case("\\..\\", SEP))]
#[cfg_attr(not(windows), case("\\..\\", "\\..\\"))]
#[case("Some/Path/.", &format!("Some{SEP}Path"))]
#[cfg_attr(windows, case("Some\\Path\\.", "Some\\Path"))]
#[cfg_attr(not(windows), case("Some\\Path\\.", "Some\\Path\\."))]
#[case("Some/Path/..", "Some")]
#[cfg_attr(windows, case("Some\\Path\\..", "Some"))]
#[cfg_attr(not(windows), case("Some\\Path\\..", "Some\\Path\\.."))]
#[case("./Some/./Path/.", &format!("Some{SEP}Path"))]
#[cfg_attr(windows, case(".\\Some\\.\\Path\\.", "Some\\Path"))]
#[cfg_attr(not(windows), case(".\\Some\\.\\Path\\.", ".\\Some\\.\\Path\\."))]
#[cfg_attr(windows, case(".///Some/Ugly/\\\\.//../Path\\/.///.\\.", "Some\\Path"))]
#[cfg_attr(not(windows), case(".///Some/Ugly/\\\\.//../Path\\/.///.\\.", "Some/Ugly/Path\\/.\\."))]
#[case("", "")]
#[case(" ", " ")]
#[case("/ ", &format!("{SEP} "))]
#[cfg_attr(windows, case("\\ ", "\\ "))]
#[cfg_attr(not(windows), case("\\ ", "\\ "))]
#[case("/ /", &format!("{SEP} "))]
#[cfg_attr(windows, case("\\ \\", &format!("{SEP} ")))]
#[cfg_attr(not(windows), case("\\ \\", "\\ \\"))]
#[case(
    "./..Weird/.../.Path/....../Components",
    &format!("..Weird{SEP}...{SEP}.Path{SEP}......{SEP}Components")
)]
#[case("Plain", "Plain")]
#[case("Simple/Path", &format!("Simple{SEP}Path"))]
#[cfg_attr(windows, case("Simple\\Path", &format!("Simple{SEP}Path")))]
#[cfg_attr(not(windows), case("Simple\\Path", "Simple\\Path"))]
#[case("C:", "C:")]
#[cfg_attr(windows, case("C:/", "C:\\"))]
#[cfg_attr(not(windows), case("C:/", "C:"))]
#[case("C:\\", "C:\\")]
#[cfg_attr(windows, case("//Server/Path", "//Server/Path\\"))]
#[cfg_attr(not(windows), case("//Server/Path", "/Server/Path"))]
// Ideally these would join raw, but this is a limitation due to the use of
// path_clean crate and its handling of drive letter path components
#[cfg_attr(windows, case("Nested/C:/Abs", "C:Abs"))]
#[cfg_attr(not(windows), case("Nested/C:/Abs", "Nested/C:/Abs"))]
#[cfg_attr(windows, case("Nested\\C:\\Abs", "C:Abs"))]
#[cfg_attr(not(windows), case("Nested\\C:\\Abs", "Nested\\C:\\Abs"))]
#[cfg_attr(windows, case("/Nested/C:/Abs", "C:Abs"))]
#[cfg_attr(not(windows), case("/Nested/C:/Abs", "/Nested/C:/Abs"))]
#[cfg_attr(windows, case("\\Nested\\C:\\Abs", "C:Abs"))]
#[cfg_attr(not(windows), case("\\Nested\\C:\\Abs", "\\Nested\\C:\\Abs"))]
#[cfg_attr(windows, case("/A/B/C:/Abs", "C:Abs"))]
#[cfg_attr(not(windows), case("/A/B/C:/Abs", "/A/B/C:/Abs"))]
#[cfg_attr(windows, case("\\A\\B\\C:\\Abs", "C:Abs"))]
#[cfg_attr(not(windows), case("\\A\\B\\C:\\Abs", "\\A\\B\\C:\\Abs"))]
fn duplicate_and_trailing_dir_seps_are_removed_and_dot_components_are_resolved_lexically(
    #[case] given: PathBuf,
    #[case] expected: &str,
) {
    let actual = given.reduce();

    assert_eq!(
        actual.to_string_lossy(),
        expected,
        "Given {}",
        given.to_string_lossy(),
    );
}

#[rstest]
#[case("..", "..")]
#[case("../", "..")]
#[cfg_attr(windows, case("..\\", ".."))]
#[cfg_attr(not(windows), case("..\\", "..\\"))]
#[case("..//", "..")]
#[cfg_attr(windows, case("..\\\\", ".."))]
#[cfg_attr(not(windows), case("..\\\\", "..\\\\"))]
#[case("../Some/Path", &format!("..{SEP}Some{SEP}Path"))]
#[case("..\\Some\\Path", "..\\Some\\Path")]
#[case("../../..", &format!("..{SEP}..{SEP}.."))]
#[case("..\\..\\..", "..\\..\\..")]
#[case("../Some/..", "..")]
#[cfg_attr(windows, case("..\\Some\\..", ".."))]
#[cfg_attr(not(windows), case("..\\Some\\..", "..\\Some\\.."))]
#[case("../Some/../Path", &format!("..{SEP}Path"))]
#[cfg_attr(windows, case("..\\Some\\..\\Path", "..\\Path"))]
#[cfg_attr(not(windows), case("..\\Some\\..\\Path", "..\\Some\\..\\Path"))]
fn leading_double_dot_components_remain(
    #[case] given: PathBuf,
    #[case] expected: &str,
) {
    let actual = given.reduce();

    assert_eq!(actual.to_string_lossy(), expected);
}

#[test]
fn non_utf8_is_preserved() {
    let mut expected_str = OsString::from(SEP);
    expected_str.push(non_utf_8_text());

    let mut given_str = OsString::from("//..//");
    given_str.push(non_utf_8_text());
    given_str.push(format!("//."));
    let actual = PathBuf::from(given_str).reduce();

    assert_eq!(actual.as_os_str(), expected_str);
}

#[test]
fn resolves_dot_dot_lexically_without_following_symlinks() {
    let temp_dir =
        TempDir::new_in(get_disposable_dir(), "Top").unwrap();
    let symdir = temp_dir.path().join("sym_dir");
    let target = temp_dir.path().join("target/misc.txt");
    std::fs::create_dir_all(&symdir).unwrap();
    std::fs::create_dir_all(target.parent().unwrap()).unwrap();
    std::fs::write(&target, "Target file contents").unwrap();
    symlink_expose(&symdir, &target, false, true).unwrap();

    let actual = PathBuf::from("Top/sym_dir/../").reduce();

    assert_eq!(actual.to_string_lossy(), "Top".to_string());
}
