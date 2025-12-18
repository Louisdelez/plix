//! Core identifier and entity types for Plix

use serde::{Deserialize, Serialize};
use std::fmt;

/// Type alias for chunk coordinates (same as ChunkPos, for clarity in chunk system code)
pub type ChunkCoord = ChunkPos;

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

/// Game mode for arena matches
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum GameMode {
    /// Team Deathmatch - teams score points for kills
    #[default]
    Tdm,
    /// Free-for-All - individual players score points for kills
    Ffa,
    /// Capture The Flag - teams capture enemy flags
    Ctf,
    /// Battle Royale Lite - last player standing wins
    BrLite,
    /// Training mode - sandbox practice with bots
    Training,
}

/// Unique identifier for training bots (server-local only)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BotId(pub u8);

impl BotId {
    /// Invalid/no bot
    pub const NONE: Self = Self(0xFF);

    /// Check if this is a valid bot ID
    pub fn is_valid(&self) -> bool {
        *self != Self::NONE
    }
}

impl fmt::Display for BotId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Bot({})", self.0)
    }
}

// ============================================================================
// Inventory Types
// ============================================================================

/// Unique item type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct ItemId(pub u16);

impl ItemId {
    /// Invalid/no item
    pub const NONE: Self = Self(0);
    /// Sword weapon
    pub const SWORD: Self = Self(1);
    /// Health pack consumable
    pub const HEALTH_PACK: Self = Self(2);
    /// Block placer tool
    pub const BLOCK_PLACER: Self = Self(3);
    /// Bow weapon (ranged)
    pub const BOW: Self = Self(4);
    /// Scrap resource (crafting ingredient)
    pub const SCRAP: Self = Self(5);

    /// Check if this is a valid item ID
    pub fn is_valid(&self) -> bool {
        *self != Self::NONE
    }
}

impl fmt::Display for ItemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Item({})", self.0)
    }
}

/// Unique loot entity identifier (server-assigned)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct LootEntityId(pub u32);

impl LootEntityId {
    /// Invalid/no loot entity
    pub const NONE: Self = Self(0);

    /// Check if this is a valid loot entity ID
    pub fn is_valid(&self) -> bool {
        *self != Self::NONE
    }
}

impl fmt::Display for LootEntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Loot({})", self.0)
    }
}

// ============================================================================
// Projectile Types
// ============================================================================

/// Unique projectile identifier using generational indices
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectileId {
    /// Slot index in the projectile pool (0-127)
    pub index: u16,
    /// Generation counter, incremented on slot reuse
    pub generation: u16,
}

impl ProjectileId {
    /// Invalid/no projectile
    pub const NONE: Self = Self {
        index: 0xFFFF,
        generation: 0,
    };

    /// Check if this is a valid projectile ID
    pub fn is_valid(&self) -> bool {
        self.index != 0xFFFF
    }
}

impl Default for ProjectileId {
    fn default() -> Self {
        Self::NONE
    }
}

impl fmt::Display for ProjectileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Projectile({}.{})", self.index, self.generation)
    }
}

