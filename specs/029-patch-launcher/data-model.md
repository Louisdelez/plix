# Data Model: Patch Updater & Launcher

**Feature**: 029-patch-launcher
**Date**: 2025-12-18

## Entities

### 1. Remote Manifest

The manifest is fetched from the update server and describes the current release.

```rust
/// Remote manifest describing a release
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// Manifest format version (for future compatibility)
    pub manifest_version: u32,

    /// Game version (semver format)
    pub version: String,

    /// Optional protocol version for server compatibility
    #[serde(default)]
    pub protocol_version: Option<u8>,

    /// Release timestamp (Unix epoch seconds)
    pub release_date: u64,

    /// Files included in this release
    pub files: Vec<ManifestFile>,

    /// Optional release notes URL
    #[serde(default)]
    pub release_notes_url: Option<String>,
}

/// A file entry in the manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestFile {
    /// Relative path within installation directory
    pub path: String,

    /// Download URL for this file
    pub url: String,

    /// File size in bytes
    pub size: u64,

    /// SHA256 checksum (hex-encoded, lowercase)
    pub sha256: String,

    /// Whether file is executable (for chmod on Unix)
    #[serde(default)]
    pub executable: bool,
}
```

**Validation Rules**:
- `manifest_version` must be 1 (current format)
- `version` must be valid semver
- `files` must not be empty
- Each `sha256` must be 64 hex characters
- Each `url` must be valid HTTP/HTTPS URL
- Each `path` must be relative (no `..` or absolute paths)

### 2. Local State

Tracks the currently installed version and file checksums.

```rust
/// Local launcher state persisted to disk
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LocalState {
    /// State format version
    pub state_version: u32,

    /// Currently installed version (None if fresh install)
    pub installed_version: Option<String>,

    /// Last successful update timestamp
    pub last_update: Option<u64>,

    /// Checksums of installed files (path -> sha256)
    pub file_checksums: HashMap<String, String>,

    /// Installation directory path
    pub install_path: Option<String>,
}
```

**Lifecycle**:
- Created on first successful installation
- Updated after each successful update
- Read on launcher startup to determine update need

### 3. Launcher Configuration

User-configurable launcher settings.

```rust
/// Launcher configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LauncherConfig {
    /// Config format version
    pub config_version: u32,

    /// URL to fetch manifest from
    pub manifest_url: String,

    /// Stay open after launching game (default: false)
    #[serde(default)]
    pub stay_open: bool,

    /// HTTP timeout in seconds (default: 30)
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,

    /// Number of download retries (default: 3)
    #[serde(default = "default_retries")]
    pub max_retries: u32,

    /// Verbose logging (default: false)
    #[serde(default)]
    pub verbose: bool,
}

fn default_timeout() -> u64 { 30 }
fn default_retries() -> u32 { 3 }

impl Default for LauncherConfig {
    fn default() -> Self {
        Self {
            config_version: 1,
            manifest_url: "https://releases.plix.example/manifest.toml".to_string(),
            stay_open: false,
            timeout_seconds: 30,
            max_retries: 3,
            verbose: false,
        }
    }
}
```

### 4. Version

Represents a parsed semantic version.

```rust
/// Semantic version wrapper
pub struct Version(semver::Version);

impl Version {
    /// Parse from string
    pub fn parse(s: &str) -> Result<Self, VersionError>;

    /// Compare versions
    pub fn compare(&self, other: &Self) -> Ordering;

    /// Check if self is newer than other
    pub fn is_newer_than(&self, other: &Self) -> bool;
}
```

**Comparison Rules**:
- Standard semver ordering: 1.0.0 < 1.0.1 < 1.1.0 < 2.0.0
- Pre-release versions: 1.0.0-alpha < 1.0.0-beta < 1.0.0
- Downgrades not supported: `is_newer_than` returns false for equal or older

### 5. Update Status

Result of version comparison.

