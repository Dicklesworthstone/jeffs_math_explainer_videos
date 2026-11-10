use std::env;
use std::process::Command;

const PROFILE_ENV: &str = "FMN_CONFORMANCE_CARGO_PROFILE";
const RUSTC_IDENTITY_ENV: &str = "FMN_CONFORMANCE_RUSTC_IDENTITY";

// One implementation for every build script that embeds a profile identity;
// unit-tested in fmn-cli.
include!("../fmn-cli/src/cargo_profile.rs");

fn main() -> Result<(), String> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=RUSTC");
    let out_dir = env::var_os("OUT_DIR")
        .map(std::path::PathBuf::from)
        .ok_or_else(|| "Cargo did not provide OUT_DIR".to_owned())?;
    let base_profile =
        env::var("PROFILE").map_err(|_| "Cargo did not provide PROFILE".to_owned())?;
    let profile = cargo_profile_from_out_dir(&out_dir, &base_profile)?;
    println!("cargo:rustc-env={PROFILE_ENV}={profile}");

    // Benchmark evidence must identify the compiler that actually built the
    // artifact, not merely trust the pin copied into a caller-authored
    // baseline. Cargo provides the exact compiler path it is using. Asking
    // that compiler for its verbose identity is a build-time operation (the
    // compiler is already the active build tool), never a runtime subprocess
    // added to the standalone product's one-external-tool surface.
    let rustc = env::var_os("RUSTC").ok_or_else(|| "Cargo did not provide RUSTC".to_owned())?;
    let output = Command::new(&rustc)
        .args(["--version", "--verbose"])
        .output()
        .map_err(|error| format!("cannot inspect Cargo's rustc {rustc:?}: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "Cargo's rustc {rustc:?} refused --version --verbose with {}",
            output.status
        ));
    }
    let identity = String::from_utf8(output.stdout)
        .map_err(|_| "rustc --version --verbose returned non-UTF-8 output".to_owned())?;
    let identity = identity.trim_end_matches(['\r', '\n']).replace('\n', "|");
    if identity.is_empty() || identity.contains('\r') {
        return Err("rustc --version --verbose returned an invalid identity".to_owned());
    }
    println!("cargo:rustc-env={RUSTC_IDENTITY_ENV}={identity}");
    Ok(())
}
