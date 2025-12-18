//! Persistence error types.
//!
//! Defines all errors that can occur during world persistence operations.

use crate::chunk::ChunkCoord;
use std::fmt;

/// Result of version compatibility check.
#[derive(Debug, Clone, PartialEq)]
pub enum VersionCheck {
    /// Version matches current.
    Current,

    /// Version is older but supported (migration available).
    NeedsMigration(u32),

    /// Version is too old (no migration path).
    TooOld(u32),

    /// Version is newer than current (from future game version).
    TooNew(u32),
}

impl fmt::Display for VersionCheck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VersionCheck::Current => write!(f, "current version"),
            VersionCheck::NeedsMigration(v) => write!(f, "needs migration from v{}", v),
            VersionCheck::TooOld(v) => write!(f, "version {} is too old", v),
            VersionCheck::TooNew(v) => write!(f, "version {} is newer than supported", v),
        }
    }
}

/// Errors that can occur during persistence operations.
#[derive(Debug, Clone)]
pub enum PersistError {
    /// File system I/O error.
    Io(String),

    /// Serialization/deserialization error.
    Codec(String),

    /// World directory not found.
    WorldNotFound(String),

    /// Version incompatibility.
    VersionMismatch {
        /// The version found in the world/chunk.
        world_version: u32,
        /// The reason for the mismatch.
        reason: VersionCheck,
    },

    /// Chunk data corrupted or invalid.
    ChunkCorrupted {
        /// The chunk coordinate.
        coord: ChunkCoord,
        /// Description of the corruption.
        reason: String,
    },

    /// Metadata corrupted or invalid.
    MetadataCorrupted(String),

    /// Permission denied.
    PermissionDenied(String),

    /// Disk full.
    DiskFull,
}

impl fmt::Display for PersistError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PersistError::Io(msg) => write!(f, "I/O error: {}", msg),
            PersistError::Codec(msg) => write!(f, "codec error: {}", msg),
            PersistError::WorldNotFound(id) => write!(f, "world not found: {}", id),
            PersistError::VersionMismatch {
                world_version,
                reason,
            } => {
                write!(f, "version mismatch: world v{} - {}", world_version, reason)
            }
            PersistError::ChunkCorrupted { coord, reason } => {
                write!(f, "chunk {:?} corrupted: {}", coord, reason)
            }
            PersistError::MetadataCorrupted(msg) => write!(f, "metadata corrupted: {}", msg),
            PersistError::PermissionDenied(path) => write!(f, "permission denied: {}", path),
            PersistError::DiskFull => write!(f, "disk full"),
        }
    }
}

impl std::error::Error for PersistError {}

impl From<std::io::Error> for PersistError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => PersistError::WorldNotFound(err.to_string()),
            std::io::ErrorKind::PermissionDenied => PersistError::PermissionDenied(err.to_string()),
            std::io::ErrorKind::StorageFull => PersistError::DiskFull,
            _ => PersistError::Io(err.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_check_display() {
        assert_eq!(VersionCheck::Current.to_string(), "current version");
        assert_eq!(
            VersionCheck::NeedsMigration(1).to_string(),
            "needs migration from v1"
        );
        assert_eq!(VersionCheck::TooOld(0).to_string(), "version 0 is too old");
        assert_eq!(
            VersionCheck::TooNew(99).to_string(),
            "version 99 is newer than supported"
        );
    }

    #[test]
    fn test_persist_error_display() {
        let err = PersistError::Io("test".to_string());
        assert!(err.to_string().contains("I/O error"));

        let err = PersistError::WorldNotFound("my_world".to_string());
        assert!(err.to_string().contains("my_world"));

        let err = PersistError::VersionMismatch {
            world_version: 5,
            reason: VersionCheck::TooNew(5),
        };
        assert!(err.to_string().contains("version mismatch"));
        assert!(err.to_string().contains("v5"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let persist_err: PersistError = io_err.into();
        assert!(matches!(persist_err, PersistError::WorldNotFound(_)));

        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let persist_err: PersistError = io_err.into();
        assert!(matches!(persist_err, PersistError::PermissionDenied(_)));

        let io_err = std::io::Error::new(std::io::ErrorKind::Other, "other");
        let persist_err: PersistError = io_err.into();
        assert!(matches!(persist_err, PersistError::Io(_)));
    }
}
