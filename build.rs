fn main() {
    println!("cargo:rerun-if-changed=assets/App.ico");
    println!("cargo:rerun-if-changed=assets/app.manifest");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/App.ico")
            .set_manifest_file("assets/app.manifest")
            .set("ProductName", "Easy MIC Booster")
            .set("FileDescription", "Easy MIC Booster")
            .set("CompanyName", "Seiya Funaoka")
            .set("LegalCopyright", "MIT License");
        if let Err(e) = res.compile() {
            println!("cargo:warning=failed to embed Windows resources: {e}");
        }
    }
}
