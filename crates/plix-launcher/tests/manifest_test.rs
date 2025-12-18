//! Manifest parsing tests

use plix_launcher::manifest::{Manifest, ManifestFile};

#[test]
fn test_parse_valid_manifest() {
    let toml_content = r#"
manifest_version = 1
version = "1.3.0"
protocol_version = 1
release_date = 1734480000

[[files]]
path = "plix-client"
url = "https://releases.plix.example/v1.3.0/plix-client"
size = 47448064
sha256 = "a1b2c3d4e5f67890abcdef1234567890a1b2c3d4e5f67890abcdef1234567890"
executable = true

[[files]]
path = "assets/arenas/test_arena.toml"
url = "https://releases.plix.example/v1.3.0/assets/arenas/test_arena.toml"
size = 2048
sha256 = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
"#;

    let manifest: Manifest = toml::from_str(toml_content).expect("Failed to parse manifest");

    assert_eq!(manifest.manifest_version, 1);
    assert_eq!(manifest.version, "1.3.0");
    assert_eq!(manifest.protocol_version, Some(1));
    assert_eq!(manifest.release_date, 1734480000);
    assert_eq!(manifest.files.len(), 2);

    let client_file = &manifest.files[0];
    assert_eq!(client_file.path, "plix-client");
    assert_eq!(client_file.size, 47448064);
    assert!(client_file.executable);

    let arena_file = &manifest.files[1];
    assert_eq!(arena_file.path, "assets/arenas/test_arena.toml");
    assert!(!arena_file.executable);
}

#[test]
fn test_parse_minimal_manifest() {
    let toml_content = r#"
manifest_version = 1
version = "1.0.0"
release_date = 1734480000

[[files]]
path = "plix-client"
url = "https://releases.plix.example/v1.0.0/plix-client"
size = 45000000
sha256 = "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
"#;

    let manifest: Manifest = toml::from_str(toml_content).expect("Failed to parse manifest");

    assert_eq!(manifest.manifest_version, 1);
    assert_eq!(manifest.version, "1.0.0");
    assert_eq!(manifest.protocol_version, None); // Optional field
    assert_eq!(manifest.release_notes_url, None); // Optional field
    assert_eq!(manifest.files.len(), 1);
    assert!(!manifest.files[0].executable); // Default false
}

#[test]
fn test_parse_invalid_manifest_missing_version() {
    let toml_content = r#"
manifest_version = 1
release_date = 1734480000

[[files]]
path = "plix-client"
url = "https://releases.plix.example/v1.0.0/plix-client"
size = 45000000
sha256 = "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
"#;

    let result: Result<Manifest, _> = toml::from_str(toml_content);
    assert!(result.is_err());
}

#[test]
fn test_parse_invalid_manifest_missing_files() {
    let toml_content = r#"
manifest_version = 1
version = "1.0.0"
release_date = 1734480000
"#;

    let result: Result<Manifest, _> = toml::from_str(toml_content);
    assert!(result.is_err());
}

#[test]
fn test_parse_invalid_manifest_bad_toml() {
    let toml_content = r#"
this is not valid toml {{{
"#;

    let result: Result<Manifest, _> = toml::from_str(toml_content);
    assert!(result.is_err());
}

#[test]
fn test_manifest_file_with_all_fields() {
    let toml_content = r#"
path = "plix-client"
url = "https://example.com/plix-client"
size = 1000
sha256 = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
executable = true
"#;

    let file: ManifestFile = toml::from_str(toml_content).expect("Failed to parse file");

    assert_eq!(file.path, "plix-client");
    assert_eq!(file.url, "https://example.com/plix-client");
    assert_eq!(file.size, 1000);
    assert_eq!(
        file.sha256,
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
    );
    assert!(file.executable);
}

#[test]
fn test_manifest_serialization() {
    let manifest = Manifest {
        manifest_version: 1,
        version: "2.0.0".to_string(),
        protocol_version: Some(2),
        release_date: 1734567890,
        files: vec![ManifestFile {
            path: "game".to_string(),
            url: "https://cdn.example/game".to_string(),
            size: 50000000,
            sha256: "deadbeef".repeat(8),
            executable: true,
        }],
        release_notes_url: Some("https://example.com/notes".to_string()),
    };

    let toml_str = toml::to_string(&manifest).expect("Failed to serialize");
    let parsed: Manifest = toml::from_str(&toml_str).expect("Failed to parse");

    assert_eq!(parsed.version, "2.0.0");
    assert_eq!(parsed.files.len(), 1);
    assert_eq!(
        parsed.release_notes_url,
        Some("https://example.com/notes".to_string())
    );
}
