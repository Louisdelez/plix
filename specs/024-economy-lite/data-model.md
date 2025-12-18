# Data Model: Economy Lite

**Feature**: 024-economy-lite
**Date**: 2025-12-17
**Status**: Complete

## Overview

This document defines the data entities, their relationships, validation rules, and state transitions for the Economy Lite feature.

---

## Entities

### PlayerWallet

Per-player transient economy state for current match.

| Field | Type | Description | Constraints |
|-------|------|-------------|-------------|
| balance | u32 | Current coin balance | >= 0, saturating add (cap at u32::MAX) |
| purchases | HashMap<String, u8> | Count of purchases per offer_id this match | Values >= 0 |

**Lifecycle**:
- Created: When player joins match (balance = 0, empty purchases)
- Reset: On match transition to Playing phase
- Destroyed: When player disconnects

**Validation Rules**:
- Balance cannot go negative (spend validation)
- Balance additions saturate at u32::MAX

**Operations**:
```rust
impl PlayerWallet {
    fn new() -> Self;
    fn get_balance(&self) -> u32;
    fn add_coins(&mut self, amount: u32);           // Saturating add
    fn try_spend(&mut self, amount: u32) -> bool;   // Returns false if insufficient
    fn get_purchase_count(&self, offer_id: &str) -> u8;
    fn record_purchase(&mut self, offer_id: &str);
    fn reset(&mut self);                            // Balance = 0, clear purchases
}
```

---

### ShopOffer

A purchasable item configuration defined in arena config.

| Field | Type | Description | Constraints |
|-------|------|-------------|-------------|
| offer_id | String | Unique identifier for the offer | Non-empty, unique within registry |
| item_id | ItemId | Item to deliver on purchase | Must be valid ItemId |
| quantity | u8 | Number of items per purchase | > 0 |
| price | u32 | Cost in coins | > 0 |
| allowed_modes | Option<Vec<GameMode>> | Modes where offer is available | None = all modes |
| max_per_match | Option<u8> | Maximum purchases per player per match | None = unlimited |

**Validation Rules (at load)**:
- offer_id must be unique
- item_id must exist in item registry
- quantity > 0
- price > 0

**Example**:
```rust
ShopOffer {
    offer_id: "health_pack".to_string(),
    item_id: ItemId::HEALTH_PACK,
    quantity: 1,
    price: 20,
    allowed_modes: None,  // Available in all modes with economy enabled
    max_per_match: Some(5),
}
```

---

### ShopRegistry

Collection of shop offers with lookup methods.

| Field | Type | Description |
|-------|------|-------------|
| offers | Vec<ShopOffer> | All configured shop offers |

**Operations**:
```rust
impl ShopRegistry {
    fn new(offers: Vec<ShopOffer>) -> Self;
    fn get(&self, offer_id: &str) -> Option<&ShopOffer>;
    fn list_for_mode(&self, mode: GameMode) -> Vec<&ShopOffer>;
    fn is_empty(&self) -> bool;
}
```

---

### EarningRule

Configuration for awarding coins on specific events.

| Field | Type | Description | Constraints |
|-------|------|-------------|-------------|
| event_type | EarningEventType | Type of event that triggers reward | Valid enum variant |
| reward | u32 | Coins to award | >= 0 (0 = disabled) |

**EarningEventType Enum**:
```rust
enum EarningEventType {
    Kill,           // Player eliminates another player
    CtfCapture,     // Team captures enemy flag
    BrPlacement1st, // BR Lite 1st place
    BrPlacement2nd, // BR Lite 2nd place
    BrPlacement3rd, // BR Lite 3rd place
}
```

---

### EconomyConfig

Per-mode economy configuration.

| Field | Type | Description | Default |
|-------|------|-------------|---------|
| enabled | bool | Whether economy is active for this mode | Mode-dependent |
| kill_reward | u32 | Coins for player kill | 10 |
| ctf_capture_reward | u32 | Coins for flag capture | 25 |
| br_placement_rewards | [u32; 3] | Coins for 1st/2nd/3rd place | [50, 30, 15] |
| shop_offers | Vec<ShopOffer> | Available shop offers | Default 4 offers |

**Default Mode Settings**:
| Mode | Enabled | Notes |
|------|---------|-------|
| Training | false | Economy disabled by default |
| Tdm | false | Competitive, economy disabled |
| Ffa | false | Competitive, economy disabled |
| Ctf | true | Economy enabled (captures reward coins) |
| BrLite | true | Economy enabled (survival rewards coins) |

---

### PurchaseFailReason

Enum for purchase rejection reasons.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PurchaseFailReason {
    EconomyDisabled,     // Economy not active for current mode
    UnknownOffer,        // offer_id not found in registry
    ModeRestricted,      // Offer not allowed in current mode
    InsufficientBalance, // Not enough coins
    HotbarFull,          // No space for item
    PurchaseLimitReached,// max_per_match exceeded
    RateLimited,         // Too many requests
    PlayerDead,          // Cannot purchase while dead
}
```

---

## Relationships

```text
┌─────────────────┐
│ EconomyConfig   │
│ - enabled       │
│ - earnings      │
│ - shop_offers[] ├────────────┐
└────────┬────────┘            │
         │                     │
         │ 1:1 per mode        │ 1:N
         ▼                     ▼
