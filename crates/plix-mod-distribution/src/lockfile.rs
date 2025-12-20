//! Lockfile (mods.lock) for reproducible mod installations

use crate::errors::DistributionError;
use crate::resolver::ResolvedGraph;
use crate::ModId;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Lockfile schema version
pub const LOCKFILE_VERSION: u32 = 1;

/// Lockfile for reproducible mod installations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lockfile {
    /// Lockfile schema version
    pub version: u32,

    /// Generation timestamp (RFC 3339)
    pub generated_at: String,

    /// Engine version used for compatibility check
    pub engine_version: String,

    /// Locked mod entries
    pub mods: Vec<LockedMod>,
}

impl Lockfile {
    /// Create a new lockfile from a resolved graph
    pub fn from_resolved(resolved: &ResolvedGraph) -> Self {
        let now = chrono_lite_now();
        let engine_version = crate::current_engine_version().to_string();

        let mods = resolved
            .mods
            .iter()
            .map(|m| LockedMod {
                id: m.id.clone(),
                version: m.version.clone(),
                sha256: m.sha256.clone(),
                source: m.source.clone(),
                download_url: m.download_url.clone(),
                dependencies: m.dependencies.clone(),
            })
            .collect();

        Self {
            version: LOCKFILE_VERSION,
            generated_at: now,
            engine_version,
            mods,
        }
    }

    /// Load lockfile from a path
    pub fn load(path: impl AsRef<Path>) -> Result<Self, DistributionError> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path).map_err(|e| {
            DistributionError::config_error(&format!(
                "Failed to read lockfile '{}': {}",
                path.display(),
                e
            ))
        })?;

        let lockfile: Self = serde_json::from_str(&content)?;
        lockfile.validate()?;
        Ok(lockfile)
    }

    /// Write lockfile to a path (deterministic output)
    pub fn write(&self, path: impl AsRef<Path>) -> Result<(), DistributionError> {
        let path = path.as_ref();

        // Sort mods by ID for deterministic output
        let mut sorted_lockfile = self.clone();
        sorted_lockfile.mods.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));

        // Also sort dependencies within each mod
        for m in &mut sorted_lockfile.mods {
            m.dependencies.sort_by(|a, b| a.as_str().cmp(b.as_str()));
        }

        let json = serde_json::to_string_pretty(&sorted_lockfile).map_err(|e| {
            DistributionError::internal(&format!("Failed to serialize lockfile: {}", e))
        })?;

        std::fs::write(path, json).map_err(|e| {
            DistributionError::internal(&format!(
                "Failed to write lockfile '{}': {}",
                path.display(),
                e
            ))
        })?;

        Ok(())
    }

    /// Validate the lockfile
    fn validate(&self) -> Result<(), DistributionError> {
        if self.version != LOCKFILE_VERSION {
            return Err(DistributionError::config_error(&format!(
                "Unsupported lockfile version {}, expected {}",
                self.version, LOCKFILE_VERSION
            )));
        }

        // Validate each locked mod
        for locked_mod in &self.mods {
            // Validate SHA-256 format
            if locked_mod.sha256.len() != 64
                || !locked_mod.sha256.chars().all(|c| c.is_ascii_hexdigit())
            {
                return Err(DistributionError::config_error(&format!(
                    "Invalid SHA-256 in lockfile for {}: must be 64 hex characters",
                    locked_mod.id
                )));
            }
        }

        Ok(())
    }

    /// Find a locked mod by ID
    pub fn find_mod(&self, id: &ModId) -> Option<&LockedMod> {
        self.mods.iter().find(|m| &m.id == id)
    }

    /// Update lockfile with new mods while preserving existing entries
    pub fn merge_with(&self, new_resolved: &ResolvedGraph) -> Self {
        let mut mods = self.mods.clone();
        let existing_ids: std::collections::HashSet<_> =
            mods.iter().map(|m| m.id.clone()).collect();

        // Add new mods that don't exist in lockfile
        for resolved_mod in &new_resolved.mods {
            if !existing_ids.contains(&resolved_mod.id) {
                mods.push(LockedMod {
                    id: resolved_mod.id.clone(),
                    version: resolved_mod.version.clone(),
                    sha256: resolved_mod.sha256.clone(),
                    source: resolved_mod.source.clone(),
                    download_url: resolved_mod.download_url.clone(),
                    dependencies: resolved_mod.dependencies.clone(),
                });
            }
        }

        Self {
            version: LOCKFILE_VERSION,
            generated_at: chrono_lite_now(),
            engine_version: crate::current_engine_version().to_string(),
            mods,
        }
    }

    /// Remove mods no longer in requirements
    pub fn prune(&self, required_ids: &[ModId]) -> Self {
        let required_set: std::collections::HashSet<_> = required_ids.iter().collect();

        // Collect all IDs that are required or are dependencies of required mods
        let mut keep_ids: std::collections::HashSet<ModId> = std::collections::HashSet::new();

        fn collect_deps(
            id: &ModId,
            mods: &[LockedMod],
            keep: &mut std::collections::HashSet<ModId>,
        ) {
            if keep.contains(id) {
                return;
            }
            if let Some(locked) = mods.iter().find(|m| &m.id == id) {
                keep.insert(id.clone());
                for dep_id in &locked.dependencies {
                    collect_deps(dep_id, mods, keep);
                }
            }
        }

        for id in &required_set {
            collect_deps(id, &self.mods, &mut keep_ids);
        }

        let mods = self
            .mods
            .iter()
            .filter(|m| keep_ids.contains(&m.id))
            .cloned()
            .collect();

        Self {
            version: LOCKFILE_VERSION,
            generated_at: chrono_lite_now(),
            engine_version: crate::current_engine_version().to_string(),
            mods,
        }
    }
}

