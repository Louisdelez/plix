//! Name registry for unique display name management
//!
//! Manages active display names with disambiguation support.
//! When multiple players request the same name, the server assigns
//! unique suffixes (#2, #3, etc.) to maintain uniqueness.

use std::collections::{HashMap, HashSet};

use plix_common::identity::DisplayName;
use plix_common::types::PlayerId;

/// Maximum suffix number (e.g., Alex#99 is the max)
const MAX_SUFFIX: u32 = 99;

/// Manages unique display names for connected players
#[derive(Debug, Default)]
pub struct NameRegistry {
    /// Active display names (full names including suffix)
    active_names: HashSet<String>,
    /// For each base name, track which suffix numbers are in use
    suffix_map: HashMap<String, HashSet<u32>>,
    /// Map from player ID to their assigned display name
    player_names: HashMap<PlayerId, String>,
}

impl NameRegistry {
    /// Create a new empty name registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a display name for a player
    ///
    /// Returns the assigned display name (may have #N suffix if base name is taken).
    /// Returns None if the name cannot be assigned (max suffix reached).
    ///
    /// # Arguments
    /// * `player_id` - The player's ID
    /// * `requested_name` - The requested display name (will be validated/sanitized)
    pub fn register(&mut self, player_id: PlayerId, requested_name: &str) -> Option<String> {
        // Validate and sanitize the name
        let display_name = DisplayName::sanitize(requested_name);
        let base_name = display_name.as_str().to_string();

        // Find an available name
        let assigned_name = self.find_available_name(&base_name)?;

        // Register the name
        self.active_names.insert(assigned_name.clone());
        self.player_names.insert(player_id, assigned_name.clone());

        // Track suffix usage
        let suffix = self.extract_suffix(&assigned_name, &base_name);
        self.suffix_map.entry(base_name).or_default().insert(suffix);

        Some(assigned_name)
    }

    /// Unregister a player's name on disconnect
    ///
    /// Frees the name and suffix for reuse by future players.
    pub fn unregister(&mut self, player_id: PlayerId) {
        if let Some(name) = self.player_names.remove(&player_id) {
            self.active_names.remove(&name);

            // Extract base name and suffix for cleanup
            let (base_name, suffix) = self.parse_name_suffix(&name);
            if let Some(suffixes) = self.suffix_map.get_mut(&base_name) {
                suffixes.remove(&suffix);
                if suffixes.is_empty() {
                    self.suffix_map.remove(&base_name);
                }
            }
        }
    }

    /// Get a player's assigned display name
    pub fn get_name(&self, player_id: PlayerId) -> Option<&str> {
        self.player_names.get(&player_id).map(|s| s.as_str())
    }

    /// Rename a player (for /name command)
    ///
    /// Returns the new assigned name, or None if rename failed.
    pub fn rename(&mut self, player_id: PlayerId, new_name: &str) -> Option<String> {
        // First unregister the old name
        self.unregister(player_id);
        // Then register the new name
        self.register(player_id, new_name)
    }

    /// Check if a base name is available (no suffix needed)
    pub fn is_base_available(&self, base_name: &str) -> bool {
        !self.active_names.contains(base_name)
    }

    /// Get number of active names
    pub fn count(&self) -> usize {
        self.player_names.len()
    }

    /// Find an available name, adding suffix if necessary
    fn find_available_name(&self, base_name: &str) -> Option<String> {
        // Try base name first
        if !self.active_names.contains(base_name) {
            return Some(base_name.to_string());
        }

        // Find lowest available suffix (starting at 2)
        let used_suffixes = self.suffix_map.get(base_name);
        for suffix in 2..=MAX_SUFFIX {
            let name_with_suffix = format!("{}#{}", base_name, suffix);
            if !self.active_names.contains(&name_with_suffix) {
                // Also check if suffix is in use (belt and suspenders)
                let suffix_in_use = used_suffixes.map(|s| s.contains(&suffix)).unwrap_or(false);
                if !suffix_in_use {
                    return Some(name_with_suffix);
                }
            }
        }

        // All suffixes exhausted
        None
    }

    /// Extract suffix number from a name (0 for base name, N for #N suffix)
    fn extract_suffix(&self, full_name: &str, base_name: &str) -> u32 {
        if full_name == base_name {
            return 0;
        }

        // Parse #N suffix
        if let Some(suffix_str) = full_name.strip_prefix(&format!("{}#", base_name)) {
            suffix_str.parse().unwrap_or(0)
        } else {
            0
        }
    }