┌─────────────────┐   ┌─────────────────┐
│  EarningRule[]  │   │  ShopRegistry   │
│ - event_type    │   │ - offers[]      │
│ - reward        │   └────────┬────────┘
└─────────────────┘            │
                               │ 1:N
                               ▼
                      ┌─────────────────┐
                      │   ShopOffer     │
                      │ - offer_id      │
                      │ - item_id       │
                      │ - quantity      │
                      │ - price         │
                      │ - restrictions  │
                      └─────────────────┘

┌─────────────────┐
│  ServerPlayer   │
│ - id            │
│ - hotbar        │
│ - wallet ───────┼───────────┐
└─────────────────┘           │
                              │ 1:1
                              ▼
                     ┌─────────────────┐
                     │  PlayerWallet   │
                     │ - balance       │
                     │ - purchases{}   │
                     └─────────────────┘
```

---

## State Transitions

### PlayerWallet State Machine

```text
                    ┌──────────────────────────────────────────┐
                    │                                          │
                    ▼                                          │
┌─────────────┐   Join    ┌─────────────┐   Disconnect   ┌─────┴─────┐
│   (none)    │ ──────────> │  Active     │ ─────────────> │ Destroyed │
└─────────────┘            │ balance >= 0│               └───────────┘
                           └──────┬──────┘
                                  │
           ┌──────────────────────┼──────────────────────┐
           │                      │                      │
           ▼                      ▼                      ▼
    ┌────────────┐        ┌────────────┐        ┌────────────┐
    │ Earn Coins │        │   Spend    │        │   Reset    │
    │ add(amt)   │        │ try_spend  │        │ reset()    │
    └────────────┘        └────────────┘        └────────────┘
```

### Purchase Flow State Machine

```text
┌────────────┐
│  Request   │
│ BuyRequest │
└─────┬──────┘
      │
      ▼
┌─────────────────────────────────────────────────────────────────┐
│                        VALIDATION CHAIN                          │
│  ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐     │
│  │ Economy  │──>│  Offer   │──>│ Rate     │──>│ Balance  │──>  │
│  │ Enabled? │   │ Exists?  │   │ Limit OK?│   │ >= Price?│     │
│  └────┬─────┘   └────┬─────┘   └────┬─────┘   └────┬─────┘     │
│       │NO            │NO            │NO            │NO          │
│       ▼              ▼              ▼              ▼            │
│  [FAIL:         [FAIL:        [FAIL:         [FAIL:            │
│   Disabled]      Unknown]      Rate]          Balance]         │
│                                                                 │
│  ┌──────────┐   ┌──────────┐                                   │
│  │ Hotbar   │──>│ Purchase │                                   │
│  │ Space?   │   │ Limit OK?│                                   │
│  └────┬─────┘   └────┬─────┘                                   │
│       │NO            │NO                                        │
│       ▼              ▼                                          │
│  [FAIL:         [FAIL:                                         │
│   Hotbar]        Limit]                                        │
└──────────────────────┬──────────────────────────────────────────┘
                       │ ALL PASS
                       ▼
              ┌────────────────┐
              │  ATOMIC APPLY  │
              │ 1. spend()     │
              │ 2. add_item()  │
              │ 3. record_buy  │
              └───────┬────────┘
                      │
                      ▼
              ┌────────────────┐
              │    RESPONSE    │
              │ PurchaseResult │
              │ BalanceUpdate  │
              │ InventoryUpdate│
              └────────────────┘
```

---

## Validation Rules Summary

### At Configuration Load
| Entity | Rule | Error |
|--------|------|-------|
| ShopOffer.offer_id | Unique, non-empty | DuplicateOfferId / EmptyOfferId |
| ShopOffer.item_id | Valid ItemId | InvalidItemId |
| ShopOffer.quantity | > 0 | InvalidQuantity |
| ShopOffer.price | > 0 | InvalidPrice |

### At Runtime (Purchase)
| Check | Order | Fail Reason |
|-------|-------|-------------|
| Economy enabled | 1 | EconomyDisabled |
| Offer exists | 2 | UnknownOffer |
| Mode allowed | 3 | ModeRestricted |
| Rate limit | 4 | RateLimited |
| Balance sufficient | 5 | InsufficientBalance |
| Hotbar space | 6 | HotbarFull |
| Purchase limit | 7 | PurchaseLimitReached |

---

## Index / Lookup Optimization

| Query | Data Structure | Complexity |
|-------|----------------|------------|
| Get offer by ID | HashMap<String, usize> index | O(1) |
| Get player balance | Direct field access | O(1) |
| Get purchase count | HashMap lookup | O(1) |
| List offers for mode | Pre-filtered or linear scan | O(n) offers |

---

## Serialization

All entities use serde derives for:
- Protocol messages (bincode)
- Configuration files (TOML via serde)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopOffer { ... }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PurchaseFailReason { ... }
```
