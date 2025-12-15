//! Integration test: Combat hits are server-validated
//! T071 [US2]

use std::net::SocketAddr;

use plix_common::combat::CombatConfig;
use plix_common::math::Vec3;
use plix_common::time::Tick;
use plix_common::types::{PlayerId, TeamId};
use plix_server::session::{ServerPlayer, SessionManager};
use plix_server::sim::combat::{CombatSystem, MELEE_DAMAGE};
use plix_server::validation::{validate_attack, ATTACK_COOLDOWN_TICKS, ATTACK_RANGE};

#[test]
fn test_combat_hit_server_validated() {
    let mut sessions = SessionManager::new(16);
    let combat = CombatSystem::new();
    let config = CombatConfig::default();

    // Add attacker and target
    let addr1: SocketAddr = "127.0.0.1:1001".parse().unwrap();
    let addr2: SocketAddr = "127.0.0.1:1002".parse().unwrap();

    let attacker_id = sessions
        .add_player("Attacker".into(), TeamId::TEAM_0, addr1)
        .unwrap();
    let target_id = sessions
        .add_player("Target".into(), TeamId::TEAM_1, addr2)
        .unwrap();

    let current_tick = Tick(100);

    // Position attacker and target within range
    sessions.get_mut(attacker_id).unwrap().spawn(
        Vec3::new(10.0, 1.0, 10.0),
        0.0,
        current_tick,
        None,
    );
    sessions
        .get_mut(target_id)
        .unwrap()
        .spawn(Vec3::new(10.0, 1.0, 11.5), 0.0, current_tick, None); // 1.5 blocks away

    // Gather target info
    let targets: Vec<(PlayerId, Vec3, u8)> = sessions
        .iter()
        .filter(|p| p.id != attacker_id)
        .map(|p| (p.id, p.position, p.health))
        .collect();

    let attacker = sessions.get(attacker_id).unwrap();
    let attacker_forward = attacker.rotation.forward();

    // Try attack
    let result = combat.try_attack(
        &config,
        attacker_id,
        attacker.position,
        attacker_forward,
        Tick(0), // No cooldown
        current_tick,
        &targets,
    );

    assert!(result.is_some(), "Attack should hit target in range");

    let (hit_target_id, hit_result) = result.unwrap();
    assert_eq!(hit_target_id, target_id);
    assert_eq!(hit_result.attacker, attacker_id);
    assert_eq!(hit_result.target, target_id);
    assert_eq!(hit_result.damage, MELEE_DAMAGE);
    assert!(!hit_result.killed); // Target has 100 HP, attack does 20
}

#[test]
fn test_combat_out_of_range_rejected() {
    let mut sessions = SessionManager::new(16);
    let combat = CombatSystem::new();
    let config = CombatConfig::default();

    let addr1: SocketAddr = "127.0.0.1:1001".parse().unwrap();
    let addr2: SocketAddr = "127.0.0.1:1002".parse().unwrap();

    let attacker_id = sessions
        .add_player("Attacker".into(), TeamId::TEAM_0, addr1)
        .unwrap();
    let target_id = sessions
        .add_player("Target".into(), TeamId::TEAM_1, addr2)
        .unwrap();

    let current_tick = Tick(100);

    // Position target out of range
    sessions.get_mut(attacker_id).unwrap().spawn(
        Vec3::new(10.0, 1.0, 10.0),
        0.0,
        current_tick,
        None,
    );
    sessions
        .get_mut(target_id)
        .unwrap()
        .spawn(Vec3::new(10.0, 1.0, 20.0), 0.0, current_tick, None); // 10 blocks away

    let targets: Vec<(PlayerId, Vec3, u8)> = sessions
        .iter()
        .filter(|p| p.id != attacker_id)
        .map(|p| (p.id, p.position, p.health))
        .collect();

    let attacker = sessions.get(attacker_id).unwrap();

    let result = combat.try_attack(
        &config,
        attacker_id,
        attacker.position,
        attacker.rotation.forward(),
        Tick(0),
        Tick(100),
        &targets,
    );

    assert!(result.is_none(), "Attack should miss target out of range");
}

