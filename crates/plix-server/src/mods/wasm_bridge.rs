//! WASM mod runtime bridge
//!
//! Integrates plix-mod-runtime-wasm with the server's ModManager.
//! This bridge handles loading WASM modules and routing events to them.

use plix_mod_core::{
    errors::{err_invalid, ErrorCode},
    events::GameEvent,
    manifest::ModManifest,
    Capability, ModApiError,
};
use plix_mod_runtime_wasm::{RuntimeConfig, RuntimeError, WasmRuntime};
use std::path::Path;
use tracing::{debug, error, info, warn};

/// Bridge between the ModManager and WASM runtime
pub struct WasmBridge {
    /// The WASM runtime
    runtime: WasmRuntime,
    /// Configuration
    config: RuntimeConfig,
}

impl WasmBridge {
    /// Create a new WASM bridge with default configuration
    pub fn new() -> Result<Self, RuntimeError> {
        Self::with_config(RuntimeConfig::default())
    }

    /// Create a new WASM bridge with custom configuration
    pub fn with_config(config: RuntimeConfig) -> Result<Self, RuntimeError> {
        let runtime = WasmRuntime::new(config.clone())?;

        Ok(Self { runtime, config })
    }

    /// Load a WASM mod from its manifest
    ///
    /// This reads the mod.wasm file from the mod directory and loads it
    /// into the WASM runtime with the capabilities from the manifest.
    pub fn load_wasm_mod(
        &mut self,
        manifest: &ModManifest,
        mod_path: &Path,
    ) -> Result<(), ModApiError> {
        let wasm_path = mod_path.join("mod.wasm");

        if !wasm_path.exists() {
            // Not a WASM mod - that's OK, might be a different type
            debug!(
                mod_id = %manifest.id,
                "No mod.wasm found, skipping WASM loading"
            );
            return Ok(());
        }

        // Read the WASM bytes
        let wasm_bytes = match std::fs::read(&wasm_path) {
            Ok(bytes) => bytes,
            Err(e) => {
                error!(
                    mod_id = %manifest.id,
                    path = %wasm_path.display(),
                    error = %e,
                    "Failed to read mod.wasm"
                );
                return Err(err_invalid(format!("Failed to read mod.wasm: {}", e)));
            }
        };

        // Get capabilities from manifest
        let capabilities = manifest_to_capabilities(manifest);

        // Load into the WASM runtime
        match self
            .runtime
            .load_mod(&manifest.id, &wasm_bytes, capabilities)
        {
            Ok(()) => {
                info!(
                    mod_id = %manifest.id,
                    name = %manifest.name,
                    version = %manifest.version,
                    capabilities = ?capabilities,
                    "WASM mod loaded"
                );
                Ok(())
            }
            Err(e) => {
                error!(
                    mod_id = %manifest.id,
                    error = %e,
                    "Failed to load WASM mod"
                );
                Err(ModApiError::new(
                    ErrorCode::Unsupported,
                    format!("WASM load error: {}", e),
                ))
            }
        }
    }

    /// Unload a WASM mod
    pub fn unload_wasm_mod(&mut self, mod_id: &str) -> Result<(), ModApiError> {
        if !self.runtime.is_mod_loaded(mod_id) {
            return Ok(()); // Not a WASM mod or already unloaded
        }

        self.runtime.unload_mod(mod_id).map_err(|e| {
            error!(mod_id = %mod_id, error = %e, "Failed to unload WASM mod");
            ModApiError::new(ErrorCode::Unsupported, format!("WASM unload error: {}", e))
        })?;

        info!(mod_id = %mod_id, "WASM mod unloaded");
        Ok(())
    }

    /// Dispatch an event to all WASM mods
    ///
    /// Returns true if any mod cancelled the event.
    pub fn dispatch_to_wasm_mods(&mut self, event: &GameEvent) -> bool {
        // Convert event type to u8 event ID
        let event_id = event_type_to_id(&event.event_type);

        // Serialize the payload
        let payload = match serialize_event_payload(event) {
            Ok(bytes) => bytes,
            Err(e) => {
                warn!(error = %e, "Failed to serialize event payload");
                return false;
            }
        };

        // Dispatch to WASM runtime
        match self
            .runtime
            .dispatch_game_event(event_id, &payload, event.clone())
        {
            Ok(cancelled) => cancelled,
            Err(e) => {
                warn!(error = %e, "Error dispatching event to WASM mods");
                false
            }
        }
    }

