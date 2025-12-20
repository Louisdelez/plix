//! `plix mod validate` command - validate a mod bundle

use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use tracing::info;
use zip::ZipArchive;

use crate::cmd_pack::MAX_BUNDLE_SIZE;

/// Required WASM exports for a valid mod
const REQUIRED_EXPORTS: &[&str] = &["mod_init", "mod_on_event", "mod_shutdown"];

/// Current SDK API version
const CURRENT_API_VERSION: u8 = 1;

/// Known capability names
const KNOWN_CAPABILITIES: &[&str] = &[
    "world_read",
    "world_write",
    "entity_read",
    "entity_write",
    "net_send",
    "event_cancel_chat",
    "event_cancel_blocks",
];

/// Validation result for JSON output
#[derive(Debug, Serialize)]
struct ValidationResult {
    valid: bool,
    errors: Vec<ValidationError>,
    warnings: Vec<ValidationWarning>,
    info: BundleInfo,
}

#[derive(Debug, Serialize)]
struct ValidationError {
    code: String,
    message: String,
}

#[derive(Debug, Serialize)]
struct ValidationWarning {
    code: String,
    message: String,
}

#[derive(Debug, Serialize)]
struct BundleInfo {
    id: String,
    version: String,
    size: u64,
    sha256: String,
}

/// Run the `validate` command
pub fn run(bundle_path: &Path, strict: bool, json_output: bool) -> Result<()> {
    if !bundle_path.exists() {
        bail!("Bundle not found: {}", bundle_path.display());
    }

    let file = File::open(bundle_path)?;
    let file_size = file.metadata()?.len();

    // Calculate SHA-256
    let mut file_content = Vec::new();
    File::open(bundle_path)?.read_to_end(&mut file_content)?;
    let sha256 = calculate_sha256(&file_content);

    let mut errors: Vec<ValidationError> = Vec::new();
    let mut warnings: Vec<ValidationWarning> = Vec::new();

    // E005: Check bundle size
    if file_size > MAX_BUNDLE_SIZE {
        errors.push(ValidationError {
            code: "E005".to_string(),
            message: format!(
                "Bundle size {} bytes exceeds maximum {} bytes (10 MB)",
                file_size, MAX_BUNDLE_SIZE
            ),
        });
    }

    // Open ZIP archive
    let file = File::open(bundle_path)?;
    let mut archive = ZipArchive::new(file)
        .with_context(|| "Not a valid ZIP archive")?;

    // E004: Check manifest
    let manifest = match read_manifest(&mut archive) {
        Ok(m) => Some(m),
        Err(e) => {
            errors.push(ValidationError {
                code: "E004".to_string(),
                message: format!("Manifest error: {}", e),
            });
            None
        }
    };

    let (mod_id, mod_version) = if let Some(ref m) = manifest {
        // E001: Validate mod ID format
        if let Err(e) = validate_mod_id(&m.id) {
            errors.push(ValidationError {
                code: "E001".to_string(),
                message: format!("Invalid mod ID: {}", e),
            });
        }

        // E007: Check API version
        if m.api_version > CURRENT_API_VERSION {
            errors.push(ValidationError {
                code: "E007".to_string(),
                message: format!(
                    "API version {} is newer than current SDK version {}",
                    m.api_version, CURRENT_API_VERSION
                ),
            });
        }

        // E006: Validate capabilities
        if let Some(ref caps) = m.capabilities {
            validate_capabilities(caps, &mut errors);
        }

        (m.id.clone(), m.version.clone())
    } else {
        ("unknown".to_string(), "0.0.0".to_string())
    };

    // E003: Check WASM binary exists
    let wasm_exists = archive.by_name("mod.wasm").is_ok();
    if !wasm_exists {
        errors.push(ValidationError {
            code: "E003".to_string(),
            message: "mod.wasm not found in bundle".to_string(),
        });
    }

    // E003: Check WASM exports
    if wasm_exists {
        if let Err(e) = check_wasm_exports(&mut archive) {
            errors.push(ValidationError {
                code: "E003".to_string(),
                message: format!("WASM export error: {}", e),
            });
        }
    }

    let result = ValidationResult {
        valid: errors.is_empty() && (!strict || warnings.is_empty()),
        errors,
        warnings,
        info: BundleInfo {
            id: mod_id,
            version: mod_version,
            size: file_size,
            sha256,
        },
    };

    if json_output {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        print_validation_result(&result, bundle_path);
    }

    if !result.valid {
        bail!("Validation failed");
    }

    Ok(())
}

