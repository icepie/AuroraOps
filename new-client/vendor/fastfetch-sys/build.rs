use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let pkg = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());

    cmake::Config::new("fastfetch")
        .out_dir(out.clone())
        .build_target("fastfetch-vendor")
        .define("ENABLE_LTO", "OFF")
        .build();

    let bindings = out.join("bindings.rs");
    println!("cargo:rerun-if-env-changed=FASTFETCH_SYS_BINDGEN");
    println!("cargo:rerun-if-changed=src/bindings/linux_64.rs");
    if use_pregenerated_bindings() {
        fs::copy(pkg.join("src/bindings/linux_64.rs"), &bindings)
            .expect("Unable to copy pregenerated bindings");
    } else {
        generate_bindings(&pkg, &out, &bindings);
    }

    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rustc-link-search={}", out.join("build").display());
    println!("cargo:rustc-link-lib=static=fastfetch-vendor");
}

fn use_pregenerated_bindings() -> bool {
    if env::var_os("FASTFETCH_SYS_BINDGEN").is_some() {
        return false;
    }

    env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("linux")
        && env::var("CARGO_CFG_TARGET_POINTER_WIDTH").as_deref() == Ok("64")
}

fn generate_bindings(pkg: &PathBuf, out: &PathBuf, bindings: &PathBuf) {
    bindgen::Builder::default()
        .header("fastfetch/wrapper.h")
        .clang_arg("-D_GNU_SOURCE")
        .clang_arg(format!("-I{}", pkg.join("fastfetch/vendor/src").display()))
        .clang_arg(format!("-I{}", out.join("build/vendor").display()))
        .layout_tests(false)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(bindings)
        .expect("Unable to write bindings");
}
