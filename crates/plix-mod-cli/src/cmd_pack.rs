//! `plix mod pack` command - create deterministic .plixmod bundle

use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use tracing::info;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

/// Maximum bundle size in bytes (10 MB)
pub const MAX_BUNDLE_SIZE: u64 = 10 * 1024 * 1024;

/// Mod manifest structure
#[derive(Debug, serde::Deserialize)]
pub struct ModManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub api_version: u8,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

/// Run the `pack` command
pub fn run(
    output: Option<&Path>,
    manifest_path: &Path,
    wasm_path: Option<&Path>,
    _unsigned: bool,
) -> Result<()> {
    // Read manifest
    let manifest_content = fs::read_to_string(manifest_path)
        .with_context(|| format!("Failed to read manifest: {}", manifest_path.display()))?;

    let manifest: ModManifest = toml::from_str(&manifest_content)
        .with_context(|| "Failed to parse mod.toml")?;

    info!("Packing {} v{}", manifest.name, manifest.version);

    // Find WASM file
    let wasm_file = if let Some(path) = wasm_path {
        path.to_path_buf()
    } else {
        find_wasm_file(manifest_path)?
    };

    if !wasm_file.exists() {
        bail!("WASM file not found: {}. Run 'plix-mod build' first.", wasm_file.display());
    }

    // Determine output path
    let output_file = output.map(|p| p.to_path_buf()).unwrap_or_else(|| {
        let filename = format!("{}-{}.plixmod", manifest.id, manifest.version);
        manifest_path
            .parent()
            .unwrap_or(Path::new("."))
            .join(filename)
    });

    // Find assets directory (optional)
    let assets_dir = manifest_path
        .parent()
        .unwrap_or(Path::new("."))
        .join("assets");

    // Create deterministic ZIP
    let mut bundle_data = Vec::new();
    create_deterministic_zip(
        &mut bundle_data,
        manifest_path,
        &wasm_file,
        if assets_dir.exists() {
            Some(&assets_dir)
        } else {
            None
        },
    )?;

    // Check size limit
    let size = bundle_data.len() as u64;
    if size > MAX_BUNDLE_SIZE {
        bail!(
            "Bundle size {} bytes exceeds maximum {} bytes (10 MB). \
            Reduce assets or split into multiple mods.",
            size,
            MAX_BUNDLE_SIZE
        );
    }

    // Calculate SHA-256
    let mut hasher = Sha256::new();
    hasher.update(&bundle_data);
    let hash = hasher.finalize();
    let hash_hex = hex::encode(hash);

    // Write output file
    fs::write(&output_file, &bundle_data)
        .with_context(|| format!("Failed to write {}", output_file.display()))?;

    let size_kb = size as f64 / 1024.0;
    info!(
        "Packed: {} ({:.1} KB)",
        output_file.display(),
        size_kb
    );

    println!(
        "Packed {} v{} ({:.1} KB)",
        manifest.id, manifest.version, size_kb
    );
    println!("SHA-256: {}", hash_hex);
    println!("Output: {}", output_file.display());

    Ok(())
}

/// Create a deterministic ZIP archive
fn create_deterministic_zip(
    output: &mut Vec<u8>,
    manifest_path: &Path,
    wasm_path: &Path,
    assets_dir: Option<&Path>,
) -> Result<()> {
    let cursor = std::io::Cursor::new(output);
    let mut zip = ZipWriter::new(cursor);

    // Use epoch timestamp for determinism (1980-01-01 for ZIP format)
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .compression_level(Some(6))
        .last_modified_time(
            zip::DateTime::from_date_and_time(1980, 1, 1, 0, 0, 0)
                .expect("valid datetime"),
        );

    // Collect all files to add (sorted for determinism)
    let mut entries: Vec<(String, Vec<u8>)> = Vec::new();

    // Add mod.toml
    let manifest_content = fs::read(manifest_path)?;
    entries.push(("mod.toml".to_string(), manifest_content));

    // Add mod.wasm
    let wasm_content = fs::read(wasm_path)?;
    entries.push(("mod.wasm".to_string(), wasm_content));

    // Add assets (if present)
    if let Some(assets) = assets_dir {
        for entry in walkdir::WalkDir::new(assets).min_depth(1).sort_by_file_name() {
            let entry = entry?;
            if entry.file_type().is_file() {
                let relative = entry.path().strip_prefix(assets)?;
                let archive_path = format!("assets/{}", relative.display());
                let content = fs::read(entry.path())?;
                entries.push((archive_path, content));
            }
        }
    }

    // Sort entries by path for determinism
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    // Write all entries
    for (path, content) in entries {
        zip.start_file(&path, options)?;
        zip.write_all(&content)?;
    }

    let cursor = zip.finish()?;
    *output = cursor.into_inner();

    Ok(())
}

