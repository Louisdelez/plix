# Quickstart Guide: Economy Lite

**Feature**: 024-economy-lite
**Date**: 2025-12-17
**Status**: Ready for Implementation

## Overview

This guide provides a step-by-step implementation path for the Economy Lite feature. Follow the phases in order to ensure proper dependency resolution.

---

## Prerequisites

Before starting implementation, ensure:

1. **Feature 021 (Hotbar/Inventory)** is implemented and working
2. **Feature 023 (Crafting Lite)** patterns are available for reference
3. All existing tests pass: `cargo test`
4. Branch created: `git checkout -b 024-economy-lite`

---

## Implementation Phases

### Phase 1: Core Data Types (plix-common)

**Goal**: Define shared types used by both client and server.

**Files to create/modify**:
- `crates/plix-common/src/economy/mod.rs` (new)
- `crates/plix-common/src/economy/types.rs` (new)
- `crates/plix-common/src/protocol/messages.rs` (extend)
- `crates/plix-common/src/lib.rs` (add module)

**Tasks**:

1. Create economy module structure:
```rust
// crates/plix-common/src/economy/mod.rs
mod types;
pub use types::*;
```

2. Define core types:
```rust
// crates/plix-common/src/economy/types.rs
use serde::{Deserialize, Serialize};

/// Reason for purchase rejection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PurchaseRejectReason {
    EconomyDisabled,
    UnknownOffer,
    ModeRestricted,
    InsufficientBalance,
    HotbarFull,
    PurchaseLimitReached,
    RateLimited,
    PlayerDead,
}
```

3. Add protocol messages to `messages.rs`:
```rust
// In ClientMessage enum
BuyRequest { offer_id: String },
BalanceRequest,

// In GameEvent enum
BalanceUpdate { balance: u32 },
PurchaseResult {
    success: bool,
    offer_id: String,
    output_item: Option<ItemId>,
    output_quantity: Option<u8>,
    fail_reason: Option<PurchaseRejectReason>,
},
```

**Validation**: `cargo check -p plix-common`

---

### Phase 2: Server Economy Module (plix-server)

**Goal**: Implement server-side economy logic.

**Files to create**:
- `crates/plix-server/src/economy/mod.rs`
- `crates/plix-server/src/economy/config.rs`
- `crates/plix-server/src/economy/wallet.rs`
- `crates/plix-server/src/economy/shop.rs`
- `crates/plix-server/src/economy/purchase.rs`
- `crates/plix-server/src/economy/earnings.rs`

#### 2.1 Module Structure

```rust
// crates/plix-server/src/economy/mod.rs
mod config;
mod wallet;
mod shop;
mod purchase;
mod earnings;

pub use config::*;
pub use wallet::*;
pub use shop::*;
pub use purchase::*;
pub use earnings::*;
```

#### 2.2 PlayerWallet

```rust
// crates/plix-server/src/economy/wallet.rs
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct PlayerWallet {
    balance: u32,
    purchases: HashMap<String, u8>,
}

impl PlayerWallet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_balance(&self) -> u32 {
        self.balance
    }

    pub fn add_coins(&mut self, amount: u32) {
        self.balance = self.balance.saturating_add(amount);
    }

    pub fn try_spend(&mut self, amount: u32) -> bool {
        if self.balance >= amount {
            self.balance -= amount;
            true
        } else {
            false
        }
    }

    pub fn get_purchase_count(&self, offer_id: &str) -> u8 {
        *self.purchases.get(offer_id).unwrap_or(&0)
    }

    pub fn record_purchase(&mut self, offer_id: &str) {
        *self.purchases.entry(offer_id.to_string()).or_insert(0) += 1;
    }

    pub fn reset(&mut self) {
        self.balance = 0;
        self.purchases.clear();
    }
}
```

#### 2.3 ShopRegistry

```rust
// crates/plix-server/src/economy/shop.rs
use crate::GameMode;
use plix_common::ItemId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopOffer {
    pub offer_id: String,
    pub item_id: ItemId,
    pub quantity: u8,
    pub price: u32,
    pub allowed_modes: Option<Vec<GameMode>>,
    pub max_per_match: Option<u8>,
}

#[derive(Debug, Clone, Default)]
pub struct ShopRegistry {
    offers: Vec<ShopOffer>,
}

impl ShopRegistry {
    pub fn new(offers: Vec<ShopOffer>) -> Self {
        Self { offers }
    }

    pub fn get(&self, offer_id: &str) -> Option<&ShopOffer> {
        self.offers.iter().find(|o| o.offer_id == offer_id)
    }

    pub fn list_for_mode(&self, mode: GameMode) -> Vec<&ShopOffer> {
        self.offers
            .iter()
            .filter(|o| {
                o.allowed_modes
                    .as_ref()
                    .map_or(true, |modes| modes.contains(&mode))
            })
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.offers.is_empty()
    }
}
```

#### 2.4 EconomyConfig

