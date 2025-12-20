# Tasks: Mod Distribution

**Input**: Design documents from `/specs/036-mod-distribution/`
**Prerequisites**: plan.md, spec.md, data-model.md, contracts/, research.md, quickstart.md

**Tests**: Tests are included per user story with integration tests for validation.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Crate**: `crates/plix-mod-distribution/src/`
- **Server integration**: `crates/plix-server/src/mods/`
- **Tests**: `crates/plix-mod-distribution/tests/`
- **Fixtures**: `tests/fixtures/mock_registry/`

---

## Phase 1: Setup (Crate Skeleton)

**Purpose**: Create new crate and module structure

- [X] T001 Create crate directory `crates/plix-mod-distribution/` with `Cargo.toml`
- [X] T002 [P] Add dependencies: semver, zip, sha2, serde, serde_json, reqwest, tokio, tracing
- [X] T003 [P] Add optional dependency: ed25519-dalek (feature = "signatures")
- [X] T004 [P] Create module skeleton `crates/plix-mod-distribution/src/lib.rs` with public exports
- [X] T005 [P] Create `crates/plix-mod-distribution/src/errors.rs` with DistributionError stub
- [X] T006 [P] Create `crates/plix-mod-distribution/src/config.rs` stub
- [X] T007 [P] Create `crates/plix-mod-distribution/src/index.rs` stub
- [X] T008 [P] Create `crates/plix-mod-distribution/src/registry.rs` stub
- [X] T009 [P] Create `crates/plix-mod-distribution/src/resolver.rs` stub
- [X] T010 [P] Create `crates/plix-mod-distribution/src/lockfile.rs` stub
- [X] T011 [P] Create `crates/plix-mod-distribution/src/downloader.rs` stub
- [X] T012 [P] Create `crates/plix-mod-distribution/src/integrity.rs` stub
- [X] T013 [P] Create `crates/plix-mod-distribution/src/signatures.rs` stub (feature-gated)
- [X] T014 [P] Create `crates/plix-mod-distribution/src/installer.rs` stub
- [X] T015 [P] Create `crates/plix-mod-distribution/src/bundle.rs` stub
- [X] T016 Add crate to workspace `Cargo.toml`
- [X] T017 Verify `cargo check -p plix-mod-distribution` passes

**Checkpoint**: Crate skeleton compiles - ready for implementation

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core types and errors that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T018 Implement `ModId` type with validation in `crates/plix-mod-distribution/src/lib.rs`
- [X] T019 [P] Implement EMREG001-008 error codes in `crates/plix-mod-distribution/src/errors.rs`
- [X] T020 [P] Implement `DistributionError` struct with code, message, context fields in `crates/plix-mod-distribution/src/errors.rs`
- [X] T021 [P] Implement error helpers (err_registry_unreachable, err_hash_mismatch, etc.) in `crates/plix-mod-distribution/src/errors.rs`
- [X] T022 [P] Implement `ModVersion`, `ModDependency`, `EngineConstraint` types in `crates/plix-mod-distribution/src/lib.rs`
- [X] T023 [P] Create test fixtures directory `tests/fixtures/mock_registry/`
- [X] T024 [P] Create minimal `tests/fixtures/mock_registry/index.json` with 3 test mods
- [X] T025 [P] Create test bundles `tests/fixtures/mock_registry/mods/test-mod-*.plixmod`
- [X] T026 Unit tests for ModId validation in `crates/plix-mod-distribution/src/lib.rs`
- [X] T027 Unit tests for error creation and display in `crates/plix-mod-distribution/tests/error_tests.rs`

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Automatic Mod Installation (Priority: P1) 🎯 MVP

**Goal**: Server administrator declares required mods in config, server automatically downloads and installs them from registries

**Independent Test**: Configure server with 2-3 required mods from test registry, start server, verify all mods installed and loaded

### Implementation for User Story 1

#### Config Parsing (server_mods.toml)

