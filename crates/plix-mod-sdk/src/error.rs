//! Error types and EMOD error code mapping
//!
//! All SDK functions return `Result<T, ModError>`. The error codes match
//! those defined in the Plix mod runtime (EMOD001-EMOD007).

use alloc::string::String;
use core::fmt;

/// Error codes matching the Plix mod runtime
///
/// These codes are returned by host function calls and can be used
/// for programmatic error handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ErrorCode {
    /// EMOD001: Invalid parameter value
    InvalidArgument = 1,
    /// EMOD002: Missing required capability
    PermissionDenied = 2,
    /// EMOD003: Entity/timer/resource not found
    NotFound = 3,
    /// EMOD004: Position/value outside valid range
    OutOfBounds = 4,
    /// EMOD005: Rate limit or quota exceeded
    RateLimited = 5,
    /// EMOD006: Chunk not loaded
    WorldNotReady = 6,
    /// EMOD007: API version mismatch
    Unsupported = 7,
}

impl ErrorCode {
    /// Returns the error code string (e.g., "EMOD001")
    pub fn code_str(&self) -> &'static str {
        match self {
            ErrorCode::InvalidArgument => "EMOD001",
            ErrorCode::PermissionDenied => "EMOD002",
            ErrorCode::NotFound => "EMOD003",
            ErrorCode::OutOfBounds => "EMOD004",
            ErrorCode::RateLimited => "EMOD005",
            ErrorCode::WorldNotReady => "EMOD006",
            ErrorCode::Unsupported => "EMOD007",
        }
    }

    /// Parse error code from u16 (as returned by ABI)
    pub fn from_u16(code: u16) -> Option<Self> {
        match code {
            1 => Some(ErrorCode::InvalidArgument),
            2 => Some(ErrorCode::PermissionDenied),
            3 => Some(ErrorCode::NotFound),
            4 => Some(ErrorCode::OutOfBounds),
            5 => Some(ErrorCode::RateLimited),
            6 => Some(ErrorCode::WorldNotReady),
            7 => Some(ErrorCode::Unsupported),
            _ => None,
        }
    }

    /// Returns a human-readable description of the error code
    pub fn description(&self) -> &'static str {
        match self {
            ErrorCode::InvalidArgument => "Invalid parameter value",
            ErrorCode::PermissionDenied => "Missing required capability",
            ErrorCode::NotFound => "Resource not found",
            ErrorCode::OutOfBounds => "Position or value out of bounds",
            ErrorCode::RateLimited => "Rate limit exceeded",
            ErrorCode::WorldNotReady => "Chunk not loaded",
            ErrorCode::Unsupported => "API version mismatch or unsupported operation",
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code_str())
    }
}

/// Error type returned by all SDK functions
#[derive(Debug, Clone)]
pub struct ModError {
    /// Machine-readable error code
    pub code: ErrorCode,
    /// Human-readable error message
    pub message: String,
}

impl ModError {
    /// Create a new error with the given code and message
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    /// Create an error from an ABI response
    pub fn from_abi(error_code: u16, message: Option<String>) -> Self {
        let code = ErrorCode::from_u16(error_code).unwrap_or(ErrorCode::Unsupported);
        let message = message.unwrap_or_else(|| code.description().into());
        Self { code, message }
    }

    /// Check if this is a permission error
    pub fn is_permission_denied(&self) -> bool {
        self.code == ErrorCode::PermissionDenied
    }

    /// Check if this is a rate limit error
    pub fn is_rate_limited(&self) -> bool {
        self.code == ErrorCode::RateLimited
    }

    /// Check if this is a world not ready error
    pub fn is_world_not_ready(&self) -> bool {
        self.code == ErrorCode::WorldNotReady
    }
}

impl fmt::Display for ModError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

// Note: We can't implement std::error::Error in no_std, but we provide Display

/// Result type alias for SDK functions
pub type Result<T> = core::result::Result<T, ModError>;

// Helper functions for creating common errors

/// Create an InvalidArgument error (EMOD001)
pub fn err_invalid(message: impl Into<String>) -> ModError {
    ModError::new(ErrorCode::InvalidArgument, message)
}

/// Create a PermissionDenied error (EMOD002)
pub fn err_perm(capability: &str) -> ModError {
    ModError::new(
        ErrorCode::PermissionDenied,
        alloc::format!("Missing required capability: {}", capability),
    )
}

/// Create a NotFound error (EMOD003)
pub fn err_not_found(resource: impl Into<String>) -> ModError {
    let resource = resource.into();
    ModError::new(ErrorCode::NotFound, alloc::format!("{} not found", resource))
}

/// Create an OutOfBounds error (EMOD004)
pub fn err_bounds(message: impl Into<String>) -> ModError {
    ModError::new(ErrorCode::OutOfBounds, message)
}

/// Create a RateLimited error (EMOD005)
pub fn err_rate(message: impl Into<String>) -> ModError {
    ModError::new(ErrorCode::RateLimited, message)
}

/// Create a WorldNotReady error (EMOD006)
pub fn err_world_not_ready(message: impl Into<String>) -> ModError {
    ModError::new(ErrorCode::WorldNotReady, message)
}

