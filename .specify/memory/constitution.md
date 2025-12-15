<!--
SYNC IMPACT REPORT
==================
Version change: 0.0.0 → 1.0.0
Bump rationale: Initial constitution creation (MAJOR - new governance document)

Modified principles: N/A (initial creation)
Added sections:
  - 10 Core Principles (Security, Performance, Architecture, Modding, Code Quality,
    Technical Standards, Player Experience, Open Source & Governance, Scoping & Realism,
    Long-term Vision)
  - Technical Standards section
  - Development Workflow section
  - Governance section

Removed sections: N/A (initial creation)

Templates requiring updates:
  - .specify/templates/plan-template.md: ✅ No updates required (Constitution Check section
    is generic and will reference this constitution)
  - .specify/templates/spec-template.md: ✅ No updates required (template is generic)
  - .specify/templates/tasks-template.md: ✅ No updates required (template is generic)

Follow-up TODOs: None
==================
-->

# Plix Constitution

## Core Principles

### I. Security (Server Authority & Isolation)

The server is the single source of truth. The client MUST never be trusted for game state.

- **Server Authoritative Architecture**: All game state, physics, and gameplay logic MUST be
  validated and controlled by the server. Client inputs are suggestions only.
- **Mod Sandboxing**: All mods MUST execute in isolated environments (WASM or script VM).
  No mod may access system resources directly.
- **Code Signing**: No unsigned or unvalidated code may execute on the client.
- **Resource Limits**: Each mod MUST operate within strict memory and CPU time budgets.
  The engine MUST terminate mods that exceed limits.
- **Attack Surface Reduction**: Client, server, and UI MUST be strictly separated.
  No component may directly access another's internal state.
- **Anti-Cheat Baseline**: The engine MUST detect and reject basic cheats (speedhack, fly,
  state falsification) at the protocol level.
- **Privacy by Design**: The game MUST NOT require personal data to play. Anonymous
  gameplay MUST be supported by default.

### II. Performance (Low Latency & Stability)

Network latency and simulation stability are non-negotiable priorities.

- **Network Priority**: All architectural decisions MUST prioritize low latency and
  stable network performance over feature richness.
- **Tick Stability**: Server MUST maintain stable tick rate (20-60 TPS depending on
  game mode) without stop-the-world garbage collection pauses.
- **Separation of Concerns**: Heavy simulation runs server-side; heavy rendering runs
  client-side. Neither may block the other.
- **Event-Driven Updates**: Systems MUST use event-driven updates rather than polling
  or global iteration loops where possible.
- **Lazy Loading**: Assets and chunks MUST load lazily. Initial connection time MUST
  NOT scale with world size.
- **Script Performance Boundary**: High-level scripts (Lua/JS) MUST NOT perform
  computationally expensive operations. Heavy work MUST use engine primitives.
- **Deterministic Multithreading**: Parallel execution MUST be deterministic.
  Race conditions are unacceptable.

### III. Architecture (Engine-First Modularity)

The engine provides optimized primitives; gameplay builds on them.

- **Engine-First Design**: The core engine (simulation, networking, rendering) MUST
  be developed independently of any specific game mode.
- **Strict Layer Separation**:
  - Core Engine: simulation, networking, rendering primitives
  - Gameplay Layer: systems and APIs built on engine primitives
  - Mod Layer: data, scripts, and WASM modules
  - UI Layer: embedded browser, fully decoupled from game logic
- **Primitive Provision**: The engine provides optimized primitives. Mods and gameplay
  code MUST use these primitives rather than reimplementing functionality.
- **Mod Independence**: Mods MUST NOT have hard dependencies on other mods.
  Optional integrations MUST degrade gracefully.
- **API Versioning**: All public APIs MUST be versioned. Breaking changes require
  major version increments and migration documentation.

### IV. Modding (First-Class Extensibility)

Modding is a core feature, not an afterthought.

- **Central Feature**: The mod system MUST be designed and documented from the start,
  not retrofitted.
- **Unified System**: No separation between "plugins" and "mods". One system, one API.
- **Three Mod Tiers**:
  - Data-only mods (declarative JSON/TOML, no code execution)
  - Script mods (JavaScript or Lua, event-driven, sandboxed)
  - Core mods (Rust compiled to WASM, full engine API access, sandboxed)
- **No Custom Simulation Loops**: Mods MUST NOT implement their own global simulation
  loops. All updates flow through engine-provided hooks.
- **Engine Performance Control**: The engine retains final authority over performance.
  Mods that degrade performance MAY be throttled or disabled.
- **Automatic Sync**: Required mods MUST sync automatically when players connect.
  Manual mod installation MUST NOT be required for basic play.
- **Early Documentation**: Official mod documentation and SDK MUST ship with initial
  releases, not as post-launch additions.

### V. Code Quality (Explicit & Tested)

Production code MUST be readable, tested, and maintainable.

- **Readability**: Code MUST be explicit and self-documenting. Clever tricks are
  forbidden in core systems.
- **No Temporary Hacks**: The core engine MUST NOT contain "temporary" solutions,
  workarounds, or technical debt accepted for convenience.
- **Mandatory Testing**: All network and simulation logic MUST have automated tests.
  Untested critical paths are release blockers.
- **Structured Logging**: All logging MUST be structured (key-value) and dynamically
  configurable at runtime.
- **No Panics in Production**: Production builds MUST NOT panic. All error paths MUST
  be handled explicitly.
- **Tracked Technical Debt**: Any accepted technical debt MUST be documented with
  rationale and tracked for resolution.

### VI. Technical Standards (Rust & Reproducibility)

