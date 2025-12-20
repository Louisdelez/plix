//! Security Limits - Centralized security boundaries for the Plix protocol
//!
//! All security-related limits are defined here as compile-time constants.
//! These values are authoritative and must be used throughout the codebase
//! instead of local magic numbers.
//!
//! # Categories
//! - Protocol limits: Message and field size boundaries
//! - Handshake limits: Connection establishment boundaries
//! - Payload sync limits: Mod payload transfer boundaries
//! - Rate limits: Message frequency boundaries
//! - Parser limits: File parsing boundaries
//! - Strike limits: Violation tracking boundaries

use std::fmt;

// =============================================================================
// Protocol Limits
// =============================================================================

/// Maximum raw packet size (64 KB)
pub const MAX_PACKET_BYTES: usize = 65536;

/// Maximum decoded message size (64 KB)
pub const MAX_MESSAGE_BYTES: usize = 65536;

/// Maximum string field length (1 KB)
pub const MAX_STRING_BYTES: usize = 1024;

/// Maximum list/array elements
pub const MAX_LIST_LEN: usize = 256;

// =============================================================================
// Handshake Limits
// =============================================================================

/// Maximum cached payload hashes a client can send
pub const MAX_CACHED_PAYLOAD_HASHES: usize = 256;

/// Time to complete handshake before timeout (seconds)
pub const HANDSHAKE_TIMEOUT_SECS: u32 = 10;

/// Maximum pending handshakes per source IP
pub const MAX_PENDING_PER_SOURCE: usize = 10;

// =============================================================================
// Payload Sync Limits
// =============================================================================

/// Maximum payload size for mod sync (25 MB)
pub const MAX_PAYLOAD_BYTES: usize = 26_214_400;

/// Maximum chunks per transfer
pub const MAX_CHUNKS_PER_TRANSFER: usize = 1024;

/// Maximum resend requests per window
pub const MAX_RESEND_PER_WINDOW: usize = 16;

/// Time to complete payload transfer before timeout (seconds)
pub const TRANSFER_TIMEOUT_SECS: u32 = 30;

/// Time between chunk acks before timeout (seconds)
pub const CHUNK_TIMEOUT_SECS: u32 = 5;

// =============================================================================
// Rate Limits
// =============================================================================

/// Maximum messages per client per second (global)
pub const GLOBAL_MSG_PER_SEC: u32 = 200;

/// Initial token bucket capacity for mod channel messages
pub const MOD_CHANNEL_TOKENS: u32 = 50;

/// Token refill rate for mod channel (tokens per second)
pub const MOD_CHANNEL_REFILL_PER_SEC: u32 = 10;

/// Maximum expensive messages per second (e.g., block edits)
pub const EXPENSIVE_MSG_PER_SEC: u32 = 10;

// =============================================================================
// Parser Limits
// =============================================================================

/// Maximum registry index.json size (5 MB)
pub const MAX_INDEX_JSON_BYTES: usize = 5_242_880;

/// Maximum mod.toml manifest size (256 KB)
pub const MAX_MOD_TOML_BYTES: usize = 262_144;

/// Maximum files in a mod bundle zip
pub const MAX_ZIP_FILES: usize = 10_000;

/// Maximum decompression ratio (zip bomb protection)
pub const MAX_ZIP_RATIO: f64 = 20.0;

/// Maximum mod bundle size (50 MB)
pub const MAX_BUNDLE_BYTES: usize = 52_428_800;

// =============================================================================
// Strike Limits
// =============================================================================

/// Number of strikes before automatic disconnect
pub const STRIKES_BEFORE_DISCONNECT: u32 = 5;

/// Time window for strike decay (seconds)
pub const STRIKES_DECAY_SECS: u32 = 60;

/// Cooldown after disconnect before reconnection allowed (seconds)
pub const BAN_COOLDOWN_SECS: u32 = 300;

// =============================================================================
// Security Errors
// =============================================================================

