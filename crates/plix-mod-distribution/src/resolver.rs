//! Dependency resolution with SemVer constraints, cycle and conflict detection

use crate::config::DistributionConfig;
use crate::errors::DistributionError;
use crate::index::{ModVersionEntry, RegistryIndex};
use crate::lockfile::Lockfile;
use crate::{check_api_compatibility, check_engine_compatibility, current_engine_version, ModId, CURRENT_API_VERSION};
use semver::{Version, VersionReq};
use std::collections::{HashMap, HashSet};

/// Maximum dependency depth to prevent runaway resolution
const MAX_DEPTH: usize = 50;

/// Complete resolution result
#[derive(Debug, Clone)]
pub struct ResolvedGraph {
    /// Resolved mods in installation order (dependencies first)
    pub mods: Vec<ResolvedMod>,

    /// Resolution statistics
    pub stats: ResolutionStats,
}

/// A single resolved mod
#[derive(Debug, Clone)]
pub struct ResolvedMod {
    /// Mod ID
    pub id: ModId,

    /// Resolved version
    pub version: Version,

    /// Source registry
    pub source: String,

    /// Download URL
    pub download_url: String,

    /// SHA-256 hash
    pub sha256: String,

    /// Bundle size in bytes
    pub size: u64,

    /// Direct dependencies (resolved)
    pub dependencies: Vec<ModId>,
}

/// Resolution statistics
#[derive(Debug, Clone, Default)]
pub struct ResolutionStats {
    /// Total mods resolved
    pub total_mods: usize,

    /// Total download size (bytes)
    pub total_size: u64,

    /// Mods already cached
    pub cached_count: usize,

    /// Resolution time (milliseconds)
    pub resolution_time_ms: u64,
}

/// Resolver for mod dependencies
pub struct Resolver<'a> {
    /// Registry indexes by name (in priority order)
    indexes: &'a HashMap<String, RegistryIndex>,

    /// Config with mod requirements
    config: &'a DistributionConfig,

    /// Registry names in priority order
    registry_order: Vec<String>,
}

impl<'a> Resolver<'a> {
    /// Create a new resolver
    pub fn new(indexes: &'a HashMap<String, RegistryIndex>, config: &'a DistributionConfig) -> Self {
        // Build registry order from config
        let registry_order: Vec<String> = config
            .enabled_registries()
            .iter()
            .map(|r| r.name.clone())
            .filter(|name| indexes.contains_key(name))
            .collect();

        Self {
            indexes,
            config,
            registry_order,
        }
    }

