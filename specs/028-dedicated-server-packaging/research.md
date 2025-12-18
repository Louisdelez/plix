# Research: Dedicated Server Packaging

**Feature**: 028-dedicated-server-packaging
**Date**: 2025-12-18

## Key Findings

### 1. Current Server Architecture

**Decision**: Build upon existing CLI-based configuration system
**Rationale**: plix-server already has comprehensive CLI arguments via clap 4.0, making environment variable mapping straightforward
**Alternatives considered**: New config system from scratch - rejected because existing CLI args cover all needed settings

**Current CLI Arguments (plix-server)**:
- `--port <PORT>` (default: 7777, not 12345 as assumed in spec)
- `--tickrate <RATE>` (default: 60, range: 20-60)
- `--max-players <N>` (default: 16)
- `--arena <NAME>` (default: test_arena)
- `--assets-dir <PATH>` (default: assets)
- `--log-level <LEVEL>` (default: info)
- `--persistence <bool>` (default: false)
- `--world-id <ID>` (optional)
- `--autosave-interval <SECS>` (default: 300)
- `--master-url <URL>` (optional)
- `--server-name <NAME>` (default: "Plix Server")
- `--region <REGION>` (default: unknown)
- `--tags <TAGS>` (comma-separated)
- `--game-modes <MODES>` (comma-separated, default: ffa)

### 2. Network Ports

**Decision**: Use UDP 7777 for game server, TCP 8080 for master server
**Rationale**: These are the actual default ports in the codebase, not UDP 12345 as assumed
**Note**: Spec assumption of port 12345 was incorrect - actual default is 7777

**Ports**:
- Game Server: UDP 7777 (configurable via `--port`)
- Master Server: HTTP 8080 (configurable via `--bind`)

### 3. Docker Base Image

**Decision**: Use `debian:bookworm-slim` with specific digest for reproducibility
**Rationale**:
- Minimal size (~80MB)
- Stable Debian release
- Good compatibility with Rust binaries
- Widely used, well-supported
**Alternatives considered**:
- `alpine:3.19` - smaller but musl libc compatibility issues with some Rust crates
- `ubuntu:24.04` - larger, no significant benefit
- `distroless/cc` - good but harder to debug

### 4. Rust Toolchain Version

**Decision**: Pin to Rust 1.75.0 via rust-toolchain.toml
**Rationale**:
- Constitution requires stable Rust only
- 1.75.0 is stable and widely tested
- Matches existing codebase requirements
**Format**:
```toml
[toolchain]
channel = "1.75.0"
profile = "minimal"
components = ["rustfmt", "clippy"]
```

### 5. Configuration File Format

**Decision**: TOML configuration file (server.toml) with env/CLI override
**Rationale**:
- TOML is already used for arena configs
- Human-readable and widely supported
- serde + toml already in dependencies
**Structure**: See data-model.md for schema

### 6. Data Directory Structure

**Decision**: Use `/data` as container root with subdirectories
**Rationale**: Standard Docker convention, single mount point option
**Structure**:
```
/data/
├── config/     # server.toml, custom arena configs
├── worlds/     # persisted world data (Feature 014)
└── logs/       # optional file logs (stdout preferred)
```

### 7. Non-Root User

**Decision**: Create `plix` user (UID 1000) in container
**Rationale**:
- Security best practice
- Matches common host user UIDs for volume permissions
- Constitution requires security-first approach

### 8. Build Reproducibility

**Decision**: Multi-stage Dockerfile with locked dependencies
**Rationale**:
- Cargo.lock already exists and is committed
- rust-toolchain.toml pins exact version
- Base image pinned by digest
**Approach**:
- Stage 1 (builder): debian:bookworm-slim with Rust toolchain
- Stage 2 (runtime): debian:bookworm-slim (same base)
- Copy only binary and assets

### 9. Healthcheck Strategy

**Decision**: Process-based healthcheck (pgrep plix-server)
**Rationale**:
- No HTTP endpoint on game server
- Simple and reliable
- Meets Docker healthcheck requirements
**Alternative considered**: UDP ping - too complex for MVP

### 10. Logging Strategy

**Decision**: stdout/stderr only, delegate rotation to Docker/host
**Rationale**:
- 12-factor app principle
- Docker handles log rotation via logging drivers
- Simplifies container design
- Clarified in spec session

### 11. Master Server Integration

**Decision**: Optional plix-master service in docker-compose
**Rationale**:
- Server browser functionality exists (Feature 026)
- plix-master already implemented with HTTP API
- Game server can run standalone without master

### 12. Existing Scripts Location

**Decision**: Place new Docker scripts in `deploy/` directory
**Rationale**:
- Separate from existing `scripts/` (development scripts)
- Clear purpose distinction
- Standard convention for deployment artifacts
**Structure**:
```
deploy/
├── docker/
│   ├── Dockerfile
│   ├── Dockerfile.master
│   └── docker-compose.yml
├── scripts/
│   ├── build.sh
│   ├── run.sh
│   ├── compose.sh
│   └── release-local.sh
└── config/
    └── server.toml.example
```

## Spec Corrections Required

1. **Port number**: Spec assumed UDP 12345, actual default is UDP 7777
2. **Configuration priority**: Already exists in code, no new implementation needed for CLI/env, only config file loading needs addition

## Dependencies Verified

- `toml` 0.8 - already in workspace
- `serde` 1.0 - already in workspace
- `dirs-next` 2.0 - already in workspace
- `tracing` 0.1 - already in workspace
- `clap` 4.0 - already in workspace

## No New Rust Dependencies Required

All configuration and CLI functionality can be built with existing dependencies.