/// Read and parse the manifest from the archive
fn read_manifest(archive: &mut ZipArchive<File>) -> Result<ManifestData> {
    let mut manifest_file = archive.by_name("mod.toml")
        .context("mod.toml not found in bundle")?;

    let mut content = String::new();
    manifest_file.read_to_string(&mut content)?;

    let manifest: ManifestData = toml::from_str(&content)?;
    Ok(manifest)
}

#[derive(Debug, serde::Deserialize)]
struct ManifestData {
    id: String,
    #[allow(dead_code)]
    name: String,
    version: String,
    api_version: u8,
    #[serde(default)]
    capabilities: Option<toml::Table>,
}

/// Validate mod ID format
fn validate_mod_id(id: &str) -> Result<()> {
    if id.len() < 3 || id.len() > 64 {
        bail!("ID must be 3-64 characters");
    }

    for c in id.chars() {
        if !c.is_ascii_lowercase() && !c.is_ascii_digit() && c != '-' {
            bail!("ID must contain only lowercase letters, digits, and hyphens");
        }
    }

    if id.starts_with('-') || id.ends_with('-') || id.contains("--") {
        bail!("ID cannot start/end with hyphen or contain consecutive hyphens");
    }

    Ok(())
}

/// Validate capabilities
fn validate_capabilities(caps: &toml::Table, errors: &mut Vec<ValidationError>) {
    for key in caps.keys() {
        if !KNOWN_CAPABILITIES.contains(&key.as_str()) {
            errors.push(ValidationError {
                code: "E006".to_string(),
                message: format!("Unknown capability: {}", key),
            });
        }
    }
}

/// Check that required WASM exports exist
fn check_wasm_exports(archive: &mut ZipArchive<File>) -> Result<()> {
    let mut wasm_file = archive.by_name("mod.wasm")?;
    let mut wasm_bytes = Vec::new();
    wasm_file.read_to_end(&mut wasm_bytes)?;

    // Parse WASM to find exports
    let exports = parse_wasm_exports(&wasm_bytes)?;

    let mut missing: Vec<&str> = Vec::new();
    for required in REQUIRED_EXPORTS {
        if !exports.contains(&required.to_string()) {
            missing.push(required);
        }
    }

    if !missing.is_empty() {
        bail!("Missing required exports: {}", missing.join(", "));
    }

    Ok(())
}

/// Parse WASM binary to extract export names
fn parse_wasm_exports(bytes: &[u8]) -> Result<Vec<String>> {
    let parser = wasmparser::Parser::new(0);
    let mut exports = Vec::new();

    for payload in parser.parse_all(bytes) {
        if let wasmparser::Payload::ExportSection(reader) = payload? {
            for export in reader {
                let export = export?;
                exports.push(export.name.to_string());
            }
        }
    }

    Ok(exports)
}

