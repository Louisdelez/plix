//! Logging API for mods
//!
//! Provides ergonomic logging macros that send messages to the server log.
//! Log messages are prefixed with the mod ID automatically.
//!
//! # Example
//!
//! ```rust,ignore
//! use plix_mod_sdk::log::{info, warn, error, debug};
//!
//! info("Mod initialized");
//! warn("Player count high");
//! error("Failed to load config");
//! debug("Processing tick 42");
//! ```
//!
//! # Constraints
//!
//! - Maximum message length: 4096 bytes (truncated if exceeded)
//! - Messages are UTF-8 encoded

use crate::abi;
use alloc::string::String;

/// Maximum log message length in bytes
pub const MAX_LOG_MESSAGE_LEN: usize = 4096;

/// Log levels matching the runtime
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum LogLevel {
    /// Error level - critical issues
    Error = 0,
    /// Warning level - potential issues
    Warn = 1,
    /// Info level - normal operation
    Info = 2,
    /// Debug level - diagnostic info
    Debug = 3,
    /// Trace level - verbose diagnostic info
    Trace = 4,
}

/// Log a message at the given level
///
/// Messages longer than 4096 bytes are truncated.
pub fn log(level: LogLevel, message: &str) {
    // Truncate if too long
    let msg = if message.len() > MAX_LOG_MESSAGE_LEN {
        &message[..MAX_LOG_MESSAGE_LEN]
    } else {
        message
    };

    abi::log(level as i32, msg);
}

/// Log a message at error level
///
/// # Example
///
/// ```rust,ignore
/// use plix_mod_sdk::log::error;
/// error("Failed to process event");
/// ```
pub fn error(message: &str) {
    log(LogLevel::Error, message);
}

/// Log a message at warning level
///
/// # Example
///
/// ```rust,ignore
/// use plix_mod_sdk::log::warn;
/// warn("Config value missing, using default");
/// ```
pub fn warn(message: &str) {
    log(LogLevel::Warn, message);
}

/// Log a message at info level
///
/// # Example
///
/// ```rust,ignore
/// use plix_mod_sdk::log::info;
/// info("Mod initialized successfully");
/// ```
pub fn info(message: &str) {
    log(LogLevel::Info, message);
}

/// Log a message at debug level
///
/// # Example
///
/// ```rust,ignore
/// use plix_mod_sdk::log::debug;
/// debug("Processing player join event");
/// ```
pub fn debug(message: &str) {
    log(LogLevel::Debug, message);
}

/// Log a message at trace level
///
/// # Example
///
/// ```rust,ignore
/// use plix_mod_sdk::log::trace;
/// trace("Entering function process_tick");
/// ```
pub fn trace(message: &str) {
    log(LogLevel::Trace, message);
}

/// Log a formatted message at error level
pub fn error_fmt(args: core::fmt::Arguments<'_>) {
    let msg = alloc::format!("{}", args);
    error(&msg);
}

/// Log a formatted message at warning level
pub fn warn_fmt(args: core::fmt::Arguments<'_>) {
    let msg = alloc::format!("{}", args);
    warn(&msg);
}

/// Log a formatted message at info level
pub fn info_fmt(args: core::fmt::Arguments<'_>) {
    let msg = alloc::format!("{}", args);
    info(&msg);
}

/// Log a formatted message at debug level
pub fn debug_fmt(args: core::fmt::Arguments<'_>) {
    let msg = alloc::format!("{}", args);
    debug(&msg);
}

// Note: Macros are tricky in no_std. We provide functions for now.
// The proc-macro crate can provide nicer macros later.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_values() {
        assert_eq!(LogLevel::Error as i32, 0);
        assert_eq!(LogLevel::Warn as i32, 1);
        assert_eq!(LogLevel::Info as i32, 2);
        assert_eq!(LogLevel::Debug as i32, 3);
        assert_eq!(LogLevel::Trace as i32, 4);
    }
}
