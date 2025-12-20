//! Per-mod WASM instance management
//!
//! Each mod runs in its own WasmInstance with isolated memory and state.

use crate::abi::ResponseBuffer;
use crate::errors::RuntimeError;
use crate::metrics::WasmModMetrics;
use crate::RuntimeConfig;
use plix_mod_core::events::{EventContext, GameEvent};
use plix_mod_core::Capability;
use wasmtime::{Memory, ResourceLimiter, Store, TypedFunc};

/// Per-mod state stored in Wasmtime Store
///
/// This is the `T` in `Store<T>` - it holds all state accessible to host functions.
pub struct ModState {
    /// Unique identifier for this mod
    pub mod_id: String,
    /// Effective capabilities granted to this mod
    pub capabilities: Capability,
    /// Context for the currently processing event
    pub event_context: EventContext,
    /// Fuel budget for the current call
    pub fuel_budget: u64,
    /// Maximum memory in bytes
    pub memory_limit: usize,
    /// Response buffer for host function returns
    pub response_buffer: ResponseBuffer,
    /// Accumulated metrics
    pub metrics: WasmModMetrics,
    /// Consecutive error count
    pub error_count: u32,
    /// Error threshold for auto-disable
    pub error_threshold: u32,
    /// Whether this mod is enabled
    pub enabled: bool,
    /// Reason if disabled
    pub disable_reason: Option<String>,
    /// Event subscriptions (bitmask)
    pub subscriptions: u32,
    /// Debug mode
    pub debug_mode: bool,
}

impl ModState {
    /// Create a new ModState for a mod
    pub fn new(
        mod_id: impl Into<String>,
        config: &RuntimeConfig,
        capabilities: Capability,
    ) -> Self {
        Self {
            mod_id: mod_id.into(),
            capabilities,
            event_context: EventContext::new(),
            fuel_budget: config.handler_cpu_budget_ms * config.fuel_per_ms,
            memory_limit: config.max_memory_bytes,
            response_buffer: ResponseBuffer::new(),
            metrics: WasmModMetrics::new(),
            error_count: 0,
            error_threshold: config.violation_threshold,
            enabled: true,
            disable_reason: None,
            subscriptions: 0,
            debug_mode: config.debug_mode,
        }
    }

    /// Check if the mod has a specific capability
    pub fn has_capability(&self, cap: Capability) -> bool {
        self.capabilities.contains(cap)
    }

    /// Record a successful handler execution
    pub fn record_success(&mut self) {
        self.error_count = 0;
    }

    /// Record a handler error
    ///
    /// Returns true if the mod should be disabled (threshold reached)
    pub fn record_error(&mut self) -> bool {
        self.error_count += 1;
        self.error_count >= self.error_threshold
    }

    /// Disable this mod
    pub fn disable(&mut self, reason: impl Into<String>) {
        self.enabled = false;
        self.disable_reason = Some(reason.into());
    }

    /// Subscribe to an event type
    pub fn subscribe(&mut self, event_type_id: u8) {
        self.subscriptions |= 1 << event_type_id;
    }

    /// Check if subscribed to an event type
    pub fn is_subscribed(&self, event_type_id: u8) -> bool {
        (self.subscriptions & (1 << event_type_id)) != 0
    }

    /// Set the current event context for cancellation support
    pub fn set_event_context(&mut self, event: GameEvent, mod_id: &str) {
        self.event_context.current_event = Some(event);
        self.event_context.current_mod_id = Some(mod_id.to_string());
    }

    /// Clear the event context after processing
    pub fn clear_event_context(&mut self) {
        self.event_context.current_event = None;
        self.event_context.current_mod_id = None;
    }

    /// Clear the response buffer before a host call
    pub fn clear_response(&mut self) {
        self.response_buffer.clear();
    }

    /// Write data to the response buffer
    pub fn write_response(&mut self, data: &[u8]) -> Result<(), RuntimeError> {
        self.response_buffer
            .write_all(data)
            .map_err(|e| RuntimeError::HostFunction(format!("Response buffer overflow: {}", e)))
    }

    /// Get the response buffer length
    pub fn response_len(&self) -> usize {
        self.response_buffer.len()
    }

    /// Get the current memory usage for resource limiting
    pub fn current_memory_usage(&self) -> usize {
        self.metrics.memory_bytes
    }
}

