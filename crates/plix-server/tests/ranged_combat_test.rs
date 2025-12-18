//! Integration tests: Ranged combat with projectiles
//! Feature 022 - Weapons & Items v1

use plix_common::math::Vec3;
use plix_common::time::Tick;
use plix_common::types::{BlockPos, ItemId, PlayerId};

use plix_server::weapons::{
    cooldown::CooldownState,
    defs::{BOW_DEF, SWORD_DEF},
    melee::{MeleeSystem, MeleeTarget},
    projectiles::{ProjectileManager, ProjectileTarget, WorldCollision, MAX_PROJECTILES},
    ranged::RangedSystem,
    recoil::RecoilState,
};

// ============================================================================
// Mock World for Testing
// ============================================================================

struct EmptyWorld;

impl WorldCollision for EmptyWorld {
    fn is_solid(&self, _pos: BlockPos) -> bool {
        false
    }
}

struct WallWorld {
    wall_z: i32,
}

impl WorldCollision for WallWorld {
    fn is_solid(&self, pos: BlockPos) -> bool {
        pos.z == self.wall_z
    }
}

// ============================================================================
// Projectile Spawning Tests
// ============================================================================

#[test]
fn test_projectile_spawn_success() {
    let mut mgr = ProjectileManager::new();

    let owner = PlayerId(1);
    let position = Vec3::new(0.0, 1.6, 0.0);
    let velocity = Vec3::new(0.0, 0.0, 0.5); // 30 blocks/sec at 60 TPS

    let id = mgr.spawn(
        owner,
        position,
        velocity,
        BOW_DEF.damage,
        BOW_DEF.projectile_ttl_ticks,
        Tick(0),
    );

    assert!(id.is_some());
    assert_eq!(mgr.active_count(), 1);
}

#[test]
fn test_projectile_spawn_at_limit() {
    let mut mgr = ProjectileManager::new();

    // Fill to capacity
    for _ in 0..MAX_PROJECTILES {
        let result = mgr.spawn(PlayerId(1), Vec3::ZERO, Vec3::Z, 15, 180, Tick(0));
        assert!(result.is_some());
    }

    assert!(mgr.is_full());
    assert_eq!(mgr.active_count(), MAX_PROJECTILES);

    // Should fail to spawn more
    let result = mgr.spawn(PlayerId(1), Vec3::ZERO, Vec3::Z, 15, 180, Tick(0));
    assert!(result.is_none());
}

// ============================================================================
// Projectile Movement Tests
// ============================================================================

#[test]
fn test_projectile_moves_forward() {
    let mut mgr = ProjectileManager::new();
    let world = EmptyWorld;

    let start_pos = Vec3::new(0.0, 1.6, 0.0);
    let velocity = Vec3::new(0.0, 0.0, 0.5); // 0.5 blocks per tick

    mgr.spawn(PlayerId(1), start_pos, velocity, 15, 180, Tick(0));

    // Tick once
    let (impacts, despawns) = mgr.tick(&[], &world, Tick(1));

    assert!(impacts.is_empty());
    assert!(despawns.is_empty());

    // Check projectile moved
    let projectile = mgr.iter().next().unwrap();
    assert!((projectile.position.z - 0.5).abs() < 0.01);
}

#[test]
fn test_projectile_expires_after_ttl() {
    let mut mgr = ProjectileManager::new();
    let world = EmptyWorld;

    // Spawn with 10 tick TTL
    mgr.spawn(PlayerId(1), Vec3::ZERO, Vec3::Z * 0.1, 15, 10, Tick(0));

    // Tick up to expiration
    for tick in 1..10 {
        let (impacts, despawns) = mgr.tick(&[], &world, Tick(tick));
        assert!(impacts.is_empty());
        assert!(despawns.is_empty());
        assert_eq!(mgr.active_count(), 1);
    }

    // On tick 10, should expire
    let (impacts, despawns) = mgr.tick(&[], &world, Tick(10));
    assert!(impacts.is_empty());
    assert_eq!(despawns.len(), 1);
    assert_eq!(mgr.active_count(), 0);
}

// ============================================================================
// Projectile Collision Tests
// ============================================================================

