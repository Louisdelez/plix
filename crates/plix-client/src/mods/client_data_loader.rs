//! Client data loader for accessing synchronized mod data
//!
//! Provides access to data-only payloads synced from the server.

use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

/// Registry for client mod data loaded from payloads
pub struct ClientModDataRegistry {
    /// Loaded data by mod ID
    data: HashMap<String, ModData>,
}

/// Data loaded from a single mod's client payload
#[derive(Debug, Clone)]
pub struct ModData {
    /// Mod identifier
    pub mod_id: String,
    /// JSON data files by path
    pub json_files: HashMap<String, JsonValue>,
    /// TOML data files by path
    pub toml_files: HashMap<String, toml::Value>,
    /// Raw data files by path
    pub raw_files: HashMap<String, Vec<u8>>,
}

impl Default for ClientModDataRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientModDataRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Check if a mod's data is loaded
    pub fn has_mod(&self, mod_id: &str) -> bool {
        self.data.contains_key(mod_id)
    }

    /// Get data for a mod
    pub fn get_mod(&self, mod_id: &str) -> Option<&ModData> {
        self.data.get(mod_id)
    }

    /// Load mod data from a payload archive (ZIP)
    pub fn load_from_archive(&mut self, mod_id: &str, archive_data: &[u8]) -> Result<(), String> {
        let cursor = std::io::Cursor::new(archive_data);
        let mut archive = zip::ZipArchive::new(cursor)
            .map_err(|e| format!("Failed to open archive: {}", e))?;

        let mut mod_data = ModData {
            mod_id: mod_id.to_string(),
            json_files: HashMap::new(),
            toml_files: HashMap::new(),
            raw_files: HashMap::new(),
        };

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| format!("Failed to read archive entry: {}", e))?;

            if file.is_dir() {
                continue;
            }

            let name = file.name().to_string();
            let mut contents = Vec::new();
            file.read_to_end(&mut contents)
                .map_err(|e| format!("Failed to read file {}: {}", name, e))?;

            // Parse based on extension
            let path = Path::new(&name);
            match path.extension().and_then(|e| e.to_str()) {
                Some("json") => {
                    let value: JsonValue = serde_json::from_slice(&contents)
                        .map_err(|e| format!("Failed to parse JSON {}: {}", name, e))?;
                    mod_data.json_files.insert(name, value);
                }
                Some("toml") => {
                    let content = String::from_utf8(contents.clone())
                        .map_err(|e| format!("Invalid UTF-8 in TOML {}: {}", name, e))?;
                    let value: toml::Value = toml::from_str(&content)
                        .map_err(|e| format!("Failed to parse TOML {}: {}", name, e))?;
                    mod_data.toml_files.insert(name, value);
                }
                _ => {
                    mod_data.raw_files.insert(name, contents);
                }
            }
        }

        self.data.insert(mod_id.to_string(), mod_data);
        Ok(())
    }

    /// Unload a mod's data
    pub fn unload_mod(&mut self, mod_id: &str) {
        self.data.remove(mod_id);
    }

    /// Clear all loaded data
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Get list of loaded mod IDs
    pub fn loaded_mods(&self) -> Vec<&str> {
        self.data.keys().map(|s| s.as_str()).collect()
    }
}

impl ModData {
    /// Get a JSON file by path
    pub fn get_json(&self, path: &str) -> Option<&JsonValue> {
        self.json_files.get(path)
    }

    /// Get a TOML file by path
    pub fn get_toml(&self, path: &str) -> Option<&toml::Value> {
        self.toml_files.get(path)
    }

    /// Get a raw file by path
    pub fn get_raw(&self, path: &str) -> Option<&[u8]> {
        self.raw_files.get(path).map(|v| v.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_operations() {
        let mut registry = ClientModDataRegistry::new();

        assert!(!registry.has_mod("test-mod"));
        assert!(registry.loaded_mods().is_empty());

        // Note: Full archive loading test would require creating a valid ZIP
        // This tests the basic API structure
    }
}
