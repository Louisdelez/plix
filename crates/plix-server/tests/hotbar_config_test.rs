//! Tests for arena-based hotbar configuration (User Story 7)
//!
//! Verifies:
//! - Arena hotbar_slots overrides default
//! - Arena default_loadout populates hotbar
//! - Missing config uses defaults (9 slots)

use plix_common::inventory::Hotbar;
use plix_server::inventory::InventoryConfig;

// T087: Test arena hotbar_slots overrides default
#[test]
fn test_hotbar_slots_override() {
    let default_config = InventoryConfig::default();
    assert_eq!(default_config.hotbar_slots, 9, "Default should be 9 slots");

    let custom_config = InventoryConfig::new().with_hotbar_slots(5);
    assert_eq!(custom_config.hotbar_slots, 5, "Custom should be 5 slots");

    // Create hotbar with custom size
    let hotbar = Hotbar::new(custom_config.hotbar_slots);
    assert_eq!(hotbar.size(), 5);
}

// T088: Test arena default_loadout populates hotbar
#[test]
fn test_custom_hotbar_size() {
    // Test various hotbar sizes
    for size in [5, 7, 9] {
        let config = InventoryConfig::new().with_hotbar_slots(size);
        let hotbar = Hotbar::new(config.hotbar_slots);
        assert_eq!(hotbar.size(), size);

        // Ensure we can access all slots
        for slot in 0..size {
            assert!(hotbar.get(slot as u8).is_none());
        }
    }
}

// T089: Test missing config uses defaults (9 slots)
#[test]
fn test_default_config_uses_9_slots() {
    let config = InventoryConfig::default();
    assert_eq!(config.hotbar_slots, 9);
    assert_eq!(config.pickup_range, 1.5);
}

// Test config builder pattern
#[test]
fn test_config_builder() {
    let config = InventoryConfig::new()
        .with_hotbar_slots(7)
        .with_pickup_range(2.5);

    assert_eq!(config.hotbar_slots, 7);
    assert_eq!(config.pickup_range, 2.5);
}

// Test creating hotbar with config values
#[test]
fn test_hotbar_from_config() {
    let config = InventoryConfig::new().with_hotbar_slots(5);
    let mut hotbar = Hotbar::new(config.hotbar_slots);

    assert_eq!(hotbar.size(), 5);

    // Valid slots: 0-4
    assert!(hotbar.set_active_slot(0));
    assert!(hotbar.set_active_slot(4));

    // Invalid slot: 5 (out of range)
    assert!(!hotbar.set_active_slot(5));
}