/// ResourceLimiter implementation to enforce memory limits
///
/// This prevents mods from allocating more memory than their configured limit
/// via `memory.grow` operations.
impl ResourceLimiter for ModState {
    fn memory_growing(
        &mut self,
        current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> Result<bool, wasmtime::Error> {
        // Check if the desired size exceeds our configured limit
        if desired > self.memory_limit {
            if self.debug_mode {
                tracing::debug!(
                    mod_id = %self.mod_id,
                    current_bytes = current,
                    desired_bytes = desired,
                    limit_bytes = self.memory_limit,
                    "Memory growth rejected: would exceed limit"
                );
            }
            // Record the failed memory allocation attempt
            self.metrics.record_memory_error();
            // Return Ok(false) to deny the growth request
            return Ok(false);
        }

        // Update metrics with new memory size
        self.metrics.memory_bytes = desired;

        if self.debug_mode {
            tracing::debug!(
                mod_id = %self.mod_id,
                current_bytes = current,
                desired_bytes = desired,
                limit_bytes = self.memory_limit,
                "Memory growth permitted"
            );
        }

        Ok(true)
    }

    fn table_growing(
        &mut self,
        _current: usize,
        _desired: usize,
        _maximum: Option<usize>,
    ) -> Result<bool, wasmtime::Error> {
        // Allow table growth (we don't limit tables in this implementation)
        Ok(true)
    }
}

/// Cached references to required WASM exports
pub struct ModExports {
    /// mod_init() -> i32
    pub mod_init: TypedFunc<(), i32>,
    /// mod_on_event(event_id: i32, payload_ptr: i32, payload_len: i32) -> i32
    pub mod_on_event: TypedFunc<(i32, i32, i32), i32>,
    /// mod_shutdown() -> i32
    pub mod_shutdown: TypedFunc<(), i32>,
    /// Linear memory
    pub memory: Memory,
}

impl ModExports {
    /// Validate and extract exports from a WASM instance
    pub fn from_instance(
        store: &mut Store<ModState>,
        instance: &wasmtime::Instance,
    ) -> Result<Self, RuntimeError> {
        // Get mod_init
        let mod_init = instance
            .get_typed_func::<(), i32>(&mut *store, "mod_init")
            .map_err(|_| RuntimeError::MissingExport("mod_init".to_string()))?;

        // Get mod_on_event
        let mod_on_event = instance
            .get_typed_func::<(i32, i32, i32), i32>(&mut *store, "mod_on_event")
            .map_err(|_| RuntimeError::MissingExport("mod_on_event".to_string()))?;

        // Get mod_shutdown
        let mod_shutdown = instance
            .get_typed_func::<(), i32>(&mut *store, "mod_shutdown")
            .map_err(|_| RuntimeError::MissingExport("mod_shutdown".to_string()))?;

        // Get memory
        let memory = instance
            .get_memory(&mut *store, "memory")
            .ok_or_else(|| RuntimeError::MissingExport("memory".to_string()))?;

        Ok(Self {
            mod_init,
            mod_on_event,
            mod_shutdown,
            memory,
        })
    }
}

/// A loaded and instantiated WASM mod
pub struct WasmInstance {
    /// Wasmtime store with mod state
    pub store: Store<ModState>,
    /// The instantiated module
    pub instance: wasmtime::Instance,
    /// Cached export references
    pub exports: ModExports,
}

impl WasmInstance {
    /// Get the mod ID
    pub fn mod_id(&self) -> &str {
        &self.store.data().mod_id
    }

    /// Check if the mod is enabled
    pub fn is_enabled(&self) -> bool {
        self.store.data().enabled
    }

    /// Get current memory size in bytes
    pub fn memory_size(&self) -> usize {
        self.exports.memory.data_size(&self.store)
    }

    /// Get a reference to the mod metrics
    pub fn metrics(&self) -> &WasmModMetrics {
        &self.store.data().metrics
    }

