use crate::path_ext::PathExt;
use crate::path_ext::tests::test_paths::cases;
use crate::test_utils::non_utf_8_text;
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{PathBuf, MAIN_SEPARATOR_STR as SEP};

/// Checks that each of the subscribed sets of test cases has a corresponding 
/// expected output.
#[test]
fn all_test_cases_covered() {
    let mut map = HashMap::new();

    for pair in given_and_exp() {
        assert!(map.insert(pair.0.to_string(), pair.1.to_string()).is_none());
    }

    // This is where the different sets of test cases are "subscribed to" for
    // this module.
    for case in cases() {
        assert!(map.contains_key(case), "Missing case: {}", case);
    }
}

#[test]
fn returns_expected() {
    for pair in given_and_exp() {
        let given = PathBuf::from(&pair.0);

        let actual = given.to_absolute();

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
    let mut expected_str = OsString::from(SEP);
    expected_str.push(non_utf_8_text());

    let actual = PathBuf::from(non_utf_8_text()).to_absolute();

    assert_eq!(actual.as_os_str(), expected_str);
}

fn given_and_exp() -> Vec<(String, String)> {
    // (Given, Expected)
    let pairs = [
        ("", SEP),
        (".", &format!("{SEP}.")),
        ("..", &format!("{SEP}..")),
        ("./", &format!("{SEP}./")),
        (".\\", &format!("{SEP}.\\")),
        ("../", &format!("{SEP}../")),
        ("..\\", &format!("{SEP}..\\")),
        ("Plain", &format!("{SEP}Plain")),
        ("Trailing/", &format!("{SEP}Trailing/")),
        ("Trailing\\", &format!("{SEP}Trailing\\")),
        ("/Leading", "/Leading"),
        #[cfg(not(windows))]
        ("\\Leading", "/\\Leading"),
        #[cfg(windows)]
        ("\\Leading", "\\Leading"),
        ("Combo/Path\\", &format!("{SEP}Combo/Path\\")),
        ("Combo\\Path/", &format!("{SEP}Combo\\Path/")),
        ("/", "/"),
        #[cfg(not(windows))]
        ("\\", "/\\"),
        #[cfg(windows)]
        ("\\", "\\"),
        ("//", "//"),
        #[cfg(not(windows))]
        ("\\\\", "/\\\\"),
        #[cfg(windows)]
        ("\\\\", "\\\\"),
        ("///", "///"),
        #[cfg(not(windows))]
        ("\\\\\\", "/\\\\\\"),
        #[cfg(windows)]
        ("\\\\\\", "\\\\\\"),
        ("/.", "/."),
        #[cfg(not(windows))]
        ("\\.", "/\\."),
        #[cfg(windows)]
        ("\\.", "\\."),
        ("/..", "/.."),
        #[cfg(not(windows))]
        ("\\..", "/\\.."),
        #[cfg(windows)]
        ("\\..", "\\.."),
        ("//.", "//."),
        #[cfg(not(windows))]
        ("\\\\.", "/\\\\."),
        #[cfg(windows)]
        ("\\\\.", "\\\\."),
        ("//..", "//.."),
        #[cfg(not(windows))]
        ("\\\\..", "/\\\\.."),
        #[cfg(windows)]
        ("\\\\..", "\\\\.."),
        #[cfg(not(windows))]
        ("C:/", "/C:/"),
        #[cfg(windows)]
        ("C:/", "C:/"),
        #[cfg(not(windows))]
        ("C:\\", "/C:\\"),
        #[cfg(windows)]
        ("C:\\", "C:\\"),
        #[cfg(not(windows))]
        ("c:/", "/c:/"),
        #[cfg(windows)]
        ("c:/", "c:/"),
        #[cfg(not(windows))]
        ("c:\\", "/c:\\"),
        #[cfg(windows)]
        ("c:\\", "c:\\"),
        #[cfg(not(windows))]
        ("X:\\", "/X:\\"),
        #[cfg(windows)]
        ("X:\\", "X:\\"),
        ("C:", &format!("{SEP}C:")),
        ("c:", &format!("{SEP}c:")),
        ("C:WithoutRoot", &format!("{SEP}C:WithoutRoot")),
        #[cfg(not(windows))]
        ("C:\\.", "/C:\\."),
        #[cfg(windows)]
        ("C:\\.", "C:\\."),
        #[cfg(not(windows))]
        ("C:\\..", "/C:\\.."),
        #[cfg(windows)]
        ("C:\\..", "C:\\.."),
        ("/C:/", "/C:/"),
        #[cfg(not(windows))]
        ("\\C:\\", "/\\C:\\"),
        #[cfg(windows)]
        ("\\C:\\", "\\C:\\"),
        ("/C:", "/C:"),
        #[cfg(not(windows))]
        ("\\C:", "/\\C:"),
        #[cfg(windows)]
        ("\\C:", "\\C:"),
        ("//Server/Share/", "//Server/Share/"),
        #[cfg(not(windows))]
        ("\\\\Server\\Share\\", "/\\\\Server\\Share\\"),
        #[cfg(windows)]
        ("\\\\Server\\Share\\", "\\\\Server\\Share\\"),
        ("//Server/Share", "//Server/Share"),
        #[cfg(not(windows))]
        ("\\\\Server\\Share", "/\\\\Server\\Share"),
        #[cfg(windows)]
        ("\\\\Server\\Share", "\\\\Server\\Share"),
        #[cfg(not(windows))]
        ("\\\\Server\\Share\\misc.txt", "/\\\\Server\\Share\\misc.txt"),
        #[cfg(windows)]
        ("\\\\Server\\Share\\misc.txt", "\\\\Server\\Share\\misc.txt"),
        #[cfg(not(windows))]
        ("\\\\Server\\misc.txt", "/\\\\Server\\misc.txt"),
        #[cfg(windows)]
        ("\\\\Server\\misc.txt", "\\\\Server\\misc.txt"),
        ("///Server/Share", "///Server/Share"),
        #[cfg(not(windows))]
        ("\\\\Server\\Share\\C:", "/\\\\Server\\Share\\C:"),
        #[cfg(windows)]
        ("\\\\Server\\Share\\C:", "\\\\Server\\Share\\C:"),
        #[cfg(not(windows))]
        ("\\\\\\Server\\Share", "/\\\\\\Server\\Share"),
        #[cfg(windows)]
        ("\\\\\\Server\\Share", "\\\\\\Server\\Share"),
        #[cfg(not(windows))]
        ("\\\\127.0.0.1\\Share", "/\\\\127.0.0.1\\Share"),
        #[cfg(windows)]
        ("\\\\127.0.0.1\\Share", "\\\\127.0.0.1\\Share"),
        #[cfg(not(windows))]
        ("\\\\Server\\C$\\", "/\\\\Server\\C$\\"),
        #[cfg(windows)]
        ("\\\\Server\\C$\\", "\\\\Server\\C$\\"),
        #[cfg(not(windows))]
        ("\\\\Server\\C$", "/\\\\Server\\C$"),
        #[cfg(windows)]
        ("\\\\Server\\C$", "\\\\Server\\C$"),
        #[cfg(not(windows))]
        ("\\\\Server\\C:\\", "/\\\\Server\\C:\\"),
        #[cfg(windows)]
        ("\\\\Server\\C:\\", "\\\\Server\\C:\\"),
        #[cfg(not(windows))]
        ("\\\\Server\\C:", "/\\\\Server\\C:"),
        #[cfg(windows)]
        ("\\\\Server\\C:", "\\\\Server\\C:"),
        #[cfg(not(windows))]
        ("\\\\Server\\C$\\.", "/\\\\Server\\C$\\."),
        #[cfg(windows)]
        ("\\\\Server\\C$\\.", "\\\\Server\\C$\\."),
        #[cfg(not(windows))]
        ("\\\\Server\\C$\\..", "/\\\\Server\\C$\\.."),
        #[cfg(windows)]
        ("\\\\Server\\C$\\..", "\\\\Server\\C$\\.."),
        #[cfg(not(windows))]
        ("\\\\.\\.", "/\\\\.\\."),
        #[cfg(windows)]
        ("\\\\.\\.", "\\\\.\\."),
        #[cfg(not(windows))]
        ("\\\\..\\..", "/\\\\..\\.."),
        #[cfg(windows)]
        ("\\\\..\\..", "\\\\..\\.."),
        ("//?/", "//?/"),
        #[cfg(not(windows))]
        ("\\\\?\\", "/\\\\?\\"),
        #[cfg(windows)]
        ("\\\\?\\", "\\\\?\\"),
        ("//?//", "//?//"),
        #[cfg(not(windows))]
        ("\\\\?\\\\", "/\\\\?\\\\"),
        #[cfg(windows)]
        ("\\\\?\\\\", "\\\\?\\\\"),
        ("//?/C:/", "//?/C:/"),
        #[cfg(not(windows))]
        ("\\\\?\\C:\\", "/\\\\?\\C:\\"),
        #[cfg(windows)]
        ("\\\\?\\C:\\", "\\\\?\\C:\\"),
        #[cfg(not(windows))]
        ("\\\\?\\C:", "/\\\\?\\C:"),
        #[cfg(windows)]
        ("\\\\?\\C:", "\\\\?\\C:"),
        #[cfg(not(windows))]
        ("\\\\?\\C:WithoutRoot", "/\\\\?\\C:WithoutRoot"),
        #[cfg(windows)]
        ("\\\\?\\C:WithoutRoot", "\\\\?\\C:WithoutRoot"),
        #[cfg(not(windows))]
        ("\\\\?\\.", "/\\\\?\\."),
        #[cfg(windows)]
        ("\\\\?\\.", "\\\\?\\."),
        #[cfg(not(windows))]
        ("\\\\?\\..", "/\\\\?\\.."),
        #[cfg(windows)]
        ("\\\\?\\..", "\\\\?\\.."),
        ("//./", "//./"),
        #[cfg(not(windows))]
        ("\\\\.\\", "/\\\\.\\"),
        #[cfg(windows)]
        ("\\\\.\\", "\\\\.\\"),
        ("//.//", "//.//"),
        #[cfg(not(windows))]
        ("\\\\.\\\\", "/\\\\.\\\\"),
        #[cfg(windows)]
        ("\\\\.\\\\", "\\\\.\\\\"),
        ("//./C:/", "//./C:/"),
        #[cfg(not(windows))]
        ("\\\\.\\C:\\", "/\\\\.\\C:\\"),
        #[cfg(windows)]
        ("\\\\.\\C:\\", "\\\\.\\C:\\"),
        #[cfg(not(windows))]
        ("\\\\.\\C:", "/\\\\.\\C:"),
        #[cfg(windows)]
        ("\\\\.\\C:", "\\\\.\\C:"),
        #[cfg(not(windows))]
        ("\\\\.\\C:WithoutRoot", "/\\\\.\\C:WithoutRoot"),
        #[cfg(windows)]
        ("\\\\.\\C:WithoutRoot", "\\\\.\\C:WithoutRoot"),
        ("//?/UNC/Server/Share", "//?/UNC/Server/Share"),
        #[cfg(not(windows))]
        ("\\\\?\\UNC\\Server\\Share", "/\\\\?\\UNC\\Server\\Share"),
        #[cfg(windows)]
        ("\\\\?\\UNC\\Server\\Share", "\\\\?\\UNC\\Server\\Share"),
        ("//?/UNC/Server/C$", "//?/UNC/Server/C$"),
        #[cfg(not(windows))]
        ("\\\\?\\UNC\\Server\\C$", "/\\\\?\\UNC\\Server\\C$"),
        #[cfg(windows)]
        ("\\\\?\\UNC\\Server\\C$", "\\\\?\\UNC\\Server\\C$"),
        #[cfg(not(windows))]
        ("\\\\?\\UNC\\Server\\C$\\", "/\\\\?\\UNC\\Server\\C$\\"),
        #[cfg(windows)]
        ("\\\\?\\UNC\\Server\\C$\\", "\\\\?\\UNC\\Server\\C$\\"),
        #[cfg(not(windows))]
        ("\\\\?\\unc\\Server\\c$\\", "/\\\\?\\unc\\Server\\c$\\"),
        #[cfg(windows)]
        ("\\\\?\\unc\\Server\\c$\\", "\\\\?\\unc\\Server\\c$\\"),
        #[cfg(not(windows))]
        ("\\\\?\\UNC\\Server\\C:", "/\\\\?\\UNC\\Server\\C:"),
        #[cfg(windows)]
        ("\\\\?\\UNC\\Server\\C:", "\\\\?\\UNC\\Server\\C:"),
        #[cfg(not(windows))]
        ("\\\\?\\UNC\\Server\\C$\\.", "/\\\\?\\UNC\\Server\\C$\\."),
        #[cfg(windows)]
        ("\\\\?\\UNC\\Server\\C$\\.", "\\\\?\\UNC\\Server\\C$\\."),
        #[cfg(not(windows))]
        ("\\\\?\\UNC\\Server\\C$\\..", "/\\\\?\\UNC\\Server\\C$\\.."),
        #[cfg(windows)]
        ("\\\\?\\UNC\\Server\\C$\\..", "\\\\?\\UNC\\Server\\C$\\.."),
        #[cfg(not(windows))]
        ("\\\\?\\UNC\\.\\.", "/\\\\?\\UNC\\.\\."),
        #[cfg(windows)]
        ("\\\\?\\UNC\\.\\.", "\\\\?\\UNC\\.\\."),
        #[cfg(not(windows))]
        ("\\\\?\\UNC\\..\\..", "/\\\\?\\UNC\\..\\.."),
        #[cfg(windows)]
        ("\\\\?\\UNC\\..\\..", "\\\\?\\UNC\\..\\.."),
        ("//./UNC/Server/Share", "//./UNC/Server/Share"),
        #[cfg(not(windows))]
        ("\\\\.\\UNC\\Server\\Share", "/\\\\.\\UNC\\Server\\Share"),
        #[cfg(windows)]
        ("\\\\.\\UNC\\Server\\Share", "\\\\.\\UNC\\Server\\Share"),
        ("//./UNC/Server/C$", "//./UNC/Server/C$"),
        #[cfg(not(windows))]
        ("\\\\.\\UNC\\Server\\C$", "/\\\\.\\UNC\\Server\\C$"),
        #[cfg(windows)]
        ("\\\\.\\UNC\\Server\\C$", "\\\\.\\UNC\\Server\\C$"),
        #[cfg(not(windows))]
        ("\\\\.\\UNC\\Server\\C$\\", "/\\\\.\\UNC\\Server\\C$\\"),
        #[cfg(windows)]
        ("\\\\.\\UNC\\Server\\C$\\", "\\\\.\\UNC\\Server\\C$\\"),
        #[cfg(not(windows))]
        ("\\\\.\\unc\\Server\\c$\\", "/\\\\.\\unc\\Server\\c$\\"),
        #[cfg(windows)]
        ("\\\\.\\unc\\Server\\c$\\", "\\\\.\\unc\\Server\\c$\\"),
        #[cfg(not(windows))]
        ("\\\\.\\UNC\\Server\\C:", "/\\\\.\\UNC\\Server\\C:"),
        #[cfg(windows)]
        ("\\\\.\\UNC\\Server\\C:", "\\\\.\\UNC\\Server\\C:"),
        #[cfg(not(windows))]
        ("\\\\.\\UNC\\Server\\C$\\.", "/\\\\.\\UNC\\Server\\C$\\."),
        #[cfg(windows)]
        ("\\\\.\\UNC\\Server\\C$\\.", "\\\\.\\UNC\\Server\\C$\\."),
        #[cfg(not(windows))]
        ("\\\\.\\UNC\\Server\\C$\\..", "/\\\\.\\UNC\\Server\\C$\\.."),
        #[cfg(windows)]
        ("\\\\.\\UNC\\Server\\C$\\..", "\\\\.\\UNC\\Server\\C$\\.."),
        #[cfg(not(windows))]
        ("\\\\.\\UNC\\.\\.", "/\\\\.\\UNC\\.\\."),
        #[cfg(windows)]
        ("\\\\.\\UNC\\.\\.", "\\\\.\\UNC\\.\\."),
        #[cfg(not(windows))]
        ("\\\\.\\UNC\\..\\..", "/\\\\.\\UNC\\..\\.."),
        #[cfg(windows)]
        ("\\\\.\\UNC\\..\\..", "\\\\.\\UNC\\..\\.."),
        (
            "//?/Volume{12a34b56-78c9-012d-ef3g-45678hij9012}",
            "//?/Volume{12a34b56-78c9-012d-ef3g-45678hij9012}",
        ),
        #[cfg(not(windows))]
        (
            "\\\\?\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}",
            "/\\\\?\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}",
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
            "/\\\\?\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}\\",
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
            "/\\\\?\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}\\misc.txt",
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
            "/\\\\.\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}",
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
            "/\\\\.\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}\\",
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
            "/\\\\.\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}\\misc.txt",
        ),
        #[cfg(windows)]
        (
            "\\\\.\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}\\misc.txt",
            "\\\\.\\Volume{12a34b56-78c9-012d-ef3g-45678hij9012}\\misc.txt",
        ),
        ("file:///../File/Protocol", &format!("{SEP}file:///../File/Protocol")),
        ("https://www.website.com", &format!("{SEP}https://www.website.com")),
    ];

    pairs.iter().map(|p| (p.0.to_string(), p.1.to_string())).collect()
}
