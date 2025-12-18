# Tasks: Economy Lite

**Input**: Design documents from `/specs/024-economy-lite/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Included per specification requirements (test coverage mandated in spec.md).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **plix-common**: `crates/plix-common/src/` - Shared types, protocol messages
- **plix-server**: `crates/plix-server/src/` - Server-side economy logic
- **plix-client**: `crates/plix-client/src/` - Console commands
- **plix-arena**: `crates/plix-arena/src/` - Arena configuration loading
- **Tests**: `crates/plix-server/tests/` - Integration tests

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create economy module structure and shared types

- [ ] T001 Create economy module directory structure at `crates/plix-server/src/economy/mod.rs`
- [ ] T002 [P] Create economy types module at `crates/plix-common/src/economy/mod.rs`
- [ ] T003 [P] Export economy module from `crates/plix-common/src/lib.rs`
- [ ] T004 [P] Export economy module from `crates/plix-server/src/lib.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**CRITICAL**: No user story work can begin until this phase is complete

### Core Types (plix-common)

- [ ] T005 [P] Define PurchaseRejectReason enum (EconomyDisabled, UnknownOffer, ModeRestricted, InsufficientBalance, HotbarFull, PurchaseLimitReached, RateLimited, PlayerDead) at `crates/plix-common/src/economy/types.rs`
- [ ] T006 [P] Add ClientMessage variants (BuyRequest, BalanceRequest, ShopListRequest) at `crates/plix-common/src/protocol/messages.rs`
- [ ] T007 [P] Add GameEvent variants (BalanceUpdate, PurchaseResult, ShopList) at `crates/plix-common/src/protocol/messages.rs`
- [ ] T008 Add serde tests for new protocol messages at `crates/plix-common/src/protocol/messages.rs`

### Server Core Types

- [ ] T009 [P] Create EconomyConfig struct (enabled, kill_reward, ctf_capture_reward, br_placement_rewards, shop_offers) at `crates/plix-server/src/economy/config.rs`
- [ ] T010 [P] Implement get_economy_config(mode, arena_config) with mode-specific defaults at `crates/plix-server/src/economy/config.rs`
- [ ] T011 [P] Add EconomyConfig validation (price > 0, quantity > 0, rewards >= 0) at `crates/plix-server/src/economy/config.rs`
- [ ] T012 Create PlayerWallet struct (balance, purchases HashMap) with new/get_balance/add_coins/try_spend/get_purchase_count/record_purchase/reset methods at `crates/plix-server/src/economy/wallet.rs`
- [ ] T013 [P] Create ShopOffer struct (offer_id, item_id, quantity, price, allowed_modes, max_per_match) at `crates/plix-server/src/economy/shop.rs`
- [ ] T014 Create ShopRegistry struct with new/get/list_for_mode/is_empty methods at `crates/plix-server/src/economy/shop.rs`
- [ ] T015 Add wallet field to ServerPlayer struct at `crates/plix-server/src/session.rs`

### Foundational Tests

- [ ] T016 [P] Unit tests for PlayerWallet (add/spend/reset, overflow/underflow safety) at `crates/plix-server/tests/economy_balance_test.rs`
- [ ] T017 [P] Unit tests for ShopRegistry (unique offers, lookup existing/nonexistent) at `crates/plix-server/tests/economy_balance_test.rs`
- [ ] T018 [P] Unit tests for EconomyConfig validation (valid/invalid configs) at `crates/plix-server/tests/economy_balance_test.rs`

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Earn Currency from Match Events (Priority: P1)

**Goal**: Players earn coins by performing in-match actions (kills, objectives, survival)

**Independent Test**: Player joins match, gets kill, immediately sees balance increase. Can test without shops.

### Implementation for User Story 1

- [ ] T019 Create EarningEvent enum (Kill, CtfCapture, BrPlacement) at `crates/plix-server/src/economy/earnings.rs`
- [ ] T020 Implement award_coins(event, wallet, config) returning Option<u32> at `crates/plix-server/src/economy/earnings.rs`
- [ ] T021 [US1] Add kill reward hook in death handling at `crates/plix-server/src/lib.rs` (call award_coins, send BalanceUpdate)
- [ ] T022 [US1] Add CTF capture reward hook in CtfCoordinator at `crates/plix-server/src/ctf/mod.rs`
- [ ] T023 [US1] Add BR Lite placement rewards hook (1st/2nd/3rd) in BrLiteCoordinator at `crates/plix-server/src/br_lite/mod.rs`
- [ ] T024 [US1] Handle BalanceRequest message (return BalanceUpdate) at `crates/plix-server/src/lib.rs`
- [ ] T025 [US1] Verify Training mode returns no rewards (economy disabled by default) at `crates/plix-server/src/economy/config.rs`

