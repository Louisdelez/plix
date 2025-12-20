//! Wasmtime engine configuration
//!
//! Wraps wasmtime::Engine with plix-specific security settings:
//! - Fuel consumption enabled for CPU budgeting
//! - No WASI imports
//! - Memory limits via ResourceLimiter

use crate::RuntimeConfig;
use wasmtime::{Config, Engine};

/// Wrapper around Wasmtime Engine with plix-specific configuration
pub struct WasmEngine {
    /// The underlying Wasmtime engine
    inner: Engine,
}

impl WasmEngine {
    /// Create a new WASM engine with the given configuration
    pub fn new(config: &RuntimeConfig) -> Result<Self, wasmtime::Error> {
        let mut wasm_config = Config::new();

        // Enable fuel consumption for CPU budgeting - this is the key security feature
        wasm_config.consume_fuel(true);

        // Enable parallel compilation for faster module loading
        wasm_config.parallel_compilation(true);

        // Enable debug info only in debug builds
        #[cfg(debug_assertions)]
        wasm_config.debug_info(true);
        #[cfg(not(debug_assertions))]
        wasm_config.debug_info(false);

        // Use Cranelift for JIT compilation
        wasm_config.strategy(wasmtime::Strategy::Cranelift);

        // Memory settings - use defaults which are secure
        wasm_config.static_memory_forced(false);

        // Keep most WASM features at their defaults - Wasmtime's defaults are secure
        // Only disable features that could be problematic:
        wasm_config.wasm_memory64(false); // Stick to 32-bit memory
        wasm_config.wasm_tail_call(false); // Not needed

        // Log configuration in debug mode
        if config.debug_mode {
            tracing::debug!(
                fuel_per_ms = config.fuel_per_ms,
                handler_budget_ms = config.handler_cpu_budget_ms,
                memory_limit = config.max_memory_bytes,
                "Creating WASM engine with configuration"
            );
        }

        let engine = Engine::new(&wasm_config)?;
        Ok(Self { inner: engine })
    }

    /// Get a reference to the inner Wasmtime engine
    pub fn inner(&self) -> &Engine {
        &self.inner
    }

    /// Consume the wrapper and return the inner engine
    pub fn into_inner(self) -> Engine {
        self.inner
    }
}

impl std::fmt::Debug for WasmEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WasmEngine").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_engine() {
        let config = RuntimeConfig::default();
        let engine = WasmEngine::new(&config);
        assert!(engine.is_ok());
    }

    #[test]
    fn test_engine_debug() {
        let config = RuntimeConfig::default();
        let engine = WasmEngine::new(&config).unwrap();
        let debug_str = format!("{:?}", engine);
        assert!(debug_str.contains("WasmEngine"));
    }
}
