use std::env;
use std::path::{Path, PathBuf};

include!("src/cargo_profile.rs");
// ADR-0025: one identity implementation for every build script.
include!("src/build_identity.rs");

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=FMN_BUILD_ID");
    let manifest_dir =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("Cargo manifest dir"));
    let repo = manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("fmn-cli remains under crates/");
    let explicit = env::var("FMN_BUILD_ID").ok();
    let build_id = build_identity(
        repo,
        explicit.as_deref(),
        &env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "fmn-cli".to_owned()),
        &env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "unknown".to_owned()),
        true,
    );
    println!("cargo:rustc-env=FMN_BUILD_ID={build_id}");
    println!(
        "cargo:rustc-env=FMN_TARGET_TRIPLE={}",
        env::var("TARGET").expect("Cargo target triple")
    );
    let out_dir = env::var_os("OUT_DIR")
        .map(PathBuf::from)
        .expect("Cargo OUT_DIR");
    let base_profile = env::var("PROFILE").expect("Cargo base PROFILE");
    println!(
        "cargo:rustc-env=FMN_CARGO_PROFILE={}",
        cargo_profile_from_out_dir(&out_dir, &base_profile).expect("Cargo profile from OUT_DIR")
    );
}
