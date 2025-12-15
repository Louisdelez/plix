//! Arena format definitions

use serde::{Deserialize, Serialize};

use plix_common::math::Vec3;
use plix_common::types::{BlockPos, BlockType, TeamId};

/// Complete arena definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Arena {
    /// Arena metadata
    pub metadata: ArenaMetadata,
    /// Spawn points
    pub spawn_points: Vec<SpawnPoint>,
    /// Block definitions
    pub blocks: BlockDefinitions,
}

/// Arena metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArenaMetadata {
    /// Arena name
    pub name: String,
    /// Arena version
    pub version: String,
    /// Arena dimensions (x, y, z)
    pub size: [u32; 3],
}

/// Spawn point definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnPoint {
    /// Team this spawn belongs to
    pub team: u8,
    /// Spawn position
    pub position: [f32; 3],
    /// Spawn rotation (yaw in degrees)
    pub rotation: f32,
}

impl SpawnPoint {
    /// Get team ID
    pub fn team_id(&self) -> TeamId {
        TeamId(self.team)
    }

    /// Get position as Vec3
    pub fn position_vec3(&self) -> Vec3 {
        Vec3::new(self.position[0], self.position[1], self.position[2])
    }

    /// Get rotation in radians
    pub fn rotation_radians(&self) -> f32 {
        self.rotation.to_radians()
    }
}

/// Block definitions for the arena
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDefinitions {
    /// Floor layer definition
    #[serde(default)]
    pub floor: Option<FloorDef>,
    /// Wall definition
    #[serde(default)]
    pub walls: Option<WallsDef>,
    /// Custom block regions
    #[serde(default)]
    pub regions: Vec<RegionDef>,
}

/// Floor layer definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloorDef {
    /// Y level of floor
    pub y: i32,
    /// Block type name
    pub block: String,
}

/// Walls definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallsDef {
    /// Whether to create border walls
    pub border: bool,
    /// Wall height
    pub height: u32,
    /// Block type name
    pub block: String,
}

/// Custom region definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionDef {
    /// Minimum corner
    pub min: [i32; 3],
    /// Maximum corner
    pub max: [i32; 3],
    /// Block type name
    pub block: String,
}

impl BlockDefinitions {
    /// Parse block type name to BlockType
    pub fn parse_block_type(name: &str) -> BlockType {
        match name.to_lowercase().as_str() {
            "air" => BlockType::AIR,
            "stone" => BlockType::STONE,
            "brick" => BlockType::BRICK,
            "metal" => BlockType::METAL,
            _ => BlockType::STONE, // Default to stone
        }
    }
}

/// Loaded arena with block data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadedArena {
    /// Original arena definition
    pub definition: Arena,
    /// Block data (3D array flattened)
    pub blocks: Vec<BlockType>,
}

impl LoadedArena {
    /// Get arena dimensions
    pub fn size(&self) -> [u32; 3] {
        self.definition.metadata.size
    }

    /// Get total spawn point count
    pub fn spawn_point_count(&self) -> usize {
        self.definition.spawn_points.len()
    }

    /// Get block at position
    pub fn get_block(&self, x: u32, y: u32, z: u32) -> BlockType {
        let [sx, sy, sz] = self.size();
        if x >= sx || y >= sy || z >= sz {
            return BlockType::AIR;
        }
        let index = (z * sy * sx + y * sx + x) as usize;
        self.blocks.get(index).copied().unwrap_or(BlockType::AIR)
    }

    /// Check if a position is solid (for collision)
    pub fn is_solid(&self, x: i32, y: i32, z: i32) -> bool {
        if x < 0 || y < 0 || z < 0 {
            return true; // Treat out of bounds as solid
        }
        let block = self.get_block(x as u32, y as u32, z as u32);
        block != BlockType::AIR
    }

    /// Get spawn points for a team
    pub fn get_team_spawns(&self, team: TeamId) -> Vec<&SpawnPoint> {
        self.definition
            .spawn_points
            .iter()
            .filter(|sp| sp.team_id() == team)
            .collect()
    }

    /// Check if a block position is within arena bounds
    pub fn is_in_bounds(&self, pos: BlockPos) -> bool {
        let [sx, sy, sz] = self.size();
        pos.x >= 0
            && pos.y >= 0
            && pos.z >= 0
            && (pos.x as u32) < sx
            && (pos.y as u32) < sy
            && (pos.z as u32) < sz
    }

    /// Get block at BlockPos
    pub fn get_block_at(&self, pos: BlockPos) -> BlockType {
        if !self.is_in_bounds(pos) {
            return BlockType::AIR;
        }
        self.get_block(pos.x as u32, pos.y as u32, pos.z as u32)
    }

    /// Set block at position (mutates arena state)
    pub fn set_block(&mut self, pos: BlockPos, block_type: BlockType) {
        if !self.is_in_bounds(pos) {
            return;
        }
        let [sx, sy, _sz] = self.size();
        let index = (pos.z as u32 * sy * sx + pos.y as u32 * sx + pos.x as u32) as usize;
        if index < self.blocks.len() {
            self.blocks[index] = block_type;
        }
    }
}
