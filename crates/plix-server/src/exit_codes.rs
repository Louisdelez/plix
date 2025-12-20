//! Server Exit Codes
//!
//! Standardized exit codes for the headless server binary.
//! Follows Unix conventions plus application-specific codes 64-78.

/// Exit codes for the plix-server binary.
///
/// These codes are used by systemd, Docker, and CI scripts to determine
/// the reason for server termination.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ExitCode {
    /// Clean shutdown (exit 0)
    Success = 0,
    /// Unspecified runtime error (exit 1)
    GeneralError = 1,
    /// Invalid CLI arguments (exit 2)
    Misuse = 2,
    /// Port already in use or permission denied (exit 64)
    BindFailed = 64,
    /// Missing arena or asset files (exit 65)
    AssetLoadFailed = 65,
    /// World save/load failure (exit 66)
    PersistenceError = 66,
    /// Socket creation or network failure (exit 67)
    NetworkError = 67,
    /// Forced exit after shutdown timeout (exit 68)
    ShutdownTimeout = 68,
    /// Migration required but not requested (exit 69)
    MigrationRequired = 69,
    /// Migration failed (exit 70)
    MigrationFailed = 70,
}

impl ExitCode {
    /// Exit the process with this exit code.
    ///
    /// This function never returns.
    pub fn exit(self) -> ! {
        std::process::exit(self as i32)
    }

    /// Get the numeric value of this exit code.
    pub fn code(self) -> i32 {
        self as i32
    }

    /// Get a human-readable description of this exit code.
    pub fn description(self) -> &'static str {
        match self {
            ExitCode::Success => "Clean shutdown",
            ExitCode::GeneralError => "Unspecified runtime error",
            ExitCode::Misuse => "Invalid CLI arguments",
            ExitCode::BindFailed => "Port already in use or permission denied",
            ExitCode::AssetLoadFailed => "Missing arena or asset files",
            ExitCode::PersistenceError => "World save/load failure",
            ExitCode::NetworkError => "Socket creation or network failure",
            ExitCode::ShutdownTimeout => "Forced exit after shutdown timeout",
            ExitCode::MigrationRequired => "Data migration required, run with --migrate",
            ExitCode::MigrationFailed => "Data migration failed",
        }
    }

    /// Convert from an i32 exit code.
    ///
    /// Returns None for unknown codes.
    pub fn from_code(code: i32) -> Option<Self> {
        match code {
            0 => Some(ExitCode::Success),
            1 => Some(ExitCode::GeneralError),
            2 => Some(ExitCode::Misuse),
            64 => Some(ExitCode::BindFailed),
            65 => Some(ExitCode::AssetLoadFailed),
            66 => Some(ExitCode::PersistenceError),
            67 => Some(ExitCode::NetworkError),
            68 => Some(ExitCode::ShutdownTimeout),
            69 => Some(ExitCode::MigrationRequired),
            70 => Some(ExitCode::MigrationFailed),
            _ => None,
        }
    }
}

impl std::fmt::Display for ExitCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (exit {})", self.description(), self.code())
    }
}

impl From<ExitCode> for i32 {
    fn from(code: ExitCode) -> Self {
        code as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_code_values() {
        assert_eq!(ExitCode::Success.code(), 0);
        assert_eq!(ExitCode::GeneralError.code(), 1);
        assert_eq!(ExitCode::Misuse.code(), 2);
        assert_eq!(ExitCode::BindFailed.code(), 64);
        assert_eq!(ExitCode::AssetLoadFailed.code(), 65);
        assert_eq!(ExitCode::PersistenceError.code(), 66);
        assert_eq!(ExitCode::NetworkError.code(), 67);
        assert_eq!(ExitCode::ShutdownTimeout.code(), 68);
        assert_eq!(ExitCode::MigrationRequired.code(), 69);
        assert_eq!(ExitCode::MigrationFailed.code(), 70);
    }

    #[test]
    fn test_from_code() {
        assert_eq!(ExitCode::from_code(0), Some(ExitCode::Success));
        assert_eq!(ExitCode::from_code(64), Some(ExitCode::BindFailed));
        assert_eq!(ExitCode::from_code(999), None);
    }

    #[test]
    fn test_display() {
        let display = format!("{}", ExitCode::BindFailed);
        assert!(display.contains("Port already in use"));
        assert!(display.contains("64"));
    }

    #[test]
    fn test_into_i32() {
        let code: i32 = ExitCode::Success.into();
        assert_eq!(code, 0);
    }
}