    /// Check if a WASM mod is loaded
    pub fn is_wasm_mod_loaded(&self, mod_id: &str) -> bool {
        self.runtime.is_mod_loaded(mod_id)
    }

    /// Get the number of loaded WASM mods
    pub fn wasm_mod_count(&self) -> usize {
        self.runtime.mod_count()
    }

    /// Get the WASM runtime configuration
    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }

    /// Get metrics for a specific WASM mod
    pub fn get_wasm_mod_metrics(
        &self,
        mod_id: &str,
    ) -> Option<&plix_mod_runtime_wasm::WasmModMetrics> {
        self.runtime.get_mod_metrics(mod_id)
    }

    /// Shutdown all WASM mods
    ///
    /// Call this when the server is stopping to ensure all mods
    /// have their mod_shutdown functions called.
    pub fn shutdown(&mut self) {
        let mod_ids: Vec<String> = self
            .runtime
            .mod_ids()
            .iter()
            .map(|s| s.to_string())
            .collect();

        for mod_id in mod_ids {
            if let Err(e) = self.runtime.unload_mod(&mod_id) {
                warn!(mod_id = %mod_id, error = %e, "Error during mod shutdown");
            }
        }

        info!("All WASM mods shut down");
    }
}

impl Default for WasmBridge {
    fn default() -> Self {
        Self::new().expect("Failed to create default WasmBridge")
    }
}

/// Convert a ModManifest's capabilities to plix_mod_core::Capability
fn manifest_to_capabilities(manifest: &ModManifest) -> Capability {
    // Use the ManifestCapabilities' built-in conversion method
    manifest.capabilities.to_capability()
}

/// Convert EventType to ABI event ID
fn event_type_to_id(event_type: &plix_mod_core::events::EventType) -> u8 {
    use plix_mod_core::events::EventType;

    match event_type {
        EventType::ServerStart => 0x01,
        EventType::ServerStop => 0x02,
        EventType::PlayerJoin => 0x03,
        EventType::PlayerLeave => 0x04,
        EventType::PlayerChat => 0x05,
        EventType::BlockPlaced => 0x06,
        EventType::BlockBroken => 0x07,
        EventType::EntityDamaged => 0x08,
        EventType::ModMessage => 0x09,
    }
}

/// Serialize event payload to bytes
fn serialize_event_payload(event: &GameEvent) -> Result<Vec<u8>, String> {
    // For now, use bincode to serialize the payload
    bincode::serialize(&event.payload).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_bridge_new() {
        let bridge = WasmBridge::new();
        assert!(bridge.is_ok());

        let bridge = bridge.unwrap();
        assert_eq!(bridge.wasm_mod_count(), 0);
    }

    #[test]
    fn test_manifest_to_capabilities() {
        use plix_mod_core::capabilities::ManifestCapabilities;
        use plix_mod_core::manifest::Entrypoints;

        let manifest = ModManifest {
            id: "test".to_string(),
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            author: None,
            api_version: 1,
            min_api_version: None,
            max_api_version: None,
            capabilities: ManifestCapabilities {
                world_read: true,
                event_cancel_chat: true,
                ..Default::default()
            },
            entrypoints: Entrypoints::default(),
            dependencies: Vec::new(),
        };

        let caps = manifest_to_capabilities(&manifest);
        assert!(caps.contains(Capability::WORLD_READ));
        assert!(caps.contains(Capability::EVENT_CANCEL_CHAT));
        assert!(!caps.contains(Capability::WORLD_WRITE));
    }

    #[test]
    fn test_event_type_to_id() {
        use plix_mod_core::events::EventType;

        assert_eq!(event_type_to_id(&EventType::PlayerChat), 0x05);
        assert_eq!(event_type_to_id(&EventType::BlockPlaced), 0x06);
        assert_eq!(event_type_to_id(&EventType::ServerStart), 0x01);
    }
}