#[test]
fn test_projectile_hits_player() {
    let mut mgr = ProjectileManager::new();
    let world = EmptyWorld;

    // Spawn projectile moving toward target
    let shooter = PlayerId(1);
    let target = PlayerId(2);
    let start_pos = Vec3::new(0.0, 1.0, 0.0);
    let velocity = Vec3::new(0.0, 0.0, 1.0); // 1 block per tick

    mgr.spawn(shooter, start_pos, velocity, 15, 180, Tick(0));

    // Target at z=2.5
    let targets = vec![ProjectileTarget {
        id: target,
        position: Vec3::new(0.0, 1.0, 2.5),
        radius: 0.4,
        half_height: 0.9,
    }];

    // Tick until collision
    let mut hit = false;
    for tick in 1..=10 {
        let (impacts, _) = mgr.tick(&targets, &world, Tick(tick));
        if !impacts.is_empty() {
            hit = true;
            assert_eq!(impacts[0].owner, shooter);
            assert_eq!(impacts[0].damage, 15);
            break;
        }
    }

    assert!(hit, "Projectile should hit target");
    assert_eq!(mgr.active_count(), 0);
}

#[test]
fn test_projectile_no_self_hit() {
    let mut mgr = ProjectileManager::new();
    let world = EmptyWorld;

    let shooter = PlayerId(1);
    let start_pos = Vec3::new(0.0, 1.0, 0.0);
    let velocity = Vec3::new(0.0, 0.0, 0.1);

    mgr.spawn(shooter, start_pos, velocity, 15, 10, Tick(0));

    // Owner as target
    let targets = vec![ProjectileTarget {
        id: shooter, // Same as shooter
        position: start_pos,
        radius: 0.4,
        half_height: 0.9,
    }];

    // Tick until expiration
    for tick in 1..=10 {
        let (impacts, _) = mgr.tick(&targets, &world, Tick(tick));
        for impact in impacts {
            if let plix_server::weapons::projectiles::ProjectileImpactType::Player(id) =
                impact.impact_type
            {
                assert_ne!(id, shooter, "Projectile should not hit its owner");
            }
        }
    }
}

#[test]
fn test_projectile_hits_block() {
    let mut mgr = ProjectileManager::new();
    let world = WallWorld { wall_z: 5 };

    // Spawn projectile moving toward wall
    let start_pos = Vec3::new(0.5, 1.0, 0.5);
    let velocity = Vec3::new(0.0, 0.0, 1.0);

    mgr.spawn(PlayerId(1), start_pos, velocity, 15, 180, Tick(0));

    // Tick until block collision
    let mut hit_block = false;
    for tick in 1..=10 {
        let (impacts, _) = mgr.tick(&[], &world, Tick(tick));
        if !impacts.is_empty() {
            hit_block = true;
            match impacts[0].impact_type {
                plix_server::weapons::projectiles::ProjectileImpactType::Block(_) => {}
                _ => panic!("Expected block impact"),
            }
            break;
        }
    }

    assert!(hit_block, "Projectile should hit wall at z=5");
}

// ============================================================================
// Cooldown Tests
// ============================================================================

#[test]
fn test_weapon_cooldown_enforced() {
    let mut cooldown = CooldownState::new();
    let current = Tick(100);

    // Initially ready
    assert!(cooldown.is_ready(ItemId::BOW, current));

    // Trigger cooldown (48 ticks for bow)
    cooldown.trigger(ItemId::BOW, current, BOW_DEF.cooldown_ticks);

    // Should not be ready during cooldown
    assert!(!cooldown.is_ready(ItemId::BOW, Tick(100)));
    assert!(!cooldown.is_ready(ItemId::BOW, Tick(147)));
    assert_eq!(cooldown.remaining(ItemId::BOW, Tick(100)), 48);
    assert_eq!(cooldown.remaining(ItemId::BOW, Tick(130)), 18);

    // Should be ready after cooldown
    assert!(cooldown.is_ready(ItemId::BOW, Tick(148)));
    assert_eq!(cooldown.remaining(ItemId::BOW, Tick(148)), 0);
}

#[test]
fn test_cooldowns_independent_per_weapon() {
    let mut cooldown = CooldownState::new();
    let current = Tick(100);

    cooldown.trigger(ItemId::SWORD, current, SWORD_DEF.cooldown_ticks); // 36 ticks
    cooldown.trigger(ItemId::BOW, current, BOW_DEF.cooldown_ticks); // 48 ticks

    // Sword ready at 136, bow ready at 148
    assert!(cooldown.is_ready(ItemId::SWORD, Tick(140)));
    assert!(!cooldown.is_ready(ItemId::BOW, Tick(140)));

    assert!(cooldown.is_ready(ItemId::SWORD, Tick(150)));
    assert!(cooldown.is_ready(ItemId::BOW, Tick(150)));
}

// ============================================================================
// Spread and Recoil Tests
// ============================================================================

#[test]
fn test_base_spread_standing() {
    let recoil = RecoilState::default();
    let spread = RangedSystem::calculate_spread(&BOW_DEF, &recoil, false, Tick(0));
    assert_eq!(spread, 2.0); // Base spread only
}