```rust
/// Result of checking for updates
#[derive(Debug, Clone)]
pub enum UpdateStatus {
    /// Already up to date
    UpToDate {
        version: String,
    },

    /// Update available
    UpdateAvailable {
        current: String,
        latest: String,
        files_to_update: Vec<ManifestFile>,
        total_download_size: u64,
    },

    /// Fresh installation required
    FreshInstall {
        version: String,
        files: Vec<ManifestFile>,
        total_download_size: u64,
    },

    /// Offline mode (manifest unreachable but local version exists)
    Offline {
        local_version: String,
    },

    /// Cannot proceed (no local version and offline)
    CannotProceed {
        reason: String,
    },
}
```

### 6. Download Progress

Progress information for UI updates.

```rust
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

#[derive(Debug, Clone)]
pub enum UpdatePhase {
    CheckingVersion,
    Downloading,
    Verifying,
    Installing,
    Complete,
    Failed(String),
}
```

## State Transitions

### Launcher Lifecycle

```
                    ┌─────────────────┐
                    │     Start       │
                    └────────┬────────┘
                             │
                    ┌────────▼────────┐
                    │  Load Config    │
                    └────────┬────────┘
                             │
                    ┌────────▼────────┐
                    │  Load State     │
                    └────────┬────────┘
                             │
                    ┌────────▼────────┐
              ┌─────│ Fetch Manifest  │─────┐
              │     └────────┬────────┘     │
           offline           │           success
              │              │              │
    ┌─────────▼─────────┐    │    ┌────────▼────────┐
    │ Check Local Valid │    │    │ Compare Version │
    └─────────┬─────────┘    │    └────────┬────────┘
              │              │             │
    ┌─────────▼─────────┐    │    ┌────────▼────────┐
    │   Launch Offline  │    │    │  Update Needed? │
    └───────────────────┘    │    └────────┬────────┘
                             │             │
                             │    ┌────────▼────────┐
                             │    │    Download     │
                             │    └────────┬────────┘
                             │             │
                             │    ┌────────▼────────┐
                             │    │     Verify      │
                             │    └────────┬────────┘
                             │             │
                             │    ┌────────▼────────┐
                             └───►│    Install      │
                                  └────────┬────────┘
                                           │
                                  ┌────────▼────────┐
                                  │  Save State     │
                                  └────────┬────────┘
                                           │
                                  ┌────────▼────────┐
                                  │  Launch Game    │
                                  └─────────────────┘
```

### File Update States

```
ManifestFile
    │
    ├── Missing locally → Download required
    │
    ├── Checksum mismatch → Re-download required
    │
    └── Checksum match → Skip (up to date)
```

## File Formats

### manifest.toml (Remote)

```toml
manifest_version = 1
version = "1.3.0"
protocol_version = 1
release_date = 1703030400

[[files]]
path = "plix-client"
url = "https://releases.plix.example/1.3.0/plix-client"
size = 47448064
sha256 = "a1b2c3d4e5f6..."
executable = true

[[files]]
path = "assets/arenas/test_arena.toml"
url = "https://releases.plix.example/1.3.0/assets/arenas/test_arena.toml"
size = 2048
sha256 = "f6e5d4c3b2a1..."
```

### state.toml (Local)

```toml
state_version = 1
installed_version = "1.3.0"
last_update = 1703030500
install_path = "/home/user/.local/share/plix/current"

[file_checksums]
"plix-client" = "a1b2c3d4e5f6..."
"assets/arenas/test_arena.toml" = "f6e5d4c3b2a1..."
```

### launcher.toml (Config)

```toml
config_version = 1
manifest_url = "https://releases.plix.example/manifest.toml"
stay_open = false
timeout_seconds = 30
max_retries = 3
verbose = false
```

## Relationships

```
┌─────────────────┐
│ LauncherConfig  │ ─── Loaded at startup, user-editable
└────────┬────────┘
         │ uses
         ▼
┌─────────────────┐         ┌─────────────────┐
│    Manifest     │◄────────│   HTTP Server   │
└────────┬────────┘ fetched └─────────────────┘
         │ contains
         ▼
┌─────────────────┐
│  ManifestFile   │ ─── One per file in release
└────────┬────────┘
         │ compared with
         ▼
┌─────────────────┐
│   LocalState    │ ─── Persisted between runs
└────────┬────────┘
         │ determines
         ▼
┌─────────────────┐
│  UpdateStatus   │ ─── Drives update flow
└─────────────────┘
```
