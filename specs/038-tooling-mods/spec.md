# Feature Specification: Tooling Mods (SDK, Templates, CLI, Hot-Reload)

**Feature Branch**: `038-tooling-mods`
**Created**: 2025-12-19
**Status**: Draft
**Input**: Improve mod developer experience (DX) with SDK, templates, CLI tooling, documentation, and optional hot-reload for rapid iteration

## Clarifications

### Session 2025-12-19

- Q: What is the maximum mod bundle size for validation? → A: 10 MB maximum bundle size

## Overview

This feature provides comprehensive tooling to drastically improve the mod developer experience (DX) for the Plix modding ecosystem. It enables modders to create, build, package, and test mods efficiently while maintaining production security guarantees.

**Core Principles**:
- Production security remains intact (hot-reload and unsigned bundles are dev-only features)
- SDK API stability follows ABI/API versioning
- "Happy path" goal: create, compile, package, load a mod in under 5 minutes

**Dependencies**:
- Feature 034: plix-mod-core (API, capabilities, events)
- Feature 035: WASM runtime + ABI v1
- Feature 036: Bundle format (.plixmod), registry, lockfile
- Feature 037: Server-only mods + client sync

## User Scenarios & Testing

### User Story 1 - Create and Load First Mod (Priority: P1)

As a new modder, I can create a mod from a template, compile it to WASM, package it as `.plixmod`, and load it on a local server in under 5 minutes.

**Why this priority**: This is the core "happy path" that validates the entire tooling chain. Without this working, no modder can get started.

**Independent Test**: Run `plix mod new`, build, pack, copy to server mods folder, start server - mod loads and executes.

**Acceptance Scenarios**:

1. **Given** the modding toolchain is installed, **When** I run `plix mod new my-mod --template chat-filter`, **Then** a complete mod project is scaffolded with manifest, source code, and build scripts
2. **Given** a scaffolded mod project, **When** I run `plix mod build`, **Then** the mod compiles to `mod.wasm` targeting `wasm32-unknown-unknown`
3. **Given** a compiled mod, **When** I run `plix mod pack`, **Then** a deterministic `.plixmod` bundle is created with stable SHA-256 hash
4. **Given** a packaged mod bundle, **When** I copy it to the server mods directory and start the server, **Then** the mod loads and executes (visible in logs)

---

### User Story 2 - Use SDK for Safe Host Interactions (Priority: P1)

As a modder, I can use the SDK's ergonomic wrappers instead of raw ABI calls to interact with the game engine safely.

**Why this priority**: The SDK is what enables modders to write code without understanding low-level ABI details. Essential for adoption.

**Independent Test**: Write a mod using SDK functions (log, subscribe, world query), compile and run - all calls succeed.

**Acceptance Scenarios**:

1. **Given** I'm writing mod code, **When** I use `plix_mod_sdk::log("message")`, **Then** the message appears in server logs
2. **Given** I want to handle game events, **When** I use `#[on_event("on_player_chat")]` macro, **Then** my handler is correctly registered and called
3. **Given** a host call fails, **When** the SDK wrapper returns an error, **Then** the error contains meaningful information (EMOD error code, description)
4. **Given** SDK version mismatch with runtime, **When** the mod initializes, **Then** a warning is logged but execution continues (graceful degradation)

---

### User Story 3 - Validate Mod Before Distribution (Priority: P2)

As a modder, I can validate my mod bundle to catch issues (missing exports, invalid manifest, size limits) before sharing it.

**Why this priority**: Validation prevents frustrating runtime failures and improves mod quality across the ecosystem.

**Independent Test**: Run `plix mod validate` on a mod bundle - get clear pass/fail results with actionable messages.

**Acceptance Scenarios**:

1. **Given** a valid mod bundle, **When** I run `plix mod validate`, **Then** validation passes with "OK" status
2. **Given** a mod missing `mod_init` export, **When** I run validation, **Then** it fails with "Missing required export: mod_init"
3. **Given** a mod exceeding 10 MB, **When** I run validation, **Then** it fails with "Bundle size exceeds 10 MB limit"
4. **Given** a malformed manifest, **When** I run validation, **Then** it fails with specific parsing errors

---

### User Story 4 - Iterate Quickly with Hot-Reload (Priority: P3)

As a modder in development mode, I can have my mod automatically reloaded when I rebuild it, avoiding server restarts.

**Why this priority**: Hot-reload is a nice-to-have DX improvement but not required for modding to work. Marked as "if possible" in requirements.

**Independent Test**: Enable hot-reload in dev config, modify mod code, rebuild - mod reloads without server restart.

**Acceptance Scenarios**:

1. **Given** hot-reload is enabled (`mods.dev.hot_reload=true`), **When** I rebuild a mod bundle, **Then** the server detects the change and reloads the mod
2. **Given** hot-reload is triggered, **When** the mod reloads, **Then** `mod_shutdown` is called on old instance and `mod_init` on new
3. **Given** a reload fails (bad WASM), **When** the error is detected, **Then** the previous mod version remains active (fallback)
4. **Given** hot-reload is disabled (default), **When** mod files change, **Then** no automatic reload occurs

---

### User Story 5 - Learn Modding from Documentation (Priority: P2)

As a new modder, I can follow comprehensive documentation to understand the SDK, distribution system, and troubleshoot common issues.

**Why this priority**: Good documentation reduces support burden and enables self-service learning.

**Independent Test**: Follow quickstart guide end-to-end - successfully create and run a mod without external help.

**Acceptance Scenarios**:

