//! Shared build-script helpers for compiling brand icon resources into
//! Windows binaries. Used as a build-dependency by every saver crate so
//! the RC-generation logic lives in exactly one place.
//!
//! Vendored from `runner::core::build_resources`.

use std::path::Path;

pub mod resource;
pub use resource::*;

const COMPANY_NAME: &str = "idlescreen";
const LEGAL_COPYRIGHT: &str = "Copyright (c) 2026 IdleScreen";

/// Embed the saver's ICO + VERSIONINFO into the Windows binary.
/// No-op on non-Windows targets or when the icon is absent.
pub fn embed_brand_icon(icon_file_path: &str, product_name: &str) {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    if !Path::new(icon_file_path).exists() {
        return;
    }

    let package_name = std::env::var("CARGO_PKG_NAME").unwrap_or_default();
    let resource_script_path = write_brand_rc(
        "build/windows_resource.rc",
        icon_file_path,
        &package_name,
        product_name,
        COMPANY_NAME,
        LEGAL_COPYRIGHT,
    );
    compile_rc(&resource_script_path);
}

/// Compile the generated `.rc` and link the result — the two things
/// `embed-resource` did, invoked directly. MSVC targets use `rc.exe`;
/// everything else uses `windres` (target-prefixed when cross-compiling).
fn compile_rc(rc_path: &str) {
    use std::process::Command;

    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    let out_dir = std::env::var("OUT_DIR").unwrap_or_else(|_| ".".into());

    if target_env == "msvc" {
        let res = format!("{out_dir}/windows_resource.res");
        let status = Command::new("rc")
            .args(["/fo", &res, rc_path])
            .status()
            .expect("failed to run rc.exe — install the Windows SDK");
        assert!(status.success(), "rc.exe failed on {rc_path}");
        println!("cargo:rustc-link-arg={res}");
    } else {
        let obj = format!("{out_dir}/windows_resource.o");
        let target = std::env::var("TARGET").unwrap_or_default();
        let windres = format!("{target}-windres");
        let status = Command::new(&windres)
            .args(["-i", rc_path, "-o", &obj, "--output-format=coff"])
            .status()
            .or_else(|_| {
                Command::new("windres")
                    .args(["-i", rc_path, "-o", &obj, "--output-format=coff"])
                    .status()
            })
            .expect("failed to run windres — install binutils-mingw-w64");
        assert!(status.success(), "windres failed on {rc_path}");
        println!("cargo:rustc-link-arg={obj}");
    }
    println!("cargo:rustc-link-search=native={out_dir}");
}
