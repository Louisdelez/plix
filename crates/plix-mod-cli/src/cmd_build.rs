//! `plix mod build` command - compile mod to WASM

use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;
use tracing::{error, info};

/// Target triple for WASM
const WASM_TARGET: &str = "wasm32-unknown-unknown";

/// Run the `build` command
pub fn run(release: bool, manifest_path: Option<&Path>) -> Result<()> {
    // Check if Rust toolchain is available
    check_rust_toolchain()?;

    // Check if WASM target is installed
    check_wasm_target()?;

    // Determine manifest path
    let manifest = manifest_path.unwrap_or_else(|| Path::new("Cargo.toml"));
    if !manifest.exists() {
        bail!("Cargo.toml not found at {}", manifest.display());
    }

    info!("Building mod for target {}", WASM_TARGET);

    // Build command
    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .arg("--target")
        .arg(WASM_TARGET)
        .arg("--manifest-path")
        .arg(manifest);

    if release {
        cmd.arg("--release");
    }

    // Run cargo build
    let status = cmd
        .status()
        .context("Failed to execute cargo build")?;

    if !status.success() {
        bail!("cargo build failed with exit code: {:?}", status.code());
    }

    // Find the output WASM file
    let target_dir = manifest
        .parent()
        .unwrap_or(Path::new("."))
        .join("target")
        .join(WASM_TARGET)
        .join(if release { "release" } else { "debug" });

    // Try to find the .wasm file
    if let Some(wasm_file) = find_wasm_file(&target_dir) {
        info!("Built: {}", wasm_file.display());
        println!("Built: {}", wasm_file.display());
    } else {
        info!("Build complete. WASM files in: {}", target_dir.display());
    }

    Ok(())
}

/// Check if Rust toolchain is available
fn check_rust_toolchain() -> Result<()> {
    let output = Command::new("rustc")
        .arg("--version")
        .output()
        .context("Failed to run rustc. Is Rust installed?")?;

    if !output.status.success() {
        bail!("rustc not working properly. Please check your Rust installation.");
    }

    Ok(())
}

/// Check if WASM target is installed
fn check_wasm_target() -> Result<()> {
    let output = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .context("Failed to run rustup. Is rustup installed?")?;

    if !output.status.success() {
        bail!("rustup not working properly.");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.lines().any(|line| line.trim() == WASM_TARGET) {
        error!("WASM target not installed");
        println!("\nThe {} target is not installed.", WASM_TARGET);
        println!("Install it with:");
        println!("  rustup target add {}", WASM_TARGET);
        bail!("Missing WASM target. Run: rustup target add {}", WASM_TARGET);
    }

    Ok(())
}

/// Find a .wasm file in the target directory
fn find_wasm_file(target_dir: &Path) -> Option<std::path::PathBuf> {
    if !target_dir.exists() {
        return None;
    }

    std::fs::read_dir(target_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| p.extension().map(|e| e == "wasm").unwrap_or(false))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_target_constant() {
        assert_eq!(WASM_TARGET, "wasm32-unknown-unknown");
    }
}