    /// Resolve all dependencies
    pub fn resolve(&self, existing_lockfile: Option<&Lockfile>) -> Result<ResolvedGraph, DistributionError> {
        let start = std::time::Instant::now();

        // Build initial requirements from config
        let mut requirements: HashMap<ModId, Vec<(String, VersionReq)>> = HashMap::new();
        for req in &self.config.mods {
            requirements
                .entry(req.id.clone())
                .or_default()
                .push(("config".to_string(), req.version.clone()));
        }

        // Resolved versions: mod_id -> (version, registry_name, version_entry)
        let mut resolved: HashMap<ModId, (Version, String, ModVersionEntry)> = HashMap::new();

        // Track pinned versions from lockfile
        let pinned_versions: HashMap<ModId, Version> = existing_lockfile
            .map(|lf| {
                lf.mods
                    .iter()
                    .map(|m| (m.id.clone(), m.version.clone()))
                    .collect()
            })
            .unwrap_or_default();

        // Queue of mods to process
        let mut queue: Vec<ModId> = self.config.mods.iter().map(|r| r.id.clone()).collect();
        let mut visited: HashSet<ModId> = HashSet::new();

        // Process queue
        while let Some(mod_id) = queue.pop() {
            if visited.contains(&mod_id) {
                continue;
            }
            visited.insert(mod_id.clone());

            // Get all requirements for this mod
            let reqs = requirements.get(&mod_id).cloned().unwrap_or_default();

            // Check if pinned in config
            let is_pinned = self
                .config
                .mods
                .iter()
                .any(|r| r.id == mod_id && r.pinned);

            // Try to use lockfile version if available and valid
            let use_lockfile_version = !is_pinned
                && pinned_versions.get(&mod_id).map_or(false, |v| {
                    reqs.iter().all(|(_, req)| req.matches(v))
                });

            // Find best version
            let (version, registry_name, version_entry) = if use_lockfile_version {
                let pinned_version = pinned_versions.get(&mod_id).unwrap();
                self.find_exact_version(&mod_id, pinned_version)?
            } else {
                self.find_best_version(&mod_id, &reqs)?
            };

            // Check API compatibility
            if !check_api_compatibility(version_entry.api_version, CURRENT_API_VERSION) {
                return Err(DistributionError::api_version_incompatible(
                    mod_id.as_str(),
                    version_entry.api_version,
                    CURRENT_API_VERSION,
                ));
            }

            // Check engine compatibility
            let engine_version = current_engine_version();
            if !check_engine_compatibility(&version_entry.engine, &engine_version) {
                let min = version_entry
                    .engine
                    .as_ref()
                    .and_then(|e| e.min.as_ref().map(|v| v.to_string()));
                let max = version_entry
                    .engine
                    .as_ref()
                    .and_then(|e| e.max.as_ref().map(|v| v.to_string()));
                return Err(DistributionError::engine_version_incompatible(
                    mod_id.as_str(),
                    &engine_version.to_string(),
                    min.as_deref(),
                    max.as_deref(),
                ));
            }

            // Add to resolved
            resolved.insert(
                mod_id.clone(),
                (version.clone(), registry_name, version_entry.clone()),
            );

            // Process dependencies
            for dep in &version_entry.dependencies {
                if dep.optional {
                    continue; // Skip optional dependencies for now
                }

                // Add requirement
                requirements
                    .entry(dep.id.clone())
                    .or_default()
                    .push((mod_id.to_string(), dep.version_req.clone()));

                // Check for conflicts with already resolved versions
                if let Some((resolved_version, _, _)) = resolved.get(&dep.id) {
                    if !dep.version_req.matches(resolved_version) {
                        // Find who else requires this dependency
                        let other_reqs = requirements.get(&dep.id).unwrap();
                        for (source, req) in other_reqs {
                            if req.matches(resolved_version) && source != mod_id.as_str() {
                                return Err(DistributionError::dependency_conflict(
                                    mod_id.as_str(),
                                    source,
                                    dep.id.as_str(),
                                    &dep.version_req.to_string(),
                                    &req.to_string(),
                                ));
                            }
                        }
                    }
                }

                // Add to queue if not visited
                if !visited.contains(&dep.id) {
                    queue.push(dep.id.clone());
                }
            }

            // Check depth limit
            if visited.len() > MAX_DEPTH {
                return Err(DistributionError::internal(&format!(
                    "Dependency resolution exceeded maximum depth of {}",
                    MAX_DEPTH
                )));
            }
        }

        // Detect cycles
        self.detect_cycles(&resolved)?;

        // Build resolved graph in topological order
        let ordered = self.topological_sort(&resolved)?;

        let stats = ResolutionStats {
            total_mods: ordered.len(),
            total_size: ordered.iter().map(|m| m.size).sum(),
            cached_count: 0, // Will be updated during download
            resolution_time_ms: start.elapsed().as_millis() as u64,
        };

        tracing::info!(
            "Resolved {} mods in {}ms",
            stats.total_mods,
            stats.resolution_time_ms
        );

        Ok(ResolvedGraph {
            mods: ordered,
            stats,
        })
    }

