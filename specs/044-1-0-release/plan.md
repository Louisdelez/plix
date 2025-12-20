# Implementation Plan: 1.0 Release

**Branch**: `044-1-0-release` | **Date**: 2025-12-20 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/044-1-0-release/spec.md`

## Summary

Prepare and deliver Plix v1.0.0 as a stable, production-ready release with:
- Unified semantic versioning across all components (client, server, protocol, mod API, content schema)
- Safe migration system with automatic backups (3 rolling) for configs, player saves, and content
- Complete documentation (user, server admin, modding, release)
- Open-source governance (MIT license, CONTRIBUTING, CODE_OF_CONDUCT, roadmap)
- Release automation (GPG-signed tags, SHA-256 checksums, cross-platform artifacts)

## Technical Context

**Language/Version**: Rust 1.83 (stable, per workspace `rust-version`)
**Primary Dependencies**: shadow-rs (build info), serde/toml (config), bincode (serialization), sha2 (checksums)
**Storage**: File system - configs (~/.config/plix/), saves (~/.local/share/plix/worlds/), backups (adjacent to data)
**Testing**: cargo test (unit/integration), manual smoke tests (cross-platform)
**Target Platform**: Windows, Linux, macOS (clients) + Linux headless (server)
**Project Type**: Multi-crate workspace (existing structure)
**Performance Goals**: No regression from Feature 039 baseline
**Constraints**: Zero blocking issues, no experimental features enabled by default
**Scale/Scope**: Release readiness audit of existing ~43 features

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Security | ✅ PASS | No new attack surface; release validation only |
| II. Performance | ✅ PASS | No new runtime code; migrations run once at startup |
| III. Architecture | ✅ PASS | Version/migration modules fit existing layer model |
| IV. Modding | ✅ PASS | Mod API versioning enables stability guarantees (FR-003/004) |
| V. Code Quality | ✅ PASS | No temporary hacks; explicit version/migration code |
| VI. Technical Standards | ✅ PASS | Stable Rust only; reproducible builds (FR-021) |
| VII. Player Experience | ✅ PASS | Version display improves debugging; migrations transparent |
| VIII. Open Source | ✅ PASS | Core feature: LICENSE, CONTRIBUTING, CoC, roadmap |
| IX. Scoping | ✅ PASS | No new gameplay; polish/packaging only |
| X. Long-Term Vision | ✅ PASS | SemVer enables non-breaking evolution |

**Gate Result**: PASS - All principles satisfied. Proceed to Phase 0.

## Project Structure

### Documentation (this feature)

```text
specs/044-1-0-release/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
# Existing workspace structure (no new crates needed)
crates/
├── plix-common/src/
│   ├── build_info.rs       # EXISTS: BuildInfo struct (extend)
│   ├── version.rs          # NEW: VersionInfo, ProtocolVersion, ModApiVersion constants
│   └── migration/          # NEW: Migration framework
│       ├── mod.rs
│       ├── backup.rs       # Backup creation/rotation (3 rolling)
│       ├── config.rs       # Config migration engine
│       └── save.rs         # Save data migration engine
├── plix-server/src/
│   └── content/
│       └── schema.rs       # EXTEND: Add ContentSchemaVersion
├── plix-mod-core/src/
│   └── lib.rs              # EXTEND: Add MOD_API_VERSION constant
├── plix-client/src/
│   └── ui_cef/
│       └── menus/about.rs  # NEW: About panel with version display

# Root-level governance files (NEW)
LICENSE                     # MIT license text
README.md                   # Project overview, quickstart, links
CONTRIBUTING.md             # PR workflow, commit format, review process
CODE_OF_CONDUCT.md          # Community standards
SECURITY.md                 # Vulnerability reporting
ROADMAP.md                  # v1.x and v2.0 planning

# Documentation (NEW)
docs/
├── user/
│   ├── installation.md     # Client install (Win/Linux/macOS)
│   ├── getting-started.md  # First steps, tutorial quest
│   ├── settings.md         # Graphics, accessibility, keybinds
│   └── faq.md              # Common questions
├── server/
│   ├── installation.md     # Headless server setup
│   ├── configuration.md    # Server config reference
│   ├── mods.md             # Mod management
│   ├── security.md         # Security settings, limits
│   ├── backups.md          # Backup/restore procedures
│   └── upgrading.md        # Migration from v0.x
├── modding/
│   ├── getting-started.md  # Hello mod tutorial
│   ├── sdk-reference.md    # API documentation
│   ├── stability.md        # Stability policy (Stable/Experimental/Deprecated)
│   └── compatibility.md    # Version compatibility guide
└── release/
    ├── CHANGELOG.md        # v1.0.0 changelog
    ├── migration-guide.md  # Upgrade instructions
    ├── known-issues.md     # Known limitations
    └── verification.md     # SHA-256 checksum verification

