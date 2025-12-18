# Feature Specification: Dedicated Server Packaging

**Feature Branch**: `028-dedicated-server-packaging`
**Created**: 2025-12-18
**Status**: Draft
**Input**: User description: "Feature 028 – Dedicated Server Packaging: Make plix-server easily deployable and reproducible (local, VPS, CI) via Docker, scripts, and documentation, with a version pinning strategy (Rust toolchain, dependencies, base image) for stable and predictable builds."

## Clarifications

### Session 2025-12-18

- Q: What should be the default game mode when no configuration is provided? → A: FFA (Free-For-All) - simpler, works with any player count
- Q: How should log growth be managed for long-running servers? → A: No built-in rotation - delegate to Docker logging driver or host tools (logrotate)

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Quick Server Deployment (Priority: P1)

As a server administrator, I want to deploy a plix game server in 2 commands so that I can quickly host games for my community without complex setup procedures.

**Why this priority**: This is the core value proposition - reducing deployment friction from hours to minutes. Without this, the feature provides no benefit.

**Independent Test**: Can be fully tested by running `docker pull` and `docker run` commands and verifying a functional game server is accessible. Delivers immediate value by enabling any admin to host a server.

**Acceptance Scenarios**:

1. **Given** Docker is installed on a Linux x86_64 system, **When** the admin runs `docker run plix-server`, **Then** a functional game server starts and accepts player connections within 30 seconds
2. **Given** a running Docker container, **When** players attempt to connect, **Then** they can join and play without additional configuration
3. **Given** the default configuration, **When** the server starts, **Then** it runs in FFA (Free-For-All) mode on the default arena

---

### User Story 2 - Runtime Configuration (Priority: P1)

As a server administrator, I want to configure server name, region, game mode, and player limits without recompiling so that I can customize my server for different events and communities.

**Why this priority**: Configuration flexibility is essential for real-world usage. A server that cannot be customized has limited practical value.

**Independent Test**: Can be tested by starting a server with different environment variables or config file settings and verifying the changes take effect.

**Acceptance Scenarios**:

1. **Given** a configuration file with server name "My Arena", **When** the server starts, **Then** the server identifies itself as "My Arena" to players and master server
2. **Given** environment variable `PLIX_MAX_PLAYERS=16`, **When** the server starts, **Then** the server limits connections to 16 players
3. **Given** a CLI flag `--mode ctf`, **When** the server starts, **Then** the server runs in Capture The Flag mode
4. **Given** conflicting configuration (CLI flag and env var for same setting), **When** the server starts, **Then** CLI takes precedence over env var, which takes precedence over config file

---

### User Story 3 - Reproducible Builds (Priority: P2)

As a developer, I want builds to be reproducible across machines and time so that I can debug production issues and ensure consistent behavior.

**Why this priority**: Reproducibility enables reliable debugging and consistent deployments but is not required for basic server operation.

**Independent Test**: Can be tested by building the Docker image on two different machines and verifying identical binary checksums.

**Acceptance Scenarios**:

1. **Given** the same source code and locked dependencies, **When** building on different machines, **Then** the resulting binaries have identical checksums
2. **Given** a specific git commit, **When** building months later, **Then** the build succeeds with the same Rust toolchain version
3. **Given** the Cargo.lock file, **When** dependencies are fetched, **Then** exact versions are used without unexpected updates

---

### User Story 4 - Multi-Service Stack (Priority: P2)

As a DevOps engineer, I want to deploy both game server and master server together so that I can run a complete game infrastructure with server browser functionality.

**Why this priority**: The multi-service stack enables the full ecosystem but individual servers can function without it.

**Independent Test**: Can be tested by running `docker-compose up` and verifying both services start and communicate.

**Acceptance Scenarios**:

1. **Given** a docker-compose.yml file, **When** running `docker-compose up`, **Then** both plix-server and plix-master start and communicate
2. **Given** the composed stack, **When** the game server starts, **Then** it registers with the master server automatically
3. **Given** the composed stack, **When** checking the server browser, **Then** the game server appears in the list

---

### User Story 5 - Data Persistence (Priority: P3)

As a server administrator, I want world data and logs to persist across container restarts so that I can maintain game progress and troubleshoot issues.

**Why this priority**: Persistence enhances the experience but servers can function without it for casual/temporary deployments.

**Independent Test**: Can be tested by starting a server with mounted volumes, making changes, restarting, and verifying data persists.

**Acceptance Scenarios**:

1. **Given** a volume mounted to `/data/worlds`, **When** the container restarts, **Then** world modifications persist
2. **Given** a volume mounted to `/data/logs`, **When** viewing logs after restart, **Then** historical logs are available
3. **Given** no volumes mounted, **When** the server runs, **Then** it functions normally with ephemeral data

---

### User Story 6 - Non-Docker Deployment (Priority: P3)

As a server administrator without Docker, I want to deploy from a release archive so that I can run the server on systems where Docker is unavailable or impractical.

**Why this priority**: Provides flexibility for environments without Docker but is not the primary deployment path.

**Independent Test**: Can be tested by extracting the release archive and running the binary directly.

**Acceptance Scenarios**:

1. **Given** a release archive (tar.gz), **When** extracted on a compatible Linux system, **Then** the server binary runs without additional dependencies
2. **Given** the release archive, **When** checking the checksum, **Then** it matches the published checksum for integrity verification
3. **Given** the extracted binary and a config file, **When** the server starts, **Then** it reads configuration from the file

---

### Edge Cases

