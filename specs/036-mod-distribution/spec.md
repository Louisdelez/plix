# Feature Specification: Mod Distribution

**Feature Branch**: `036-mod-distribution`
**Created**: 2025-12-18
**Status**: Draft
**Input**: User description: Mod Distribution system with registry, dependencies, versioning, and optional signatures

## Overview

This feature provides a production-ready mod distribution system for Plix servers, enabling reliable installation, dependency management, version compatibility checking, and integrity verification of mods. The system supports both local (offline) and remote registries, with optional cryptographic signatures for trust verification.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Automatic Mod Installation (Priority: P1)

As a server administrator, I declare a list of required mods (id + version constraints) in my server configuration, and the server automatically downloads and installs them from configured registries.

**Why this priority**: This is the core value proposition - enabling servers to declaratively specify their mod requirements and have them automatically resolved and installed. Without this, administrators must manually download and place mod files.

**Independent Test**: Can be fully tested by configuring a server with 2-3 required mods from a test registry, starting the server, and verifying all mods are installed and loaded correctly.

**Acceptance Scenarios**:

1. **Given** a server configuration specifying `mod-a@^1.0` and `mod-b@~2.1`, **When** the server starts with no cached mods, **Then** both mods are downloaded, verified (SHA-256), extracted, and loaded successfully
2. **Given** a server with `mod-a@1.0.0` already cached, **When** the server starts requiring `mod-a@^1.0`, **Then** the cached version is used without re-downloading
3. **Given** a registry is unreachable, **When** the server tries to install a mod not in cache, **Then** an explicit error (EMREG001) is returned with the failed registry URL

---

### User Story 2 - Reproducible Deployments via Lockfile (Priority: P1)

As a server administrator, I lock my mod set to exact versions via a lockfile (`mods.lock`) to guarantee reproducible deployments across different servers or environments.

**Why this priority**: Reproducibility is essential for production deployments - servers must have identical mod versions to ensure consistent behavior and prevent "works on my machine" issues.

**Independent Test**: Can be tested by generating a lockfile on one server, copying it to another, and verifying that both servers install exactly the same mod versions (including transitive dependencies).

**Acceptance Scenarios**:

1. **Given** a server configuration with version constraints, **When** dependency resolution completes, **Then** a `mods.lock` file is generated containing exact versions and SHA-256 hashes for all mods
2. **Given** a `mods.lock` file exists, **When** the server installs mods, **Then** only the exact versions specified in the lockfile are installed (ignoring newer compatible versions)
3. **Given** a lockfile specifies `mod-a@1.0.5 sha256:abc123`, **When** the downloaded file has a different hash, **Then** installation fails with EMREG004 (hash mismatch)

---

### User Story 3 - Dependency Resolution (Priority: P1)

As a mod author, I declare dependencies on other mods with version constraints, and the system automatically resolves and installs all transitive dependencies in compatible versions.

**Why this priority**: Mods frequently depend on other mods (libraries, frameworks, shared assets). Without automatic dependency resolution, installation becomes manual and error-prone.

**Independent Test**: Can be tested by installing a mod with 2 transitive dependencies and verifying all three are installed in compatible versions.

**Acceptance Scenarios**:

1. **Given** `mod-a` depends on `mod-b@^1.0` and `mod-b` depends on `mod-c@^2.0`, **When** I install `mod-a`, **Then** all three mods are installed with compatible versions
2. **Given** `mod-a` requires `mod-c@^1.0` and `mod-b` requires `mod-c@^2.0`, **When** I try to install both, **Then** resolution fails with EMREG006 (dependency conflict) listing the incompatible requirements
3. **Given** `mod-a` depends on `mod-b` and `mod-b` depends on `mod-a`, **When** I try to install `mod-a`, **Then** resolution fails with EMREG008 (cycle detected)

---

### User Story 4 - Integrity Verification (Priority: P1)

As a server administrator, the system verifies downloaded mod bundles using SHA-256 hashes before installation to prevent corrupted or tampered bundles from being loaded.

**Why this priority**: Security is critical - loading corrupted or malicious bundles could crash servers or compromise security. Integrity verification is the minimum security baseline.

**Independent Test**: Can be tested by modifying a downloaded bundle and verifying that installation fails with a hash mismatch error.

**Acceptance Scenarios**:

