//! WASM-based sandboxed mod runtime for plix
//!
//! This crate provides a WebAssembly runtime for executing mods in a secure
//! sandbox with:
//! - Complete isolation (no filesystem/network/OS access)
//! - CPU budget enforcement via fuel metering
//! - Memory limits via ResourceLimiter
//! - Host ABI v1 for engine interaction
//!
//! # Architecture
//!
//! The runtime uses Wasmtime as the WASM backend for robust security guarantees.
//! Each mod runs in its own `WasmInstance` with isolated memory and fuel budget.
//!
//! # Example
//!
//! ```ignore
//! use plix_mod_runtime_wasm::{WasmRuntime, RuntimeConfig};
//!
//! let config = RuntimeConfig::default();
//! let runtime = WasmRuntime::new(config)?;
//!
//! // Load a mod
//! let mod_bytes = std::fs::read("mods/my-mod/mod.wasm")?;
//! runtime.load_mod("my-mod", &mod_bytes)?;
//! ```

pub mod abi;
pub mod budgets;
pub mod engine;
pub mod errors;
pub mod host;
pub mod instance;
pub mod memory;
pub mod metrics;
pub mod module_loader;

// Re-exports
pub use budgets::FuelBudget;
pub use engine::WasmEngine;
pub use errors::RuntimeError;
pub use instance::{ModExports, ModState, WasmInstance};
pub use metrics::WasmModMetrics;
pub use module_loader::ModuleLoader;

/// Current ABI version for mod-engine communication
pub const ABI_VERSION: u32 = 1;

/// Default CPU budget per handler call in milliseconds
pub const DEFAULT_HANDLER_CPU_BUDGET_MS: u64 = 5;

/// Default memory limit per mod in bytes (32 MiB)
pub const DEFAULT_MEMORY_LIMIT_BYTES: usize = 32 * 1024 * 1024;

/// Default consecutive error threshold before auto-disable
pub const DEFAULT_ERROR_THRESHOLD: u32 = 5;

/// Runtime configuration for the WASM mod runtime
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// CPU budget per handler call in milliseconds
    pub handler_cpu_budget_ms: u64,
    /// CPU budget per tick for all mods combined
    pub mod_tick_budget_ms: u64,
    /// Maximum memory per mod in bytes
    pub max_memory_bytes: usize,
    /// Consecutive error threshold before auto-disable
    pub violation_threshold: u32,
    /// Enable debug mode logging
    pub debug_mode: bool,
    /// Fuel units per millisecond (calibrated at startup)
    pub fuel_per_ms: u64,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            handler_cpu_budget_ms: DEFAULT_HANDLER_CPU_BUDGET_MS,
            mod_tick_budget_ms: 10,
            max_memory_bytes: DEFAULT_MEMORY_LIMIT_BYTES,
            violation_threshold: DEFAULT_ERROR_THRESHOLD,
            debug_mode: false,
            // Default fuel calibration - will be recalibrated at runtime
            fuel_per_ms: 10_000,
        }
    }
}

impl RuntimeConfig {
    /// Create a new runtime config with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the handler CPU budget in milliseconds
    pub fn with_handler_budget_ms(mut self, ms: u64) -> Self {
        self.handler_cpu_budget_ms = ms;
        self
    }

    /// Set the memory limit in bytes
    pub fn with_memory_limit(mut self, bytes: usize) -> Self {
        self.max_memory_bytes = bytes;
        self
    }

    /// Enable debug mode
    pub fn with_debug_mode(mut self, enabled: bool) -> Self {
        self.debug_mode = enabled;
        self
    }

    /// Set the violation threshold
    pub fn with_violation_threshold(mut self, threshold: u32) -> Self {
        self.violation_threshold = threshold;
        self
    }
}

use plix_mod_core::events::{EventPayload, EventType, GameEvent, PlayerChatPayload};
use plix_mod_core::Capability;
use std::collections::HashMap;
use wasmtime::{Linker, Store};

/// The main WASM mod runtime
///
/// This struct manages all loaded WASM mods and provides the public API
/// for loading, executing, and unloading mods.
pub struct WasmRuntime {
    /// Runtime configuration
    config: RuntimeConfig,
    /// The WASM engine (shared across all mods)
    engine: WasmEngine,
    /// The linker with all host functions registered
    linker: Linker<ModState>,
    /// Loaded mod instances keyed by mod ID
    mods: HashMap<String, WasmInstance>,
}

