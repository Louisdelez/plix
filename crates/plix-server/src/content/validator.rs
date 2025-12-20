//! Content validation for reference checking and constraint validation

use std::collections::HashSet;

use thiserror::Error;
use tracing::{error, warn};

use plix_common::content::{
    ChapterId, DungeonId, LootTableId, MobDefId, NpcId, QuestId, QuestStep, RegionId,
};

use super::loader::ContentRegistry;

/// Validation mode determines how errors are handled
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationMode {
    /// Development mode: fail fast on first error
    Development,
    /// Production mode: log warnings and continue with valid content
    Production,
}

/// Errors that can occur during content validation
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Invalid reference: {entity_type} '{entity_id}' references unknown {ref_type} '{ref_id}'")]
    InvalidReference {
        entity_type: String,
        entity_id: String,
        ref_type: String,
        ref_id: String,
    },

    #[error("Constraint violation in {entity_type} '{entity_id}': {message}")]
    ConstraintViolation {
        entity_type: String,
        entity_id: String,
        message: String,
    },

    #[error("Duplicate ID: {entity_type} '{id}' defined multiple times")]
    DuplicateId { entity_type: String, id: String },

    #[error("Missing required field in {entity_type} '{entity_id}': {field}")]
    MissingField {
        entity_type: String,
        entity_id: String,
        field: String,
    },
}

/// Result of validation
#[derive(Debug)]
pub struct ValidationResult {
    /// Errors found during validation
    pub errors: Vec<ValidationError>,
    /// Warnings found during validation
    pub warnings: Vec<String>,
    /// Whether validation passed (no errors)
    pub passed: bool,
}

impl ValidationResult {
    fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            passed: true,
        }
    }

    fn add_error(&mut self, error: ValidationError) {
        self.passed = false;
        self.errors.push(error);
    }

    fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
}

/// Validates content registry for reference integrity and constraints
pub struct ContentValidator {
    mode: ValidationMode,
}

impl ContentValidator {
    /// Create a new validator with the given mode
    pub fn new(mode: ValidationMode) -> Self {
        Self { mode }
    }

