//! Save Data Migrations
//!
//! Migrations for player save files (quest progress, inventory, etc).
//! Handles upgrades from v0.x (pre-release) to v1.x format.

use super::{Migration, MigrationError, MigrationResult, MigrationVersion};

/// Migration from v0 (unversioned) to v1.0 save format.
///
/// Changes:
/// - Adds `save_version = "1.0.0"` field
/// - Preserves all existing save data
/// - No data transformations needed for initial release
pub struct V0ToV1SaveMigration;

impl Migration for V0ToV1SaveMigration {
    fn from_version(&self) -> MigrationVersion {
        MigrationVersion::ZERO
    }

    fn to_version(&self) -> MigrationVersion {
        MigrationVersion::V1_0_0
    }

    fn description(&self) -> &str {
        "Add save_version field for v1.0 release"
    }

    fn needs_migration(&self, content: &[u8]) -> bool {
        let text = String::from_utf8_lossy(content);
        !text.contains("save_version")
    }

    fn migrate(&self, content: &[u8]) -> MigrationResult<Vec<u8>> {
        let text = String::from_utf8_lossy(content);

        // Check if already has version
        if text.contains("save_version") {
            return Ok(content.to_vec());
        }

        // Parse as TOML to validate and find insertion point
        let doc = text.parse::<toml_edit::DocumentMut>().map_err(|e| {
            MigrationError::ParseError(format!("Invalid TOML: {}", e))
        })?;

        // Create new document with version at top
        let mut new_doc = toml_edit::DocumentMut::new();

        // Add version field first
        new_doc["save_version"] = toml_edit::value("1.0.0");

        // Copy all existing keys
        for (key, value) in doc.iter() {
            new_doc[key] = value.clone();
        }

        Ok(new_doc.to_string().into_bytes())
    }
}

/// Extract the save version from TOML content.
///
/// Returns `MigrationVersion::ZERO` if no version field is present.
pub fn detect_save_version(content: &[u8]) -> MigrationVersion {
    let text = match std::str::from_utf8(content) {
        Ok(s) => s,
        Err(_) => return MigrationVersion::ZERO,
    };

    let doc = match text.parse::<toml_edit::DocumentMut>() {
        Ok(d) => d,
        Err(_) => return MigrationVersion::ZERO,
    };

    let version_str = match doc.get("save_version") {
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
        let migration = V0ToV1SaveMigration;

        let input = b"player_id = 1\nquest_progress = 5\n";
        let output = migration.migrate(input).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        assert!(output_str.contains("save_version = \"1.0.0\""));
        assert!(output_str.contains("player_id = 1"));
        assert!(output_str.contains("quest_progress = 5"));
    }

    #[test]
    fn test_v0_to_v1_migration_preserves_existing_version() {
        let migration = V0ToV1SaveMigration;

        let input = b"save_version = \"1.0.0\"\nplayer_id = 1\n";
        let output = migration.migrate(input).unwrap();

        // Should be unchanged
        assert_eq!(output, input.to_vec());
    }

    #[test]
    fn test_v0_to_v1_needs_migration() {
        let migration = V0ToV1SaveMigration;

        // Needs migration (no version)
        assert!(migration.needs_migration(b"player_id = 1\n"));

        // Does not need migration (has version)
        assert!(!migration.needs_migration(b"save_version = \"1.0.0\"\n"));
    }

    #[test]
    fn test_detect_save_version_present() {
        let content = b"save_version = \"1.0.0\"\nplayer_id = 1\n";
        let version = detect_save_version(content);

        assert_eq!(version, MigrationVersion::V1_0_0);
    }

    #[test]
    fn test_detect_save_version_missing() {
        let content = b"player_id = 1\nquest_progress = 5\n";
        let version = detect_save_version(content);

        assert_eq!(version, MigrationVersion::ZERO);
    }

    #[test]
    fn test_v0_to_v1_complex_save() {
        let migration = V0ToV1SaveMigration;

        let input = r#"
player_id = 12345
display_name = "TestPlayer"

[quest_progress]
chapter_1 = true
chapter_2 = false

[inventory]
coins = 100
items = ["sword", "shield"]

[stats]
kills = 50
deaths = 10
blocks_placed = 1000
"#;

        let output = migration.migrate(input.as_bytes()).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        // Version should be added
        assert!(output_str.contains("save_version = \"1.0.0\""));

        // All original content preserved
        assert!(output_str.contains("player_id = 12345"));
        assert!(output_str.contains("display_name = \"TestPlayer\""));
        assert!(output_str.contains("[quest_progress]"));
        assert!(output_str.contains("chapter_1 = true"));
        assert!(output_str.contains("[inventory]"));
        assert!(output_str.contains("coins = 100"));
        assert!(output_str.contains("[stats]"));
        assert!(output_str.contains("kills = 50"));
    }
}
