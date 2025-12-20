//! Mod manifest parsing and validation
//!
//! Mods declare their metadata and requirements in `mod.toml`.

use crate::capabilities::ManifestCapabilities;
use crate::errors::{err_invalid, err_unsupported, ModApiError};
use crate::ENGINE_API_VERSION;
use serde::{Deserialize, Serialize};

/// Maximum length for mod IDs
pub const MAX_MOD_ID_LENGTH: usize = 64;

/// Entrypoints for mod initialization
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Entrypoints {
    /// Server-side entry point function name
    #[serde(default)]
    pub server: Option<String>,
    /// Client-side entry point function name (future use)
    #[serde(default)]
    pub client: Option<String>,
}

/// Mod dependency declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    /// Mod ID of the dependency
    pub id: String,
    /// Minimum version required (SemVer)
    #[serde(default)]
    pub version: Option<String>,
    /// Whether this dependency is optional
    #[serde(default)]
    pub optional: bool,
}

/// Parsed and validated mod manifest from `mod.toml`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModManifest {
    /// Unique mod identifier (snake_case or kebab-case)
    pub id: String,
    /// Human-readable display name
    pub name: String,
    /// Mod version (SemVer)
    pub version: String,
    /// Mod author name
    #[serde(default)]
    pub author: Option<String>,
    /// Required API version
    pub api_version: u32,
    /// Minimum compatible API version
    #[serde(default)]
    pub min_api_version: Option<u32>,
    /// Maximum compatible API version
    #[serde(default)]
    pub max_api_version: Option<u32>,
    /// Requested capabilities
    #[serde(default)]
    pub capabilities: ManifestCapabilities,
    /// Entry points
    #[serde(default)]
    pub entrypoints: Entrypoints,
    /// Dependencies on other mods
    #[serde(default)]
    pub dependencies: Vec<Dependency>,
}

impl ModManifest {
    /// Parse a manifest from TOML string
    pub fn parse(toml_str: &str) -> Result<Self, ModApiError> {
        let manifest: ModManifest = toml::from_str(toml_str)
            .map_err(|e| err_invalid(format!("Failed to parse mod.toml: {}", e)))?;

        manifest.validate()?;
        Ok(manifest)
    }

