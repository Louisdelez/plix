//! Loot table definitions for mobs and chests

use rand::prelude::*;
use serde::{Deserialize, Serialize};

use super::ids::LootTableId;
use crate::types::ItemId;

/// A loot table defining possible drops.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LootTable {
    /// Unique stable identifier
    pub loot_table_id: LootTableId,

    /// Guaranteed drops (always included)
    #[serde(default)]
    pub guaranteed: Vec<LootEntry>,

    /// Random drops (rolled based on chance)
    #[serde(default)]
    pub entries: Vec<LootEntry>,
}

impl LootTable {
    /// Roll loot using the given RNG.
    pub fn roll_with_rng<R: Rng>(&self, rng: &mut R) -> Vec<(ItemId, u32)> {
        let mut drops = Vec::new();

        // Add guaranteed drops
        for entry in &self.guaranteed {
            let qty = rng.gen_range(entry.quantity.0..=entry.quantity.1);
            if qty > 0 {
                drops.push((entry.item_id.clone(), qty));
            }
        }

        // Roll random drops
        for entry in &self.entries {
            if rng.gen::<f32>() < entry.chance {
                let qty = rng.gen_range(entry.quantity.0..=entry.quantity.1);
                if qty > 0 {
                    drops.push((entry.item_id.clone(), qty));
                }
            }
        }

        drops
    }

    /// Roll loot with an optional seed for deterministic testing.
    /// If seed is None, uses thread_rng() for randomness.
    pub fn roll(&self, seed: Option<u64>) -> Vec<(ItemId, u32)> {
        match seed {
            Some(s) => {
                let mut rng = StdRng::seed_from_u64(s);
                self.roll_with_rng(&mut rng)
            }
            None => {
                let mut rng = thread_rng();
                self.roll_with_rng(&mut rng)
            }
        }
    }

    /// Get all possible item IDs from this table (for validation)
    pub fn all_item_ids(&self) -> Vec<&ItemId> {
        self.guaranteed
            .iter()
            .chain(self.entries.iter())
            .map(|e| &e.item_id)
            .collect()
    }
}

/// A single entry in a loot table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LootEntry {
    /// Item to drop
    pub item_id: ItemId,

    /// Quantity range [min, max] (inclusive)
    pub quantity: (u32, u32),

    /// Drop chance (0.0 to 1.0)
    pub chance: f32,
}

impl LootEntry {
    /// Create a new loot entry
    pub fn new(item_id: ItemId, min_qty: u32, max_qty: u32, chance: f32) -> Self {
        Self {
            item_id,
            quantity: (min_qty, max_qty),
            chance: chance.clamp(0.0, 1.0),
        }
    }

    /// Validate the entry has sensible values
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.quantity.0 > self.quantity.1 {
            return Err("Quantity min must be <= max");
        }
        if self.quantity.0 < 1 {
            return Err("Quantity min must be >= 1");
        }
        if !(0.0..=1.0).contains(&self.chance) {
            return Err("Chance must be between 0.0 and 1.0");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_table() -> LootTable {
        LootTable {
            loot_table_id: LootTableId::new("test_drops"),
            guaranteed: vec![LootEntry {
                item_id: ItemId::new_raw(1), // bone
                quantity: (1, 2),
                chance: 1.0,
            }],
            entries: vec![
                LootEntry {
                    item_id: ItemId::new_raw(2), // rare_item
                    quantity: (1, 1),
                    chance: 0.1,
                },
                LootEntry {
                    item_id: ItemId::new_raw(3), // common_item
                    quantity: (1, 5),
                    chance: 0.5,
                },
            ],
        }
    }

    #[test]
    fn test_deterministic_roll() {
        let table = sample_table();

        // Same seed should give same results
        let roll1 = table.roll(Some(12345));
        let roll2 = table.roll(Some(12345));
        assert_eq!(roll1, roll2);

        // Different seed should (usually) give different results
        let roll3 = table.roll(Some(99999));
        // Guaranteed drops should always be present
        assert!(!roll1.is_empty());
        assert!(!roll3.is_empty());
    }

    #[test]
    fn test_guaranteed_always_drops() {
        let table = sample_table();

        // Run 100 rolls and verify guaranteed drop is always present
        for seed in 0..100 {
            let drops = table.roll(Some(seed));
            assert!(
                drops.iter().any(|(id, _)| id.raw() == 1),
                "Guaranteed drop missing at seed {}",
                seed
            );
        }
    }

    #[test]
    fn test_entry_validation() {
        let valid = LootEntry::new(ItemId::new_raw(1), 1, 5, 0.5);
        assert!(valid.validate().is_ok());

        let bad_qty = LootEntry {
            item_id: ItemId::new_raw(1),
            quantity: (5, 1), // min > max
            chance: 0.5,
        };
        assert!(bad_qty.validate().is_err());

        let bad_chance = LootEntry {
            item_id: ItemId::new_raw(1),
            quantity: (1, 5),
            chance: 1.5, // > 1.0
        };
        assert!(bad_chance.validate().is_err());
    }

    #[test]
    fn test_toml_deserialization() {
        let toml_str = r#"
            loot_table_id = "cave_rat_drops"

            [[guaranteed]]
            item_id = { raw = 1 }
            quantity = [1, 1]
            chance = 1.0

            [[entries]]
            item_id = { raw = 2 }
            quantity = [1, 2]
            chance = 0.3
        "#;

        let table: LootTable = toml::from_str(toml_str).unwrap();
        assert_eq!(table.loot_table_id.as_str(), "cave_rat_drops");
        assert_eq!(table.guaranteed.len(), 1);
        assert_eq!(table.entries.len(), 1);
    }

    #[test]
    fn test_all_item_ids() {
        let table = sample_table();
        let ids = table.all_item_ids();
        assert_eq!(ids.len(), 3);
    }
}
