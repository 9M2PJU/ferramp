//! Embeds the application icon and version information in Windows builds.

fn main() {
    println!("cargo:rerun-if-changed=assets/icons/ferramp.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut res = winresource::WindowsResource::new();
    res.set_icon("assets/icons/ferramp.ico");
    res.set("FileDescription", "Ferramp");
    res.set("ProductName", "Ferramp");
    if let Err(e) = res.compile() {
        // Cross-checks without a resource compiler still build, just without an icon.
        println!("cargo:warning=no Windows resources embedded: {e}");
    }
}
