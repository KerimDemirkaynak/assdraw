//! Embeds the application icon into the Windows executable (Explorer, taskbar, Alt+Tab).

fn main() {
    println!("cargo:rerun-if-changed=assets/assdraw.ico");
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/assdraw.ico");
        if let Err(e) = res.compile() {
            // Never fail the build just because the icon could not be embedded.
            println!("cargo:warning=could not embed icon: {e}");
        }
    }
}
