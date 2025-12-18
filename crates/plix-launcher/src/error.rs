//! Error types for the launcher

use thiserror::Error;

/// Launcher error types
#[derive(Error, Debug)]
pub enum LauncherError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Network error
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// Manifest parsing error
    #[error("Manifest error: {0}")]
    Manifest(String),

    /// Checksum verification failed
    #[error("Checksum verification failed for {path}: expected {expected}, got {actual}")]
    ChecksumMismatch {
        path: String,
        expected: String,
        actual: String,
    },

    /// File system error
    #[error("File system error: {0}")]
    Io(#[from] std::io::Error),

    /// Version parsing error
    #[error("Version error: {0}")]
    Version(String),

    /// Installation error
    #[error("Installation error: {0}")]
    Install(String),

    /// Launch error
    #[error("Launch error: {0}")]
    Launch(String),

    /// TOML parsing error
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    /// Already running
    #[error("Another instance is already running")]
    AlreadyRunning,

    /// Cannot proceed (offline with no local version)
    #[error("Cannot proceed: {0}")]
    CannotProceed(String),
}

/// Exit codes for the launcher
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    /// Success
    Success = 0,
    /// General error
    GeneralError = 1,
    /// Network error (could not fetch manifest)
    NetworkError = 2,
    /// Checksum verification failed
    ChecksumError = 3,
    /// Installation failed
    InstallError = 4,
    /// Launch failed
    LaunchError = 5,
    /// Already running (single instance violation)
    AlreadyRunning = 10,
}

impl From<&LauncherError> for ExitCode {
    fn from(err: &LauncherError) -> Self {
        match err {
            LauncherError::Network(_) => ExitCode::NetworkError,
            LauncherError::ChecksumMismatch { .. } => ExitCode::ChecksumError,
            LauncherError::Install(_) => ExitCode::InstallError,
            LauncherError::Launch(_) => ExitCode::LaunchError,
            LauncherError::AlreadyRunning => ExitCode::AlreadyRunning,
            _ => ExitCode::GeneralError,
        }
    }
}