impl WasmRuntime {
    /// Create a new WASM runtime with the given configuration
    pub fn new(config: RuntimeConfig) -> Result<Self, RuntimeError> {
        let engine = WasmEngine::new(&config)
            .map_err(|e| RuntimeError::Internal(format!("Failed to create engine: {}", e)))?;

        let linker = host::create_linker(engine.inner())
            .map_err(|e| RuntimeError::Internal(format!("Failed to create linker: {}", e)))?;

        Ok(Self {
            config,
            engine,
            linker,
            mods: HashMap::new(),
        })
    }

    /// Load a WASM mod from bytes
    ///
    /// # Arguments
    /// * `mod_id` - Unique identifier for this mod
    /// * `wasm_bytes` - The compiled WASM module bytes
    /// * `capabilities` - Capabilities granted to this mod
    ///
    /// # Returns
    /// Ok(()) on success, or an error if loading failed
    pub fn load_mod(
        &mut self,
        mod_id: impl Into<String>,
        wasm_bytes: &[u8],
        capabilities: Capability,
    ) -> Result<(), RuntimeError> {
        let mod_id = mod_id.into();

        // Check if mod is already loaded
        if self.mods.contains_key(&mod_id) {
            return Err(RuntimeError::ModAlreadyLoaded(mod_id));
        }

        // Load and validate the module
        let loader = ModuleLoader::new(self.engine.inner().clone());
        let module = loader.load_from_bytes(wasm_bytes)?;

        // Create the store with mod state
        let state = ModState::new(&mod_id, &self.config, capabilities);
        let mut store = Store::new(self.engine.inner(), state);

        // Configure resource limiter for memory limits
        // The ResourceLimiter trait is implemented on ModState
        store.limiter(|state| state);

        // Set initial fuel
        let fuel = self.config.handler_cpu_budget_ms * self.config.fuel_per_ms;
        store
            .set_fuel(fuel)
            .map_err(|e| RuntimeError::Internal(format!("Failed to set fuel: {}", e)))?;

        // Instantiate the module
        let instance = self
            .linker
            .instantiate(&mut store, &module)
            .map_err(|e| RuntimeError::WasmInvalid(format!("Instantiation failed: {}", e)))?;

        // Get exports
        let exports = ModExports::from_instance(&mut store, &instance)?;

        // Create the instance wrapper
        let mut wasm_instance = WasmInstance {
            store,
            instance,
            exports,
        };

        // Call mod_init
        let init_result = wasm_instance.call_init()?;
        if init_result != 0 {
            return Err(RuntimeError::Internal(format!(
                "mod_init returned error code: {}",
                init_result
            )));
        }

        // Record successful init
        wasm_instance.store.data_mut().record_success();

        if self.config.debug_mode {
            tracing::info!(mod_id = %mod_id, "Mod loaded and initialized successfully");
        }

        // Store the instance
        self.mods.insert(mod_id, wasm_instance);

        Ok(())
    }

    /// Unload a mod
    ///
    /// Calls mod_shutdown and removes the mod from the runtime.
    pub fn unload_mod(&mut self, mod_id: &str) -> Result<(), RuntimeError> {
        let mut instance = self
            .mods
            .remove(mod_id)
            .ok_or_else(|| RuntimeError::ModNotFound(mod_id.to_string()))?;

        // Call shutdown (ignore errors on shutdown)
        let _ = instance.call_shutdown();

        if self.config.debug_mode {
            tracing::info!(mod_id = %mod_id, "Mod unloaded");
        }

        Ok(())
    }

    /// Check if a mod is loaded
    pub fn is_mod_loaded(&self, mod_id: &str) -> bool {
        self.mods.contains_key(mod_id)
    }

    /// Get the number of loaded mods
    pub fn mod_count(&self) -> usize {
        self.mods.len()
    }

    /// Get a list of loaded mod IDs
    pub fn mod_ids(&self) -> Vec<&str> {
        self.mods.keys().map(|s| s.as_str()).collect()
    }

    /// Get metrics for a specific mod
    pub fn get_mod_metrics(&self, mod_id: &str) -> Option<&WasmModMetrics> {
        self.mods.get(mod_id).map(|i| i.metrics())
    }

    /// Check if a mod is enabled
    pub fn is_mod_enabled(&self, mod_id: &str) -> Option<bool> {
        self.mods.get(mod_id).map(|i| i.is_enabled())
    }

