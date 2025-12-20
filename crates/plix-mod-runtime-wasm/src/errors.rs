//! Runtime-specific error types for the WASM mod runtime
//!
//! These errors are internal to the runtime and are mapped to ModApiError
//! codes (EMOD001-007) when returned to mods.

use plix_mod_core::errors::{ErrorCode, ModApiError};
use thiserror::Error;

/// Errors that can occur during WASM mod execution
#[derive(Debug, Error)]
pub enum RuntimeError {
    /// Invalid WASM binary format
    #[error("Invalid WASM binary: {0}")]
    WasmInvalid(String),

    /// WASM execution trap (div by zero, OOB, etc.)
    #[error("WASM trap: {0}")]
    Trap(String),

    /// Fuel exhausted (CPU budget exceeded)
    #[error("Fuel exhausted: mod exceeded CPU budget")]
    FuelExhausted,

    /// Memory limit exceeded
    #[error("Memory limit exceeded: mod tried to allocate beyond {limit} bytes")]
    MemoryLimit { limit: usize },

    /// Missing required export from WASM module
    #[error("Missing required export: {0}")]
    MissingExport(String),

    /// Invalid export signature
    #[error("Invalid export signature for {name}: expected {expected}, got {actual}")]
    InvalidExportSignature {
        name: String,
        expected: String,
        actual: String,
    },

    /// Unsupported import (e.g., WASI)
    #[error("Unsupported import: {namespace}::{name}")]
    UnsupportedImport { namespace: String, name: String },

    /// Memory access out of bounds
    #[error(
        "Memory access out of bounds: offset {offset}, length {length}, memory size {memory_size}"
    )]
    MemoryAccessOutOfBounds {
        offset: usize,
        length: usize,
        memory_size: usize,
    },

    /// Host function error
    #[error("Host function error: {0}")]
    HostFunction(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Mod not found
    #[error("Mod not found: {0}")]
    ModNotFound(String),

    /// Mod already loaded
    #[error("Mod already loaded: {0}")]
    ModAlreadyLoaded(String),

    /// Mod is disabled
    #[error("Mod is disabled: {mod_id} - {reason}")]
    ModDisabled { mod_id: String, reason: String },

    /// Internal runtime error
    #[error("Internal runtime error: {0}")]
    Internal(String),
}

impl RuntimeError {
    /// Convert to ModApiError with appropriate error code
    pub fn to_mod_api_error(&self) -> ModApiError {
        let (code, message) = match self {
            RuntimeError::WasmInvalid(msg) => (ErrorCode::InvalidArgument, msg.clone()),
            RuntimeError::Trap(msg) => (ErrorCode::OutOfBounds, msg.clone()),
            RuntimeError::FuelExhausted => {
                (ErrorCode::RateLimited, "CPU budget exceeded".to_string())
            }
            RuntimeError::MemoryLimit { limit } => (
                ErrorCode::RateLimited,
                format!("Memory limit exceeded: {} bytes", limit),
            ),
            RuntimeError::MissingExport(name) => (
                ErrorCode::Unsupported,
                format!("Missing required export: {}", name),
            ),
            RuntimeError::InvalidExportSignature {
                name,
                expected,
                actual,
            } => (
                ErrorCode::Unsupported,
                format!(
                    "Invalid signature for {}: expected {}, got {}",
                    name, expected, actual
                ),
            ),
            RuntimeError::UnsupportedImport { namespace, name } => (
                ErrorCode::Unsupported,
                format!("Unsupported import: {}::{}", namespace, name),
            ),
            RuntimeError::MemoryAccessOutOfBounds {
                offset,
                length,
                memory_size,
            } => (
                ErrorCode::OutOfBounds,
                format!(
                    "Memory access [{}..{}] out of bounds (size {})",
                    offset,
                    offset + length,
                    memory_size
                ),
            ),
            RuntimeError::HostFunction(msg) => (ErrorCode::InvalidArgument, msg.clone()),
            RuntimeError::Serialization(msg) => (
                ErrorCode::InvalidArgument,
                format!("Serialization: {}", msg),
            ),
            RuntimeError::ModNotFound(id) => {
                (ErrorCode::NotFound, format!("Mod not found: {}", id))
            }
            RuntimeError::ModAlreadyLoaded(id) => (
                ErrorCode::InvalidArgument,
                format!("Mod already loaded: {}", id),
            ),
            RuntimeError::ModDisabled { mod_id, reason } => (
                ErrorCode::InvalidArgument,
                format!("Mod {} is disabled: {}", mod_id, reason),
            ),
            RuntimeError::Internal(msg) => {
                (ErrorCode::InvalidArgument, format!("Internal: {}", msg))
            }
        };

        ModApiError::new(code, message)
    }

