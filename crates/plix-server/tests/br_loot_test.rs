//! BR Lite loot integration tests
//!
//! Tests for User Story 4: Loot Collection

use glam::{Vec2, Vec3};
use plix_common::time::Tick;
use plix_common::types::PlayerId;
use plix_server::br_lite::{BrLiteConfig, BrLiteCoordinator, BrLiteEvent, LootType, ZonePhase};

fn test_config() -> BrLiteConfig {
    BrLiteConfig {
        min_players: 2,
        post_match_delay_secs: 5,
        bonus_duration_secs: 10,
        phases: vec![ZonePhase::new(60, 30, 0.8, 5)],
        initial_radius: Some(50.0),
        loot_enabled: true,
        ..Default::default()
    }
}

/// T058: Test loot spawn creates item
#[test]
fn test_loot_spawn_creates_item() {
    let config = test_config();
    let mut coordinator = BrLiteCoordinator::new(config, Vec2::new(64.0, 64.0));

    coordinator.add_player(PlayerId(1));
    coordinator.add_player(PlayerId(2));
    coordinator.start_match();

    let position = Vec3::new(10.0, 2.0, 10.0);
    let event = coordinator.spawn_loot(position, LootType::HealthPack { heal_amount: 25 });

    // Check event was generated
    match event {
        BrLiteEvent::LootSpawn {
            loot_id,
            position: pos,
            loot_type,
        } => {
            assert_eq!(loot_id, 0); // First loot
            assert_eq!(pos.x, 10.0);
            assert_eq!(pos.z, 10.0);
            assert!(matches!(
                loot_type,
                LootType::HealthPack { heal_amount: 25 }
            ));
        }
        _ => panic!("Expected LootSpawn event"),
    }

    // Verify loot exists
    let items = coordinator.loot_items();
    assert_eq!(items.len(), 1);
    assert!(!items[0].collected);
}

/// T059: Test player can pick up nearby loot
#[test]
fn test_loot_pickup() {
    let config = test_config();
    let mut coordinator = BrLiteCoordinator::new(config, Vec2::new(64.0, 64.0));

    coordinator.add_player(PlayerId(1));
    coordinator.add_player(PlayerId(2));
    coordinator.start_match();

    // Spawn loot
    let loot_pos = Vec3::new(10.0, 2.0, 10.0);
    coordinator.spawn_loot(loot_pos, LootType::HealthPack { heal_amount: 25 });

    // Player moves to loot position
    let events = coordinator.check_loot_pickup(PlayerId(1), loot_pos, Tick(100));

    // Should have pickup event
    let has_pickup = events.iter().any(
        |e| matches!(e, BrLiteEvent::LootPickup { player_id, .. } if *player_id == PlayerId(1)),
    );
    assert!(has_pickup, "Expected LootPickup event");

    // Loot should be marked as collected
    let items = coordinator.loot_items();
    assert!(items[0].collected);
}

/// T060: Test loot cannot be picked up twice
#[test]
fn test_loot_cannot_pickup_twice() {
    let config = test_config();
    let mut coordinator = BrLiteCoordinator::new(config, Vec2::new(64.0, 64.0));

    coordinator.add_player(PlayerId(1));
    coordinator.add_player(PlayerId(2));
    coordinator.start_match();

    let loot_pos = Vec3::new(10.0, 2.0, 10.0);
    coordinator.spawn_loot(loot_pos, LootType::HealthPack { heal_amount: 25 });

    // First pickup
    let events1 = coordinator.check_loot_pickup(PlayerId(1), loot_pos, Tick(100));
    assert!(!events1.is_empty());

    // Second pickup attempt (same player)
    let events2 = coordinator.check_loot_pickup(PlayerId(1), loot_pos, Tick(101));
    assert!(
        events2.is_empty(),
        "Should not be able to pick up same loot twice"
    );

    // Third pickup attempt (different player)
    let events3 = coordinator.check_loot_pickup(PlayerId(2), loot_pos, Tick(102));
    assert!(
        events3.is_empty(),
        "Other player should not be able to pick up collected loot"
    );
}

