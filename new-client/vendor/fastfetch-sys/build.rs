use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let pkg = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=fastfetch/CMakeLists.txt");
    println!("cargo:rerun-if-changed=fastfetch/vendor/CMakeLists.txt");

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    let mut config = cmake::Config::new("fastfetch");
    config
        .out_dir(out.clone())
        .build_target("fastfetch-vendor")
        .define("ENABLE_LTO", "OFF")
        .define("BUILD_FLASHFETCH", "OFF")
        .define("BUILD_TESTS", "OFF")
        .define("ENABLE_IMAGEMAGICK7", "OFF")
        .define("ENABLE_IMAGEMAGICK6", "OFF")
        .define("ENABLE_CHAFA", "OFF")
        .define("ENABLE_ZLIB", "OFF")
        .define("ENABLE_OPENCL", "OFF")
        .define("ENABLE_VULKAN", "OFF")
        .define("ENABLE_EGL", "OFF");
    if target_os == "windows" {
        config.define("CMAKE_SYSTEM_NAME", "Windows");
        config.define("FASTFETCH_SYS_WINDOWS_AGENT_BUILD", "ON");
        if target_env != "msvc" {
            let shim_dir = out.join("windows-include-shim");
            create_windows_include_shims(&shim_dir);
            let include_flags = format!("-I{}", shim_dir.display());
            config.cflag(&include_flags).cxxflag(&include_flags);
        } else {
            config.generator_toolset("ClangCL,host=x64");
            for flag in [
                "-Dssize_t=intptr_t",
                "-Dstrcasecmp=_stricmp",
                "-Dstrncasecmp=_strnicmp",
                "/clang:-fshort-enums",
            ] {
                config.cflag(flag).cxxflag(flag);
            }
        }
        if target_arch == "x86_64" {
            config.define("CMAKE_SYSTEM_PROCESSOR", "x86_64");
            if target_env == "msvc" {
                config
                    .cflag("-D_M_AMD64=100")
                    .cflag("-D_M_X64=100")
                    .cflag("-D_AMD64_=1")
                    .cxxflag("-D_M_AMD64=100")
                    .cxxflag("-D_M_X64=100")
                    .cxxflag("-D_AMD64_=1");
            }
        } else if target_arch == "aarch64" {
            config.define("CMAKE_SYSTEM_PROCESSOR", "ARM64");
            if target_env == "msvc" {
                config
                    .cflag("-D_M_ARM64=1")
                    .cflag("-D_ARM64_=1")
                    .cxxflag("-D_M_ARM64=1")
                    .cxxflag("-D_ARM64_=1");
            }
        }
    } else if target_os == "macos" {
        config.define("CMAKE_SYSTEM_NAME", "Darwin");
        if target_arch == "aarch64" {
            config.define("CMAKE_SYSTEM_PROCESSOR", "arm64");
            config.define("CMAKE_OSX_ARCHITECTURES", "arm64");
        } else if target_arch == "x86_64" {
            config.define("CMAKE_SYSTEM_PROCESSOR", "x86_64");
            config.define("CMAKE_OSX_ARCHITECTURES", "x86_64");
        }
    }
    config.build();

    let bindings = out.join("bindings.rs");
    println!("cargo:rerun-if-env-changed=FASTFETCH_SYS_BINDGEN");
    println!("cargo:rerun-if-changed=src/bindings/linux_64.rs");
    if use_pregenerated_bindings() {
        fs::copy(pkg.join("src/bindings/linux_64.rs"), &bindings)
            .expect("Unable to copy pregenerated bindings");
    } else {
        generate_bindings(&pkg, &out, &bindings, &target_os, &target_arch, &target_env);
    }

    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rustc-link-search={}", out.join("build").display());
    println!("cargo:rustc-link-lib=static=fastfetch-vendor");
    if target_os == "windows" {
        for lib in [
            "dwmapi", "gdi32", "iphlpapi", "ole32", "oleaut32", "ws2_32", "ntdll", "version",
            "setupapi", "hid", "wtsapi32", "imagehlp", "cfgmgr32", "winbrand", "propsys",
            "secur32", "pdh", "wbemuuid", "uuid", "shlwapi",
        ] {
            println!("cargo:rustc-link-lib={lib}");
        }
    } else if target_os == "macos" {
        for framework in [
            "AVFoundation",
            "Cocoa",
            "CoreFoundation",
            "CoreAudio",
            "CoreMedia",
            "CoreVideo",
            "CoreWLAN",
            "IOBluetooth",
            "IOKit",
            "Metal",
            "OpenGL",
            "OpenCL",
            "SystemConfiguration",
        ] {
            println!("cargo:rustc-link-lib=framework={framework}");
        }
        // Cargo has no `weak_framework` link-lib kind; pass it through to ld.
        for framework in ["CoreDisplay", "DisplayServices", "MediaRemote"] {
            println!("cargo:rustc-link-arg=-Wl,-weak_framework,{framework}");
        }
    }
}

