//! Integration tests for mod distribution
//!
//! These tests use the mock registry in tests/fixtures/mock_registry/

use plix_mod_distribution::{
    config::{DistributionConfig, ModRequirement, RegistryConfig},
    index::RegistryIndex,
    lockfile::Lockfile,
    resolver::Resolver,
    ModId,
};
use semver::VersionReq;
use std::collections::HashMap;

/// Get the path to the mock registry
fn mock_registry_path() -> std::path::PathBuf {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests")
        .join("fixtures")
        .join("mock_registry")
}

/// Load the mock registry index
fn load_mock_index() -> Result<RegistryIndex, Box<dyn std::error::Error>> {
    let index_path = mock_registry_path().join("index.json");
    let content = std::fs::read_to_string(&index_path)?;
    Ok(RegistryIndex::from_json(&content)?)
}

#[test]
fn test_load_mock_registry_index() {
    let index = load_mock_index().expect("Should load mock index");

    assert_eq!(index.name, "Mock Test Registry");
    assert!(!index.mods.is_empty(), "Index should have mods");

    // Check test-core exists
    let core_mod = index.find_mod(&ModId::new_unchecked("test-core"));
    assert!(core_mod.is_some(), "test-core should exist");

    let core_mod = core_mod.unwrap();
    assert!(!core_mod.versions.is_empty(), "test-core should have versions");
}

#[test]
fn test_resolve_simple_mod() {
    let index = load_mock_index().expect("Should load mock index");

    let mut indexes = HashMap::new();
    indexes.insert("mock".to_string(), index);

    let config = DistributionConfig {
        registries: vec![RegistryConfig {
            name: "mock".to_string(),
            url: mock_registry_path().to_string_lossy().to_string(),
            priority: 1,
            enabled: true,
        }],
        mods: vec![ModRequirement {
            id: ModId::new_unchecked("test-core"),
            version: VersionReq::parse("^1.0").unwrap(),
            pinned: false,
            optional: false,
        }],
        ..Default::default()
    };

    let resolver = Resolver::new(&indexes, &config);
    let result = resolver.resolve(None);

    assert!(result.is_ok(), "Resolution should succeed: {:?}", result.err());

    let resolved = result.unwrap();
    assert_eq!(resolved.mods.len(), 1, "Should resolve one mod");

    let resolved_mod = &resolved.mods[0];
    assert_eq!(resolved_mod.id.as_str(), "test-core");
    // Should get latest compatible version (1.1.0)
    assert_eq!(resolved_mod.version.major, 1);
}

#[test]
fn test_resolve_with_transitive_deps() {
    let index = load_mock_index().expect("Should load mock index");

    let mut indexes = HashMap::new();
    indexes.insert("mock".to_string(), index);

    let config = DistributionConfig {
        registries: vec![RegistryConfig {
            name: "mock".to_string(),
            url: mock_registry_path().to_string_lossy().to_string(),
            priority: 1,
            enabled: true,
        }],
        mods: vec![ModRequirement {
            id: ModId::new_unchecked("test-app"),
            version: VersionReq::parse("^3.0").unwrap(),
            pinned: false,
            optional: false,
        }],
        ..Default::default()
    };

    let resolver = Resolver::new(&indexes, &config);
    let result = resolver.resolve(None);

    assert!(result.is_ok(), "Resolution should succeed: {:?}", result.err());

    let resolved = result.unwrap();

    // Should resolve: test-app -> test-utils -> test-core
    assert_eq!(resolved.mods.len(), 3, "Should resolve three mods (including transitive deps)");

    // Check all required mods are present
    let mod_ids: Vec<_> = resolved.mods.iter().map(|m| m.id.as_str()).collect();
    assert!(mod_ids.contains(&"test-app"), "Should have test-app");
    assert!(mod_ids.contains(&"test-utils"), "Should have test-utils");
    assert!(mod_ids.contains(&"test-core"), "Should have test-core");

    // Verify topological order: dependencies should come before dependents
    let positions: HashMap<_, _> = resolved
        .mods
        .iter()
        .enumerate()
        .map(|(i, m)| (m.id.as_str(), i))
        .collect();

    assert!(
        positions["test-core"] < positions["test-utils"],
        "test-core should come before test-utils"
    );
    assert!(
        positions["test-utils"] < positions["test-app"],
        "test-utils should come before test-app"
    );
}

#[test]
fn test_lockfile_generation() {
    let index = load_mock_index().expect("Should load mock index");

    let mut indexes = HashMap::new();
    indexes.insert("mock".to_string(), index);

    let config = DistributionConfig {
        registries: vec![RegistryConfig {
            name: "mock".to_string(),
            url: mock_registry_path().to_string_lossy().to_string(),
            priority: 1,
            enabled: true,
        }],
        mods: vec![ModRequirement {
            id: ModId::new_unchecked("test-utils"),
            version: VersionReq::parse("^2.0").unwrap(),
            pinned: false,
            optional: false,
        }],
        ..Default::default()
    };

    let resolver = Resolver::new(&indexes, &config);
    let resolved = resolver.resolve(None).expect("Resolution should succeed");

    // Generate lockfile
    let lockfile = Lockfile::from_resolved(&resolved);

    assert_eq!(lockfile.mods.len(), 2, "Lockfile should have 2 entries");

    // Verify mods are in lockfile
    let locked_utils = lockfile.find_mod(&ModId::new_unchecked("test-utils"));
    assert!(locked_utils.is_some(), "test-utils should be in lockfile");

    let locked_core = lockfile.find_mod(&ModId::new_unchecked("test-core"));
    assert!(locked_core.is_some(), "test-core should be in lockfile");

    // Verify SHA256 hashes are present
    let locked_utils = locked_utils.unwrap();
    assert!(!locked_utils.sha256.is_empty(), "SHA256 should be present");
    assert_eq!(locked_utils.sha256.len(), 64, "SHA256 should be 64 hex chars");
}

