fn main() {
    println!("cargo::rustc-check-cfg=cfg(cargo_build)");

    // Poor man's attempt to check if the current build is started by meson
    let meson_build = std::path::Path::new("src/config.rs").exists();

    if !meson_build {
        println!("cargo:rustc-cfg=cargo_build");
    }

    let mut gresource_path = "data/resources.gresource.xml";

    // Detect if we're building from workspace root
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    if let Some(workspace_root) = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
    {
        if workspace_root.join("Cargo.toml").exists() && workspace_root.join("client").exists() {
            gresource_path = "demo/client/data/resources.gresource.xml";
        }
    }

    println!("cargo:rustc-env=GRESOURCE_PATH={}", gresource_path);
}
