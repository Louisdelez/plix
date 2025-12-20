//! Logging host function
//!
//! Implements plix_log for mod-to-server logging.
//!
//! The logging system provides:
//! - Level-based logging (ERROR, WARN, INFO, DEBUG, TRACE)
//! - Message truncation (4KB max)
//! - Invalid UTF-8 handling (lossy conversion)
//! - mod_id tagging for all log messages
//!
//! Note: The actual plix_log function is implemented in host/mod.rs
//! as it needs direct access to the Caller context.

/// Maximum log message length in bytes
pub const MAX_LOG_MESSAGE_LEN: usize = 4096;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_log_message_len() {
        assert_eq!(MAX_LOG_MESSAGE_LEN, 4096);
    }
}
