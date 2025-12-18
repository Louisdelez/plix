//! Arena format definitions

use serde::{Deserialize, Serialize};

use plix_common::block_physics::BlockWorld;
use plix_common::math::Vec3;
use plix_common::types::{BlockPos, BlockType, FlagZone, FlagZoneType, GameMode, TeamId};
use plix_common::ChunkedWorld;

/// Complete arena definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Arena {
    /// Arena metadata
    pub metadata: ArenaMetadata,
    /// Spawn points
    pub spawn_points: Vec<SpawnPoint>,
    /// Block definitions
    pub blocks: BlockDefinitions,
    /// CTF flag zones (optional, required for CTF game mode)
    #[serde(default)]
    pub flag_zones: Vec<FlagZoneDefinition>,
    /// BR Lite configuration (optional, for BR Lite game mode)
    #[serde(default)]
    pub br_lite: Option<BrLiteArenaConfig>,
    /// Training mode configuration (optional, for Training game mode)
    #[serde(default)]
    pub training: Option<TrainingArenaConfig>,
    /// Economy configuration (optional, overrides mode defaults)
    #[serde(default)]
    pub economy: Option<EconomyArenaConfig>,
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
    /// Game mode for this arena (defaults to TDM for backward compatibility)
    #[serde(default)]
    pub game_mode: GameMode,
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

/// CTF flag zone definition (parsed from TOML)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagZoneDefinition {
    /// Team this zone belongs to (0 or 1)
    pub team: u8,
    /// Zone type: "flag_base" or "capture_zone"
    pub zone_type: String,
    /// Minimum corner (x, y, z)
    pub min: [f32; 3],
    /// Maximum corner (x, y, z)
    pub max: [f32; 3],
}

impl FlagZoneDefinition {
    /// Convert to FlagZone type used by game logic
    pub fn to_flag_zone(&self) -> FlagZone {
        let team = TeamId(self.team);
        let zone_type = match self.zone_type.to_lowercase().as_str() {
            "flag_base" => FlagZoneType::FlagBase,
            "capture_zone" => FlagZoneType::CaptureZone,
            _ => FlagZoneType::FlagBase, // Default to flag base
        };
        let min = Vec3::new(self.min[0], self.min[1], self.min[2]);
        let max = Vec3::new(self.max[0], self.max[1], self.max[2]);

        FlagZone::new(team, zone_type, min, max)
    }
}

/// BR Lite arena configuration (for Battle Royale mode)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrLiteArenaConfig {
    /// Zone center (XZ coordinates) - defaults to arena center if not specified
    #[serde(default)]
    pub zone_center: Option<[f32; 2]>,
    /// Initial zone radius - defaults to half of smallest arena dimension
    #[serde(default)]
    pub initial_radius: Option<f32>,
    /// Minimum players to start (defaults to 4)
    #[serde(default = "default_min_players")]
    pub min_players: u8,
    /// Post-match delay in seconds (defaults to 10)
    #[serde(default = "default_post_match_delay")]
    pub post_match_delay_secs: u32,
    /// Bonus duration in seconds for temporary effects (defaults to 10)
    #[serde(default = "default_bonus_duration")]
    pub bonus_duration_secs: u32,
    /// Whether loot is enabled (defaults to true)
    #[serde(default = "default_loot_enabled")]
    pub loot_enabled: bool,
    /// Loot spawn points
    #[serde(default)]
    pub loot_spawns: Vec<LootSpawnConfig>,
}

fn default_min_players() -> u8 {
    4
}
fn default_post_match_delay() -> u32 {
    10
}
fn default_bonus_duration() -> u32 {
    10
}
fn default_loot_enabled() -> bool {
    true
}

impl Default for BrLiteArenaConfig {
    fn default() -> Self {
        Self {
            zone_center: None,
            initial_radius: None,
            min_players: 4,
            post_match_delay_secs: 10,
            bonus_duration_secs: 10,
            loot_enabled: true,
            loot_spawns: Vec::new(),
        }
    }
}

