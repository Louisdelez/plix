//! Snapshot generation for clients

use plix_common::inventory::Hotbar;
use plix_common::protocol::{
    InventorySnapshot, MatchState, PlayerSnapshot, SlotSnapshot, WorldSnapshot,
};
use plix_common::time::Tick;
use plix_common::types::{InputSeq, PlayerId};

use super::state::ReplicatedState;

/// Create an inventory snapshot from a player's hotbar (T027)
pub fn create_inventory_snapshot(hotbar: &Hotbar) -> InventorySnapshot {
    let slots: Vec<SlotSnapshot> = hotbar
        .slots()
        .iter()
        .map(|slot| match slot {
            Some(stack) => SlotSnapshot::new(stack.item_id, stack.quantity),
            None => SlotSnapshot::empty(),
        })
        .collect();

    InventorySnapshot {
        slots,
        active_slot: hotbar.active_slot(),
    }
}

/// Generates world snapshots for clients
#[derive(Debug, Default)]
pub struct SnapshotGenerator;

impl SnapshotGenerator {
    /// Create a new snapshot generator
    pub fn new() -> Self {
        Self
    }

    /// Generate a snapshot for a specific client
    pub fn generate(
        &self,
        state: &ReplicatedState,
        match_state: &MatchState,
        tick: Tick,
        client_id: PlayerId,
        last_input_seq: InputSeq,
    ) -> WorldSnapshot {
        // Convert replicated players to snapshots
        let players: Vec<PlayerSnapshot> = state
            .players
            .iter()
            .map(|p| PlayerSnapshot {
                id: p.id,
                position: p.position,
                rotation: p.rotation,
                health: p.health,
                is_dead: p.is_dead,
                animation: p.animation,
                spectate_target: p.spectate_target,
                display_name: p.display_name.clone(),
            })
            .collect();

        WorldSnapshot {
            tick,
            last_input_seq,
            players,
            match_state: match_state.clone(),
            rtt_nonce_echo: 0, // RTT echo is set per-client in main server loop
            bots: Vec::new(),  // Populated by TrainingCoordinator in training mode
        }
    }

    /// Generate a delta snapshot (only changed state since last_tick)
    /// For MVP, we just generate full snapshots
    pub fn generate_delta(
        &self,
        state: &ReplicatedState,
        match_state: &MatchState,
        tick: Tick,
        _client_id: PlayerId,
        last_input_seq: InputSeq,
        _last_tick: Tick,
    ) -> WorldSnapshot {
        // TODO: Implement delta encoding for bandwidth optimization
        self.generate(state, match_state, tick, _client_id, last_input_seq)
    }
}