    /// Load a manifest from a file path
    pub fn load_from_file(path: &std::path::Path) -> Result<Self, ModApiError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| err_invalid(format!("Failed to read mod.toml: {}", e)))?;
        Self::parse(&content)
    }

    /// Validate the manifest contents
    pub fn validate(&self) -> Result<(), ModApiError> {
        // Validate mod ID format
        Self::validate_mod_id(&self.id)?;

        // Validate API version compatibility
        Self::validate_api_version(self.api_version, self.min_api_version, self.max_api_version)?;

        // Validate version is non-empty
        if self.version.is_empty() {
            return Err(err_invalid("Mod version cannot be empty"));
        }

        // Validate name is non-empty
        if self.name.is_empty() {
            return Err(err_invalid("Mod name cannot be empty"));
        }

        Ok(())
    }

    /// Validate mod ID format: ^[a-z][a-z0-9_-]*$, max 64 chars
    fn validate_mod_id(id: &str) -> Result<(), ModApiError> {
        if id.is_empty() {
            return Err(err_invalid("Mod ID cannot be empty"));
        }

        if id.len() > MAX_MOD_ID_LENGTH {
            return Err(err_invalid(format!(
                "Mod ID exceeds maximum length of {} characters",
                MAX_MOD_ID_LENGTH
            )));
        }

        let mut chars = id.chars();

        // First character must be lowercase letter
        match chars.next() {
            Some(c) if c.is_ascii_lowercase() => {}
            _ => {
                return Err(err_invalid(
                    "Mod ID must start with a lowercase letter (a-z)",
                ));
            }
        }

        // Remaining characters must be lowercase letters, digits, underscore, or hyphen
        for c in chars {
            if !c.is_ascii_lowercase() && !c.is_ascii_digit() && c != '_' && c != '-' {
                return Err(err_invalid(format!(
                    "Mod ID contains invalid character '{}'. Only lowercase letters, digits, underscore, and hyphen are allowed",
                    c
                )));
            }
        }

        Ok(())
    }

    /// Validate API version compatibility with current engine
    fn validate_api_version(
        api_version: u32,
        min_api_version: Option<u32>,
        max_api_version: Option<u32>,
    ) -> Result<(), ModApiError> {
        let current = ENGINE_API_VERSION;

        // Check primary api_version
        if api_version > current {
            return Err(err_unsupported(format!(
                "Mod requires API version {} but engine only supports up to {}",
                api_version, current
            )));
        }

        // Check min_api_version if specified
        if let Some(min) = min_api_version {
            if current < min {
                return Err(err_unsupported(format!(
                    "Mod requires minimum API version {} but engine is at {}",
                    min, current
                )));
            }
        }

        // Check max_api_version if specified
        if let Some(max) = max_api_version {
            if current > max {
                return Err(err_unsupported(format!(
                    "Mod requires maximum API version {} but engine is at {}",
                    max, current
                )));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_manifest() {
        let toml = r#"
id = "my-mod"
name = "My Awesome Mod"
version = "1.0.0"
author = "Developer"
api_version = 1

[capabilities]
world_read = true
world_write = true

[entrypoints]
server = "on_load"
"#;

        let manifest = ModManifest::parse(toml).unwrap();
        assert_eq!(manifest.id, "my-mod");
        assert_eq!(manifest.name, "My Awesome Mod");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.author, Some("Developer".to_string()));
        assert_eq!(manifest.api_version, 1);
        assert!(manifest.capabilities.world_read);
        assert!(manifest.capabilities.world_write);
        assert!(!manifest.capabilities.entity_read);
        assert_eq!(manifest.entrypoints.server, Some("on_load".to_string()));
    }

    #[test]
    fn test_parse_minimal_manifest() {
        let toml = r#"
id = "minimal"
name = "Minimal Mod"
version = "0.1.0"
api_version = 1
"#;

        let manifest = ModManifest::parse(toml).unwrap();
        assert_eq!(manifest.id, "minimal");
        assert!(manifest.author.is_none());
        assert!(!manifest.capabilities.world_read);
    }

    #[test]
    fn test_invalid_mod_id_empty() {
        let toml = r#"
id = ""
name = "Test"
version = "1.0.0"
api_version = 1
"#;

        let result = ModManifest::parse(toml);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("empty"));
    }

    #[test]
    fn test_invalid_mod_id_uppercase() {
        let toml = r#"
id = "MyMod"
name = "Test"
version = "1.0.0"
api_version = 1
"#;

        let result = ModManifest::parse(toml);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("lowercase"));
    }

    #[test]
    fn test_invalid_mod_id_special_chars() {
        let toml = r#"
id = "my@mod"
name = "Test"
version = "1.0.0"
api_version = 1
"#;

        let result = ModManifest::parse(toml);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("invalid character"));
    }

    #[test]
    fn test_invalid_api_version_too_high() {
        let toml = r#"
id = "test-mod"
name = "Test"
version = "1.0.0"
api_version = 99
"#;

        let result = ModManifest::parse(toml);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, crate::errors::ErrorCode::Unsupported);
        assert!(err.message.contains("99"));
    }

    #[test]
    fn test_valid_mod_id_formats() {
        // snake_case
        assert!(ModManifest::validate_mod_id("my_mod").is_ok());
        // kebab-case
        assert!(ModManifest::validate_mod_id("my-mod").is_ok());
        // with numbers
        assert!(ModManifest::validate_mod_id("mod123").is_ok());
        // mixed
        assert!(ModManifest::validate_mod_id("my-mod_v2").is_ok());
    }

    #[test]
    fn test_mod_id_too_long() {
        let long_id = "a".repeat(65);
        let toml = format!(
            r#"
id = "{}"
name = "Test"
version = "1.0.0"
api_version = 1
"#,
            long_id
        );

        let result = ModManifest::parse(&toml);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("maximum length"));
    }

    #[test]
    fn test_api_version_range() {
        let toml = r#"
id = "test-mod"
name = "Test"
version = "1.0.0"
api_version = 1
min_api_version = 1
max_api_version = 1
"#;

        let manifest = ModManifest::parse(toml).unwrap();
        assert_eq!(manifest.min_api_version, Some(1));
        assert_eq!(manifest.max_api_version, Some(1));
    }
}
