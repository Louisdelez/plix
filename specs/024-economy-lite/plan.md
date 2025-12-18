# Implementation Plan: Economy Lite

**Branch**: `024-economy-lite` | **Date**: 2025-12-17 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/024-economy-lite/spec.md`

## Summary

Implement a minimalist server-authoritative economy system that allows players to earn coins via match events (kills, captures, placements) and spend them in a static shop to purchase items (health packs, weapons, resources). The system integrates with the existing hotbar inventory (Feature 021), uses console commands for interaction (`/balance`, `/shop`, `/buy`), and enforces per-match currency reset with rate limiting on purchases.

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel only per constitution)
**Primary Dependencies**: plix-common (types, inventory, protocol), plix-server (game loop, session, match_state), plix-client (console commands), Hotbar (Feature 021), Crafting (Feature 023)
**Storage**: N/A (in-memory state only - balances reset on match end)
**Testing**: cargo test (unit + integration tests)
**Target Platform**: Linux server + cross-platform client
**Project Type**: Rust workspace with multiple crates
**Performance Goals**: <1ms purchase validation, 100 req/s server-wide, <100ms feedback latency
**Constraints**: Atomic purchases, server-authoritative, no UI required v1, per-match reset
**Scale/Scope**: ~4 shop offers, 5 game modes, ~32 players max

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Security (Server Authority) | PASS | Server is sole authority on balances; client cannot modify coins directly |
| II. Performance | PASS | O(1) balance lookup, O(slots) purchase validation; event-driven earnings |
| III. Architecture (Modularity) | PASS | New `economy/` module follows existing patterns (crafting/, weapons/) |
| IV. Modding | PASS | Config-driven shop offers via TOML; future: mod-defined shops |
| V. Code Quality | PASS | Atomic operations, explicit error handling, mandatory tests |
| VI. Technical Standards | PASS | Stable Rust, clippy/fmt compliant, versioned protocol |
| VII. Player Experience | PASS | Console command MVP; multiplayer-first design |
| VIII. Open Source | PASS | No proprietary dependencies |
| IX. Scoping (MVP) | PASS | 4 offers, console-only, no UI, no persistence, no trading |
| X. Long-Term Vision | PASS | Extensible config for future persistence/UI |

**Gate Result**: PASS - No violations. Proceeding to Phase 0.

## Project Structure

### Documentation (this feature)

```text
specs/024-economy-lite/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (protocol messages)
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
crates/plix-common/src/
├── protocol/
│   └── messages.rs             # BuyRequest, BalanceUpdate, PurchaseResult messages
└── types.rs                    # (existing) PlayerId, ItemId

crates/plix-server/src/
├── economy/                    # NEW MODULE
│   ├── mod.rs                  # Module exports
│   ├── config.rs               # EconomyConfig, get_economy_config(mode)
│   ├── currency.rs             # CurrencyLedger (balances)
│   ├── earnings.rs             # EarningRules, award_for_kill/capture/placement
│   ├── shop.rs                 # ShopOffer, ShopRegistry
│   ├── purchase.rs             # PurchaseSystem (validate + apply atomic)
│   ├── errors.rs               # PurchaseFailReason enum
│   └── metrics.rs              # EconomyMetrics counters
├── session.rs                  # Add wallet field to ServerPlayer
├── match_state.rs              # Reset wallets on match start
└── lib.rs                      # Wire EconomySystem, handle earnings events

crates/plix-server/tests/
├── economy_balance_test.rs     # Unit tests for currency ledger
├── economy_purchase_test.rs    # Unit tests for purchase validation
├── economy_earnings_test.rs    # Unit tests for earning rules
└── economy_integration_test.rs # Integration tests with full server

crates/plix-client/src/
└── (TBD - console command handling)  # /balance, /shop, /buy commands
```

**Structure Decision**: Follows existing `crafting/` module pattern with submodules for separation of concerns. New `economy/` module in plix-server for server-side logic.

## Complexity Tracking

> No violations - table not required.

## Architecture Overview

### Component Diagram

```text
┌─────────────────────────────────────────────────────────────────┐
│                        plix-server                               │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                     economy/                              │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐   │   │
│  │  │ EconomyConfig│  │CurrencyLedger│ │ ShopRegistry    │   │   │
│  │  │ - enabled    │  │ - balances   │ │ - offers[]      │   │   │
│  │  │ - earnings   │  │ - add()      │ │ - get(offer_id) │   │   │
│  │  │ - shop_offers│  │ - spend()    │ │ - list_for_mode │   │   │
│  │  └─────────────┘  │ - reset()    │ └─────────────────┘   │   │
│  │                   └─────────────┘                         │   │
│  │  ┌─────────────┐  ┌────────────────────────────────────┐ │   │
│  │  │EarningRules │  │       PurchaseSystem               │ │   │
│  │  │ - on_kill   │  │ - try_purchase(player, offer_id)   │ │   │
│  │  │ - on_capture│  │   1. validate_economy_enabled      │ │   │
│  │  │ - on_place  │  │   2. validate_offer_exists         │ │   │
│  │  └─────────────┘  │   3. validate_rate_limit           │ │   │
│  │                   │   4. validate_balance              │ │   │
│  │                   │   5. validate_hotbar_space         │ │   │
│  │                   │   6. validate_purchase_limit       │ │   │
│  │                   │   7. atomic_apply                  │ │   │
│  │                   └────────────────────────────────────┘ │   │
│  └──────────────────────────────────────────────────────────┘   │
│                              │                                   │
│                              ▼                                   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                    Server (lib.rs)                        │   │
│  │  - handle_buy_request()                                   │   │
│  │  - on_player_kill() → earnings.award()                   │   │
│  │  - on_ctf_capture() → earnings.award()                   │   │
│  │  - on_match_reset() → ledger.reset_all()                 │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼ Protocol Messages
┌─────────────────────────────────────────────────────────────────┐
│                       plix-common                                │
│  ClientMessage::BuyRequest { offer_id }                         │
│  ClientMessage::BalanceRequest                                  │
│  GameEvent::BalanceUpdate { balance }                           │
│  GameEvent::PurchaseResult { success, offer_id, reason?, ... }  │
└─────────────────────────────────────────────────────────────────┘
```

### Data Flow: Purchase

```text
Client                    Server
  │                         │
  │ BuyRequest{offer_id}    │
  │─────────────────────────>│
  │                         │ PurchaseSystem.try_purchase()
  │                         │   ├─ Check economy enabled
  │                         │   ├─ Lookup offer
  │                         │   ├─ Check rate limit
  │                         │   ├─ Check balance >= price
  │                         │   ├─ Check hotbar.can_add()
  │                         │   ├─ Check purchase limit
  │                         │   └─ If all pass:
  │                         │       ├─ ledger.spend(price)
  │                         │       └─ hotbar.try_add_item()
  │                         │
  │  PurchaseResult{...}    │
  │<─────────────────────────│
  │  BalanceUpdate{balance} │
  │<─────────────────────────│
  │  InventoryUpdate{...}   │
  │<─────────────────────────│