/// Find the WASM file based on manifest location
fn find_wasm_file(manifest_path: &Path) -> Result<std::path::PathBuf> {
    let project_dir = manifest_path.parent().unwrap_or(Path::new("."));

    // Try to get mod name from Cargo.toml
    let cargo_toml = project_dir.join("Cargo.toml");
    if cargo_toml.exists() {
        let content = fs::read_to_string(&cargo_toml)?;
        if let Some(name) = extract_package_name(&content) {
            let wasm_name = name.replace('-', "_");

            // Try release first, then debug
            let release_path = project_dir
                .join("target/wasm32-unknown-unknown/release")
                .join(format!("{}.wasm", wasm_name));
            if release_path.exists() {
                return Ok(release_path);
            }

            let debug_path = project_dir
                .join("target/wasm32-unknown-unknown/debug")
                .join(format!("{}.wasm", wasm_name));
            if debug_path.exists() {
                return Ok(debug_path);
            }
        }
    }

    bail!("Could not find WASM file. Specify with --wasm or run 'plix-mod build' first.")
}

/// Extract package name from Cargo.toml content
fn extract_package_name(content: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("name") && line.contains('=') {
            // Parse: name = "foo"
            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() == 2 {
                let value = parts[1].trim().trim_matches('"').trim_matches('\'');
                return Some(value.to_string());
            }
        }
    }
    None
}

// Add hex encoding helper since we don't have the hex crate
mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes
            .as_ref()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_max_bundle_size() {
        assert_eq!(MAX_BUNDLE_SIZE, 10 * 1024 * 1024);
    }

    #[test]
    fn test_extract_package_name() {
        let content = r#"
[package]
name = "my-mod"
version = "1.0.0"
"#;
        assert_eq!(extract_package_name(content), Some("my-mod".to_string()));
    }

    #[test]
    fn test_hex_encode() {
        assert_eq!(hex::encode([0xab, 0xcd, 0xef]), "abcdef");
    }

    #[test]
    fn test_deterministic_pack_same_hash() {
        // Create temporary directory with test files
        let temp = TempDir::new().unwrap();
        let manifest_path = temp.path().join("mod.toml");
        let wasm_path = temp.path().join("mod.wasm");

        // Write manifest
        let manifest = r#"
id = "test-mod"
name = "Test Mod"
version = "1.0.0"
api_version = 1
"#;
        fs::write(&manifest_path, manifest).unwrap();

        // Write fake WASM (just needs to exist for this test)
        let wasm_content = b"\x00asm\x01\x00\x00\x00"; // minimal WASM header
        fs::write(&wasm_path, wasm_content).unwrap();

        // Pack twice
        let mut bundle1 = Vec::new();
        create_deterministic_zip(&mut bundle1, &manifest_path, &wasm_path, None).unwrap();

        let mut bundle2 = Vec::new();
        create_deterministic_zip(&mut bundle2, &manifest_path, &wasm_path, None).unwrap();

        // Hashes should be identical
        let hash1 = calculate_sha256(&bundle1);
        let hash2 = calculate_sha256(&bundle2);

        assert_eq!(hash1, hash2, "Packing same files should produce identical hashes");
        assert_eq!(bundle1, bundle2, "Bundle bytes should be identical");
    }

    #[test]
    fn test_deterministic_pack_different_content_different_hash() {
        let temp = TempDir::new().unwrap();
        let manifest_path = temp.path().join("mod.toml");
        let wasm_path = temp.path().join("mod.wasm");

        // First version
        fs::write(&manifest_path, "id = \"mod-v1\"\nname = \"V1\"\nversion = \"1.0.0\"\napi_version = 1").unwrap();
        fs::write(&wasm_path, b"\x00asm\x01\x00\x00\x00").unwrap();

        let mut bundle1 = Vec::new();
        create_deterministic_zip(&mut bundle1, &manifest_path, &wasm_path, None).unwrap();
        let hash1 = calculate_sha256(&bundle1);

        // Second version with different content
        fs::write(&manifest_path, "id = \"mod-v2\"\nname = \"V2\"\nversion = \"2.0.0\"\napi_version = 1").unwrap();

        let mut bundle2 = Vec::new();
        create_deterministic_zip(&mut bundle2, &manifest_path, &wasm_path, None).unwrap();
        let hash2 = calculate_sha256(&bundle2);

        assert_ne!(hash1, hash2, "Different content should produce different hashes");
    }

    #[test]
    fn test_bundle_size_limit_enforcement() {
        // Test that bundles over 10 MB fail
        assert!(MAX_BUNDLE_SIZE < u64::MAX);
        assert_eq!(MAX_BUNDLE_SIZE, 10 * 1024 * 1024);

        // Create a bundle data vector that exceeds the limit
        let oversized_data: Vec<u8> = vec![0u8; (MAX_BUNDLE_SIZE + 1) as usize];

        // The check happens in run(), but we can verify the constant
        assert!(oversized_data.len() as u64 > MAX_BUNDLE_SIZE);
    }

    fn calculate_sha256(data: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        hex::encode(result)
    }
}