```rust
// crates/plix-server/src/economy/config.rs
use super::ShopOffer;
use crate::GameMode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomyConfig {
    pub enabled: bool,
    pub kill_reward: u32,
    pub ctf_capture_reward: u32,
    pub br_placement_rewards: [u32; 3],
    pub shop_offers: Vec<ShopOffer>,
}

impl Default for EconomyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            kill_reward: 10,
            ctf_capture_reward: 25,
            br_placement_rewards: [50, 30, 15],
            shop_offers: vec![],
        }
    }
}

pub fn get_economy_config(mode: GameMode, arena_config: Option<&EconomyConfig>) -> EconomyConfig {
    let base = arena_config.cloned().unwrap_or_default();

    // Apply mode-specific defaults
    let enabled = match mode {
        GameMode::Training | GameMode::Tdm | GameMode::Ffa => false,
        GameMode::Ctf | GameMode::BrLite => true,
    };

    EconomyConfig {
        enabled: arena_config.map_or(enabled, |c| c.enabled),
        ..base
    }
}
```

#### 2.5 PurchaseSystem

```rust
// crates/plix-server/src/economy/purchase.rs
use super::{EconomyConfig, PlayerWallet, ShopRegistry};
use plix_common::economy::PurchaseRejectReason;
use plix_common::inventory::Hotbar;

pub struct PurchaseResult {
    pub success: bool,
    pub offer_id: String,
    pub item_id: Option<plix_common::ItemId>,
    pub quantity: Option<u8>,
    pub fail_reason: Option<PurchaseRejectReason>,
    pub new_balance: u32,
}

pub fn try_purchase(
    offer_id: &str,
    wallet: &mut PlayerWallet,
    hotbar: &mut Hotbar,
    shop: &ShopRegistry,
    config: &EconomyConfig,
    is_alive: bool,
) -> PurchaseResult {
    // 1. Check economy enabled
    if !config.enabled {
        return fail(offer_id, wallet, PurchaseRejectReason::EconomyDisabled);
    }

    // 2. Check player alive
    if !is_alive {
        return fail(offer_id, wallet, PurchaseRejectReason::PlayerDead);
    }

    // 3. Look up offer
    let offer = match shop.get(offer_id) {
        Some(o) => o,
        None => return fail(offer_id, wallet, PurchaseRejectReason::UnknownOffer),
    };

    // 4. Check balance
    if wallet.get_balance() < offer.price {
        return fail(offer_id, wallet, PurchaseRejectReason::InsufficientBalance);
    }

    // 5. Check hotbar space
    if !hotbar.can_add(offer.item_id, offer.quantity) {
        return fail(offer_id, wallet, PurchaseRejectReason::HotbarFull);
    }

    // 6. Check purchase limit
    if let Some(max) = offer.max_per_match {
        if wallet.get_purchase_count(offer_id) >= max {
            return fail(offer_id, wallet, PurchaseRejectReason::PurchaseLimitReached);
        }
    }

    // 7. Apply atomically
    wallet.try_spend(offer.price);
    hotbar.try_add_item(offer.item_id, offer.quantity);
    wallet.record_purchase(offer_id);

    PurchaseResult {
        success: true,
        offer_id: offer_id.to_string(),
        item_id: Some(offer.item_id),
        quantity: Some(offer.quantity),
        fail_reason: None,
        new_balance: wallet.get_balance(),
    }
}

fn fail(offer_id: &str, wallet: &PlayerWallet, reason: PurchaseRejectReason) -> PurchaseResult {
    PurchaseResult {
        success: false,
        offer_id: offer_id.to_string(),
        item_id: None,
        quantity: None,
        fail_reason: Some(reason),
        new_balance: wallet.get_balance(),
    }
}
```

#### 2.6 Earnings

```rust
// crates/plix-server/src/economy/earnings.rs
use super::{EconomyConfig, PlayerWallet};

pub enum EarningEvent {
    Kill,
    CtfCapture,
    BrPlacement(u8), // 1st, 2nd, 3rd
}

pub fn award_coins(
    event: EarningEvent,
    wallet: &mut PlayerWallet,
    config: &EconomyConfig,
) -> Option<u32> {
    if !config.enabled {
        return None;
    }

    let amount = match event {
        EarningEvent::Kill => config.kill_reward,
        EarningEvent::CtfCapture => config.ctf_capture_reward,
        EarningEvent::BrPlacement(place) => match place {
            1 => config.br_placement_rewards[0],
            2 => config.br_placement_rewards[1],
            3 => config.br_placement_rewards[2],
            _ => 0,
        },
    };

    if amount > 0 {
        wallet.add_coins(amount);
        Some(wallet.get_balance())
    } else {
        None
    }
}
```

**Validation**: `cargo check -p plix-server`

---

### Phase 3: Session Integration

**Goal**: Add wallet to player session and handle match reset.

**Files to modify**:
- `crates/plix-server/src/session.rs`
- `crates/plix-server/src/match_state.rs`

**Tasks**:

1. Add wallet field to `ServerPlayer`:
```rust
// In ServerPlayer struct
pub wallet: PlayerWallet,
```

2. Initialize wallet in player creation:
```rust
wallet: PlayerWallet::new(),
```

3. Reset wallets on match start in `match_state.rs`:
```rust
// In transition_to_playing() or match reset logic
for player in players.values_mut() {
    player.wallet.reset();
}
```