    /// Call mod_init
    pub fn call_init(&mut self) -> Result<i32, RuntimeError> {
        // Set fuel budget
        let fuel = self.store.data().fuel_budget;
        self.store
            .set_fuel(fuel)
            .map_err(|e| RuntimeError::Internal(format!("Failed to set fuel: {}", e)))?;

        // Call mod_init
        let result = self
            .exports
            .mod_init
            .call(&mut self.store, ())
            .map_err(|e| {
                // Check if this was a fuel exhaustion
                if e.to_string().contains("fuel") {
                    RuntimeError::FuelExhausted
                } else {
                    RuntimeError::Trap(e.to_string())
                }
            })?;

        // Record fuel consumption
        let remaining = self.store.get_fuel().unwrap_or(0);
        let consumed = fuel.saturating_sub(remaining);
        let fuel_per_ms = self.store.data().fuel_budget / 5; // Approximate
        self.store
            .data_mut()
            .metrics
            .record_cpu_time(consumed, fuel_per_ms);

        Ok(result)
    }

    /// Call mod_on_event
    pub fn call_on_event(
        &mut self,
        event_id: i32,
        payload_ptr: i32,
        payload_len: i32,
    ) -> Result<i32, RuntimeError> {
        // Set fuel budget
        let fuel = self.store.data().fuel_budget;
        self.store
            .set_fuel(fuel)
            .map_err(|e| RuntimeError::Internal(format!("Failed to set fuel: {}", e)))?;

        // Call mod_on_event
        let result = self
            .exports
            .mod_on_event
            .call(&mut self.store, (event_id, payload_ptr, payload_len))
            .map_err(|e| {
                if e.to_string().contains("fuel") {
                    RuntimeError::FuelExhausted
                } else {
                    RuntimeError::Trap(e.to_string())
                }
            })?;

        // Record fuel consumption
        let remaining = self.store.get_fuel().unwrap_or(0);
        let consumed = fuel.saturating_sub(remaining);
        let fuel_per_ms = self.store.data().fuel_budget / 5;
        self.store
            .data_mut()
            .metrics
            .record_cpu_time(consumed, fuel_per_ms);

        Ok(result)
    }

    /// Call mod_shutdown
    pub fn call_shutdown(&mut self) -> Result<i32, RuntimeError> {
        // Set fuel budget
        let fuel = self.store.data().fuel_budget;
        self.store
            .set_fuel(fuel)
            .map_err(|e| RuntimeError::Internal(format!("Failed to set fuel: {}", e)))?;

        // Call mod_shutdown
        let result = self
            .exports
            .mod_shutdown
            .call(&mut self.store, ())
            .map_err(|e| {
                if e.to_string().contains("fuel") {
                    RuntimeError::FuelExhausted
                } else {
                    RuntimeError::Trap(e.to_string())
                }
            })?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mod_state_new() {
        let config = RuntimeConfig::default();
        let state = ModState::new("test-mod", &config, Capability::WORLD_READ);

        assert_eq!(state.mod_id, "test-mod");
        assert!(state.has_capability(Capability::WORLD_READ));
        assert!(!state.has_capability(Capability::WORLD_WRITE));
        assert!(state.enabled);
        assert_eq!(state.error_count, 0);
    }

    #[test]
    fn test_mod_state_error_tracking() {
        let config = RuntimeConfig::default().with_violation_threshold(3);
        let mut state = ModState::new("test-mod", &config, Capability::empty());

        assert!(!state.record_error()); // 1
        assert!(!state.record_error()); // 2
        assert!(state.record_error()); // 3 - threshold reached
    }

    #[test]
    fn test_mod_state_success_resets_errors() {
        let config = RuntimeConfig::default();
        let mut state = ModState::new("test-mod", &config, Capability::empty());

        state.record_error();
        state.record_error();
        assert_eq!(state.error_count, 2);

        state.record_success();
        assert_eq!(state.error_count, 0);
    }

    #[test]
    fn test_mod_state_subscriptions() {
        let config = RuntimeConfig::default();
        let mut state = ModState::new("test-mod", &config, Capability::empty());

        state.subscribe(0x05); // PlayerChat
        assert!(state.is_subscribed(0x05));
        assert!(!state.is_subscribed(0x06));
    }

    #[test]
    fn test_mod_state_disable() {
        let config = RuntimeConfig::default();
        let mut state = ModState::new("test-mod", &config, Capability::empty());

        assert!(state.enabled);
        state.disable("Too many errors");
        assert!(!state.enabled);
        assert_eq!(state.disable_reason.as_deref(), Some("Too many errors"));
    }
}