1. **Given** a mod bundle is downloaded, **When** its SHA-256 hash matches the index, **Then** installation proceeds
2. **Given** a mod bundle is downloaded, **When** its SHA-256 hash does not match the index, **Then** the bundle is deleted and EMREG004 (hash mismatch) is returned
3. **Given** a partial download (incomplete file), **When** hash verification runs, **Then** it fails and triggers a re-download attempt

---

### User Story 5 - Version Compatibility Checking (Priority: P2)

As the game engine, I reject mod installations that are incompatible with the current API version or engine version to prevent runtime errors.

**Why this priority**: Installing incompatible mods leads to crashes, undefined behavior, or security issues. Checking compatibility at install time provides clear feedback before problems occur.

**Independent Test**: Can be tested by attempting to install a mod with `api_version=99` (higher than current) and verifying rejection with a clear error message.

**Acceptance Scenarios**:

1. **Given** a mod requires `api_version=1` and the engine supports `api_version=1`, **When** installation is attempted, **Then** the mod is installed
2. **Given** a mod requires `api_version=2` and the engine only supports `api_version=1`, **When** installation is attempted, **Then** installation fails with EMREG007 (version incompatible)
3. **Given** a mod specifies `engine={min="1.5"}` and the engine version is `1.4`, **When** installation is attempted, **Then** installation fails with EMREG007

---

### User Story 6 - Optional Signature Verification (Priority: P3)

As a server administrator, I can optionally require cryptographic signatures on mods to verify they come from trusted publishers.

**Why this priority**: Signatures provide identity verification beyond integrity, useful for curated servers or production environments, but integrity verification alone is sufficient for most use cases.

**Independent Test**: Can be tested by configuring `require_signature=true`, attempting to install an unsigned mod, and verifying rejection.

**Acceptance Scenarios**:

1. **Given** `require_signature=false` (default), **When** an unsigned mod is installed, **Then** installation proceeds (only hash is verified)
2. **Given** `require_signature=true` and a mod has a valid signature from an allowed key, **When** installation is attempted, **Then** the mod is installed
3. **Given** `require_signature=true` and a mod has a signature from an unknown key, **When** installation is attempted, **Then** installation fails with EMREG005 (signature invalid)
4. **Given** `require_signature=true` and a mod has no signature, **When** installation is attempted, **Then** installation fails with EMREG005

---

### Edge Cases

- What happens when a download is interrupted mid-transfer? System retries up to 3 times before failing with EMREG003
- What happens when disk space is insufficient for extraction? Clear error message returned, no partial extraction left on disk
- What happens when multiple registries have the same mod? First registry with matching version wins (priority order)
- What happens when a mod is pinned but a newer version has a security fix? Pinned version is used; administrators must manually update the pin
- What happens when the lockfile and config disagree on required mods? Lockfile takes precedence for versions; new mods in config are resolved and added to lockfile
- What happens when a mod bundle exceeds the size limit (50 MB default)? Download is aborted with EMREG003

## Requirements *(mandatory)*

### Functional Requirements

#### Mod Package Format

- **FR-001**: System MUST support a standard mod bundle format (`.plixmod`) as a compressed archive containing `mod.toml` manifest, optional `mod.wasm`, optional `assets/` directory, and optional documentation files
- **FR-002**: System MUST produce deterministic bundles (stable file ordering) to ensure reproducible SHA-256 hashes
- **FR-003**: System MUST enforce a configurable maximum bundle size (default: 50 MB)
- **FR-004**: System MUST validate that every bundle contains a valid `mod.toml` manifest before installation

#### Registry

- **FR-005**: System MUST support local registry storage with a cache directory containing installed bundles and an index of available mods
- **FR-006**: System MUST support remote registries via HTTP with an `index.json` catalog containing mod metadata, versions, download URLs, and hashes
- **FR-007**: System MUST support multiple registries with configurable priority order
- **FR-008**: System MUST cache downloaded bundles and skip re-downloading when a matching hash exists locally
- **FR-009**: System MUST support configurable download timeouts (default: 30s connect, 120s read) and retry count (default: 3)

#### Dependency Resolution

- **FR-010**: System MUST parse SemVer version constraints including exact (`=`), caret (`^`), tilde (`~`), and range operators (`>=`, `<`, etc.)
- **FR-011**: System MUST resolve dependencies transitively, selecting the newest compatible version by default
- **FR-012**: System MUST detect dependency cycles and fail with EMREG008
- **FR-013**: System MUST detect conflicting version requirements and fail with EMREG006
- **FR-014**: System MUST generate a lockfile (`mods.lock`) containing exact versions, SHA-256 hashes, source registry, and resolved dependency graph

