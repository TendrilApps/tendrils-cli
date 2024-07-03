#[cfg(windows)]
fn main() {
    #[cfg(feature = "metadata")] {
        let mut res = winresource::WindowsResource::new();

        #[cfg(feature = "logo")]
        res.set_icon("./assets/logo.ico");
        res.set("FileDescription", "Tendrils");
        res.set("ProductName", "Tendrils");
        res.set("OriginalFilename", "td.exe");
        res.set("CompanyName", "https://github.com/TendrilApps");
        res.compile().expect("Failed to run rc.exe");
    }
}

#[cfg(unix)]
fn main() {}
