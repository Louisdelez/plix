# Feature Specification: Weapons & Items v1

**Feature Branch**: `022-weapons-items-v1`
**Created**: 2025-12-17
**Status**: Draft
**Input**: Weapons system with melee (sword) and ranged (bow) weapons, projectiles, cooldowns, accuracy, recoil

## Clarifications

### Session 2025-12-17

- Q: What damage should the bow deal per arrow hit? → A: 15 damage
- Q: What is the maximum concurrent projectile count per server? → A: 128 projectiles
- Q: When projectile limit (128) is reached, what should happen? → A: Reject new projectiles (cooldown triggers but no arrow spawns)
- Q: What cooldown should the bow have between shots? → A: 0.8s
- Q: What melee cone angle and radius should the sword use? → A: 60° angle, 2.5 blocks radius

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Melee Combat with Sword (Priority: P1)

A player equips a sword in their hotbar and uses it to attack nearby enemies. The sword deals 25 damage with a 0.6s cooldown between swings. Hit detection uses a cone/radius check in front of the player.

**Why this priority**: Core combat mechanic that provides immediate gameplay value. Builds on existing hotbar system from Feature 021.

**Independent Test**: Player can select sword from hotbar, swing at enemy, deal damage. Combat is functional without ranged weapons.

**Acceptance Scenarios**:

1. **Given** a player has a sword in active hotbar slot, **When** they use the item, **Then** enemies within melee range (cone in front) take 25 damage
2. **Given** a player just swung their sword, **When** they try to swing again within 0.6s, **Then** the attack is rejected (cooldown enforced)
3. **Given** a player swings at empty space, **When** no entity is in the hit cone, **Then** no damage is dealt but cooldown still applies
4. **Given** multiple enemies in melee cone, **When** player swings, **Then** all enemies in cone take damage

---

### User Story 2 - Ranged Combat with Bow (Priority: P1)

A player equips a bow and fires arrow projectiles. Arrows travel with velocity, have a lifetime, and deal damage on impact with players, bots, or blocks.

**Why this priority**: Ranged combat is essential for tactical gameplay. Provides attack option beyond melee range.

**Independent Test**: Player can select bow, fire arrow, arrow travels and damages target on impact.

**Acceptance Scenarios**:

1. **Given** a player has a bow in active hotbar slot, **When** they use the item, **Then** an arrow projectile spawns and travels in the aim direction
2. **Given** an arrow is in flight, **When** it collides with a player/bot, **Then** the target takes bow damage and the arrow despawns
3. **Given** an arrow is in flight, **When** it collides with a block, **Then** the arrow despawns (impact event)
4. **Given** an arrow is in flight, **When** its lifetime expires, **Then** the arrow despawns

---

### User Story 3 - Weapon Cooldowns (Priority: P1)

Each weapon has its own cooldown that prevents spam attacks. The server enforces cooldowns and rejects attacks that come too quickly.

**Why this priority**: Prevents exploit abuse and ensures balanced combat pacing.

**Independent Test**: Attack rate is limited per weapon type, server rejects rapid attacks.

**Acceptance Scenarios**:

1. **Given** a sword with 0.6s cooldown, **When** player attacks, **Then** next attack is blocked until 0.6s passes
2. **Given** a bow with its cooldown, **When** player fires, **Then** next shot is blocked until cooldown passes
3. **Given** player switches from sword to bow, **When** they immediately attack, **Then** the attack succeeds (cooldown is per-weapon, not carried over)
4. **Given** client sends attack faster than cooldown, **When** server receives it, **Then** server rejects with "cooldown not ready"

---

### User Story 4 - Accuracy and Movement Spread (Priority: P2)

Ranged weapons have a base accuracy with spread. Moving while firing increases spread (movement penalty).

**Why this priority**: Adds skill depth to ranged combat. Can be tested independently with bow.

**Independent Test**: Arrows have spread from aim direction, spread increases when player is moving.

**Acceptance Scenarios**:

1. **Given** a stationary player fires a bow, **When** arrow spawns, **Then** direction has random spread within base spread angle
2. **Given** a moving player fires a bow, **When** arrow spawns, **Then** spread is increased by movement penalty
3. **Given** spread angle of ±2 degrees, **When** multiple shots fired, **Then** arrows form a cone pattern (server-calculated)

---

### User Story 5 - Recoil System (Priority: P2)

Rapid firing accumulates spread penalty (recoil). After a recovery window without firing, spread resets.

**Why this priority**: Rewards paced shooting over spam. Adds tactical depth.