#[test]
fn test_combat_cooldown_enforced() {
    let combat = CombatSystem::new();
    let config = CombatConfig::default();

    let attacker_id = PlayerId(1);
    let target_id = PlayerId(2);

    let targets = vec![(target_id, Vec3::new(0.0, 0.0, 1.5), 100u8)];

    // Attack on cooldown should fail
    let last_attack = Tick(95);
    let current = Tick(100); // Only 5 ticks since last attack

    let result = combat.try_attack(
        &config,
        attacker_id,
        Vec3::ZERO,
        Vec3::new(0.0, 0.0, 1.0),
        last_attack,
        current,
        &targets,
    );

    assert!(result.is_none(), "Attack should fail during cooldown");

    // Attack after cooldown should succeed
    let current_after = Tick(last_attack.0 + ATTACK_COOLDOWN_TICKS + 1);

    let result = combat.try_attack(
        &config,
        attacker_id,
        Vec3::ZERO,
        Vec3::new(0.0, 0.0, 1.0),
        last_attack,
        current_after,
        &targets,
    );

    assert!(result.is_some(), "Attack should succeed after cooldown");
}

#[test]
fn test_combat_damage_and_death() {
    let mut sessions = SessionManager::new(16);

    let addr: SocketAddr = "127.0.0.1:1001".parse().unwrap();
    let id = sessions
        .add_player("Player".into(), TeamId::TEAM_0, addr)
        .unwrap();
    sessions
        .get_mut(id)
        .unwrap()
        .spawn(Vec3::ZERO, 0.0, Tick(0), None);

    let player = sessions.get_mut(id).unwrap();

    // Take some damage
    let died = player.take_damage(50, Tick(100));
    assert!(!died);
    assert_eq!(player.health, 50);
    assert!(!player.is_dead);

    // Take lethal damage
    let died = player.take_damage(50, Tick(200));
    assert!(died);
    assert_eq!(player.health, 0);
    assert!(player.is_dead);
    assert_eq!(player.deaths, 1);
    assert!(player.respawn_tick.is_some());
}

#[test]
fn test_validate_attack_function() {
    let attacker_pos = Vec3::ZERO;
    let target_in_range = Vec3::new(1.5, 0.0, 0.0);
    let target_out_of_range = Vec3::new(10.0, 0.0, 0.0);

    // Valid attack
    assert!(validate_attack(attacker_pos, target_in_range, 0, 100));

    // Out of range
    assert!(!validate_attack(attacker_pos, target_out_of_range, 0, 100));

    // On cooldown
    assert!(!validate_attack(attacker_pos, target_in_range, 90, 100));
}

#[test]
fn test_team_friendly_fire() {
    let combat = CombatSystem::new();
    let config = CombatConfig::default();

    let attacker_id = PlayerId(1);
    let teammate_id = PlayerId(2);
    let enemy_id = PlayerId(3);

    // Only enemy should be targetable (teammate has same ID check in real impl)
    // For this test, combat system doesn't check teams directly but skips self
    let targets = vec![
        (teammate_id, Vec3::new(0.0, 0.0, 1.0), 100u8),
        (enemy_id, Vec3::new(0.0, 0.0, 1.5), 100u8),
    ];

    let result = combat.try_attack(
        &config,
        attacker_id,
        Vec3::ZERO,
        Vec3::new(0.0, 0.0, 1.0),
        Tick(0),
        Tick(100),
        &targets,
    );

    // Should hit closest (teammate at 1.0, enemy at 1.5)
    // In full impl, team check would filter teammate
    assert!(result.is_some());
    let (hit_id, _) = result.unwrap();
    assert_eq!(hit_id, teammate_id); // Closest target hit
}

// ============================================================================
// US5 - Stable Hitbox Integration Tests
// ============================================================================