#[test]
fn test_spread_increases_when_moving() {
    let recoil = RecoilState::default();

    let standing = RangedSystem::calculate_spread(&BOW_DEF, &recoil, false, Tick(0));
    let moving = RangedSystem::calculate_spread(&BOW_DEF, &recoil, true, Tick(0));

    assert!(moving > standing);
    assert_eq!(moving, 3.0); // 2.0 * 1.5 = 3.0
}

#[test]
fn test_recoil_accumulates() {
    let mut recoil = RecoilState::new(30, 5.0);

    // First shot
    recoil.add_shot(1.0, Tick(100));
    assert_eq!(recoil.current_recoil(Tick(100)), 1.0);

    // Second shot immediately after
    recoil.add_shot(1.0, Tick(100));
    assert_eq!(recoil.current_recoil(Tick(100)), 2.0);

    // Third shot
    recoil.add_shot(1.0, Tick(100));
    assert_eq!(recoil.current_recoil(Tick(100)), 3.0);
}

#[test]
fn test_recoil_capped_at_max() {
    let mut recoil = RecoilState::new(30, 5.0);

    // Add lots of recoil
    for _ in 0..10 {
        recoil.add_shot(1.0, Tick(100));
    }

    // Should be capped at 5.0
    assert_eq!(recoil.current_recoil(Tick(100)), 5.0);
}

#[test]
fn test_recoil_decays_over_time() {
    let mut recoil = RecoilState::new(30, 5.0); // 30 tick recovery

    recoil.add_shot(3.0, Tick(100));
    assert_eq!(recoil.current_recoil(Tick(100)), 3.0);

    // At 15 ticks (50% decay)
    let mid_recoil = recoil.current_recoil(Tick(115));
    assert!((mid_recoil - 1.5).abs() < 0.01);

    // At 30 ticks (full decay)
    assert_eq!(recoil.current_recoil(Tick(130)), 0.0);
}

#[test]
fn test_spread_includes_recoil() {
    let mut recoil = RecoilState::new(30, 5.0);
    recoil.add_shot(2.0, Tick(100));

    // Standing: base 2.0 + recoil 2.0 = 4.0
    let spread = recoil.get_effective_spread(2.0, false, 1.5, Tick(100));
    assert_eq!(spread, 4.0);

    // Moving: base 2.0 * 1.5 + recoil 2.0 = 5.0
    let spread = recoil.get_effective_spread(2.0, true, 1.5, Tick(100));
    assert_eq!(spread, 5.0);
}

// ============================================================================
// Melee vs Ranged Weapon Definition Tests
// ============================================================================

#[test]
fn test_bow_is_ranged_weapon() {
    use plix_common::inventory::WeaponType;
    assert_eq!(BOW_DEF.weapon_type, WeaponType::Ranged);
    assert_eq!(BOW_DEF.damage, 15);
    assert_eq!(BOW_DEF.cooldown_ticks, 48);
    assert_eq!(BOW_DEF.projectile_speed, 30.0);
    assert_eq!(BOW_DEF.projectile_ttl_ticks, 180);
}

#[test]
fn test_sword_is_melee_weapon() {
    use plix_common::inventory::WeaponType;
    assert_eq!(SWORD_DEF.weapon_type, WeaponType::Melee);
    assert_eq!(SWORD_DEF.damage, 25);
    assert_eq!(SWORD_DEF.cooldown_ticks, 36);
    assert_eq!(SWORD_DEF.cone_angle_deg, 30.0);
    assert_eq!(SWORD_DEF.reach_blocks, 2.5);
}

// ============================================================================
// Melee Cone Hit Detection Tests
// ============================================================================

#[test]
fn test_melee_cone_hit_in_front() {
    let attacker_pos = Vec3::new(0.0, 1.0, 0.0);
    let forward = Vec3::new(0.0, 0.0, 1.0);
    let target_pos = Vec3::new(0.0, 1.0, 2.0);

    assert!(MeleeSystem::is_in_cone(
        attacker_pos,
        forward,
        target_pos,
        0.4,
        30.0,
        2.5
    ));
}

#[test]
fn test_melee_cone_miss_behind() {
    let attacker_pos = Vec3::new(0.0, 1.0, 0.0);
    let forward = Vec3::new(0.0, 0.0, 1.0);
    let target_pos = Vec3::new(0.0, 1.0, -2.0); // Behind

    assert!(!MeleeSystem::is_in_cone(
        attacker_pos,
        forward,
        target_pos,
        0.4,
        30.0,
        2.5
    ));
}

