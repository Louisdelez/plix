# Feature Specification: Economy Lite

**Feature Branch**: `024-economy-lite`
**Created**: 2025-12-17
**Status**: Draft
**Input**: Server-authoritative currency and shop system for match-based progression

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Earn Currency from Match Events (Priority: P1)

As a player, I want to earn coins by performing in-match actions (kills, objectives, survival) so that I can spend them to purchase useful items.

**Why this priority**: Without earning currency, the entire economy system has no value. This is the foundation that enables all other features.

**Independent Test**: Player joins a match, gets a kill, and immediately sees their coin balance increase. Can be tested without shops.

**Acceptance Scenarios**:

1. **Given** a player in an active match with 0 coins, **When** they eliminate an enemy player, **Then** they receive the configured coin reward (default: 10 coins) and their balance updates.

2. **Given** a player in CTF mode, **When** their team captures the enemy flag, **Then** they receive the flag capture reward (default: 25 coins).

3. **Given** a player in BR Lite mode who survives to top 3, **When** the match ends, **Then** they receive a survival bonus based on placement (1st: 50, 2nd: 30, 3rd: 15 coins).

4. **Given** a player in Training mode, **When** they perform any action, **Then** they receive no coins (economy disabled by default).

---

### User Story 2 - Purchase Items from Shop (Priority: P1)

As a player, I want to spend my earned coins to buy items (weapons, consumables, resources) so that I can gain an advantage in the match.

**Why this priority**: Purchasing is the core spending mechanism. Without it, earned currency has no purpose.

**Independent Test**: Player with sufficient coins executes `/buy health_pack` and receives the item in their hotbar while their balance decreases.

**Acceptance Scenarios**:

1. **Given** a player with 50 coins and an offer "health_pack" costing 20 coins, **When** they execute `/buy health_pack`, **Then** they receive 1 Health Pack in their hotbar and their balance becomes 30 coins.

2. **Given** a player with 10 coins and an offer costing 20 coins, **When** they attempt to purchase, **Then** the purchase fails with "Insufficient funds" and their balance remains unchanged.

3. **Given** a player with a full hotbar, **When** they attempt to purchase any item, **Then** the purchase fails with "Inventory full" and their balance remains unchanged.

4. **Given** a player in TDM mode with economy disabled, **When** they attempt to purchase, **Then** the purchase fails with "Shop not available in this mode".

---

### User Story 3 - View Balance and Shop Offers (Priority: P2)

As a player, I want to check my current coin balance and see available shop offers so that I can make informed purchasing decisions.

**Why this priority**: Essential for usability but not required for the core earn/spend loop to function.

**Independent Test**: Player executes `/balance` and sees their current coins; `/shop` lists available offers with prices.

**Acceptance Scenarios**:

1. **Given** a player with 75 coins, **When** they execute `/balance`, **Then** they see "Balance: 75 coins".

2. **Given** a player in a mode with economy enabled, **When** they execute `/shop`, **Then** they see a list of available offers with item names, quantities, and prices.

3. **Given** a player who just earned coins from a kill, **When** they execute `/balance`, **Then** they see the updated balance including the recent earnings.

---

### User Story 4 - Server Admin Configures Economy (Priority: P2)

As a server admin, I want to configure earning rules and shop offers per arena so that I can customize the economy for my server's playstyle.

**Why this priority**: Allows server customization but the system works with defaults without configuration.

**Independent Test**: Admin modifies arena TOML with custom prices, restarts server, and players see the new prices.

**Acceptance Scenarios**:

1. **Given** an arena TOML with `kill_reward = 20`, **When** a player gets a kill, **Then** they receive 20 coins instead of the default 10.

2. **Given** a shop offer configured with `max_per_match = 2`, **When** a player attempts a third purchase of that offer, **Then** the purchase fails with "Purchase limit reached".

3. **Given** an arena with `economy_enabled = false`, **When** players perform any action, **Then** no coins are earned and shop commands return "Economy disabled".

---

### User Story 5 - Match Reset Economy (Priority: P3)

As a player, I want my coin balance to reset at the start of each match so that every match starts fair and fresh.

**Why this priority**: Important for competitive integrity but can be the default behavior without explicit handling.

**Independent Test**: Player ends match with 100 coins, joins new match, and starts with 0 coins.

**Acceptance Scenarios**:

1. **Given** a player who ended the previous match with 100 coins, **When** they join a new match, **Then** their balance is reset to 0 coins.

2. **Given** a player who purchased items in the previous match, **When** they join a new match, **Then** their purchase counts are reset (no more "limit reached").

---

### Edge Cases

- What happens when a player disconnects mid-match? Balance and purchase counts are lost (no persistence v1).
- What happens when a player tries to buy during match countdown/warmup? Purchase allowed if economy is enabled for that phase (default: enabled during Playing phase only).
- What happens when kill reward would overflow u32? Cap at u32::MAX (extremely unlikely scenario).
- What happens when two players kill each other simultaneously? Both receive kill rewards.
- What happens when a player is eliminated and respawns? They keep their earned coins.
- What happens if shop configuration is invalid (negative price, unknown item)? Server logs error and disables that offer.

