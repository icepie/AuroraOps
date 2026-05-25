use std::env;
use std::path::PathBuf;

fn main() {
    let pkg = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());

    cmake::Config::new("fastfetch")
        .out_dir(out.clone())
        .build_target("fastfetch-vendor")
        .define("ENABLE_LTO", "OFF")
        .build();

    bindgen::Builder::default()
        .header("fastfetch/wrapper.h")
        .clang_arg("-D_GNU_SOURCE")
        .clang_arg(format!("-I{}", pkg.join("fastfetch/vendor/src").display()))
        .clang_arg(format!("-I{}", out.join("build/vendor").display()))
        .layout_tests(false)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out.join("bindings.rs"))
        .expect("Unable to write bindings");

    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rustc-link-search={}", out.join("build").display());
    println!("cargo:rustc-link-lib=static=fastfetch-vendor");
}
