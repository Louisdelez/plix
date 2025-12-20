//! Bundle extraction and cache management

use crate::config::DistributionConfig;
use crate::errors::DistributionError;
use crate::ModId;
use semver::Version;
use std::path::{Path, PathBuf};

/// Installer for extracting bundles to cache directory
pub struct Installer {
    /// Cache directory root
    cache_dir: PathBuf,
}

impl Installer {
    /// Create a new installer from config
    pub fn new(config: &DistributionConfig) -> Result<Self, DistributionError> {
        let cache_dir = config.cache_dir();

        // Create cache directories if needed
        std::fs::create_dir_all(cache_dir.join("bundles")).map_err(|e| {
            DistributionError::internal(&format!(
                "Failed to create bundles cache directory: {}",
                e
            ))
        })?;

        std::fs::create_dir_all(cache_dir.join("installed")).map_err(|e| {
            DistributionError::internal(&format!(
                "Failed to create installed cache directory: {}",
                e
            ))
        })?;

        Ok(Self { cache_dir })
    }

    /// Create an installer with a specific cache directory
    pub fn with_cache_dir(cache_dir: impl Into<PathBuf>) -> Result<Self, DistributionError> {
        let cache_dir = cache_dir.into();

        std::fs::create_dir_all(cache_dir.join("bundles")).map_err(|e| {
            DistributionError::internal(&format!(
                "Failed to create bundles cache directory: {}",
                e
            ))
        })?;

        std::fs::create_dir_all(cache_dir.join("installed")).map_err(|e| {
            DistributionError::internal(&format!(
                "Failed to create installed cache directory: {}",
                e
            ))
        })?;

        Ok(Self { cache_dir })
    }

    /// Get the path where a bundle should be stored (content-addressed by SHA-256)
    pub fn bundle_path(&self, sha256: &str) -> PathBuf {
        self.cache_dir.join("bundles").join(format!("{}.plixmod", sha256))
    }

    /// Get the path where a mod should be installed
    pub fn install_path(&self, mod_id: &ModId, version: &Version) -> PathBuf {
        self.cache_dir
            .join("installed")
            .join(mod_id.as_str())
            .join(version.to_string())
    }