impl std::fmt::Display for GameMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameMode::Tdm => write!(f, "TDM"),
            GameMode::Ffa => write!(f, "FFA"),
            GameMode::Ctf => write!(f, "CTF"),
            GameMode::BrLite => write!(f, "BR"),
            GameMode::Training => write!(f, "Training"),
        }
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
    /// Grass block (plains surface)
    pub const GRASS: Self = Self(4);
    /// Dirt block (plains subsurface)
    pub const DIRT: Self = Self(5);
    /// Sand block (desert surface)
    pub const SAND: Self = Self(6);
    /// Sandstone block (desert subsurface)
    pub const SANDSTONE: Self = Self(7);
    /// Bedrock block (world floor at y=0)
    pub const BEDROCK: Self = Self(8);
    /// Water block (liquid)
    pub const WATER: Self = Self(9);

    /// Check if this block is solid (not air and not liquid)
    pub fn is_solid(&self) -> bool {
        *self != Self::AIR && !self.is_liquid()
    }

    /// Check if this block type is affected by gravity (e.g., sand falls)
    pub fn is_gravity_affected(&self) -> bool {
        matches!(*self, Self::SAND)
    }

    /// Check if this block type is a liquid (e.g., water)
    pub fn is_liquid(&self) -> bool {
        matches!(*self, Self::WATER)
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

// ============================================================================
// CTF (Capture The Flag) Types
// ============================================================================

use crate::math::Vec3;
use crate::time::Tick;

/// State of a team's flag in CTF mode
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FlagState {
    /// Flag is at its home base
    AtBase,
    /// Flag is being carried by a player
    Carried {
        /// Player carrying the flag
        carrier: PlayerId,
    },
    /// Flag was dropped and is on the ground
    Dropped {
        /// Position where flag was dropped
        position: Vec3,
        /// Tick when flag will auto-return to base
        return_tick: Tick,
    },
}

impl Default for FlagState {
    fn default() -> Self {
        FlagState::AtBase
    }
}

impl FlagState {
    /// Check if flag is at base
    pub fn is_at_base(&self) -> bool {
        matches!(self, FlagState::AtBase)
    }

    /// Check if flag is being carried
    pub fn is_carried(&self) -> bool {
        matches!(self, FlagState::Carried { .. })
    }

    /// Check if flag is dropped
    pub fn is_dropped(&self) -> bool {
        matches!(self, FlagState::Dropped { .. })
    }

    /// Get carrier ID if flag is being carried
    pub fn carrier(&self) -> Option<PlayerId> {
        match self {
            FlagState::Carried { carrier } => Some(*carrier),
            _ => None,
        }
    }
}

/// A team's flag in CTF mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Flag {
    /// Team that owns this flag
    pub team: TeamId,
    /// Current state of the flag
    pub state: FlagState,
    /// Base position where flag spawns/returns
    pub base_position: Vec3,
}

impl Flag {
    /// Create a new flag at base position
    pub fn new(team: TeamId, base_position: Vec3) -> Self {
        Self {
            team,
            state: FlagState::AtBase,
            base_position,
        }
    }

    /// Get current position of the flag
    pub fn position(&self) -> Vec3 {
        match &self.state {
            FlagState::AtBase => self.base_position,
            FlagState::Carried { .. } => Vec3::ZERO, // Position comes from carrier
            FlagState::Dropped { position, .. } => *position,
        }
    }

    /// Check if flag is at its base
    pub fn is_at_base(&self) -> bool {
        self.state.is_at_base()
    }

    /// Check if flag is being carried
    pub fn is_carried(&self) -> bool {
        self.state.is_carried()
    }

    /// Get carrier ID if flag is being carried
    pub fn carrier(&self) -> Option<PlayerId> {
        self.state.carrier()
    }

    /// Reset flag to base
    pub fn reset_to_base(&mut self) {
        self.state = FlagState::AtBase;
    }
}

/// Type of CTF zone
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlagZoneType {
    /// Where the flag spawns and can be picked up
    FlagBase,
    /// Where enemy flag must be brought to capture
    CaptureZone,
}

/// A spatial zone for CTF flag interactions (axis-aligned bounding box)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagZone {
    /// Team that owns this zone
    pub team: TeamId,
    /// Type of zone (flag base or capture zone)
    pub zone_type: FlagZoneType,
    /// Minimum corner of bounding box
    pub min: Vec3,
    /// Maximum corner of bounding box
    pub max: Vec3,
}

impl FlagZone {
    /// Create a new flag zone
    pub fn new(team: TeamId, zone_type: FlagZoneType, min: Vec3, max: Vec3) -> Self {
        Self {
            team,
            zone_type,
            min,
            max,
        }
    }