- [X] T028 [US1] Implement `DistributionConfig` struct in `crates/plix-mod-distribution/src/config.rs`
- [X] T029 [US1] Implement `RegistryConfig` struct (name, url, priority, enabled) in `crates/plix-mod-distribution/src/config.rs`
- [X] T030 [US1] Implement `ModRequirement` struct (id, version, pinned, optional) in `crates/plix-mod-distribution/src/config.rs`
- [X] T031 [US1] Implement `TrustPolicy` struct (require_signature, allowed_keys) in `crates/plix-mod-distribution/src/config.rs`
- [X] T032 [US1] Implement `DownloadSettings` struct (timeouts, retries, max_size) in `crates/plix-mod-distribution/src/config.rs`
- [X] T033 [US1] Implement `CacheSettings` struct (path, max_size) in `crates/plix-mod-distribution/src/config.rs`
- [X] T034 [US1] Implement `DistributionConfig::load(path)` with TOML parsing in `crates/plix-mod-distribution/src/config.rs`
- [X] T035 [US1] Unit tests for config parsing (valid/invalid) in `crates/plix-mod-distribution/tests/config_tests.rs`

#### Index Schema (index.json)

- [X] T036 [US1] Implement `RegistryIndex` struct in `crates/plix-mod-distribution/src/index.rs`
- [X] T037 [US1] Implement `RegistryMod` struct in `crates/plix-mod-distribution/src/index.rs`
- [X] T038 [US1] Implement `ModVersionEntry` struct with sha256, download_url, dependencies in `crates/plix-mod-distribution/src/index.rs`
- [X] T039 [US1] Implement index validation (registry_version=1, valid sha256, valid semver) in `crates/plix-mod-distribution/src/index.rs`
- [X] T040 [US1] Unit tests for index parsing (valid/invalid cases) in `crates/plix-mod-distribution/tests/index_tests.rs`

#### Registry Sources