Technical choices prioritize stability, reproducibility, and interoperability.

- **Stable Rust Only**: Production code MUST compile on stable Rust. Nightly features
  are forbidden in shipped code.
- **Tooling Compliance**: All code MUST pass `cargo clippy` and `cargo fmt` without
  warnings or modifications.
- **Deterministic APIs**: All engine APIs MUST produce deterministic, predictable
  results for identical inputs.
- **Documented Protocols**: All network protocols MUST be versioned and documented
  in human-readable specifications.
- **Explicit Serialization**: Data formats MUST be explicit, documented, and support
  forward/backward evolution (e.g., no implicit schema changes).
- **Reproducible Builds**: Builds MUST be reproducible across all supported platforms
  given identical inputs.

### VII. Player Experience (Multiplayer-First)

Multiplayer is the primary experience; everything else is secondary.

- **Multiplayer Priority**: Design decisions MUST optimize for multiplayer first.
  Single-player is a special case of multiplayer (local server).
- **Simple Connection**: Server browser and connection MUST be integrated into the
  main game UI. External tools MUST NOT be required.
- **Zero Technical Prerequisites**: Players MUST be able to join any server without
  manual mod installation or configuration.
- **Responsive UI**: The UI MUST remain responsive at all times. UI rendering MUST
  NOT block game logic or vice versa.
- **Performance Isolation**: UI operations MUST NOT impact game performance.
  Heavy UI MUST be throttled or deferred.
- **Competitive Features First**: Competitive gameplay features (fair play, spectating,
  replays) take priority over cosmetic features.

### VIII. Open Source & Governance (Transparent Development)

The project is open source with documented decision-making.

- **Open by Default**: All source code MUST be public and auditable. Private forks
  for security fixes are acceptable only temporarily.
- **No Proprietary Lock-in**: The project MUST NOT depend on proprietary services or
  libraries that would prevent community forks.
- **Documented Decisions**: All significant technical decisions MUST be documented
  via RFCs or ADRs before implementation.
- **Guided Contributions**: Community contributions are welcome but MUST follow
  project standards. Maintainers MAY reject contributions that violate principles.
- **Technical Coherence**: Architectural consistency takes precedence over faster
  delivery. Shortcuts that compromise design are forbidden.

### IX. Scoping & Realism (Minimal Viable Scope)

Scope MUST remain achievable by a small team.

- **Minimal MVP**: The initial release MUST include only essential features.
  Each feature MUST justify its complexity cost.
- **No Feature Creep**: Features not in the current milestone MUST be deferred,
  not partially implemented.
- **Simple Over Complex**: A simple, stable, optimized feature MUST always be
  preferred over a complex, unstable alternative.
- **Small Team Viability**: All architectural decisions MUST consider long-term
  maintainability by a small team (1-5 developers).

### X. Long-Term Vision (Platform Durability)

The project is a platform designed to outlive any single game mode.

- **Multi-Year Horizon**: Architectural decisions MUST assume a 5+ year lifespan.
  Short-term gains that create long-term problems are forbidden.
- **Zero Voluntary Debt**: Technical debt MUST NOT be accepted voluntarily.
  All debt MUST result from genuinely unavoidable constraints.
- **Non-Breaking Evolution**: The engine MUST support evolution without breaking
  existing mods or game modes. Deprecation cycles MUST be respected.
- **Engine Survives Modes**: The core engine MUST remain viable even if all current
  game modes are replaced or deprecated.
- **Platform, Not Product**: The game is a platform for experiences, not a fixed
  content package. Flexibility MUST be preserved.

## Technical Standards

### Language & Tooling

- **Language**: Rust (stable channel only)
- **Formatting**: `cargo fmt` (enforced in CI)
- **Linting**: `cargo clippy` (zero warnings policy)
- **Testing**: `cargo test` for unit/integration tests
- **Documentation**: Rustdoc for API documentation

### Network Protocol Requirements

- All protocols MUST be versioned (major.minor format)
- Protocol documentation MUST be human-readable
- Breaking changes require major version increment
- Backward compatibility MUST be maintained within major versions

### Serialization Requirements

- Use explicit, self-describing formats (e.g., MessagePack with schema, Cap'n Proto)
- Schema changes MUST be backward compatible or versioned
- Binary formats MUST have documented specifications

## Development Workflow

### Code Review Requirements

- All changes to core engine MUST be reviewed
- Constitution compliance MUST be verified in review
- Performance-critical changes MUST include benchmarks

### Testing Requirements

- Network code: mandatory unit and integration tests
- Simulation code: mandatory determinism tests
- Mod API: mandatory contract tests
- UI: visual regression tests recommended

### Release Process

- All releases MUST pass full test suite
- Release builds MUST be reproducible
- Breaking changes MUST be documented in changelog
- Mod API changes MUST include migration guides

## Governance

This constitution is the supreme technical authority for the project. All code,
documentation, and architectural decisions MUST comply with these principles.

### Amendment Process

1. Propose amendment via RFC document
2. Community review period (minimum 7 days)
3. Core maintainer approval required
4. Amendment MUST include migration plan if breaking
5. Version increment per semantic versioning rules

### Compliance

- All pull requests MUST verify constitution compliance
- Complexity MUST be justified against Scoping principles
- Violations MUST be resolved before merge
- Exceptions require documented RFC with expiration date

### Version Policy

- MAJOR: Principle removal or incompatible redefinition
- MINOR: New principle or significant expansion
- PATCH: Clarification or wording improvements

**Version**: 1.0.0 | **Ratified**: 2025-12-14 | **Last Amended**: 2025-12-14