### Tests for User Story 1

- [ ] T026 [P] [US1] Integration test: kill -> balance increases at `crates/plix-server/tests/economy_earnings_test.rs`
- [ ] T027 [P] [US1] Integration test: CTF capture -> balance increases at `crates/plix-server/tests/economy_earnings_test.rs`
- [ ] T028 [P] [US1] Integration test: BR Lite placement (1st/2nd/3rd) -> correct rewards at `crates/plix-server/tests/economy_earnings_test.rs`
- [ ] T029 [P] [US1] Integration test: Training mode -> no coins awarded at `crates/plix-server/tests/economy_earnings_test.rs`

**Checkpoint**: User Story 1 complete - players can earn coins from match events

---

## Phase 4: User Story 2 - Purchase Items from Shop (Priority: P1)

**Goal**: Players spend earned coins to buy items that go into their hotbar

**Independent Test**: Player with sufficient coins executes /buy health_pack and receives item in hotbar while balance decreases

### Implementation for User Story 2

- [ ] T030 Add ActionType::ShopBuy variant for rate limiting at `crates/plix-server/src/anti_cheat/mod.rs`
- [ ] T031 Configure ShopBuy rate limit (5 req/sec = 200ms cooldown) at `crates/plix-server/src/anti_cheat/mod.rs`
- [ ] T032 Create PurchaseResult struct (success, offer_id, item_id, quantity, fail_reason, new_balance) at `crates/plix-server/src/economy/purchase.rs`
- [ ] T033 [US2] Implement try_purchase validation chain at `crates/plix-server/src/economy/purchase.rs`:
  - Check economy_enabled
  - Check player alive
  - Lookup offer
  - Check rate limit
  - Check balance >= price
  - Check hotbar.can_add()
  - Check max_per_match limit
- [ ] T034 [US2] Implement atomic purchase application (spend coins, add item, record purchase) at `crates/plix-server/src/economy/purchase.rs`
- [ ] T035 [US2] Handle BuyRequest message (call try_purchase, send PurchaseResult + BalanceUpdate + InventoryUpdate) at `crates/plix-server/src/lib.rs`
- [ ] T036 [US2] Integrate with Hotbar.can_add() and Hotbar.try_add_item() from Feature 021 at `crates/plix-server/src/economy/purchase.rs`

### Tests for User Story 2

- [ ] T037 [P] [US2] Unit test: successful purchase -> item added, balance deducted at `crates/plix-server/tests/economy_purchase_test.rs`
- [ ] T038 [P] [US2] Unit test: EconomyDisabled rejection at `crates/plix-server/tests/economy_purchase_test.rs`
- [ ] T039 [P] [US2] Unit test: UnknownOffer rejection at `crates/plix-server/tests/economy_purchase_test.rs`
- [ ] T040 [P] [US2] Unit test: InsufficientBalance rejection (balance unchanged) at `crates/plix-server/tests/economy_purchase_test.rs`
- [ ] T041 [P] [US2] Unit test: HotbarFull rejection (balance unchanged) at `crates/plix-server/tests/economy_purchase_test.rs`
- [ ] T042 [P] [US2] Unit test: RateLimited rejection at `crates/plix-server/tests/economy_purchase_test.rs`
- [ ] T043 [P] [US2] Unit test: PurchaseLimitReached rejection (max_per_match exceeded) at `crates/plix-server/tests/economy_purchase_test.rs`
- [ ] T044 [P] [US2] Unit test: PlayerDead rejection at `crates/plix-server/tests/economy_purchase_test.rs`
- [ ] T045 [US2] Unit test: atomicity (fail -> no state changes) at `crates/plix-server/tests/economy_purchase_test.rs`

**Checkpoint**: User Stories 1 AND 2 complete - core earn/spend loop functional

---

## Phase 5: User Story 3 - View Balance and Shop Offers (Priority: P2)

**Goal**: Players can check balance and see available shop offers via console commands

**Independent Test**: /balance shows current coins; /shop lists available offers with prices

### Implementation for User Story 3

- [ ] T046 [P] [US3] Implement /balance command at `crates/plix-client/src/console.rs` (send BalanceRequest, display result)
- [ ] T047 [P] [US3] Implement /buy <offer_id> command at `crates/plix-client/src/console.rs` (parse argument, send BuyRequest)
- [ ] T048 [P] [US3] Implement /shop command (optional v1) at `crates/plix-client/src/console.rs` (send ShopListRequest, display offers)
- [ ] T049 [US3] Handle ShopListRequest message (return ShopList with mode-filtered offers) at `crates/plix-server/src/lib.rs`