    /// Validate the entire content registry
    pub fn validate(&self, registry: &ContentRegistry) -> Result<ValidationResult, ValidationError> {
        let mut result = ValidationResult::new();

        // Collect all known IDs for reference checking
        let known_quest_ids: HashSet<_> = registry.quests.keys().collect();
        let known_mob_ids: HashSet<_> = registry.mobs.keys().collect();
        let known_loot_table_ids: HashSet<_> = registry.loot_tables.keys().collect();
        let known_dungeon_ids: HashSet<_> = registry.dungeons.keys().collect();
        let known_npc_ids: HashSet<_> = registry.npcs.keys().collect();
        let known_chapter_ids: HashSet<_> = registry.chapters.keys().collect();

        // Validate chapters
        for (id, chapter) in &registry.chapters {
            // Check mainline quest references
            for quest_id in &chapter.mainline_quests {
                if !known_quest_ids.contains(quest_id) {
                    self.handle_error(
                        &mut result,
                        ValidationError::InvalidReference {
                            entity_type: "chapter".to_string(),
                            entity_id: id.to_string(),
                            ref_type: "quest".to_string(),
                            ref_id: quest_id.to_string(),
                        },
                    )?;
                }
            }

            // Check side quest references
            for quest_id in &chapter.side_quests {
                if !known_quest_ids.contains(quest_id) {
                    self.handle_error(
                        &mut result,
                        ValidationError::InvalidReference {
                            entity_type: "chapter".to_string(),
                            entity_id: id.to_string(),
                            ref_type: "quest".to_string(),
                            ref_id: quest_id.to_string(),
                        },
                    )?;
                }
            }
        }

        // Validate quests
        for (id, quest) in &registry.quests {
            // Check chapter reference
            if let Some(ref chapter_id) = quest.chapter_id {
                if !known_chapter_ids.contains(chapter_id) {
                    self.handle_error(
                        &mut result,
                        ValidationError::InvalidReference {
                            entity_type: "quest".to_string(),
                            entity_id: id.to_string(),
                            ref_type: "chapter".to_string(),
                            ref_id: chapter_id.to_string(),
                        },
                    )?;
                }
            }

            // Check prerequisite quest references
            for prereq_id in &quest.prerequisites.completed_quests {
                if !known_quest_ids.contains(prereq_id) {
                    self.handle_error(
                        &mut result,
                        ValidationError::InvalidReference {
                            entity_type: "quest".to_string(),
                            entity_id: id.to_string(),
                            ref_type: "prerequisite quest".to_string(),
                            ref_id: prereq_id.to_string(),
                        },
                    )?;
                }
            }

            // Check step references
            for step in &quest.steps {
                match step {
                    QuestStep::KillMob { mob_id, count, .. } => {
                        if !known_mob_ids.contains(mob_id) {
                            self.handle_error(
                                &mut result,
                                ValidationError::InvalidReference {
                                    entity_type: "quest".to_string(),
                                    entity_id: id.to_string(),
                                    ref_type: "mob".to_string(),
                                    ref_id: mob_id.to_string(),
                                },
                            )?;
                        }
                        if *count == 0 {
                            self.handle_error(
                                &mut result,
                                ValidationError::ConstraintViolation {
                                    entity_type: "quest".to_string(),
                                    entity_id: id.to_string(),
                                    message: "KillMob count must be > 0".to_string(),
                                },
                            )?;
                        }
                    }
                    QuestStep::TalkToNpc { npc_id, .. } => {
                        if !known_npc_ids.contains(npc_id) {
                            self.handle_error(
                                &mut result,
                                ValidationError::InvalidReference {
                                    entity_type: "quest".to_string(),
                                    entity_id: id.to_string(),
                                    ref_type: "npc".to_string(),
                                    ref_id: npc_id.to_string(),
                                },
                            )?;
                        }
                    }
                    QuestStep::DungeonClear { dungeon_id, .. } => {
                        if !known_dungeon_ids.contains(dungeon_id) {
                            self.handle_error(
                                &mut result,
                                ValidationError::InvalidReference {
                                    entity_type: "quest".to_string(),
                                    entity_id: id.to_string(),
                                    ref_type: "dungeon".to_string(),
                                    ref_id: dungeon_id.to_string(),
                                },
                            )?;
                        }
                    }
                    QuestStep::CollectItem { count, .. } => {
                        if *count == 0 {
                            self.handle_error(
                                &mut result,
                                ValidationError::ConstraintViolation {
                                    entity_type: "quest".to_string(),
                                    entity_id: id.to_string(),
                                    message: "CollectItem count must be > 0".to_string(),
                                },
                            )?;
                        }
                    }
                    QuestStep::VisitLocation { .. } => {
                        // Region validation could be added here
                    }
                }
            }
        }

        // Validate mobs
        for (id, mob) in &registry.mobs {
            // Check loot table reference
            if !known_loot_table_ids.contains(&mob.loot_table) {
                self.handle_error(
                    &mut result,
                    ValidationError::InvalidReference {
                        entity_type: "mob".to_string(),
                        entity_id: id.to_string(),
                        ref_type: "loot_table".to_string(),
                        ref_id: mob.loot_table.to_string(),
                    },
                )?;
            }

            // Validate stats
            if let Err(msg) = mob.stats.validate() {
                self.handle_error(
                    &mut result,
                    ValidationError::ConstraintViolation {
                        entity_type: "mob".to_string(),
                        entity_id: id.to_string(),
                        message: msg.to_string(),
                    },
                )?;
            }
        }

        // Validate loot tables
        for (id, loot_table) in &registry.loot_tables {
            for entry in loot_table.guaranteed.iter().chain(loot_table.entries.iter()) {
                if let Err(msg) = entry.validate() {
                    self.handle_error(
                        &mut result,
                        ValidationError::ConstraintViolation {
                            entity_type: "loot_table".to_string(),
                            entity_id: id.to_string(),
                            message: msg.to_string(),
                        },
                    )?;
                }
            }
        }

        // Validate spawns
        for (id, spawn) in &registry.spawns {
            // Check mob reference
            if !known_mob_ids.contains(&spawn.mob_id) {
                self.handle_error(
                    &mut result,
                    ValidationError::InvalidReference {
                        entity_type: "spawn".to_string(),
                        entity_id: id.to_string(),
                        ref_type: "mob".to_string(),
                        ref_id: spawn.mob_id.to_string(),
                    },
                )?;
            }

            // Validate constraints
            if let Err(msg) = spawn.validate() {
                self.handle_error(
                    &mut result,
                    ValidationError::ConstraintViolation {
                        entity_type: "spawn".to_string(),
                        entity_id: id.to_string(),
                        message: msg.to_string(),
                    },
                )?;
            }
        }

        // Validate dungeons
        for (id, dungeon) in &registry.dungeons {
            // Check boss mob reference
            if !known_mob_ids.contains(&dungeon.boss_mob_id) {
                self.handle_error(
                    &mut result,
                    ValidationError::InvalidReference {
                        entity_type: "dungeon".to_string(),
                        entity_id: id.to_string(),
                        ref_type: "boss mob".to_string(),
                        ref_id: dungeon.boss_mob_id.to_string(),
                    },
                )?;
            }

            // Check reward loot table reference
            if !known_loot_table_ids.contains(&dungeon.reward_loot_table) {
                self.handle_error(
                    &mut result,
                    ValidationError::InvalidReference {
                        entity_type: "dungeon".to_string(),
                        entity_id: id.to_string(),
                        ref_type: "loot_table".to_string(),
                        ref_id: dungeon.reward_loot_table.to_string(),
                    },
                )?;
            }

            // Validate constraints
            if let Err(msg) = dungeon.validate() {
                self.handle_error(
                    &mut result,
                    ValidationError::ConstraintViolation {
                        entity_type: "dungeon".to_string(),
                        entity_id: id.to_string(),
                        message: msg.to_string(),
                    },
                )?;
            }

            // Check spawn references in rooms
            for room in &dungeon.rooms {
                for spawn in &room.spawns {
                    if !known_mob_ids.contains(&spawn.mob_id) {
                        self.handle_error(
                            &mut result,
                            ValidationError::InvalidReference {
                                entity_type: "dungeon room".to_string(),
                                entity_id: format!("{}:{}", id, room.room_id),
                                ref_type: "mob".to_string(),
                                ref_id: spawn.mob_id.to_string(),
                            },
                        )?;
                    }
                }
            }
        }

        // Validate NPCs
        for (id, npc) in &registry.npcs {
            // Check quest giver references
            for quest_id in &npc.quest_giver {
                if !known_quest_ids.contains(quest_id) {
                    self.handle_error(
                        &mut result,
                        ValidationError::InvalidReference {
                            entity_type: "npc".to_string(),
                            entity_id: id.to_string(),
                            ref_type: "quest".to_string(),
                            ref_id: quest_id.to_string(),
                        },
                    )?;
                }
            }
        }

        // Log summary
        if result.passed {
            tracing::info!("Content validation passed");
        } else {
            tracing::error!(
                "Content validation failed with {} errors",
                result.errors.len()
            );
        }

        for warning in &result.warnings {
            warn!("{}", warning);
        }

        Ok(result)
    }