- What happens when required ports are already in use?
  - Server logs a clear error message and exits with non-zero status
- What happens when the configuration file has invalid TOML syntax?
  - Server logs the parsing error with line number and exits with non-zero status
- What happens when an environment variable has an invalid value?
  - Server logs which variable is invalid and expected format, then exits
- How does the system handle running out of disk space for logs?
  - Server continues running but logs a warning about failed log writes
  - Log rotation is delegated to Docker logging drivers or host tools (not built-in)
- What happens when the master server is unreachable?
  - Game server continues functioning; retries registration periodically

## Requirements *(mandatory)*

### Functional Requirements

#### Docker Image

- **FR-001**: System MUST provide a multi-stage Dockerfile that produces a minimal runtime image
- **FR-002**: System MUST expose the game port (UDP 7777) by default
- **FR-003**: System MUST start the game server with a single `docker run` command
- **FR-004**: System MUST support passing configuration via environment variables prefixed with `PLIX_`
- **FR-005**: System MUST support passing configuration via CLI flags to the container

#### Configuration

- **FR-006**: System MUST support a TOML configuration file for server settings
- **FR-007**: Configuration MUST include: server name, region, tags, max players, game mode, arena/map
- **FR-008**: Configuration MUST include master server settings: URL and advertise enabled flag
- **FR-009**: System MUST apply configuration priority: CLI flags > environment variables > config file > defaults
- **FR-010**: System MUST validate configuration on startup and report errors clearly

#### Data Management

- **FR-011**: System MUST use `/data` as the container data directory root
- **FR-012**: System MUST organize data into subdirectories: `worlds/`, `logs/`, `config/`
- **FR-013**: System MUST function without persistent volumes (ephemeral mode)
- **FR-014**: Container MUST run as non-root user by default

#### Docker Compose

- **FR-015**: System MUST provide a docker-compose.yml for plix-server
- **FR-016**: Docker Compose MUST optionally include plix-master service
- **FR-017**: Docker Compose MUST document all ports, volumes, and environment variables

#### Scripts and Packaging

- **FR-018**: System MUST provide scripts to build the Docker image
- **FR-019**: System MUST provide scripts to run server and compose stack
- **FR-020**: System MUST provide a release script that creates: binary, checksum file, and tar.gz archive
- **FR-021**: Scripts MUST work on Linux x86_64 systems with bash

#### Version Pinning

- **FR-022**: System MUST pin Rust toolchain version via rust-toolchain.toml
- **FR-023**: System MUST pin Docker base image version (not use :latest)
- **FR-024**: System MUST use Cargo.lock to pin dependency versions

#### Observability

- **FR-025**: System MUST output logs to stdout/stderr in Docker-friendly format
- **FR-026**: System MUST support RUST_LOG environment variable for log level control
- **FR-027**: Docker image MAY include a healthcheck (process-based if no HTTP endpoint)

#### Documentation

- **FR-028**: System MUST provide dedicated server documentation covering all deployment methods
- **FR-029**: Documentation MUST include troubleshooting guide for common issues (ports, NAT, logs)
- **FR-030**: Documentation MUST include configuration examples for TDM, CTF, and BR Lite modes
- **FR-031**: Documentation MUST explain log management delegation (Docker logging drivers, host-side logrotate)

### Key Entities

- **Server Configuration**: Settings controlling server behavior (name, region, mode, limits, master registration)
- **Data Directory Structure**: Organized paths for persistent data (worlds, logs, config)
- **Release Archive**: Distributable package containing binary, assets, and checksums
- **Docker Image**: Containerized server runtime with all dependencies

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Server administrators can deploy a functional game server in 2 commands or less
- **SC-002**: Server starts and accepts connections within 30 seconds of container start
- **SC-003**: Configuration changes take effect without rebuilding the Docker image
- **SC-004**: Runtime Docker image size is under 100MB (minimal footprint)
- **SC-005**: Builds from the same source produce identical binary checksums
- **SC-006**: Documentation enables a new administrator to deploy successfully on first attempt
- **SC-007**: All advertised game modes (TDM, CTF, FFA, BR Lite) are configurable via settings
- **SC-008**: Server operates correctly without root privileges inside the container
- **SC-009**: Data persists across container restarts when volumes are mounted
- **SC-010**: Release archive can be deployed on a fresh Linux x86_64 system without Docker

## Scope

### In Scope

- Docker image for plix-server (Linux x86_64)
- Docker Compose for server + optional master
- TOML configuration file with environment variable and CLI override support
- Bash scripts for building, running, and packaging
- rust-toolchain.toml for Rust version pinning
- Dedicated server documentation
- Non-Docker release archive (tar.gz with checksum)

### Out of Scope

- Kubernetes/Helm charts
- Automatic Docker Hub publishing
- Prometheus/Grafana observability stack
- Auto-scaling infrastructure
- ARM64 support (future consideration)
- Windows container support
- GUI-based server management

## Assumptions

- Docker Engine 20.10+ is available on target deployment systems
- Linux x86_64 is the primary target platform
- UDP port 7777 is the game server port (verified from existing codebase)
- plix-master exists and provides server browser functionality (Feature 026)
- World persistence (Feature 014) is optional and disabled by default
- Target administrators have basic familiarity with Docker and command-line tools

## Dependencies

- Feature 014 (World Persistence): Optional integration for persistent worlds
- Feature 026 (Server Browser): Master server for server registration
- Feature 027 (Matchmaking): Master server integration
- Existing game modes: TDM, CTF, FFA, BR Lite must be configurable