#[test]
fn test_melee_cone_miss_too_far() {
    let attacker_pos = Vec3::new(0.0, 1.0, 0.0);
    let forward = Vec3::new(0.0, 0.0, 1.0);
    let target_pos = Vec3::new(0.0, 1.0, 5.0); // Beyond 2.5 blocks

    assert!(!MeleeSystem::is_in_cone(
        attacker_pos,
        forward,
        target_pos,
        0.4,
        30.0,
        2.5
    ));
}

#[test]
fn test_melee_hits_multiple_targets() {
    let attacker_pos = Vec3::new(0.0, 1.0, 0.0);
    let forward = Vec3::new(0.0, 0.0, 1.0);

    let targets = vec![
        MeleeTarget {
            id: PlayerId(1),
            position: Vec3::new(0.0, 1.0, 1.5),
            radius: 0.4,
        },
        MeleeTarget {
            id: PlayerId(2),
            position: Vec3::new(0.3, 1.0, 1.8),
            radius: 0.4,
        },
        MeleeTarget {
            id: PlayerId(3),
            position: Vec3::new(0.0, 1.0, 10.0),
            radius: 0.4,
        }, // Too far
    ];

    let result = MeleeSystem::attack(
        PlayerId(0),
        attacker_pos,
        forward,
        targets.into_iter(),
        &SWORD_DEF,
    );

    assert_eq!(result.hit_players.len(), 2);
    assert!(result.hit_players.contains(&PlayerId(1)));
    assert!(result.hit_players.contains(&PlayerId(2)));
    assert_eq!(result.damage, 25);
}

// ============================================================================
// Player Weapon State Tests
// ============================================================================

#[test]
fn test_player_weapon_state_default() {
    use plix_server::weapons::PlayerWeaponState;

    let state = PlayerWeaponState::default();

    // Should be ready for all weapons initially
    assert!(state.cooldowns.is_ready(ItemId::SWORD, Tick(0)));
    assert!(state.cooldowns.is_ready(ItemId::BOW, Tick(0)));

    // No recoil initially
    assert_eq!(state.recoil.current_recoil(Tick(0)), 0.0);
}

// ============================================================================
// Velocity Calculation Tests
// ============================================================================

#[test]
fn test_projectile_velocity_calculation() {
    let direction = Vec3::new(0.0, 0.0, 1.0);
    let speed = 30.0; // 30 blocks per second

    let velocity = RangedSystem::calculate_velocity(direction, speed);

    // At 60 TPS, velocity should be 0.5 blocks per tick
    assert!((velocity.z - 0.5).abs() < 0.001);
    assert_eq!(velocity.x, 0.0);
    assert_eq!(velocity.y, 0.0);
}

#[test]
fn test_projectile_velocity_diagonal() {
    let direction = Vec3::new(1.0, 0.0, 1.0).normalize();
    let speed = 30.0;

    let velocity = RangedSystem::calculate_velocity(direction, speed);

    // Magnitude should still be 0.5 blocks/tick
    let magnitude = velocity.length();
    assert!((magnitude - 0.5).abs() < 0.001);
}

// ============================================================================
// Item Registry Tests
// ============================================================================

#[test]
fn test_bow_in_item_registry() {
    use plix_server::inventory::item_registry::{get_item_def, get_weapon_damage};

    let bow_def = get_item_def(ItemId::BOW);
    assert!(bow_def.is_some());

    let bow = bow_def.unwrap();
    assert_eq!(bow.name, "Bow");
    assert_eq!(bow.damage(), 15);

    assert_eq!(get_weapon_damage(ItemId::BOW), Some(15));
}

// ============================================================================
// Spread Application Tests
// ============================================================================

#[test]
fn test_spread_deterministic_with_same_seed() {
    let direction = Vec3::new(0.0, 0.0, 1.0);

    let result1 = RangedSystem::apply_spread(direction, 5.0, 12345);
    let result2 = RangedSystem::apply_spread(direction, 5.0, 12345);

    assert!((result1 - result2).length() < 0.001);
}

#[test]
fn test_spread_different_with_different_seed() {
    let direction = Vec3::new(0.0, 0.0, 1.0);

    let result1 = RangedSystem::apply_spread(direction, 5.0, 100);
    let result2 = RangedSystem::apply_spread(direction, 5.0, 200);

    // Should be different (with high probability)
    assert!((result1 - result2).length() > 0.001);
}

#[test]
fn test_zero_spread_preserves_direction() {
    let direction = Vec3::new(0.0, 0.0, 1.0);

    let result = RangedSystem::apply_spread(direction, 0.0, 12345);

    assert!((result - direction).length() < 0.001);
}
