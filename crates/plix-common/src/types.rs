//! Core identifier and entity types for Plix

use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique player identifier (assigned by server)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlayerId(pub u16);

impl PlayerId {
    /// Invalid/no player
    pub const NONE: Self = Self(0xFFFF);

    /// Check if this is a valid player ID
    pub fn is_valid(&self) -> bool {
        *self != Self::NONE
    }
}

impl fmt::Display for PlayerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Player({})", self.0)
    }
}

/// Unique entity identifier (server-assigned)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(pub u32);

impl EntityId {
    /// Invalid/no entity
    pub const NONE: Self = Self(0);
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Entity({})", self.0)
    }
}

/// Input sequence number (per-player, wraps at u16::MAX)
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub struct InputSeq(pub u16);

impl InputSeq {
    /// Get the next sequence number (wrapping)
    pub fn next(self) -> Self {
        Self(self.0.wrapping_add(1))
    }

    /// Check if self is more recent than other (handles wrapping)
    pub fn is_newer_than(self, other: Self) -> bool {
        // Handle wrap-around: if difference is > half the range, it wrapped
        let diff = self.0.wrapping_sub(other.0);
        diff > 0 && diff < 32768
    }
}

impl fmt::Display for InputSeq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Seq({})", self.0)
    }
}

/// Team identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct TeamId(pub u8);

impl TeamId {
    /// No team / spectator
    pub const NONE: Self = Self(0xFF);
    /// Team 0 (e.g., Red)
    pub const TEAM_0: Self = Self(0);
    /// Team 1 (e.g., Blue)
    pub const TEAM_1: Self = Self(1);
}

impl fmt::Display for TeamId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if *self == Self::NONE {
            write!(f, "NoTeam")
        } else {
            write!(f, "Team({})", self.0)
        }
    }
}

/// Block type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct BlockType(pub u8);

impl BlockType {
    /// Air (empty block)
    pub const AIR: Self = Self(0);
    /// Stone block
    pub const STONE: Self = Self(1);
    /// Brick block
    pub const BRICK: Self = Self(2);
    /// Metal block
    pub const METAL: Self = Self(3);

    /// Check if this block is solid (not air)
    pub fn is_solid(&self) -> bool {
        *self != Self::AIR
    }
}

/// Block position (integer coordinates)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    /// Create a new block position
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// Origin block position
    pub const ZERO: Self = Self { x: 0, y: 0, z: 0 };

    /// Convert from floating-point position
    pub fn from_vec3(v: crate::math::Vec3) -> Self {
        Self {
            x: v.x.floor() as i32,
            y: v.y.floor() as i32,
            z: v.z.floor() as i32,
        }
    }

    /// Convert to floating-point position (block center)
    pub fn to_vec3(self) -> crate::math::Vec3 {
        crate::math::Vec3::new(
            self.x as f32 + 0.5,
            self.y as f32 + 0.5,
            self.z as f32 + 0.5,
        )
    }

    /// Get chunk position this block belongs to
    pub fn chunk_pos(&self) -> ChunkPos {
        ChunkPos {
            x: self.x.div_euclid(16),
            y: self.y.div_euclid(16),
            z: self.z.div_euclid(16),
        }
    }

    /// Get local position within chunk (0-15)
    pub fn local_pos(&self) -> (usize, usize, usize) {
        (
            self.x.rem_euclid(16) as usize,
            self.y.rem_euclid(16) as usize,
            self.z.rem_euclid(16) as usize,
        )
    }
}

impl fmt::Display for BlockPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}, {}]", self.x, self.y, self.z)
    }
}

/// Chunk position in world (each chunk is 16x16x16 blocks)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChunkPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl ChunkPos {
    /// Create a new chunk position
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// Get the world-space block position of the chunk's origin
    pub fn origin_block(&self) -> BlockPos {
        BlockPos {
            x: self.x * 16,
            y: self.y * 16,
            z: self.z * 16,
        }
    }
}

impl fmt::Display for ChunkPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Chunk[{}, {}, {}]", self.x, self.y, self.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_seq_wrapping() {
        let seq = InputSeq(u16::MAX);
        assert_eq!(seq.next(), InputSeq(0));
    }

    #[test]
    fn test_input_seq_newer() {
        assert!(InputSeq(5).is_newer_than(InputSeq(3)));
        assert!(!InputSeq(3).is_newer_than(InputSeq(5)));
        // Wrap-around case
        assert!(InputSeq(1).is_newer_than(InputSeq(65535)));
    }

    #[test]
    fn test_block_pos_chunk() {
        let pos = BlockPos::new(17, -5, 32);
        assert_eq!(pos.chunk_pos(), ChunkPos::new(1, -1, 2));

        let local = pos.local_pos();
        assert_eq!(local, (1, 11, 0));
    }
}