    /// Find the best version of a mod matching all requirements
    fn find_best_version(
        &self,
        mod_id: &ModId,
        requirements: &[(String, VersionReq)],
    ) -> Result<(Version, String, ModVersionEntry), DistributionError> {
        // Search registries in priority order
        for registry_name in &self.registry_order {
            if let Some(index) = self.indexes.get(registry_name) {
                if let Some(mod_entry) = index.find_mod(mod_id) {
                    // Find versions matching all requirements
                    let matching: Vec<_> = mod_entry
                        .versions
                        .iter()
                        .filter(|v| {
                            !v.yanked
                                && requirements.iter().all(|(_, req)| req.matches(&v.version))
                        })
                        .collect();

                    if !matching.is_empty() {
                        // Select newest matching version
                        let best = matching
                            .into_iter()
                            .max_by(|a, b| a.version.cmp(&b.version))
                            .unwrap();

                        return Ok((
                            best.version.clone(),
                            registry_name.clone(),
                            best.clone(),
                        ));
                    }
                }
            }
        }

        // Not found in any registry
        if requirements.is_empty() {
            Err(DistributionError::mod_not_found(mod_id.as_str()))
        } else {
            let req_strs: Vec<_> = requirements.iter().map(|(_, r)| r.to_string()).collect();
            Err(DistributionError::version_not_found(
                mod_id.as_str(),
                &req_strs.join(", "),
            ))
        }
    }

    /// Find an exact version in any registry
    fn find_exact_version(
        &self,
        mod_id: &ModId,
        version: &Version,
    ) -> Result<(Version, String, ModVersionEntry), DistributionError> {
        for registry_name in &self.registry_order {
            if let Some(index) = self.indexes.get(registry_name) {
                if let Some(version_entry) = index.find_version(mod_id, version) {
                    return Ok((
                        version.clone(),
                        registry_name.clone(),
                        version_entry.clone(),
                    ));
                }
            }
        }

        Err(DistributionError::version_not_found(
            mod_id.as_str(),
            &format!("={}", version),
        ))
    }

    /// Detect dependency cycles using DFS
    fn detect_cycles(
        &self,
        resolved: &HashMap<ModId, (Version, String, ModVersionEntry)>,
    ) -> Result<(), DistributionError> {
        #[derive(Clone, Copy, PartialEq)]
        enum State {
            Unvisited,
            InProgress,
            Done,
        }

        let mut states: HashMap<&ModId, State> = resolved.keys().map(|k| (k, State::Unvisited)).collect();
        let mut path: Vec<String> = Vec::new();

        fn dfs<'a>(
            node: &'a ModId,
            resolved: &'a HashMap<ModId, (Version, String, ModVersionEntry)>,
            states: &mut HashMap<&'a ModId, State>,
            path: &mut Vec<String>,
        ) -> Option<Vec<String>> {
            match states.get(&node) {
                Some(State::InProgress) => {
                    // Found cycle
                    let mut cycle = path.clone();
                    cycle.push(node.to_string());
                    return Some(cycle);
                }
                Some(State::Done) => return None,
                _ => {}
            }

            states.insert(node, State::InProgress);
            path.push(node.to_string());

            if let Some((_, _, version_entry)) = resolved.get(node) {
                for dep in &version_entry.dependencies {
                    if !dep.optional {
                        if let Some(cycle) = dfs(&dep.id, resolved, states, path) {
                            return Some(cycle);
                        }
                    }
                }
            }

            path.pop();
            states.insert(node, State::Done);
            None
        }

        for node in resolved.keys() {
            if states.get(node) == Some(&State::Unvisited) {
                if let Some(cycle) = dfs(node, resolved, &mut states, &mut path) {
                    return Err(DistributionError::cycle_detected(&cycle));
                }
            }
        }