/// A mod locked to a specific version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedMod {
    /// Mod ID
    pub id: ModId,

    /// Exact version
    #[serde(with = "version_serde")]
    pub version: Version,

    /// SHA-256 hash for integrity
    pub sha256: String,

    /// Source registry name
    pub source: String,

    /// Download URL (resolved)
    pub download_url: String,

    /// Resolved dependencies (mod IDs only, versions in their own entries)
    #[serde(default)]
    pub dependencies: Vec<ModId>,
}

// Serde helper for Version
mod version_serde {
    use semver::Version;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(version: &Version, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&version.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Version, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Version::parse(&s).map_err(serde::de::Error::custom)
    }
}

/// Simple RFC 3339 timestamp without external dependency
fn chrono_lite_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();

    // Simple conversion (doesn't account for leap seconds, good enough for timestamps)
    let days_since_epoch = secs / 86400;
    let time_of_day = secs % 86400;

    // Calculate year/month/day (simplified, assumes no leap seconds)
    let mut year = 1970;
    let mut remaining_days = days_since_epoch as i64;

    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }

    let mut month = 1;
    let days_in_months = if is_leap_year(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    for days in days_in_months {
        if remaining_days < days as i64 {
            break;
        }
        remaining_days -= days as i64;
        month += 1;
    }

    let day = remaining_days + 1;
    let hour = time_of_day / 3600;
    let minute = (time_of_day % 3600) / 60;
    let second = time_of_day % 60;

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hour, minute, second
    )
}

fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{ResolvedMod, ResolutionStats};
    use tempfile::tempdir;

    fn make_resolved_mod(id: &str, version: &str, deps: Vec<&str>) -> ResolvedMod {
        ResolvedMod {
            id: ModId::new_unchecked(id),
            version: Version::parse(version).unwrap(),
            source: "test".to_string(),
            download_url: format!("https://example.com/{}-{}.plixmod", id, version),
            sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(),
            size: 1000,
            dependencies: deps.into_iter().map(ModId::new_unchecked).collect(),
        }
    }

    #[test]
    fn test_lockfile_from_resolved() {
        let resolved = ResolvedGraph {
            mods: vec![
                make_resolved_mod("mod-a", "1.0.0", vec!["mod-b"]),
                make_resolved_mod("mod-b", "2.0.0", vec![]),
            ],
            stats: ResolutionStats::default(),
        };

        let lockfile = Lockfile::from_resolved(&resolved);

        assert_eq!(lockfile.version, LOCKFILE_VERSION);
        assert_eq!(lockfile.mods.len(), 2);

        let mod_a = lockfile.find_mod(&ModId::new_unchecked("mod-a")).unwrap();
        assert_eq!(mod_a.version, Version::new(1, 0, 0));
        assert_eq!(mod_a.dependencies.len(), 1);
    }

    #[test]
    fn test_lockfile_write_read() {
        let temp = tempdir().unwrap();
        let lockfile_path = temp.path().join("mods.lock");

        let resolved = ResolvedGraph {
            mods: vec![
                make_resolved_mod("mod-b", "2.0.0", vec![]),
                make_resolved_mod("mod-a", "1.0.0", vec!["mod-b"]),
            ],
            stats: ResolutionStats::default(),
        };

        let lockfile = Lockfile::from_resolved(&resolved);
        lockfile.write(&lockfile_path).unwrap();

        let loaded = Lockfile::load(&lockfile_path).unwrap();
        assert_eq!(loaded.version, LOCKFILE_VERSION);
        assert_eq!(loaded.mods.len(), 2);
    }

    #[test]
    fn test_lockfile_determinism() {
        let temp = tempdir().unwrap();

        let resolved = ResolvedGraph {
            mods: vec![
                make_resolved_mod("mod-z", "1.0.0", vec!["mod-a"]),
                make_resolved_mod("mod-a", "1.0.0", vec![]),
                make_resolved_mod("mod-m", "1.0.0", vec!["mod-z", "mod-a"]),
            ],
            stats: ResolutionStats::default(),
        };

        let lockfile1_path = temp.path().join("lock1.json");
        let lockfile2_path = temp.path().join("lock2.json");

        let lockfile = Lockfile::from_resolved(&resolved);
        lockfile.write(&lockfile1_path).unwrap();

        // Create again and write
        let lockfile2 = Lockfile::from_resolved(&resolved);
        lockfile2.write(&lockfile2_path).unwrap();

        // Read both files (excluding timestamp which will differ)
        let content1 = std::fs::read_to_string(&lockfile1_path).unwrap();
        let content2 = std::fs::read_to_string(&lockfile2_path).unwrap();

        // Parse and compare mods (timestamps will differ)
        let parsed1: Lockfile = serde_json::from_str(&content1).unwrap();
        let parsed2: Lockfile = serde_json::from_str(&content2).unwrap();

        // Mods should be in same order (sorted by ID)
        assert_eq!(parsed1.mods[0].id.as_str(), "mod-a");
        assert_eq!(parsed2.mods[0].id.as_str(), "mod-a");
        assert_eq!(parsed1.mods[1].id.as_str(), "mod-m");
        assert_eq!(parsed2.mods[1].id.as_str(), "mod-m");
        assert_eq!(parsed1.mods[2].id.as_str(), "mod-z");
        assert_eq!(parsed2.mods[2].id.as_str(), "mod-z");

        // Dependencies within mods should also be sorted
        assert_eq!(parsed1.mods[1].dependencies[0].as_str(), "mod-a");
        assert_eq!(parsed1.mods[1].dependencies[1].as_str(), "mod-z");
    }

    #[test]
    fn test_lockfile_merge() {
        let original = Lockfile {
            version: LOCKFILE_VERSION,
            generated_at: "2025-01-01T00:00:00Z".to_string(),
            engine_version: "0.36.0".to_string(),
            mods: vec![LockedMod {
                id: ModId::new_unchecked("existing-mod"),
                version: Version::new(1, 0, 0),
                sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
                    .to_string(),
                source: "test".to_string(),
                download_url: "https://example.com/existing.plixmod".to_string(),
                dependencies: vec![],
            }],
        };

        let new_resolved = ResolvedGraph {
            mods: vec![
                make_resolved_mod("new-mod", "1.0.0", vec![]),
                make_resolved_mod("existing-mod", "2.0.0", vec![]), // Should not override
            ],
            stats: ResolutionStats::default(),
        };

        let merged = original.merge_with(&new_resolved);

        assert_eq!(merged.mods.len(), 2);

        // Existing mod should keep original version
        let existing = merged
            .find_mod(&ModId::new_unchecked("existing-mod"))
            .unwrap();
        assert_eq!(existing.version, Version::new(1, 0, 0));

        // New mod should be added
        let new_mod = merged.find_mod(&ModId::new_unchecked("new-mod")).unwrap();
        assert_eq!(new_mod.version, Version::new(1, 0, 0));
    }

    #[test]
    fn test_chrono_lite() {
        let timestamp = chrono_lite_now();
        // Should match RFC 3339 format
        assert!(timestamp.contains("T"));
        assert!(timestamp.ends_with("Z"));
        assert_eq!(timestamp.len(), 20); // YYYY-MM-DDTHH:MM:SSZ
    }
}