/// T061: Test health pack heals player
#[test]
fn test_health_pack_heals() {
    let config = test_config();
    let mut coordinator = BrLiteCoordinator::new(config, Vec2::new(64.0, 64.0));

    coordinator.add_player(PlayerId(1));
    coordinator.add_player(PlayerId(2));
    coordinator.start_match();

    let loot_pos = Vec3::new(10.0, 2.0, 10.0);
    coordinator.spawn_loot(loot_pos, LootType::HealthPack { heal_amount: 30 });

    let events = coordinator.check_loot_pickup(PlayerId(1), loot_pos, Tick(100));

    // Should have heal event
    let has_heal = events.iter().any(|e| {
        matches!(e, BrLiteEvent::PlayerHealed { player_id, heal_amount }
            if *player_id == PlayerId(1) && *heal_amount == 30)
    });
    assert!(has_heal, "Expected PlayerHealed event with heal_amount 30");
}

/// T062: Test speed boost applies effect
#[test]
fn test_speed_boost_applies_effect() {
    let config = test_config();
    let mut coordinator = BrLiteCoordinator::new(config, Vec2::new(64.0, 64.0));

    coordinator.add_player(PlayerId(1));
    coordinator.add_player(PlayerId(2));
    coordinator.start_match();

    // Default speed multiplier is 1.0
    assert_eq!(coordinator.get_speed_multiplier(PlayerId(1)), 1.0);

    let loot_pos = Vec3::new(10.0, 2.0, 10.0);
    coordinator.spawn_loot(
        loot_pos,
        LootType::SpeedBoost {
            multiplier: 1.5,
            duration_ticks: 600,
        },
    );

    let events = coordinator.check_loot_pickup(PlayerId(1), loot_pos, Tick(100));

    // Should have speed boost event
    let has_boost = events.iter().any(|e| {
        matches!(e, BrLiteEvent::SpeedBoostApplied { player_id, multiplier, .. }
            if *player_id == PlayerId(1) && (*multiplier - 1.5).abs() < 0.01)
    });
    assert!(has_boost, "Expected SpeedBoostApplied event");

    // Speed multiplier should be updated
    assert_eq!(coordinator.get_speed_multiplier(PlayerId(1)), 1.5);
}

/// T063: Test speed boost expires
#[test]
fn test_speed_boost_expires() {
    let config = BrLiteConfig {
        bonus_duration_secs: 1, // 1 second = 60 ticks
        ..test_config()
    };
    let mut coordinator = BrLiteCoordinator::new(config, Vec2::new(64.0, 64.0));

    coordinator.add_player(PlayerId(1));
    coordinator.add_player(PlayerId(2));
    coordinator.start_match();

    let loot_pos = Vec3::new(10.0, 2.0, 10.0);
    coordinator.spawn_loot(
        loot_pos,
        LootType::SpeedBoost {
            multiplier: 1.5,
            duration_ticks: 60, // 1 second
        },
    );

    // Pickup at tick 100
    coordinator.check_loot_pickup(PlayerId(1), loot_pos, Tick(100));
    assert_eq!(coordinator.get_speed_multiplier(PlayerId(1)), 1.5);

    // Tick at 159 - effect still active
    let positions = std::collections::HashMap::new();
    coordinator.tick(Tick(159), &positions);
    assert_eq!(coordinator.get_speed_multiplier(PlayerId(1)), 1.5);

    // Tick at 160 - effect should expire (100 + 60 = 160)
    coordinator.tick(Tick(160), &positions);
    assert_eq!(
        coordinator.get_speed_multiplier(PlayerId(1)),
        1.0,
        "Speed boost should expire after duration"
    );
}

/// T064: Test loot pickup radius
#[test]
fn test_loot_pickup_radius() {
    let config = test_config();
    let mut coordinator = BrLiteCoordinator::new(config, Vec2::new(64.0, 64.0));

    coordinator.add_player(PlayerId(1));
    coordinator.add_player(PlayerId(2));
    coordinator.start_match();

    let loot_pos = Vec3::new(10.0, 2.0, 10.0);
    coordinator.spawn_loot(loot_pos, LootType::HealthPack { heal_amount: 25 });

    // Player too far away (pickup radius is 1.0)
    let far_pos = Vec3::new(12.0, 2.0, 10.0); // 2 units away
    let events1 = coordinator.check_loot_pickup(PlayerId(1), far_pos, Tick(100));
    assert!(
        events1.is_empty(),
        "Should not pickup loot from 2 units away"
    );

    // Player close enough
    let near_pos = Vec3::new(10.5, 2.0, 10.0); // 0.5 units away
    let events2 = coordinator.check_loot_pickup(PlayerId(1), near_pos, Tick(101));
    assert!(
        !events2.is_empty(),
        "Should pickup loot from 0.5 units away"
    );
}