impl BrLiteArenaConfig {
    /// Validate the BR Lite configuration against arena bounds
    pub fn validate(&self, arena_size: [u32; 3]) -> Result<(), String> {
        // Validate zone center is within arena bounds
        if let Some([cx, cz]) = self.zone_center {
            if cx < 0.0 || cz < 0.0 || cx > arena_size[0] as f32 || cz > arena_size[2] as f32 {
                return Err(format!(
                    "zone_center ({}, {}) is outside arena bounds (0-{}, 0-{})",
                    cx, cz, arena_size[0], arena_size[2]
                ));
            }
        }

        // Validate initial radius is positive
        if let Some(radius) = self.initial_radius {
            if radius <= 0.0 {
                return Err("initial_radius must be positive".to_string());
            }
        }

        // Validate min_players
        if self.min_players < 2 {
            return Err("min_players must be at least 2".to_string());
        }
        if self.min_players > 16 {
            return Err("min_players cannot exceed 16".to_string());
        }

        // Validate loot spawn positions
        for (i, loot) in self.loot_spawns.iter().enumerate() {
            loot.validate(arena_size)
                .map_err(|e| format!("loot_spawns[{}]: {}", i, e))?;
        }

        Ok(())
    }
}

/// Loot spawn configuration for BR Lite arenas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LootSpawnConfig {
    /// Spawn position (x, y, z)
    pub position: [f32; 3],
    /// Loot type: "health_pack" or "speed_boost"
    pub loot_type: String,
    /// Loot parameter (heal_amount for health, multiplier*100 for speed)
    #[serde(default = "default_loot_param")]
    pub param: u16,
}

fn default_loot_param() -> u16 {
    25 // Default heal amount / 1.25x speed multiplier
}

impl LootSpawnConfig {
    /// Get position as Vec3
    pub fn position_vec3(&self) -> Vec3 {
        Vec3::new(self.position[0], self.position[1], self.position[2])
    }

    /// Parse loot type string to enum discriminant (0 = health, 1 = speed)
    pub fn loot_type_id(&self) -> u8 {
        match self.loot_type.to_lowercase().as_str() {
            "health_pack" | "health" => 0,
            "speed_boost" | "speed" => 1,
            _ => 0, // Default to health pack
        }
    }

    /// Validate loot spawn position is within arena bounds
    pub fn validate(&self, arena_size: [u32; 3]) -> Result<(), String> {
        let [x, y, z] = self.position;
        if x < 0.0 || y < 0.0 || z < 0.0 {
            return Err(format!(
                "position ({}, {}, {}) has negative coordinates",
                x, y, z
            ));
        }
        if x > arena_size[0] as f32 || y > arena_size[1] as f32 || z > arena_size[2] as f32 {
            return Err(format!(
                "position ({}, {}, {}) is outside arena bounds (0-{}, 0-{}, 0-{})",
                x, y, z, arena_size[0], arena_size[1], arena_size[2]
            ));
        }
        Ok(())
    }
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

/// Training mode arena configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TrainingArenaConfig {
    /// Number of bots (overrides server default)
    #[serde(default)]
    pub bot_count: Option<u8>,

    /// Bot behavior type: "dummy", "roam", or "strafe"
    #[serde(default)]
    pub bot_behavior: Option<String>,

    /// Bot respawn delay in seconds
    #[serde(default)]
    pub bot_respawn_delay_secs: Option<f32>,

    /// Player respawn delay in seconds
    #[serde(default)]
    pub player_respawn_delay_secs: Option<f32>,

    /// Whether player is invincible
    #[serde(default)]
    pub invincibility_player: Option<bool>,

    /// Whether bots are invincible (hits still count for stats)
    #[serde(default)]
    pub invincibility_bots: Option<bool>,

