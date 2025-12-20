# Threat Model

This document describes the security threat model for the Plix server.

## Untrusted Inputs

### 1. Network Packets

**Source**: Client connections (UDP/TCP)
**Threats**:
- Oversized packets (DoS via memory exhaustion)
- Malformed messages (decode panics)
- Invalid fields (logic errors)
- Rate flooding (CPU exhaustion)

**Mitigations**:
- Pre-decode size validation (`MAX_MESSAGE_BYTES = 64 KB`)
- Fuzz-tested decode paths
- Rate limiting (`GLOBAL_MSG_PER_SEC = 200`)
- Strike-based disconnect

### 2. Handshake Messages

**Source**: Connecting clients
**Threats**:
- Pending connection exhaustion
- Slow handshake attacks
- Cached hash spam

**Mitigations**:
- Per-source pending limit (`MAX_PENDING_PER_SOURCE = 10`)
- Handshake timeout (`HANDSHAKE_TIMEOUT_SECS = 10`)
- Cached hash limit (`MAX_CACHED_PAYLOAD_HASHES = 256`)

### 3. Payload Sync Messages

**Source**: Clients during mod sync
**Threats**:
- Invalid chunk indices
- Duplicate chunk requests
- Resend spam
- Hash mismatch attacks

**Mitigations**:
- Chunk index bounds validation
- Duplicate detection
- Resend rate limiting (`MAX_RESEND_PER_WINDOW = 16`)
- Transfer timeout (`TRANSFER_TIMEOUT_SECS = 30`)
- Hash verification with strike on mismatch

### 4. Registry Index Files

**Source**: Remote registries, local files
**Threats**:
- Oversized JSON (OOM)
- Malformed JSON (parsing issues)
- Invalid mod entries

**Mitigations**:
- Size limit (`MAX_INDEX_JSON_BYTES = 5 MB`)
- Schema validation
- Field validation (SHA-256 format, URLs, etc.)

### 5. Mod Bundle Files

**Source**: Downloads, local files
**Threats**:
- Path traversal in zip
- Zip bombs (compression ratio attack)
- Excessive file count
- Oversized bundles

**Mitigations**:
- Path validation (no `../`, no absolute paths)
- Compression ratio limit (`MAX_ZIP_RATIO = 20:1`)
- File count limit (`MAX_ZIP_FILES = 10,000`)
- Bundle size limit (`MAX_BUNDLE_BYTES = 50 MB`)

### 6. Mod Manifest Files

**Source**: mod.toml in bundles
**Threats**:
- Oversized TOML (OOM)
- Malformed TOML (parsing issues)
- Invalid field values

**Mitigations**:
- Size limit (`MAX_MOD_TOML_BYTES = 256 KB`)
- Schema validation
- Field validation (mod ID format, version format, etc.)

## Trust Boundaries

```
┌─────────────────────────────────────────────────────────────┐
│                      UNTRUSTED                               │
│                                                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐    │
│  │  Client  │  │  Client  │  │ Registry │  │  Bundle  │    │
│  │  Socket  │  │  Socket  │  │   HTTP   │  │   File   │    │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘    │
│       │             │             │             │           │
└───────┼─────────────┼─────────────┼─────────────┼───────────┘
        │             │             │             │
        ▼             ▼             ▼             ▼
┌─────────────────────────────────────────────────────────────┐
│                    SECURITY LAYER                            │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │              Size Validation                          │   │
│  │  - Pre-decode packet size check                      │   │
│  │  - Pre-parse JSON/TOML size check                    │   │
│  │  - Zip file count and ratio check                    │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │              Rate Limiting                            │   │
│  │  - Global message rate (200/sec)                     │   │
│  │  - Per-type message rate                             │   │
│  │  - Strike accumulation                               │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │              Content Validation                       │   │
│  │  - Field length limits                               │   │
│  │  - Numeric bounds checks                             │   │
│  │  - Path traversal prevention                         │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
└──────────────────────────┬──────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                       TRUSTED                                │
│                                                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐    │
│  │   Game   │  │  Match   │  │   Mod    │  │  World   │    │
│  │   Loop   │  │  State   │  │  System  │  │  State   │    │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘    │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## Attack Scenarios

### DoS via Memory Exhaustion

**Attack**: Send oversized packets to exhaust server memory.
**Defense**: Pre-decode size validation rejects packets before allocation.
**Limit**: MAX_MESSAGE_BYTES = 64 KB

### DoS via CPU Exhaustion

**Attack**: Flood server with valid messages requiring processing.
**Defense**: Token bucket rate limiting drops excess messages.
**Limit**: GLOBAL_MSG_PER_SEC = 200

### Handshake Exhaustion

**Attack**: Open many connections and never complete handshake.
**Defense**: Per-source pending limit + timeout.
**Limits**: MAX_PENDING_PER_SOURCE = 10, HANDSHAKE_TIMEOUT_SECS = 10

### Zip Bomb

**Attack**: Send bundle with extreme compression ratio.
**Defense**: Pre-extraction ratio check.
**Limit**: MAX_ZIP_RATIO = 20:1

### Path Traversal

**Attack**: Bundle contains `../../../etc/passwd`.
**Defense**: Path validation before extraction.
**Check**: No `..`, no absolute paths

## Security Metrics

Monitor these counters for attack detection:

| Metric | Normal | Alert Threshold |
|--------|--------|-----------------|
| `invalid_messages_total` | Low | >10/min from single source |
| `disconnects_strikes_total` | Rare | >5/min |
| `rate_limited_total` | Low | >100/min from single source |
| `handshake_timeouts_total` | Low | >10/min from single source |
| `zip_safety_rejections_total` | Zero | Any |