#### Compatibility

- **FR-015**: System MUST verify mod `api_version` compatibility against the engine's supported API version (Feature 034) before installation
- **FR-016**: System MUST verify optional `engine.min` and `engine.max` version constraints before installation
- **FR-017**: System MUST reject incompatible mods with EMREG007 error

#### Integrity & Signatures

- **FR-018**: System MUST calculate SHA-256 hash of downloaded bundles and compare against the registry index
- **FR-019**: System MUST delete and refuse to install bundles with hash mismatches (EMREG004)
- **FR-020**: System MUST support optional signature verification when configured
- **FR-021**: System MUST maintain an allowlist of trusted public keys for signature verification
- **FR-022**: System MUST reject mods with invalid signatures when signature verification is required (EMREG005)

#### Configuration

- **FR-023**: System MUST support a server configuration file (`server_mods.toml`) specifying registry URLs, required mods with version constraints, and trust policy
- **FR-024**: System MUST support pinning specific mod versions in configuration to prevent automatic updates

#### Installation Lifecycle

- **FR-025**: System MUST follow the installation flow: read config → resolve dependencies → download → verify hash → extract → load manifest → initialize mod runtime
- **FR-026**: System MUST support enabling/disabling individual mods without uninstalling
- **FR-027**: System MUST support removing mods that are no longer required

#### Observability

- **FR-028**: System MUST log resolution steps, download progress, verification results, and errors
- **FR-029**: System MUST return structured error codes: EMREG001 (registry unreachable), EMREG002 (invalid index), EMREG003 (download failed), EMREG004 (hash mismatch), EMREG005 (signature invalid), EMREG006 (dependency conflict), EMREG007 (version incompatible), EMREG008 (cycle detected)

### Key Entities

- **ModBundle**: A packaged mod archive containing manifest, code, and assets with associated SHA-256 hash
- **ModManifest**: Metadata describing a mod including id, name, version, dependencies, and capability requirements
- **Registry**: A source of mod packages, either local (filesystem) or remote (HTTP), with an index of available mods
- **RegistryIndex**: A catalog listing all mods in a registry with their versions, download URLs, hashes, and signatures
- **ModVersion**: A specific version of a mod with associated metadata, download location, and integrity hash
- **Lockfile**: A file recording the exact resolved versions and hashes for all installed mods
- **TrustPolicy**: Configuration specifying signature requirements and allowed public keys

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Server administrators can declare required mods and have them installed automatically in under 60 seconds for a typical 5-mod setup
- **SC-002**: Dependency resolution handles 50+ mods with complex dependency graphs in under 5 seconds
- **SC-003**: 100% of corrupted or tampered bundles are detected and rejected via SHA-256 verification
- **SC-004**: All version constraint formats (exact, caret, tilde, ranges) resolve correctly as per SemVer specification
- **SC-005**: Cycle detection identifies all circular dependencies without hanging or excessive memory usage
- **SC-006**: Two servers with identical lockfiles produce identical mod installations 100% of the time
- **SC-007**: Offline operation (local registry only) works without any network connectivity
- **SC-008**: Clear error messages identify the specific cause and affected mods for all failure scenarios

## Scope

### In Scope

- Mod package format (`.plixmod`) definition
- Local and remote registry support
- Dependency resolution with SemVer constraints
- SHA-256 integrity verification
- Optional signature verification
- Server configuration for mod requirements
- Lockfile generation for reproducibility
- Integration with plix-mod-core (Feature 034) and plix-mod-runtime-wasm (Feature 035)

### Out of Scope

- In-game mod browser UI
- Marketplace or payment systems
- Peer-to-peer distribution
- Delta updates or binary patching
- Mandatory signatures (optional only in MVP)
- Antivirus scanning or mod review pipeline
- Client-side mod installation (server-only for MVP)

## Assumptions

- Registries use standard HTTP/HTTPS protocols; no custom protocols required
- SHA-256 is sufficient for integrity verification
- Ed25519 or similar algorithm for signatures (specific algorithm to be determined in planning)
- Bundle size limit of 50 MB is sufficient for most mods
- Servers have sufficient disk space for mod cache
- The `semver` crate or equivalent provides reliable version parsing and comparison
- Mods are distributed as complete bundles, not incremental patches

## Dependencies

- **Feature 034 (Mod API Core)**: Provides manifest parsing, capability system, and mod registry integration
- **Feature 035 (Sandboxed Mod Runtime)**: Provides WASM mod execution environment