    /// Extract a bundle to the installed directory
    pub fn extract_bundle(
        &self,
        bundle_path: &Path,
        install_dir: &Path,
    ) -> Result<(), DistributionError> {
        let file = std::fs::File::open(bundle_path).map_err(|e| {
            DistributionError::internal(&format!(
                "Failed to open bundle '{}': {}",
                bundle_path.display(),
                e
            ))
        })?;

        let mut archive = zip::ZipArchive::new(file)?;

        // Create install directory
        std::fs::create_dir_all(install_dir).map_err(|e| {
            DistributionError::internal(&format!(
                "Failed to create install directory '{}': {}",
                install_dir.display(),
                e
            ))
        })?;

        // Extract all files
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = file.name().to_string();

            // Skip directories (we create them as needed)
            if name.ends_with('/') {
                continue;
            }

            let dest_path = install_dir.join(&name);

            // Create parent directories
            if let Some(parent) = dest_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    DistributionError::internal(&format!(
                        "Failed to create directory '{}': {}",
                        parent.display(),
                        e
                    ))
                })?;
            }

            // Extract file
            let mut dest_file = std::fs::File::create(&dest_path).map_err(|e| {
                DistributionError::internal(&format!(
                    "Failed to create file '{}': {}",
                    dest_path.display(),
                    e
                ))
            })?;

            std::io::copy(&mut file, &mut dest_file).map_err(|e| {
                DistributionError::internal(&format!(
                    "Failed to extract '{}': {}",
                    name,
                    e
                ))
            })?;
        }

        // Validate extracted content
        self.validate_installation(install_dir)?;

        tracing::info!(
            "Extracted bundle to {}",
            install_dir.display()
        );

        Ok(())
    }

    /// Validate that an installation has required files
    fn validate_installation(&self, install_dir: &Path) -> Result<(), DistributionError> {
        // Check for mod.toml
        let manifest_path = install_dir.join("mod.toml");
        if !manifest_path.exists() {
            // Clean up failed installation
            let _ = std::fs::remove_dir_all(install_dir);
            return Err(DistributionError::invalid_index(
                "bundle",
                "Extracted bundle missing mod.toml",
            ));
        }

        // Validate manifest is parseable
        let manifest_content = std::fs::read_to_string(&manifest_path).map_err(|e| {
            let _ = std::fs::remove_dir_all(install_dir);
            DistributionError::internal(&format!("Failed to read mod.toml: {}", e))
        })?;

        let _: toml::Value = toml::from_str(&manifest_content).map_err(|e| {
            let _ = std::fs::remove_dir_all(install_dir);
            DistributionError::invalid_index("bundle", &format!("Invalid mod.toml: {}", e))
        })?;

        Ok(())
    }

    /// Check if a mod version is already installed
    pub fn is_installed(&self, mod_id: &ModId, version: &Version) -> bool {
        let install_dir = self.install_path(mod_id, version);
        install_dir.join("mod.toml").exists()
    }

    /// Remove an installed mod version
    pub fn uninstall(&self, mod_id: &ModId, version: &Version) -> Result<(), DistributionError> {
        let install_dir = self.install_path(mod_id, version);

        if install_dir.exists() {
            std::fs::remove_dir_all(&install_dir).map_err(|e| {
                DistributionError::internal(&format!(
                    "Failed to uninstall {} v{}: {}",
                    mod_id, version, e
                ))
            })?;

            // Clean up empty parent directories
            let mod_dir = install_dir.parent().unwrap();
            if mod_dir.read_dir().map(|mut d| d.next().is_none()).unwrap_or(false) {
                let _ = std::fs::remove_dir(mod_dir);
            }

            tracing::info!("Uninstalled {} v{}", mod_id, version);
        }

        Ok(())
    }

    /// List all installed mod versions
    pub fn list_installed(&self) -> Result<Vec<(ModId, Version)>, DistributionError> {
        let installed_dir = self.cache_dir.join("installed");
        let mut result = Vec::new();

        if !installed_dir.exists() {
            return Ok(result);
        }

        for mod_entry in std::fs::read_dir(&installed_dir)? {
            let mod_entry = mod_entry?;
            if !mod_entry.file_type()?.is_dir() {
                continue;
            }

            let mod_id = mod_entry.file_name().to_string_lossy().to_string();
            let mod_id = match ModId::new(&mod_id) {
                Ok(id) => id,
                Err(_) => continue, // Skip invalid mod IDs
            };

            for version_entry in std::fs::read_dir(mod_entry.path())? {
                let version_entry = version_entry?;
                if !version_entry.file_type()?.is_dir() {
                    continue;
                }

                let version_str = version_entry.file_name().to_string_lossy().to_string();
                if let Ok(version) = Version::parse(&version_str) {
                    result.push((mod_id.clone(), version));
                }
            }
        }

        Ok(result)
    }

    /// Get the manifest path for an installed mod
    pub fn manifest_path(&self, mod_id: &ModId, version: &Version) -> PathBuf {
        self.install_path(mod_id, version).join("mod.toml")
    }

    /// Get the WASM path for an installed mod (if it has one)
    pub fn wasm_path(&self, mod_id: &ModId, version: &Version) -> PathBuf {
        self.install_path(mod_id, version).join("mod.wasm")
    }

    /// Clean up orphaned bundles not referenced by any installed mod
    pub fn cleanup_orphaned_bundles(&self) -> Result<usize, DistributionError> {
        let bundles_dir = self.cache_dir.join("bundles");
        let _installed = self.list_installed()?;

        // Build set of hashes for installed mods
        // Note: This requires reading each installed mod's info, which we don't track
        // For now, just count bundles
        let removed = 0;

        for entry in std::fs::read_dir(&bundles_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map(|e| e == "plixmod").unwrap_or(false) {
                // Check if this bundle is referenced
                // TODO: Implement proper orphan detection using lockfile or metadata
                let _ = path;
            }
        }

        Ok(removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_test_bundle(path: &Path, id: &str, version: &str) {
        use std::io::Write;
        use zip::write::FileOptions;

        let file = std::fs::File::create(path).unwrap();
        let mut zip = zip::ZipWriter::new(file);

        let manifest = format!(
            r#"
id = "{}"
name = "Test Mod"
version = "{}"
api_version = 1
"#,
            id, version
        );

        zip.start_file::<_, ()>("mod.toml", FileOptions::default())
            .unwrap();
        zip.write_all(manifest.as_bytes()).unwrap();

        zip.start_file::<_, ()>("assets/test.txt", FileOptions::default())
            .unwrap();
        zip.write_all(b"test asset").unwrap();

        zip.finish().unwrap();
    }

    #[test]
    fn test_installer_creation() {
        let temp = tempdir().unwrap();
        let installer = Installer::with_cache_dir(temp.path()).unwrap();

        assert!(temp.path().join("bundles").exists());
        assert!(temp.path().join("installed").exists());
    }

    #[test]
    fn test_bundle_path() {
        let temp = tempdir().unwrap();
        let installer = Installer::with_cache_dir(temp.path()).unwrap();

        let path = installer.bundle_path("abc123");
        assert_eq!(
            path,
            temp.path().join("bundles").join("abc123.plixmod")
        );
    }

    #[test]
    fn test_install_path() {
        let temp = tempdir().unwrap();
        let installer = Installer::with_cache_dir(temp.path()).unwrap();

        let mod_id = ModId::new("my-mod").unwrap();
        let version = Version::new(1, 2, 3);

        let path = installer.install_path(&mod_id, &version);
        assert_eq!(
            path,
            temp.path().join("installed").join("my-mod").join("1.2.3")
        );
    }

    #[test]
    fn test_extract_bundle() {
        let temp = tempdir().unwrap();
        let installer = Installer::with_cache_dir(temp.path()).unwrap();

        let bundle_path = temp.path().join("test.plixmod");
        create_test_bundle(&bundle_path, "test-mod", "1.0.0");

        let mod_id = ModId::new("test-mod").unwrap();
        let version = Version::new(1, 0, 0);
        let install_dir = installer.install_path(&mod_id, &version);

        installer.extract_bundle(&bundle_path, &install_dir).unwrap();

        assert!(install_dir.join("mod.toml").exists());
        assert!(install_dir.join("assets/test.txt").exists());
    }

    #[test]
    fn test_is_installed() {
        let temp = tempdir().unwrap();
        let installer = Installer::with_cache_dir(temp.path()).unwrap();

        let mod_id = ModId::new("test-mod").unwrap();
        let version = Version::new(1, 0, 0);

        assert!(!installer.is_installed(&mod_id, &version));

        // Install
        let bundle_path = temp.path().join("test.plixmod");
        create_test_bundle(&bundle_path, "test-mod", "1.0.0");
        let install_dir = installer.install_path(&mod_id, &version);
        installer.extract_bundle(&bundle_path, &install_dir).unwrap();

        assert!(installer.is_installed(&mod_id, &version));
    }

    #[test]
    fn test_uninstall() {
        let temp = tempdir().unwrap();
        let installer = Installer::with_cache_dir(temp.path()).unwrap();

        // Install first
        let bundle_path = temp.path().join("test.plixmod");
        create_test_bundle(&bundle_path, "test-mod", "1.0.0");

        let mod_id = ModId::new("test-mod").unwrap();
        let version = Version::new(1, 0, 0);
        let install_dir = installer.install_path(&mod_id, &version);
        installer.extract_bundle(&bundle_path, &install_dir).unwrap();

        assert!(installer.is_installed(&mod_id, &version));

        // Uninstall
        installer.uninstall(&mod_id, &version).unwrap();

        assert!(!installer.is_installed(&mod_id, &version));
    }

    #[test]
    fn test_list_installed() {
        let temp = tempdir().unwrap();
        let installer = Installer::with_cache_dir(temp.path()).unwrap();

        // Install multiple versions
        let bundle1_path = temp.path().join("test1.plixmod");
        create_test_bundle(&bundle1_path, "mod-a", "1.0.0");

        let bundle2_path = temp.path().join("test2.plixmod");
        create_test_bundle(&bundle2_path, "mod-a", "2.0.0");

        let bundle3_path = temp.path().join("test3.plixmod");
        create_test_bundle(&bundle3_path, "mod-b", "1.0.0");

        let mod_a = ModId::new("mod-a").unwrap();
        let mod_b = ModId::new("mod-b").unwrap();

        installer
            .extract_bundle(
                &bundle1_path,
                &installer.install_path(&mod_a, &Version::new(1, 0, 0)),
            )
            .unwrap();
        installer
            .extract_bundle(
                &bundle2_path,
                &installer.install_path(&mod_a, &Version::new(2, 0, 0)),
            )
            .unwrap();
        installer
            .extract_bundle(
                &bundle3_path,
                &installer.install_path(&mod_b, &Version::new(1, 0, 0)),
            )
            .unwrap();

        let installed = installer.list_installed().unwrap();
        assert_eq!(installed.len(), 3);
    }

    #[test]
    fn test_extract_invalid_bundle() {
        let temp = tempdir().unwrap();
        let installer = Installer::with_cache_dir(temp.path()).unwrap();

        // Create a bundle without mod.toml
        let bundle_path = temp.path().join("bad.plixmod");
        {
            use std::io::Write;
            let file = std::fs::File::create(&bundle_path).unwrap();
            let mut zip = zip::ZipWriter::new(file);
            zip.start_file::<_, ()>("other.txt", Default::default())
                .unwrap();
            zip.write_all(b"test").unwrap();
            zip.finish().unwrap();
        }

        let install_dir = temp.path().join("installed/bad-mod/1.0.0");
        let result = installer.extract_bundle(&bundle_path, &install_dir);

        assert!(result.is_err());
        // Install directory should be cleaned up
        assert!(!install_dir.exists());
    }
}
