//! API Stability Markers for Mod API
//!
//! This module provides types and constants for marking API stability.
//! Mod developers can use these to understand which APIs are safe to rely on.
//!
//! # Stability Levels
//!
//! - **Stable**: Guaranteed for the v1.x lifetime. No breaking changes.
//! - **Experimental**: May change in minor versions. Use with caution.
//! - **Deprecated**: Will be removed in v2.0. Migration path provided.
//!
//! # Usage
//!
//! Check stability of an API before relying on it in production mods:
//!
//! ```rust
//! use plix_mod_core::stability::{Stability, STABLE, EXPERIMENTAL};
//!
//! // EntityApi is stable
//! assert!(STABLE.is_stable());
//!
//! // WorldApi::set_block is experimental
//! assert!(EXPERIMENTAL.is_experimental());
//! ```

use std::fmt;

/// Stability level for an API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stability {
    /// Stable API - guaranteed for v1.x lifetime.
    ///
    /// These APIs will not have breaking changes until v2.0.
    /// Mod developers can safely rely on these.
    Stable,

    /// Experimental API - may change in minor versions.
    ///
    /// These APIs are functional but may be modified in future
    /// v1.x releases. Use with caution in production mods.
    Experimental,

    /// Deprecated API - will be removed in v2.0.
    ///
    /// These APIs still work but are scheduled for removal.
    /// A migration path to replacement APIs is provided.
    Deprecated {
        /// Version when deprecation was announced
        since: &'static str,
        /// Replacement API (if any)
        replacement: Option<&'static str>,
    },
}

impl Stability {
    /// Returns true if this is a stable API.
    #[inline]
    pub const fn is_stable(&self) -> bool {
        matches!(self, Stability::Stable)
    }

    /// Returns true if this is an experimental API.
    #[inline]
    pub const fn is_experimental(&self) -> bool {
        matches!(self, Stability::Experimental)
    }

    /// Returns true if this is a deprecated API.
    #[inline]
    pub const fn is_deprecated(&self) -> bool {
        matches!(self, Stability::Deprecated { .. })
    }

    /// Get the deprecation version, if deprecated.
    #[inline]
    pub const fn deprecated_since(&self) -> Option<&'static str> {
        match self {
            Stability::Deprecated { since, .. } => Some(since),
            _ => None,
        }
    }

    /// Get the replacement API, if deprecated.
    #[inline]
    pub const fn replacement(&self) -> Option<&'static str> {
        match self {
            Stability::Deprecated { replacement, .. } => *replacement,
            _ => None,
        }
    }
}

impl fmt::Display for Stability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stability::Stable => write!(f, "stable"),
            Stability::Experimental => write!(f, "experimental"),
            Stability::Deprecated { since, replacement } => {
                write!(f, "deprecated since {}", since)?;
                if let Some(r) = replacement {
                    write!(f, ", use {} instead", r)?;
                }
                Ok(())
            }
        }
    }
}

/// Stable API marker.
///
/// Use this for APIs that are guaranteed for the v1.x lifetime.
pub const STABLE: Stability = Stability::Stable;

/// Experimental API marker.
///
/// Use this for APIs that may change in minor versions.
pub const EXPERIMENTAL: Stability = Stability::Experimental;

/// Deprecated API marker constructor.
///
/// Use this for APIs that will be removed in v2.0.
#[inline]
pub const fn deprecated(since: &'static str, replacement: Option<&'static str>) -> Stability {
    Stability::Deprecated { since, replacement }
}

/// Convenience constant for deprecated without replacement.
pub const DEPRECATED: Stability = Stability::Deprecated {
    since: "1.0.0",
    replacement: None,
};

// ============================================================================
// API Stability Registry
// ============================================================================

/// Stability information for each public API.
///
/// This struct documents the stability status of all public APIs in the mod system.
/// Mod developers can query this to understand API guarantees.
pub struct ApiStabilityInfo {
    /// API name (e.g., "EntityApi::get_player")
    pub name: &'static str,
    /// Stability level
    pub stability: Stability,
    /// Brief description
    pub description: &'static str,
}