/// T065: Test dead player cannot pickup loot
#[test]
fn test_dead_player_cannot_pickup() {
    let config = test_config();
    let mut coordinator = BrLiteCoordinator::new(config, Vec2::new(64.0, 64.0));

    coordinator.add_player(PlayerId(1));
    coordinator.add_player(PlayerId(2));
    coordinator.add_player(PlayerId(3));
    coordinator.start_match();

    let loot_pos = Vec3::new(10.0, 2.0, 10.0);
    coordinator.spawn_loot(loot_pos, LootType::HealthPack { heal_amount: 25 });

    // Eliminate player 1
    coordinator.eliminate_player(PlayerId(1));

    // Dead player tries to pickup
    let events = coordinator.check_loot_pickup(PlayerId(1), loot_pos, Tick(100));
    assert!(
        events.is_empty(),
        "Dead player should not be able to pickup loot"
    );

    // Alive player can still pickup
    let events2 = coordinator.check_loot_pickup(PlayerId(2), loot_pos, Tick(101));
    assert!(!events2.is_empty(), "Alive player should be able to pickup");
}

/// T066: Test multiple loot spawns
#[test]
fn test_multiple_loot_spawns() {
    let config = test_config();
    let mut coordinator = BrLiteCoordinator::new(config, Vec2::new(64.0, 64.0));

    coordinator.add_player(PlayerId(1));
    coordinator.add_player(PlayerId(2));
    coordinator.start_match();

    // Spawn multiple loot items
    coordinator.spawn_loot(
        Vec3::new(10.0, 2.0, 10.0),
        LootType::HealthPack { heal_amount: 25 },
    );
    coordinator.spawn_loot(
        Vec3::new(20.0, 2.0, 20.0),
        LootType::SpeedBoost {
            multiplier: 1.5,
            duration_ticks: 600,
        },
    );
    coordinator.spawn_loot(
        Vec3::new(30.0, 2.0, 30.0),
        LootType::HealthPack { heal_amount: 50 },
    );

    let items = coordinator.loot_items();
    assert_eq!(items.len(), 3);
    assert_eq!(items[0].id, 0);
    assert_eq!(items[1].id, 1);
    assert_eq!(items[2].id, 2);
}

/// T067: Test loot disabled config
#[test]
fn test_loot_disabled() {
    let config = BrLiteConfig {
        loot_enabled: false,
        ..test_config()
    };
    let mut coordinator = BrLiteCoordinator::new(config, Vec2::new(64.0, 64.0));

    coordinator.add_player(PlayerId(1));
    coordinator.add_player(PlayerId(2));
    coordinator.start_match();

    // Spawn loot anyway (server can still spawn)
    let loot_pos = Vec3::new(10.0, 2.0, 10.0);
    coordinator.spawn_loot(loot_pos, LootType::HealthPack { heal_amount: 25 });

    // But pickup should not work when loot is disabled
    let events = coordinator.check_loot_pickup(PlayerId(1), loot_pos, Tick(100));
    assert!(
        events.is_empty(),
        "Should not be able to pickup when loot is disabled"
    );
}

/// Test effects cleared on elimination
#[test]
fn test_effects_cleared_on_elimination() {
    let config = test_config();
    let mut coordinator = BrLiteCoordinator::new(config, Vec2::new(64.0, 64.0));

    coordinator.add_player(PlayerId(1));
    coordinator.add_player(PlayerId(2));
    coordinator.add_player(PlayerId(3));
    coordinator.start_match();

    // Player 1 picks up speed boost
    let loot_pos = Vec3::new(10.0, 2.0, 10.0);
    coordinator.spawn_loot(
        loot_pos,
        LootType::SpeedBoost {
            multiplier: 1.5,
            duration_ticks: 6000,
        },
    );
    coordinator.check_loot_pickup(PlayerId(1), loot_pos, Tick(100));
    assert_eq!(coordinator.get_speed_multiplier(PlayerId(1)), 1.5);

    // Eliminate player 1
    coordinator.eliminate_player(PlayerId(1));

    // Effect should be cleared
    assert_eq!(
        coordinator.get_speed_multiplier(PlayerId(1)),
        1.0,
        "Effects should be cleared on elimination"
    );
}