**Validation**: `cargo test -p plix-server`

---

### Phase 4: Message Handling

**Goal**: Wire up buy request handling in server.

**Files to modify**:
- `crates/plix-server/src/lib.rs`

**Tasks**:

1. Handle `BuyRequest` in message processing:
```rust
ClientMessage::BuyRequest { offer_id } => {
    let result = economy::try_purchase(
        &offer_id,
        &mut player.wallet,
        &mut player.hotbar,
        &self.shop_registry,
        &self.economy_config,
        player.is_alive(),
    );

    // Send PurchaseResult
    self.send_to_player(player_id, GameEvent::PurchaseResult {
        success: result.success,
        offer_id: result.offer_id,
        output_item: result.item_id,
        output_quantity: result.quantity,
        fail_reason: result.fail_reason,
    });

    // Send BalanceUpdate if changed
    if result.success {
        self.send_to_player(player_id, GameEvent::BalanceUpdate {
            balance: result.new_balance,
        });
    }
}

ClientMessage::BalanceRequest => {
    self.send_to_player(player_id, GameEvent::BalanceUpdate {
        balance: player.wallet.get_balance(),
    });
}
```

2. Add earning hooks to kill/capture handlers:
```rust
// In kill handling
if let Some(new_balance) = economy::award_coins(
    EarningEvent::Kill,
    &mut killer.wallet,
    &self.economy_config,
) {
    self.send_to_player(killer_id, GameEvent::BalanceUpdate {
        balance: new_balance,
    });
}
```

**Validation**: `cargo test`

---

### Phase 5: Configuration Loading

**Goal**: Load economy config from arena TOML.

**Files to modify**:
- `crates/plix-arena/src/format.rs`
- Arena TOML files

**Tasks**:

1. Add economy section to arena format:
```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ArenaEconomyConfig {
    pub enabled: Option<bool>,
    pub kill_reward: Option<u32>,
    pub ctf_capture_reward: Option<u32>,
    pub br_placement_rewards: Option<[u32; 3]>,
    pub shop_offers: Option<Vec<ShopOfferConfig>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopOfferConfig {
    pub offer_id: String,
    pub item_id: String,
    pub quantity: u8,
    pub price: u32,
    pub max_per_match: Option<u8>,
}
```

2. Add default shop offers to test arena:
```toml
[economy]
enabled = true
kill_reward = 10

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
```

**Validation**: `cargo test -p plix-arena`

---

### Phase 6: Rate Limiting

**Goal**: Add purchase rate limiting via anti-cheat module.

**Files to modify**:
- `crates/plix-server/src/anti_cheat/mod.rs`

**Tasks**:

1. Add `ShopBuy` variant to `ActionType` enum:
```rust
pub enum ActionType {
    // ... existing variants
    ShopBuy,
}
```

2. Configure rate limit (5 requests/second):
```rust
ActionType::ShopBuy => Duration::from_millis(200),
```

3. Check rate limit before processing buy request

**Validation**: `cargo test -p plix-server`

---

### Phase 7: Tests

**Goal**: Comprehensive test coverage.

**Files to create**:
- `crates/plix-server/tests/economy_balance_test.rs`
- `crates/plix-server/tests/economy_purchase_test.rs`
- `crates/plix-server/tests/economy_earnings_test.rs`
- `crates/plix-server/tests/economy_integration_test.rs`

**Test cases**:

1. **Wallet tests**:
   - add_coins saturates at u32::MAX
   - try_spend fails with insufficient balance
   - reset clears balance and purchases

2. **Purchase tests**:
   - Success path with item delivery
   - Fail: economy disabled
   - Fail: unknown offer
   - Fail: insufficient balance
   - Fail: hotbar full
   - Fail: purchase limit reached
   - Fail: player dead

3. **Earnings tests**:
   - Kill reward awarded correctly
   - CTF capture reward awarded
   - BR placement rewards (1st, 2nd, 3rd)
   - No reward when economy disabled

4. **Integration tests**:
   - Full kill → earn → buy → receive flow
   - Match reset clears balances
   - Mode restrictions respected

**Validation**: `cargo test` - all tests pass

---

## Checklist

- [ ] Phase 1: Core types in plix-common
- [ ] Phase 2: Economy module in plix-server
- [ ] Phase 3: Session integration
- [ ] Phase 4: Message handling
- [ ] Phase 5: Configuration loading
- [ ] Phase 6: Rate limiting
- [ ] Phase 7: Tests
- [ ] All tests pass: `cargo test`
- [ ] Clippy clean: `cargo clippy`
- [ ] Format clean: `cargo fmt --check`

---

## Common Issues

### Issue: "Unknown ItemId"
**Solution**: Ensure item IDs in shop config match entries in item registry.

### Issue: "Hotbar full" even with space
**Solution**: Check stack limits in item registry; some items may not stack.

### Issue: Balance not resetting
**Solution**: Verify match state transition calls `wallet.reset()` for all players.

---

## Next Steps

After implementation:
1. Run full test suite: `cargo test`
2. Manual testing in BR Lite and CTF modes
3. Consider v2 features: UI shop, persistence, trading