### Tests for User Story 3

- [ ] T050 [US3] Test: /balance displays correct coin amount at `crates/plix-client/tests/console_test.rs` (if test infra exists)
- [ ] T051 [US3] Test: /buy parses offer_id correctly at `crates/plix-client/tests/console_test.rs`

**Checkpoint**: User Story 3 complete - players have visibility into economy state

---

## Phase 6: User Story 4 - Server Admin Configures Economy (Priority: P2)

**Goal**: Server admins customize earning rules and shop offers via arena TOML

**Independent Test**: Admin modifies arena TOML with custom prices, restarts server, players see new prices

### Implementation for User Story 4

- [ ] T052 [P] [US4] Add ArenaEconomyConfig struct (enabled, kill_reward, ctf_capture_reward, br_placement_rewards, shop_offers) at `crates/plix-arena/src/format.rs`
- [ ] T053 [P] [US4] Add ShopOfferConfig struct (offer_id, item_id, quantity, price, max_per_match) at `crates/plix-arena/src/format.rs`
- [ ] T054 [US4] Parse [economy] section from arena TOML at `crates/plix-arena/src/loader.rs`
- [ ] T055 [US4] Load EconomyConfig from arena on server startup at `crates/plix-server/src/lib.rs`
- [ ] T056 [US4] Add default shop offers to test_arena.toml (health_pack: 20, sword: 50, bow: 75, scrap: 10) at `assets/arenas/test_arena.toml`

### Tests for User Story 4

- [ ] T057 [P] [US4] Test: custom kill_reward in TOML applied correctly at `crates/plix-arena/tests/economy_config_test.rs`
- [ ] T058 [P] [US4] Test: invalid config (price=0) logs error, offer disabled at `crates/plix-arena/tests/economy_config_test.rs`

**Checkpoint**: User Story 4 complete - admins can customize economy

---

## Phase 7: User Story 5 - Match Reset Economy (Priority: P3)

**Goal**: Coin balances reset at match start for fair competitive play

**Independent Test**: Player ends match with 100 coins, joins new match, starts with 0 coins

### Implementation for User Story 5

- [ ] T059 [US5] Reset all player wallets on transition to Playing phase at `crates/plix-server/src/match_state.rs`
- [ ] T060 [US5] Clear purchase counters (max_per_match tracking) on match reset at `crates/plix-server/src/match_state.rs`

### Tests for User Story 5

- [ ] T061 [US5] Integration test: match end -> new match -> balance = 0 at `crates/plix-server/tests/economy_integration_test.rs`
- [ ] T062 [US5] Integration test: purchase counts reset between matches at `crates/plix-server/tests/economy_integration_test.rs`

**Checkpoint**: User Story 5 complete - matches start fair

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Observability, non-regression, and documentation

### Observability & Metrics

- [ ] T063 [P] Create EconomyMetrics struct (coins_earned_total, coins_spent_total, purchases_total, purchases_failed_total by reason, rate_limited_total) at `crates/plix-server/src/economy/metrics.rs`
- [ ] T064 [P] Add info-level logging for successful purchases (player_id, offer_id, price) at `crates/plix-server/src/economy/purchase.rs`
- [ ] T065 [P] Add debug-level logging for failed purchases (player_id, offer_id, reason) at `crates/plix-server/src/economy/purchase.rs`
- [ ] T066 Add event logging for major reward events (captures, victories) at `crates/plix-server/src/economy/earnings.rs`

### Integration Tests

- [ ] T067 [P] Full flow test: kill -> earn -> /balance -> /buy health_pack -> inventory + balance update at `crates/plix-server/tests/economy_integration_test.rs`
- [ ] T068 [P] Full flow test: CTF capture -> earn -> buy possible at `crates/plix-server/tests/economy_integration_test.rs`
- [ ] T069 [P] Full flow test: BR Lite winner reward -> buy possible at `crates/plix-server/tests/economy_integration_test.rs`

### Non-Regression Tests

- [ ] T070 [P] Verify TDM mode works with economy disabled at `crates/plix-server/tests/economy_integration_test.rs`
- [ ] T071 [P] Verify FFA mode works with economy disabled at `crates/plix-server/tests/economy_integration_test.rs`
- [ ] T072 [P] Verify hotbar/loot/crafting/weapons unchanged by economy at `crates/plix-server/tests/economy_integration_test.rs`

### Documentation

