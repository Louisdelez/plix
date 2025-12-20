//! Server-side mod integration
//!
//! This module handles loading mods from the filesystem and integrating
//! them with the server game loop.

pub mod distribution;
pub mod pending_connection;
pub mod sync_session;
pub mod wasm_bridge;

pub use distribution::{init_mod_distribution, is_distribution_configured, DistributionStats};
pub use pending_connection::PendingModConnection;
pub use sync_session::{ModInfo, SyncSession, SyncState};
pub use wasm_bridge::WasmBridge;

use plix_mod_core::{
    events::{EventBus, GameEvent},
    manifest::ModManifest,
    observability::ModMetrics,
    registry::ModRegistry,
    ModApiError,
};
use plix_mod_runtime_wasm::RuntimeConfig;
use std::path::{Path, PathBuf};
use tracing::{error, info, warn};

/// Directory name for mods
pub const MODS_DIRECTORY: &str = "mods";

/// Mod loader and manager for the server
pub struct ModManager {
    /// Mod registry
    pub registry: ModRegistry,
    /// Event bus for mod events
    pub event_bus: EventBus,
    /// Metrics tracking
    pub metrics: ModMetrics,
    /// Mods directory path
    mods_path: PathBuf,
    /// WASM runtime bridge (optional, created on demand)
    wasm_bridge: Option<WasmBridge>,
}

impl ModManager {
    /// Create a new mod manager
    pub fn new(server_root: impl AsRef<Path>) -> Self {
        let mods_path = server_root.as_ref().join(MODS_DIRECTORY);

        Self {
            registry: ModRegistry::new(),
            event_bus: EventBus::new(),
            metrics: ModMetrics::new(),
            mods_path,
            wasm_bridge: None,
        }
    }

    /// Create a new mod manager with WASM runtime enabled
    pub fn with_wasm_runtime(server_root: impl AsRef<Path>) -> Self {
        let mods_path = server_root.as_ref().join(MODS_DIRECTORY);

        let wasm_bridge = match WasmBridge::new() {
            Ok(bridge) => {
                info!("WASM mod runtime initialized");
                Some(bridge)
            }
            Err(e) => {
                error!(error = %e, "Failed to initialize WASM runtime");
                None
            }
        };

        Self {
            registry: ModRegistry::new(),
            event_bus: EventBus::new(),
            metrics: ModMetrics::new(),
            mods_path,
            wasm_bridge,
        }
    }

    /// Create a new mod manager with custom WASM runtime config
    pub fn with_wasm_config(server_root: impl AsRef<Path>, config: RuntimeConfig) -> Self {
        let mods_path = server_root.as_ref().join(MODS_DIRECTORY);

        let wasm_bridge = match WasmBridge::with_config(config) {
            Ok(bridge) => {
                info!("WASM mod runtime initialized with custom config");
                Some(bridge)
            }
            Err(e) => {
                error!(error = %e, "Failed to initialize WASM runtime");
                None
            }
        };

        Self {
            registry: ModRegistry::new(),
            event_bus: EventBus::new(),
            metrics: ModMetrics::new(),
            mods_path,
            wasm_bridge,
        }
    }