    /// Dedicated bot spawn points (if different from player spawns)
    #[serde(default)]
    pub bot_spawns: Vec<BotSpawnPoint>,
}

/// Bot spawn point definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotSpawnPoint {
    /// Spawn position (x, y, z)
    pub position: [f32; 3],
}

impl BotSpawnPoint {
    /// Get position as Vec3
    pub fn position_vec3(&self) -> Vec3 {
        Vec3::new(self.position[0], self.position[1], self.position[2])
    }
}

impl TrainingArenaConfig {
    /// Validate the training configuration
    pub fn validate(&self, arena_size: [u32; 3]) -> Result<(), String> {
        // Validate bot count
        if let Some(count) = self.bot_count {
            if count > 20 {
                return Err("bot_count must be 0-20".to_string());
            }
        }

        // Validate bot spawn positions
        for (i, spawn) in self.bot_spawns.iter().enumerate() {
            let [x, y, z] = spawn.position;
            if x < 0.0 || y < 0.0 || z < 0.0 {
                return Err(format!(
                    "bot_spawns[{}]: position ({}, {}, {}) has negative coordinates",
                    i, x, y, z
                ));
            }
            if x > arena_size[0] as f32 || y > arena_size[1] as f32 || z > arena_size[2] as f32 {
                return Err(format!(
                    "bot_spawns[{}]: position ({}, {}, {}) is outside arena bounds",
                    i, x, y, z
                ));
            }
        }

        // Validate bot behavior string
        if let Some(ref behavior) = self.bot_behavior {
            match behavior.to_lowercase().as_str() {
                "dummy" | "roam" | "strafe" => {}
                _ => {
                    return Err(format!(
                        "bot_behavior '{}' is invalid (expected: dummy, roam, or strafe)",
                        behavior
                    ));
                }
            }
        }

        Ok(())
    }

    /// Parse bot behavior string to enum discriminant
    pub fn bot_behavior_type(&self) -> Option<&str> {
        self.bot_behavior.as_deref()
    }
}

/// Economy arena configuration (T052-T053)
/// Allows arena authors to customize economy parameters for this arena.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EconomyArenaConfig {
    /// Whether economy is enabled for this arena (default: true for most modes)
    #[serde(default)]
    pub enabled: Option<bool>,

    /// Reward for getting a kill (default: 10)
    #[serde(default)]
    pub kill_reward: Option<u32>,

    /// Reward for capturing the flag in CTF mode (default: 25)
    #[serde(default)]
    pub ctf_capture_reward: Option<u32>,

    /// Rewards for 1st, 2nd, 3rd place in BR Lite (defaults: [50, 30, 15])
    #[serde(default)]
    pub br_placement_rewards: Option<[u32; 3]>,

    /// Custom shop offers for this arena
    #[serde(default)]
    pub shop_offers: Vec<ShopOfferConfig>,
}

impl EconomyArenaConfig {
    /// Validate the economy configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate shop offers
        for (i, offer) in self.shop_offers.iter().enumerate() {
            offer
                .validate()
                .map_err(|e| format!("shop_offers[{}]: {}", i, e))?;
        }
        Ok(())
    }
}

/// Shop offer configuration for arena TOML (T053)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopOfferConfig {
    /// Unique offer identifier
    pub offer_id: String,
    /// Item ID (numeric)
    pub item_id: u16,
    /// Quantity to receive
    #[serde(default = "default_quantity")]
    pub quantity: u8,
    /// Price in coins
    pub price: u32,
    /// Maximum purchases per match (optional)
    #[serde(default)]
    pub max_per_match: Option<u8>,
}

fn default_quantity() -> u8 {
    1
}

