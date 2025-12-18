//! Arena validation

use thiserror::Error;

use plix_common::types::{FlagZoneType, GameMode, TeamId};

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

    // CTF-specific validation errors
    #[error("CTF mode requires flag zones but none defined")]
    CtfNoFlagZones,

    #[error("CTF mode missing flag base for team {0}")]
    CtfMissingFlagBase(u8),

    #[error("CTF mode missing capture zone for team {0}")]
    CtfMissingCaptureZone(u8),

    #[error("Duplicate flag base for team {0}")]
    CtfDuplicateFlagBase(u8),

    #[error("Duplicate capture zone for team {0}")]
    CtfDuplicateCaptureZone(u8),

    #[error("Flag zone out of bounds: team {0}, min {1:?}, max {2:?}")]
    CtfFlagZoneOutOfBounds(u8, [f32; 3], [f32; 3]),
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

    // CTF-specific validation
    if arena.metadata.game_mode == GameMode::Ctf {
        validate_ctf_zones(arena, &mut errors);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Validate CTF flag zones
fn validate_ctf_zones(arena: &Arena, errors: &mut Vec<ValidationError>) {
    let [sx, sy, sz] = arena.metadata.size;

    // CTF mode requires flag zones
    if arena.flag_zones.is_empty() {
        errors.push(ValidationError::CtfNoFlagZones);
        return;
    }

    // Convert definitions to flag zones for analysis
    let zones: Vec<_> = arena.flag_zones.iter().map(|z| z.to_flag_zone()).collect();

    // Track which zones we've seen
    let mut team0_flag_base = 0;
    let mut team1_flag_base = 0;
    let mut team0_capture = 0;
    let mut team1_capture = 0;

    for (i, zone_def) in arena.flag_zones.iter().enumerate() {
        let zone = &zones[i];

        // Check zone is in bounds
        if zone_def.min[0] < 0.0
            || zone_def.min[1] < 0.0
            || zone_def.min[2] < 0.0
            || zone_def.max[0] > sx as f32
            || zone_def.max[1] > sy as f32
            || zone_def.max[2] > sz as f32
        {
            errors.push(ValidationError::CtfFlagZoneOutOfBounds(
                zone_def.team,
                zone_def.min,
                zone_def.max,
            ));
        }

        // Count zone types per team
        match (zone.team, zone.zone_type) {
            (t, FlagZoneType::FlagBase) if t == TeamId::TEAM_0 => team0_flag_base += 1,
            (t, FlagZoneType::FlagBase) if t == TeamId::TEAM_1 => team1_flag_base += 1,
            (t, FlagZoneType::CaptureZone) if t == TeamId::TEAM_0 => team0_capture += 1,
            (t, FlagZoneType::CaptureZone) if t == TeamId::TEAM_1 => team1_capture += 1,
            _ => {}
        }
    }

    // Check for missing zones
    if team0_flag_base == 0 {
        errors.push(ValidationError::CtfMissingFlagBase(0));
    }
    if team1_flag_base == 0 {
        errors.push(ValidationError::CtfMissingFlagBase(1));
    }
    if team0_capture == 0 {
        errors.push(ValidationError::CtfMissingCaptureZone(0));
    }
    if team1_capture == 0 {
        errors.push(ValidationError::CtfMissingCaptureZone(1));
    }

    // Check for duplicate zones
    if team0_flag_base > 1 {
        errors.push(ValidationError::CtfDuplicateFlagBase(0));
    }
    if team1_flag_base > 1 {
        errors.push(ValidationError::CtfDuplicateFlagBase(1));
    }
    if team0_capture > 1 {
        errors.push(ValidationError::CtfDuplicateCaptureZone(0));
    }
    if team1_capture > 1 {
        errors.push(ValidationError::CtfDuplicateCaptureZone(1));
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
    use crate::format::{ArenaMetadata, BlockDefinitions, FlagZoneDefinition, SpawnPoint};

    fn make_valid_arena() -> Arena {
        Arena {
            metadata: ArenaMetadata {
                name: "Test".to_string(),
                version: "1.0".to_string(),
                size: [32, 16, 32],
                game_mode: plix_common::types::GameMode::Tdm,
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
            flag_zones: vec![],
            br_lite: None,
            training: None,
            economy: None,
        }
    }

    fn make_valid_ctf_arena() -> Arena {
        Arena {
            metadata: ArenaMetadata {
                name: "CTF Test".to_string(),
                version: "1.0".to_string(),
                size: [64, 16, 64],
                game_mode: GameMode::Ctf,
            },
            spawn_points: vec![
                SpawnPoint {
                    team: 0,
                    position: [8.0, 1.0, 32.0],
                    rotation: 0.0,
                },
                SpawnPoint {
                    team: 1,
                    position: [56.0, 1.0, 32.0],
                    rotation: 180.0,
                },
            ],
            blocks: BlockDefinitions {
                floor: None,
                walls: None,
                regions: vec![],
            },
            flag_zones: vec![
                // Team 0 flag base (where team 0's flag spawns)
                FlagZoneDefinition {
                    team: 0,
                    zone_type: "flag_base".to_string(),
                    min: [2.0, 0.0, 28.0],
                    max: [6.0, 4.0, 36.0],
                },
                // Team 0 capture zone (where team 0 brings enemy flag)
                FlagZoneDefinition {
                    team: 0,
                    zone_type: "capture_zone".to_string(),
                    min: [0.0, 0.0, 26.0],
                    max: [8.0, 4.0, 38.0],
                },
                // Team 1 flag base
                FlagZoneDefinition {
                    team: 1,
                    zone_type: "flag_base".to_string(),
                    min: [58.0, 0.0, 28.0],
                    max: [62.0, 4.0, 36.0],
                },
                // Team 1 capture zone
                FlagZoneDefinition {
                    team: 1,
                    zone_type: "capture_zone".to_string(),
                    min: [56.0, 0.0, 26.0],
                    max: [64.0, 4.0, 38.0],
                },
            ],
            br_lite: None,
            training: None,
            economy: None,
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

    #[test]
    fn test_valid_ctf_arena() {
        let arena = make_valid_ctf_arena();
        assert!(validate_arena(&arena).is_ok());
    }

    #[test]
    fn test_ctf_no_flag_zones() {
        let mut arena = make_valid_ctf_arena();
        arena.flag_zones.clear();
        let result = validate_arena(&arena);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::CtfNoFlagZones)));
    }

    #[test]
    fn test_ctf_missing_flag_base() {
        let mut arena = make_valid_ctf_arena();
        // Remove team 0's flag base
        arena
            .flag_zones
            .retain(|z| !(z.team == 0 && z.zone_type == "flag_base"));
        let result = validate_arena(&arena);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::CtfMissingFlagBase(0))));
    }

    #[test]
    fn test_ctf_missing_capture_zone() {
        let mut arena = make_valid_ctf_arena();
        // Remove team 1's capture zone
        arena
            .flag_zones
            .retain(|z| !(z.team == 1 && z.zone_type == "capture_zone"));
        let result = validate_arena(&arena);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::CtfMissingCaptureZone(1))));
    }

    #[test]
    fn test_ctf_duplicate_flag_base() {
        let mut arena = make_valid_ctf_arena();
        // Add duplicate flag base for team 0
        arena.flag_zones.push(FlagZoneDefinition {
            team: 0,
            zone_type: "flag_base".to_string(),
            min: [10.0, 0.0, 28.0],
            max: [14.0, 4.0, 36.0],
        });
        let result = validate_arena(&arena);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::CtfDuplicateFlagBase(0))));
    }

    #[test]
    fn test_ctf_zone_out_of_bounds() {
        let mut arena = make_valid_ctf_arena();
        // Make one zone extend beyond arena bounds
        arena.flag_zones[0].max = [100.0, 4.0, 36.0];
        let result = validate_arena(&arena);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::CtfFlagZoneOutOfBounds(..))))
    }
}