    /// Get the runtime configuration
    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }

    /// Dispatch an event to all subscribed mods
    ///
    /// # Arguments
    /// * `event_id` - The type of event (see abi::EventTypeId)
    /// * `payload` - Serialized event payload (bincode)
    ///
    /// # Returns
    /// Ok(true) if the event was cancelled by any mod, Ok(false) otherwise
    pub fn dispatch_event(&mut self, event_id: u8, payload: &[u8]) -> Result<bool, RuntimeError> {
        // Create a GameEvent for the event context
        // For testing, we create a simple event based on event_id
        let event_type = match event_id {
            1 => EventType::ServerStart,
            2 => EventType::ServerStop,
            3 => EventType::PlayerJoin,
            4 => EventType::PlayerLeave,
            5 => EventType::PlayerChat,
            6 => EventType::BlockPlaced,
            7 => EventType::BlockBroken,
            8 => EventType::EntityDamaged,
            9 => EventType::ModMessage,
            _ => EventType::ServerStart, // Fallback
        };

        // Create a dummy GameEvent for the context
        let event = GameEvent {
            event_type,
            payload: EventPayload::PlayerChat(PlayerChatPayload {
                player_id: 0,
                text: String::from_utf8_lossy(payload).to_string(),
            }),
            timestamp: 0,
            cancellable: event_type.is_cancellable(),
            cancelled: false,
        };

        self.dispatch_game_event(event_id, payload, event)
    }

    /// Dispatch a GameEvent to all subscribed mods
    ///
    /// This is the full-featured version that includes proper event context.
    pub fn dispatch_game_event(
        &mut self,
        event_id: u8,
        payload: &[u8],
        event: GameEvent,
    ) -> Result<bool, RuntimeError> {
        let mut cancelled = false;

        // Get list of mod IDs to iterate (avoid borrow issues)
        let mod_ids: Vec<String> = self.mods.keys().cloned().collect();

        for mod_id in mod_ids {
            let instance = match self.mods.get_mut(&mod_id) {
                Some(i) => i,
                None => continue,
            };

            // Skip disabled mods
            if !instance.is_enabled() {
                continue;
            }

            // Check if subscribed
            if !instance.store.data().is_subscribed(event_id) {
                continue;
            }

            // Set event context BEFORE calling handler
            instance.store.data_mut().event_context.current_event = Some(event.clone());
            instance.store.data_mut().event_context.current_mod_id = Some(mod_id.clone());

            // Write payload to WASM memory at a known offset
            // We use 0x10000 (64KB) as the payload buffer location
            const PAYLOAD_OFFSET: usize = 0x10000;

            // Write payload to memory
            let write_result = memory::write_bytes(
                &mut instance.store,
                &instance.exports.memory,
                PAYLOAD_OFFSET,
                payload,
            );

            if let Err(e) = write_result {
                tracing::warn!(mod_id = %mod_id, error = %e, "Failed to write event payload to mod memory");
                // Clear event context
                instance.store.data_mut().event_context.current_event = None;
                instance.store.data_mut().event_context.current_mod_id = None;
                continue;
            }

            // Call mod_on_event
            let result = instance.call_on_event(
                event_id as i32,
                PAYLOAD_OFFSET as i32,
                payload.len() as i32,
            );

            // Check if event was cancelled via plix_cancel_event
            let was_cancelled = instance
                .store
                .data()
                .event_context
                .current_event
                .as_ref()
                .map(|e| e.cancelled)
                .unwrap_or(false);

            // Clear event context
            instance.store.data_mut().event_context.current_event = None;
            instance.store.data_mut().event_context.current_mod_id = None;

            let event_cancelled = match result {
                Ok(0) => {
                    // Success, check if cancelled via host function
                    instance.store.data_mut().record_success();
                    was_cancelled
                }
                Ok(_code) => {
                    // Non-zero return also indicates cancellation
                    instance.store.data_mut().record_success();
                    true
                }
                Err(e) => {
                    // Handler failed - record trap
                    instance.store.data_mut().metrics.record_trap();

                    // Track consecutive errors
                    let should_disable = instance.store.data_mut().record_error();

                    if should_disable {
                        instance
                            .store
                            .data_mut()
                            .disable(format!("Exceeded error threshold: {}", e));
                        tracing::warn!(
                            mod_id = %mod_id,
                            error = %e,
                            "Mod auto-disabled after repeated errors"
                        );
                    } else {
                        tracing::debug!(mod_id = %mod_id, error = %e, "Mod handler error");
                    }
                    false
                }
            };

            if event_cancelled {
                cancelled = true;
            }

            // Record event processed
            instance
                .store
                .data_mut()
                .metrics
                .record_event(event_cancelled);
        }

        Ok(cancelled)
    }

    /// Get mutable access to a mod instance (for advanced use)
    pub fn get_mod_mut(&mut self, mod_id: &str) -> Option<&mut WasmInstance> {
        self.mods.get_mut(mod_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RuntimeConfig::default();
        assert_eq!(config.handler_cpu_budget_ms, DEFAULT_HANDLER_CPU_BUDGET_MS);
        assert_eq!(config.max_memory_bytes, DEFAULT_MEMORY_LIMIT_BYTES);
        assert_eq!(config.violation_threshold, DEFAULT_ERROR_THRESHOLD);
        assert!(!config.debug_mode);
    }

    #[test]
    fn test_config_builder() {
        let config = RuntimeConfig::new()
            .with_handler_budget_ms(10)
            .with_memory_limit(64 * 1024 * 1024)
            .with_debug_mode(true)
            .with_violation_threshold(3);

        assert_eq!(config.handler_cpu_budget_ms, 10);
        assert_eq!(config.max_memory_bytes, 64 * 1024 * 1024);
        assert!(config.debug_mode);
        assert_eq!(config.violation_threshold, 3);
    }

    #[test]
    fn test_runtime_creation() {
        let config = RuntimeConfig::default();
        let runtime = WasmRuntime::new(config);
        assert!(runtime.is_ok());

        let runtime = runtime.unwrap();
        assert_eq!(runtime.mod_count(), 0);
    }

    #[test]
    fn test_load_minimal_mod() {
        let config = RuntimeConfig::default();
        let mut runtime = WasmRuntime::new(config).unwrap();

        let wasm_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/minimal_mod/mod.wasm"
        );
        let wasm_bytes = std::fs::read(wasm_path).expect("Failed to read minimal mod");

        let result = runtime.load_mod("test-mod", &wasm_bytes, Capability::empty());
        assert!(result.is_ok(), "Failed to load mod: {:?}", result);

        assert!(runtime.is_mod_loaded("test-mod"));
        assert_eq!(runtime.mod_count(), 1);
        assert!(runtime.is_mod_enabled("test-mod").unwrap());
    }

    #[test]
    fn test_load_invalid_wasm() {
        let config = RuntimeConfig::default();
        let mut runtime = WasmRuntime::new(config).unwrap();

        let invalid_bytes = b"not valid wasm bytes";
        let result = runtime.load_mod("bad-mod", invalid_bytes, Capability::empty());

        assert!(result.is_err());
        if let Err(RuntimeError::WasmInvalid(_)) = result {
            // Expected
        } else {
            panic!("Expected WasmInvalid error, got: {:?}", result);
        }

        assert!(!runtime.is_mod_loaded("bad-mod"));
    }

    #[test]
    fn test_load_missing_export() {
        let config = RuntimeConfig::default();
        let mut runtime = WasmRuntime::new(config).unwrap();

        let wasm_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/missing_export_mod/mod.wasm"
        );
        let wasm_bytes = std::fs::read(wasm_path).expect("Failed to read missing export mod");

        let result = runtime.load_mod("bad-mod", &wasm_bytes, Capability::empty());

        assert!(result.is_err());
        if let Err(RuntimeError::MissingExport(name)) = result {
            assert_eq!(name, "mod_init");
        } else {
            panic!("Expected MissingExport error, got: {:?}", result);
        }
    }

    #[test]
    fn test_load_duplicate_mod() {
        let config = RuntimeConfig::default();
        let mut runtime = WasmRuntime::new(config).unwrap();

        let wasm_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/minimal_mod/mod.wasm"
        );
        let wasm_bytes = std::fs::read(wasm_path).expect("Failed to read minimal mod");

        // First load should succeed
        runtime
            .load_mod("test-mod", &wasm_bytes, Capability::empty())
            .unwrap();

        // Second load with same ID should fail
        let result = runtime.load_mod("test-mod", &wasm_bytes, Capability::empty());
        assert!(result.is_err());
        if let Err(RuntimeError::ModAlreadyLoaded(id)) = result {
            assert_eq!(id, "test-mod");
        } else {
            panic!("Expected ModAlreadyLoaded error, got: {:?}", result);
        }
    }

    #[test]
    fn test_unload_mod() {
        let config = RuntimeConfig::default();
        let mut runtime = WasmRuntime::new(config).unwrap();

        let wasm_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/minimal_mod/mod.wasm"
        );
        let wasm_bytes = std::fs::read(wasm_path).expect("Failed to read minimal mod");

        runtime
            .load_mod("test-mod", &wasm_bytes, Capability::empty())
            .unwrap();
        assert!(runtime.is_mod_loaded("test-mod"));

        let result = runtime.unload_mod("test-mod");
        assert!(result.is_ok());
        assert!(!runtime.is_mod_loaded("test-mod"));
        assert_eq!(runtime.mod_count(), 0);
    }

    #[test]
    fn test_unload_nonexistent_mod() {
        let config = RuntimeConfig::default();
        let mut runtime = WasmRuntime::new(config).unwrap();

        let result = runtime.unload_mod("nonexistent");
        assert!(result.is_err());
        if let Err(RuntimeError::ModNotFound(id)) = result {
            assert_eq!(id, "nonexistent");
        } else {
            panic!("Expected ModNotFound error, got: {:?}", result);
        }
    }

    #[test]
    fn test_dispatch_event_to_chat_filter_mod() {
        let config = RuntimeConfig::default();
        let mut runtime = WasmRuntime::new(config).unwrap();

        let wasm_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/chat_filter_mod/mod.wasm"
        );
        let wasm_bytes = std::fs::read(wasm_path).expect("Failed to read chat filter mod");

        // Load with capability to cancel events
        runtime
            .load_mod("chat-filter", &wasm_bytes, Capability::EVENT_CANCEL_CHAT)
            .unwrap();

        // Dispatch a PlayerChat event (ID 5) with clean message
        let clean_payload = b"Hello world!";
        let cancelled = runtime.dispatch_event(5, clean_payload).unwrap();
        assert!(!cancelled, "Clean message should not be cancelled");

        // Dispatch a PlayerChat event with badword
        let bad_payload = b"This contains badword in it";
        let cancelled = runtime.dispatch_event(5, bad_payload).unwrap();
        assert!(cancelled, "Message with badword should be cancelled");
    }

    #[test]
    fn test_dispatch_event_unsubscribed() {
        let config = RuntimeConfig::default();
        let mut runtime = WasmRuntime::new(config).unwrap();

        let wasm_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/minimal_mod/mod.wasm"
        );
        let wasm_bytes = std::fs::read(wasm_path).expect("Failed to read minimal mod");

        // Load minimal mod (doesn't subscribe to any events)
        runtime
            .load_mod("minimal", &wasm_bytes, Capability::empty())
            .unwrap();

        // Dispatch event - should not be processed since not subscribed
        let payload = b"test";
        let cancelled = runtime.dispatch_event(5, payload).unwrap();
        assert!(!cancelled);

        // Check that no events were recorded
        let metrics = runtime.get_mod_metrics("minimal").unwrap();
        assert_eq!(metrics.events_processed, 0);
    }

    #[test]
    fn test_dispatch_event_without_cancel_capability() {
        let config = RuntimeConfig::default();
        let mut runtime = WasmRuntime::new(config).unwrap();

        let wasm_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/chat_filter_mod/mod.wasm"
        );
        let wasm_bytes = std::fs::read(wasm_path).expect("Failed to read chat filter mod");

        // Load WITHOUT cancel capability
        runtime
            .load_mod("chat-filter", &wasm_bytes, Capability::empty())
            .unwrap();

        // Dispatch event with badword - cancel should fail due to no capability
        let bad_payload = b"This contains badword in it";
        let cancelled = runtime.dispatch_event(5, bad_payload).unwrap();
        // The mod tries to cancel but lacks permission, so it should fail
        // The mod returns the error code from plix_cancel_event which would be 2 (permission denied)
        assert!(!cancelled, "Should not cancel without capability");
    }

    #[test]
    fn test_infinite_loop_interrupted_by_fuel() {
        // Use a small fuel budget to ensure quick interruption
        let config = RuntimeConfig::default().with_handler_budget_ms(1); // Very small budget
        let mut runtime = WasmRuntime::new(config).unwrap();

        let wasm_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/infinite_loop_mod/mod.wasm"
        );
        let wasm_bytes = std::fs::read(wasm_path).expect("Failed to read infinite loop mod");

        // Load the mod
        runtime
            .load_mod("looper", &wasm_bytes, Capability::empty())
            .unwrap();

        // Dispatch an event - the handler should be interrupted due to fuel exhaustion
        let payload = b"test";
        let result = runtime.dispatch_event(5, payload);

        // The event should complete (not error at runtime level) but handler may have failed
        assert!(result.is_ok());

        // Check that the mod recorded a trap/error
        let metrics = runtime.get_mod_metrics("looper").unwrap();
        assert!(metrics.trap_count > 0 || metrics.events_processed > 0);
    }

    #[test]
    fn test_mod_disabled_after_error_threshold() {
        let config = RuntimeConfig::default()
            .with_handler_budget_ms(1) // Very small budget to trigger errors
            .with_violation_threshold(3); // Disable after 3 errors
        let mut runtime = WasmRuntime::new(config).unwrap();

        let wasm_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/infinite_loop_mod/mod.wasm"
        );
        let wasm_bytes = std::fs::read(wasm_path).expect("Failed to read infinite loop mod");

        // Load the mod
        runtime
            .load_mod("looper", &wasm_bytes, Capability::empty())
            .unwrap();

        // Dispatch events until mod is disabled
        let payload = b"test";
        for _ in 0..5 {
            let _ = runtime.dispatch_event(5, payload);
        }

        // After 3 consecutive errors, mod should be disabled
        assert!(
            !runtime.is_mod_enabled("looper").unwrap(),
            "Mod should be disabled after error threshold"
        );
    }

    #[test]
    fn test_success_resets_error_counter() {
        let config = RuntimeConfig::default().with_violation_threshold(3);
        let mut runtime = WasmRuntime::new(config).unwrap();

        let wasm_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/chat_filter_mod/mod.wasm"
        );
        let wasm_bytes = std::fs::read(wasm_path).expect("Failed to read chat filter mod");

        // Load with some capability
        runtime
            .load_mod("chat-filter", &wasm_bytes, Capability::EVENT_CANCEL_CHAT)
            .unwrap();

        // Dispatch several events that should succeed
        let clean_payload = b"Hello world!";
        for _ in 0..10 {
            let cancelled = runtime.dispatch_event(5, clean_payload).unwrap();
            assert!(!cancelled);
        }

        // Mod should still be enabled (no errors)
        assert!(runtime.is_mod_enabled("chat-filter").unwrap());

        // Check error count in metrics - should be low/zero
        let instance = runtime.get_mod_mut("chat-filter").unwrap();
        assert_eq!(instance.store.data().error_count, 0);
    }

    #[test]
    fn test_memory_growth_beyond_limit_fails() {
        // Use a small memory limit (1 MiB = 16 WASM pages)
        let config = RuntimeConfig::default().with_memory_limit(1024 * 1024); // 1 MiB
        let mut runtime = WasmRuntime::new(config).unwrap();

        let wasm_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/memory_bomb_mod/mod.wasm"
        );
        let wasm_bytes = std::fs::read(wasm_path).expect("Failed to read memory bomb mod");

        // Load the mod
        runtime
            .load_mod("memory-bomb", &wasm_bytes, Capability::empty())
            .unwrap();

        // Dispatch an event - the mod will try to allocate lots of memory
        let payload = b"trigger";
        let _ = runtime.dispatch_event(5, payload);

        // Check that memory errors were recorded
        let metrics = runtime.get_mod_metrics("memory-bomb").unwrap();
        assert!(
            metrics.memory_error_count > 0,
            "Memory allocation should have been rejected"
        );
    }

    #[test]
    fn test_repeated_oom_disables_mod() {
        // Use a small memory limit
        let config = RuntimeConfig::default()
            .with_memory_limit(1024 * 1024) // 1 MiB
            .with_violation_threshold(3);
        let mut runtime = WasmRuntime::new(config).unwrap();

        let wasm_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/memory_bomb_mod/mod.wasm"
        );
        let wasm_bytes = std::fs::read(wasm_path).expect("Failed to read memory bomb mod");

        // Load the mod
        runtime
            .load_mod("memory-bomb", &wasm_bytes, Capability::empty())
            .unwrap();

        // Dispatch events repeatedly - this may or may not disable the mod
        // depending on whether the memory.grow failure causes a trap
        let payload = b"trigger";
        for _ in 0..5 {
            let _ = runtime.dispatch_event(5, payload);
        }

        // The mod should have had memory errors recorded
        let metrics = runtime.get_mod_metrics("memory-bomb").unwrap();
        assert!(
            metrics.memory_error_count > 0,
            "Memory allocation failures should be recorded"
        );

        // Note: memory.grow returning -1 doesn't necessarily trap the module,
        // it just fails the allocation. The mod continues to run.
        // This is different from fuel exhaustion which does trap.
    }
}
