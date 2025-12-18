# Research: Economy Lite

**Feature**: 024-economy-lite
**Date**: 2025-12-17
**Status**: Complete

## Overview

This document captures research findings and design decisions for the Economy Lite feature. All NEEDS CLARIFICATION items from the technical context have been resolved.

---

## R1: Existing Rate Limiting Integration

### Question
How does the existing anti-cheat rate limiting system work, and can it be extended for shop purchases?

### Research Findings
Examined `crates/plix-server/src/anti_cheat/` module:

1. **ActionType enum** exists in the anti-cheat module with variants for different player actions
2. **Per-player rate tracking** uses `HashMap<ActionType, LastActionTime>`
3. **Configuration** allows different rate limits per action type
4. **Pattern**: `check_rate_limit(ActionType, current_tick, config) -> bool`

### Decision
**Extend existing ActionType enum** with `ShopBuy` variant.

### Rationale
- Consistent with existing patterns
- No new infrastructure needed
- Reuses battle-tested rate limiting logic
- Configuration-driven limits

### Alternatives Considered
- Custom rate limiter for economy: Rejected (redundant, inconsistent)
- No rate limiting: Rejected (abuse vector)

---

## R2: Hotbar Integration for Purchased Items

### Question
What Hotbar methods are available for validating and adding purchased items?

### Research Findings
Examined `crates/plix-common/src/inventory/hotbar.rs` (Feature 021):

1. **`can_add(item_id, quantity, max_stack) -> bool`**: Validates if item can fit
2. **`try_add_item(item_id, quantity, max_stack) -> u8`**: Adds item, returns overflow
3. **`count_item(item_id) -> u16`**: Counts existing items
4. **Stack limits**: Defined per item in `item_registry.rs`

### Decision
**Use existing `can_add()` + `try_add_item()` pattern** (same as crafting).

### Rationale
- Proven pattern from Crafting Lite (Feature 023)
- Handles stacking automatically
- Consistent item delivery mechanism

### Alternatives Considered
- Direct slot manipulation: Rejected (bypasses stacking logic)
- Custom purchase delivery: Rejected (duplicates existing logic)

---

## R3: Match State Integration for Reset

### Question
How does the match state machine handle phase transitions, and where should balance reset occur?

### Research Findings
Examined `crates/plix-server/src/match_state.rs`:

1. **MatchStateMachine** tracks current phase (Warmup, Playing, PostMatch)
2. **Phase transitions** are explicit via `transition_to()` method
3. **Existing reset patterns**: Hotbar loadouts reset on respawn/match start
4. **Event hooks**: Match phase changes can trigger callbacks

### Decision
**Reset balances on transition to Playing phase** (match start).

### Rationale
- Consistent with match lifecycle
- Ensures all players start equal
- Purchase counts reset alongside balance

### Alternatives Considered
- Reset on player join: Rejected (doesn't handle reconnects correctly)
- Reset on warmup: Rejected (may want warmup purchases in future)
- Persistent balances: Out of scope (v1 is per-match)

---

## R4: Protocol Message Patterns

### Question
What protocol patterns exist for client-server communication in similar features?

### Research Findings
Examined `crates/plix-common/src/protocol/messages.rs`:

1. **ClientMessage variants**: Request patterns (e.g., `CraftRequest`, `BlockEditRequest`)
2. **GameEvent variants**: Result patterns (e.g., `CraftResult`, `InventoryUpdate`)
3. **Serialization**: All messages use bincode via derive macros
4. **Rejection reasons**: Enums for failure cases (e.g., `CraftRejectReason`)

### Decision
**Follow existing request/result pattern**:
- `ClientMessage::BuyRequest { offer_id: String }`
- `ClientMessage::BalanceRequest`
- `GameEvent::BalanceUpdate { balance: u32 }`
- `GameEvent::PurchaseResult { success, offer_id, output_item?, fail_reason? }`
- `PurchaseRejectReason` enum for failure cases

### Rationale
- Consistent with existing protocol design
- Leverages existing serialization infrastructure
- Matches client expectation patterns

### Alternatives Considered
- Generic Command/Response: Rejected (too abstract, loses type safety)
- Polling-based balance: Rejected (inefficient)

---

## R5: Earning Event Integration Points

### Question
Where in the server code should earning events be triggered?

### Research Findings
Examined kill/capture handling in `crates/plix-server/src/lib.rs`:

1. **Kill handling**: `handle_damage()` → `apply_death()` flow
2. **CTF captures**: `CtfCoordinator.process_capture()` emits events
3. **BR placements**: `BrLiteCoordinator` tracks eliminations and placements
4. **Event emission**: Server emits `GameEvent` to relevant players

### Decision
**Hook into existing event handlers**:
- Add `award_kill_coins()` call in death handling
- Add `award_capture_coins()` call in CTF coordinator
- Add `award_placement_coins()` call in BR Lite match end

### Rationale
- Minimal intrusion into existing code
- Events are already well-defined
- Single point of currency award per event type

### Alternatives Considered
- Event bus pattern: Rejected (over-engineering for v1)
- Observer pattern: Rejected (adds complexity without benefit)

---

## R6: Configuration Loading Pattern

### Question
How should economy configuration be loaded and applied?

### Research Findings
Examined arena loading in `crates/plix-arena/`:

1. **Arena TOML**: Already supports metadata, spawn points, game mode
2. **ServerConfig**: Runtime configuration for server
3. **Pattern**: TOML → strongly-typed structs via serde

### Decision
**Add [economy] section to arena TOML**:
- `economy_enabled`: bool (default based on mode)
- `earnings`: kill_reward, capture_reward, placement_rewards
- `shop_offers`: array of offer definitions

### Rationale
- Consistent with existing arena configuration
- Server admins already edit arena TOML
- Serde handles parsing automatically

### Alternatives Considered
- Separate economy.toml: Rejected (fragmented config)
- Hardcoded values: Rejected (no customization)
- JSON config: Rejected (inconsistent with project style)

---

## R7: Wallet Storage Location

### Question
Where should per-player wallet (balance + purchase counts) be stored?

### Research Findings
Examined `crates/plix-server/src/session.rs`:

1. **ServerPlayer struct**: Per-player server-side state
2. **Existing fields**: hotbar, health, position, craft_cooldown
3. **Pattern**: Transient per-match state stored on player struct

### Decision
**Add `wallet: PlayerWallet` field to ServerPlayer**.

### Rationale
- Consistent with existing per-player state (craft_cooldown, hotbar)
- Natural reset on disconnect/reconnect
- Easy access during purchase validation

### Alternatives Considered
- Separate HashMap in Server: Rejected (fragmented state)
- EconomyManager with player lookup: Rejected (over-engineering)

---

## Summary of Resolved Items

| Item | Resolution |
|------|------------|
| Rate limiting | Extend existing ActionType enum |
| Hotbar integration | Use can_add() + try_add_item() |
| Match reset | Transition to Playing phase |
| Protocol messages | Follow request/result pattern |
| Earning hooks | Hook into existing kill/capture handlers |
| Configuration | Add [economy] section to arena TOML |
| Wallet storage | Add field to ServerPlayer |

---

## Outstanding Questions

None - all research items resolved.

## Next Steps

Proceed to Phase 1: Design & Contracts
- Create data-model.md with entity definitions
- Generate protocol contracts in contracts/
- Create quickstart.md with implementation guide
