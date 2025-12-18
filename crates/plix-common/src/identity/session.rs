//! Session and account identity types
//!
//! Provides unique identifiers for session tracking and future authentication.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique session identifier (per connection, server-local)
///
/// Used for logging, metrics, and anti-cheat correlation.
/// Each connection gets a new SessionId; it does not persist across reconnects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct SessionId(pub u64);

impl SessionId {
    /// Invalid/no session (value 0)
    pub const NONE: Self = Self(0);

    /// Create a new session ID
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Check if this is a valid session ID (non-zero)
    pub fn is_valid(&self) -> bool {
        self.0 != 0
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Session({})", self.0)
    }
}

/// Unique account identifier (v2 placeholder - always None in v1)
///
/// This type is reserved for future authentication support.
/// In v1, all players are anonymous and AccountId is not used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct AccountId(pub u64);

impl AccountId {
    /// Invalid/no account (anonymous player)
    pub const NONE: Self = Self(0);

    /// Create a new account ID
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Check if this is a valid account ID (non-zero)
    pub fn is_valid(&self) -> bool {
        self.0 != 0
    }
}

impl fmt::Display for AccountId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_valid() {
            write!(f, "Account({})", self.0)
        } else {
            write!(f, "Anonymous")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // T012: Unit tests for SessionId/AccountId

    #[test]
    fn test_session_id_none() {
        let session = SessionId::NONE;
        assert_eq!(session.0, 0);
        assert!(!session.is_valid());
    }

    #[test]
    fn test_session_id_valid() {
        let session = SessionId::new(42);
        assert_eq!(session.0, 42);
        assert!(session.is_valid());
    }

    #[test]
    fn test_session_id_display() {
        let session = SessionId::new(123);
        assert_eq!(format!("{}", session), "Session(123)");
    }

    #[test]
    fn test_session_id_default() {
        let session = SessionId::default();
        assert_eq!(session, SessionId::NONE);
    }

    #[test]
    fn test_session_id_serde_roundtrip() {
        let session = SessionId::new(999);
        let json = serde_json::to_string(&session).unwrap();
        let parsed: SessionId = serde_json::from_str(&json).unwrap();
        assert_eq!(session, parsed);
    }

    #[test]
    fn test_account_id_none() {
        let account = AccountId::NONE;
        assert_eq!(account.0, 0);
        assert!(!account.is_valid());
    }

    #[test]
    fn test_account_id_valid() {
        let account = AccountId::new(12345);
        assert_eq!(account.0, 12345);
        assert!(account.is_valid());
    }

    #[test]
    fn test_account_id_display() {
        let valid = AccountId::new(42);
        assert_eq!(format!("{}", valid), "Account(42)");

        let anonymous = AccountId::NONE;
        assert_eq!(format!("{}", anonymous), "Anonymous");
    }

    #[test]
    fn test_account_id_default() {
        let account = AccountId::default();
        assert_eq!(account, AccountId::NONE);
    }

    #[test]
    fn test_account_id_serde_roundtrip() {
        let account = AccountId::new(54321);
        let json = serde_json::to_string(&account).unwrap();
        let parsed: AccountId = serde_json::from_str(&json).unwrap();
        assert_eq!(account, parsed);
    }
}