**Independent Test**: Rapid shots have increasing spread, waiting resets accuracy.

**Acceptance Scenarios**:

1. **Given** player fires first shot, **When** they fire second shot quickly, **Then** second shot has increased spread (cumulative)
2. **Given** player has accumulated recoil spread, **When** they wait beyond recovery window, **Then** spread resets to base
3. **Given** recoil spread, **When** it reaches maximum cap, **Then** it does not increase further

---

### User Story 6 - Hotbar Integration (Priority: P1)

Weapons are items in the hotbar. UseActiveItem action determines which weapon is used. Empty slot uses default melee (fist punch, lower damage).

**Why this priority**: Essential integration with existing inventory system from Feature 021.

**Independent Test**: Selecting different hotbar slots changes active weapon, each weapon type behaves correctly.

**Acceptance Scenarios**:

1. **Given** sword in slot 0 and bow in slot 1, **When** player selects slot 0 and attacks, **Then** melee sword attack occurs
2. **Given** sword in slot 0 and bow in slot 1, **When** player selects slot 1 and attacks, **Then** arrow fires
3. **Given** empty active slot, **When** player attacks, **Then** default melee (fist) occurs with lower damage
4. **Given** non-weapon item in slot (e.g., health pack), **When** player uses, **Then** item-specific action (not weapon attack)

---

### User Story 7 - Game Mode Compatibility (Priority: P2)

Weapons work in all game modes: Training, TDM, FFA, CTF, BR Lite. Damage applies according to mode rules (friendly fire, etc.).

**Why this priority**: Ensures weapons integrate with existing game modes.

**Independent Test**: Weapon attacks function correctly in each mode with proper damage application.

**Acceptance Scenarios**:

1. **Given** Training mode, **When** player attacks bot, **Then** bot takes weapon damage
2. **Given** TDM mode, **When** player attacks enemy team member, **Then** enemy takes damage
3. **Given** TDM mode with no friendly fire, **When** player attacks teammate, **Then** no damage dealt
4. **Given** BR Lite mode, **When** player attacks another player, **Then** target takes damage

---

### User Story 8 - Projectile Replication (Priority: P2)

Projectiles are replicated via spawn/despawn/impact events. No per-tick position updates for network efficiency. Client interpolates position from spawn data.

**Why this priority**: Network efficiency is crucial for multiplayer performance.

**Independent Test**: Clients receive projectile events, render projectiles smoothly, see impacts.

**Acceptance Scenarios**:

1. **Given** server spawns projectile, **When** spawn event sent, **Then** clients receive spawn with position, velocity, direction
2. **Given** projectile impacts target, **When** impact event sent, **Then** clients receive impact position and despawn projectile
3. **Given** projectile in flight, **When** tick passes, **Then** no position update sent (client interpolates)
4. **Given** projectile lifetime expires, **When** despawn event sent, **Then** clients remove projectile

---

### Edge Cases

- What happens when player fires bow with no arrows (if ammo system exists)? Currently no ammo system, bow fires freely.
- What happens when projectile count exceeds server limit? New projectiles are rejected (cooldown triggers but no arrow spawns); existing projectiles are never despawned early.
- What happens when player disconnects with projectiles in flight? Projectiles continue, owned by server.
- What happens when melee attack hits both enemy and friendly? Damage applies per mode rules (friendly fire setting).
- What happens when projectile spawns inside a block? Immediate impact and despawn.
- What happens when player switches weapons mid-cooldown? Each weapon tracks its own cooldown state.

## Requirements *(mandatory)*

### Functional Requirements

**Weapon System (Core)**

- **FR-001**: System MUST support weapon definitions with: id, name, damage, cooldown, range, weapon_type (melee/ranged)
- **FR-002**: System MUST distinguish between melee weapons (instant hit) and ranged weapons (projectile)
- **FR-003**: Sword MUST deal 25 damage with 0.6s cooldown
- **FR-004**: Melee hit detection MUST use cone check (60° angle, 2.5 blocks radius) in front of player
- **FR-005**: Bow MUST fire arrow projectiles when used, dealing 15 damage per hit with 0.8s cooldown
- **FR-006**: Default melee (fist/empty slot) MUST deal lower damage than sword (e.g., 10 damage)

**Projectile System**

- **FR-010**: Projectiles MUST be server-side entities with position, velocity, direction, lifetime
- **FR-011**: Projectiles MUST move according to velocity each server tick
- **FR-012**: Projectiles MUST check collision with players, bots, and blocks
- **FR-013**: On player/bot collision, projectile MUST deal damage and despawn
- **FR-014**: On block collision, projectile MUST despawn (impact event)
- **FR-015**: On lifetime expiry, projectile MUST despawn
- **FR-016**: Server MUST enforce maximum projectile count of 128 for performance
- **FR-017**: Projectile spawns MUST include owner player ID for damage attribution