fn use_pregenerated_bindings() -> bool {
    if env::var_os("FASTFETCH_SYS_BINDGEN").is_some() {
        return false;
    }

    env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("linux")
        && env::var("CARGO_CFG_TARGET_POINTER_WIDTH").as_deref() == Ok("64")
}

fn generate_bindings(
    pkg: &PathBuf,
    out: &PathBuf,
    bindings: &PathBuf,
    target_os: &str,
    target_arch: &str,
    target_env: &str,
) {
    let mut builder = bindgen::Builder::default()
        .header("fastfetch/wrapper.h")
        .clang_arg("-D_GNU_SOURCE")
        .clang_arg(format!("-I{}", pkg.join("fastfetch/vendor/src").display()))
        .clang_arg(format!("-I{}", out.join("build/vendor").display()))
        .layout_tests(false)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));
    if target_os == "windows" {
        let triple = match (target_arch, target_env) {
            ("aarch64", "msvc") => "aarch64-pc-windows-msvc",
            ("aarch64", _) => "aarch64-w64-windows-gnu",
            ("x86_64", "msvc") => "x86_64-pc-windows-msvc",
            _ => "x86_64-w64-windows-gnu",
        };
        builder = builder
            .clang_arg(format!("--target={triple}"))
            .clang_arg("-DWIN32_LEAN_AND_MEAN")
            .clang_arg("-D_WIN32_WINNT=0x0A00")
            .clang_arg("-DNOMINMAX")
            .clang_arg("-DUNICODE");
    } else if target_os == "macos" {
        let triple = match target_arch {
            "aarch64" => "arm64-apple-macosx",
            _ => "x86_64-apple-macosx",
        };
        builder = builder
            .clang_arg(format!("--target={triple}"))
            .clang_arg("-D_DARWIN_C_SOURCE");
    }
    builder
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(bindings)
        .expect("Unable to write bindings");
}

fn create_windows_include_shims(dir: &PathBuf) {
    fs::create_dir_all(dir).expect("Unable to create Windows include shim directory");
    for (upper, lower) in [
        ("Windows.h", "windows.h"),
        ("Wbemidl.h", "wbemidl.h"),
        ("Objbase.h", "objbase.h"),
        ("OleAuto.h", "oleauto.h"),
        ("SetupAPI.h", "setupapi.h"),
        ("WinUser.h", "winuser.h"),
        ("Propkey.h", "propkey.h"),
        ("Cfgmgr32.h", "cfgmgr32.h"),
        ("Iphlpapi.h", "iphlpapi.h"),
        ("TlHelp32.h", "tlhelp32.h"),
        ("VersionHelpers.h", "versionhelpers.h"),
    ] {
        fs::write(dir.join(upper), format!("#include <{lower}>\n"))
            .expect("Unable to write Windows include shim");
    }
    fs::write(dir.join("tbs.h"), "#include <windows.h>\n#include_next <tbs.h>\n")
        .expect("Unable to write Windows include shim");
}