    /// Check if a point is inside this zone (AABB collision)
    pub fn contains(&self, point: Vec3) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
            && point.z >= self.min.z
            && point.z <= self.max.z
    }

    /// Get the center of the zone
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
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

    // ========================================================================
    // CTF Type Tests (T005)
    // ========================================================================

    #[test]
    fn test_game_mode_ctf_serde() {
        // Test JSON serialization for GameMode::Ctf
        let mode = GameMode::Ctf;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, "\"ctf\"");

        // Test deserialization
        let parsed: GameMode = serde_json::from_str("\"ctf\"").unwrap();
        assert_eq!(parsed, GameMode::Ctf);
    }

    #[test]
    fn test_game_mode_display() {
        assert_eq!(format!("{}", GameMode::Tdm), "TDM");
        assert_eq!(format!("{}", GameMode::Ffa), "FFA");
        assert_eq!(format!("{}", GameMode::Ctf), "CTF");
        assert_eq!(format!("{}", GameMode::Training), "Training");
    }

    #[test]
    fn test_game_mode_training_serde() {
        // Test JSON serialization for GameMode::Training
        let mode = GameMode::Training;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, "\"training\"");

        // Test deserialization
        let parsed: GameMode = serde_json::from_str("\"training\"").unwrap();
        assert_eq!(parsed, GameMode::Training);
    }

    #[test]
    fn test_bot_id_valid() {
        let valid = BotId(0);
        assert!(valid.is_valid());

        let valid_max = BotId(254);
        assert!(valid_max.is_valid());

        let invalid = BotId::NONE;
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_flag_state_at_base() {
        let state = FlagState::AtBase;
        assert!(state.is_at_base());
        assert!(!state.is_carried());
        assert!(!state.is_dropped());
        assert_eq!(state.carrier(), None);
    }

    #[test]
    fn test_flag_state_carried() {
        let player = PlayerId(42);
        let state = FlagState::Carried { carrier: player };
        assert!(!state.is_at_base());
        assert!(state.is_carried());
        assert!(!state.is_dropped());
        assert_eq!(state.carrier(), Some(player));
    }

    #[test]
    fn test_flag_state_dropped() {
        let state = FlagState::Dropped {
            position: Vec3::new(10.0, 1.0, 20.0),
            return_tick: Tick(1000),
        };
        assert!(!state.is_at_base());
        assert!(!state.is_carried());
        assert!(state.is_dropped());
        assert_eq!(state.carrier(), None);
    }

    #[test]
    fn test_flag_state_serde() {
        // Test AtBase
        let at_base = FlagState::AtBase;
        let json = serde_json::to_string(&at_base).unwrap();
        let parsed: FlagState = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, at_base);

        // Test Carried
        let carried = FlagState::Carried {
            carrier: PlayerId(7),
        };
        let json = serde_json::to_string(&carried).unwrap();
        let parsed: FlagState = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, carried);

        // Test Dropped
        let dropped = FlagState::Dropped {
            position: Vec3::new(5.0, 2.0, 3.0),
            return_tick: Tick(500),
        };
        let json = serde_json::to_string(&dropped).unwrap();
        let parsed: FlagState = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, dropped);
    }

    #[test]
    fn test_flag_new_and_methods() {
        let flag = Flag::new(TeamId::TEAM_0, Vec3::new(10.0, 1.0, 10.0));
        assert_eq!(flag.team, TeamId::TEAM_0);
        assert!(flag.is_at_base());
        assert!(!flag.is_carried());
        assert_eq!(flag.carrier(), None);
        assert_eq!(flag.position(), Vec3::new(10.0, 1.0, 10.0));
    }

    #[test]
    fn test_flag_reset_to_base() {
        let mut flag = Flag::new(TeamId::TEAM_1, Vec3::new(50.0, 1.0, 50.0));
        flag.state = FlagState::Carried {
            carrier: PlayerId(5),
        };
        assert!(flag.is_carried());

        flag.reset_to_base();
        assert!(flag.is_at_base());
    }

    #[test]
    fn test_flag_zone_contains() {
        let zone = FlagZone::new(
            TeamId::TEAM_0,
            FlagZoneType::FlagBase,
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(10.0, 4.0, 10.0),
        );

        // Inside
        assert!(zone.contains(Vec3::new(5.0, 2.0, 5.0)));
        // On edge (inclusive)
        assert!(zone.contains(Vec3::new(0.0, 0.0, 0.0)));
        assert!(zone.contains(Vec3::new(10.0, 4.0, 10.0)));
        // Outside
        assert!(!zone.contains(Vec3::new(-1.0, 2.0, 5.0)));
        assert!(!zone.contains(Vec3::new(11.0, 2.0, 5.0)));
        assert!(!zone.contains(Vec3::new(5.0, 5.0, 5.0)));
    }

    #[test]
    fn test_flag_zone_center() {
        let zone = FlagZone::new(
            TeamId::TEAM_0,
            FlagZoneType::CaptureZone,
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(10.0, 4.0, 10.0),
        );
        let center = zone.center();
        assert_eq!(center.x, 5.0);
        assert_eq!(center.y, 2.0);
        assert_eq!(center.z, 5.0);
    }
}