    /// Handle an error based on validation mode
    fn handle_error(
        &self,
        result: &mut ValidationResult,
        error: ValidationError,
    ) -> Result<(), ValidationError> {
        match self.mode {
            ValidationMode::Development => {
                error!("{}", error);
                Err(error)
            }
            ValidationMode::Production => {
                warn!("Validation warning (skipping): {}", error);
                result.add_error(error);
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plix_common::content::*;

    fn create_valid_registry() -> ContentRegistry {
        let mut registry = ContentRegistry::new();

        // Add a loot table
        registry.loot_tables.insert(
            LootTableId::new("test_drops"),
            LootTable {
                loot_table_id: LootTableId::new("test_drops"),
                guaranteed: vec![],
                entries: vec![],
            },
        );

        // Add a mob
        registry.mobs.insert(
            MobDefId::new("test_mob"),
            MobDefinition {
                mob_id: MobDefId::new("test_mob"),
                display_name: "Test Mob".to_string(),
                stats: MobStats {
                    hp: 50,
                    damage: 5,
                    speed: 3.0,
                    armor: 0,
                    aggro_radius: 8.0,
                    leash_radius: 20.0,
                    attack_range: 2.0,
                    attack_cooldown: 1.0,
                },
                behavior: MobBehavior::Aggro,
                loot_table: LootTableId::new("test_drops"),
                xp_reward: 10,
                credit_reward: 5,
                tags: vec![],
            },
        );

        // Add an NPC
        registry.npcs.insert(
            NpcId::new("test_npc"),
            NpcDefinition {
                npc_id: NpcId::new("test_npc"),
                name: "Test NPC".to_string(),
                position: plix_common::math::Vec3::new(0.0, 0.0, 0.0),
                rotation: 0.0,
                quest_giver: vec![],
                idle_dialogue: vec![],
                quest_offer_dialogue: Default::default(),
                quest_complete_dialogue: Default::default(),
            },
        );

        registry
    }

    #[test]
    fn test_valid_registry() {
        let registry = create_valid_registry();
        let validator = ContentValidator::new(ValidationMode::Development);
        let result = validator.validate(&registry).unwrap();
        assert!(result.passed);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_invalid_mob_reference_dev_mode() {
        let mut registry = create_valid_registry();

        // Add mob with invalid loot table reference
        registry.mobs.insert(
            MobDefId::new("bad_mob"),
            MobDefinition {
                mob_id: MobDefId::new("bad_mob"),
                display_name: "Bad Mob".to_string(),
                stats: MobStats {
                    hp: 50,
                    damage: 5,
                    speed: 3.0,
                    armor: 0,
                    aggro_radius: 8.0,
                    leash_radius: 20.0,
                    attack_range: 2.0,
                    attack_cooldown: 1.0,
                },
                behavior: MobBehavior::Aggro,
                loot_table: LootTableId::new("nonexistent_drops"),
                xp_reward: 10,
                credit_reward: 5,
                tags: vec![],
            },
        );

        let validator = ContentValidator::new(ValidationMode::Development);
        let result = validator.validate(&registry);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_mob_reference_prod_mode() {
        let mut registry = create_valid_registry();

        // Add mob with invalid loot table reference
        registry.mobs.insert(
            MobDefId::new("bad_mob"),
            MobDefinition {
                mob_id: MobDefId::new("bad_mob"),
                display_name: "Bad Mob".to_string(),
                stats: MobStats {
                    hp: 50,
                    damage: 5,
                    speed: 3.0,
                    armor: 0,
                    aggro_radius: 8.0,
                    leash_radius: 20.0,
                    attack_range: 2.0,
                    attack_cooldown: 1.0,
                },
                behavior: MobBehavior::Aggro,
                loot_table: LootTableId::new("nonexistent_drops"),
                xp_reward: 10,
                credit_reward: 5,
                tags: vec![],
            },
        );

        let validator = ContentValidator::new(ValidationMode::Production);
        let result = validator.validate(&registry).unwrap();
        assert!(!result.passed);
        assert_eq!(result.errors.len(), 1);
    }
}
