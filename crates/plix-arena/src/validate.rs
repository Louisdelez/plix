//! Arena validation

use thiserror::Error;

use crate::format::{Arena, LoadedArena};

/// Validation errors
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Arena too small: minimum 8x4x8, got {0:?}")]
    TooSmall([u32; 3]),

    #[error("Arena too large: maximum 256x64x256, got {0:?}")]
    TooLarge([u32; 3]),

    #[error("No spawn points defined")]
    NoSpawnPoints,

    #[error("No spawn points for team {0}")]
    NoTeamSpawns(u8),

    #[error("Spawn point out of bounds: {0:?}")]
    SpawnOutOfBounds([f32; 3]),

    #[error("Spawn point inside solid block: {0:?}")]
    SpawnInSolid([f32; 3]),

    #[error("Arena name is empty")]
    EmptyName,
}

/// Validate an arena definition
pub fn validate_arena(arena: &Arena) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    // Check name
    if arena.metadata.name.trim().is_empty() {
        errors.push(ValidationError::EmptyName);
    }

    // Check size bounds
    let [sx, sy, sz] = arena.metadata.size;
    if sx < 8 || sy < 4 || sz < 8 {
        errors.push(ValidationError::TooSmall(arena.metadata.size));
    }
    if sx > 256 || sy > 64 || sz > 256 {
        errors.push(ValidationError::TooLarge(arena.metadata.size));
    }

    // Check spawn points
    if arena.spawn_points.is_empty() {
        errors.push(ValidationError::NoSpawnPoints);
    } else {
        // Check each team has at least one spawn
        let teams: Vec<u8> = arena.spawn_points.iter().map(|sp| sp.team).collect();
        let unique_teams: std::collections::HashSet<_> = teams.iter().collect();

        // For a 2-team game, check teams 0 and 1
        for team in 0..=1 {
            if !unique_teams.contains(&team) {
                errors.push(ValidationError::NoTeamSpawns(team));
            }
        }

        // Check spawn positions are in bounds
        for spawn in &arena.spawn_points {
            let [x, y, z] = spawn.position;
            if x < 0.0 || y < 0.0 || z < 0.0 || x >= sx as f32 || y >= sy as f32 || z >= sz as f32 {
                errors.push(ValidationError::SpawnOutOfBounds(spawn.position));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Validate a loaded arena (with block data)
pub fn validate_loaded_arena(arena: &LoadedArena) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    // Run basic validation first
    if let Err(basic_errors) = validate_arena(&arena.definition) {
        errors.extend(basic_errors);
    }

    // Check spawns aren't inside solid blocks
    for spawn in &arena.definition.spawn_points {
        let [x, y, z] = spawn.position;
        let block = arena.get_block(x as u32, y as u32, z as u32);

        if block.is_solid() {
            errors.push(ValidationError::SpawnInSolid(spawn.position));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::{ArenaMetadata, BlockDefinitions, SpawnPoint};

    fn make_valid_arena() -> Arena {
        Arena {
            metadata: ArenaMetadata {
                name: "Test".to_string(),
                version: "1.0".to_string(),
                size: [32, 16, 32],
            },
            spawn_points: vec![
                SpawnPoint {
                    team: 0,
                    position: [5.0, 1.0, 5.0],
                    rotation: 0.0,
                },
                SpawnPoint {
                    team: 1,
                    position: [27.0, 1.0, 27.0],
                    rotation: 180.0,
                },
            ],
            blocks: BlockDefinitions {
                floor: None,
                walls: None,
                regions: vec![],
            },
        }
    }

    #[test]
    fn test_valid_arena() {
        let arena = make_valid_arena();
        assert!(validate_arena(&arena).is_ok());
    }

    #[test]
    fn test_too_small() {
        let mut arena = make_valid_arena();
        arena.metadata.size = [4, 4, 4];
        let result = validate_arena(&arena);
        assert!(result.is_err());
    }

    #[test]
    fn test_no_spawns() {
        let mut arena = make_valid_arena();
        arena.spawn_points.clear();
        let result = validate_arena(&arena);
        assert!(result.is_err());
    }

    #[test]
    fn test_spawn_out_of_bounds() {
        let mut arena = make_valid_arena();
        arena.spawn_points[0].position = [100.0, 0.0, 0.0];
        let result = validate_arena(&arena);
        assert!(result.is_err());
    }
}
