use crate::path_ext::PathExt;
use rstest::rstest;
use std::ffi::OsString;
use std::path::PathBuf;

#[rstest]
#[case("Plain", "/")]
#[case("Plain", "\\")]
#[case("Plain", "/ ")]
#[case("Plain", "\\ ")]
#[case("/", "Plain")]
#[case("\\", "Plain")]
#[case(" /", "Plain")]
#[case(" \\", "Plain")]
#[case("Plain", "/Leading")]
#[case("Plain", "\\Leading")]
#[case("Plain", "//DblLeading")]
#[case("Plain", "\\\\DblLeading")]
#[case("Plain", "/\\MixedLeading")]
#[case("Plain", "\\/MixedLeading")]
#[case("Plain", "\\\\Server\\Share")]
#[case("Plain", "\\\\Server\\Share\\Path")]
#[case("Plain", "\\\\.\\Device\\Path")]
#[case("Plain", "\\\\.\\UNC\\Server\\Share")]
#[case("Plain", "\\\\.\\UNC\\Server\\Share\\Path")]
#[case("Plain", "\\\\?\\Verbatim\\Path")]
#[case("Plain", "\\\\?\\UNC\\Server\\Share")]
#[case("Plain", "\\\\?\\UNC\\Server\\Share\\Path")]
#[case("/Mixed\\", "/Mixed\\")]
#[case("\\Mixed/", "/Mixed\\")]
#[case("/Mixed\\", "\\Mixed/")]
#[case("\\Mixed/", "\\Mixed/")]
#[case("Trailing/", "Plain")]
#[case("Trailing\\", "Plain")]
#[case("Trailing/", "C:\\Abs\\Path")]
#[case("Trailing\\", "C:\\Abs\\Path")]
#[case("Trailing/", "\\\\Server\\Path")]
#[case("Trailing\\", "\\\\Server\\Path")]
#[case("DblTrailing//", "Plain")]
#[case("DblTrailing\\\\", "Plain")]
#[case("DblTrailing//", "//DblLeading")]
#[case("DblTrailing\\\\", "\\\\DblLeading")]
#[case("DblTrailing//", "\\\\DblLeading")]
#[case("DblTrailing\\\\", "//DblLeading")]
#[case("DblTrailing//", "C:\\Abs\\Path")]
#[case("DblTrailing\\\\", "C:\\Abs\\Path")]
#[case("DblTrailing//", "\\\\Server\\Path")]
#[case("DblTrailing\\\\", "\\\\Server\\Path")]
#[case("RelTrailing/.", "/Leading")]
#[case("RelTrailing\\.", "\\Leading")]
#[case("Trailing/", "./RelLeading")]
#[case("Trailing\\", ".\\RelLeading")]
#[case("RelTrailing/..", "/Leading")]
#[case("RelTrailing\\..", "\\Leading")]
#[case("Trailing/", "../RelLeading")]
#[case("Trailing\\", "..\\RelLeading")]
#[case("Crazy`~!@#$%^&*()-_=+|\\[]{}'\";:/?.,<>Symbols/", "Plain")]
#[case("Plain", "/Crazy`~!@#$%^&*()-_=+|\\[]{}'\";:/?.,<>Symbols")]
fn appends_path_preserving_dir_seps(#[case] parent: &str, #[case] child: &str) {
    let mut expected_str = parent.to_string();
    expected_str.push_str(child);

    let actual = PathBuf::from(parent).join_raw(&PathBuf::from(child));

    assert_eq!(actual.as_os_str(), OsString::from(expected_str));
}

#[rstest]
#[case("Plain", "")]
#[case("Plain", " ")]
#[case("Plain", " /")]
#[case("Plain", " \\")]
#[case("", "Plain")]
#[case(" ", "Plain")]
#[case("/ ", "Plain")]
#[case("\\ ", "Plain")]
#[case("Plain", "Plain")]
#[case("/Leading", "Plain")]
#[case("\\Leading", "Plain")]
#[case("//DblLeading", "Plain")]
#[case("\\\\DblLeading", "Plain")]
#[case("Plain", "Trailing/")]
#[case("Plain", "Trailing\\")]
#[case("Plain", "DblTrailing//")]
#[case("Plain", "DblTrailing\\\\")]
#[case("Plain", "C:\\Abs\\Path")]
#[case("/Leading", "Trailing/")]
#[case("\\Leading", "Trailing\\")]
#[case("RelTrailing/.", "Plain")]
#[case("RelTrailing\\.", "Plain")]
#[case("Plain", "./RelLeading")]
#[case("Plain", ".\\RelLeading")]
#[case("RelTrailing/..", "Plain")]
#[case("RelTrailing\\..", "Plain")]
#[case("Plain", "../RelLeading")]
#[case("Plain", "..\\RelLeading")]
#[case("Crazy`~!@#$%^&*()-_=+|\\[]{}'\";:/?.,<>Symbols", "Plain")]
#[case("Plain", "Crazy`~!@#$%^&*()-_=+|\\[]{}'\";:/?.,<>Symbols")]
fn adds_platform_dir_sep_if_parent_doesnt_have_trailing_and_child_doesnt_have_leading(
    #[case] parent: &str,
    #[case] child: &str,
) {
    let mut expected_str = parent.to_string();
    expected_str.push_str(std::path::MAIN_SEPARATOR_STR);
    expected_str.push_str(child);

    let actual = PathBuf::from(parent).join_raw(&PathBuf::from(child));

    assert_eq!(actual.as_os_str(), OsString::from(expected_str));
}
