fn main() {
    println!("cargo:rerun-if-changed=assets/App.ico");
    println!("cargo:rerun-if-changed=assets/app.manifest");
    // Mirrors the target condition on the deep_filter dependency in Cargo.toml.
    println!("cargo::rustc-check-cfg=cfg(deepfilter)");
    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    if !(os == "windows" && arch == "aarch64") {
        println!("cargo:rustc-cfg=deepfilter");
    }
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/App.ico")
            .set_manifest_file("assets/app.manifest")
            .set("ProductName", "Easy MIC Booster")
            .set("FileDescription", "Easy MIC Booster")
            .set("CompanyName", "Seiya Funaoka")
            .set("LegalCopyright", "MIT OR Apache-2.0");
        if let Err(e) = res.compile() {
            println!("cargo:warning=failed to embed Windows resources: {e}");
        }
    }
}