/// Create an Unsupported error (EMOD007)
pub fn err_unsupported(message: impl Into<String>) -> ModError {
    ModError::new(ErrorCode::Unsupported, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_strings() {
        assert_eq!(ErrorCode::InvalidArgument.code_str(), "EMOD001");
        assert_eq!(ErrorCode::PermissionDenied.code_str(), "EMOD002");
        assert_eq!(ErrorCode::NotFound.code_str(), "EMOD003");
        assert_eq!(ErrorCode::OutOfBounds.code_str(), "EMOD004");
        assert_eq!(ErrorCode::RateLimited.code_str(), "EMOD005");
        assert_eq!(ErrorCode::WorldNotReady.code_str(), "EMOD006");
        assert_eq!(ErrorCode::Unsupported.code_str(), "EMOD007");
    }

    #[test]
    fn test_error_code_from_u16_all_codes() {
        // Test all valid EMOD codes 1-7
        assert_eq!(ErrorCode::from_u16(1), Some(ErrorCode::InvalidArgument));
        assert_eq!(ErrorCode::from_u16(2), Some(ErrorCode::PermissionDenied));
        assert_eq!(ErrorCode::from_u16(3), Some(ErrorCode::NotFound));
        assert_eq!(ErrorCode::from_u16(4), Some(ErrorCode::OutOfBounds));
        assert_eq!(ErrorCode::from_u16(5), Some(ErrorCode::RateLimited));
        assert_eq!(ErrorCode::from_u16(6), Some(ErrorCode::WorldNotReady));
        assert_eq!(ErrorCode::from_u16(7), Some(ErrorCode::Unsupported));

        // Test invalid codes
        assert_eq!(ErrorCode::from_u16(0), None);
        assert_eq!(ErrorCode::from_u16(8), None);
        assert_eq!(ErrorCode::from_u16(99), None);
        assert_eq!(ErrorCode::from_u16(u16::MAX), None);
    }

    #[test]
    fn test_error_code_repr_values() {
        // Ensure repr(u8) values match expected EMOD numbers
        assert_eq!(ErrorCode::InvalidArgument as u8, 1);
        assert_eq!(ErrorCode::PermissionDenied as u8, 2);
        assert_eq!(ErrorCode::NotFound as u8, 3);
        assert_eq!(ErrorCode::OutOfBounds as u8, 4);
        assert_eq!(ErrorCode::RateLimited as u8, 5);
        assert_eq!(ErrorCode::WorldNotReady as u8, 6);
        assert_eq!(ErrorCode::Unsupported as u8, 7);
    }

    #[test]
    fn test_mod_error_display() {
        let err = ModError::new(ErrorCode::InvalidArgument, "bad value");
        let display = alloc::format!("{}", err);
        assert!(display.contains("EMOD001"));
        assert!(display.contains("bad value"));
    }

    #[test]
    fn test_mod_error_from_abi_with_message() {
        let err = ModError::from_abi(2, Some("custom message".into()));
        assert_eq!(err.code, ErrorCode::PermissionDenied);
        assert_eq!(err.message, "custom message");
    }

    #[test]
    fn test_mod_error_from_abi_without_message() {
        let err = ModError::from_abi(3, None);
        assert_eq!(err.code, ErrorCode::NotFound);
        // Should use default description
        assert_eq!(err.message, "Resource not found");
    }

    #[test]
    fn test_mod_error_from_abi_unknown_code() {
        let err = ModError::from_abi(99, None);
        // Unknown codes should map to Unsupported
        assert_eq!(err.code, ErrorCode::Unsupported);
    }

    #[test]
    fn test_error_helpers_all() {
        let err = err_invalid("bad value");
        assert_eq!(err.code, ErrorCode::InvalidArgument);

        let err = err_perm("world.read");
        assert_eq!(err.code, ErrorCode::PermissionDenied);
        assert!(err.message.contains("world.read"));

        let err = err_not_found("entity");
        assert_eq!(err.code, ErrorCode::NotFound);

        let err = err_bounds("y > 256");
        assert_eq!(err.code, ErrorCode::OutOfBounds);

        let err = err_rate("100 calls/tick exceeded");
        assert_eq!(err.code, ErrorCode::RateLimited);

        let err = err_world_not_ready("chunk not loaded");
        assert_eq!(err.code, ErrorCode::WorldNotReady);

        let err = err_unsupported("API v2 required");
        assert_eq!(err.code, ErrorCode::Unsupported);
    }

    #[test]
    fn test_error_is_methods() {
        let perm_err = err_perm("test");
        assert!(perm_err.is_permission_denied());
        assert!(!perm_err.is_rate_limited());
        assert!(!perm_err.is_world_not_ready());

        let rate_err = err_rate("limit");
        assert!(rate_err.is_rate_limited());
        assert!(!rate_err.is_permission_denied());

        let world_err = err_world_not_ready("chunk");
        assert!(world_err.is_world_not_ready());
        assert!(!world_err.is_rate_limited());
    }

    #[test]
    fn test_error_code_descriptions() {
        // Verify all codes have non-empty descriptions
        assert!(!ErrorCode::InvalidArgument.description().is_empty());
        assert!(!ErrorCode::PermissionDenied.description().is_empty());
        assert!(!ErrorCode::NotFound.description().is_empty());
        assert!(!ErrorCode::OutOfBounds.description().is_empty());
        assert!(!ErrorCode::RateLimited.description().is_empty());
        assert!(!ErrorCode::WorldNotReady.description().is_empty());
        assert!(!ErrorCode::Unsupported.description().is_empty());
    }
}