- [ ] T073 Document economy config options in arena TOML example at `assets/arenas/test_arena.toml`
- [ ] T074 Document console commands (/balance, /buy, /shop) in spec.md
- [ ] T075 Run final cargo test, cargo clippy, cargo fmt --check

**Checkpoint**: Feature 024 complete and polished

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-7)**: All depend on Foundational phase completion
  - US1 (Earning) and US2 (Purchasing) are P1 - core MVP
  - US3 (View Balance) and US4 (Admin Config) are P2
  - US5 (Match Reset) is P3
- **Polish (Phase 8)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational - earns coins without purchases
- **User Story 2 (P1)**: Can start after Foundational - requires US1 for realistic testing (needs coins to spend)
- **User Story 3 (P2)**: Can start after US1+US2 - displays balance and executes purchases
- **User Story 4 (P2)**: Can start after Foundational - config loading independent of runtime
- **User Story 5 (P3)**: Can start after Foundational - match reset independent of specific economy features

### Within Each User Story

- Models/types before services
- Services before handlers
- Core implementation before integration hooks
- Story complete before next priority

### Parallel Opportunities

- T002, T003, T004: Setup can run in parallel
- T005, T006, T007, T009, T010, T011, T013: Type definitions can run in parallel
- T016, T017, T018: Foundational tests can run in parallel
- T026-T029: US1 tests can run in parallel
- T037-T045: US2 tests can run in parallel (except T045)
- T046, T047, T048: Console commands can run in parallel
- T052, T053: Config structs can run in parallel
- T063-T066: Metrics and logging can run in parallel
- T067-T072: Integration/regression tests can run in parallel

---

## Parallel Example: Phase 2 Foundational

```bash
# Launch all type definitions in parallel:
Task T005: "Define PurchaseRejectReason enum"
Task T006: "Add ClientMessage variants"
Task T007: "Add GameEvent variants"
Task T009: "Create EconomyConfig struct"
Task T010: "Implement get_economy_config"
Task T011: "Add EconomyConfig validation"
Task T013: "Create ShopOffer struct"

# Launch all foundational tests in parallel:
Task T016: "Unit tests for PlayerWallet"
Task T017: "Unit tests for ShopRegistry"
Task T018: "Unit tests for EconomyConfig validation"
```

## Parallel Example: User Story 2 Tests

```bash
# Launch all US2 rejection tests in parallel:
Task T037: "successful purchase test"
Task T038: "EconomyDisabled rejection"
Task T039: "UnknownOffer rejection"
Task T040: "InsufficientBalance rejection"
Task T041: "HotbarFull rejection"
Task T042: "RateLimited rejection"
Task T043: "PurchaseLimitReached rejection"
Task T044: "PlayerDead rejection"
```

---

## Implementation Strategy

### MVP First (User Stories 1 + 2 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1 (Earning)
4. Complete Phase 4: User Story 2 (Purchasing)
5. **STOP and VALIDATE**: Core earn/spend loop works
6. Deploy/demo MVP

### Incremental Delivery

1. Setup + Foundational -> Foundation ready
2. Add US1 (Earning) -> Test independently -> Players can earn coins
3. Add US2 (Purchasing) -> Test independently -> Players can spend coins (MVP!)
4. Add US3 (Commands) -> Test independently -> Better UX
5. Add US4 (Config) -> Test independently -> Admin customization
6. Add US5 (Reset) -> Test independently -> Competitive fairness
7. Add Polish -> Full observability and docs

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 (Earning)
   - Developer B: User Story 2 (Purchasing)
   - Developer C: User Story 4 (Config)
3. After US1+US2:
   - Developer A: User Story 3 (Commands)
   - Developer B: User Story 5 (Reset)
   - Developer C: Polish phase

---

## Summary

| Phase | Task Count | Parallel Tasks |
|-------|------------|----------------|
| Phase 1: Setup | 4 | 3 |
| Phase 2: Foundational | 14 | 10 |
| Phase 3: US1 Earning | 11 | 4 |
| Phase 4: US2 Purchasing | 16 | 9 |
| Phase 5: US3 Commands | 6 | 3 |
| Phase 6: US4 Config | 7 | 4 |
| Phase 7: US5 Reset | 4 | 0 |
| Phase 8: Polish | 13 | 10 |
| **Total** | **75** | **43** |

**MVP Scope**: Phases 1-4 (US1 + US2) = 45 tasks for core earn/spend loop

---

## Notes

- [P] tasks = different files, no dependencies
- [US#] label maps task to specific user story for traceability
- Each user story is independently completable and testable
- Tests are included as mandated by spec.md success criteria (SC-007)
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
