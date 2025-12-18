//! Display name type with validation
//!
//! A validated string representing a player's visible identity (1-32 characters).

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Minimum display name length after trimming
pub const MIN_LEN: usize = 1;
/// Maximum display name length
pub const MAX_LEN: usize = 32;
/// Default fallback name when validation fails
pub const DEFAULT_NAME: &str = "Player";

/// Validation error for display names
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DisplayNameError {
    #[error("Name is empty after trimming")]
    Empty,
    #[error("Name exceeds maximum length of {}", MAX_LEN)]
    TooLong,
    #[error("Name contains invalid characters")]
    InvalidCharacters,
}

/// A validated display name (1-32 characters, alphanumeric + underscore/hyphen/space)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DisplayName(String);

impl DisplayName {
    /// Create and validate a display name
    ///
    /// # Validation Rules
    /// - Trim leading/trailing whitespace
    /// - Length: 1-32 characters after trim
    /// - Allowed characters: a-z, A-Z, 0-9, underscore, hyphen, space
    pub fn new(input: &str) -> Result<Self, DisplayNameError> {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Err(DisplayNameError::Empty);
        }

        if trimmed.len() > MAX_LEN {
            return Err(DisplayNameError::TooLong);
        }

        // Check for invalid characters
        if !trimmed
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == ' ')
        {
            return Err(DisplayNameError::InvalidCharacters);
        }

        Ok(Self(trimmed.to_string()))
    }

    /// Create with sanitization (never fails, returns fallback if invalid)
    ///
    /// This is the safe alternative when you always need a valid name.
    pub fn sanitize(input: &str) -> Self {
        match Self::new(input) {
            Ok(name) => name,
            Err(_) => Self(DEFAULT_NAME.to_string()),
        }
    }

    /// Create a display name from a pre-validated string (internal use)
    ///
    /// # Safety
    /// The caller must ensure the string is valid (1-32 chars, valid charset).
    pub(crate) fn from_validated(s: String) -> Self {
        Self(s)
    }

    /// Get the inner string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get base name (without #N suffix if present)
    ///
    /// Examples:
    /// - "Alex" -> "Alex"
    /// - "Alex#2" -> "Alex"
    /// - "Alex#99" -> "Alex"
    pub fn base_name(&self) -> &str {
        // Find last # followed by digits only
        if let Some(hash_pos) = self.0.rfind('#') {
            let suffix = &self.0[hash_pos + 1..];
            if !suffix.is_empty() && suffix.chars().all(|c| c.is_ascii_digit()) {
                return &self.0[..hash_pos];
            }
        }
        &self.0
    }

    /// Check if name has disambiguation suffix (#N)
    pub fn has_suffix(&self) -> bool {
        self.base_name().len() < self.0.len()
    }

    /// Consume and return the inner string
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl fmt::Display for DisplayName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for DisplayName {
    fn default() -> Self {
        Self(DEFAULT_NAME.to_string())
    }
}

impl AsRef<str> for DisplayName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Validate a display name string without creating a DisplayName object.
///
/// Returns true if the name is valid (1-32 chars, alphanumeric + underscore/hyphen/space).
/// Used for quick validation in rename requests.
pub fn validate_display_name(input: &str) -> bool {
    let trimmed = input.trim();
    if trimmed.is_empty() || trimmed.len() > MAX_LEN {
        return false;
    }
    trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == ' ')
}

#[cfg(test)]
mod tests {
    use super::*;

    // T011: Unit tests for DisplayName validation

    #[test]
    fn test_valid_simple_names() {
        assert!(DisplayName::new("Alice").is_ok());
        assert!(DisplayName::new("Player123").is_ok());
        assert!(DisplayName::new("John_Doe").is_ok());
        assert!(DisplayName::new("Jane-Doe").is_ok());
        assert!(DisplayName::new("Two Words").is_ok());
    }

    #[test]
    fn test_valid_with_whitespace_trim() {
        let name = DisplayName::new("  Alice  ").unwrap();
        assert_eq!(name.as_str(), "Alice");
    }

    #[test]
    fn test_valid_single_char() {
        assert!(DisplayName::new("A").is_ok());
        assert!(DisplayName::new("1").is_ok());
    }

    #[test]
    fn test_valid_max_length() {
        let name = "a".repeat(32);
        assert!(DisplayName::new(&name).is_ok());
    }

    #[test]
    fn test_invalid_empty() {
        assert_eq!(DisplayName::new(""), Err(DisplayNameError::Empty));
        assert_eq!(DisplayName::new("   "), Err(DisplayNameError::Empty));
    }

    #[test]
    fn test_invalid_too_long() {
        let name = "a".repeat(33);
        assert_eq!(DisplayName::new(&name), Err(DisplayNameError::TooLong));
    }

    #[test]
    fn test_invalid_characters() {
        assert_eq!(
            DisplayName::new("Invalid@Name"),
            Err(DisplayNameError::InvalidCharacters)
        );
        assert_eq!(
            DisplayName::new("Name!"),
            Err(DisplayNameError::InvalidCharacters)
        );
        assert_eq!(
            DisplayName::new("Name#Tag"),
            Err(DisplayNameError::InvalidCharacters)
        );
        assert_eq!(
            DisplayName::new("Émile"),
            Err(DisplayNameError::InvalidCharacters)
        );
    }

    #[test]
    fn test_sanitize_valid() {
        let name = DisplayName::sanitize("Alice");
        assert_eq!(name.as_str(), "Alice");
    }

    #[test]
    fn test_sanitize_invalid_falls_back() {
        let name = DisplayName::sanitize("");
        assert_eq!(name.as_str(), DEFAULT_NAME);

        let name = DisplayName::sanitize("Invalid@Name!");
        assert_eq!(name.as_str(), DEFAULT_NAME);
    }

    #[test]
    fn test_base_name_no_suffix() {
        let name = DisplayName::new("Alex").unwrap();
        assert_eq!(name.base_name(), "Alex");
        assert!(!name.has_suffix());
    }

    #[test]
    fn test_base_name_with_suffix() {
        // Note: We can't create "Alex#2" via new() because # is invalid.
        // This tests the parsing for names that come from the server.
        let name = DisplayName::from_validated("Alex#2".to_string());
        assert_eq!(name.base_name(), "Alex");
        assert!(name.has_suffix());

        let name = DisplayName::from_validated("Alex#99".to_string());
        assert_eq!(name.base_name(), "Alex");
        assert!(name.has_suffix());
    }

    #[test]
    fn test_base_name_with_hash_not_suffix() {
        // Hash not followed by digits only
        let name = DisplayName::from_validated("Name#Tag".to_string());
        assert_eq!(name.base_name(), "Name#Tag"); // No stripping
        assert!(!name.has_suffix());
    }

    #[test]
    fn test_display_impl() {
        let name = DisplayName::new("Alice").unwrap();
        assert_eq!(format!("{}", name), "Alice");
    }

    #[test]
    fn test_default() {
        let name = DisplayName::default();
        assert_eq!(name.as_str(), DEFAULT_NAME);
    }

    #[test]
    fn test_serde_roundtrip() {
        let name = DisplayName::new("TestPlayer").unwrap();
        let json = serde_json::to_string(&name).unwrap();
        let parsed: DisplayName = serde_json::from_str(&json).unwrap();
        assert_eq!(name, parsed);
    }
}
