//! Bundle downloading with timeout, retry, and size limit support

use crate::config::DistributionConfig;
use crate::errors::DistributionError;
use std::path::Path;
use std::time::Duration;

/// Bundle downloader with configurable timeouts and retries
pub struct Downloader {
    client: reqwest::Client,
    retries: u32,
    max_bundle_size: u64,
}

impl Downloader {
    /// Create a new downloader from config
    pub fn new(config: &DistributionConfig) -> Self {
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(config.download.connect_timeout_secs))
            .read_timeout(Duration::from_secs(config.download.read_timeout_secs))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            retries: config.download.retries,
            max_bundle_size: config.download.max_bundle_size,
        }
    }

    /// Create a downloader with custom settings
    pub fn with_settings(
        connect_timeout: Duration,
        read_timeout: Duration,
        retries: u32,
        max_bundle_size: u64,
    ) -> Result<Self, DistributionError> {
        let client = reqwest::Client::builder()
            .connect_timeout(connect_timeout)
            .read_timeout(read_timeout)
            .build()
            .map_err(|e| DistributionError::internal(&e.to_string()))?;

        Ok(Self {
            client,
            retries,
            max_bundle_size,
        })
    }

    /// Download a bundle from URL to destination path
    pub async fn download_bundle(
        &self,
        url: &str,
        dest: &Path,
    ) -> Result<u64, DistributionError> {
        // Handle local files (file:// URLs)
        if url.starts_with("file://") {
            return self.copy_local_file(url, dest);
        }

        // HTTP download with retries
        let mut last_error = None;

        for attempt in 1..=self.retries {
            tracing::debug!(
                "Downloading {} (attempt {}/{})",
                url,
                attempt,
                self.retries
            );

            match self.download_http(url, dest).await {
                Ok(size) => {
                    tracing::info!(
                        "Downloaded {} bytes from {} to {}",
                        size,
                        url,
                        dest.display()
                    );
                    return Ok(size);
                }
                Err(e) => {
                    tracing::warn!(
                        "Download attempt {} failed for {}: {}",
                        attempt,
                        url,
                        e
                    );
                    last_error = Some(e);

                    // Clean up partial download
                    if dest.exists() {
                        let _ = std::fs::remove_file(dest);
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            DistributionError::download_failed(url, "unknown error")
        }))
    }

    /// Download via HTTP
    async fn download_http(&self, url: &str, dest: &Path) -> Result<u64, DistributionError> {
        let response = self.client.get(url).send().await.map_err(|e| {
            if e.is_connect() || e.is_timeout() {
                DistributionError::download_failed(url, &format!("connection failed: {}", e))
            } else {
                DistributionError::download_failed(url, &e.to_string())
            }
        })?;

        if !response.status().is_success() {
            return Err(DistributionError::download_failed(
                url,
                &format!("HTTP {}", response.status()),
            ));
        }

        // Check content length if available
        if let Some(content_length) = response.content_length() {
            if content_length > self.max_bundle_size {
                return Err(DistributionError::bundle_too_large(
                    url,
                    content_length,
                    self.max_bundle_size,
                ));
            }
        }

        // Create parent directory if needed
        if let Some(parent) = dest.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                DistributionError::internal(&format!("Failed to create directory: {}", e))
            })?;
        }

        // Stream to file
        let mut file = tokio::fs::File::create(dest).await.map_err(|e| {
            DistributionError::internal(&format!(
                "Failed to create file '{}': {}",
                dest.display(),
                e
            ))
        })?;

        let bytes = response.bytes().await.map_err(|e| {
            DistributionError::download_failed(url, &format!("failed to read body: {}", e))
        })?;

        // Check size after download
        let size = bytes.len() as u64;
        if size > self.max_bundle_size {
            return Err(DistributionError::bundle_too_large(
                url,
                size,
                self.max_bundle_size,
            ));
        }

        use tokio::io::AsyncWriteExt;
        file.write_all(&bytes).await.map_err(|e| {
            DistributionError::internal(&format!("Failed to write file: {}", e))
        })?;

        Ok(size)
    }

    /// Copy a local file (file:// URL)
    fn copy_local_file(&self, url: &str, dest: &Path) -> Result<u64, DistributionError> {
        let source_path = url.strip_prefix("file://").unwrap_or(url);

        // Check source exists
        if !Path::new(source_path).exists() {
            return Err(DistributionError::download_failed(
                url,
                "file not found",
            ));
        }

        // Check size
        let metadata = std::fs::metadata(source_path).map_err(|e| {
            DistributionError::download_failed(url, &format!("failed to read metadata: {}", e))
        })?;

        let size = metadata.len();
        if size > self.max_bundle_size {
            return Err(DistributionError::bundle_too_large(
                url,
                size,
                self.max_bundle_size,
            ));
        }

        // Create parent directory if needed
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                DistributionError::internal(&format!("Failed to create directory: {}", e))
            })?;
        }

        // Copy file
        std::fs::copy(source_path, dest).map_err(|e| {
            DistributionError::download_failed(url, &format!("copy failed: {}", e))
        })?;

        tracing::info!(
            "Copied {} bytes from {} to {}",
            size,
            source_path,
            dest.display()
        );

        Ok(size)
    }

    /// Check if a bundle is already cached with matching hash
    pub fn is_cached(&self, cache_path: &Path, expected_hash: &str) -> bool {
        if !cache_path.exists() {
            return false;
        }

        // Verify hash matches
        match crate::integrity::calculate_hash(cache_path) {
            Ok(actual_hash) => actual_hash == expected_hash,
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_download_local_file() {
        let temp = tempdir().unwrap();
        let source_path = temp.path().join("source.plixmod");
        let dest_path = temp.path().join("dest.plixmod");

        // Create source file
        std::fs::write(&source_path, b"test bundle content").unwrap();

        let downloader = Downloader::with_settings(
            Duration::from_secs(5),
            Duration::from_secs(30),
            1,
            1024 * 1024,
        )
        .unwrap();

        let url = format!("file://{}", source_path.display());
        let size = downloader.download_bundle(&url, &dest_path).await.unwrap();

        assert_eq!(size, 19); // "test bundle content" length
        assert!(dest_path.exists());
        assert_eq!(
            std::fs::read_to_string(&dest_path).unwrap(),
            "test bundle content"
        );
    }

    #[tokio::test]
    async fn test_download_local_file_not_found() {
        let temp = tempdir().unwrap();
        let dest_path = temp.path().join("dest.plixmod");

        let downloader = Downloader::with_settings(
            Duration::from_secs(5),
            Duration::from_secs(30),
            1,
            1024 * 1024,
        )
        .unwrap();

        let result = downloader
            .download_bundle("file:///nonexistent/path.plixmod", &dest_path)
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_download_size_limit() {
        let temp = tempdir().unwrap();
        let source_path = temp.path().join("large.plixmod");
        let dest_path = temp.path().join("dest.plixmod");

        // Create a file larger than the limit
        std::fs::write(&source_path, vec![0u8; 1000]).unwrap();

        let downloader = Downloader::with_settings(
            Duration::from_secs(5),
            Duration::from_secs(30),
            1,
            500, // 500 byte limit
        )
        .unwrap();

        let url = format!("file://{}", source_path.display());
        let result = downloader.download_bundle(&url, &dest_path).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, crate::errors::ErrorCode::DownloadFailed);
    }

    #[test]
    fn test_is_cached() {
        let temp = tempdir().unwrap();
        let cache_path = temp.path().join("cached.plixmod");

        // Create a file with known content
        let content = b"test content for hash";
        std::fs::write(&cache_path, content).unwrap();

        let downloader = Downloader::with_settings(
            Duration::from_secs(5),
            Duration::from_secs(30),
            1,
            1024 * 1024,
        )
        .unwrap();

        // Calculate the actual hash
        let actual_hash = crate::integrity::calculate_hash(&cache_path).unwrap();

        // Should return true for matching hash
        assert!(downloader.is_cached(&cache_path, &actual_hash));

        // Should return false for wrong hash
        assert!(!downloader.is_cached(&cache_path, "0000000000000000000000000000000000000000000000000000000000000000"));

        // Should return false for non-existent file
        assert!(!downloader.is_cached(&temp.path().join("nonexistent"), &actual_hash));
    }
}
