//! Arena loading from TOML files

use std::fs;
use std::path::Path;

use thiserror::Error;
use tracing::info;

use plix_common::types::BlockType;

use crate::format::{Arena, BlockDefinitions, LoadedArena};

/// Arena loading errors
#[derive(Debug, Error)]
pub enum LoadError {
    #[error("Failed to read arena file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to parse arena TOML: {0}")]
    ParseError(#[from] toml::de::Error),

    #[error("Arena file not found: {0}")]
    NotFound(String),

    #[error("Invalid arena: {0}")]
    Invalid(String),
}

/// Load an arena from a TOML file
pub fn load_arena<P: AsRef<Path>>(path: P) -> Result<LoadedArena, LoadError> {
    let path = path.as_ref();

    if !path.exists() {
        return Err(LoadError::NotFound(path.display().to_string()));
    }

    let content = fs::read_to_string(path)?;
    let definition: Arena = toml::from_str(&content)?;

    info!(
        name = %definition.metadata.name,
        version = %definition.metadata.version,
        size = ?definition.metadata.size,
        spawns = definition.spawn_points.len(),
        "Loading arena"
    );

    // Generate block data from definition
    let blocks = generate_blocks(&definition)?;

    Ok(LoadedArena { definition, blocks })
}

/// Load arena by name from assets directory
pub fn load_arena_by_name(name: &str, assets_dir: &Path) -> Result<LoadedArena, LoadError> {
    let path = assets_dir.join("arenas").join(format!("{}.toml", name));
    load_arena(path)
}

/// Generate block data from arena definition
fn generate_blocks(arena: &Arena) -> Result<Vec<BlockType>, LoadError> {
    let [sx, sy, sz] = arena.metadata.size;
    let total_blocks = (sx * sy * sz) as usize;

    // Initialize all blocks to air
    let mut blocks = vec![BlockType::AIR; total_blocks];

    let block_defs = &arena.blocks;

    // Apply floor
    if let Some(floor) = &block_defs.floor {
        let block_type = BlockDefinitions::parse_block_type(&floor.block);
        let y = floor.y.max(0) as u32;

        if y < sy {
            for z in 0..sz {
                for x in 0..sx {
                    let index = (z * sy * sx + y * sx + x) as usize;
                    blocks[index] = block_type;
                }
            }
        }
    }

    // Apply border walls
    if let Some(walls) = &block_defs.walls {
        if walls.border {
            let block_type = BlockDefinitions::parse_block_type(&walls.block);
            let height = walls.height.min(sy);

            for y in 0..height {
                // X walls (z = 0 and z = sz-1)
                for x in 0..sx {
                    // z = 0 wall
                    let idx1 = (y * sx + x) as usize;
                    // z = sz-1 wall
                    let idx2 = ((sz - 1) * sy * sx + y * sx + x) as usize;
                    blocks[idx1] = block_type;
                    blocks[idx2] = block_type;
                }

                // Z walls (x = 0 and x = sx-1)
                for z in 0..sz {
                    // x = 0 wall
                    let idx1 = (z * sy * sx + y * sx) as usize;
                    // x = sx-1 wall
                    let idx2 = (z * sy * sx + y * sx + sx - 1) as usize;
                    blocks[idx1] = block_type;
                    blocks[idx2] = block_type;
                }
            }
        }
    }

    // Apply custom regions
    for region in &block_defs.regions {
        let block_type = BlockDefinitions::parse_block_type(&region.block);

        let min_x = region.min[0].max(0) as u32;
        let min_y = region.min[1].max(0) as u32;
        let min_z = region.min[2].max(0) as u32;

        let max_x = (region.max[0] as u32).min(sx);
        let max_y = (region.max[1] as u32).min(sy);
        let max_z = (region.max[2] as u32).min(sz);

        for z in min_z..max_z {
            for y in min_y..max_y {
                for x in min_x..max_x {
                    let index = (z * sy * sx + y * sx + x) as usize;
                    blocks[index] = block_type;
                }
            }
        }
    }

    Ok(blocks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_arena() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        write!(
            file,
            r#"
[metadata]
name = "Test Arena"
version = "0.1.0"
size = [16, 8, 16]

[[spawn_points]]
team = 0
position = [4.0, 1.0, 4.0]
rotation = 0.0

[[spawn_points]]
team = 1
position = [12.0, 1.0, 12.0]
rotation = 180.0

[blocks]
floor = {{ y = 0, block = "stone" }}
walls = {{ border = true, height = 4, block = "brick" }}
"#
        )
        .unwrap();
        file
    }

    #[test]
    fn test_load_arena() {
        let file = create_test_arena();
        let arena = load_arena(file.path()).unwrap();

        assert_eq!(arena.definition.metadata.name, "Test Arena");
        assert_eq!(arena.size(), [16, 8, 16]);
        assert_eq!(arena.definition.spawn_points.len(), 2);
    }

    #[test]
    fn test_floor_generation() {
        let file = create_test_arena();
        let arena = load_arena(file.path()).unwrap();

        // Check floor is stone
        assert_eq!(arena.get_block(5, 0, 5), BlockType::STONE);
        // Check above floor is air
        assert_eq!(arena.get_block(5, 1, 5), BlockType::AIR);
    }

    #[test]
    fn test_wall_generation() {
        let file = create_test_arena();
        let arena = load_arena(file.path()).unwrap();

        // Check border walls are brick
        assert_eq!(arena.get_block(0, 0, 0), BlockType::BRICK);
        assert_eq!(arena.get_block(0, 1, 5), BlockType::BRICK);
        assert_eq!(arena.get_block(15, 1, 5), BlockType::BRICK);
    }
}
