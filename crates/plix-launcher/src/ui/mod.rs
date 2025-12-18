//! UI and progress display

mod console;

pub use console::*;

/// Update phase for progress display
#[derive(Debug, Clone)]
pub enum UpdatePhase {
    CheckingVersion,
    Downloading,
    Verifying,
    Installing,
    Complete,
    Failed(String),
}

/// Download progress for a single file
#[derive(Debug, Clone)]
pub struct DownloadProgress {
    /// File being downloaded
    pub file_path: String,

    /// Bytes downloaded so far
    pub bytes_downloaded: u64,

    /// Total file size
    pub total_bytes: u64,

    /// Download speed (bytes per second)
    pub speed_bps: u64,
}

/// Overall update progress
#[derive(Debug, Clone)]
pub struct UpdateProgress {
    /// Current phase
    pub phase: UpdatePhase,

    /// Files completed
    pub files_completed: usize,

    /// Total files to process
    pub total_files: usize,

    /// Current file progress (if downloading)
    pub current_file: Option<DownloadProgress>,
}