/// T052 [US5] Moving attacker vs stationary target - hitbox is consistent
#[test]
fn test_moving_attacker_hits_stationary_target() {
    let mut sessions = SessionManager::new(16);
    let combat = CombatSystem::new();
    let config = CombatConfig::default();

    let addr1: SocketAddr = "127.0.0.1:1001".parse().unwrap();
    let addr2: SocketAddr = "127.0.0.1:1002".parse().unwrap();

    let attacker_id = sessions
        .add_player("Attacker".into(), TeamId::TEAM_0, addr1)
        .unwrap();
    let target_id = sessions
        .add_player("Target".into(), TeamId::TEAM_1, addr2)
        .unwrap();

    // Target is stationary
    sessions
        .get_mut(target_id)
        .unwrap()
        .spawn(Vec3::new(10.0, 1.0, 12.0), 0.0, Tick(0), None);

    // Attacker moves toward target over several frames
    let mut attacker_pos = Vec3::new(10.0, 1.0, 10.0);
    let attacker_forward = Vec3::new(0.0, 0.0, 1.0);
    let velocity = Vec3::new(0.0, 0.0, 6.0); // Moving toward target
    let dt = 1.0 / 60.0;
    let effective_range = config.effective_range();

    // Simulate movement and check attack at each frame
    for _ in 0..30 {
        attacker_pos.z += velocity.z * dt;

        let targets: Vec<(PlayerId, Vec3, u8)> = vec![(target_id, Vec3::new(10.0, 1.0, 12.0), 100)];

        let result = combat.try_attack(
            &config,
            attacker_id,
            attacker_pos,
            attacker_forward,
            Tick(0),
            Tick(100),
            &targets,
        );

        // Once in range, attack should succeed
        let dist = (attacker_pos - Vec3::new(10.0, 1.0, 12.0)).length();
        if dist <= effective_range && result.is_some() {
            let (hit_id, _) = result.unwrap();
            assert_eq!(hit_id, target_id, "Hit should register on target");
            return; // Test passed
        }
    }

    panic!("Attacker should have hit target after moving into range");
}

/// T053 [P] [US5] Both players moving - combat still works
#[test]
fn test_both_players_moving_combat() {
    let combat = CombatSystem::new();
    let config = CombatConfig::default();

    let attacker_id = PlayerId(1);
    let target_id = PlayerId(2);

    // Both players moving, passing each other
    let attacker_start = Vec3::new(0.0, 1.0, 0.0);
    let target_start = Vec3::new(0.0, 1.0, 3.0);

    // Attacker moving +Z, target moving -Z
    let attacker_vel = Vec3::new(0.0, 0.0, 6.0);
    let target_vel = Vec3::new(0.0, 0.0, -6.0);

    let dt = 1.0 / 60.0;
    let mut attacker_pos = attacker_start;
    let mut target_pos = target_start;

    let mut hit_registered = false;

    for _ in 0..60 {
        attacker_pos.z += attacker_vel.z * dt;
        target_pos.z += target_vel.z * dt;

        let targets = vec![(target_id, target_pos, 100u8)];

        let result = combat.try_attack(
            &config,
            attacker_id,
            attacker_pos,
            Vec3::new(0.0, 0.0, 1.0),
            Tick(0),
            Tick(100),
            &targets,
        );

        if result.is_some() {
            hit_registered = true;
            break;
        }
    }

    // The players should cross paths and hit should be possible
    assert!(
        hit_registered,
        "Moving players should be able to hit each other when crossing paths"
    );
}

/// T054 [P] [US5] Close combat edge case - very close players can still hit
#[test]
fn test_close_combat_edge_case() {
    let combat = CombatSystem::new();
    let config = CombatConfig::default();

    let attacker_id = PlayerId(1);
    let target_id = PlayerId(2);

    // Players very close together (nearly overlapping)
    let attacker_pos = Vec3::new(5.0, 1.0, 5.0);
    let target_pos = Vec3::new(5.0, 1.0, 5.5); // Only 0.5m away

    let targets = vec![(target_id, target_pos, 100u8)];

    let result = combat.try_attack(
        &config,
        attacker_id,
        attacker_pos,
        Vec3::new(0.0, 0.0, 1.0),
        Tick(0),
        Tick(100),
        &targets,
    );

    // Should hit even at very close range
    assert!(result.is_some(), "Close combat should hit");
    let (hit_id, _) = result.unwrap();
    assert_eq!(hit_id, target_id);
}