- [X] T041 [US1] Implement `RegistrySource` trait in `crates/plix-mod-distribution/src/registry.rs`
- [X] T042 [US1] Implement `LocalRegistrySource` (file:// or path) in `crates/plix-mod-distribution/src/registry.rs`
- [X] T043 [US1] Implement `HttpRegistrySource` with async fetch in `crates/plix-mod-distribution/src/registry.rs`
- [X] T044 [US1] Implement timeout/retry logic for HTTP fetches in `crates/plix-mod-distribution/src/registry.rs`
- [X] T045 [US1] Implement index caching (in-memory + disk) in `crates/plix-mod-distribution/src/registry.rs`
- [X] T046 [US1] Implement `RegistryManager` for priority-ordered registry access in `crates/plix-mod-distribution/src/registry.rs`
- [X] T047 [US1] Unit tests for local registry source in `crates/plix-mod-distribution/tests/registry_tests.rs`

#### Bundle Download

- [X] T048 [US1] Implement `Downloader` struct in `crates/plix-mod-distribution/src/downloader.rs`
- [X] T049 [US1] Implement `download_bundle(url, dest)` with streaming in `crates/plix-mod-distribution/src/downloader.rs`
- [X] T050 [US1] Implement timeout configuration (connect 30s, read 120s default) in `crates/plix-mod-distribution/src/downloader.rs`
- [X] T051 [US1] Implement retry logic (3 attempts default) in `crates/plix-mod-distribution/src/downloader.rs`
- [X] T052 [US1] Implement size limit enforcement (50MB default, EMREG003 on exceed) in `crates/plix-mod-distribution/src/downloader.rs`
- [X] T053 [US1] Implement cache check (skip download if sha256 matches local) in `crates/plix-mod-distribution/src/downloader.rs`

#### Bundle Extraction

- [X] T054 [US1] Implement `BundleMetadata` quick extraction in `crates/plix-mod-distribution/src/bundle.rs`
- [X] T055 [US1] Implement `ModBundle` struct in `crates/plix-mod-distribution/src/bundle.rs`
- [X] T056 [US1] Implement zip extraction to cache directory in `crates/plix-mod-distribution/src/installer.rs`
- [X] T057 [US1] Implement manifest validation (mod.toml present and valid) in `crates/plix-mod-distribution/src/installer.rs`
- [X] T058 [US1] Implement cache layout (`bundles/<sha256>.plixmod`, `installed/<id>/<version>/`) in `crates/plix-mod-distribution/src/installer.rs`

#### Integration

- [X] T059 [US1] Implement `resolve_and_install()` orchestrating config→registry→download→install in `crates/plix-mod-distribution/src/lib.rs`
- [X] T060 [US1] Integration test: install 2 mods from local registry in `crates/plix-mod-distribution/tests/integration_tests.rs`
- [X] T061 [US1] Integration test: cache hit (no re-download) in `crates/plix-mod-distribution/tests/integration_tests.rs`
- [X] T062 [US1] Integration test: EMREG001 on unreachable registry in `crates/plix-mod-distribution/tests/integration_tests.rs`

**Checkpoint**: User Story 1 complete - can install mods from config

---

## Phase 4: User Story 2 - Reproducible Deployments via Lockfile (Priority: P1)

**Goal**: Generate and respect `mods.lock` for reproducible installations

**Independent Test**: Generate lockfile, copy to another server, verify identical installations

### Implementation for User Story 2

- [X] T063 [US2] Implement `Lockfile` struct in `crates/plix-mod-distribution/src/lockfile.rs`
- [X] T064 [US2] Implement `LockedMod` struct (id, version, sha256, source, download_url, dependencies) in `crates/plix-mod-distribution/src/lockfile.rs`
- [X] T065 [US2] Implement `Lockfile::write(path)` with deterministic JSON output in `crates/plix-mod-distribution/src/lockfile.rs`
- [X] T066 [US2] Implement `Lockfile::load(path)` in `crates/plix-mod-distribution/src/lockfile.rs`
- [X] T067 [US2] Implement lockfile precedence: use exact versions from lockfile when present in `crates/plix-mod-distribution/src/lib.rs`
- [X] T068 [US2] Implement lockfile update: add new mods from config to existing lockfile in `crates/plix-mod-distribution/src/lockfile.rs`
- [X] T069 [US2] Unit tests for lockfile write/read determinism in `crates/plix-mod-distribution/tests/lockfile_tests.rs`
- [X] T070 [US2] Integration test: generate lockfile, reinstall from lockfile only in `crates/plix-mod-distribution/tests/integration_tests.rs`

**Checkpoint**: User Story 2 complete - reproducible deployments via lockfile

---

## Phase 5: User Story 3 - Dependency Resolution (Priority: P1)

**Goal**: Automatically resolve and install transitive dependencies with conflict/cycle detection

**Independent Test**: Install mod with 2 transitive dependencies, verify all three installed

### Implementation for User Story 3

- [X] T071 [US3] Implement SemVer constraint parsing using `semver` crate in `crates/plix-mod-distribution/src/resolver.rs`
- [X] T072 [US3] Implement constraint types: exact (=), caret (^), tilde (~), range (>=,<) in `crates/plix-mod-distribution/src/resolver.rs`
- [X] T073 [US3] Implement `DependencyGraph` builder from config + index in `crates/plix-mod-distribution/src/resolver.rs`
- [X] T074 [US3] Implement greedy "latest compatible" version selection in `crates/plix-mod-distribution/src/resolver.rs`
- [X] T075 [US3] Implement cycle detection (DFS) returning EMREG008 in `crates/plix-mod-distribution/src/resolver.rs`
- [X] T076 [US3] Implement conflict detection returning EMREG006 with details in `crates/plix-mod-distribution/src/resolver.rs`
- [X] T077 [US3] Implement `ResolvedGraph` with topological ordering in `crates/plix-mod-distribution/src/resolver.rs`
- [X] T078 [US3] Implement depth limit (50 levels) to prevent runaway resolution in `crates/plix-mod-distribution/src/resolver.rs`
- [X] T079 [US3] Unit tests for SemVer constraint parsing/matching in `crates/plix-mod-distribution/tests/resolver_tests.rs`
- [X] T080 [US3] Unit tests for cycle detection in `crates/plix-mod-distribution/tests/resolver_tests.rs`
- [X] T081 [US3] Unit tests for conflict detection in `crates/plix-mod-distribution/tests/resolver_tests.rs`
- [X] T082 [US3] Integration test: transitive dependencies A→B→C in `crates/plix-mod-distribution/tests/integration_tests.rs`
- [X] T083 [US3] Integration test: EMREG006 on conflict in `crates/plix-mod-distribution/tests/integration_tests.rs`
- [X] T084 [US3] Integration test: EMREG008 on cycle in `crates/plix-mod-distribution/tests/integration_tests.rs`

**Checkpoint**: User Story 3 complete - dependency resolution with conflict/cycle detection

---

## Phase 6: User Story 4 - Integrity Verification (Priority: P1)

**Goal**: SHA-256 verification of all downloaded bundles before installation

**Independent Test**: Modify downloaded bundle, verify installation fails with hash mismatch

### Implementation for User Story 4

- [X] T085 [US4] Implement streaming SHA-256 hash calculation in `crates/plix-mod-distribution/src/integrity.rs`
- [X] T086 [US4] Implement `verify_bundle_hash(path, expected_sha256)` in `crates/plix-mod-distribution/src/integrity.rs`
- [X] T087 [US4] Implement bundle deletion on hash mismatch with EMREG004 in `crates/plix-mod-distribution/src/integrity.rs`
- [X] T088 [US4] Implement re-download trigger on partial/corrupted file in `crates/plix-mod-distribution/src/downloader.rs`
- [X] T089 [US4] Integrate hash verification into install pipeline in `crates/plix-mod-distribution/src/lib.rs`
- [X] T090 [US4] Unit tests for hash calculation in `crates/plix-mod-distribution/tests/integrity_tests.rs`
- [X] T091 [US4] Integration test: valid hash passes in `crates/plix-mod-distribution/tests/integration_tests.rs`
- [X] T092 [US4] Integration test: EMREG004 on tampered bundle in `crates/plix-mod-distribution/tests/integration_tests.rs`

**Checkpoint**: User Story 4 complete - integrity verification mandatory

---

## Phase 7: User Story 5 - Version Compatibility Checking (Priority: P2)

**Goal**: Reject mods incompatible with current API version or engine version

**Independent Test**: Attempt to install mod with api_version=99, verify rejection with clear error

### Implementation for User Story 5

- [X] T093 [US5] Implement `check_api_compatibility(mod_api_version, engine_api_version)` in `crates/plix-mod-distribution/src/lib.rs`
- [X] T094 [US5] Implement `check_engine_compatibility(engine_constraint, current_engine_version)` in `crates/plix-mod-distribution/src/lib.rs`
- [X] T095 [US5] Integrate compatibility check into resolution (reject before download) in `crates/plix-mod-distribution/src/resolver.rs`
- [X] T096 [US5] Return EMREG007 with details (required vs available) on mismatch in `crates/plix-mod-distribution/src/errors.rs`
- [X] T097 [US5] Unit tests for api_version check in `crates/plix-mod-distribution/tests/compatibility_tests.rs`
- [X] T098 [US5] Unit tests for engine min/max check in `crates/plix-mod-distribution/tests/compatibility_tests.rs`
- [X] T099 [US5] Integration test: EMREG007 on api_version mismatch in `crates/plix-mod-distribution/tests/integration_tests.rs`

**Checkpoint**: User Story 5 complete - compatibility checking enforced

---

## Phase 8: User Story 6 - Optional Signature Verification (Priority: P3)

**Goal**: Optional Ed25519 signature verification for trusted publishers

**Independent Test**: Configure `require_signature=true`, attempt unsigned mod, verify rejection

### Implementation for User Story 6

- [X] T100 [US6] Implement `SignatureVerifier` trait in `crates/plix-mod-distribution/src/signatures.rs`
- [X] T101 [US6] Implement Ed25519 signature verification using `ed25519-dalek` in `crates/plix-mod-distribution/src/signatures.rs`
- [X] T102 [US6] Implement `verify_signature(bundle_hash, signature, public_key)` in `crates/plix-mod-distribution/src/signatures.rs`
- [X] T103 [US6] Implement allowed_keys check against TrustPolicy in `crates/plix-mod-distribution/src/signatures.rs`
- [X] T104 [US6] Implement EMREG005 for invalid/missing signature when required in `crates/plix-mod-distribution/src/signatures.rs`
- [X] T105 [US6] Integrate signature verification into install pipeline (when enabled) in `crates/plix-mod-distribution/src/lib.rs`
- [X] T106 [US6] Unit tests for signature verification in `crates/plix-mod-distribution/tests/signature_tests.rs`
- [X] T107 [US6] Integration test: unsigned mod passes when require_signature=false in `crates/plix-mod-distribution/tests/integration_tests.rs`
- [X] T108 [US6] Integration test: EMREG005 when require_signature=true and unsigned in `crates/plix-mod-distribution/tests/integration_tests.rs`
- [X] T109 [US6] Integration test: valid signature from allowed key passes in `crates/plix-mod-distribution/tests/integration_tests.rs`

**Checkpoint**: User Story 6 complete - optional signature verification

---

## Phase 9: Server Integration

**Purpose**: Integrate mod distribution into plix-server startup

- [X] T110 Create `crates/plix-server/src/mods/distribution.rs` integration module
- [X] T111 Implement `init_mod_distribution(server_root)` loading server_mods.toml
- [X] T112 Implement startup pipeline: config→resolve→lock→download→verify→install→load
- [X] T113 Integrate with `ModManager` from Feature 034 for manifest loading
- [X] T114 Integrate with `WasmRuntime` from Feature 035 for WASM mod initialization
- [X] T115 Implement graceful error handling (fail startup on resolution/verify failure)
- [X] T116 Add tracing logs for each pipeline step
- [X] T117 Integration test: full server startup with mod distribution in `crates/plix-server/tests/mod_distribution_test.rs`

---

## Phase 10: Observability & Local Registry Tools

**Purpose**: Logging, metrics, and offline-first support

- [X] T118 [P] Add structured tracing for resolution steps in `crates/plix-mod-distribution/src/resolver.rs`
- [X] T119 [P] Add download progress logging in `crates/plix-mod-distribution/src/downloader.rs`
- [X] T120 [P] Add verification result logging in `crates/plix-mod-distribution/src/integrity.rs`
- [X] T121 [P] Implement `DistributionMetrics` counters (downloads, hash_mismatches, conflicts) in `crates/plix-mod-distribution/src/lib.rs`
- [X] T122 Implement `import_bundle(path)` for local registry population in `crates/plix-mod-distribution/src/registry.rs`
- [X] T123 Implement `list_local_mods()` for debugging in `crates/plix-mod-distribution/src/registry.rs`
- [X] T124 Implement pin support in resolver (pinned versions skip upgrade) in `crates/plix-mod-distribution/src/resolver.rs`
- [X] T125 Unit tests for pin behavior in `crates/plix-mod-distribution/tests/resolver_tests.rs`

---

## Phase 11: Polish & Documentation

**Purpose**: Final validation and documentation

- [X] T126 [P] Create comprehensive test fixtures with dependency chains in `tests/fixtures/mock_registry/`
- [ ] T127 [P] Create signed test bundles for signature tests in `tests/fixtures/mock_registry/`
- [X] T128 End-to-end test: full pipeline from config to loaded mods in `crates/plix-mod-distribution/tests/e2e_test.rs`
- [ ] T129 Negative test suite: all EMREG error codes triggered in `crates/plix-mod-distribution/tests/error_scenarios_test.rs`
- [X] T130 [P] Create `docs/feature-036.md` with bundle format, index schema, config, lockfile documentation
- [ ] T131 Run `cargo clippy -p plix-mod-distribution` and fix warnings
- [ ] T132 Run `cargo test -p plix-mod-distribution` - all tests pass
- [ ] T133 Run `cargo build --release` - verify clean build

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-8)**: All depend on Foundational phase completion
  - US1-US4 (P1) can proceed in parallel if staffed
  - US5 (P2) can start after US1-US4 or in parallel
  - US6 (P3) can start after US1-US4 or in parallel
