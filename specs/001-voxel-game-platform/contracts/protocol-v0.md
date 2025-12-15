# Plix Network Protocol v0

**Version**: 0.1.0
**Status**: Draft
**Date**: 2025-12-14

## Overview

This document specifies the network protocol for Plix MVP v0.1. The protocol operates over UDP with a custom reliability layer for messages that require guaranteed delivery.

## Transport Layer

### Packet Structure

All packets follow this structure:

```
┌─────────────────────────────────────────────────────────┐
│ Header (1 byte)                                         │
├─────────────────────────────────────────────────────────┤
│ Sequence Number (2 bytes, big-endian)                   │
├─────────────────────────────────────────────────────────┤
│ Ack Number (2 bytes, big-endian)                        │
├─────────────────────────────────────────────────────────┤
│ Ack Bits (4 bytes, big-endian)                          │
├─────────────────────────────────────────────────────────┤
│ Payload (variable, max 1389 bytes)                      │
└─────────────────────────────────────────────────────────┘
```

**Total header size**: 9 bytes
**Max payload size**: 1389 bytes (MTU 1400 - header - safety margin)

### Header Byte

```
┌───┬───┬───┬───┬───┬───┬───┬───┐
│ 7 │ 6 │ 5 │ 4 │ 3 │ 2 │ 1 │ 0 │
├───┴───┴───┴───┴───┴───┴───┴───┤
│ Version │ Channel │ Reserved  │
│ (2 bits)│ (2 bits)│ (4 bits)  │
└───────────────────────────────┘
```

**Version**: Protocol version (0-3, current = 0)
**Channel**:
- 0 = Unreliable (no resend, no ordering)
- 1 = Reliable-unordered (resend until ack, no ordering)
- 2 = Reliable-ordered (resend until ack, in-order delivery)
- 3 = Reserved

### Sequence & Acknowledgment

- **Sequence Number**: Increments per packet sent (wraps at 65535)
- **Ack Number**: Last received sequence number
- **Ack Bits**: Bitmap of previous 32 packets received (bit 0 = ack-1, bit 31 = ack-32)

### Reliability Mechanism

**Reliable channels**:
1. Sender tracks unacknowledged packets
2. If no ack received within RTT × 1.5 (min 100ms), resend
3. Max resend attempts: 10
4. Connection terminated after 10 failures

**Unreliable channel**:
- No resend, no ordering guarantee
- Receiver processes immediately or drops if stale

---

## Connection Lifecycle

### Handshake (3-way)

```
Client                          Server
  │                                │
  │ ──── Connect ─────────────────→│
  │      (reliable-ordered)        │
  │                                │
  │ ←─── Connected/Rejected ────── │
  │      (reliable-ordered)        │
  │                                │
  │ ──── SnapshotAck ─────────────→│
  │      (first snapshot ack)      │
  │                                │
  │        [Connected]             │
```

### Keepalive

- Client sends input every tick (acts as keepalive)
- If no packet received for 5 seconds, send explicit keepalive
- If no response for 10 seconds, disconnect

### Disconnection

**Graceful**:
```
Client                          Server
  │                                │
  │ ──── Disconnect ──────────────→│
  │      (reliable-ordered)        │
  │                                │
  │        [Close socket]          │
```

**Timeout**:
- No packets for 10 seconds → connection dropped
- Server broadcasts PlayerLeft event

---

## Message Types

### Client → Server Messages

#### Connect (0x01)

```
┌────────────────────────────────────────┐
│ Type: 0x01 (1 byte)                    │
├────────────────────────────────────────┤
│ Protocol Version: u8                   │
├────────────────────────────────────────┤
│ Name Length: u8                        │
├────────────────────────────────────────┤
│ Name: UTF-8 (variable, max 32 bytes)   │
└────────────────────────────────────────┘
```

**Channel**: Reliable-ordered
**Response**: Connected or Rejected

#### Disconnect (0x02)

```
┌────────────────────────────────────────┐
│ Type: 0x02 (1 byte)                    │
└────────────────────────────────────────┘
```

**Channel**: Reliable-ordered
**Response**: None (connection closed)

#### Input (0x10)

```
┌────────────────────────────────────────┐
│ Type: 0x10 (1 byte)                    │
├────────────────────────────────────────┤
│ Sequence: u16                          │
├────────────────────────────────────────┤
│ Tick: u32                              │
├────────────────────────────────────────┤
│ Move Forward: i8 (-127 to 127)         │
├────────────────────────────────────────┤
│ Move Right: i8 (-127 to 127)           │
├────────────────────────────────────────┤
│ Flags: u8                              │
│   bit 0: jump                          │
│   bit 1: crouch                        │
│   bit 2: attack                        │
├────────────────────────────────────────┤
│ Yaw: i16 (angle × 100)                 │
├────────────────────────────────────────┤
│ Pitch: i16 (angle × 100)               │
└────────────────────────────────────────┘
```

**Size**: 14 bytes
**Channel**: Unreliable
**Rate**: Every client tick (60 Hz)

#### SnapshotAck (0x11)

```
┌────────────────────────────────────────┐
│ Type: 0x11 (1 byte)                    │
├────────────────────────────────────────┤
│ Tick: u32                              │
└────────────────────────────────────────┘
```

**Size**: 5 bytes
**Channel**: Unreliable
**Purpose**: Acknowledge snapshot receipt, enable delta encoding

---