/// Typed security errors with context
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecurityError {
    /// Message exceeds maximum size
    MessageTooLarge {
        actual: usize,
        limit: usize,
    },

    /// String field exceeds maximum length
    StringTooLong {
        field: &'static str,
        actual: usize,
        limit: usize,
    },

    /// List field exceeds maximum length
    ListTooLong {
        field: &'static str,
        actual: usize,
        limit: usize,
    },

    /// Numeric value out of bounds
    ValueOutOfBounds {
        field: &'static str,
        value: i64,
        min: i64,
        max: i64,
    },

    /// Decode failed due to invalid format
    DecodeInvalid {
        reason: String,
    },

    /// Rate limit exceeded
    RateLimitExceeded {
        limit_type: &'static str,
    },

    /// Handshake violation
    HandshakeViolation {
        reason: &'static str,
    },

    /// Payload sync violation
    PayloadSyncViolation {
        reason: &'static str,
    },

    /// Parser abuse detected
    ParserAbuse {
        reason: &'static str,
    },

    /// Path traversal attempt
    PathTraversal,

    /// Zip bomb detected
    ZipBomb {
        ratio: f64,
    },

    /// Too many files in archive
    TooManyFiles {
        count: usize,
    },
}

impl fmt::Display for SecurityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SecurityError::MessageTooLarge { actual, limit } => {
                write!(f, "message too large: {} bytes (limit: {})", actual, limit)
            }
            SecurityError::StringTooLong { field, actual, limit } => {
                write!(f, "string '{}' too long: {} bytes (limit: {})", field, actual, limit)
            }
            SecurityError::ListTooLong { field, actual, limit } => {
                write!(f, "list '{}' too long: {} elements (limit: {})", field, actual, limit)
            }
            SecurityError::ValueOutOfBounds { field, value, min, max } => {
                write!(f, "value '{}' out of bounds: {} (range: {}..{})", field, value, min, max)
            }
            SecurityError::DecodeInvalid { reason } => {
                write!(f, "decode error: {}", reason)
            }
            SecurityError::RateLimitExceeded { limit_type } => {
                write!(f, "rate limit exceeded: {}", limit_type)
            }
            SecurityError::HandshakeViolation { reason } => {
                write!(f, "handshake violation: {}", reason)
            }
            SecurityError::PayloadSyncViolation { reason } => {
                write!(f, "payload sync violation: {}", reason)
            }
            SecurityError::ParserAbuse { reason } => {
                write!(f, "parser abuse: {}", reason)
            }
            SecurityError::PathTraversal => {
                write!(f, "path traversal attempt detected")
            }
            SecurityError::ZipBomb { ratio } => {
                write!(f, "zip bomb detected: compression ratio {:.1}:1", ratio)
            }
            SecurityError::TooManyFiles { count } => {
                write!(f, "too many files in archive: {}", count)
            }
        }
    }
}

impl std::error::Error for SecurityError {}

// =============================================================================
// Helper Functions
// =============================================================================

/// Check if a byte slice exceeds the maximum length.
///
/// Returns `Ok(())` if within limits, `Err(SecurityError::MessageTooLarge)` otherwise.
#[inline]
pub fn check_len(data: &[u8], limit: usize) -> Result<(), SecurityError> {
    if data.len() > limit {
        Err(SecurityError::MessageTooLarge {
            actual: data.len(),
            limit,
        })
    } else {
        Ok(())
    }
}

/// Check if a string exceeds the maximum length.
///
/// Returns `Ok(())` if within limits, `Err(SecurityError::StringTooLong)` otherwise.
#[inline]
pub fn check_str_len(s: &str, field: &'static str, limit: usize) -> Result<(), SecurityError> {
    if s.len() > limit {
        Err(SecurityError::StringTooLong {
            field,
            actual: s.len(),
            limit,
        })
    } else {
        Ok(())
    }
}

/// Check if a list exceeds the maximum length.
///
/// Returns `Ok(())` if within limits, `Err(SecurityError::ListTooLong)` otherwise.
#[inline]
pub fn check_list_len<T>(list: &[T], field: &'static str, limit: usize) -> Result<(), SecurityError> {
    if list.len() > limit {
        Err(SecurityError::ListTooLong {
            field,
            actual: list.len(),
            limit,
        })
    } else {
        Ok(())
    }
}

