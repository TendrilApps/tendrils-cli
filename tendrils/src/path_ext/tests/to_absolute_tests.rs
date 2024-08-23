use crate::path_ext::PathExt;
use crate::test_utils::non_utf_8_text;
use rstest::rstest;
use std::ffi::OsString;
use std::path::{PathBuf, MAIN_SEPARATOR_STR as SEP};

#[rstest]
#[case("", SEP)]
#[case(".", &format!("{SEP}."))]
#[case("..", &format!("{SEP}.."))]
#[case("./", &format!("{SEP}./"))]
#[case(".\\", &format!("{SEP}.\\"))]
#[case("../", &format!("{SEP}../"))]
#[case("..\\", &format!("{SEP}..\\"))]
#[case("Plain", &format!("{SEP}Plain"))]
#[case("Trailing/", &format!("{SEP}Trailing/"))]
#[case("Trailing\\", &format!("{SEP}Trailing\\"))]
#[case("Combo/Path\\", &format!("{SEP}Combo/Path\\"))]
#[case("Combo\\Path/", &format!("{SEP}Combo\\Path/"))]
#[cfg_attr(not(windows), case("\\", "/\\"))]
#[cfg_attr(not(windows), case("\\\\\\", "/\\\\\\"))]
#[cfg_attr(not(windows), case("\\.", "/\\."))]
#[cfg_attr(not(windows), case("\\..", "/\\.."))]
#[cfg_attr(not(windows), case("C:\\", "/C:\\"))]
#[cfg_attr(not(windows), case("C:/", "/C:/"))]
#[cfg_attr(not(windows), case("X:\\", "/X:\\"))]
#[cfg_attr(not(windows), case("\\C:\\", "/\\C:\\"))]
fn relative_paths_prepended_with_platform_dir_sep(
    #[case] given: PathBuf,
    #[case] expected: &str,
) {
    let expected_str = OsString::from(expected);

    let actual = given.to_absolute();

    assert_eq!(actual.as_os_str(), expected_str);
}

#[rstest]
#[case("/")]
#[cfg_attr(windows, case("\\"))]
#[case("///")]
#[cfg_attr(windows, case("\\\\\\"))]
#[case("/.")]
#[cfg_attr(windows, case("\\."))]
#[case("/..")]
#[cfg_attr(windows, case("\\.."))]
#[cfg_attr(windows, case("C:\\"))]
#[cfg_attr(windows, case("C:/"))]
#[cfg_attr(windows, case("X:\\"))]
#[case("/C:/")]
#[cfg_attr(windows, case("\\C:\\"))]
fn absolute_paths_returned_unmodified(#[case] given: &str) {
    let expected_str = OsString::from(given);

    let actual = PathBuf::from(given).to_absolute();

    assert_eq!(actual.as_os_str(), expected_str);
}

#[test]
fn non_utf8_is_preserved() {
    let mut expected_str = OsString::from(SEP);
    expected_str.push(non_utf_8_text());

    let actual = PathBuf::from(non_utf_8_text()).to_absolute();

    assert_eq!(actual.as_os_str(), expected_str);
}