    /// Load all mods from the mods directory
    pub fn load_mods(&mut self) -> Result<usize, ModApiError> {
        if !self.mods_path.exists() {
            info!(path = %self.mods_path.display(), "Mods directory does not exist, skipping mod loading");
            return Ok(0);
        }

        let mut loaded_count = 0;

        // Read directory entries
        let entries = match std::fs::read_dir(&self.mods_path) {
            Ok(entries) => entries,
            Err(e) => {
                error!(error = %e, path = %self.mods_path.display(), "Failed to read mods directory");
                return Ok(0);
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();

            // Skip non-directories
            if !path.is_dir() {
                continue;
            }

            // Look for mod.toml
            let manifest_path = path.join("mod.toml");
            if !manifest_path.exists() {
                warn!(path = %path.display(), "Directory in mods folder has no mod.toml, skipping");
                continue;
            }

            // Load the manifest
            match ModManifest::load_from_file(&manifest_path) {
                Ok(manifest) => {
                    let mod_id = manifest.id.clone();
                    let mod_name = manifest.name.clone();
                    let mod_version = manifest.version.clone();

                    // First, try to load as WASM mod if we have a WASM bridge
                    let wasm_loaded = if let Some(ref mut bridge) = self.wasm_bridge {
                        match bridge.load_wasm_mod(&manifest, &path) {
                            Ok(()) => bridge.is_wasm_mod_loaded(&mod_id),
                            Err(e) => {
                                warn!(
                                    mod_id = %mod_id,
                                    error = %e,
                                    "Failed to load WASM mod"
                                );
                                false
                            }
                        }
                    } else {
                        false
                    };

                    // Register in the manifest registry
                    match self.registry.load(manifest) {
                        Ok(()) => {
                            info!(
                                mod_id = %mod_id,
                                name = %mod_name,
                                version = %mod_version,
                                wasm = wasm_loaded,
                                "Mod loaded"
                            );
                            self.metrics.record_mod_loaded();
                            loaded_count += 1;
                        }
                        Err(e) => {
                            error!(
                                mod_id = %mod_id,
                                error = %e,
                                "Failed to load mod"
                            );
                        }
                    }
                }
                Err(e) => {
                    error!(
                        path = %manifest_path.display(),
                        error = %e,
                        "Failed to parse mod manifest"
                    );
                }
            }
        }

        let wasm_count = self
            .wasm_bridge
            .as_ref()
            .map(|b| b.wasm_mod_count())
            .unwrap_or(0);
        info!(count = loaded_count, wasm_count = wasm_count, "Mods loaded");
        Ok(loaded_count)
    }

    /// Emit a game event for mods to handle
    pub fn emit_event(&mut self, event: GameEvent) {
        self.event_bus.emit(event);
    }

    /// Dispatch all pending events to mods
    ///
    /// Call this at the end of each game tick.
    /// Note: WASM mods are dispatched via the `dispatch_event` method for individual events
    /// since the EventBus dispatch handles registry-based mods with handlers.
    pub fn dispatch_events(&mut self) -> usize {
        // The EventBus.dispatch() handles registry-based mods through the handler mechanism.
        // WASM mods should use dispatch_event() for individual event dispatch.
        let cancelled = self.event_bus.dispatch(&mut self.registry);

        // Record metrics
        let snapshot = self.metrics.snapshot();
        if snapshot.events_dispatched > 0 {
            self.metrics.record_event_dispatch();
        }

        cancelled
    }

    /// Dispatch a single event immediately to all mods
    ///
    /// Returns true if the event was cancelled by any mod.
    pub fn dispatch_event(&mut self, event: &GameEvent) -> bool {
        // Dispatch to WASM mods
        if let Some(ref mut bridge) = self.wasm_bridge {
            if bridge.dispatch_to_wasm_mods(event) {
                return true;
            }
        }

        false
    }

    /// Unload a specific mod
    pub fn unload_mod(&mut self, mod_id: &str) -> Result<(), ModApiError> {
        // Unload from WASM runtime if present
        if let Some(ref mut bridge) = self.wasm_bridge {
            bridge.unload_wasm_mod(mod_id)?;
        }

        self.registry.unload(mod_id)?;
        info!(mod_id = %mod_id, "Mod unloaded");
        Ok(())
    }

    /// Shutdown all mods
    ///
    /// Call this when the server is stopping to ensure all mods
    /// have their shutdown handlers called.
    pub fn shutdown(&mut self) {
        // Shutdown WASM mods first
        if let Some(ref mut bridge) = self.wasm_bridge {
            bridge.shutdown();
        }

        // Clear the registry
        self.registry = ModRegistry::new();

        info!("All mods shut down");
    }

    /// Get the number of loaded mods
    pub fn mod_count(&self) -> usize {
        self.registry.mod_count()
    }

    /// Get the number of active mods
    pub fn active_mod_count(&self) -> usize {
        self.registry.active_mod_count()
    }

    /// Check if a mod is loaded
    pub fn is_mod_loaded(&self, mod_id: &str) -> bool {
        self.registry.is_loaded(mod_id)
    }

    /// Get metrics snapshot
    pub fn metrics_snapshot(&self) -> plix_mod_core::observability::MetricsSnapshot {
        self.metrics.snapshot()
    }

    /// Check if WASM runtime is available
    pub fn has_wasm_runtime(&self) -> bool {
        self.wasm_bridge.is_some()
    }

    /// Get the number of WASM mods loaded
    pub fn wasm_mod_count(&self) -> usize {
        self.wasm_bridge
            .as_ref()
            .map(|b| b.wasm_mod_count())
            .unwrap_or(0)
    }

    /// Get WASM metrics for a specific mod
    pub fn wasm_mod_metrics(&self, mod_id: &str) -> Option<&plix_mod_runtime_wasm::WasmModMetrics> {
        self.wasm_bridge
            .as_ref()
            .and_then(|b| b.get_wasm_mod_metrics(mod_id))
    }

    /// Get information about mods with client payloads (Feature 037)
    ///
    /// Returns ModInfo for each loaded mod that has a client payload.
    /// This is used to build the ModSetDescriptor for client sync.
    pub fn mods_with_client_payloads(&self) -> Vec<ModInfo> {
        let mut result = Vec::new();

        for manifest in self.registry.list_all() {
            // Check if mod has client payload - we look for client_payload_hash in the manifest
            // The manifest stores client_payload info when the mod is built with a client data directory
            if let Some(payload_hash) = self.get_mod_client_payload_hash(&manifest.id) {
                let payload_path = self.mods_path.join(&manifest.id).join("client_data.zip");
                let payload_size = std::fs::metadata(&payload_path)
                    .map(|m| m.len())
                    .unwrap_or(0);

                if payload_size > 0 {
                    result.push(ModInfo {
                        id: manifest.id.clone(),
                        version: manifest.version.to_string(),
                        client_payload_hash: payload_hash,
                        client_payload_size: payload_size,
                        payload_path,
                        has_network_channels: !manifest.network.custom_channels.is_empty(),
                    });
                }
            }
        }

        result
    }

    /// Get the client payload hash for a mod (if it has one)
    fn get_mod_client_payload_hash(&self, mod_id: &str) -> Option<String> {
        // Look for a .hash file alongside the client_data.zip
        let hash_path = self.mods_path.join(mod_id).join("client_data.sha256");
        if hash_path.exists() {
            std::fs::read_to_string(&hash_path)
                .ok()
                .map(|s| s.trim().to_string())
        } else {
            None
        }
    }
}

impl Default for ModManager {
    fn default() -> Self {
        Self::new(".")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_mod_manager_new() {
        let manager = ModManager::new("/tmp/test-server");
        assert_eq!(manager.mod_count(), 0);
    }

    #[test]
    fn test_load_mods_empty_dir() {
        let temp = tempdir().unwrap();
        let mods_dir = temp.path().join("mods");
        fs::create_dir(&mods_dir).unwrap();

        let mut manager = ModManager::new(temp.path());
        let count = manager.load_mods().unwrap();

        assert_eq!(count, 0);
    }

    #[test]
    fn test_load_mods_with_valid_mod() {
        let temp = tempdir().unwrap();
        let mods_dir = temp.path().join("mods");
        let mod_dir = mods_dir.join("test-mod");
        fs::create_dir_all(&mod_dir).unwrap();

        let manifest = r#"
id = "test-mod"
name = "Test Mod"
version = "1.0.0"
api_version = 1
"#;
        fs::write(mod_dir.join("mod.toml"), manifest).unwrap();

        let mut manager = ModManager::new(temp.path());
        let count = manager.load_mods().unwrap();

        assert_eq!(count, 1);
        assert!(manager.is_mod_loaded("test-mod"));
    }

    #[test]
    fn test_load_mods_skips_invalid_manifest() {
        let temp = tempdir().unwrap();
        let mods_dir = temp.path().join("mods");
        let mod_dir = mods_dir.join("bad-mod");
        fs::create_dir_all(&mod_dir).unwrap();

        // Invalid manifest (missing required fields)
        let manifest = r#"
name = "Bad Mod"
"#;
        fs::write(mod_dir.join("mod.toml"), manifest).unwrap();

        let mut manager = ModManager::new(temp.path());
        let count = manager.load_mods().unwrap();

        assert_eq!(count, 0);
    }

    #[test]
    fn test_load_mods_nonexistent_dir() {
        let temp = tempdir().unwrap();
        // Don't create mods directory

        let mut manager = ModManager::new(temp.path());
        let count = manager.load_mods().unwrap();

        assert_eq!(count, 0);
    }

    #[test]
    fn test_emit_and_dispatch() {
        let temp = tempdir().unwrap();
        let mut manager = ModManager::new(temp.path());

        // Emit some events
        manager.emit_event(GameEvent::server_start(1));
        manager.emit_event(GameEvent::player_join(1, "Player1", 2));

        // Dispatch (no mods loaded, so nothing happens)
        let cancelled = manager.dispatch_events();
        assert_eq!(cancelled, 0);
    }
}
