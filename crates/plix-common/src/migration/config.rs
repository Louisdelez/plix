//! Configuration File Migrations
//!
//! Migrations for server and client configuration files.
//! Handles upgrades from v0.x (pre-release) to v1.x format.

use super::{Migration, MigrationError, MigrationResult, MigrationVersion};

/// Migration from v0 (unversioned) to v1.0 configuration format.
///
/// Changes:
/// - Adds `config_version = "1.0.0"` field
/// - Preserves all existing configuration values
pub struct V0ToV1ConfigMigration;

impl Migration for V0ToV1ConfigMigration {
    fn from_version(&self) -> MigrationVersion {
        MigrationVersion::ZERO
    }

    fn to_version(&self) -> MigrationVersion {
        MigrationVersion::V1_0_0
    }

    fn description(&self) -> &str {
        "Add config_version field for v1.0 release"
    }

    fn needs_migration(&self, content: &[u8]) -> bool {
        let text = String::from_utf8_lossy(content);
        !text.contains("config_version")
    }

    fn migrate(&self, content: &[u8]) -> MigrationResult<Vec<u8>> {
        let text = String::from_utf8_lossy(content);

        // Check if already has version
        if text.contains("config_version") {
            return Ok(content.to_vec());
        }

        // Parse as TOML to validate and find insertion point
        let doc = text.parse::<toml_edit::DocumentMut>().map_err(|e| {
            MigrationError::ParseError(format!("Invalid TOML: {}", e))
        })?;

        // Create new document with version at top
        let mut new_doc = toml_edit::DocumentMut::new();

        // Add version field first
        new_doc["config_version"] = toml_edit::value("1.0.0");

        // Copy all existing keys
        for (key, value) in doc.iter() {
            new_doc[key] = value.clone();
        }

        Ok(new_doc.to_string().into_bytes())
    }
}

/// Extract the config version from TOML content.
///
/// Returns `MigrationVersion::ZERO` if no version field is present.
pub fn detect_config_version(content: &[u8]) -> MigrationVersion {
    let text = match std::str::from_utf8(content) {
        Ok(s) => s,
        Err(_) => return MigrationVersion::ZERO,
    };

    let doc = match text.parse::<toml_edit::DocumentMut>() {
        Ok(d) => d,
        Err(_) => return MigrationVersion::ZERO,
    };

    let version_str = match doc.get("config_version") {
        Some(item) => match item.as_str() {
            Some(s) => s,
            None => return MigrationVersion::ZERO,
        },
        None => return MigrationVersion::ZERO,
    };

    parse_version_string(version_str).unwrap_or(MigrationVersion::ZERO)
}

/// Parse a version string like "1.0.0" into a MigrationVersion.
fn parse_version_string(s: &str) -> Option<MigrationVersion> {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 3 {
        return None;
    }

    let major = parts[0].parse().ok()?;
    let minor = parts[1].parse().ok()?;
    let patch = parts[2].parse().ok()?;

    Some(MigrationVersion::new(major, minor, patch))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v0_to_v1_migration_adds_version() {
        let migration = V0ToV1ConfigMigration;

        let input = b"server_name = \"My Server\"\nmax_players = 32\n";
        let output = migration.migrate(input).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        assert!(output_str.contains("config_version = \"1.0.0\""));
        assert!(output_str.contains("server_name = \"My Server\""));
        assert!(output_str.contains("max_players = 32"));
    }

    #[test]
    fn test_v0_to_v1_migration_preserves_existing_version() {
        let migration = V0ToV1ConfigMigration;

        let input = b"config_version = \"1.0.0\"\nserver_name = \"Test\"\n";
        let output = migration.migrate(input).unwrap();

        // Should be unchanged
        assert_eq!(output, input.to_vec());
    }

    #[test]
    fn test_v0_to_v1_needs_migration() {
        let migration = V0ToV1ConfigMigration;

        // Needs migration (no version)
        assert!(migration.needs_migration(b"server_name = \"Test\"\n"));

        // Does not need migration (has version)
        assert!(!migration.needs_migration(b"config_version = \"1.0.0\"\n"));
    }

    #[test]
    fn test_v0_to_v1_invalid_toml() {
        let migration = V0ToV1ConfigMigration;

        let input = b"this is not valid toml {{{";
        let result = migration.migrate(input);

        assert!(matches!(result, Err(MigrationError::ParseError(_))));
    }

    #[test]
    fn test_detect_config_version_present() {
        let content = b"config_version = \"1.0.0\"\nname = \"test\"\n";
        let version = detect_config_version(content);

        assert_eq!(version, MigrationVersion::V1_0_0);
    }

    #[test]
    fn test_detect_config_version_missing() {
        let content = b"name = \"test\"\nmax_players = 32\n";
        let version = detect_config_version(content);

        assert_eq!(version, MigrationVersion::ZERO);
    }

    #[test]
    fn test_detect_config_version_invalid() {
        // Not a string
        let content = b"config_version = 123\n";
        assert_eq!(detect_config_version(content), MigrationVersion::ZERO);

        // Invalid format
        let content = b"config_version = \"1.0\"\n";
        assert_eq!(detect_config_version(content), MigrationVersion::ZERO);

        // Invalid TOML
        let content = b"not valid {{{";
        assert_eq!(detect_config_version(content), MigrationVersion::ZERO);
    }

    #[test]
    fn test_parse_version_string() {
        assert_eq!(
            parse_version_string("1.0.0"),
            Some(MigrationVersion::new(1, 0, 0))
        );
        assert_eq!(
            parse_version_string("2.3.4"),
            Some(MigrationVersion::new(2, 3, 4))
        );
        assert_eq!(parse_version_string("1.0"), None);
        assert_eq!(parse_version_string("invalid"), None);
        assert_eq!(parse_version_string(""), None);
    }

    #[test]
    fn test_v0_to_v1_complex_config() {
        let migration = V0ToV1ConfigMigration;

        let input = r#"
server_name = "My Epic Server"
max_players = 64

[network]
port = 7777
tick_rate = 60

[game]
mode = "tdm"
respawn_time = 5.0

[[arenas]]
name = "dust2"
path = "arenas/dust2.toml"
"#;

        let output = migration.migrate(input.as_bytes()).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        // Version should be added
        assert!(output_str.contains("config_version = \"1.0.0\""));

        // All original content preserved
        assert!(output_str.contains("server_name = \"My Epic Server\""));
        assert!(output_str.contains("max_players = 64"));
        assert!(output_str.contains("[network]"));
        assert!(output_str.contains("port = 7777"));
        assert!(output_str.contains("[game]"));
        assert!(output_str.contains("mode = \"tdm\""));
        assert!(output_str.contains("[[arenas]]"));
        assert!(output_str.contains("name = \"dust2\""));
    }
}
