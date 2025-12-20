# Implementation Plan: Tooling Mods (SDK, Templates, CLI, Hot-Reload)

**Branch**: `038-tooling-mods` | **Date**: 2025-12-19 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/038-tooling-mods/spec.md`

## Summary

Deliver comprehensive mod developer tooling to reduce "time-to-first-mod" to under 5 minutes:
- **plix-mod-sdk**: Rust crate with safe ABI v1 wrappers, ergonomic macros, and stable types
- **Templates**: Ready-to-compile mod scaffolds (chat-filter, world-query, timers-net)
- **CLI tooling**: `plix mod new/build/pack/validate/install` commands
- **Documentation**: Quickstart, SDK reference, troubleshooting guides
- **Hot-reload (DEV-only)**: Optional filesystem watcher for rapid iteration

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel only per constitution)
**Primary Dependencies**: plix-mod-core (034), plix-mod-runtime-wasm (035), plix-mod-distribution (036), clap (CLI), notify (file watching), proc-macro2/quote/syn (macros)
**Storage**: Filesystem only (templates, bundle cache, mod projects)
**Testing**: cargo test (unit + integration), deterministic hash verification
**Target Platform**: Host (CLI) + wasm32-unknown-unknown (SDK output)
**Project Type**: Multi-crate Rust workspace (SDK, CLI, templates)
**Performance Goals**: Build + pack + validate < 30s on cold start; hot-reload detect < 500ms, reload < 2s
**Constraints**: Bundle size ≤ 10 MB; SDK must compile to WASM without OS dependencies; dev-only features must not leak to production
**Scale/Scope**: 2-3 starter templates; CLI with 5 core commands; SDK covering all Feature 034 host functions

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Security | ✅ PASS | Hot-reload dev-only (FR-016, FR-028); unsigned bundles dev-only (FR-027) |
| II. Performance | ✅ PASS | SDK uses engine primitives; no custom simulation loops |
| III. Architecture | ✅ PASS | SDK wraps engine API; strict layer separation maintained |
| IV. Modding | ✅ PASS | This IS the "Early Documentation" and SDK requirement |
| V. Code Quality | ✅ PASS | Mandatory tests for pack determinism and validation |
| VI. Technical Standards | ✅ PASS | Stable Rust only; deterministic builds; versioned API |
| VII. Player Experience | ✅ PASS | Enables automatic mod sync (037 dependency) |
| VIII. Open Source | ✅ PASS | All tooling is open source; no proprietary dependencies |
| IX. Scoping | ✅ PASS | Minimal MVP: 2 templates, 5 CLI commands, core SDK wrappers |
| X. Long-Term Vision | ✅ PASS | API versioned (SDK_ABI_VERSION, SDK_API_VERSION); deprecation-safe |

**GATE RESULT**: PASS - No violations. Proceed to Phase 0.

## Project Structure

### Documentation (this feature)

```text
specs/038-tooling-mods/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
crates/
├── plix-mod-sdk/                    # NEW: Modder-facing SDK
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs                   # Re-exports, prelude
│   │   ├── version.rs               # SDK_ABI_VERSION, SDK_API_VERSION
│   │   ├── error.rs                 # ModError, EMOD mapping
│   │   ├── log.rs                   # info!, warn!, error!, debug!
│   │   ├── caps.rs                  # has(), capability IDs
│   │   ├── events.rs                # subscribe(), event types
│   │   ├── world.rs                 # get_block, raycast, query_aabb, set_block
│   │   ├── entities.rs              # get_transform, apply_damage, apply_impulse
│   │   ├── net.rs                   # send_to_client, broadcast
│   │   └── timers.rs                # set_timeout, set_interval, clear
│   └── macros/                      # Proc-macro crate
│       ├── Cargo.toml
│       └── src/lib.rs               # #[plix_mod], #[on_event]
│
├── plix-mod-cli/                    # NEW: CLI tooling
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs                  # Entry point, clap setup
│       ├── cmd_new.rs               # plix mod new
│       ├── cmd_build.rs             # plix mod build
│       ├── cmd_pack.rs              # plix mod pack
│       ├── cmd_validate.rs          # plix mod validate
│       └── cmd_install.rs           # plix mod install --local
│
├── plix-server/src/mods/
│   └── dev_hot_reload.rs            # NEW: DEV-only hot-reload watcher

templates/mods/                       # NEW: Mod templates
├── chat-filter/
│   ├── Cargo.toml
│   ├── mod.toml
│   ├── src/lib.rs
│   └── build.sh
├── world-query/
│   ├── Cargo.toml
│   ├── mod.toml
│   ├── src/lib.rs
│   └── build.sh
└── timers-net/
    ├── Cargo.toml
    ├── mod.toml
    ├── src/lib.rs
    └── build.sh

docs/modding/                         # NEW: Modding documentation
├── quickstart.md
├── sdk.md
├── distribution.md
└── troubleshooting.md
```

**Structure Decision**: Multi-crate workspace with dedicated SDK (plix-mod-sdk), CLI (plix-mod-cli), and templates directory. SDK includes a separate proc-macro subcrate for attribute macros.

## Complexity Tracking

> No violations to justify - all gates passed.

---

## Phase 0: Research Tasks

### R-001: Proc-Macro Pattern for WASM Exports

**Question**: Best pattern for generating `mod_init`, `mod_on_event`, `mod_shutdown` exports from attribute macros in WASM context?

**Research Scope**:
- How to generate `#[no_mangle] pub extern "C"` functions from proc-macros
- How to handle event routing table generation
- Existing patterns from wasmtime/wasmer plugin systems

### R-002: Deterministic ZIP Creation

**Question**: How to ensure `plix mod pack` produces byte-identical archives?

**Research Scope**:
- Deterministic file ordering (sorted)
- Fixed timestamps (epoch or zero)
- Consistent compression settings
- Rust ZIP library options (zip crate capabilities)

### R-003: Filesystem Watcher for Hot-Reload

**Question**: Best Rust library for cross-platform file watching with debounce?

**Research Scope**:
- `notify` crate capabilities and cross-platform support
- Debounce implementation patterns
- Integration with tokio async runtime

### R-004: Feature 034/035 ABI Surface

**Question**: What exact host functions exist in Feature 034/035 that SDK must wrap?

**Research Scope**:
- Review plix-mod-core host function exports
- Review plix-mod-runtime-wasm ABI v1 definitions
- Identify complete function list for SDK coverage

---

## Phase 1: Design Artifacts

After research completion, generate:
1. `data-model.md` - SDK types, ModProject structure, DevConfig schema
2. `contracts/` - SDK public API surface, CLI command contracts
3. `quickstart.md` - End-to-end mod creation walkthrough
