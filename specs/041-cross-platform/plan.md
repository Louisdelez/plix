# Implementation Plan: Cross-Platform Client Packaging & Headless Server

**Branch**: `041-cross-platform` | **Date**: 2025-12-19 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/041-cross-platform/spec.md`

## Summary

Deliver reproducible multi-platform distribution for plix:
- **Client bundles** for Windows/Linux/macOS with assets, CEF runtimes, and build metadata
- **Headless server binary** with proper signal handling, exit codes, and config validation
- **CI workflow** for automated builds, packaging, smoke tests, and artifact uploads
- **Optional Docker** support for containerized server deployment

## Technical Context

**Language/Version**: Rust 1.83+ (stable channel per constitution, workspace rust-version)
**Primary Dependencies**: tokio (async), clap (CLI), tracing (logging), wgpu/winit (client), bincode (serialization)
**Storage**: Filesystem for bundles, configs, and assets
**Testing**: cargo test for unit/integration, custom smoke tests for packaging validation
**Target Platform**: Windows x64, Linux x86_64, macOS x64/ARM (universal binary)
**Project Type**: Multi-crate workspace (existing structure with plix-server, plix-client)
**Performance Goals**: CI build under 30 minutes, graceful shutdown within 5 seconds
**Constraints**: No graphical dependencies for headless server, CEF bundling for client
**Scale/Scope**: 6 artifacts (3 client + 3 server), 3 packaging scripts per type

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
|-----------|--------|----------|
| VI. Technical Standards: Stable Rust Only | PASS | Workspace uses rust-version = "1.83" (stable) |
| VI. Technical Standards: Tooling Compliance | PASS | CI enforces cargo fmt/clippy |
| VI. Technical Standards: Reproducible Builds | PASS | Build info (version/sha/date) ensures traceability |
| V. Code Quality: No Panics in Production | PASS | Exit codes for all error conditions, graceful shutdown |
| V. Code Quality: Structured Logging | PASS | tracing already in use, flush on shutdown |
| IX. Scoping: Minimal MVP | PASS | ZIP/tar.gz only, defers MSI/AppImage/notarization |
| IX. Scoping: Simple Over Complex | PASS | Scripts + CI matrix, no complex installer frameworks |
| III. Architecture: Strict Layer Separation | PASS | Server headless = no CEF/client deps; client = full stack |
| I. Security: Attack Surface Reduction | PASS | Separate binaries, no client-only code in server |

**No violations detected.** Proceeding to Phase 0.

## Project Structure

### Documentation (this feature)

```text
specs/041-cross-platform/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (packaging contracts)
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
crates/
├── plix-client/         # Client binary (existing)
│   └── src/
│       ├── main.rs      # Entry point, version display
│       └── build_info.rs # NEW: Build metadata module
├── plix-server/         # Server binary (existing)
│   └── src/
│       ├── main.rs      # Entry point with signal handling
│       ├── bin/
│       │   └── plix-server-headless.rs  # NEW: Dedicated headless binary
│       ├── config.rs    # Config validation (enhance)
│       └── shutdown.rs  # NEW: Graceful shutdown module
└── plix-common/
    └── src/
        └── build_info.rs # NEW: Shared build info types

scripts/
├── package/             # NEW: Packaging scripts
│   ├── client_windows.ps1
│   ├── client_linux.sh
│   ├── client_macos.sh
│   ├── server_windows.ps1
│   ├── server_linux.sh
│   └── server_macos.sh
└── validate_bundle.sh   # NEW: Smoke test script

.github/workflows/
├── ci.yml               # Existing CI (enhance for release)
└── release.yml          # NEW: Release workflow with packaging

deploy/
├── docker/
│   ├── Dockerfile       # NEW: Headless server image
│   └── docker-compose.yml
└── configs/
    └── examples/        # Server config examples
        ├── server.toml
        └── server_mods.toml

docs/
├── release/
│   ├── client-packaging.md   # NEW
│   └── ci-artifacts.md       # NEW
└── server/
    ├── README.md        # Existing (enhance)
    └── headless-deploy.md    # NEW
```

**Structure Decision**: Extends existing multi-crate workspace. New packaging infrastructure lives in `scripts/package/`. CI workflow enhanced with dedicated release workflow. Docker configs in `deploy/docker/`.

## Complexity Tracking

No constitution violations require justification.
