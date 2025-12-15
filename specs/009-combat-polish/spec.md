# Feature Specification: Combat Polish

**Feature Branch**: `009-combat-polish`
**Created**: 2025-12-15
**Status**: Draft
**Input**: User description: "Feature 009 – Combat Polish: Refine combat system with cooldowns, knockback, invulnerability, and latency-tolerant hit registration"

---

## Overview

This feature refines the existing combat system to improve fairness, responsiveness, and game feel under real multiplayer conditions. It introduces attack cooldowns, tuned attack ranges, knockback feedback, spawn invulnerability, and latency-tolerant hit registration.

The goal is to make combat feel consistent, readable, and fair without introducing complex weapon systems.

### Goals

- Prevent attack spamming via server-authoritative cooldowns
- Tune melee range for clarity and consistency
- Add physical feedback through knockback
- Protect respawning players from instant deaths
- Improve hit registration under network latency

### Non-Goals (Out of Scope)

- Weapon types or loadouts
- Client-side hit authority
- Lag compensation with server rewind
- Advanced physics impulses (ragdolls, force stacking)

---

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Cooldown-Based Attacks (Priority: P1)

As a player, I want attacks to have a cooldown so combat feels deliberate and fair.

**Why this priority**: Attack spam undermines competitive integrity. Without cooldowns, combat becomes a button-mashing contest rather than tactical gameplay. This is foundational to all other combat improvements.

**Independent Test**: Spawn two players in melee range. One player rapidly presses attack - only attacks respecting the cooldown should register damage on the target.

**Acceptance Scenarios**:

1. **Given** a player who just attacked, **When** they attempt another attack before cooldown expires (0.5 seconds), **Then** the attack is rejected and no damage occurs
2. **Given** a player who attacked 0.5+ seconds ago, **When** they attack again, **Then** the attack processes normally
3. **Given** the client predicts an attack animation, **When** the server rejects the attack due to cooldown, **Then** the server result is authoritative (no damage dealt)

---

### User Story 2 - Tuned Attack Range (Priority: P1)

As a player, I want hits to land only when targets are clearly in range so combat has clear spatial rules.

**Why this priority**: Range consistency is essential for fair combat. Players must be able to judge when they can hit and when they're safe. This works in parallel with cooldowns as core combat mechanics.

**Independent Test**: Position two players at exactly 1.8 blocks apart (attack range boundary). Attack should succeed. Move target 0.1 blocks further away and attack should fail.

**Acceptance Scenarios**:

1. **Given** an attacker and target within 1.8 blocks, **When** the attacker attacks while facing the target, **Then** the hit registers
2. **Given** an attacker and target at 2.0 blocks apart (beyond range), **When** the attacker attacks, **Then** the attack misses cleanly
3. **Given** both players are moving, **When** an attack is validated, **Then** the server uses post-movement, post-collision positions for range check

---

### User Story 3 - Knockback Feedback (Priority: P2)

As a player, I want hits to push enemies slightly so combat feels impactful and provides visual feedback.

**Why this priority**: Knockback adds game feel and tactical depth (positioning matters). Depends on US1/US2 hit validation being in place first.

**Independent Test**: Hit a stationary target near a wall. Target should be pushed toward the wall but stop at it (no clipping).

**Acceptance Scenarios**:

1. **Given** a valid hit on a target in open space, **When** damage is applied, **Then** the target receives a velocity impulse in the attacker-to-victim direction
2. **Given** a valid hit with knockback pushing toward a wall, **When** collision is processed, **Then** the target stops at the wall surface without penetrating
3. **Given** knockback is applied, **When** the game runs at different frame rates, **Then** the total knockback distance is consistent (frame-rate independent)

---

### User Story 4 - Respawn Invulnerability (Priority: P2)

As a player, I want brief invulnerability after respawn to avoid immediate spawn-killing.

**Why this priority**: Spawn protection is a standard fairness mechanic. Players need time to orient themselves after death. Can be implemented independently of knockback.

**Independent Test**: Kill a player, wait for respawn, immediately attack them during the 2-second invulnerability window - no damage should occur.

**Acceptance Scenarios**:

1. **Given** a player just respawned, **When** they are attacked within 2 seconds of respawn, **Then** no damage is dealt
2. **Given** an invulnerable player, **When** they are hit, **Then** no knockback is applied
3. **Given** a player's invulnerability expires (2+ seconds after respawn), **When** they are attacked, **Then** damage and knockback apply normally

---

### User Story 5 - Latency-Tolerant Hit Registration (Priority: P3)

As a player, I want hits to feel fair even with moderate network latency (30-80ms typical).

**Why this priority**: Network tolerance ensures the previous improvements feel consistent for all players. Lower priority because it's polish on top of working core mechanics.

**Independent Test**: Simulate 50ms latency, have attacker at 1.9 blocks (just outside 1.8 range). With 0.15 block tolerance, the hit should register.

