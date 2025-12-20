//! Mod registry: tracks loaded mods and their runtime state

use crate::capabilities::{Capability, CapabilityPolicy};
use crate::errors::{err_not_found, ModApiError};
use crate::events::EventType;
use crate::manifest::ModManifest;
use std::collections::{HashMap, HashSet};
use tracing::{info, warn};

/// Maximum consecutive errors before auto-disable
pub const ERROR_THRESHOLD: u32 = 5;

/// Lifecycle state of a mod
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModState {
    /// Manifest validated, initializing
    Loading,
    /// Fully operational
    Active,
    /// Disabled due to errors or admin action
    Disabled,
}

/// Runtime state for a loaded mod instance
#[derive(Debug)]
pub struct ModContext {
    /// Unique mod identifier
    pub id: String,
    /// Parsed manifest
    pub manifest: ModManifest,
    /// Effective permissions (manifest ∩ server policy)
    pub granted_capabilities: Capability,
    /// Current mod state
    pub state: ModState,
    /// Consecutive handler errors (resets on success)
    pub error_count: u32,
    /// Subscribed event types
    pub subscriptions: HashSet<EventType>,
    /// Reason for disable (if disabled)
    pub disable_reason: Option<String>,
}

impl ModContext {
    /// Create a new mod context in Loading state
    pub fn new(manifest: ModManifest, granted_capabilities: Capability) -> Self {
        Self {
            id: manifest.id.clone(),
            manifest,
            granted_capabilities,
            state: ModState::Loading,
            error_count: 0,
            subscriptions: HashSet::new(),
            disable_reason: None,
        }
    }

    /// Check if mod has a specific capability
    pub fn has_capability(&self, cap: Capability) -> bool {
        self.granted_capabilities.contains(cap)
    }

    /// Record a successful handler execution (resets error count)
    pub fn record_success(&mut self) {
        self.error_count = 0;
    }

    /// Record a handler error, returns true if mod should be disabled
    pub fn record_error(&mut self) -> bool {
        self.error_count += 1;
        self.error_count >= ERROR_THRESHOLD
    }

    /// Subscribe to an event type
    pub fn subscribe(&mut self, event_type: EventType) {
        self.subscriptions.insert(event_type);
    }

    /// Unsubscribe from an event type
    pub fn unsubscribe(&mut self, event_type: EventType) -> bool {
        self.subscriptions.remove(&event_type)
    }

    /// Check if subscribed to an event type
    pub fn is_subscribed(&self, event_type: EventType) -> bool {
        self.subscriptions.contains(&event_type)
    }
}

/// Registry of all loaded mods
#[derive(Debug, Default)]
pub struct ModRegistry {
    /// Loaded mods by ID
    mods: HashMap<String, ModContext>,
    /// Server capability policy
    policy: CapabilityPolicy,
}