1. **Given** I'm new to modding, **When** I follow `docs/modding/quickstart.md`, **Then** I can complete the full create-build-pack-run cycle
2. **Given** I need API details, **When** I read `docs/modding/sdk.md`, **Then** I find all host functions documented with examples
3. **Given** my mod fails to load, **When** I consult `docs/modding/troubleshooting.md`, **Then** I find my error code and resolution steps

---

### Edge Cases

- What happens when a mod is compiled with an incompatible SDK version? Warning logged, graceful degradation or clear failure message
- What happens when hot-reload is attempted with players connected? Policy-configurable: either warn and proceed or block reload
- What happens when packing the same source twice? Identical hash (deterministic bundling)
- What happens when a template is requested that doesn't exist? Clear error with list of available templates
- What happens when `plix mod build` fails due to missing Rust toolchain? Clear error with installation instructions

## Requirements

### Functional Requirements

**SDK (plix-mod-sdk crate)**:

- **FR-001**: SDK MUST provide stable type definitions (ModId, EventId, capability IDs, event payloads)
- **FR-002**: SDK MUST provide safe wrappers for all host functions (log, subscribe, world, entities, net, timers)
- **FR-003**: SDK MUST map EMOD error codes to ergonomic `Result<T, ModError>` types
- **FR-004**: SDK MUST provide attribute macros for mod entry points (`#[mod_init]`, `#[mod_shutdown]`, `#[on_event]`)
- **FR-005**: SDK MUST compile to `wasm32-unknown-unknown` target without OS-specific dependencies
- **FR-006**: SDK MUST expose version constants (`sdk_abi_version`, `sdk_api_version`) for compatibility checking

**Templates**:

- **FR-007**: System MUST provide at least 2 starter templates (chat-filter, world-query)
- **FR-008**: Each template MUST include complete `mod.toml`, Rust source with SDK usage, and build scripts
- **FR-009**: Templates MUST compile and run successfully against the current runtime

**CLI Tooling (plix mod)**:

- **FR-010**: CLI MUST support `plix mod new <name> --template <template>` to scaffold projects
- **FR-011**: CLI MUST support `plix mod build` to compile WASM
- **FR-012**: CLI MUST support `plix mod pack` to create deterministic `.plixmod` bundles
- **FR-013**: CLI MUST support `plix mod validate` to check bundle integrity (exports, manifest, size ≤ 10 MB)
- **FR-014**: CLI MUST support `plix mod install --local <path>` to install mods to local dev cache
- **FR-015**: Pack command MUST produce identical hashes for identical inputs (deterministic)

**Hot-Reload (dev-only)**:

- **FR-016**: Hot-reload MUST be disabled by default (`mods.dev.hot_reload=false`)
- **FR-017**: When enabled, system MUST watch mod directories for changes
- **FR-018**: On change detection, system MUST call `mod_shutdown`, reload WASM, call `mod_init`
- **FR-019**: On reload failure, system MUST fall back to previous version OR disable mod (configurable)
- **FR-020**: Hot-reload MUST log all reload attempts (start, success, failure with reason)

**Documentation**:

- **FR-021**: System MUST provide quickstart documentation covering full mod lifecycle
- **FR-022**: System MUST provide SDK API reference with all host functions and error codes
- **FR-023**: System MUST provide troubleshooting guide for common errors (ABI mismatch, join refused, hash mismatch)

**Security & Validation**:

- **FR-024**: Tooling MUST validate required WASM exports are present (mod_init, mod_on_event, mod_shutdown)
- **FR-025**: Tooling MUST validate manifest capabilities are coherent with declared permissions
- **FR-026**: Tooling MUST reject bundles exceeding 10 MB with clear error message
- **FR-027**: Dev mode MAY allow `--unsigned` flag for local testing only
- **FR-028**: Unsigned/dev-mode features MUST NOT be usable in production configurations

### Key Entities

- **ModProject**: A mod development workspace containing manifest, source, and build configuration
- **ModBundle**: A packaged `.plixmod` file containing manifest, WASM binary, and optional assets
- **Template**: A pre-configured mod scaffold (chat-filter, world-query, timers-net)
- **DevConfig**: Server-side development configuration controlling hot-reload and dev features
- **SDKVersion**: Version identifier for SDK/ABI/API compatibility tracking

## Success Criteria

### Measurable Outcomes

- **SC-001**: A modder can complete the full create-build-pack-load cycle in under 5 minutes
- **SC-002**: SDK wrappers cover 100% of Feature 034 host functions with ergonomic APIs
- **SC-003**: Pack command produces identical SHA-256 hashes when run twice on same inputs
- **SC-004**: Validation catches 100% of known invalid configurations (missing exports, bad manifest, bundles > 10 MB)
- **SC-005**: At least 2 templates compile successfully and execute on the runtime
- **SC-006**: Quickstart documentation enables a new modder to create their first mod without external assistance
- **SC-007**: Hot-reload (when enabled) detects file changes within 500ms and completes reload within 2 seconds
- **SC-008**: Zero dev-only features (hot-reload, unsigned) are accessible in production mode

## Assumptions

- Modders have Rust toolchain installed with `wasm32-unknown-unknown` target
- Features 034, 035, 036, and 037 are complete and stable
- CLI can be implemented as subcommands of main `plix` binary or as standalone `plix-mod` tool
- Templates are stored in the repository under `templates/mods/` directory
- Hot-reload uses filesystem watching with configurable debounce (default 200ms)

## Out of Scope

- Public marketplace / authenticated publishing (future feature)
- Advanced WASM debugger
- Client-side mod runtime
- Hot-reload in production without restrictions
- Multi-language support (JavaScript, Lua) - Rust-only for now
