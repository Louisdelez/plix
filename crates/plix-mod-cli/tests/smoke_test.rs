//! Smoke integration test for mod toolchain
//!
//! Tests the basic bundle structure and validation without requiring
//! actual WASM compilation.

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use tempfile::TempDir;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

/// Create a minimal valid mod bundle for testing
fn create_test_bundle(dir: &Path, mod_id: &str, version: &str) -> std::path::PathBuf {
    let bundle_path = dir.join(format!("{}-{}.plixmod", mod_id, version));

    let file = File::create(&bundle_path).unwrap();
    let mut zip = ZipWriter::new(file);

    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .last_modified_time(
            zip::DateTime::from_date_and_time(1980, 1, 1, 0, 0, 0).expect("valid datetime"),
        );

    // Add mod.toml
    let manifest = format!(
        r#"id = "{mod_id}"
name = "Test Mod"
version = "{version}"
api_version = 1
"#
    );
    zip.start_file("mod.toml", options).unwrap();
    zip.write_all(manifest.as_bytes()).unwrap();

    // Add minimal WASM with required exports
    // This is a valid WASM with export section containing mod_init, mod_on_event, mod_shutdown
    let wasm_with_exports = create_minimal_wasm_with_exports();
    zip.start_file("mod.wasm", options).unwrap();
    zip.write_all(&wasm_with_exports).unwrap();

    zip.finish().unwrap();
    bundle_path
}

/// Create a WASM binary with required exports (mod_init, mod_on_event, mod_shutdown)
fn create_minimal_wasm_with_exports() -> Vec<u8> {
    // Minimal valid WASM module with three exported functions
    // This is a hand-crafted binary that exports the required functions
    let mut wasm = Vec::new();

    // Magic number and version
    wasm.extend_from_slice(b"\x00asm"); // Magic
    wasm.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]); // Version 1

    // Type section (section 1) - define function signatures
    // Type 0: () -> i32
    // Type 1: (i32, i32, i32) -> i32
    let type_section = [
        0x01, // Section ID: Type
        0x0c, // Section size: 12 bytes
        0x02, // Num types: 2
        // Type 0: () -> i32
        0x60, // func type
        0x00, // 0 params
        0x01, 0x7f, // 1 result: i32
        // Type 1: (i32, i32, i32) -> i32
        0x60, // func type
        0x03, 0x7f, 0x7f, 0x7f, // 3 params: i32, i32, i32
        0x01, 0x7f, // 1 result: i32
    ];
    wasm.extend_from_slice(&type_section);

    // Function section (section 3) - declare functions
    let func_section = [
        0x03, // Section ID: Function
        0x04, // Section size: 4 bytes
        0x03, // Num functions: 3
        0x00, // func 0 uses type 0 (mod_init)
        0x01, // func 1 uses type 1 (mod_on_event)
        0x00, // func 2 uses type 0 (mod_shutdown)
    ];
    wasm.extend_from_slice(&func_section);

    // Export section (section 7) - export functions
    let export_section = [
        0x07, // Section ID: Export
        0x2d, // Section size: 45 bytes
        0x03, // Num exports: 3
        // Export 0: "mod_init"
        0x08, // name length
        b'm', b'o', b'd', b'_', b'i', b'n', b'i', b't',
        0x00, // kind: function
        0x00, // func index: 0
        // Export 1: "mod_on_event"
        0x0c, // name length
        b'm', b'o', b'd', b'_', b'o', b'n', b'_', b'e', b'v', b'e', b'n', b't',
        0x00, // kind: function
        0x01, // func index: 1
        // Export 2: "mod_shutdown"
        0x0c, // name length
        b'm', b'o', b'd', b'_', b's', b'h', b'u', b't', b'd', b'o', b'w', b'n',
        0x00, // kind: function
        0x02, // func index: 2
    ];
    wasm.extend_from_slice(&export_section);

    // Code section (section 10) - function bodies
    let code_section = [
        0x0a, // Section ID: Code
        0x0d, // Section size: 13 bytes
        0x03, // Num functions: 3
        // Function 0 body (mod_init): return 0
        0x04, // body size: 4 bytes
        0x00, // local decl count: 0
        0x41, 0x00, // i32.const 0
        0x0b, // end
        // Function 1 body (mod_on_event): return 0
        0x04, // body size: 4 bytes
        0x00, // local decl count: 0
        0x41, 0x00, // i32.const 0
        0x0b, // end
        // Function 2 body (mod_shutdown): return 0
        0x04, // body size: 4 bytes
        0x00, // local decl count: 0
        0x41, 0x00, // i32.const 0
        0x0b, // end
    ];
    wasm.extend_from_slice(&code_section);

    wasm
}

