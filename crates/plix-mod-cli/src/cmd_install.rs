//! `plix mod install` command - install a mod bundle to local cache

use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use tracing::info;
use zip::ZipArchive;

/// Run the `install` command
pub fn run(bundle_path: &Path, cache_dir: Option<&Path>, force: bool) -> Result<()> {
    if !bundle_path.exists() {
        bail!("Bundle not found: {}", bundle_path.display());
    }

    // Determine cache directory
    let cache = cache_dir
        .map(|p| p.to_path_buf())
        .unwrap_or_else(default_cache_dir);

    info!("Installing to cache: {}", cache.display());

    // Read and validate bundle
    let mut file = File::open(bundle_path)?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;

    // Calculate SHA-256
    let mut hasher = Sha256::new();
    hasher.update(&content);
    let hash = hasher.finalize();
    let hash_hex: String = hash.iter().map(|b| format!("{:02x}", b)).collect();

    // Extract manifest to get mod ID and version
    let file = File::open(bundle_path)?;
    let mut archive = ZipArchive::new(file)?;
    let manifest = read_manifest(&mut archive)?;

    let install_dir = cache
        .join(&manifest.id)
        .join(&manifest.version);

    // Check if already installed
    if install_dir.exists() && !force {
        bail!(
            "Mod {} v{} is already installed. Use --force to overwrite.",
            manifest.id,
            manifest.version
        );
    }

    // Create installation directory
    fs::create_dir_all(&install_dir)?;

    // Extract bundle contents
    extract_bundle(bundle_path, &install_dir)?;

    // Write hash file
    let hash_file = install_dir.join(".sha256");
    fs::write(&hash_file, &hash_hex)?;

    info!(
        "Installed {} v{} to {}",
        manifest.id,
        manifest.version,
        install_dir.display()
    );

    println!(
        "Installed {} v{} to {}",
        manifest.id,
        manifest.version,
        install_dir.display()
    );
    println!("SHA-256: {}", hash_hex);

    Ok(())
}

/// Get the default cache directory
fn default_cache_dir() -> PathBuf {
    // Try XDG_DATA_HOME first (Linux standard)
    if let Ok(xdg_data) = std::env::var("XDG_DATA_HOME") {
        return PathBuf::from(xdg_data).join("plix/mods");
    }

    // Platform-specific defaults
    if let Some(data_dir) = dirs_next::data_dir() {
        #[cfg(target_os = "linux")]
        return data_dir.join("plix/mods");

        #[cfg(target_os = "macos")]
        return data_dir.join("plix/mods");

        #[cfg(target_os = "windows")]
        return data_dir.join("plix\\mods");

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        return data_dir.join("plix/mods");
    }

    // Fallback to home directory
    if let Some(home) = dirs_next::home_dir() {
        return home.join(".local/share/plix/mods");
    }

    // Last resort
    PathBuf::from("./mods_cache")
}

/// Read manifest from archive
fn read_manifest(archive: &mut ZipArchive<File>) -> Result<ManifestInfo> {
    let mut manifest_file = archive.by_name("mod.toml")
        .context("mod.toml not found in bundle")?;

    let mut content = String::new();
    manifest_file.read_to_string(&mut content)?;

    let manifest: ManifestInfo = toml::from_str(&content)?;
    Ok(manifest)
}

#[derive(Debug, serde::Deserialize)]
struct ManifestInfo {
    id: String,
    version: String,
}

/// Extract bundle contents to directory
fn extract_bundle(bundle_path: &Path, target_dir: &Path) -> Result<()> {
    let file = File::open(bundle_path)?;
    let mut archive = ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();

        let target_path = target_dir.join(&name);

        if file.is_dir() {
            fs::create_dir_all(&target_path)?;
        } else {
            // Create parent directories
            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent)?;
            }

            let mut content = Vec::new();
            file.read_to_end(&mut content)?;
            fs::write(&target_path, content)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_cache_dir() {
        let cache = default_cache_dir();
        // Should return some path, not panic
        assert!(!cache.to_string_lossy().is_empty());
    }
}