```

### Data Flow: Earning

```text
Server Event (Kill/Capture)
         │
         ▼
  EarningRules.get_reward(event_type, mode)
         │
         ▼
  CurrencyLedger.add(player_id, amount)
         │
         ▼
  Send BalanceUpdate to player
```

## Key Design Decisions

### D1: Per-Match Currency Reset
- **Decision**: Balances reset to 0 at match start
- **Rationale**: Ensures fair play, simplifies implementation (no persistence)
- **Alternatives Rejected**: Cross-match persistence (scope creep, requires DB)

### D2: Static Shop Offers
- **Decision**: Shop offers are fixed per arena config, no dynamic pricing
- **Rationale**: Simple, predictable, server-admin configurable
- **Alternatives Rejected**: Dynamic pricing (complex, balance issues)

### D3: Console Commands Only (v1)
- **Decision**: `/buy`, `/balance`, `/shop` commands, no GUI shop
- **Rationale**: MVP simplicity, follows crafting pattern
- **Alternatives Rejected**: Full shop UI (scope creep, requires UI work)

### D4: Reuse Anti-Cheat Rate Limiting
- **Decision**: Add ActionType::ShopBuy to existing rate limit system
- **Rationale**: Consistent with existing patterns, no new infrastructure
- **Alternatives Rejected**: Custom rate limiter (redundant)

### D5: Atomic Purchase Pattern
- **Decision**: Validate-all-then-apply pattern (same as crafting)
- **Rationale**: Ensures consistency, no partial states
- **Alternatives Rejected**: Non-atomic (race conditions)

## Integration Points

### With Feature 021 (Hotbar/Inventory)
- Use `Hotbar.can_add()` for space validation
- Use `Hotbar.try_add_item()` for item delivery
- Purchased items follow same stacking rules

### With Feature 023 (Crafting)
- Shop can sell SCRAP for crafting recipes
- Economy and crafting are orthogonal (both can be enabled)
- No conflict: one uses coins, other uses SCRAP

### With Game Modes (TDM/FFA/CTF/BR/Training)
- Economy enabled/disabled per mode via config
- Default: enabled for BR Lite/CTF, disabled for TDM/FFA/Training
- Earning rules configurable per mode

### With Match State
- Reset balances on match transition to Playing phase
- Reset purchase counts per player on match start

## Test Strategy

### Unit Tests
1. **CurrencyLedger**: add/spend/reset, overflow protection, no negative balances
2. **PurchaseSystem**: validation failures (balance, hotbar, limit, mode)
3. **EarningRules**: correct rewards per event type and mode
4. **ShopRegistry**: offer lookup, mode filtering, validation at load

### Integration Tests
1. **Full Purchase Flow**: Kill → Earn → Buy → Receive item
2. **CTF Capture Reward**: Flag capture → Team reward → Shop access
3. **Mode Restrictions**: Economy disabled in TDM → Buy fails
4. **Match Reset**: End match → New match → Balance is 0
5. **Non-Regression**: Existing hotbar/crafting/combat unchanged

## Default Configuration

```toml
[economy]
enabled = true  # Override per mode below

[economy.earnings]
kill_reward = 10
ctf_capture_reward = 25
br_placement_rewards = [50, 30, 15]  # 1st, 2nd, 3rd

[economy.mode_overrides]
Training = { enabled = false }
Tdm = { enabled = false }
Ffa = { enabled = false }
Ctf = { enabled = true }
BrLite = { enabled = true }

[[economy.shop_offers]]
offer_id = "health_pack"
item_id = "HEALTH_PACK"
quantity = 1
price = 20

[[economy.shop_offers]]
offer_id = "sword"
item_id = "SWORD"
quantity = 1
price = 50

[[economy.shop_offers]]
offer_id = "bow"
item_id = "BOW"
quantity = 1
price = 75

[[economy.shop_offers]]
offer_id = "scrap"
item_id = "SCRAP"
quantity = 5
price = 10
```
