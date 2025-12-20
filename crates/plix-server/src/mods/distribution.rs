//! Mod distribution integration
//!
//! This module integrates the mod distribution system with the server,
//! handling automatic download, verification, and installation of mods
//! during server startup.

use plix_mod_distribution::{
    DistributionConfig, DistributionError, ResolvedGraph,
};
use std::path::{Path, PathBuf};
use tracing::info;

/// Default config file name
pub const CONFIG_FILE_NAME: &str = "server_mods.toml";

/// Result type for distribution operations
pub type DistributionResult<T> = Result<T, DistributionError>;

/// Initialize mod distribution for the server
///
/// This function:
/// 1. Loads the distribution config from `server_mods.toml`
/// 2. Resolves dependencies
/// 3. Updates/generates the lockfile
/// 4. Downloads and verifies bundles
/// 5. Extracts mods to the cache
///
/// Returns the list of installed mod directories that should be loaded.
pub async fn init_mod_distribution(
    server_root: impl AsRef<Path>,
) -> DistributionResult<Vec<PathBuf>> {
    let server_root = server_root.as_ref();
    let config_path = server_root.join(CONFIG_FILE_NAME);

    // Check if config exists
    if !config_path.exists() {
        info!(
            path = %config_path.display(),
            "No {} found, skipping mod distribution",
            CONFIG_FILE_NAME
        );
        return Ok(Vec::new());
    }

    info!(
        path = %config_path.display(),
        "Loading mod distribution config"
    );

    // Load configuration
    let config = DistributionConfig::load(&config_path)?;

    if config.mods.is_empty() {
        info!("No mods configured in server_mods.toml");
        return Ok(Vec::new());
    }

    info!(
        registries = config.registries.len(),
        mods = config.mods.len(),
        "Mod distribution config loaded"
    );

    // Run the distribution pipeline
    let resolved = plix_mod_distribution::resolve_and_install(&config, server_root).await?;

    // Collect installed mod paths
    let installer = plix_mod_distribution::installer::Installer::new(&config)?;
    let mod_paths: Vec<PathBuf> = resolved
        .mods
        .iter()
        .map(|m| installer.install_path(&m.id, &m.version))
        .collect();

    info!(
        count = mod_paths.len(),
        "Mod distribution complete, {} mods installed",
        mod_paths.len()
    );

    Ok(mod_paths)
}

/// Initialize mod distribution with a custom config path
pub async fn init_mod_distribution_with_config(
    server_root: impl AsRef<Path>,
    config_path: impl AsRef<Path>,
) -> DistributionResult<Vec<PathBuf>> {
    let server_root = server_root.as_ref();
    let config_path = config_path.as_ref();

    info!(
        path = %config_path.display(),
        "Loading mod distribution config"
    );

    // Load configuration
    let config = DistributionConfig::load(config_path)?;

    if config.mods.is_empty() {
        info!("No mods configured");
        return Ok(Vec::new());
    }

    // Run the distribution pipeline
    let resolved = plix_mod_distribution::resolve_and_install(&config, server_root).await?;

    // Collect installed mod paths
    let installer = plix_mod_distribution::installer::Installer::new(&config)?;
    let mod_paths: Vec<PathBuf> = resolved
        .mods
        .iter()
        .map(|m| installer.install_path(&m.id, &m.version))
        .collect();

    Ok(mod_paths)
}

/// Check if mod distribution is configured
pub fn is_distribution_configured(server_root: impl AsRef<Path>) -> bool {
    server_root.as_ref().join(CONFIG_FILE_NAME).exists()
}

/// Get the lockfile path for a server
pub fn lockfile_path(server_root: impl AsRef<Path>) -> PathBuf {
    server_root.as_ref().join("mods.lock")
}

/// Check if a lockfile exists
pub fn has_lockfile(server_root: impl AsRef<Path>) -> bool {
    lockfile_path(server_root).exists()
}

/// Distribution statistics for server startup
#[derive(Debug, Clone, Default)]
pub struct DistributionStats {
    /// Number of mods resolved
    pub mods_resolved: usize,
    /// Total download size in bytes
    pub total_download_size: u64,
    /// Number of mods already cached
    pub cache_hits: usize,
    /// Number of mods downloaded
    pub downloads: usize,
}

impl From<ResolvedGraph> for DistributionStats {
    fn from(resolved: ResolvedGraph) -> Self {
        let downloads = resolved.mods.len().saturating_sub(resolved.stats.cached_count);
        Self {
            mods_resolved: resolved.mods.len(),
            total_download_size: resolved.stats.total_size,
            cache_hits: resolved.stats.cached_count,
            downloads,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_is_distribution_configured() {
        let temp = tempdir().unwrap();

        // No config
        assert!(!is_distribution_configured(temp.path()));

        // With config
        std::fs::write(temp.path().join(CONFIG_FILE_NAME), "").unwrap();
        assert!(is_distribution_configured(temp.path()));
    }

    #[test]
    fn test_lockfile_path() {
        let temp = tempdir().unwrap();
        let path = lockfile_path(temp.path());
        assert!(path.ends_with("mods.lock"));
    }

    #[test]
    fn test_has_lockfile() {
        let temp = tempdir().unwrap();

        // No lockfile
        assert!(!has_lockfile(temp.path()));

        // With lockfile
        std::fs::write(temp.path().join("mods.lock"), "{}").unwrap();
        assert!(has_lockfile(temp.path()));
    }
}
