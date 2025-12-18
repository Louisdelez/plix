//! File download with retry

use crate::download::checksum::calculate_checksum;
use crate::error::LauncherError;
use crate::manifest::ManifestFile;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::time::Duration;

/// Download a file to the specified path with retry and checksum verification
pub fn download_file(
    file: &ManifestFile,
    dest_path: &Path,
    timeout_seconds: u64,
    max_retries: u32,
) -> Result<(), LauncherError> {
    let mut last_error = None;

    for attempt in 1..=max_retries {
        tracing::debug!(
            "Downloading {} (attempt {}/{})",
            file.path,
            attempt,
            max_retries
        );

        match download_file_once(file, dest_path, timeout_seconds) {
            Ok(()) => {
                tracing::debug!("Download complete: {}", file.path);
                return Ok(());
            }
            Err(e) => {
                tracing::warn!(
                    "Download attempt {} failed for {}: {}",
                    attempt,
                    file.path,
                    e
                );
                last_error = Some(e);

                if attempt < max_retries {
                    // Wait before retry (5 seconds)
                    std::thread::sleep(Duration::from_secs(5));
                }
            }
        }
    }

    Err(last_error
        .unwrap_or_else(|| LauncherError::Install("Download failed after all retries".into())))
}

/// Single download attempt
fn download_file_once(
    file: &ManifestFile,
    dest_path: &Path,
    timeout_seconds: u64,
) -> Result<(), LauncherError> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(timeout_seconds))
        .user_agent(format!("plix-launcher/{}", env!("CARGO_PKG_VERSION")))
        .build()?;

    let response = client.get(&file.url).send()?;

    if !response.status().is_success() {
        return Err(LauncherError::Manifest(format!(
            "HTTP {} downloading {}",
            response.status(),
            file.path
        )));
    }

    let bytes = response.bytes()?;

    // Verify size
    if bytes.len() as u64 != file.size {
        return Err(LauncherError::ChecksumMismatch {
            path: file.path.clone(),
            expected: format!("{} bytes", file.size),
            actual: format!("{} bytes", bytes.len()),
        });
    }

    // Verify checksum
    let actual_checksum = calculate_checksum(&bytes);
    if actual_checksum.to_lowercase() != file.sha256.to_lowercase() {
        return Err(LauncherError::ChecksumMismatch {
            path: file.path.clone(),
            expected: file.sha256.clone(),
            actual: actual_checksum,
        });
    }

    // Ensure parent directory exists
    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Write to file
    let mut dest_file = File::create(dest_path)?;
    dest_file.write_all(&bytes)?;
    dest_file.sync_all()?;

    Ok(())
}

/// Download files to a temporary directory
pub fn download_files_to_temp(
    files: &[ManifestFile],
    temp_dir: &Path,
    timeout_seconds: u64,
    max_retries: u32,
    progress_callback: Option<&dyn Fn(usize, usize, &str)>,
) -> Result<(), LauncherError> {
    fs::create_dir_all(temp_dir)?;

    for (index, file) in files.iter().enumerate() {
        if let Some(callback) = progress_callback {
            callback(index, files.len(), &file.path);
        }

        let dest_path = temp_dir.join(&file.path);
        download_file(file, &dest_path, timeout_seconds, max_retries)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_file_checksum_calc() {
        // Just test that checksum calculation works
        let data = b"test data for checksum";
        let checksum = calculate_checksum(data);
        assert_eq!(checksum.len(), 64);
    }
}