- **Server Integration (Phase 9)**: Depends on US1-US4 completion (core functionality)
- **Observability (Phase 10)**: Can start after Phase 9
- **Polish (Phase 11)**: Depends on all phases complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational - core installation flow
- **User Story 2 (P1)**: Can start after Foundational - uses US1 types but independently testable
- **User Story 3 (P1)**: Can start after Foundational - uses US1 types but independently testable
- **User Story 4 (P1)**: Can start after Foundational - uses US1 download flow
- **User Story 5 (P2)**: Can start after Foundational - extends resolution
- **User Story 6 (P3)**: Can start after Foundational - optional feature

### Within Each User Story

- Types/structs before functions
- Core implementation before integration
- Unit tests alongside implementation
- Integration tests after core complete

### Parallel Opportunities

- All Setup tasks T004-T015 can run in parallel (different files)
- All Foundational tasks T018-T025 can run in parallel (different files)
- User stories US1-US4 can be worked on in parallel by different developers
- All [P] marked tasks within a phase can run in parallel

---

## Parallel Example: User Story 1

```bash
# Launch config tasks in parallel:
Task: "T028 [US1] Implement DistributionConfig struct"
Task: "T029 [US1] Implement RegistryConfig struct"
Task: "T030 [US1] Implement ModRequirement struct"
Task: "T031 [US1] Implement TrustPolicy struct"

# After structs complete, launch index tasks in parallel:
Task: "T036 [US1] Implement RegistryIndex struct"
Task: "T037 [US1] Implement RegistryMod struct"
Task: "T038 [US1] Implement ModVersionEntry struct"
```

