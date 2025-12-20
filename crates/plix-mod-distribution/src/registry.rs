//! Registry sources: local filesystem and HTTP

use crate::config::DistributionConfig;
use crate::errors::DistributionError;
use crate::index::RegistryIndex;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

/// Registry source trait for fetching indexes
#[async_trait::async_trait]
pub trait RegistrySource: Send + Sync {
    /// Fetch the registry index
    async fn fetch_index(&self) -> Result<RegistryIndex, DistributionError>;

    /// Get the registry name
    fn name(&self) -> &str;
}

/// Local filesystem registry source
pub struct LocalRegistrySource {
    name: String,
    path: PathBuf,
}

impl LocalRegistrySource {
    /// Create a new local registry source
    pub fn new(name: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
        }
    }
}

#[async_trait::async_trait]
impl RegistrySource for LocalRegistrySource {
    async fn fetch_index(&self) -> Result<RegistryIndex, DistributionError> {
        let index_path = self.path.join("index.json");
        tracing::debug!(
            "Loading local index from {}",
            index_path.display()
        );

        let content = tokio::fs::read(&index_path).await.map_err(|e| {
            DistributionError::registry_unreachable(
                &self.name,
                &index_path.to_string_lossy(),
                &e.to_string(),
            )
        })?;

        RegistryIndex::from_bytes(&content).map_err(|e| {
            DistributionError::invalid_index(&self.name, &e.to_string())
        })
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// HTTP registry source with timeout and retry support
pub struct HttpRegistrySource {
    name: String,
    url: String,
    client: reqwest::Client,
    retries: u32,
}

impl HttpRegistrySource {
    /// Create a new HTTP registry source
    pub fn new(
        name: impl Into<String>,
        url: impl Into<String>,
        connect_timeout: Duration,
        read_timeout: Duration,
        retries: u32,
    ) -> Result<Self, DistributionError> {
        let client = reqwest::Client::builder()
            .connect_timeout(connect_timeout)
            .read_timeout(read_timeout)
            .build()
            .map_err(|e| DistributionError::internal(&e.to_string()))?;

        Ok(Self {
            name: name.into(),
            url: url.into(),
            client,
            retries,
        })
    }
}

#[async_trait::async_trait]
impl RegistrySource for HttpRegistrySource {
    async fn fetch_index(&self) -> Result<RegistryIndex, DistributionError> {
        let mut last_error = None;

        for attempt in 1..=self.retries {
            tracing::debug!(
                "Fetching index from {} (attempt {}/{})",
                self.url,
                attempt,
                self.retries
            );

            match self.client.get(&self.url).send().await {
                Ok(response) => {
                    if !response.status().is_success() {
                        last_error = Some(DistributionError::registry_unreachable(
                            &self.name,
                            &self.url,
                            &format!("HTTP {}", response.status()),
                        ));
                        continue;
                    }

                    let bytes = response.bytes().await.map_err(|e| {
                        DistributionError::registry_unreachable(
                            &self.name,
                            &self.url,
                            &e.to_string(),
                        )
                    })?;

                    return RegistryIndex::from_bytes(&bytes).map_err(|e| {
                        DistributionError::invalid_index(&self.name, &e.to_string())
                    });
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to fetch index from {} (attempt {}): {}",
                        self.url,
                        attempt,
                        e
                    );
                    last_error = Some(DistributionError::registry_unreachable(
                        &self.name,
                        &self.url,
                        &e.to_string(),
                    ));
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            DistributionError::registry_unreachable(&self.name, &self.url, "unknown error")
        }))
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Manager for multiple registry sources with priority ordering
pub struct RegistryManager {
    sources: Vec<Box<dyn RegistrySource>>,
}

impl RegistryManager {
    /// Create a new registry manager from config
    pub fn new(config: &DistributionConfig) -> Result<Self, DistributionError> {
        let mut sources: Vec<Box<dyn RegistrySource>> = Vec::new();

        for reg_config in config.enabled_registries() {
            let source: Box<dyn RegistrySource> = if reg_config.is_local() {
                let path = reg_config.local_path().ok_or_else(|| {
                    DistributionError::config_error(&format!(
                        "Invalid local registry path: {}",
                        reg_config.url
                    ))
                })?;
                Box::new(LocalRegistrySource::new(&reg_config.name, path))
            } else {
                Box::new(HttpRegistrySource::new(
                    &reg_config.name,
                    &reg_config.url,
                    Duration::from_secs(config.download.connect_timeout_secs),
                    Duration::from_secs(config.download.read_timeout_secs),
                    config.download.retries,
                )?)
            };
            sources.push(source);
        }

        if sources.is_empty() {
            return Err(DistributionError::config_error(
                "No enabled registries configured",
            ));
        }

        Ok(Self { sources })
    }

    /// Fetch all registry indexes
    pub async fn fetch_all_indexes(&self) -> Result<HashMap<String, RegistryIndex>, DistributionError> {
        let mut indexes = HashMap::new();

        for source in &self.sources {
            match source.fetch_index().await {
                Ok(index) => {
                    tracing::info!(
                        "Loaded index from registry '{}' ({} mods)",
                        source.name(),
                        index.mods.len()
                    );
                    indexes.insert(source.name().to_string(), index);
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to load index from registry '{}': {}",
                        source.name(),
                        e
                    );
                    // Continue with other registries, but track the error
                    // If all registries fail, we'll return an error
                }
            }
        }

        if indexes.is_empty() {
            return Err(DistributionError::registry_unreachable(
                "all",
                "all configured registries",
                "Failed to load any registry index",
            ));
        }

        Ok(indexes)
    }

    /// Get registry names in priority order
    pub fn registry_names(&self) -> Vec<&str> {
        self.sources.iter().map(|s| s.name()).collect()
    }
}

/// Import a bundle into a local registry
pub fn import_bundle(
    registry_path: &std::path::Path,
    bundle_path: &std::path::Path,
) -> Result<(), DistributionError> {
    use crate::bundle::BundleMetadata;
    use crate::integrity::calculate_hash;

    // Calculate hash
    let hash = calculate_hash(bundle_path)?;

    // Extract metadata
    let metadata = BundleMetadata::from_bundle(bundle_path)?;

    // Copy bundle to registry
    let bundles_dir = registry_path.join("bundles");
    std::fs::create_dir_all(&bundles_dir)?;

    let dest_filename = format!(
        "{}-{}.plixmod",
        metadata.id,
        metadata.version
    );
    let dest_path = bundles_dir.join(&dest_filename);
    std::fs::copy(bundle_path, &dest_path)?;

    tracing::info!(
        "Imported {} v{} to registry (sha256: {})",
        metadata.id,
        metadata.version,
        hash
    );

    // Note: The index.json would need to be updated manually or via a separate tool
    // This is a simplified import that just copies the bundle

    Ok(())
}

/// List mods in a local registry
pub fn list_local_mods(
    registry_path: &std::path::Path,
) -> Result<Vec<(String, String)>, DistributionError> {
    let index_path = registry_path.join("index.json");

    if !index_path.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(&index_path)?;
    let index = RegistryIndex::from_json(&content)?;

    let mut mods = Vec::new();
    for mod_entry in &index.mods {
        for version in &mod_entry.versions {
            mods.push((mod_entry.id.to_string(), version.version.to_string()));
        }
    }

    Ok(mods)
}

// Re-export async_trait macro for convenience
pub use async_trait::async_trait;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_local_registry_source() {
        let temp = tempdir().unwrap();
        let index_json = r#"{
            "registry_version": 1,
            "name": "Test Local",
            "mods": [{
                "id": "test-mod",
                "name": "Test Mod",
                "versions": [{
                    "version": "1.0.0",
                    "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                    "download_url": "bundles/test-mod-1.0.0.plixmod",
                    "size": 100,
                    "api_version": 1
                }]
            }]
        }"#;

        std::fs::write(temp.path().join("index.json"), index_json).unwrap();

        let source = LocalRegistrySource::new("local", temp.path());
        let index = source.fetch_index().await.unwrap();

        assert_eq!(index.name, "Test Local");
        assert_eq!(index.mods.len(), 1);
    }

    #[tokio::test]
    async fn test_local_registry_not_found() {
        let source = LocalRegistrySource::new("nonexistent", "/nonexistent/path");
        let result = source.fetch_index().await;
        assert!(result.is_err());
    }

    #[test]
    fn test_list_local_mods() {
        let temp = tempdir().unwrap();
        let index_json = r#"{
            "registry_version": 1,
            "name": "Test",
            "mods": [{
                "id": "mod-a",
                "name": "Mod A",
                "versions": [
                    {"version": "1.0.0", "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855", "download_url": "a.plixmod", "size": 100, "api_version": 1},
                    {"version": "1.1.0", "sha256": "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2", "download_url": "a.plixmod", "size": 100, "api_version": 1}
                ]
            }]
        }"#;

        std::fs::write(temp.path().join("index.json"), index_json).unwrap();

        let mods = list_local_mods(temp.path()).unwrap();
        assert_eq!(mods.len(), 2);
        assert!(mods.contains(&("mod-a".to_string(), "1.0.0".to_string())));
        assert!(mods.contains(&("mod-a".to_string(), "1.1.0".to_string())));
    }
}