impl ModRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            mods: HashMap::new(),
            policy: CapabilityPolicy::permissive(),
        }
    }

    /// Create a registry with a custom capability policy
    pub fn with_policy(policy: CapabilityPolicy) -> Self {
        Self {
            mods: HashMap::new(),
            policy,
        }
    }

    /// Set the capability policy
    pub fn set_policy(&mut self, policy: CapabilityPolicy) {
        self.policy = policy;
    }

    /// Get the capability policy
    pub fn policy(&self) -> &CapabilityPolicy {
        &self.policy
    }

    /// Load a mod from its manifest
    pub fn load(&mut self, manifest: ModManifest) -> Result<(), ModApiError> {
        let mod_id = manifest.id.clone();

        if self.mods.contains_key(&mod_id) {
            return Err(err_not_found(format!("Mod '{}' is already loaded", mod_id)));
        }

        // Calculate effective capabilities
        let manifest_caps = manifest.capabilities.to_capability();
        let effective_caps = self.policy.effective_capabilities(&mod_id, manifest_caps);

        let mut context = ModContext::new(manifest, effective_caps);
        context.state = ModState::Active;

        info!(mod_id = %mod_id, capabilities = ?effective_caps, "Mod loaded");
        self.mods.insert(mod_id, context);

        Ok(())
    }

    /// Unload a mod
    pub fn unload(&mut self, mod_id: &str) -> Result<ModContext, ModApiError> {
        self.mods
            .remove(mod_id)
            .ok_or_else(|| err_not_found(format!("Mod '{}' not found", mod_id)))
    }

    /// Get a mod context by ID
    pub fn get(&self, mod_id: &str) -> Option<&ModContext> {
        self.mods.get(mod_id)
    }

    /// Get a mutable mod context by ID
    pub fn get_mut(&mut self, mod_id: &str) -> Option<&mut ModContext> {
        self.mods.get_mut(mod_id)
    }

    /// Check if a mod is loaded
    pub fn is_loaded(&self, mod_id: &str) -> bool {
        self.mods.contains_key(mod_id)
    }

    /// Check if a mod is enabled (active state)
    pub fn is_enabled(&self, mod_id: &str) -> bool {
        self.mods
            .get(mod_id)
            .map(|ctx| ctx.state == ModState::Active)
            .unwrap_or(false)
    }

    /// Disable a mod with a reason
    pub fn disable_mod(
        &mut self,
        mod_id: &str,
        reason: impl Into<String>,
    ) -> Result<(), ModApiError> {
        let context = self
            .mods
            .get_mut(mod_id)
            .ok_or_else(|| err_not_found(format!("Mod '{}' not found", mod_id)))?;

        let reason = reason.into();
        warn!(mod_id = %mod_id, reason = %reason, "Mod disabled");

        context.state = ModState::Disabled;
        context.disable_reason = Some(reason);

        Ok(())
    }

    /// Enable a previously disabled mod
    pub fn enable_mod(&mut self, mod_id: &str) -> Result<(), ModApiError> {
        let context = self
            .mods
            .get_mut(mod_id)
            .ok_or_else(|| err_not_found(format!("Mod '{}' not found", mod_id)))?;

        info!(mod_id = %mod_id, "Mod enabled");
        context.state = ModState::Active;
        context.error_count = 0;
        context.disable_reason = None;

        Ok(())
    }

    /// Get all loaded mod IDs
    pub fn mod_ids(&self) -> impl Iterator<Item = &str> {
        self.mods.keys().map(|s| s.as_str())
    }

    /// Get all active (enabled) mod IDs
    pub fn active_mod_ids(&self) -> impl Iterator<Item = &str> {
        self.mods
            .iter()
            .filter(|(_, ctx)| ctx.state == ModState::Active)
            .map(|(id, _)| id.as_str())
    }

    /// Get count of loaded mods
    pub fn mod_count(&self) -> usize {
        self.mods.len()
    }

    /// Get count of active mods
    pub fn active_mod_count(&self) -> usize {
        self.mods
            .values()
            .filter(|ctx| ctx.state == ModState::Active)
            .count()
    }

    /// Iterate over all mod contexts
    pub fn iter(&self) -> impl Iterator<Item = (&str, &ModContext)> {
        self.mods.iter().map(|(id, ctx)| (id.as_str(), ctx))
    }

    /// Iterate over all active mod contexts
    pub fn iter_active(&self) -> impl Iterator<Item = (&str, &ModContext)> {
        self.mods
            .iter()
            .filter(|(_, ctx)| ctx.state == ModState::Active)
            .map(|(id, ctx)| (id.as_str(), ctx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_manifest(id: &str) -> ModManifest {
        ModManifest {
            id: id.to_string(),
            name: format!("Test Mod {}", id),
            version: "1.0.0".to_string(),
            author: None,
            api_version: 1,
            min_api_version: None,
            max_api_version: None,
            capabilities: Default::default(),
            entrypoints: Default::default(),
            dependencies: vec![],
        }
    }

    #[test]
    fn test_registry_load() {
        let mut registry = ModRegistry::new();
        let manifest = test_manifest("test-mod");

        assert!(registry.load(manifest).is_ok());
        assert!(registry.is_loaded("test-mod"));
        assert!(registry.is_enabled("test-mod"));
    }

    #[test]
    fn test_registry_duplicate_load() {
        let mut registry = ModRegistry::new();
        let manifest1 = test_manifest("test-mod");
        let manifest2 = test_manifest("test-mod");

        assert!(registry.load(manifest1).is_ok());
        assert!(registry.load(manifest2).is_err());
    }

    #[test]
    fn test_registry_unload() {
        let mut registry = ModRegistry::new();
        let manifest = test_manifest("test-mod");

        registry.load(manifest).unwrap();
        assert!(registry.is_loaded("test-mod"));

        let ctx = registry.unload("test-mod").unwrap();
        assert_eq!(ctx.id, "test-mod");
        assert!(!registry.is_loaded("test-mod"));
    }

    #[test]
    fn test_registry_disable_enable() {
        let mut registry = ModRegistry::new();
        let manifest = test_manifest("test-mod");

        registry.load(manifest).unwrap();
        assert!(registry.is_enabled("test-mod"));

        registry.disable_mod("test-mod", "test reason").unwrap();
        assert!(!registry.is_enabled("test-mod"));
        assert_eq!(
            registry.get("test-mod").unwrap().disable_reason,
            Some("test reason".to_string())
        );

        registry.enable_mod("test-mod").unwrap();
        assert!(registry.is_enabled("test-mod"));
        assert!(registry.get("test-mod").unwrap().disable_reason.is_none());
    }

    #[test]
    fn test_mod_context_error_tracking() {
        let manifest = test_manifest("test-mod");
        let mut ctx = ModContext::new(manifest, Capability::empty());

        // Record 4 errors - should not trigger disable
        for _ in 0..4 {
            assert!(!ctx.record_error());
        }
        assert_eq!(ctx.error_count, 4);

        // 5th error triggers disable
        assert!(ctx.record_error());
        assert_eq!(ctx.error_count, 5);

        // Success resets count
        ctx.record_success();
        assert_eq!(ctx.error_count, 0);
    }

    #[test]
    fn test_mod_context_subscriptions() {
        let manifest = test_manifest("test-mod");
        let mut ctx = ModContext::new(manifest, Capability::empty());

        ctx.subscribe(EventType::PlayerJoin);
        assert!(ctx.is_subscribed(EventType::PlayerJoin));
        assert!(!ctx.is_subscribed(EventType::PlayerChat));

        ctx.unsubscribe(EventType::PlayerJoin);
        assert!(!ctx.is_subscribed(EventType::PlayerJoin));
    }

    #[test]
    fn test_registry_counts() {
        let mut registry = ModRegistry::new();

        registry.load(test_manifest("mod1")).unwrap();
        registry.load(test_manifest("mod2")).unwrap();
        registry.load(test_manifest("mod3")).unwrap();

        assert_eq!(registry.mod_count(), 3);
        assert_eq!(registry.active_mod_count(), 3);

        registry.disable_mod("mod2", "test").unwrap();

        assert_eq!(registry.mod_count(), 3);
        assert_eq!(registry.active_mod_count(), 2);
    }

    #[test]
    fn test_registry_policy_integration() {
        let mut policy = CapabilityPolicy::permissive();
        policy.deny_from_mod("restricted", Capability::WORLD_WRITE);

        let mut registry = ModRegistry::with_policy(policy);

        let mut manifest = test_manifest("restricted");
        manifest.capabilities.world_read = true;
        manifest.capabilities.world_write = true;

        registry.load(manifest).unwrap();

        let ctx = registry.get("restricted").unwrap();
        assert!(ctx.has_capability(Capability::WORLD_READ));
        assert!(!ctx.has_capability(Capability::WORLD_WRITE));
    }
}
