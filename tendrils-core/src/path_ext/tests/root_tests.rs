use crate::path_ext::PathExt;
use crate::path_ext::tests::test_paths::cases;
use crate::test_utils::non_utf_8_text;
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf, MAIN_SEPARATOR_STR as SEP};

/// Checks that each of the subscribed sets of test cases has a corresponding 
/// expected output.
#[test]
fn all_given_path_test_cases_covered() {
    let mut map = HashMap::new();

    for pair in given_path_and_exp() {
        assert!(map.insert(pair.0.to_string(), pair.1.to_string()).is_none());
    }

    // This is where the different sets of test cases are "subscribed to" for
    // this module.
    for case in cases() {
        assert!(map.contains_key(case), "Missing case: {}", case);
    }
}

/// Checks that each of the subscribed sets of test cases has a corresponding 
/// expected output.
#[test]
fn all_given_root_test_cases_covered() {
    let mut map = HashMap::new();

    for pair in given_root_and_exp() {
        assert!(map.insert(pair.0.to_string(), pair.1.to_string()).is_none());
    }

    // This is where the different sets of test cases are "subscribed to" for
    // this module.
    for case in cases() {
        assert!(map.contains_key(case), "Missing case: {}", case);
    }
}

#[test]
fn given_path_and_abs_root_returns_expected() {
    for pair in given_path_and_exp() {
        let given_path = PathBuf::from(&pair.0);

        let actual = given_path.root(&Path::new("/MyRoot"));

        assert_eq!(
            actual.to_string_lossy(),
            pair.1,
            "Given: {:?}",
            pair.0,
        );
    }
}

#[test]
fn given_root_and_relative_path_returns_expected() {
    for pair in given_root_and_exp() {
        let given_root = PathBuf::from(&pair.0);

        let actual = &Path::new("RelPath").root(&given_root);

        assert_eq!(
            actual.to_string_lossy(),
            pair.1,
            "Given: {:?}",
            pair.0,
        );
    }
}

#[test]
fn non_utf8_is_preserved() {
    let mut given_root = OsString::from(SEP);
    given_root.push(non_utf_8_text());
    let mut expected_str = OsString::from(SEP);
    expected_str.push(non_utf_8_text());
    expected_str.push(SEP);
    expected_str.push(non_utf_8_text());

    let actual = PathBuf::from(non_utf_8_text())
        .root(&PathBuf::from(given_root));

    assert_eq!(actual.as_os_str(), expected_str);
}