# CI/CD (EXTEND)
.github/workflows/
└── release.yml             # EXISTS: Extend with GPG signing, checksums
```

**Structure Decision**: Extend existing multi-crate workspace. Add version/migration modules to plix-common. Add governance files at repo root. Add structured documentation under docs/.

## Complexity Tracking

No violations requiring justification. Feature adds no new architectural complexity - it packages and documents existing functionality.

## Clarifications Applied

From `/speckit.clarify` session 2025-12-20:
- License: MIT (FR-016)
- Checksum algorithm: SHA-256 (FR-026)
- Backup retention: 3 rolling backups (FR-007)
- Tag signing: GPG-signed (FR-027)

## Key Design Decisions

### D1: Version Source of Truth

Single source: `Cargo.toml` workspace version ("1.0.0")
- `plix-common::BuildInfo` already embeds this via shadow-rs
- Add `plix-common::version::PROTOCOL_VERSION` (major.minor, const)
- Add `plix-mod-core::MOD_API_VERSION` (major.minor, const)
- Add `ContentSchemaVersion` to content registry

### D2: Migration Strategy

Migrations run automatically on startup:
1. Detect current version from data files
2. Create timestamped backup (keep 3 most recent)
3. Apply migrations sequentially (N → N+1 → ... → current)
4. Log all changes; fail-safe on error

### D3: Documentation Structure

Modular Markdown under `docs/`:
- User docs: installation, gameplay, settings, FAQ
- Server docs: deployment, config, mods, security, upgrades
- Modding docs: SDK, stability policy, tutorials
- Release docs: changelog, migration, verification

### D4: Governance Files

Standard open-source structure at repo root:
- LICENSE (MIT full text)
- README.md (vision, status, contribution quickstart)
- CONTRIBUTING.md (PR rules, commit format, review)
- CODE_OF_CONDUCT.md (Contributor Covenant)
- SECURITY.md (vulnerability disclosure)
- ROADMAP.md (v1.x maintenance, v2.0 planning)

## Constitution Check (Post-Design)

*Re-evaluation after Phase 1 design completion.*

| Principle | Status | Post-Design Notes |
|-----------|--------|-------------------|
| I. Security | ✅ PASS | Migration backup uses SHA-256 checksums; no new attack vectors |
| II. Performance | ✅ PASS | Migrations are O(n) sequential; backup rotation O(1) |
| III. Architecture | ✅ PASS | Version/migration in plix-common; clear layer separation |
| IV. Modding | ✅ PASS | MOD_API_VERSION constant enables compatibility checks |
| V. Code Quality | ✅ PASS | Migration trait pattern is explicit and testable |
| VI. Technical Standards | ✅ PASS | Using stable Rust, sha2 crate, standard patterns |
| VII. Player Experience | ✅ PASS | Transparent migrations with logged changes |
| VIII. Open Source | ✅ PASS | Full governance file set designed |
| IX. Scoping | ✅ PASS | No scope creep; all work maps to spec requirements |
| X. Long-Term Vision | ✅ PASS | Version infrastructure supports 5+ year evolution |

**Final Gate Result**: PASS - Design validated against constitution. Ready for `/speckit.tasks`.

## Generated Artifacts

| Artifact | Path | Purpose |
|----------|------|---------|
| research.md | specs/044-1-0-release/research.md | Research findings, decisions, alternatives |
| data-model.md | specs/044-1-0-release/data-model.md | Entity definitions, relationships |
| version-api.md | specs/044-1-0-release/contracts/version-api.md | Version API contract |
| migration-api.md | specs/044-1-0-release/contracts/migration-api.md | Migration API contract |
| release-process.md | specs/044-1-0-release/contracts/release-process.md | Release workflow |
| quickstart.md | specs/044-1-0-release/quickstart.md | Implementation guide |