    /// Parse a name into base name and suffix
    fn parse_name_suffix(&self, name: &str) -> (String, u32) {
        if let Some(hash_pos) = name.rfind('#') {
            let suffix_str = &name[hash_pos + 1..];
            if !suffix_str.is_empty() && suffix_str.chars().all(|c| c.is_ascii_digit()) {
                if let Ok(suffix) = suffix_str.parse::<u32>() {
                    return (name[..hash_pos].to_string(), suffix);
                }
            }
        }
        (name.to_string(), 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // T029: Unit tests for NameRegistry

    #[test]
    fn test_register_unique_name() {
        let mut registry = NameRegistry::new();
        let name = registry.register(PlayerId(1), "Alex").unwrap();
        assert_eq!(name, "Alex");
        assert_eq!(registry.get_name(PlayerId(1)), Some("Alex"));
    }

    #[test]
    fn test_register_duplicate_gets_suffix() {
        let mut registry = NameRegistry::new();

        let name1 = registry.register(PlayerId(1), "Alex").unwrap();
        let name2 = registry.register(PlayerId(2), "Alex").unwrap();

        assert_eq!(name1, "Alex");
        assert_eq!(name2, "Alex#2");
    }

    #[test]
    fn test_register_multiple_duplicates() {
        let mut registry = NameRegistry::new();

        let name1 = registry.register(PlayerId(1), "Player").unwrap();
        let name2 = registry.register(PlayerId(2), "Player").unwrap();
        let name3 = registry.register(PlayerId(3), "Player").unwrap();

        assert_eq!(name1, "Player");
        assert_eq!(name2, "Player#2");
        assert_eq!(name3, "Player#3");
    }

    #[test]
    fn test_unregister_frees_name() {
        let mut registry = NameRegistry::new();

        registry.register(PlayerId(1), "Alex").unwrap();
        registry.unregister(PlayerId(1));

        // Name should be available again
        let name = registry.register(PlayerId(2), "Alex").unwrap();
        assert_eq!(name, "Alex");
    }

    #[test]
    fn test_unregister_frees_suffix() {
        let mut registry = NameRegistry::new();

        registry.register(PlayerId(1), "Alex").unwrap();
        let name2 = registry.register(PlayerId(2), "Alex").unwrap();
        assert_eq!(name2, "Alex#2");

        // Unregister the #2 player
        registry.unregister(PlayerId(2));

        // New player should get #2 again (lowest available)
        let name3 = registry.register(PlayerId(3), "Alex").unwrap();
        assert_eq!(name3, "Alex#2");
    }

    #[test]
    fn test_sanitize_invalid_name() {
        let mut registry = NameRegistry::new();

        // Empty name should fall back to "Player"
        let name = registry.register(PlayerId(1), "").unwrap();
        assert_eq!(name, "Player");

        // Invalid characters should fall back to "Player"
        let name2 = registry.register(PlayerId(2), "Invalid@Name!").unwrap();
        assert_eq!(name2, "Player#2");
    }

    #[test]
    fn test_max_suffix_exhaustion() {
        let mut registry = NameRegistry::new();

        // Register base name
        registry.register(PlayerId(0), "Test").unwrap();

        // Fill up all suffixes (2 through 99)
        for i in 2..=MAX_SUFFIX {
            let name = registry.register(PlayerId(i as u16), "Test").unwrap();
            assert_eq!(name, format!("Test#{}", i));
        }

        // Next registration should fail
        let result = registry.register(PlayerId(100), "Test");
        assert!(result.is_none());
    }

    #[test]
    fn test_rename_player() {
        let mut registry = NameRegistry::new();

        registry.register(PlayerId(1), "Alex").unwrap();
        let new_name = registry.rename(PlayerId(1), "Bob").unwrap();

        assert_eq!(new_name, "Bob");
        assert_eq!(registry.get_name(PlayerId(1)), Some("Bob"));

        // Alex should be free now
        let name = registry.register(PlayerId(2), "Alex").unwrap();
        assert_eq!(name, "Alex");
    }

    #[test]
    fn test_is_base_available() {
        let mut registry = NameRegistry::new();

        assert!(registry.is_base_available("Alex"));

        registry.register(PlayerId(1), "Alex").unwrap();

        assert!(!registry.is_base_available("Alex"));
        assert!(registry.is_base_available("Bob"));
    }

    #[test]
    fn test_count() {
        let mut registry = NameRegistry::new();

        assert_eq!(registry.count(), 0);

        registry.register(PlayerId(1), "Alex").unwrap();
        assert_eq!(registry.count(), 1);

        registry.register(PlayerId(2), "Bob").unwrap();
        assert_eq!(registry.count(), 2);

        registry.unregister(PlayerId(1));
        assert_eq!(registry.count(), 1);
    }

    #[test]
    fn test_whitespace_trim() {
        let mut registry = NameRegistry::new();

        let name = registry.register(PlayerId(1), "  Alex  ").unwrap();
        assert_eq!(name, "Alex");
    }
}