#[test]
fn test_lockfile_persistence() {
    let temp = tempfile::tempdir().unwrap();
    let lockfile_path = temp.path().join("mods.lock");

    let index = load_mock_index().expect("Should load mock index");

    let mut indexes = HashMap::new();
    indexes.insert("mock".to_string(), index);

    let config = DistributionConfig {
        registries: vec![RegistryConfig {
            name: "mock".to_string(),
            url: mock_registry_path().to_string_lossy().to_string(),
            priority: 1,
            enabled: true,
        }],
        mods: vec![ModRequirement {
            id: ModId::new_unchecked("test-core"),
            version: VersionReq::parse("^1.0").unwrap(),
            pinned: false,
            optional: false,
        }],
        ..Default::default()
    };

    let resolver = Resolver::new(&indexes, &config);
    let resolved = resolver.resolve(None).expect("Resolution should succeed");

    // Write lockfile
    let lockfile = Lockfile::from_resolved(&resolved);
    lockfile.write(&lockfile_path).expect("Should write lockfile");

    assert!(lockfile_path.exists(), "Lockfile should exist on disk");

    // Read back lockfile
    let loaded = Lockfile::load(&lockfile_path).expect("Should load lockfile");

    assert_eq!(loaded.mods.len(), lockfile.mods.len());
    assert_eq!(loaded.version, lockfile.version);
}

#[test]
fn test_mod_not_found() {
    let index = load_mock_index().expect("Should load mock index");

    let mut indexes = HashMap::new();
    indexes.insert("mock".to_string(), index);

    let config = DistributionConfig {
        registries: vec![RegistryConfig {
            name: "mock".to_string(),
            url: mock_registry_path().to_string_lossy().to_string(),
            priority: 1,
            enabled: true,
        }],
        mods: vec![ModRequirement {
            id: ModId::new_unchecked("nonexistent-mod"),
            version: VersionReq::parse("^1.0").unwrap(),
            pinned: false,
            optional: false,
        }],
        ..Default::default()
    };

    let resolver = Resolver::new(&indexes, &config);
    let result = resolver.resolve(None);

    assert!(result.is_err(), "Resolution should fail for nonexistent mod");
}

#[test]
fn test_version_not_found() {
    let index = load_mock_index().expect("Should load mock index");

    let mut indexes = HashMap::new();
    indexes.insert("mock".to_string(), index);

    let config = DistributionConfig {
        registries: vec![RegistryConfig {
            name: "mock".to_string(),
            url: mock_registry_path().to_string_lossy().to_string(),
            priority: 1,
            enabled: true,
        }],
        mods: vec![ModRequirement {
            id: ModId::new_unchecked("test-core"),
            // Request version 99.0 which doesn't exist
            version: VersionReq::parse("^99.0").unwrap(),
            pinned: false,
            optional: false,
        }],
        ..Default::default()
    };

    let resolver = Resolver::new(&indexes, &config);
    let result = resolver.resolve(None);

    assert!(result.is_err(), "Resolution should fail for unavailable version");
}

#[test]
fn test_multiple_mods_resolution() {
    let index = load_mock_index().expect("Should load mock index");

    let mut indexes = HashMap::new();
    indexes.insert("mock".to_string(), index);

    let config = DistributionConfig {
        registries: vec![RegistryConfig {
            name: "mock".to_string(),
            url: mock_registry_path().to_string_lossy().to_string(),
            priority: 1,
            enabled: true,
        }],
        mods: vec![
            ModRequirement {
                id: ModId::new_unchecked("test-core"),
                version: VersionReq::parse("^1.0").unwrap(),
                pinned: false,
                optional: false,
            },
            ModRequirement {
                id: ModId::new_unchecked("test-utils"),
                version: VersionReq::parse("^2.0").unwrap(),
                pinned: false,
                optional: false,
            },
        ],
        ..Default::default()
    };

    let resolver = Resolver::new(&indexes, &config);
    let result = resolver.resolve(None);

    assert!(result.is_ok(), "Resolution should succeed: {:?}", result.err());

    let resolved = result.unwrap();
    // test-core is shared between direct requirement and test-utils dependency
    // So we should have exactly 2 mods (not 3)
    assert_eq!(resolved.mods.len(), 2, "Should resolve two unique mods");
}

#[test]
fn test_config_file_loading() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("server_mods.toml");

    let config_content = r#"
[download]
connect_timeout_secs = 10
read_timeout_secs = 30
retries = 3
max_bundle_size = 10485760

[[registries]]
name = "test"
url = "file:///tmp/registry"
priority = 1
enabled = true

[[mods]]
id = "test-mod"
version = "^1.0.0"
"#;

    std::fs::write(&config_path, config_content).unwrap();

    let config = DistributionConfig::load(&config_path).expect("Should load config");

    assert_eq!(config.registries.len(), 1);
    assert_eq!(config.registries[0].name, "test");
    assert_eq!(config.mods.len(), 1);
    assert_eq!(config.mods[0].id.as_str(), "test-mod");
}
