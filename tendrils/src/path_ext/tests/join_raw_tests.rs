use crate::path_ext::PathExt;
use rstest::rstest;
use std::ffi::OsString;
use std::path::PathBuf;

#[rstest]
#[case("Plain", "/")]
#[cfg_attr(windows, case("Plain", "\\"))]
#[case("Plain", "/ ")]
#[cfg_attr(windows, case("Plain", "\\ "))]
#[case("/", "Plain")]
#[cfg_attr(windows, case("\\", "Plain"))]
#[case(" /", "Plain")]
#[cfg_attr(windows, case(" \\", "Plain"))]
#[case("Plain", "/Leading")]
#[cfg_attr(windows, case("Plain", "\\Leading"))]
#[case("Plain", "//DblLeading")]
#[cfg_attr(windows, case("Plain", "\\\\DblLeading"))]
#[case("Plain", "/\\MixedLeading")]
#[cfg_attr(windows, case("Plain", "\\/MixedLeading"))]
#[cfg_attr(windows, case("Plain", "\\\\Server\\Share"))]
#[cfg_attr(windows, case("Plain", "\\\\Server\\Share\\Path"))]
#[cfg_attr(windows, case("Plain", "\\\\.\\Device\\Path"))]
#[cfg_attr(windows, case("Plain", "\\\\.\\UNC\\Server\\Share"))]
#[cfg_attr(windows, case("Plain", "\\\\.\\UNC\\Server\\Share\\Path"))]
#[cfg_attr(windows, case("Plain", "\\\\?\\Verbatim\\Path"))]
#[cfg_attr(windows, case("Plain", "\\\\?\\UNC\\Server\\Share"))]
#[cfg_attr(windows, case("Plain", "\\\\?\\UNC\\Server\\Share\\Path"))]
#[case("/Mixed\\", "/Mixed\\")]
#[case("\\Mixed/", "/Mixed\\")]
#[cfg_attr(windows, case("/Mixed\\", "\\Mixed/"))]
#[case("\\Mixed/", "\\Mixed/")]
#[case("Trailing/", "Plain")]
#[cfg_attr(windows, case("Trailing\\", "Plain"))]
#[case("Trailing/", "C:\\Abs\\Path")]
#[cfg_attr(windows, case("Trailing\\", "C:\\Abs\\Path"))]
#[case("Trailing/", "\\\\Server\\Path")]
#[cfg_attr(windows, case("Trailing\\", "\\\\Server\\Path"))]
#[case("DblTrailing//", "Plain")]
#[cfg_attr(windows, case("DblTrailing\\\\", "Plain"))]
#[case("DblTrailing//", "//DblLeading")]
#[cfg_attr(windows, case("DblTrailing\\\\", "\\\\DblLeading"))]
#[case("DblTrailing//", "\\\\DblLeading")]
#[case("DblTrailing\\\\", "//DblLeading")]
#[case("DblTrailing//", "C:\\Abs\\Path")]
#[cfg_attr(windows, case("DblTrailing\\\\", "C:\\Abs\\Path"))]
#[case("DblTrailing//", "\\\\Server\\Path")]
#[cfg_attr(windows, case("DblTrailing\\\\", "\\\\Server\\Path"))]
#[case("RelTrailing/.", "/Leading")]
#[cfg_attr(windows, case("RelTrailing\\.", "\\Leading"))]
#[case("Trailing/", "./RelLeading")]
#[cfg_attr(windows, case("Trailing\\", ".\\RelLeading"))]
#[case("RelTrailing/..", "/Leading")]
#[cfg_attr(windows, case("RelTrailing\\..", "\\Leading"))]
#[case("Trailing/", "../RelLeading")]
#[cfg_attr(windows, case("Trailing\\", "..\\RelLeading"))]
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
#[cfg_attr(not(windows), case("Plain", "\\"))]
#[case("Plain", " ")]
#[case("Plain", " /")]
#[case("Plain", " \\")]
#[cfg_attr(not(windows), case("Plain", "\\ "))]
#[case("", "Plain")]
#[cfg_attr(not(windows), case("\\", "Plain"))]
#[case(" ", "Plain")]
#[case("/ ", "Plain")]
#[case("\\ ", "Plain")]
#[cfg_attr(not(windows), case(" \\", "Plain"))]
#[case("Plain", "Plain")]
#[case("/Leading", "Plain")]
#[case("\\Leading", "Plain")]
#[case("//DblLeading", "Plain")]
#[case("\\\\DblLeading", "Plain")]
#[cfg_attr(not(windows), case("Plain", "\\Leading"))]
#[cfg_attr(not(windows), case("Plain", "\\\\DblLeading"))]
#[cfg_attr(not(windows), case("Plain", "\\/MixedLeading"))]
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