fn given_path_and_exp() -> Vec<(String, String)> {
    // (Given path, Expected)
    let pairs: [(&str, &str); 114] = [
        ("", &format!("/MyRoot{SEP}")),
        (".", &format!("/MyRoot{SEP}.")),
        ("..", &format!("/MyRoot{SEP}..")),
        ("./", &format!("/MyRoot{SEP}./")),
        (".\\", &format!("/MyRoot{SEP}.\\")),
        ("../", &format!("/MyRoot{SEP}../")),
        ("..\\", &format!("/MyRoot{SEP}..\\")),
        ("Plain", &format!("/MyRoot{SEP}Plain")),
        ("Trailing/", &format!("/MyRoot{SEP}Trailing/")),
        ("Trailing\\", &format!("/MyRoot{SEP}Trailing\\")),
        ("/Leading", "/Leading"),
        #[cfg(not(windows))]
        ("\\Leading", "/MyRoot/\\Leading"),
        #[cfg(windows)]
        ("\\Leading", "\\Leading"),
        ("Combo/Path\\", &format!("/MyRoot{SEP}Combo/Path\\")),
        ("Combo\\Path/", &format!("/MyRoot{SEP}Combo\\Path/")),
        ("/", "/"),
        #[cfg(not(windows))]
        ("\\", "/MyRoot/\\"),
        #[cfg(windows)]
        ("\\", "\\"),
        ("//", "//"),
        #[cfg(not(windows))]
        ("\\\\", "/MyRoot/\\\\"),
        #[cfg(windows)]
        ("\\\\", "\\\\"),
        ("///", "///"),
        #[cfg(not(windows))]
        ("\\\\\\", "/MyRoot/\\\\\\"),
        #[cfg(windows)]
        ("\\\\\\", "\\\\\\"),
        ("/.", "/."),
        #[cfg(not(windows))]
        ("\\.", "/MyRoot/\\."),
        #[cfg(windows)]
        ("\\.", "\\."),
        ("/..", "/.."),
        #[cfg(not(windows))]
        ("\\..", "/MyRoot/\\.."),
        #[cfg(windows)]
        ("\\..", "\\.."),
        ("//.", "//."),
        #[cfg(not(windows))]
        ("\\\\.", "/MyRoot/\\\\."),
        #[cfg(windows)]
        ("\\\\.", "\\\\."),
        ("//..", "//.."),
        #[cfg(not(windows))]
        ("\\\\..", "/MyRoot/\\\\.."),
        #[cfg(windows)]
        ("\\\\..", "\\\\.."),
        #[cfg(not(windows))]
        ("C:/", "/MyRoot/C:/"),
        #[cfg(windows)]
        ("C:/", "C:/"),
        #[cfg(not(windows))]
        ("C:\\", "/MyRoot/C:\\"),
        #[cfg(windows)]
        ("C:\\", "C:\\"),
        #[cfg(not(windows))]
        ("c:/", "/MyRoot/c:/"),
        #[cfg(windows)]
        ("c:/", "c:/"),
        #[cfg(not(windows))]
        ("c:\\", "/MyRoot/c:\\"),
        #[cfg(windows)]
        ("c:\\", "c:\\"),
        #[cfg(not(windows))]
        ("X:\\", "/MyRoot/X:\\"),
        #[cfg(windows)]
        ("X:\\", "X:\\"),
        ("C:", &format!("/MyRoot{SEP}C:")),
        ("c:", &format!("/MyRoot{SEP}c:")),
        ("C:WithoutRoot", &format!("/MyRoot{SEP}C:WithoutRoot")),
        #[cfg(not(windows))]
        ("C:\\.", "/MyRoot/C:\\."),
        #[cfg(windows)]
        ("C:\\.", "C:\\."),
        #[cfg(not(windows))]
        ("C:\\..", "/MyRoot/C:\\.."),
        #[cfg(windows)]
        ("C:\\..", "C:\\.."),
        ("/C:/", "/C:/"),
        #[cfg(not(windows))]
        ("\\C:\\", "/MyRoot/\\C:\\"),
        #[cfg(windows)]
        ("\\C:\\", "\\C:\\"),
        ("/C:", "/C:"),
        #[cfg(not(windows))]
        ("\\C:", "/MyRoot/\\C:"),
        #[cfg(windows)]
        ("\\C:", "\\C:"),
        ("//Server/Share/", "//Server/Share/"),
        #[cfg(not(windows))]
        ("\\\\Server\\Share\\", "/MyRoot/\\\\Server\\Share\\"),
        #[cfg(windows)]
        ("\\\\Server\\Share\\", "\\\\Server\\Share\\"),
        ("//Server/Share", "//Server/Share"),
        #[cfg(not(windows))]
        ("\\\\Server\\Share", "/MyRoot/\\\\Server\\Share"),
        #[cfg(windows)]
        ("\\\\Server\\Share", "\\\\Server\\Share"),
        #[cfg(not(windows))]
        ("\\\\Server\\Share\\misc.txt", "/MyRoot/\\\\Server\\Share\\misc.txt"),
        #[cfg(windows)]
        ("\\\\Server\\Share\\misc.txt", "\\\\Server\\Share\\misc.txt"),
        #[cfg(not(windows))]
        ("\\\\Server\\misc.txt", "/MyRoot/\\\\Server\\misc.txt"),
        #[cfg(windows)]
        ("\\\\Server\\misc.txt", "\\\\Server\\misc.txt"),
        ("///Server/Share", "///Server/Share"),
        #[cfg(not(windows))]
        ("\\\\Server\\Share\\C:", "/MyRoot/\\\\Server\\Share\\C:"),
        #[cfg(windows)]
        ("\\\\Server\\Share\\C:", "\\\\Server\\Share\\C:"),
        #[cfg(not(windows))]
        ("\\\\\\Server\\Share", "/MyRoot/\\\\\\Server\\Share"),
        #[cfg(windows)]
        ("\\\\\\Server\\Share", "\\\\\\Server\\Share"),
        #[cfg(not(windows))]
        ("\\\\127.0.0.1\\Share", "/MyRoot/\\\\127.0.0.1\\Share"),
        #[cfg(windows)]
        ("\\\\127.0.0.1\\Share", "\\\\127.0.0.1\\Share"),
        #[cfg(not(windows))]
        ("\\\\Server\\C$\\", "/MyRoot/\\\\Server\\C$\\"),
        #[cfg(windows)]
        ("\\\\Server\\C$\\", "\\\\Server\\C$\\"),
        #[cfg(not(windows))]
        ("\\\\Server\\C$", "/MyRoot/\\\\Server\\C$"),
        #[cfg(windows)]
        ("\\\\Server\\C$", "\\\\Server\\C$"),
        #[cfg(not(windows))]
        ("\\\\Server\\C:\\", "/MyRoot/\\\\Server\\C:\\"),
        #[cfg(windows)]
        ("\\\\Server\\C:\\", "\\\\Server\\C:\\"),
        #[cfg(not(windows))]
        ("\\\\Server\\C:", "/MyRoot/\\\\Server\\C:"),
        #[cfg(windows)]
        ("\\\\Server\\C:", "\\\\Server\\C:"),
        #[cfg(not(windows))]
        ("\\\\Server\\C$\\.", "/MyRoot/\\\\Server\\C$\\."),
        #[cfg(windows)]
        ("\\\\Server\\C$\\.", "\\\\Server\\C$\\."),
        #[cfg(not(windows))]
        ("\\\\Server\\C$\\..", "/MyRoot/\\\\Server\\C$\\.."),
        #[cfg(windows)]
        ("\\\\Server\\C$\\..", "\\\\Server\\C$\\.."),
        #[cfg(not(windows))]
        ("\\\\.\\.", "/MyRoot/\\\\.\\."),
        #[cfg(windows)]
        ("\\\\.\\.", "\\\\.\\."),
        #[cfg(not(windows))]
        ("\\\\..\\..", "/MyRoot/\\\\..\\.."),
        #[cfg(windows)]
        ("\\\\..\\..", "\\\\..\\.."),
        ("//?/", "//?/"),
        #[cfg(not(windows))]
        ("\\\\?\\", "/MyRoot/\\\\?\\"),
        #[cfg(windows)]
        ("\\\\?\\", "\\\\?\\"),
        ("//?//", "//?//"),
        #[cfg(not(windows))]
        ("\\\\?\\\\", "/MyRoot/\\\\?\\\\"),
        #[cfg(windows)]
        ("\\\\?\\\\", "\\\\?\\\\"),
        ("//?/C:/", "//?/C:/"),
        #[cfg(not(windows))]
        ("\\\\?\\C:\\", "/MyRoot/\\\\?\\C:\\"),
        #[cfg(windows)]
        ("\\\\?\\C:\\", "\\\\?\\C:\\"),
        #[cfg(not(windows))]
        ("\\\\?\\C:", "/MyRoot/\\\\?\\C:"),
        #[cfg(windows)]
        ("\\\\?\\C:", "\\\\?\\C:"),
        #[cfg(not(windows))]
        ("\\\\?\\C:WithoutRoot", "/MyRoot/\\\\?\\C:WithoutRoot"),
        #[cfg(windows)]
        ("\\\\?\\C:WithoutRoot", "\\\\?\\C:WithoutRoot"),
        #[cfg(not(windows))]
        ("\\\\?\\.", "/MyRoot/\\\\?\\."),
        #[cfg(windows)]
        ("\\\\?\\.", "\\\\?\\."),
        #[cfg(not(windows))]
        ("\\\\?\\..", "/MyRoot/\\\\?\\.."),
        #[cfg(windows)]
        ("\\\\?\\..", "\\\\?\\.."),
        ("//./", "//./"),
        #[cfg(not(windows))]
        ("\\\\.\\", "/MyRoot/\\\\.\\"),
        #[cfg(windows)]
        ("\\\\.\\", "\\\\.\\"),
        ("//.//", "//.//"),
        #[cfg(not(windows))]
        ("\\\\.\\\\", "/MyRoot/\\\\.\\\\"),
        #[cfg(windows)]
        ("\\\\.\\\\", "\\\\.\\\\"),
        ("//./C:/", "//./C:/"),
        #[cfg(not(windows))]
        ("\\\\.\\C:\\", "/MyRoot/\\\\.\\C:\\"),
        #[cfg(windows)]
        ("\\\\.\\C:\\", "\\\\.\\C:\\"),
        #[cfg(not(windows))]
        ("\\\\.\\C:", "/MyRoot/\\\\.\\C:"),
        #[cfg(windows)]
        ("\\\\.\\C:", "\\\\.\\C:"),
        #[cfg(not(windows))]
        ("\\\\.\\C:WithoutRoot", "/MyRoot/\\\\.\\C:WithoutRoot"),
        #[cfg(windows)]
        ("\\\\.\\C:WithoutRoot", "\\\\.\\C:WithoutRoot"),
        ("//?/UNC/Server/Share", "//?/UNC/Server/Share"),
        #[cfg(not(windows))]
        ("\\\\?\\UNC\\Server\\Share", "/MyRoot/\\\\?\\UNC\\Server\\Share"),
        #[cfg(windows)]
        ("\\\\?\\UNC\\Server\\Share", "\\\\?\\UNC\\Server\\Share"),
        ("//?/UNC/Server/C$", "//?/UNC/Server/C$"),
        #[cfg(not(windows))]
        ("\\\\?\\UNC\\Server\\C$", "/MyRoot/\\\\?\\UNC\\Server\\C$"),
        #[cfg(windows)]
        ("\\\\?\\UNC\\Server\\C$", "\\\\?\\UNC\\Server\\C$"),
        #[cfg(not(windows))]
        ("\\\\?\\UNC\\Server\\C$\\", "/MyRoot/\\\\?\\UNC\\Server\\C$\\"),
        #[cfg(windows)]
        ("\\\\?\\UNC\\Server\\C$\\", "\\\\?\\UNC\\Server\\C$\\"),
        #[cfg(not(windows))]
        ("\\\\?\\unc\\Server\\c$\\", "/MyRoot/\\\\?\\unc\\Server\\c$\\"),
        #[cfg(windows)]
        ("\\\\?\\unc\\Server\\c$\\", "\\\\?\\unc\\Server\\c$\\"),
        #[cfg(not(windows))]
        ("\\\\?\\UNC\\Server\\C:", "/MyRoot/\\\\?\\UNC\\Server\\C:"),
        #[cfg(windows)]
        ("\\\\?\\UNC\\Server\\C:", "\\\\?\\UNC\\Server\\C:"),
        #[cfg(not(windows))]
        ("\\\\?\\UNC\\Server\\C$\\.", "/MyRoot/\\\\?\\UNC\\Server\\C$\\."),
        #[cfg(windows)]
        ("\\\\?\\UNC\\Server\\C$\\.", "\\\\?\\UNC\\Server\\C$\\."),
        #[cfg(not(windows))]
        ("\\\\?\\UNC\\Server\\C$\\..", "/MyRoot/\\\\?\\UNC\\Server\\C$\\.."),
        #[cfg(windows)]
        ("\\\\?\\UNC\\Server\\C$\\..", "\\\\?\\UNC\\Server\\C$\\.."),
        #[cfg(not(windows))]
        ("\\\\?\\UNC\\.\\.", "/MyRoot/\\\\?\\UNC\\.\\."),
        #[cfg(windows)]
        ("\\\\?\\UNC\\.\\.", "\\\\?\\UNC\\.\\."),
        #[cfg(not(windows))]
        ("\\\\?\\UNC\\..\\..", "/MyRoot/\\\\?\\UNC\\..\\.."),
        #[cfg(windows)]
        ("\\\\?\\UNC\\..\\..", "\\\\?\\UNC\\..\\.."),
        ("//./UNC/Server/Share", "//./UNC/Server/Share"),
        #[cfg(not(windows))]
        ("\\\\.\\UNC\\Server\\Share", "/MyRoot/\\\\.\\UNC\\Server\\Share"),
        #[cfg(windows)]
        ("\\\\.\\UNC\\Server\\Share", "\\\\.\\UNC\\Server\\Share"),
        ("//./UNC/Server/C$", "//./UNC/Server/C$"),
        #[cfg(not(windows))]
        ("\\\\.\\UNC\\Server\\C$", "/MyRoot/\\\\.\\UNC\\Server\\C$"),
        #[cfg(windows)]
        ("\\\\.\\UNC\\Server\\C$", "\\\\.\\UNC\\Server\\C$"),
        #[cfg(not(windows))]
        ("\\\\.\\UNC\\Server\\C$\\", "/MyRoot/\\\\.\\UNC\\Server\\C$\\"),
        #[cfg(windows)]
        ("\\\\.\\UNC\\Server\\C$\\", "\\\\.\\UNC\\Server\\C$\\"),
        #[cfg(not(windows))]
        ("\\\\.\\unc\\Server\\c$\\", "/MyRoot/\\\\.\\unc\\Server\\c$\\"),
        #[cfg(windows)]
        ("\\\\.\\unc\\Server\\c$\\", "\\\\.\\unc\\Server\\c$\\"),
        #[cfg(not(windows))]
        ("\\\\.\\UNC\\Server\\C:", "/MyRoot/\\\\.\\UNC\\Server\\C:"),
        #[cfg(windows)]
        ("\\\\.\\UNC\\Server\\C:", "\\\\.\\UNC\\Server\\C:"),
        #[cfg(not(windows))]
        ("\\\\.\\UNC\\Server\\C$\\.", "/MyRoot/\\\\.\\UNC\\Server\\C$\\."),
        #[cfg(windows)]
        ("\\\\.\\UNC\\Server\\C$\\.", "\\\\.\\UNC\\Server\\C$\\."),
        #[cfg(not(windows))]
        ("\\\\.\\UNC\\Server\\C$\\..", "/MyRoot/\\\\.\\UNC\\Server\\C$\\.."),
        #[cfg(windows)]
        ("\\\\.\\UNC\\Server\\C$\\..", "\\\\.\\UNC\\Server\\C$\\.."),
        #[cfg(not(windows))]
        ("\\\\.\\UNC\\.\\.", "/MyRoot/\\\\.\\UNC\\.\\."),
        #[cfg(windows)]
        ("\\\\.\\UNC\\.\\.", "\\\\.\\UNC\\.\\."),
        #[cfg(not(windows))]
        ("\\\\.\\UNC\\..\\..", "/MyRoot/\\\\.\\UNC\\..\\.."),
        #[cfg(windows)]
        ("\\\\.\\UNC\\..\\..", "\\\\.\\UNC\\..\\.."),
        (
            "//?/Volume{12a34b56-78c9-012d-ef3g-45678hij9012}",
            "//?/Volume{12a34b56-78c9-012d-ef3g-45678hij9012}",
        ),
        #[cfg(not(windows))]
        (
            "\\\\?\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}",
            "/MyRoot/\\\\?\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}",
        ),
        #[cfg(windows)]
        (
            "\\\\?\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}",
            "\\\\?\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}",
        ),
        (
            "//?/Volume{12a34b56-78c9-012d-ef3g-45678hij9012}/",
            "//?/Volume{12a34b56-78c9-012d-ef3g-45678hij9012}/",
        ),
        #[cfg(not(windows))]
        (
            "\\\\?\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}\\",
            "/MyRoot/\\\\?\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}\\",
        ),
        #[cfg(windows)]
        (
            "\\\\?\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}\\",
            "\\\\?\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}\\",
        ),
        (
            "//?/Volume{12a34b56-78c9-012d-ef3g-45678hij9012}/misc.txt",
            "//?/Volume{12a34b56-78c9-012d-ef3g-45678hij9012}/misc.txt",
        ),
        #[cfg(not(windows))]
        (
            "\\\\?\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}\\misc.txt",
            "/MyRoot/\\\\?\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}\\misc.txt",
        ),
        #[cfg(windows)]
        (
            "\\\\?\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}\\misc.txt",
            "\\\\?\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}\\misc.txt",
        ),
        (
            "//./Volume{12a34b56-78c9-012d-ef3g-45678hij9012}",
            "//./Volume{12a34b56-78c9-012d-ef3g-45678hij9012}",
        ),
        #[cfg(not(windows))]
        (
            "\\\\.\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}",
            "/MyRoot/\\\\.\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}",
        ),
        #[cfg(windows)]
        (
            "\\\\.\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}",
            "\\\\.\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}",
        ),
        (
            "//./Volume{12a34b56-78c9-012d-ef3g-45678hij9012}/",
            "//./Volume{12a34b56-78c9-012d-ef3g-45678hij9012}/",
        ),
        #[cfg(not(windows))]
        (
            "\\\\.\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}\\",
            "/MyRoot/\\\\.\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}\\",
        ),
        #[cfg(windows)]
        (
            "\\\\.\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}\\",
            "\\\\.\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}\\",
        ),
        (
            "//./Volume{12a34b56-78c9-012d-ef3g-45678hij9012}/misc.txt",
            "//./Volume{12a34b56-78c9-012d-ef3g-45678hij9012}/misc.txt",
        ),
        #[cfg(not(windows))]
        (
            "\\\\.\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}\\misc.txt",
            "/MyRoot/\\\\.\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}\\misc.txt",
        ),
        #[cfg(windows)]
        (
            "\\\\.\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}\\misc.txt",
            "\\\\.\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}\\misc.txt",
        ),
        (
            "file:///../File/Protocol",
            &format!("/MyRoot{SEP}file:///../File/Protocol"),
        ),
        (
            "https://www.website.com",
            &format!("/MyRoot{SEP}https://www.website.com"),
        ),
    ];

    pairs.iter().map(|p| (p.0.to_string(), p.1.to_string())).collect()
}

