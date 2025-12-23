fn main() {
    println!("cargo::rustc-check-cfg=cfg(cargo_build)");

    // Poor man's attempt to check if the current build is started by meson
    let meson_build = std::path::Path::new("src/config.rs").exists();

    if !meson_build {
        println!("cargo:rustc-cfg=cargo_build");
    }
}