**Cooldown System**

- **FR-020**: Each weapon type MUST have a defined cooldown duration
- **FR-021**: Server MUST track cooldown state per player per weapon type
- **FR-022**: Server MUST reject attacks before cooldown expires
- **FR-023**: Cooldown MUST NOT carry over when switching weapons
- **FR-024**: Cooldown timer MUST start when attack is executed (not when started)

**Accuracy System**

- **FR-030**: Ranged weapons MUST have a base spread angle (e.g., ±2 degrees)
- **FR-031**: Server MUST calculate final shot direction with random spread applied
- **FR-032**: Movement MUST increase spread by movement penalty factor
- **FR-033**: Spread calculation MUST be server-authoritative (client sends aim, server adds spread)

**Recoil System**

- **FR-040**: Rapid firing MUST accumulate spread penalty (recoil)
- **FR-041**: Recoil spread MUST be cumulative across rapid shots
- **FR-042**: After recovery window without firing, recoil MUST reset
- **FR-043**: Recoil spread MUST have a maximum cap
- **FR-044**: Recoil state MUST be per player per weapon type

**Hotbar Integration**

- **FR-050**: UseActiveItem action MUST determine weapon used based on active slot
- **FR-051**: Weapon attacks MUST use the item in active hotbar slot
- **FR-052**: Empty slot MUST trigger default melee attack
- **FR-053**: Non-weapon items MUST NOT trigger weapon attacks (use item-specific action)

**Game Mode Compatibility**

- **FR-060**: Weapon damage MUST respect game mode friendly fire rules
- **FR-061**: Weapon attacks MUST work in Training, TDM, FFA, CTF, BR Lite modes
- **FR-062**: Kill attribution MUST use projectile owner for ranged kills

**Replication**

- **FR-070**: Server MUST send ProjectileSpawn event with: id, position, velocity, direction, owner
- **FR-071**: Server MUST send ProjectileImpact event with: id, position, impact_type
- **FR-072**: Server MUST send ProjectileDespawn event when lifetime expires
- **FR-073**: Server MUST NOT send per-tick projectile position updates
- **FR-074**: Clients MUST interpolate projectile positions from spawn data

**Anti-Cheat**

- **FR-080**: Server MUST validate all weapon attacks (cooldown, range, ownership)
- **FR-081**: Server MUST reject attacks from dead players
- **FR-082**: Server MUST validate projectile spawn requests match weapon type

### Key Entities

- **WeaponDef**: Weapon definition with id, name, damage, cooldown_ms, range, weapon_type (Melee/Ranged), base_spread_deg, recoil_per_shot, recoil_recovery_ms, recoil_max
- **Projectile**: Server-side entity with id, owner_player_id, position, velocity, direction, spawn_tick, lifetime_ticks, damage
- **WeaponCooldownState**: Per-player tracking of last_attack_tick per weapon type
- **RecoilState**: Per-player tracking of current_spread, last_shot_tick per weapon type
- **ProjectileSpawn** (event): Sent to clients when projectile created
- **ProjectileImpact** (event): Sent to clients when projectile hits something
- **ProjectileDespawn** (event): Sent to clients when projectile removed

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Sword attack deals exactly 25 damage when hitting a target
- **SC-002**: Sword attacks are limited to once per 0.6s (server enforced)
- **SC-003**: Bow fires projectile that travels and damages target on impact
- **SC-004**: Projectile lifetime prevents indefinite flight (despawn after timeout)
- **SC-005**: Weapon switching does not inherit previous weapon's cooldown
- **SC-006**: Moving while shooting increases projectile spread visibly
- **SC-007**: Rapid firing accumulates spread, waiting resets it
- **SC-008**: All game modes support weapon combat appropriately

## Assumptions

- Arrow/bow damage: 15 damage per hit
- Bow cooldown: 0.8s
- No ammo system in v1 - bow fires freely
- Melee cone: 60° angle, 2.5 blocks radius
- Maximum projectile count per server: 128
- Projectile lifetime will be defined (suggest 3-5 seconds / ~180-300 ticks at 60Hz)
- Base spread for bow: ±2 degrees
- Movement penalty: +50% spread when moving
- Recoil per shot: +1 degree spread
- Recoil recovery window: 0.5s
- Recoil maximum cap: +5 degrees