/// Create a bundle with missing exports for negative testing
fn create_bundle_without_exports(dir: &Path) -> std::path::PathBuf {
    let bundle_path = dir.join("bad-exports.plixmod");

    let file = File::create(&bundle_path).unwrap();
    let mut zip = ZipWriter::new(file);

    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    // Add mod.toml
    let manifest = r#"id = "bad-exports"
name = "Bad Mod"
version = "1.0.0"
api_version = 1
"#;
    zip.start_file("mod.toml", options).unwrap();
    zip.write_all(manifest.as_bytes()).unwrap();

    // Add minimal WASM without exports
    let minimal_wasm = [
        0x00, 0x61, 0x73, 0x6d, // magic
        0x01, 0x00, 0x00, 0x00, // version
    ];
    zip.start_file("mod.wasm", options).unwrap();
    zip.write_all(&minimal_wasm).unwrap();

    zip.finish().unwrap();
    bundle_path
}

#[test]
fn test_bundle_structure_valid() {
    let temp = TempDir::new().unwrap();
    let bundle_path = create_test_bundle(temp.path(), "test-mod", "1.0.0");

    // Verify bundle exists
    assert!(bundle_path.exists());

    // Verify it's a valid ZIP
    let file = File::open(&bundle_path).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();

    // Check required files exist
    assert!(archive.by_name("mod.toml").is_ok());
    assert!(archive.by_name("mod.wasm").is_ok());
}

#[test]
fn test_bundle_manifest_parseable() {
    let temp = TempDir::new().unwrap();
    let bundle_path = create_test_bundle(temp.path(), "my-test-mod", "2.0.0");

    let file = File::open(&bundle_path).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();

    let mut manifest_file = archive.by_name("mod.toml").unwrap();
    let mut content = String::new();
    manifest_file.read_to_string(&mut content).unwrap();

    // Parse as TOML
    let manifest: toml::Value = toml::from_str(&content).unwrap();

    assert_eq!(manifest["id"].as_str(), Some("my-test-mod"));
    assert_eq!(manifest["version"].as_str(), Some("2.0.0"));
    assert_eq!(manifest["api_version"].as_integer(), Some(1));
}

#[test]
fn test_bundle_wasm_has_exports() {
    let temp = TempDir::new().unwrap();
    let bundle_path = create_test_bundle(temp.path(), "export-test", "1.0.0");

    let file = File::open(&bundle_path).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();

    let mut wasm_file = archive.by_name("mod.wasm").unwrap();
    let mut wasm_bytes = Vec::new();
    wasm_file.read_to_end(&mut wasm_bytes).unwrap();

    // Parse WASM to find exports
    let parser = wasmparser::Parser::new(0);
    let mut exports = Vec::new();

    for payload in parser.parse_all(&wasm_bytes) {
        if let wasmparser::Payload::ExportSection(reader) = payload.unwrap() {
            for export in reader {
                exports.push(export.unwrap().name.to_string());
            }
        }
    }

    // Verify required exports
    assert!(exports.contains(&"mod_init".to_string()));
    assert!(exports.contains(&"mod_on_event".to_string()));
    assert!(exports.contains(&"mod_shutdown".to_string()));
}

#[test]
fn test_bundle_missing_exports_detectable() {
    let temp = TempDir::new().unwrap();
    let bundle_path = create_bundle_without_exports(temp.path());

    let file = File::open(&bundle_path).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();

    let mut wasm_file = archive.by_name("mod.wasm").unwrap();
    let mut wasm_bytes = Vec::new();
    wasm_file.read_to_end(&mut wasm_bytes).unwrap();

    // Parse WASM to find exports
    let parser = wasmparser::Parser::new(0);
    let mut exports = Vec::new();

    for payload in parser.parse_all(&wasm_bytes) {
        if let wasmparser::Payload::ExportSection(reader) = payload.unwrap() {
            for export in reader {
                exports.push(export.unwrap().name.to_string());
            }
        }
    }

    // Should have no exports
    assert!(exports.is_empty(), "Expected no exports in minimal WASM");
    assert!(!exports.contains(&"mod_init".to_string()));
}

#[test]
fn test_bundle_deterministic_entries() {
    let temp = TempDir::new().unwrap();
    let bundle_path = create_test_bundle(temp.path(), "det-test", "1.0.0");

    let file = File::open(&bundle_path).unwrap();
    let archive = zip::ZipArchive::new(file).unwrap();

    // Verify entries are sorted (mod.toml before mod.wasm alphabetically)
    let names: Vec<_> = (0..archive.len())
        .map(|i| {
            let file = archive.index_raw(i).unwrap();
            file.name().to_string()
        })
        .collect();

    // Should be sorted
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted, "Bundle entries should be sorted");
}

#[test]
fn test_bundle_size_reasonable() {
    let temp = TempDir::new().unwrap();
    let bundle_path = create_test_bundle(temp.path(), "size-test", "1.0.0");

    let metadata = fs::metadata(&bundle_path).unwrap();
    let size = metadata.len();

    // A minimal bundle should be well under 1 KB
    assert!(size < 1024, "Minimal bundle should be small");

    // Should be larger than just the header
    assert!(size > 100, "Bundle should contain actual data");
}