impl ShopOfferConfig {
    /// Validate the shop offer configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.offer_id.is_empty() {
            return Err("offer_id cannot be empty".to_string());
        }
        if self.price == 0 {
            return Err("price must be greater than 0".to_string());
        }
        if self.quantity == 0 {
            return Err("quantity must be greater than 0".to_string());
        }
        Ok(())
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

    /// T026 [US1]: Convert this arena to a ChunkedWorld for chunk-based rendering.
    ///
    /// This transfers all blocks from the flat arena storage into the chunked
    /// world container, which enables per-chunk meshing and streaming.
    pub fn to_chunked_world(&self) -> ChunkedWorld {
        let [size_x, size_y, size_z] = self.size();
        ChunkedWorld::from_flat_blocks(&self.blocks, size_x, size_y, size_z)
    }

    /// Get flag zones for CTF mode
    pub fn flag_zones(&self) -> Vec<FlagZone> {
        self.definition
            .flag_zones
            .iter()
            .map(|z| z.to_flag_zone())
            .collect()
    }

    /// Get game mode for this arena
    pub fn game_mode(&self) -> GameMode {
        self.definition.metadata.game_mode
    }

    /// Get BR Lite configuration for this arena (if available)
    pub fn br_lite_config(&self) -> Option<&BrLiteArenaConfig> {
        self.definition.br_lite.as_ref()
    }

    /// Get loot spawn points for BR Lite mode
    pub fn br_lite_loot_spawns(&self) -> Vec<&LootSpawnConfig> {
        self.definition
            .br_lite
            .as_ref()
            .map(|br| br.loot_spawns.iter().collect())
            .unwrap_or_default()
    }

    /// Get Training mode configuration for this arena (if available)
    pub fn training_config(&self) -> Option<&TrainingArenaConfig> {
        self.definition.training.as_ref()
    }

    /// Get bot spawn points for Training mode
    pub fn training_bot_spawns(&self) -> Vec<Vec3> {
        self.definition
            .training
            .as_ref()
            .map(|t| t.bot_spawns.iter().map(|s| s.position_vec3()).collect())
            .unwrap_or_default()
    }

    /// Get Economy configuration for this arena (if available)
    pub fn economy_config(&self) -> Option<&EconomyArenaConfig> {
        self.definition.economy.as_ref()
    }
}

// Implement BlockWorld trait for physics system compatibility
impl BlockWorld for LoadedArena {
    fn get_block(&self, pos: BlockPos) -> BlockType {
        self.get_block_at(pos)
    }

    fn set_block(&mut self, pos: BlockPos, block: BlockType) {
        LoadedArena::set_block(self, pos, block);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_br_lite_config_defaults() {
        let config = BrLiteArenaConfig::default();
        assert_eq!(config.min_players, 4);
        assert_eq!(config.post_match_delay_secs, 10);
        assert_eq!(config.bonus_duration_secs, 10);
        assert!(config.loot_enabled);
        assert!(config.zone_center.is_none());
        assert!(config.initial_radius.is_none());
        assert!(config.loot_spawns.is_empty());
    }

    #[test]
    fn test_br_lite_config_validation_valid() {
        let config = BrLiteArenaConfig {
            zone_center: Some([32.0, 32.0]),
            initial_radius: Some(25.0),
            min_players: 4,
            loot_spawns: vec![LootSpawnConfig {
                position: [10.0, 2.0, 10.0],
                loot_type: "health_pack".to_string(),
                param: 25,
            }],
            ..Default::default()
        };
        let arena_size = [64, 16, 64];
        assert!(config.validate(arena_size).is_ok());
    }

    #[test]
    fn test_br_lite_config_validation_zone_out_of_bounds() {
        let config = BrLiteArenaConfig {
            zone_center: Some([100.0, 32.0]), // Outside 64x64 arena
            ..Default::default()
        };
        let arena_size = [64, 16, 64];
        let result = config.validate(arena_size);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("zone_center"));
    }

