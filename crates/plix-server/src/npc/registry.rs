//! NPC Registry - manages NPC instances (T079)

use std::collections::HashMap;

use plix_common::content::ids::NpcId;
use plix_common::content::npc::NpcDefinition;
use plix_common::math::Vec3;
use tracing::{debug, info};

/// NPC interaction range in blocks
pub const NPC_INTERACT_RANGE: f32 = 4.0;

/// NPC registry managing all NPC instances.
pub struct NpcRegistry {
    /// NPC definitions indexed by ID
    npcs: HashMap<NpcId, NpcDefinition>,
}

impl Default for NpcRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl NpcRegistry {
    /// Create a new NPC registry.
    pub fn new() -> Self {
        Self {
            npcs: HashMap::new(),
        }
    }

    /// Register an NPC definition.
    pub fn register(&mut self, npc: NpcDefinition) {
        info!(
            npc_id = %npc.npc_id,
            name = %npc.name,
            "Registered NPC"
        );
        self.npcs.insert(npc.npc_id.clone(), npc);
    }

    /// Get an NPC by ID.
    pub fn get(&self, npc_id: &NpcId) -> Option<&NpcDefinition> {
        self.npcs.get(npc_id)
    }

    /// Get an NPC by ID (mutable).
    pub fn get_mut(&mut self, npc_id: &NpcId) -> Option<&mut NpcDefinition> {
        self.npcs.get_mut(npc_id)
    }

    /// Find NPCs within range of a position.
    pub fn npcs_in_range(&self, pos: &Vec3, range: f32) -> Vec<&NpcDefinition> {
        self.npcs
            .values()
            .filter(|npc| {
                let dist = (npc.position - *pos).length();
                dist <= range
            })
            .collect()
    }

    /// Find the closest NPC to a position within interaction range.
    pub fn closest_interactable(&self, pos: &Vec3) -> Option<&NpcDefinition> {
        self.npcs_in_range(pos, NPC_INTERACT_RANGE)
            .into_iter()
            .min_by(|a, b| {
                let dist_a = (a.position - *pos).length();
                let dist_b = (b.position - *pos).length();
                dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Check if a player position can interact with an NPC.
    pub fn can_interact(&self, npc_id: &NpcId, player_pos: &Vec3) -> bool {
        if let Some(npc) = self.npcs.get(npc_id) {
            let dist = (npc.position - *player_pos).length();
            dist <= NPC_INTERACT_RANGE
        } else {
            false
        }
    }

    /// Get count of registered NPCs.
    pub fn count(&self) -> usize {
        self.npcs.len()
    }

    /// Iterate over all NPCs.
    pub fn iter(&self) -> impl Iterator<Item = (&NpcId, &NpcDefinition)> {
        self.npcs.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plix_common::content::ids::QuestId;

    fn sample_npc(id: &str, x: f32, z: f32) -> NpcDefinition {
        NpcDefinition {
            npc_id: NpcId::new(id),
            name: format!("NPC {}", id),
            position: Vec3::new(x, 0.0, z),
            rotation: 0.0,
            quest_giver: vec![],
            idle_dialogue: vec!["Hello!".to_string()],
            quest_offer_dialogue: Default::default(),
            quest_complete_dialogue: Default::default(),
        }
    }

    #[test]
    fn test_registry_basic() {
        let mut registry = NpcRegistry::new();
        assert_eq!(registry.count(), 0);

        registry.register(sample_npc("elder", 50.0, 50.0));
        assert_eq!(registry.count(), 1);
        assert!(registry.get(&NpcId::new("elder")).is_some());
        assert!(registry.get(&NpcId::new("unknown")).is_none());
    }

    #[test]
    fn test_npcs_in_range() {
        let mut registry = NpcRegistry::new();
        registry.register(sample_npc("near", 0.0, 0.0));
        registry.register(sample_npc("far", 100.0, 100.0));

        let pos = Vec3::new(0.0, 0.0, 0.0);
        let in_range = registry.npcs_in_range(&pos, 10.0);
        assert_eq!(in_range.len(), 1);
        assert_eq!(in_range[0].npc_id.as_str(), "near");
    }

    #[test]
    fn test_closest_interactable() {
        let mut registry = NpcRegistry::new();
        registry.register(sample_npc("closest", 1.0, 1.0));
        registry.register(sample_npc("further", 3.0, 3.0));
        registry.register(sample_npc("too_far", 10.0, 10.0));

        let pos = Vec3::new(0.0, 0.0, 0.0);
        let closest = registry.closest_interactable(&pos);
        assert!(closest.is_some());
        assert_eq!(closest.unwrap().npc_id.as_str(), "closest");
    }

    #[test]
    fn test_can_interact() {
        let mut registry = NpcRegistry::new();
        registry.register(sample_npc("near", 2.0, 0.0));

        let near_pos = Vec3::new(0.0, 0.0, 0.0);
        let far_pos = Vec3::new(100.0, 0.0, 0.0);

        assert!(registry.can_interact(&NpcId::new("near"), &near_pos));
        assert!(!registry.can_interact(&NpcId::new("near"), &far_pos));
        assert!(!registry.can_interact(&NpcId::new("unknown"), &near_pos));
    }
}
