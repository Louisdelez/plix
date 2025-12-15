# Plix Network Protocol

## Overview

- Transport: UDP
- Encoding: Binary (bincode)
- Protocol Version: 1
- Max Packet Size: 1200 bytes

## Packet Header

```
┌────────────────────────────────────────┐
│ Version (1 byte)                       │
├────────────────────────────────────────┤
│ Channel (1 byte)                       │
├────────────────────────────────────────┤
│ Sequence (2 bytes, big-endian)         │
├────────────────────────────────────────┤
│ Ack (2 bytes, big-endian)              │
├────────────────────────────────────────┤
│ Ack Bits (4 bytes)                     │
├────────────────────────────────────────┤
│ Payload (variable)                     │
└────────────────────────────────────────┘
```

## Channels

| ID | Name | Delivery |
|----|------|----------|
| 0 | Unreliable | Fire-and-forget |
| 1 | Reliable | ACK/resend |
| 2 | Ordered | Reliable + ordered |

## Client Messages

### Connect

Sent when client wants to join server.

```rust
ClientMessage::Connect {
    protocol_version: u16,
    name: String,
}
```

Channel: Reliable

### Disconnect

Sent when client disconnects gracefully.

```rust
ClientMessage::Disconnect
```

Channel: Reliable

### Input

Sent every tick with player input.

```rust
ClientMessage::Input(PlayerInput)

PlayerInput {
    seq: InputSeq,      // Sequence number
    tick: Tick,         // Server tick this input is for
    move_forward: f32,  // -1.0 to 1.0
    move_right: f32,    // -1.0 to 1.0
    jump: bool,
    crouch: bool,
    attack: bool,
    yaw: f32,           // Radians
    pitch: f32,         // Radians
}
```

Channel: Unreliable

### SnapshotAck

Acknowledges receipt of server snapshot.

```rust
ClientMessage::SnapshotAck {
    tick: Tick,
}
```

Channel: Unreliable

## Server Messages

### Connected

Sent in response to successful Connect.

```rust
ServerMessage::Connected {
    player_id: PlayerId,
    tick: Tick,
    tick_rate: u8,
    arena_data: Vec<u8>,
}
```

Channel: Reliable

### Rejected

Sent when connection is rejected.

```rust
ServerMessage::Rejected {
    reason: String,
}
```

Channel: Reliable

### Kicked

Sent when player is kicked from server.

```rust
ServerMessage::Kicked {
    reason: String,
}
```

Channel: Reliable

### Snapshot

Sent every tick with world state.

```rust
ServerMessage::Snapshot(WorldSnapshot)

WorldSnapshot {
    tick: Tick,
    players: Vec<PlayerSnapshot>,
    match_state: MatchState,
}

PlayerSnapshot {
    id: PlayerId,
    position: Vec3,
    rotation: Rotation,
    velocity: Vec3,
    health: u8,
    input_ack: InputSeq,
}

MatchState {
    state: RoundState,  // Waiting, Countdown, Playing, RoundEnd
    time_remaining: u16,
    scores: Vec<(PlayerId, u16)>,
}
```

Channel: Unreliable

### Event

Sent when game events occur.

```rust
ServerMessage::Event(GameEvent)

GameEvent {
    PlayerJoined { id: PlayerId, name: String },
    PlayerLeft { id: PlayerId },
    PlayerDied { id: PlayerId, killer: Option<PlayerId> },
    PlayerRespawned { id: PlayerId, position: Vec3 },
    RoundStart { number: u16 },
    RoundEnd { winner: Option<PlayerId> },
}
```

Channel: Reliable

## Connection Flow

```
Client                          Server
  │                               │
  │──── Connect ─────────────────►│
  │                               │
  │◄──── Connected ───────────────│
  │                               │
  │──── Input ───────────────────►│
  │◄──── Snapshot ────────────────│
  │──── SnapshotAck ─────────────►│
  │                               │
  │      ... game loop ...        │
  │                               │
  │──── Disconnect ──────────────►│
  │                               │
```

## Timing

- Server tick: 60 Hz
- Client input send: 60 Hz
- Snapshot broadcast: 20-60 Hz
- Connection timeout: 10 seconds
- Keepalive interval: 1 second

## Validation

Server validates all input:
- Movement speed limits
- Attack cooldown (30 ticks)
- Attack range (2.5 blocks)
- Input rate limits
