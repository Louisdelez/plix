//! Block physics events
//!
//! Event types for the block physics system.

use crate::types::BlockPos;
use serde::{Deserialize, Serialize};

/// Physics event kind - what type of physics update to perform
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BlockPhysicsEventKind {
    /// Block should check if it can fall (gravity-affected blocks)
    Fall,
    /// Liquid should try to spread, with current depth from source
    LiquidSpread { depth: u8 },
}

impl BlockPhysicsEventKind {
    /// Get discriminant value for deduplication (0 = Fall, 1 = LiquidSpread)
    pub fn discriminant(&self) -> u8 {
        match self {
            BlockPhysicsEventKind::Fall => 0,
            BlockPhysicsEventKind::LiquidSpread { .. } => 1,
        }
    }
}

/// A single physics update event to be processed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockPhysicsEvent {
    /// World position of the block
    pub pos: BlockPos,
    /// Type of physics update
    pub kind: BlockPhysicsEventKind,
}

impl BlockPhysicsEvent {
    /// Create a new fall event at the given position
    pub fn fall(pos: BlockPos) -> Self {
        Self {
            pos,
            kind: BlockPhysicsEventKind::Fall,
        }
    }

    /// Create a new liquid spread event at the given position with depth
    pub fn liquid_spread(pos: BlockPos, depth: u8) -> Self {
        Self {
            pos,
            kind: BlockPhysicsEventKind::LiquidSpread { depth },
        }
    }

    /// Get deduplication key (pos + kind discriminant)
    /// Two events with the same dedup key are considered duplicates
    pub fn dedup_key(&self) -> (BlockPos, u8) {
        (self.pos, self.kind.discriminant())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fall_event() {
        let pos = BlockPos::new(1, 2, 3);
        let event = BlockPhysicsEvent::fall(pos);
        assert_eq!(event.pos, pos);
        assert_eq!(event.kind, BlockPhysicsEventKind::Fall);
    }

    #[test]
    fn test_liquid_spread_event() {
        let pos = BlockPos::new(4, 5, 6);
        let event = BlockPhysicsEvent::liquid_spread(pos, 3);
        assert_eq!(event.pos, pos);
        assert_eq!(event.kind, BlockPhysicsEventKind::LiquidSpread { depth: 3 });
    }

    #[test]
    fn test_discriminant() {
        assert_eq!(BlockPhysicsEventKind::Fall.discriminant(), 0);
        assert_eq!(
            BlockPhysicsEventKind::LiquidSpread { depth: 0 }.discriminant(),
            1
        );
        assert_eq!(
            BlockPhysicsEventKind::LiquidSpread { depth: 5 }.discriminant(),
            1
        );
    }

    #[test]
    fn test_dedup_key() {
        let pos = BlockPos::new(1, 2, 3);

        let fall1 = BlockPhysicsEvent::fall(pos);
        let fall2 = BlockPhysicsEvent::fall(pos);
        assert_eq!(fall1.dedup_key(), fall2.dedup_key());

        let liquid1 = BlockPhysicsEvent::liquid_spread(pos, 0);
        let liquid2 = BlockPhysicsEvent::liquid_spread(pos, 5);
        // Same position and kind discriminant = same dedup key
        assert_eq!(liquid1.dedup_key(), liquid2.dedup_key());

        // Different kind = different dedup key
        assert_ne!(fall1.dedup_key(), liquid1.dedup_key());
    }
}