fn given_root_and_exp() -> Vec<(String, String)> {
    // (Given root, Expected)
    let pairs: [(&str, &str); 114] = [
        ("", &format!("{SEP}RelPath")),
        (".", &format!("{SEP}RelPath")),
        ("..", &format!("{SEP}RelPath")),
        ("./", &format!("{SEP}RelPath")),
        (".\\", &format!("{SEP}RelPath")),
        ("../", &format!("{SEP}RelPath")),
        ("..\\", &format!("{SEP}RelPath")),
        ("Plain", &format!("{SEP}RelPath")),
        ("Trailing/", &format!("{SEP}RelPath")),
        ("Trailing\\", &format!("{SEP}RelPath")),
        ("/Leading", &format!("/Leading{SEP}RelPath")),
        #[cfg(not(windows))]
        ("\\Leading", "/RelPath"),
        #[cfg(windows)]
        ("\\Leading", "\\Leading\\RelPath"),
        ("Combo/Path\\", &format!("{SEP}RelPath")),
        ("Combo\\Path/", &format!("{SEP}RelPath")),
        ("/", "/RelPath"),
        #[cfg(not(windows))]
        ("\\", "/RelPath"),
        #[cfg(windows)]
        ("\\", "\\RelPath"),
        ("//", "//RelPath"),
        #[cfg(not(windows))]
        ("\\\\", "/RelPath"),
        #[cfg(windows)]
        ("\\\\", "\\\\RelPath"),
        ("///", "///RelPath"),
        #[cfg(not(windows))]
        ("\\\\\\", "/RelPath"),
        #[cfg(windows)]
        ("\\\\\\", "\\\\\\RelPath"),
        ("/.", &format!("/.{SEP}RelPath")),
        #[cfg(not(windows))]
        ("\\.", "/RelPath"),
        #[cfg(windows)]
        ("\\.", "\\.\\RelPath"),
        ("/..", &format!("/..{SEP}RelPath")),
        #[cfg(not(windows))]
        ("\\..", "/RelPath"),
        #[cfg(windows)]
        ("\\..", "\\..\\RelPath"),
        ("//.", &format!("//.{SEP}RelPath")),
        #[cfg(not(windows))]
        ("\\\\.", "/RelPath"),
        #[cfg(windows)]
        ("\\\\.", "\\\\.\\RelPath"),
        ("//..", &format!("//..{SEP}RelPath")),
        #[cfg(not(windows))]
        ("\\\\..", "/RelPath"),
        #[cfg(windows)]
        ("\\\\..", "\\\\..\\RelPath"),
        #[cfg(not(windows))]
        ("C:/", "/RelPath"),
        #[cfg(windows)]
        ("C:/", "C:/RelPath"),
        #[cfg(not(windows))]
        ("C:\\", "/RelPath"),
        #[cfg(windows)]
        ("C:\\", "C:\\RelPath"),
        #[cfg(not(windows))]
        ("c:/", "/RelPath"),
        #[cfg(windows)]
        ("c:/", "c:/RelPath"),
        #[cfg(not(windows))]
        ("c:\\", "/RelPath"),
        #[cfg(windows)]
        ("c:\\", "c:\\RelPath"),
        #[cfg(not(windows))]
        ("X:\\", "/RelPath"),
        #[cfg(windows)]
        ("X:\\", "X:\\RelPath"),
        ("C:", &format!("{SEP}RelPath")),
        ("c:", &format!("{SEP}RelPath")),
        ("C:WithoutRoot", &format!("{SEP}RelPath")),
        #[cfg(not(windows))]
        ("C:\\.", "/RelPath"),
        #[cfg(windows)]
        ("C:\\.", "C:\\.\\RelPath"),
        #[cfg(not(windows))]
        ("C:\\..", "/RelPath"),
        #[cfg(windows)]
        ("C:\\..", "C:\\..\\RelPath"),
        ("/C:/", "/C:/RelPath"),
        #[cfg(not(windows))]
        ("\\C:\\", "/RelPath"),
        #[cfg(windows)]
        ("\\C:\\", "\\C:\\RelPath"),
        ("/C:", &format!("/C:{SEP}RelPath")),
        #[cfg(not(windows))]
        ("\\C:", "/RelPath"),
        #[cfg(windows)]
        ("\\C:", "\\C:\\RelPath"),
        ("//Server/Share/", "//Server/Share/RelPath"),
        #[cfg(not(windows))]
        ("\\\\Server\\Share\\", "/RelPath"),
        #[cfg(windows)]
        ("\\\\Server\\Share\\", "\\\\Server\\Share\\RelPath"),
        ("//Server/Share", &format!("//Server/Share{SEP}RelPath")),
        #[cfg(not(windows))]
        ("\\\\Server\\Share", "/RelPath"),
        #[cfg(windows)]
        ("\\\\Server\\Share", "\\\\Server\\Share\\RelPath"),
        #[cfg(not(windows))]
        ("\\\\Server\\Share\\misc.txt", "/RelPath"),
        #[cfg(windows)]
        (
            "\\\\Server\\Share\\misc.txt",
            "\\\\Server\\Share\\misc.txt\\RelPath"
        ),
        #[cfg(not(windows))]
        ("\\\\Server\\misc.txt", "/RelPath"),
        #[cfg(windows)]
        ("\\\\Server\\misc.txt", "\\\\Server\\misc.txt\\RelPath"),
        ("///Server/Share", &format!("///Server/Share{SEP}RelPath")),
        #[cfg(not(windows))]
        ("\\\\Server\\Share\\C:", "/RelPath"),
        #[cfg(windows)]
        ("\\\\Server\\Share\\C:", "\\\\Server\\Share\\C:\\RelPath"),
        #[cfg(not(windows))]
        ("\\\\\\Server\\Share", "/RelPath"),
        #[cfg(windows)]
        ("\\\\\\Server\\Share", "\\\\\\Server\\Share\\RelPath"),
        #[cfg(not(windows))]
        ("\\\\127.0.0.1\\Share", "/RelPath"),
        #[cfg(windows)]
        ("\\\\127.0.0.1\\Share", "\\\\127.0.0.1\\Share\\RelPath"),
        #[cfg(not(windows))]
        ("\\\\Server\\C$\\", "/RelPath"),
        #[cfg(windows)]
        ("\\\\Server\\C$\\", "\\\\Server\\C$\\RelPath"),
        #[cfg(not(windows))]
        ("\\\\Server\\C$", "/RelPath"),
        #[cfg(windows)]
        ("\\\\Server\\C$", "\\\\Server\\C$\\RelPath"),
        #[cfg(not(windows))]
        ("\\\\Server\\C:\\", "/RelPath"),
        #[cfg(windows)]
        ("\\\\Server\\C:\\", "\\\\Server\\C:\\RelPath"),
        #[cfg(not(windows))]
        ("\\\\Server\\C:", "/RelPath"),
        #[cfg(windows)]
        ("\\\\Server\\C:", "\\\\Server\\C:\\RelPath"),
        #[cfg(not(windows))]
        ("\\\\Server\\C$\\.", "/RelPath"),
        #[cfg(windows)]
        ("\\\\Server\\C$\\.", "\\\\Server\\C$\\.\\RelPath"),
        #[cfg(not(windows))]
        ("\\\\Server\\C$\\..", "/RelPath"),
        #[cfg(windows)]
        ("\\\\Server\\C$\\..", "\\\\Server\\C$\\..\\RelPath"),
        #[cfg(not(windows))]
        ("\\\\.\\.", "/RelPath"),
        #[cfg(windows)]
        ("\\\\.\\.", "\\\\.\\.\\RelPath"),
        #[cfg(not(windows))]
        ("\\\\..\\..", "/RelPath"),
        #[cfg(windows)]
        ("\\\\..\\..", "\\\\..\\..\\RelPath"),
        ("//?/", "//?/RelPath"),
        #[cfg(not(windows))]
        ("\\\\?\\", "/RelPath"),
        #[cfg(windows)]
        ("\\\\?\\", "\\\\?\\RelPath"),
        ("//?//", "//?//RelPath"),
        #[cfg(not(windows))]
        ("\\\\?\\\\", "/RelPath"),
        #[cfg(windows)]
        ("\\\\?\\\\", "\\\\?\\\\RelPath"),
        ("//?/C:/", "//?/C:/RelPath"),
        #[cfg(not(windows))]
        ("\\\\?\\C:\\", "/RelPath"),
        #[cfg(windows)]
        ("\\\\?\\C:\\", "\\\\?\\C:\\RelPath"),
        #[cfg(not(windows))]
        ("\\\\?\\C:", "/RelPath"),
        #[cfg(windows)]
        ("\\\\?\\C:", "\\\\?\\C:\\RelPath"),
        #[cfg(not(windows))]
        ("\\\\?\\C:WithoutRoot", "/RelPath"),
        #[cfg(windows)]
        ("\\\\?\\C:WithoutRoot", "\\\\?\\C:WithoutRoot\\RelPath"),
        #[cfg(not(windows))]
        ("\\\\?\\.", "/RelPath"),
        #[cfg(windows)]
        ("\\\\?\\.", "\\\\?\\.\\RelPath"),
        #[cfg(not(windows))]
        ("\\\\?\\..", "/RelPath"),
        #[cfg(windows)]
        ("\\\\?\\..", "\\\\?\\..\\RelPath"),
        ("//./", "//./RelPath"),
        #[cfg(not(windows))]
        ("\\\\.\\", "/RelPath"),
        #[cfg(windows)]
        ("\\\\.\\", "\\\\.\\RelPath"),
        ("//.//", "//.//RelPath"),
        #[cfg(not(windows))]
        ("\\\\.\\\\", "/RelPath"),
        #[cfg(windows)]
        ("\\\\.\\\\", "\\\\.\\\\RelPath"),
        ("//./C:/", "//./C:/RelPath"),
        #[cfg(not(windows))]
        ("\\\\.\\C:\\", "/RelPath"),
        #[cfg(windows)]
        ("\\\\.\\C:\\", "\\\\.\\C:\\RelPath"),
        #[cfg(not(windows))]
        ("\\\\.\\C:", "/RelPath"),
        #[cfg(windows)]
        ("\\\\.\\C:", "\\\\.\\C:\\RelPath"),
        #[cfg(not(windows))]
        ("\\\\.\\C:WithoutRoot", "/RelPath"),
        #[cfg(windows)]
        ("\\\\.\\C:WithoutRoot", "\\\\.\\C:WithoutRoot\\RelPath"),
        ("//?/UNC/Server/Share", &format!("//?/UNC/Server/Share{SEP}RelPath")),
        #[cfg(not(windows))]
        ("\\\\?\\UNC\\Server\\Share", "/RelPath"),
        #[cfg(windows)]
        ("\\\\?\\UNC\\Server\\Share", "\\\\?\\UNC\\Server\\Share\\RelPath"),
        ("//?/UNC/Server/C$", &format!("//?/UNC/Server/C${SEP}RelPath")),
        #[cfg(not(windows))]
        ("\\\\?\\UNC\\Server\\C$", "/RelPath"),
        #[cfg(windows)]
        ("\\\\?\\UNC\\Server\\C$", "\\\\?\\UNC\\Server\\C$\\RelPath"),
        #[cfg(not(windows))]
        ("\\\\?\\UNC\\Server\\C$\\", "/RelPath"),
        #[cfg(windows)]
        ("\\\\?\\UNC\\Server\\C$\\", "\\\\?\\UNC\\Server\\C$\\RelPath"),
        #[cfg(not(windows))]
        ("\\\\?\\unc\\Server\\c$\\", "/RelPath"),
        #[cfg(windows)]
        ("\\\\?\\unc\\Server\\c$\\", "\\\\?\\unc\\Server\\c$\\RelPath"),
        #[cfg(not(windows))]
        ("\\\\?\\UNC\\Server\\C:", "/RelPath"),
        #[cfg(windows)]
        ("\\\\?\\UNC\\Server\\C:", "\\\\?\\UNC\\Server\\C:\\RelPath"),
        #[cfg(not(windows))]
        ("\\\\?\\UNC\\Server\\C$\\.", "/RelPath"),
        #[cfg(windows)]
        ("\\\\?\\UNC\\Server\\C$\\.", "\\\\?\\UNC\\Server\\C$\\.\\RelPath"),
        #[cfg(not(windows))]
        ("\\\\?\\UNC\\Server\\C$\\..", "/RelPath"),
        #[cfg(windows)]
        ("\\\\?\\UNC\\Server\\C$\\..", "\\\\?\\UNC\\Server\\C$\\..\\RelPath"),
        #[cfg(not(windows))]
        ("\\\\?\\UNC\\.\\.", "/RelPath"),
        #[cfg(windows)]
        ("\\\\?\\UNC\\.\\.", "\\\\?\\UNC\\.\\.\\RelPath"),
        #[cfg(not(windows))]
        ("\\\\?\\UNC\\..\\..", "/RelPath"),
        #[cfg(windows)]
        ("\\\\?\\UNC\\..\\..", "\\\\?\\UNC\\..\\..\\RelPath"),
        ("//./UNC/Server/Share", &format!("//./UNC/Server/Share{SEP}RelPath")),
        #[cfg(not(windows))]
        ("\\\\.\\UNC\\Server\\Share", "/RelPath"),
        #[cfg(windows)]
        ("\\\\.\\UNC\\Server\\Share", "\\\\.\\UNC\\Server\\Share\\RelPath"),
        ("//./UNC/Server/C$", &format!("//./UNC/Server/C${SEP}RelPath")),
        #[cfg(not(windows))]
        ("\\\\.\\UNC\\Server\\C$", "/RelPath"),
        #[cfg(windows)]
        ("\\\\.\\UNC\\Server\\C$", "\\\\.\\UNC\\Server\\C$\\RelPath"),
        #[cfg(not(windows))]
        ("\\\\.\\UNC\\Server\\C$\\", "/RelPath"),
        #[cfg(windows)]
        ("\\\\.\\UNC\\Server\\C$\\", "\\\\.\\UNC\\Server\\C$\\RelPath"),
        #[cfg(not(windows))]
        ("\\\\.\\unc\\Server\\c$\\", "/RelPath"),
        #[cfg(windows)]
        ("\\\\.\\unc\\Server\\c$\\", "\\\\.\\unc\\Server\\c$\\RelPath"),
        #[cfg(not(windows))]
        ("\\\\.\\UNC\\Server\\C:", "/RelPath"),
        #[cfg(windows)]
        ("\\\\.\\UNC\\Server\\C:", "\\\\.\\UNC\\Server\\C:\\RelPath"),
        #[cfg(not(windows))]
        ("\\\\.\\UNC\\Server\\C$\\.", "/RelPath"),
        #[cfg(windows)]
        ("\\\\.\\UNC\\Server\\C$\\.", "\\\\.\\UNC\\Server\\C$\\.\\RelPath"),
        #[cfg(not(windows))]
        ("\\\\.\\UNC\\Server\\C$\\..", "/RelPath"),
        #[cfg(windows)]
        ("\\\\.\\UNC\\Server\\C$\\..", "\\\\.\\UNC\\Server\\C$\\..\\RelPath"),
        #[cfg(not(windows))]
        ("\\\\.\\UNC\\.\\.", "/RelPath"),
        #[cfg(windows)]
        ("\\\\.\\UNC\\.\\.", "\\\\.\\UNC\\.\\.\\RelPath"),
        #[cfg(not(windows))]
        ("\\\\.\\UNC\\..\\..", "/RelPath"),
        #[cfg(windows)]
        ("\\\\.\\UNC\\..\\..", "\\\\.\\UNC\\..\\..\\RelPath"),
        (
            "//?/Volume{12a34b56-78c9-012d-ef3g-45678hij9012}",
            &format!(
                "//?/Volume{{12a34b56-78c9-012d-ef3g-45678hij9012}}{SEP}RelPath"
            ),
        ),
        #[cfg(not(windows))]
        ("\\\\?\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}", "/RelPath"),
        #[cfg(windows)]
        (
            "\\\\?\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}",
            "\\\\?\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}\\RelPath",
        ),
        (
            "//?/Volume{12a34b56-78c9-012d-ef3g-45678hij9012}/",
            "//?/Volume{12a34b56-78c9-012d-ef3g-45678hij9012}/RelPath",
        ),
        #[cfg(not(windows))]
        ("\\\\?\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}\\", "/RelPath"),
        #[cfg(windows)]
        (
            "\\\\?\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}\\",
            "\\\\?\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}\\RelPath",
        ),
        (
            "//?/Volume{12a34b56-78c9-012d-ef3g-45678hij9012}/misc.txt",
            &format!(
                "//?/Volume{{12a34b56-78c9-012d-ef3g-45678hij9012}}/misc.txt{SEP}RelPath"
            ),
        ),
        #[cfg(not(windows))]
        ("\\\\?\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}\\misc.txt", "/RelPath"),
        #[cfg(windows)]
        (
            "\\\\?\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}\\misc.txt",
            "\\\\?\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}\\misc.txt\\RelPath",
        ),
        (
            "//./Volume{12a34b56-78c9-012d-ef3g-45678hij9012}",
            &format!(
                "//./Volume{{12a34b56-78c9-012d-ef3g-45678hij9012}}{SEP}RelPath"
            ),
        ),
        #[cfg(not(windows))]
        ("\\\\.\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}", "/RelPath"),
        #[cfg(windows)]
        (
            "\\\\.\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}",
            "\\\\.\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}\\RelPath",
        ),
        (
            "//./Volume{12a34b56-78c9-012d-ef3g-45678hij9012}/",
            "//./Volume{12a34b56-78c9-012d-ef3g-45678hij9012}/RelPath",
        ),
        #[cfg(not(windows))]
        ("\\\\.\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}\\", "/RelPath"),
        #[cfg(windows)]
        (
            "\\\\.\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}\\",
            "\\\\.\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}\\RelPath",
        ),
        (
            "//./Volume{12a34b56-78c9-012d-ef3g-45678hij9012}/misc.txt",
            &format!(
                "//./Volume{{12a34b56-78c9-012d-ef3g-45678hij9012}}/misc.txt{SEP}RelPath"
            ),
        ),
        #[cfg(not(windows))]
        ("\\\\.\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}\\misc.txt", "/RelPath"),
        #[cfg(windows)]
        (
            "\\\\.\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}\\misc.txt",
            "\\\\.\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}\\misc.txt\\RelPath",
        ),
        ("file:///../File/Protocol", &format!("{SEP}RelPath")),
        ("https://www.website.com", &format!("{SEP}RelPath")),
    ];

    pairs.iter().map(|p| (p.0.to_string(), p.1.to_string())).collect()
}