## Requirements *(mandatory)*

### Functional Requirements

#### Currency System

- **FR-001**: System MUST track a per-player coin balance as an unsigned 32-bit integer.
- **FR-002**: Server MUST be the sole authority on coin balances (clients cannot modify directly).
- **FR-003**: System MUST reset all player balances to 0 at match start.
- **FR-004**: System MUST send balance updates to clients when their balance changes.

#### Earning System

- **FR-005**: System MUST award coins for player kills based on configurable `kill_reward` (default: 10).
- **FR-006**: System MUST award coins for CTF flag captures based on configurable `capture_reward` (default: 25).
- **FR-007**: System MUST award coins for BR Lite placement based on configurable placement rewards.
- **FR-008**: System MUST support enabling/disabling earnings per game mode.
- **FR-009**: Earnings MUST be disabled by default for Training mode.

#### Shop System

- **FR-010**: System MUST support static shop offers defined in arena configuration.
- **FR-011**: Each shop offer MUST have: offer_id, item_id, quantity, price.
- **FR-012**: Shop offers MAY have optional restrictions: allowed_modes, max_per_match.
- **FR-013**: System MUST validate offer availability for current game mode before purchase.

#### Purchase System

- **FR-014**: System MUST validate sufficient balance before purchase.
- **FR-015**: System MUST validate hotbar space before purchase (using existing Hotbar.can_add).
- **FR-016**: System MUST validate purchase limit (max_per_match) before purchase.
- **FR-017**: Purchases MUST be atomic: debit coins and add item together, or nothing.
- **FR-018**: System MUST return detailed failure reasons on purchase rejection.
- **FR-019**: System MUST apply rate limiting to buy requests (reuse existing ActionType system).

#### Commands

- **FR-020**: System MUST support `/buy <offer_id>` command to purchase items.
- **FR-021**: System MUST support `/balance` command to view current coins.
- **FR-022**: System MUST support `/shop` command to list available offers.

#### Protocol

- **FR-023**: System MUST send BalanceUpdate events when player balance changes.
- **FR-024**: System MUST send PurchaseResult events with success/failure and reason.

#### Configuration

- **FR-025**: System MUST support `economy_enabled` toggle per arena (default: true for BR Lite/CTF, false for TDM/FFA/Training).
- **FR-026**: System MUST support configurable earning rules per mode.
- **FR-027**: System MUST support shop offers defined in arena TOML.

#### Observability

- **FR-028**: System MUST log successful purchases with player_id, offer_id, price at info level.
- **FR-029**: System MUST log failed purchases with player_id, offer_id, reason at debug level.
- **FR-030**: System MUST track metrics: coins_earned_total, purchases_total, purchases_failed_by_reason.

### Key Entities

- **PlayerWallet**: Per-player coin balance and purchase history for current match. Attributes: player_id, balance (u32), purchases_this_match (HashMap<offer_id, count>).

- **ShopOffer**: A purchasable item configuration. Attributes: offer_id (String), item_id (ItemId), quantity (u8), price (u32), allowed_modes (optional Vec<GameMode>), max_per_match (optional u8).

- **EarningRule**: Configuration for coin awards. Attributes: event_type (Kill/Capture/Placement), reward (u32), enabled_modes (Vec<GameMode>).

- **EconomyConfig**: Per-arena economy settings. Attributes: enabled (bool), earning_rules, shop_offers.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Players can earn and spend coins within a single match session with 100% consistency (no desync between server and client balance).

- **SC-002**: Purchase validation completes in under 1ms (O(1) lookup and validation).

- **SC-003**: System handles 100 purchase requests per second server-wide without performance degradation.

- **SC-004**: All purchase attempts result in clear success or failure feedback within 100ms of request.

- **SC-005**: Server admins can customize all earning rates and shop prices via arena configuration without code changes.

- **SC-006**: Economy system adds less than 1% overhead to server tick processing when no transactions occur.

- **SC-007**: 100% of test scenarios pass, covering earning, purchasing, validation, and edge cases.

## Assumptions

- Players are identified by PlayerId which is already established per session.
- The existing Hotbar system (Feature 021) provides can_add() and try_add_item() methods for inventory validation.
- The existing anti-cheat rate limiting system can be extended with a new ActionType for buy requests.
- Game mode is accessible from MatchStateMachine.
- Existing protocol serialization (bincode) will be used for new message types.
- Default shop offers will include: health_pack (20 coins), sword (50 coins), bow (75 coins), scrap (10 coins).

## Out of Scope

- Player-to-player trading or marketplace.
- Dynamic pricing based on supply/demand.
- Persistent currency across matches (v1 resets per match).
- Visual shop UI (v1 uses console commands only).
- Advanced anti-fraud beyond rate limiting.
- Currency conversion or multiple currency types.
