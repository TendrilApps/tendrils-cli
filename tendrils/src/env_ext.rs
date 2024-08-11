pub(crate) fn get_home_dir() -> Option<String> {
    use std::env::var_os;
    if let Some(v) = var_os("HOME") {
        return Some(v.to_string_lossy().into_owned());
    };
    match (var_os("HOMEDRIVE"), var_os("HOMEPATH")) {
        (Some(hd), Some(hp)) => {
            let mut combo = String::from(hd.to_string_lossy());
            combo.push_str(hp.to_string_lossy().as_ref());
            return Some(combo);
        }
        _ => None,
    }
}

/// Returns `true` if the current Tendrils process is capable
/// of creating symlinks.
///
/// This is mainly applicable on Windows, where creating symlinks
/// requires administrator priviledges, or enabling *Developer Mode*.
/// On Unix platforms this always returns `true`.
pub(crate) fn can_symlink() -> bool {
    #[cfg(windows)]
    match std::env::consts::FAMILY {
        "windows" => is_root::is_root() || is_dev_mode(),
        _ => true,
    }

    #[cfg(unix)]
    true
}

/// Returns `true` if *Developer Mode* is enabled on Windows.
/// Returns `false` if the setting cannot be determined for any reason.
#[cfg(windows)]
fn is_dev_mode() -> bool {
    use winreg::enums::HKEY_LOCAL_MACHINE;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let app_model = match hklm.open_subkey(
        "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\AppModelUnlock",
    ) {
        Ok(v) => v,
        _ => return false,
    };

    let reg_value: u32 =
        match app_model.get_value("AllowDevelopmentWithoutDevLicense") {
            Ok(v) => v,
            _ => return false,
        };

    reg_value == 1
}