    /// Get the EMOD error code for this runtime error
    pub fn error_code(&self) -> ErrorCode {
        match self {
            RuntimeError::WasmInvalid(_) => ErrorCode::InvalidArgument,
            RuntimeError::Trap(_) => ErrorCode::OutOfBounds,
            RuntimeError::FuelExhausted => ErrorCode::RateLimited,
            RuntimeError::MemoryLimit { .. } => ErrorCode::RateLimited,
            RuntimeError::MissingExport(_) => ErrorCode::Unsupported,
            RuntimeError::InvalidExportSignature { .. } => ErrorCode::Unsupported,
            RuntimeError::UnsupportedImport { .. } => ErrorCode::Unsupported,
            RuntimeError::MemoryAccessOutOfBounds { .. } => ErrorCode::OutOfBounds,
            RuntimeError::HostFunction(_) => ErrorCode::InvalidArgument,
            RuntimeError::Serialization(_) => ErrorCode::InvalidArgument,
            RuntimeError::ModNotFound(_) => ErrorCode::NotFound,
            RuntimeError::ModAlreadyLoaded(_) => ErrorCode::InvalidArgument,
            RuntimeError::ModDisabled { .. } => ErrorCode::InvalidArgument,
            RuntimeError::Internal(_) => ErrorCode::InvalidArgument,
        }
    }

    /// Returns true if this error should increment the mod's error counter
    pub fn is_mod_error(&self) -> bool {
        matches!(
            self,
            RuntimeError::Trap(_)
                | RuntimeError::FuelExhausted
                | RuntimeError::MemoryLimit { .. }
                | RuntimeError::MemoryAccessOutOfBounds { .. }
        )
    }
}

impl From<bincode::Error> for RuntimeError {
    fn from(err: bincode::Error) -> Self {
        RuntimeError::Serialization(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_to_mod_api_error() {
        let err = RuntimeError::FuelExhausted;
        let api_err = err.to_mod_api_error();
        assert_eq!(api_err.code, ErrorCode::RateLimited);
        assert!(api_err.message.contains("CPU budget"));
    }

    #[test]
    fn test_error_code_mapping() {
        assert_eq!(
            RuntimeError::WasmInvalid("test".into()).error_code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            RuntimeError::Trap("test".into()).error_code(),
            ErrorCode::OutOfBounds
        );
        assert_eq!(
            RuntimeError::FuelExhausted.error_code(),
            ErrorCode::RateLimited
        );
        assert_eq!(
            RuntimeError::MissingExport("mod_init".into()).error_code(),
            ErrorCode::Unsupported
        );
    }

    #[test]
    fn test_is_mod_error() {
        assert!(RuntimeError::Trap("test".into()).is_mod_error());
        assert!(RuntimeError::FuelExhausted.is_mod_error());
        assert!(RuntimeError::MemoryLimit { limit: 1000 }.is_mod_error());
        assert!(!RuntimeError::WasmInvalid("test".into()).is_mod_error());
        assert!(!RuntimeError::ModNotFound("test".into()).is_mod_error());
    }

    #[test]
    fn test_error_display() {
        let err = RuntimeError::MemoryAccessOutOfBounds {
            offset: 100,
            length: 50,
            memory_size: 200,
        };
        let display = format!("{}", err);
        assert!(display.contains("offset 100"));
        assert!(display.contains("length 50"));
        assert!(display.contains("out of bounds"));
    }
}
