//! Platform-specific launch operations

use crate::error::LauncherError;
use std::path::Path;
use std::process::{Command, Stdio};

/// Launch the game binary with optional arguments
pub fn launch_game(
    binary_path: &Path,
    args: &[String],
    stay_open: bool,
) -> Result<(), LauncherError> {
    if !binary_path.exists() {
        return Err(LauncherError::Launch(format!(
            "Game binary not found: {}",
            binary_path.display()
        )));
    }

    tracing::info!("Launching game: {}", binary_path.display());

    let mut command = Command::new(binary_path);

    // Add any game arguments
    if !args.is_empty() {
        tracing::debug!("Game arguments: {:?}", args);
        command.args(args);
    }

    // Set working directory to binary's parent
    if let Some(parent) = binary_path.parent() {
        command.current_dir(parent);
    }

    if stay_open {
        // Wait for game to exit
        let status = command
            .stdin(Stdio::null())
            .status()
            .map_err(|e| LauncherError::Launch(format!("Failed to launch game: {}", e)))?;

        if !status.success() {
            tracing::warn!("Game exited with status: {}", status);
        }
    } else {
        // Spawn and detach
        command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| LauncherError::Launch(format!("Failed to launch game: {}", e)))?;

        tracing::info!("Game launched, launcher exiting");
    }

    Ok(())
}

/// Ensure binary is executable (Unix only)
#[cfg(unix)]
pub fn ensure_executable(path: &Path) -> Result<(), LauncherError> {
    use std::os::unix::fs::PermissionsExt;

    if !path.exists() {
        return Ok(());
    }

    let metadata = std::fs::metadata(path)?;
    let mut perms = metadata.permissions();
    let mode = perms.mode();

    // Add execute permission if not present
    if mode & 0o111 == 0 {
        perms.set_mode(mode | 0o755);
        std::fs::set_permissions(path, perms)?;
        tracing::debug!("Set executable permission on {}", path.display());
    }

    Ok(())
}

#[cfg(windows)]
pub fn ensure_executable(_path: &Path) -> Result<(), LauncherError> {
    // No-op on Windows
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_launch_game_missing_binary() {
        let result = launch_game(Path::new("/nonexistent/binary"), &[], false);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    #[cfg(unix)]
    fn test_ensure_executable() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempdir().unwrap();
        let path = dir.path().join("test_binary");

        let mut file = std::fs::File::create(&path).unwrap();
        file.write_all(b"#!/bin/sh\necho test").unwrap();

        // Remove execute permission
        let perms = std::fs::Permissions::from_mode(0o644);
        std::fs::set_permissions(&path, perms).unwrap();

        ensure_executable(&path).unwrap();

        let metadata = std::fs::metadata(&path).unwrap();
        let mode = metadata.permissions().mode();
        assert!(mode & 0o111 != 0); // Has execute permission
    }
}
