#[cfg(windows)]
fn main() {
    let mut res = winresource::WindowsResource::new();
    res.set_icon("./assets/logo.ico");
    res.compile().expect("Failed to run Windows Resource (rc.exe)");
}

#[cfg(unix)]
fn main() {}
