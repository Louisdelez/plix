//! Loot spawning utilities for death drops.

use glam::Vec3;
use plix_common::inventory::Hotbar;
use plix_common::time::Tick;
use plix_common::types::LootEntityId;
use plix_common::GameMode;

use super::LootManager;

/// Check if a game mode should drop items on death.
pub fn mode_drops_items(mode: GameMode) -> bool {
    matches!(mode, GameMode::Ffa | GameMode::BrLite)
}

/// Spawn loot items from a player's hotbar at their death position.
///
/// Items are spread in a small radius around the death position.
/// Returns the IDs of all spawned loot entities.
pub fn drop_player_items(
    hotbar: &Hotbar,
    death_pos: Vec3,
    loot_manager: &mut LootManager,
    tick: Tick,
) -> Vec<LootEntityId> {
    let mut spawned = Vec::new();

    for slot in 0..hotbar.size() {
        if let Some(stack) = hotbar.get(slot as u8) {
            // Calculate spread offset (deterministic spread based on slot index)
            // This spreads items in a circle around the death position
            let angle = slot as f32 * std::f32::consts::TAU / 9.0; // Assuming max 9 slots
            let spread_radius = 0.5 + (slot as f32 % 2.0) * 0.25; // Vary between 0.5 and 0.75

            let offset_x = angle.cos() * spread_radius;
            let offset_z = angle.sin() * spread_radius;

            let spawn_pos = Vec3::new(death_pos.x + offset_x, death_pos.y, death_pos.z + offset_z);

            let loot_id = loot_manager.spawn(spawn_pos, *stack, tick);
            spawned.push(loot_id);
        }
    }

    spawned
}

#[cfg(test)]
mod tests {
    use super::*;
    use plix_common::inventory::ItemStack;
    use plix_common::types::ItemId;

    #[test]
    fn test_mode_drops_items() {
        assert!(mode_drops_items(GameMode::Ffa));
        assert!(mode_drops_items(GameMode::BrLite));
        assert!(!mode_drops_items(GameMode::Tdm));
        assert!(!mode_drops_items(GameMode::Ctf));
        assert!(!mode_drops_items(GameMode::Training));
    }

    #[test]
    fn test_drop_player_items() {
        let mut hotbar = Hotbar::new(9);
        hotbar.set(0, Some(ItemStack::single(ItemId::SWORD)));
        hotbar.set(1, Some(ItemStack::new(ItemId::HEALTH_PACK, 5)));

        let mut loot_manager = LootManager::new();
        let death_pos = Vec3::new(10.0, 1.0, 10.0);

        let spawned = drop_player_items(&hotbar, death_pos, &mut loot_manager, Tick(100));

        assert_eq!(spawned.len(), 2);
        for id in &spawned {
            assert!(loot_manager.get(*id).is_some());
        }
    }

    #[test]
    fn test_drop_empty_hotbar() {
        let hotbar = Hotbar::new(9);
        let mut loot_manager = LootManager::new();
        let death_pos = Vec3::new(10.0, 1.0, 10.0);

        let spawned = drop_player_items(&hotbar, death_pos, &mut loot_manager, Tick(100));

        assert!(spawned.is_empty());
    }

    #[test]
    fn test_drop_spread_distance() {
        let mut hotbar = Hotbar::new(9);
        for i in 0..5 {
            hotbar.set(i, Some(ItemStack::single(ItemId::SWORD)));
        }

        let mut loot_manager = LootManager::new();
        let death_pos = Vec3::new(10.0, 1.0, 10.0);

        let spawned = drop_player_items(&hotbar, death_pos, &mut loot_manager, Tick(100));

        for id in &spawned {
            let loot = loot_manager.get(*id).unwrap();
            let distance = (loot.position - death_pos).length();
            assert!(
                distance <= 1.0,
                "Loot should be within 1.0 blocks, got {}",
                distance
            );
            assert!(
                distance >= 0.4,
                "Loot should be at least 0.4 blocks away, got {}",
                distance
            );
        }
    }
}