**Acceptance Scenarios**:

1. **Given** an attacker at 1.9 blocks from target (within 1.8 + 0.15 tolerance), **When** attacking with network latency, **Then** the hit registers due to forgiveness radius
2. **Given** an attacker at 2.1 blocks from target (beyond tolerance), **When** attacking, **Then** the hit fails even with forgiveness
3. **Given** two clients with different latencies, **When** both see hits processed, **Then** results are deterministic (server-authoritative)

---

### Edge Cases

- **Attacking during cooldown**: Attack rejected silently, no feedback to attacker beyond animation reset
- **Attacking invulnerable player**: No damage, no knockback, hit counts as "miss" for cooldown purposes
- **High-latency client slightly out of range**: Tolerated within epsilon (0.15 blocks) to reduce frustration
- **Simultaneous attacks**: Processed independently per tick, both may hit if both in range
- **Knockback into wall**: Clamped by existing collision system, target stops at wall surface
- **Knockback while airborne**: Applied normally, gravity still affects vertical velocity
- **Attack during invulnerability window ending**: If attack lands on exact tick invulnerability expires, hit counts

---

## Requirements *(mandatory)*

### Functional Requirements

**Cooldown**
- **FR-001**: Each player MUST have an attack cooldown timer tracked server-side
- **FR-002**: Attack requests received before cooldown expiry MUST be rejected
- **FR-003**: Cooldown duration MUST be configurable (default: 30 ticks / 0.5 seconds at 60Hz)

**Range**
- **FR-004**: Attack range MUST be a fixed scalar value (default: 1.8 blocks)
- **FR-005**: Distance check MUST use server-side post-collision positions
- **FR-006**: Range check MUST include latency tolerance epsilon (default: 0.15 blocks)

**Knockback**
- **FR-007**: Valid hits MUST apply a velocity impulse to the target (default: 4.0 m/s)
- **FR-008**: Knockback MUST respect collision system (no wall penetration)
- **FR-009**: Knockback MUST be ignored if target is invulnerable

**Respawn Protection**
- **FR-010**: Respawned players MUST be invulnerable for a configurable duration (default: 120 ticks / 2 seconds)
- **FR-011**: Invulnerable players MUST NOT take damage
- **FR-012**: Invulnerable players MUST NOT receive knockback

**Hit Registration**
- **FR-013**: Server MUST use last validated player positions for hit checks
- **FR-014**: Distance check MUST include forgiveness radius for latency tolerance
- **FR-015**: Hit logic MUST remain deterministic across all clients

### Key Entities

- **CombatConfig**: Configuration values for attack cooldown, range, knockback strength, and invulnerability duration. Centralized to allow tuning without code changes.

- **ServerPlayer (additions)**:
  - `last_attack_tick`: Tracks when player last attacked for cooldown enforcement
  - `invulnerable_until`: Tick when invulnerability expires (None if vulnerable)

---

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: No attack spam possible - consecutive attack attempts within 0.5 seconds are rejected 100% of the time
- **SC-002**: Combat feels consistent at 30-80ms latency - hits register reliably when visually in range
- **SC-003**: No spawn-kill deaths occur within the 2-second invulnerability window
- **SC-004**: Knockback never causes wall clipping - 100% of knockback scenarios respect collision boundaries
- **SC-005**: All combat tests pass deterministically - identical inputs produce identical outcomes across multiple test runs

---

## Assumptions

- Existing combat system from Feature 003 (combat-visible) provides base attack/damage infrastructure
- Collision system from Feature 008 (movement-polish) handles knockback collision correctly
- Server tick rate is 60Hz as established in previous features
- Players have a single melee attack (no weapon variety per non-goals)

---

## Test Strategy

### Automated Tests

- Cooldown enforcement: Attack rejected during cooldown, accepted after
- Range boundary: Hit at exactly max range, miss just beyond
- Range with epsilon: Hit within tolerance, miss beyond tolerance
- Invulnerability: No damage/knockback during window, normal after expiry
- Knockback collision: Target pushed toward wall stops correctly
- Knockback direction: Impulse direction matches attacker-to-victim vector
- Determinism: Same inputs produce same combat outcomes

### Manual Testing

- Two players attacking simultaneously (both hits should register)
- Respawn under pressure (enemy waiting at spawn point)
- Combat while both players moving and jumping
- Knockback near level geometry (walls, corners, ledges)

---

## Configuration Defaults (MVP)

| Parameter              | Value | Description                       |
|------------------------|-------|-----------------------------------|
| attack_cooldown_ticks  | 30    | 0.5 seconds at 60Hz               |
| attack_range           | 1.8   | Blocks (meters)                   |
| attack_range_epsilon   | 0.15  | Latency forgiveness in blocks     |
| knockback_strength     | 4.0   | Meters per second impulse         |
| respawn_invuln_ticks   | 120   | 2 seconds at 60Hz                 |
