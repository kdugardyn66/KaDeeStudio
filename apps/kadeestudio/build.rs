//! Embed the app icon and version info into the Windows executable.
//! Does nothing on macOS and Linux (they get the icon from the .app bundle / .desktop file).

fn main() {
    println!("cargo:rerun-if-changed=../../assets/app-icon/kadeestudio.ico");
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("../../assets/app-icon/kadeestudio.ico");
        res.set("ProductName", "KadeeStudio");
        res.set("FileDescription", "KadeeStudio");
        res.set("CompanyName", "Kris Dugardyn");
        res.set("LegalCopyright", "MIT OR Apache-2.0");
        if let Err(e) = res.compile() {
            // A missing resource compiler should not break the build; the exe just has no icon.
            println!("cargo:warning=could not embed the Windows icon: {e}");
        }
    }
}