### Server → Client Messages

#### Connected (0x81)

```
┌────────────────────────────────────────┐
│ Type: 0x81 (1 byte)                    │
├────────────────────────────────────────┤
│ Player ID: u16                         │
├────────────────────────────────────────┤
│ Current Tick: u32                      │
├────────────────────────────────────────┤
│ Tick Rate: u8                          │
├────────────────────────────────────────┤
│ Arena Data Length: u32                 │
├────────────────────────────────────────┤
│ Arena Data: compressed bytes           │
└────────────────────────────────────────┘
```

**Channel**: Reliable-ordered

#### Rejected (0x82)

```
┌────────────────────────────────────────┐
│ Type: 0x82 (1 byte)                    │
├────────────────────────────────────────┤
│ Reason Length: u8                      │
├────────────────────────────────────────┤
│ Reason: UTF-8 (variable, max 255)      │
└────────────────────────────────────────┘
```

**Channel**: Reliable-ordered
**Reasons**: "Server full", "Protocol mismatch", "Banned"

#### Kicked (0x83)

```
┌────────────────────────────────────────┐
│ Type: 0x83 (1 byte)                    │
├────────────────────────────────────────┤
│ Reason Length: u8                      │
├────────────────────────────────────────┤
│ Reason: UTF-8 (variable, max 255)      │
└────────────────────────────────────────┘
```

**Channel**: Reliable-ordered

#### Snapshot (0x90)

```
┌────────────────────────────────────────┐
│ Type: 0x90 (1 byte)                    │
├────────────────────────────────────────┤
│ Tick: u32                              │
├────────────────────────────────────────┤
│ Last Input Seq: u16                    │
├────────────────────────────────────────┤
│ Player Count: u8                       │
├────────────────────────────────────────┤
│ Players: [PlayerSnapshot]              │
├────────────────────────────────────────┤
│ Match State: MatchState                │
└────────────────────────────────────────┘
```

**PlayerSnapshot** (26 bytes each):
```
┌────────────────────────────────────────┐
│ Player ID: u16                         │
├────────────────────────────────────────┤
│ Position X: f32                        │
├────────────────────────────────────────┤
│ Position Y: f32                        │
├────────────────────────────────────────┤
│ Position Z: f32                        │
├────────────────────────────────────────┤
│ Yaw: i16 (angle × 100)                 │
├────────────────────────────────────────┤
│ Pitch: i16 (angle × 100)               │
├────────────────────────────────────────┤
│ Health: u8                             │
├────────────────────────────────────────┤
│ Flags: u8                              │
│   bit 0: is_dead                       │
│   bits 1-3: animation (0-7)            │
└────────────────────────────────────────┘
```

**MatchState**:
```
┌────────────────────────────────────────┐
│ Phase: u8                              │
├────────────────────────────────────────┤
│ Round Number: u16                      │
├────────────────────────────────────────┤
│ Round Time Remaining: u16 (seconds)    │
├────────────────────────────────────────┤
│ Team Count: u8                         │
├────────────────────────────────────────┤
│ Teams: [TeamScore]                     │
│   Team ID: u8                          │
│   Score: u32                           │
└────────────────────────────────────────┘
```

**Channel**: Unreliable
**Rate**: 20-30 Hz

#### Event (0x91)

```
┌────────────────────────────────────────┐
│ Type: 0x91 (1 byte)                    │
├────────────────────────────────────────┤
│ Event Type: u8                         │
├────────────────────────────────────────┤
│ Event Data: variable                   │
└────────────────────────────────────────┘
```

**Event Types**:
| Code | Event | Data |
|------|-------|------|
| 0x01 | PlayerJoined | player_id: u16, name_len: u8, name: UTF-8, team: u8 |
| 0x02 | PlayerLeft | player_id: u16 |
| 0x03 | PlayerDied | victim_id: u16, killer_id: u16 (0xFFFF = no killer) |
| 0x04 | PlayerRespawned | player_id: u16 |
| 0x10 | RoundStart | round: u16 |
| 0x11 | RoundEnd | winner_team: u8 (0xFF = draw) |
| 0x12 | MatchEnd | winner_team: u8 (0xFF = draw) |

**Channel**: Reliable-unordered

---

## Bandwidth Estimates

### Per Client (16 players)

**Client → Server**:
- Input: 14 bytes × 60 Hz = 840 bytes/sec = 6.7 kbps

**Server → Client**:
- Snapshot: (7 + 26×16 + 15) = 438 bytes × 30 Hz = 13,140 bytes/sec = 105 kbps
- Events: ~100 bytes/sec average = 0.8 kbps

**Total per client**: ~113 kbps

### Server Total (16 players)

- Incoming: 16 × 6.7 kbps = 107 kbps
- Outgoing: 16 × 113 kbps = 1.8 Mbps

---

## Version Compatibility

| Client Version | Server Version | Compatible |
|----------------|----------------|------------|
| 0.x.x | 0.x.x | Yes (MVP) |
| 0.x.x | 1.x.x | No |

**Rule**: Major version must match. Minor/patch differences allowed within major.

---

## Security Considerations

1. **No client trust**: Server validates all inputs
2. **Rate limiting**: Max 120 messages/sec per client
3. **Size limits**: Max 1400 bytes per packet, reject larger
4. **Sequence validation**: Reject out-of-order reliable messages
5. **Timeout**: Disconnect silent clients after 10 seconds