    #[test]
    fn test_br_lite_config_validation_negative_radius() {
        let config = BrLiteArenaConfig {
            initial_radius: Some(-10.0),
            ..Default::default()
        };
        let arena_size = [64, 16, 64];
        let result = config.validate(arena_size);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("initial_radius"));
    }

    #[test]
    fn test_br_lite_config_validation_min_players() {
        let mut config = BrLiteArenaConfig::default();
        config.min_players = 1;
        assert!(config.validate([64, 16, 64]).is_err());

        config.min_players = 17;
        assert!(config.validate([64, 16, 64]).is_err());

        config.min_players = 4;
        assert!(config.validate([64, 16, 64]).is_ok());
    }

    #[test]
    fn test_loot_spawn_config_position() {
        let loot = LootSpawnConfig {
            position: [10.0, 2.5, 15.0],
            loot_type: "health_pack".to_string(),
            param: 30,
        };
        let pos = loot.position_vec3();
        assert_eq!(pos.x, 10.0);
        assert_eq!(pos.y, 2.5);
        assert_eq!(pos.z, 15.0);
    }

    #[test]
    fn test_loot_spawn_config_type_parsing() {
        let health = LootSpawnConfig {
            position: [0.0, 0.0, 0.0],
            loot_type: "health_pack".to_string(),
            param: 25,
        };
        assert_eq!(health.loot_type_id(), 0);

        let health_short = LootSpawnConfig {
            position: [0.0, 0.0, 0.0],
            loot_type: "health".to_string(),
            param: 25,
        };
        assert_eq!(health_short.loot_type_id(), 0);

        let speed = LootSpawnConfig {
            position: [0.0, 0.0, 0.0],
            loot_type: "speed_boost".to_string(),
            param: 150,
        };
        assert_eq!(speed.loot_type_id(), 1);

        let speed_short = LootSpawnConfig {
            position: [0.0, 0.0, 0.0],
            loot_type: "speed".to_string(),
            param: 150,
        };
        assert_eq!(speed_short.loot_type_id(), 1);
    }

    #[test]
    fn test_loot_spawn_validation_valid() {
        let loot = LootSpawnConfig {
            position: [10.0, 2.0, 10.0],
            loot_type: "health_pack".to_string(),
            param: 25,
        };
        assert!(loot.validate([64, 16, 64]).is_ok());
    }

    #[test]
    fn test_loot_spawn_validation_out_of_bounds() {
        let loot = LootSpawnConfig {
            position: [100.0, 2.0, 10.0], // Outside arena
            loot_type: "health_pack".to_string(),
            param: 25,
        };
        assert!(loot.validate([64, 16, 64]).is_err());
    }

    #[test]
    fn test_loot_spawn_validation_negative() {
        let loot = LootSpawnConfig {
            position: [-5.0, 2.0, 10.0], // Negative position
            loot_type: "health_pack".to_string(),
            param: 25,
        };
        assert!(loot.validate([64, 16, 64]).is_err());
    }

    #[test]
    fn test_br_lite_config_toml_parsing() {
        let toml_str = r#"
            zone_center = [32.0, 32.0]
            initial_radius = 25.0
            min_players = 6
            post_match_delay_secs = 15
            bonus_duration_secs = 8
            loot_enabled = true

            [[loot_spawns]]
            position = [10.0, 2.0, 10.0]
            loot_type = "health_pack"
            param = 30

            [[loot_spawns]]
            position = [50.0, 2.0, 50.0]
            loot_type = "speed_boost"
            param = 150
        "#;

        let config: BrLiteArenaConfig = toml::from_str(toml_str).expect("Failed to parse TOML");
        assert_eq!(config.zone_center, Some([32.0, 32.0]));
        assert_eq!(config.initial_radius, Some(25.0));
        assert_eq!(config.min_players, 6);
        assert_eq!(config.post_match_delay_secs, 15);
        assert_eq!(config.bonus_duration_secs, 8);
        assert!(config.loot_enabled);
        assert_eq!(config.loot_spawns.len(), 2);
        assert_eq!(config.loot_spawns[0].loot_type, "health_pack");
        assert_eq!(config.loot_spawns[1].loot_type, "speed_boost");
    }
}