/// Check if a numeric value is within bounds.
///
/// Returns `Ok(())` if within bounds, `Err(SecurityError::ValueOutOfBounds)` otherwise.
#[inline]
pub fn check_bounds(value: i64, field: &'static str, min: i64, max: i64) -> Result<(), SecurityError> {
    if value < min || value > max {
        Err(SecurityError::ValueOutOfBounds {
            field,
            value,
            min,
            max,
        })
    } else {
        Ok(())
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_len_within_limit() {
        let data = vec![0u8; 100];
        assert!(check_len(&data, 1000).is_ok());
    }

    #[test]
    fn test_check_len_at_limit() {
        let data = vec![0u8; 1000];
        assert!(check_len(&data, 1000).is_ok());
    }

    #[test]
    fn test_check_len_exceeds_limit() {
        let data = vec![0u8; 1001];
        let result = check_len(&data, 1000);
        assert!(matches!(
            result,
            Err(SecurityError::MessageTooLarge { actual: 1001, limit: 1000 })
        ));
    }

    #[test]
    fn test_check_str_len_within_limit() {
        assert!(check_str_len("hello", "name", MAX_STRING_BYTES).is_ok());
    }

    #[test]
    fn test_check_str_len_exceeds_limit() {
        let long_string = "x".repeat(MAX_STRING_BYTES + 1);
        let result = check_str_len(&long_string, "name", MAX_STRING_BYTES);
        assert!(matches!(
            result,
            Err(SecurityError::StringTooLong { field: "name", .. })
        ));
    }

    #[test]
    fn test_check_list_len_within_limit() {
        let list = vec![1, 2, 3];
        assert!(check_list_len(&list, "items", MAX_LIST_LEN).is_ok());
    }

    #[test]
    fn test_check_list_len_exceeds_limit() {
        let list: Vec<i32> = (0..MAX_LIST_LEN + 1).map(|i| i as i32).collect();
        let result = check_list_len(&list, "items", MAX_LIST_LEN);
        assert!(matches!(
            result,
            Err(SecurityError::ListTooLong { field: "items", .. })
        ));
    }

    #[test]
    fn test_check_bounds_within() {
        assert!(check_bounds(50, "index", 0, 100).is_ok());
    }

    #[test]
    fn test_check_bounds_at_min() {
        assert!(check_bounds(0, "index", 0, 100).is_ok());
    }

    #[test]
    fn test_check_bounds_at_max() {
        assert!(check_bounds(100, "index", 0, 100).is_ok());
    }

    #[test]
    fn test_check_bounds_below_min() {
        let result = check_bounds(-1, "index", 0, 100);
        assert!(matches!(
            result,
            Err(SecurityError::ValueOutOfBounds { field: "index", value: -1, min: 0, max: 100 })
        ));
    }

    #[test]
    fn test_check_bounds_above_max() {
        let result = check_bounds(101, "index", 0, 100);
        assert!(matches!(
            result,
            Err(SecurityError::ValueOutOfBounds { field: "index", value: 101, min: 0, max: 100 })
        ));
    }

    #[test]
    fn test_security_error_display() {
        let err = SecurityError::MessageTooLarge { actual: 70000, limit: 65536 };
        assert!(err.to_string().contains("70000"));
        assert!(err.to_string().contains("65536"));
    }

    #[test]
    fn test_limits_values() {
        // Verify limit values match spec
        assert_eq!(MAX_PACKET_BYTES, 65536);
        assert_eq!(MAX_STRING_BYTES, 1024);
        assert_eq!(MAX_LIST_LEN, 256);
        assert_eq!(HANDSHAKE_TIMEOUT_SECS, 10);
        assert_eq!(TRANSFER_TIMEOUT_SECS, 30);
        assert_eq!(GLOBAL_MSG_PER_SEC, 200);
        assert_eq!(STRIKES_BEFORE_DISCONNECT, 5);
        assert_eq!(MAX_ZIP_RATIO, 20.0);
    }
}