        Ok(())
    }

    /// Sort resolved mods in topological order (dependencies first)
    fn topological_sort(
        &self,
        resolved: &HashMap<ModId, (Version, String, ModVersionEntry)>,
    ) -> Result<Vec<ResolvedMod>, DistributionError> {
        let mut result: Vec<ResolvedMod> = Vec::new();
        let mut visited: HashSet<ModId> = HashSet::new();

        fn visit(
            mod_id: &ModId,
            resolved: &HashMap<ModId, (Version, String, ModVersionEntry)>,
            visited: &mut HashSet<ModId>,
            result: &mut Vec<ResolvedMod>,
            indexes: &HashMap<String, RegistryIndex>,
        ) {
            if visited.contains(mod_id) {
                return;
            }
            visited.insert(mod_id.clone());

            if let Some((version, registry_name, version_entry)) = resolved.get(mod_id) {
                // Visit dependencies first
                for dep in &version_entry.dependencies {
                    if !dep.optional {
                        visit(&dep.id, resolved, visited, result, indexes);
                    }
                }

                // Resolve full download URL
                let download_url = indexes
                    .get(registry_name)
                    .map(|idx| idx.resolve_download_url(version_entry))
                    .unwrap_or_else(|| version_entry.download_url.clone());

                // Add this mod
                result.push(ResolvedMod {
                    id: mod_id.clone(),
                    version: version.clone(),
                    source: registry_name.clone(),
                    download_url,
                    sha256: version_entry.sha256.clone(),
                    size: version_entry.size,
                    dependencies: version_entry
                        .dependencies
                        .iter()
                        .filter(|d| !d.optional)
                        .map(|d| d.id.clone())
                        .collect(),
                });
            }
        }

        for mod_id in resolved.keys() {
            visit(mod_id, resolved, &mut visited, &mut result, self.indexes);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ModRequirement;
    use crate::index::RegistryMod;
    use crate::ModDependency;

    fn make_version(
        version: &str,
        deps: Vec<(&str, &str)>,
    ) -> ModVersionEntry {
        ModVersionEntry {
            version: Version::parse(version).unwrap(),
            sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(),
            download_url: format!("test-{}.plixmod", version),
            size: 1000,
            dependencies: deps
                .into_iter()
                .map(|(id, req)| ModDependency {
                    id: ModId::new_unchecked(id),
                    version_req: VersionReq::parse(req).unwrap(),
                    optional: false,
                })
                .collect(),
            api_version: 1,
            engine: None,
            signature: None,
            signer: None,
            published_at: None,
            yanked: false,
        }
    }

    fn make_mod(id: &str, versions: Vec<ModVersionEntry>) -> RegistryMod {
        RegistryMod {
            id: ModId::new_unchecked(id),
            name: id.to_string(),
            description: None,
            author: None,
            homepage: None,
            versions,
        }
    }

    fn make_index(mods: Vec<RegistryMod>) -> RegistryIndex {
        RegistryIndex {
            registry_version: 1,
            name: "test".to_string(),
            base_url: None,
            updated_at: None,
            mods,
        }
    }

    #[test]
    fn test_simple_resolution() {
        let index = make_index(vec![
            make_mod("mod-a", vec![make_version("1.0.0", vec![])]),
        ]);

        let mut indexes = HashMap::new();
        indexes.insert("test".to_string(), index);

        let config = DistributionConfig {
            registries: vec![crate::config::RegistryConfig {
                name: "test".to_string(),
                url: "/test".to_string(),
                priority: 1,
                enabled: true,
            }],
            mods: vec![ModRequirement {
                id: ModId::new_unchecked("mod-a"),
                version: VersionReq::parse("^1.0").unwrap(),
                pinned: false,
                optional: false,
            }],
            ..Default::default()
        };

        let resolver = Resolver::new(&indexes, &config);
        let result = resolver.resolve(None).unwrap();

        assert_eq!(result.mods.len(), 1);
        assert_eq!(result.mods[0].id.as_str(), "mod-a");
        assert_eq!(result.mods[0].version, Version::new(1, 0, 0));
    }

    #[test]
    fn test_transitive_dependencies() {
        // mod-a -> mod-b -> mod-c
        let index = make_index(vec![
            make_mod("mod-a", vec![make_version("1.0.0", vec![("mod-b", "^1.0")])]),
            make_mod("mod-b", vec![make_version("1.0.0", vec![("mod-c", "^2.0")])]),
            make_mod("mod-c", vec![make_version("2.0.0", vec![])]),
        ]);

        let mut indexes = HashMap::new();
        indexes.insert("test".to_string(), index);

        let config = DistributionConfig {
            registries: vec![crate::config::RegistryConfig {
                name: "test".to_string(),
                url: "/test".to_string(),
                priority: 1,
                enabled: true,
            }],
            mods: vec![ModRequirement {
                id: ModId::new_unchecked("mod-a"),
                version: VersionReq::parse("^1.0").unwrap(),
                pinned: false,
                optional: false,
            }],
            ..Default::default()
        };

        let resolver = Resolver::new(&indexes, &config);
        let result = resolver.resolve(None).unwrap();

        assert_eq!(result.mods.len(), 3);

        // Check topological order: mod-c should come before mod-b, mod-b before mod-a
        let positions: HashMap<_, _> = result
            .mods
            .iter()
            .enumerate()
            .map(|(i, m)| (m.id.as_str(), i))
            .collect();

        assert!(positions["mod-c"] < positions["mod-b"]);
        assert!(positions["mod-b"] < positions["mod-a"]);
    }

    #[test]
    fn test_cycle_detection() {
        // mod-a -> mod-b -> mod-a (cycle)
        let index = make_index(vec![
            make_mod("mod-a", vec![make_version("1.0.0", vec![("mod-b", "^1.0")])]),
            make_mod("mod-b", vec![make_version("1.0.0", vec![("mod-a", "^1.0")])]),
        ]);

        let mut indexes = HashMap::new();
        indexes.insert("test".to_string(), index);

        let config = DistributionConfig {
            registries: vec![crate::config::RegistryConfig {
                name: "test".to_string(),
                url: "/test".to_string(),
                priority: 1,
                enabled: true,
            }],
            mods: vec![ModRequirement {
                id: ModId::new_unchecked("mod-a"),
                version: VersionReq::parse("^1.0").unwrap(),
                pinned: false,
                optional: false,
            }],
            ..Default::default()
        };

        let resolver = Resolver::new(&indexes, &config);
        let result = resolver.resolve(None);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, crate::errors::ErrorCode::CycleDetected);
    }

    #[test]
    fn test_latest_compatible_selection() {
        let index = make_index(vec![make_mod(
            "mod-a",
            vec![
                make_version("1.0.0", vec![]),
                make_version("1.1.0", vec![]),
                make_version("1.2.0", vec![]),
                make_version("2.0.0", vec![]),
            ],
        )]);

        let mut indexes = HashMap::new();
        indexes.insert("test".to_string(), index);

        let config = DistributionConfig {
            registries: vec![crate::config::RegistryConfig {
                name: "test".to_string(),
                url: "/test".to_string(),
                priority: 1,
                enabled: true,
            }],
            mods: vec![ModRequirement {
                id: ModId::new_unchecked("mod-a"),
                version: VersionReq::parse("^1.0").unwrap(),
                pinned: false,
                optional: false,
            }],
            ..Default::default()
        };

        let resolver = Resolver::new(&indexes, &config);
        let result = resolver.resolve(None).unwrap();

        // Should select 1.2.0 (latest compatible with ^1.0)
        assert_eq!(result.mods[0].version, Version::new(1, 2, 0));
    }

    #[test]
    fn test_mod_not_found() {
        let index = make_index(vec![]);

        let mut indexes = HashMap::new();
        indexes.insert("test".to_string(), index);

        let config = DistributionConfig {
            registries: vec![crate::config::RegistryConfig {
                name: "test".to_string(),
                url: "/test".to_string(),
                priority: 1,
                enabled: true,
            }],
            mods: vec![ModRequirement {
                id: ModId::new_unchecked("nonexistent"),
                version: VersionReq::parse("^1.0").unwrap(),
                pinned: false,
                optional: false,
            }],
            ..Default::default()
        };

        let resolver = Resolver::new(&indexes, &config);
        let result = resolver.resolve(None);

        assert!(result.is_err());
    }
}
