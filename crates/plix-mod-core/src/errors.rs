//! Error types and codes for the Mod API
//!
//! All API calls return `Result<T, ModApiError>` - no panics allowed.

use std::fmt;

/// Error codes for mod API operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    /// EMOD001: Invalid parameter value
    InvalidArgument,
    /// EMOD002: Missing required capability
    PermissionDenied,
    /// EMOD003: Entity/timer/resource not found
    NotFound,
    /// EMOD004: Position/value outside valid range
    OutOfBounds,
    /// EMOD005: Rate limit or quota exceeded
    RateLimited,
    /// EMOD006: Chunk not loaded
    WorldNotReady,
    /// EMOD007: API version mismatch
    Unsupported,
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
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code_str())
    }
}

/// Context for a mod API error
#[derive(Debug, Clone, Default)]
pub struct ErrorContext {
    /// ID of the mod that triggered the error
    pub mod_id: Option<String>,
    /// Name of the API function that failed
    pub api_call: Option<String>,
    /// Summary of parameters passed
    pub params: Option<String>,
}

/// Error type returned by all mod API calls
#[derive(Debug, Clone)]
pub struct ModApiError {
    /// Machine-readable error code
    pub code: ErrorCode,
    /// Human-readable error message
    pub message: String,
    /// Additional context for debugging
    pub context: ErrorContext,
}

impl ModApiError {
    /// Create a new error with the given code and message
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            context: ErrorContext::default(),
        }
    }

    /// Add mod ID to the error context
    pub fn with_mod_id(mut self, mod_id: impl Into<String>) -> Self {
        self.context.mod_id = Some(mod_id.into());
        self
    }

    /// Add API call name to the error context
    pub fn with_api_call(mut self, api_call: impl Into<String>) -> Self {
        self.context.api_call = Some(api_call.into());
        self
    }

    /// Add parameter summary to the error context
    pub fn with_params(mut self, params: impl Into<String>) -> Self {
        self.context.params = Some(params.into());
        self
    }
}

impl fmt::Display for ModApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)?;
        if let Some(ref mod_id) = self.context.mod_id {
            write!(f, " (mod: {})", mod_id)?;
        }
        if let Some(ref api_call) = self.context.api_call {
            write!(f, " (call: {})", api_call)?;
        }
        Ok(())
    }
}

impl std::error::Error for ModApiError {}

// Helper functions for creating common errors

/// Create an InvalidArgument error (EMOD001)
pub fn err_invalid(message: impl Into<String>) -> ModApiError {
    ModApiError::new(ErrorCode::InvalidArgument, message)
}

/// Create a PermissionDenied error (EMOD002)
pub fn err_perm(capability: &str) -> ModApiError {
    ModApiError::new(
        ErrorCode::PermissionDenied,
        format!("Missing required capability: {}", capability),
    )
}

/// Create a NotFound error (EMOD003)
pub fn err_not_found(resource: impl Into<String>) -> ModApiError {
    let resource = resource.into();
    ModApiError::new(ErrorCode::NotFound, format!("{} not found", resource))
}

/// Create an OutOfBounds error (EMOD004)
pub fn err_bounds(message: impl Into<String>) -> ModApiError {
    ModApiError::new(ErrorCode::OutOfBounds, message)
}

/// Create a RateLimited error (EMOD005)
pub fn err_rate(message: impl Into<String>) -> ModApiError {
    ModApiError::new(ErrorCode::RateLimited, message)
}

/// Create a WorldNotReady error (EMOD006)
pub fn err_world_not_ready(message: impl Into<String>) -> ModApiError {
    ModApiError::new(ErrorCode::WorldNotReady, message)
}

/// Create an Unsupported error (EMOD007)
pub fn err_unsupported(message: impl Into<String>) -> ModApiError {
    ModApiError::new(ErrorCode::Unsupported, message)
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
    fn test_error_display() {
        let err = ModApiError::new(ErrorCode::InvalidArgument, "test message")
            .with_mod_id("my-mod")
            .with_api_call("get_block");
        let display = format!("{}", err);
        assert!(display.contains("EMOD001"));
        assert!(display.contains("test message"));
        assert!(display.contains("my-mod"));
        assert!(display.contains("get_block"));
    }

    #[test]
    fn test_error_helpers() {
        let err = err_invalid("bad value");
        assert_eq!(err.code, ErrorCode::InvalidArgument);

        let err = err_perm("world.read");
        assert_eq!(err.code, ErrorCode::PermissionDenied);
        assert!(err.message.contains("world.read"));

        let err = err_not_found("entity");
        assert_eq!(err.code, ErrorCode::NotFound);

        let err = err_bounds("position out of range");
        assert_eq!(err.code, ErrorCode::OutOfBounds);

        let err = err_rate("too many messages");
        assert_eq!(err.code, ErrorCode::RateLimited);

        let err = err_world_not_ready("chunk not loaded");
        assert_eq!(err.code, ErrorCode::WorldNotReady);

        let err = err_unsupported("API version mismatch");
        assert_eq!(err.code, ErrorCode::Unsupported);
    }

    #[test]
    fn test_error_context_builder() {
        let err = err_invalid("test")
            .with_mod_id("test-mod")
            .with_api_call("set_block")
            .with_params("pos=(1,2,3)");

        assert_eq!(err.context.mod_id.as_deref(), Some("test-mod"));
        assert_eq!(err.context.api_call.as_deref(), Some("set_block"));
        assert_eq!(err.context.params.as_deref(), Some("pos=(1,2,3)"));
    }
}
