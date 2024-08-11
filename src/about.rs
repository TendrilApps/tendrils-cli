//! Functionality for getting license info, third-party software licenses, etc.

use crate::ansi_hyperlink;

fn cli_license_type() -> String {
    String::from(env!["CARGO_PKG_LICENSE"])
}

fn cli_repo() -> String {
    String::from(env!["CARGO_PKG_REPOSITORY"])
}

fn cli_version() -> String {
    String::from(env!["CARGO_PKG_VERSION"])
}

pub(crate) fn cli_license() -> String {
    let license_type = cli_license_type();
    let version = cli_version();
    let repo_url = cli_repo();
    let mut url = format!["{repo_url}/blob/{version}/LICENSE.md"];
    url = ansi_hyperlink(&url, &url);

    format![
        "td is licensed under a {license_type} license.\n\nThe license text \
         is here:\n{url}"
    ]
}

pub(crate) fn cli_acknowledgements() -> String {
    let version = cli_version();
    let repo_url = cli_repo();
    let mut url = format!["{repo_url}/blob/{version}/LICENSE-3RD-PARTY.md"];
    url = ansi_hyperlink(&url, &url);

    format![
        "td uses several open source dependencies.\n\nTheir acknowledgements \
         and licensing information are here:\n{url}"
    ]
}
