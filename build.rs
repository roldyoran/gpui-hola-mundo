fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");
    if matches!(
        std::env::var("CARGO_CFG_TARGET_OS").as_deref(),
        Ok("windows")
    ) {
        let mut resource = winresource::WindowsResource::new();
        resource.set_icon("assets/icon.ico");
        resource.set("ProductName", "PNGTuber Lite");
        resource.set(
            "FileDescription",
            "PNGTuber Lite - overlay PNGTuber ultraligero",
        );
        resource.set("OriginalFilename", "pngtuber-lite.exe");
        resource.set("LegalCopyright", "MIT - roldyoran");
        resource
            .compile()
            .expect("no se pudo compilar el recurso de Windows");
    }
}
