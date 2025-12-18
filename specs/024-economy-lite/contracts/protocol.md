# Protocol Contracts: Economy Lite

**Feature**: 024-economy-lite
**Date**: 2025-12-17
**Protocol Version**: 1.0

## Overview

This document defines the network protocol messages for the Economy Lite feature. All messages use bincode serialization and follow the existing plix protocol patterns.

---

## Client → Server Messages

### BuyRequest

Request to purchase an item from the shop.

```rust
/// ClientMessage variant
BuyRequest {
    /// Unique identifier of the shop offer to purchase
    offer_id: String,
}
```

**Constraints**:
- `offer_id`: Non-empty string, max 64 characters
- Rate limited: Max 5 requests per second per player

**Server Response**: `GameEvent::PurchaseResult`

---

### BalanceRequest

Request current coin balance.

```rust
/// ClientMessage variant
BalanceRequest
```

**Rate Limit**: Max 10 requests per second per player

**Server Response**: `GameEvent::BalanceUpdate`

---

### ShopListRequest (Optional v1)

Request list of available shop offers for current mode.

```rust
/// ClientMessage variant
ShopListRequest
```

**Rate Limit**: Max 2 requests per second per player

**Server Response**: `GameEvent::ShopList`

---

## Server → Client Messages (GameEvent)

### BalanceUpdate

Sent when player's coin balance changes (earnings or purchases).

```rust
/// GameEvent variant
BalanceUpdate {
    /// Current coin balance after change
    balance: u32,
}
```

**Triggered By**:
- Player earns coins (kill, capture, placement)
- Player spends coins (successful purchase)
- Match reset (balance = 0)
- Client `BalanceRequest`

---

### PurchaseResult

Sent in response to `BuyRequest`.

```rust
/// GameEvent variant
PurchaseResult {
    /// Whether purchase succeeded
    success: bool,
    /// Offer ID that was attempted
    offer_id: String,
    /// Item received (if successful)
    output_item: Option<ItemId>,
    /// Quantity received (if successful)
    output_quantity: Option<u8>,
    /// Failure reason (if failed)
    fail_reason: Option<PurchaseRejectReason>,
}
```

**Notes**:
- On success: `output_item` and `output_quantity` are populated
- On failure: `fail_reason` is populated
- Always followed by `BalanceUpdate` if balance changed

---

### PurchaseRejectReason

Enum for purchase rejection reasons (protocol-level).

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PurchaseRejectReason {
    /// Economy not active for current game mode
    EconomyDisabled,
    /// offer_id not found in shop registry
    UnknownOffer,
    /// Offer not available in current game mode
    ModeRestricted,
    /// Player does not have enough coins
    InsufficientBalance,
    /// No space in hotbar for purchased item
    HotbarFull,
    /// max_per_match limit exceeded for this offer
    PurchaseLimitReached,
    /// Too many purchase requests (rate limited)
    RateLimited,
    /// Player is dead and cannot purchase
    PlayerDead,
}
```

---

### ShopList (Optional v1)

List of available shop offers for the current mode.

```rust
/// GameEvent variant
ShopList {
    /// Available offers in current mode
    offers: Vec<ShopOfferInfo>,
}

/// Minimal offer info for client display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopOfferInfo {
    pub offer_id: String,
    pub item_id: ItemId,
    pub quantity: u8,
    pub price: u32,
}
```

---

## Message Flow Diagrams

### Successful Purchase

```
Client                              Server
   │                                   │
   │  BuyRequest { "health_pack" }    │
   │ ─────────────────────────────────>│
   │                                   │ validate & apply
   │                                   │
   │  PurchaseResult { success: true, │
   │    offer_id: "health_pack",      │
   │    output_item: HEALTH_PACK,     │
   │    output_quantity: 1 }          │
   │ <─────────────────────────────────│
   │                                   │
   │  BalanceUpdate { balance: 30 }   │
   │ <─────────────────────────────────│
   │                                   │
   │  InventoryUpdate { ... }         │
   │ <─────────────────────────────────│
```

### Failed Purchase (Insufficient Balance)

```
Client                              Server
   │                                   │
   │  BuyRequest { "sword" }          │
   │ ─────────────────────────────────>│
   │                                   │ validate → fail
   │                                   │
   │  PurchaseResult { success: false,│
   │    offer_id: "sword",            │
   │    fail_reason: InsufficientBalance } │
   │ <─────────────────────────────────│
   │                                   │
   │  (no BalanceUpdate - unchanged)  │
   │  (no InventoryUpdate)            │
```

### Balance Query

```
Client                              Server
   │                                   │
   │  BalanceRequest                  │
   │ ─────────────────────────────────>│
   │                                   │
   │  BalanceUpdate { balance: 75 }   │
   │ <─────────────────────────────────│
```

### Earning Coins (Server-Initiated)

```
Server Event                        Client
   │ Player kills enemy               │
   │                                   │
   │ (internal: ledger.add(10))       │
   │                                   │
   │  BalanceUpdate { balance: 60 }   │
   │ ─────────────────────────────────>│
```

---

## Serialization Format

All messages use **bincode** serialization, consistent with existing plix protocol.

```rust
// Encoding
let bytes = bincode::serialize(&ClientMessage::BuyRequest {
    offer_id: "health_pack".to_string()
})?;

// Decoding
let msg: ClientMessage = bincode::deserialize(&bytes)?;
```

---

## Error Handling

### Client Responsibilities
- Handle all `PurchaseRejectReason` variants
- Display user-friendly error messages
- Respect rate limits (exponential backoff on RateLimited)

### Server Responsibilities
- Always respond to valid requests
- Never send partial responses
- Log all failed purchases at debug level

---

## Versioning

Protocol changes follow semantic versioning:
- **MAJOR**: Breaking changes to message format
- **MINOR**: New message types or optional fields
- **PATCH**: Clarifications, no wire format changes

Current version: **1.0**