---

## Implementation Strategy

### MVP First (User Stories 1-4)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1 (Automatic Installation)
4. Complete Phase 4: User Story 2 (Lockfile)
5. Complete Phase 5: User Story 3 (Dependencies)
6. Complete Phase 6: User Story 4 (Integrity)
7. **STOP and VALIDATE**: Test full flow with local registry
8. Complete Phase 9: Server Integration
9. Deploy/demo MVP

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Can install mods from config
3. Add User Story 2 → Reproducible with lockfile
4. Add User Story 3 → Transitive dependencies work
5. Add User Story 4 → Security baseline (integrity)
6. Add User Story 5 → Compatibility enforced
7. Add User Story 6 → Optional signatures
8. Each story adds value without breaking previous stories

### Suggested MVP Scope

**MVP = User Stories 1-4 + Server Integration**
- Config parsing and registry access
- Bundle download and extraction
- Dependency resolution with conflict/cycle detection
- SHA-256 integrity verification
- Lockfile for reproducibility
- Server startup integration

This covers SC-001 through SC-008 success criteria.

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Feature 034 (plix-mod-core) provides ModManifest parsing
- Feature 035 (plix-mod-runtime-wasm) provides WASM execution

---

## Summary

| Category | Count |
|----------|-------|
| **Total Tasks** | 133 |
| **Setup (Phase 1)** | 17 |
| **Foundational (Phase 2)** | 10 |
| **US1: Automatic Installation** | 35 |
| **US2: Lockfile** | 8 |
| **US3: Dependencies** | 14 |
| **US4: Integrity** | 8 |
| **US5: Compatibility** | 7 |
| **US6: Signatures** | 10 |
| **Server Integration** | 8 |
| **Observability** | 8 |
| **Polish** | 8 |
| **Parallel Opportunities** | 45+ tasks marked [P] |
