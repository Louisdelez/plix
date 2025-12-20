//! WASM module loading and validation
//!
//! Handles loading, validating, and compiling WASM modules.
//! Ensures mods have required exports and no unsupported imports.

use crate::errors::RuntimeError;
use wasmtime::{Engine, Module};

/// Loader for WASM mod modules
///
/// Handles validation of WASM binary format and export requirements.
pub struct ModuleLoader {
    engine: Engine,
}

impl ModuleLoader {
    /// Create a new module loader using the given engine
    pub fn new(engine: Engine) -> Self {
        Self { engine }
    }

    /// Load and validate a WASM module from bytes
    ///
    /// This performs:
    /// 1. WASM binary validation
    /// 2. Module compilation
    /// 3. Export validation (mod_init, mod_on_event, mod_shutdown, memory)
    /// 4. Import validation (reject WASI imports)
    pub fn load_from_bytes(&self, bytes: &[u8]) -> Result<Module, RuntimeError> {
        // Validate WASM binary format
        Module::validate(&self.engine, bytes)
            .map_err(|e| RuntimeError::WasmInvalid(e.to_string()))?;

        // Compile the module
        let module = Module::new(&self.engine, bytes)
            .map_err(|e| RuntimeError::WasmInvalid(e.to_string()))?;

        // Validate no WASI imports
        self.validate_no_wasi(&module)?;

        // Validate required exports
        self.validate_exports(&module)?;

        Ok(module)
    }

    /// Validate that the module has all required exports
    fn validate_exports(&self, module: &Module) -> Result<(), RuntimeError> {
        let mut has_mod_init = false;
        let mut has_mod_on_event = false;
        let mut has_mod_shutdown = false;
        let mut has_memory = false;

        for export in module.exports() {
            match export.name() {
                "mod_init" => {
                    // Validate signature: () -> i32
                    if let Some(func) = export.ty().func() {
                        if func.params().len() != 0 || func.results().len() != 1 {
                            return Err(RuntimeError::InvalidExportSignature {
                                name: "mod_init".to_string(),
                                expected: "() -> i32".to_string(),
                                actual: format!(
                                    "({} params) -> ({} results)",
                                    func.params().len(),
                                    func.results().len()
                                ),
                            });
                        }
                        has_mod_init = true;
                    }
                }
                "mod_on_event" => {
                    // Validate signature: (i32, i32, i32) -> i32
                    if let Some(func) = export.ty().func() {
                        if func.params().len() != 3 || func.results().len() != 1 {
                            return Err(RuntimeError::InvalidExportSignature {
                                name: "mod_on_event".to_string(),
                                expected: "(i32, i32, i32) -> i32".to_string(),
                                actual: format!(
                                    "({} params) -> ({} results)",
                                    func.params().len(),
                                    func.results().len()
                                ),
                            });
                        }
                        has_mod_on_event = true;
                    }
                }
                "mod_shutdown" => {
                    // Validate signature: () -> i32
                    if let Some(func) = export.ty().func() {
                        if func.params().len() != 0 || func.results().len() != 1 {
                            return Err(RuntimeError::InvalidExportSignature {
                                name: "mod_shutdown".to_string(),
                                expected: "() -> i32".to_string(),
                                actual: format!(
                                    "({} params) -> ({} results)",
                                    func.params().len(),
                                    func.results().len()
                                ),
                            });
                        }
                        has_mod_shutdown = true;
                    }
                }
                "memory" => {
                    if export.ty().memory().is_some() {
                        has_memory = true;
                    }
                }
                _ => {}
            }
        }

        if !has_mod_init {
            return Err(RuntimeError::MissingExport("mod_init".to_string()));
        }
        if !has_mod_on_event {
            return Err(RuntimeError::MissingExport("mod_on_event".to_string()));
        }
        if !has_mod_shutdown {
            return Err(RuntimeError::MissingExport("mod_shutdown".to_string()));
        }
        if !has_memory {
            return Err(RuntimeError::MissingExport("memory".to_string()));
        }

        Ok(())
    }

    /// Validate that the module has no WASI imports
    ///
    /// WASI would allow filesystem/network access, breaking sandbox isolation.
    fn validate_no_wasi(&self, module: &Module) -> Result<(), RuntimeError> {
        for import in module.imports() {
            let namespace = import.module();

            // Reject WASI imports
            if namespace.starts_with("wasi") || namespace == "wasi_snapshot_preview1" {
                return Err(RuntimeError::UnsupportedImport {
                    namespace: namespace.to_string(),
                    name: import.name().to_string(),
                });
            }
        }

        Ok(())
    }

    /// Get a reference to the engine
    pub fn engine(&self) -> &Engine {
        &self.engine
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RuntimeConfig;
    use crate::WasmEngine;

    fn create_test_engine() -> Engine {
        let config = RuntimeConfig::default();
        WasmEngine::new(&config).unwrap().into_inner()
    }

    #[test]
    fn test_loader_creation() {
        let engine = create_test_engine();
        let _loader = ModuleLoader::new(engine);
        // Engine is created successfully with fuel enabled (configured in WasmEngine)
    }

    #[test]
    fn test_invalid_wasm_bytes() {
        let engine = create_test_engine();
        let loader = ModuleLoader::new(engine);

        let invalid_bytes = b"not valid wasm";
        let result = loader.load_from_bytes(invalid_bytes);

        assert!(result.is_err());
        if let Err(RuntimeError::WasmInvalid(msg)) = result {
            assert!(!msg.is_empty());
        } else {
            panic!("Expected WasmInvalid error");
        }
    }

    #[test]
    fn test_load_minimal_mod() {
        let engine = create_test_engine();
        let loader = ModuleLoader::new(engine);

        // Load the pre-compiled minimal mod
        let wasm_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/minimal_mod/mod.wasm"
        );
        let wasm_bytes = std::fs::read(wasm_path).expect("Failed to read minimal mod");

        let result = loader.load_from_bytes(&wasm_bytes);
        assert!(result.is_ok(), "Failed to load minimal mod: {:?}", result);
    }

    #[test]
    fn test_missing_export_rejected() {
        let engine = create_test_engine();
        let loader = ModuleLoader::new(engine);

        // Load the mod missing mod_init export
        let wasm_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/missing_export_mod/mod.wasm"
        );
        let wasm_bytes = std::fs::read(wasm_path).expect("Failed to read missing export mod");

        let result = loader.load_from_bytes(&wasm_bytes);
        assert!(result.is_err());
        if let Err(RuntimeError::MissingExport(name)) = result {
            assert_eq!(name, "mod_init");
        } else {
            panic!("Expected MissingExport error, got: {:?}", result);
        }
    }
}
