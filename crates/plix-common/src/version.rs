//! Version Information and Compatibility Checking
//!
//! This module provides version types and compatibility checking for the Plix engine.
//! All version constants are defined here as the single source of truth.

use serde::{Deserialize, Serialize};

/// Protocol version for network compatibility.
///
/// Client and server must have matching major versions to communicate.
/// Minor version differences are allowed (higher server supports lower client).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolVersion {
    /// Major version - must match for compatibility
    pub major: u8,
    /// Minor version - additive changes only
    pub minor: u8,
}

/// Current protocol version constant.
///
/// This is the authoritative protocol version for this build.
/// - Clients send this in the Connect handshake
/// - Servers validate incoming connections against this
pub const PROTOCOL_VERSION: ProtocolVersion = ProtocolVersion { major: 1, minor: 0 };

impl ProtocolVersion {
    /// Create a new ProtocolVersion.
    pub const fn new(major: u8, minor: u8) -> Self {
        Self { major, minor }
    }

    /// Check if this version is compatible with another version.
    ///
    /// Compatibility rules:
    /// - Major versions MUST match exactly
    /// - This version's minor MUST be >= other's minor (backward compatible)
    ///
    /// # Examples
    ///
    /// ```
    /// use plix_common::version::ProtocolVersion;
    ///
    /// let v1_0 = ProtocolVersion::new(1, 0);
    /// let v1_1 = ProtocolVersion::new(1, 1);
    /// let v2_0 = ProtocolVersion::new(2, 0);
    ///
    /// // Same version is compatible
    /// assert!(v1_0.is_compatible_with(&v1_0));
    ///
    /// // Higher minor can support lower minor
    /// assert!(v1_1.is_compatible_with(&v1_0));
    ///
    /// // Lower minor cannot support higher minor
    /// assert!(!v1_0.is_compatible_with(&v1_1));
    ///
    /// // Different major versions are never compatible
    /// assert!(!v2_0.is_compatible_with(&v1_0));
    /// ```
    pub fn is_compatible_with(&self, other: &ProtocolVersion) -> bool {
        self.major == other.major && self.minor >= other.minor
    }

    /// Get a display string for the version.
    pub fn display(&self) -> String {
        format!("{}.{}", self.major, self.minor)
    }
}

impl std::fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

/// Mod API version for mod compatibility.
///
/// Mods declare their required engine version in their manifest.
/// The engine checks compatibility before loading.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModApiVersion {
    /// Major version - breaking API changes
    pub major: u8,
    /// Minor version - additive API changes
    pub minor: u8,
}

impl ModApiVersion {
    /// Create a new ModApiVersion.
    pub const fn new(major: u8, minor: u8) -> Self {
        Self { major, minor }
    }

    /// Check if a mod requiring this version can run on an engine with the given version.
    ///
    /// A mod can run if:
    /// - Major versions match exactly
    /// - Engine minor >= mod required minor
    pub fn can_run_on(&self, engine: &ModApiVersion) -> bool {
        self.major == engine.major && self.minor <= engine.minor
    }

    /// Get a display string for the version.
    pub fn display(&self) -> String {
        format!("{}.{}", self.major, self.minor)
    }
}

impl std::fmt::Display for ModApiVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

/// Content schema version for content file compatibility.
///
/// Content definitions (quests, mobs, dungeons) are validated against this version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentSchemaVersion {
    /// Major version - breaking schema changes
    pub major: u8,
    /// Minor version - additive schema changes
    pub minor: u8,
}

impl ContentSchemaVersion {
    /// Create a new ContentSchemaVersion.
    pub const fn new(major: u8, minor: u8) -> Self {
        Self { major, minor }
    }

    /// Check if content with this schema version is compatible with the engine's expected version.
    pub fn is_compatible_with(&self, engine: &ContentSchemaVersion) -> bool {
        self.major == engine.major && self.minor <= engine.minor
    }

    /// Get a display string for the version.
    pub fn display(&self) -> String {
        format!("{}.{}", self.major, self.minor)
    }
}

impl std::fmt::Display for ContentSchemaVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

/// Current content schema version constant.
pub const CONTENT_SCHEMA_VERSION: ContentSchemaVersion = ContentSchemaVersion { major: 1, minor: 0 };

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_version_same_version_compatible() {
        let v1 = ProtocolVersion::new(1, 0);
        assert!(v1.is_compatible_with(&v1));
    }

    #[test]
    fn test_protocol_version_higher_minor_compatible() {
        let v1_1 = ProtocolVersion::new(1, 1);
        let v1_0 = ProtocolVersion::new(1, 0);
        assert!(v1_1.is_compatible_with(&v1_0));
    }

    #[test]
    fn test_protocol_version_lower_minor_incompatible() {
        let v1_0 = ProtocolVersion::new(1, 0);
        let v1_1 = ProtocolVersion::new(1, 1);
        assert!(!v1_0.is_compatible_with(&v1_1));
    }

    #[test]
    fn test_protocol_version_different_major_incompatible() {
        let v1_0 = ProtocolVersion::new(1, 0);
        let v2_0 = ProtocolVersion::new(2, 0);
        assert!(!v1_0.is_compatible_with(&v2_0));
        assert!(!v2_0.is_compatible_with(&v1_0));
    }

    #[test]
    fn test_mod_api_version_can_run_on_same() {
        let v1_0 = ModApiVersion::new(1, 0);
        assert!(v1_0.can_run_on(&v1_0));
    }

    #[test]
    fn test_mod_api_version_can_run_on_higher_minor() {
        let mod_v1_0 = ModApiVersion::new(1, 0);
        let engine_v1_1 = ModApiVersion::new(1, 1);
        assert!(mod_v1_0.can_run_on(&engine_v1_1));
    }

    #[test]
    fn test_mod_api_version_cannot_run_on_lower_minor() {
        let mod_v1_1 = ModApiVersion::new(1, 1);
        let engine_v1_0 = ModApiVersion::new(1, 0);
        assert!(!mod_v1_1.can_run_on(&engine_v1_0));
    }

    #[test]
    fn test_mod_api_version_cannot_run_on_different_major() {
        let mod_v1_0 = ModApiVersion::new(1, 0);
        let engine_v2_0 = ModApiVersion::new(2, 0);
        assert!(!mod_v1_0.can_run_on(&engine_v2_0));
    }

    #[test]
    fn test_protocol_version_display() {
        let v = ProtocolVersion::new(1, 2);
        assert_eq!(v.display(), "1.2");
        assert_eq!(format!("{}", v), "1.2");
    }

    #[test]
    fn test_content_schema_version_compatible() {
        let content_v1_0 = ContentSchemaVersion::new(1, 0);
        let engine_v1_1 = ContentSchemaVersion::new(1, 1);
        assert!(content_v1_0.is_compatible_with(&engine_v1_1));
    }

    #[test]
    fn test_content_schema_version_incompatible_major() {
        let content_v2_0 = ContentSchemaVersion::new(2, 0);
        let engine_v1_0 = ContentSchemaVersion::new(1, 0);
        assert!(!content_v2_0.is_compatible_with(&engine_v1_0));
    }
}