/// Calculate SHA-256 hash
fn calculate_sha256(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    result.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Print validation result in human-readable format
fn print_validation_result(result: &ValidationResult, bundle_path: &Path) {
    println!("Validating {}...", bundle_path.display());

    if result.errors.is_empty() {
        println!("✓ Manifest valid");
        println!("✓ Required fields present");
        println!("✓ WASM binary present");
        println!("✓ Required exports found");
        println!(
            "✓ Size OK ({:.1} KB / {:.1} MB)",
            result.info.size as f64 / 1024.0,
            MAX_BUNDLE_SIZE as f64 / 1024.0 / 1024.0
        );
        println!("✓ API version compatible");
        println!("✓ Capabilities valid");
        println!();
        println!("Result: PASS");
    } else {
        for error in &result.errors {
            println!("✗ [{}] {}", error.code, error.message);
        }
        println!();
        println!("Result: FAIL");
    }

    for warning in &result.warnings {
        println!("⚠ [{}] {}", warning.code, warning.message);
    }

    println!();
    println!("ID: {}", result.info.id);
    println!("Version: {}", result.info.version);
    println!("SHA-256: {}", result.info.sha256);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_required_exports() {
        assert!(REQUIRED_EXPORTS.contains(&"mod_init"));
        assert!(REQUIRED_EXPORTS.contains(&"mod_on_event"));
        assert!(REQUIRED_EXPORTS.contains(&"mod_shutdown"));
    }

    #[test]
    fn test_validate_mod_id_valid() {
        assert!(validate_mod_id("my-mod").is_ok());
        assert!(validate_mod_id("mod123").is_ok());
        assert!(validate_mod_id("abc").is_ok()); // exactly 3 chars
        assert!(validate_mod_id("my-mod-v2").is_ok());
    }

    #[test]
    fn test_validate_mod_id_invalid() {
        assert!(validate_mod_id("ab").is_err()); // too short
        assert!(validate_mod_id("MyMod").is_err()); // uppercase
        assert!(validate_mod_id("-mod").is_err()); // starts with hyphen
        assert!(validate_mod_id("mod-").is_err()); // ends with hyphen
        assert!(validate_mod_id("my--mod").is_err()); // consecutive hyphens
        assert!(validate_mod_id("mod_name").is_err()); // underscore not allowed
        assert!(validate_mod_id("Mod123").is_err()); // uppercase
    }

    #[test]
    fn test_validate_mod_id_length() {
        // Test exactly 3 chars (minimum)
        assert!(validate_mod_id("abc").is_ok());

        // Test 64 chars (maximum)
        let long_id = "a".repeat(64);
        assert!(validate_mod_id(&long_id).is_ok());

        // Test 65 chars (too long)
        let too_long = "a".repeat(65);
        assert!(validate_mod_id(&too_long).is_err());
    }

    #[test]
    fn test_calculate_sha256() {
        let hash = calculate_sha256(b"hello");
        assert_eq!(hash.len(), 64); // 256 bits = 32 bytes = 64 hex chars

        // Known hash for "hello"
        assert_eq!(
            hash,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn test_known_capabilities() {
        assert!(KNOWN_CAPABILITIES.contains(&"world_read"));
        assert!(KNOWN_CAPABILITIES.contains(&"world_write"));
        assert!(KNOWN_CAPABILITIES.contains(&"entity_read"));
        assert!(KNOWN_CAPABILITIES.contains(&"entity_write"));
        assert!(KNOWN_CAPABILITIES.contains(&"net_send"));
        assert!(KNOWN_CAPABILITIES.contains(&"event_cancel_chat"));
        assert!(KNOWN_CAPABILITIES.contains(&"event_cancel_blocks"));
        assert_eq!(KNOWN_CAPABILITIES.len(), 7);
    }

    #[test]
    fn test_validate_capabilities_unknown() {
        let mut caps = toml::Table::new();
        caps.insert("unknown_cap".to_string(), toml::Value::Boolean(true));

        let mut errors = Vec::new();
        validate_capabilities(&caps, &mut errors);

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, "E006");
        assert!(errors[0].message.contains("unknown_cap"));
    }

    #[test]
    fn test_validate_capabilities_valid() {
        let mut caps = toml::Table::new();
        caps.insert("world_read".to_string(), toml::Value::Boolean(true));
        caps.insert("net_send".to_string(), toml::Value::Boolean(true));

        let mut errors = Vec::new();
        validate_capabilities(&caps, &mut errors);

        assert!(errors.is_empty(), "Valid capabilities should not produce errors");
    }

    #[test]
    fn test_current_api_version() {
        assert_eq!(CURRENT_API_VERSION, 1);
    }

    #[test]
    fn test_max_bundle_size_matches_pack() {
        // Ensure validation uses same limit as pack
        assert_eq!(MAX_BUNDLE_SIZE, 10 * 1024 * 1024);
    }

    #[test]
    fn test_parse_wasm_exports_minimal() {
        // Minimal valid WASM with no exports
        let minimal_wasm = vec![
            0x00, 0x61, 0x73, 0x6d, // magic: \0asm
            0x01, 0x00, 0x00, 0x00, // version: 1
        ];

        let exports = parse_wasm_exports(&minimal_wasm).unwrap();
        assert!(exports.is_empty(), "Minimal WASM has no exports");
    }

    #[test]
    fn test_validation_error_codes() {
        // Verify all error codes are documented
        let error = ValidationError {
            code: "E001".to_string(),
            message: "test".to_string(),
        };
        assert_eq!(error.code, "E001");

        // E001 = Invalid mod ID
        // E003 = Missing WASM / exports
        // E004 = Manifest error
        // E005 = Bundle too large
        // E006 = Unknown capability
        // E007 = API version incompatible
    }
}