/// Registry of all API stability information.
///
/// This is the authoritative source for API stability status.
pub const API_STABILITY: &[ApiStabilityInfo] = &[
    // Entity API - Stable
    ApiStabilityInfo {
        name: "EntityApi::get_players",
        stability: STABLE,
        description: "Get list of online player IDs",
    },
    ApiStabilityInfo {
        name: "EntityApi::get_player_info",
        stability: STABLE,
        description: "Get player information (name, position, health)",
    },
    ApiStabilityInfo {
        name: "EntityApi::get_player_count",
        stability: STABLE,
        description: "Get number of online players",
    },
    // Net API - Stable
    ApiStabilityInfo {
        name: "NetApi::send_message",
        stability: STABLE,
        description: "Send message to specific player",
    },
    ApiStabilityInfo {
        name: "NetApi::broadcast",
        stability: STABLE,
        description: "Send message to all players",
    },
    ApiStabilityInfo {
        name: "NetApi::send_channel",
        stability: STABLE,
        description: "Send message on custom channel",
    },
    // Timer API - Stable
    ApiStabilityInfo {
        name: "TimerApi::set_timer",
        stability: STABLE,
        description: "Schedule a one-shot or repeating timer",
    },
    ApiStabilityInfo {
        name: "TimerApi::cancel_timer",
        stability: STABLE,
        description: "Cancel an active timer",
    },
    // Event API - Stable
    ApiStabilityInfo {
        name: "EventBus::subscribe",
        stability: STABLE,
        description: "Subscribe to game events",
    },
    ApiStabilityInfo {
        name: "EventBus::unsubscribe",
        stability: STABLE,
        description: "Unsubscribe from game events",
    },
    // World API - Experimental
    ApiStabilityInfo {
        name: "WorldApi::get_block",
        stability: EXPERIMENTAL,
        description: "Query block at position",
    },
    ApiStabilityInfo {
        name: "WorldApi::set_block",
        stability: EXPERIMENTAL,
        description: "Set block at position (requires capability)",
    },
    ApiStabilityInfo {
        name: "WorldApi::raycast",
        stability: EXPERIMENTAL,
        description: "Cast ray to find block/entity",
    },
];

/// Find stability info for a named API.
pub fn get_api_stability(name: &str) -> Option<&'static ApiStabilityInfo> {
    API_STABILITY.iter().find(|info| info.name == name)
}

/// Get all stable APIs.
pub fn stable_apis() -> impl Iterator<Item = &'static ApiStabilityInfo> {
    API_STABILITY.iter().filter(|info| info.stability.is_stable())
}

/// Get all experimental APIs.
pub fn experimental_apis() -> impl Iterator<Item = &'static ApiStabilityInfo> {
    API_STABILITY
        .iter()
        .filter(|info| info.stability.is_experimental())
}

/// Get all deprecated APIs.
pub fn deprecated_apis() -> impl Iterator<Item = &'static ApiStabilityInfo> {
    API_STABILITY
        .iter()
        .filter(|info| info.stability.is_deprecated())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stability_stable() {
        assert!(STABLE.is_stable());
        assert!(!STABLE.is_experimental());
        assert!(!STABLE.is_deprecated());
    }

    #[test]
    fn test_stability_experimental() {
        assert!(!EXPERIMENTAL.is_stable());
        assert!(EXPERIMENTAL.is_experimental());
        assert!(!EXPERIMENTAL.is_deprecated());
    }

    #[test]
    fn test_stability_deprecated() {
        let dep = deprecated("1.0.0", Some("new_api"));
        assert!(!dep.is_stable());
        assert!(!dep.is_experimental());
        assert!(dep.is_deprecated());
        assert_eq!(dep.deprecated_since(), Some("1.0.0"));
        assert_eq!(dep.replacement(), Some("new_api"));
    }

    #[test]
    fn test_stability_display() {
        assert_eq!(format!("{}", STABLE), "stable");
        assert_eq!(format!("{}", EXPERIMENTAL), "experimental");
        assert_eq!(
            format!("{}", deprecated("1.0.0", Some("new_api"))),
            "deprecated since 1.0.0, use new_api instead"
        );
    }

    #[test]
    fn test_api_registry() {
        // All EntityApi should be stable
        let entity_api = get_api_stability("EntityApi::get_players").unwrap();
        assert!(entity_api.stability.is_stable());

        // WorldApi::set_block should be experimental
        let world_api = get_api_stability("WorldApi::set_block").unwrap();
        assert!(world_api.stability.is_experimental());
    }

    #[test]
    fn test_api_filters() {
        // Should have some stable APIs
        assert!(stable_apis().count() > 0);

        // Should have some experimental APIs
        assert!(experimental_apis().count() > 0);

        // Currently no deprecated APIs
        assert_eq!(deprecated_apis().count(), 0);
    }
}
