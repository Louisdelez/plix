# Tasks: Cross-Platform Client Packaging & Headless Server

**Input**: Design documents from `/specs/041-cross-platform/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Not explicitly requested - smoke tests included as part of packaging workflow.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

This is a multi-crate Rust workspace:
- `crates/plix-common/src/` - Shared types
- `crates/plix-server/src/` - Server binary
- `crates/plix-client/src/` - Client binary
- `scripts/package/` - Packaging scripts
- `.github/workflows/` - CI workflows
- `deploy/` - Docker and deployment configs
- `docs/` - Documentation

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and foundational types needed by all stories

- [x] T001 Add shadow-rs dependency to workspace Cargo.toml for build info embedding
- [x] T002 [P] Create BuildInfo struct in crates/plix-common/src/build_info.rs
- [x] T003 [P] Create build.rs for plix-common to generate shadow-rs constants
- [x] T004 [P] Create ExitCode enum in crates/plix-server/src/exit_codes.rs
- [x] T005 Create scripts/package/ directory structure for packaging scripts
- [x] T006 [P] Create deploy/configs/examples/server.toml with example configuration
- [x] T007 [P] Create deploy/configs/examples/server_mods.toml with example mod config

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**CRITICAL**: No user story work can begin until this phase is complete

- [x] T008 Implement BuildInfo::from_shadow() constructor in crates/plix-common/src/build_info.rs
- [x] T009 Implement BuildInfo::display_version() method in crates/plix-common/src/build_info.rs
- [x] T010 Implement BuildInfo::to_json() method for serialization in crates/plix-common/src/build_info.rs
- [x] T011 [P] Create scripts/package/generate_build_info.sh to generate build_info.json from binary
- [x] T012 [P] Create scripts/package/validate_bundle.sh for bundle structure validation
- [x] T013 Implement ExitCode::exit() method in crates/plix-server/src/exit_codes.rs

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Player Downloads and Launches Client (Priority: P1)

**Goal**: Players on Windows, Linux, or macOS can download, extract, and launch the client with version info displayed

**Independent Test**: Download appropriate OS bundle, extract it, run executable, verify version info displayed

### Implementation for User Story 1

- [x] T014 [US1] Add build.rs to plix-client for shadow-rs integration in crates/plix-client/build.rs
- [x] T015 [US1] Import and expose BuildInfo in plix-client main in crates/plix-client/src/main.rs
- [x] T016 [US1] Add --version flag handler to display version info in crates/plix-client/src/main.rs
- [x] T017 [US1] Log version info on client startup via tracing in crates/plix-client/src/main.rs
- [x] T018 [P] [US1] Create scripts/package/client_linux.sh packaging script
- [x] T019 [P] [US1] Create scripts/package/client_windows.ps1 packaging script
- [x] T020 [P] [US1] Create scripts/package/client_macos.sh packaging script with .app bundle structure
- [x] T021 [US1] Add CEF runtime copying logic to client_linux.sh in scripts/package/client_linux.sh
- [x] T022 [US1] Add CEF runtime copying logic to client_windows.ps1 in scripts/package/client_windows.ps1
- [x] T023 [US1] Add CEF framework copying logic to client_macos.sh (Frameworks/) in scripts/package/client_macos.sh
- [x] T024 [P] [US1] Create scripts/package/smoke_client_bundle.sh for client bundle validation
- [x] T025 [P] [US1] Create scripts/package/smoke_client_bundle.ps1 for Windows client validation

**Checkpoint**: Client packaging complete - players can download and launch on all platforms

---

## Phase 4: User Story 2 - Server Administrator Deploys Headless Server (Priority: P1)

**Goal**: Admins can deploy headless server with proper signal handling, exit codes, and config validation

**Independent Test**: Deploy on Linux VM without X11, start server, verify graceful shutdown on SIGTERM

### Implementation for User Story 2

- [x] T026 [US2] Create dedicated headless binary entry point in crates/plix-server/src/bin/plix-server-headless.rs
- [x] T027 [US2] Add [[bin]] entry for plix-server-headless in crates/plix-server/Cargo.toml
- [x] T028 [US2] Create shutdown module with GracefulShutdown struct in crates/plix-server/src/shutdown.rs
- [x] T029 [US2] Implement signal handling (SIGINT/SIGTERM) via tokio::signal in crates/plix-server/src/shutdown.rs
- [x] T030 [US2] Implement shutdown timeout (5s) with force-exit in crates/plix-server/src/shutdown.rs
- [x] T031 [US2] Enhance ServerConfig with validation method in crates/plix-server/src/config.rs
- [x] T032 [US2] Add ConfigError enum with user-friendly messages in crates/plix-server/src/config.rs
- [x] T033 [US2] Integrate exit codes for config validation errors (exit 1) in crates/plix-server/src/bin/plix-server-headless.rs
- [x] T034 [US2] Integrate exit codes for bind failures (exit 64) in crates/plix-server/src/bin/plix-server-headless.rs
- [x] T035 [US2] Add build.rs to plix-server for shadow-rs integration in crates/plix-server/build.rs
- [x] T036 [US2] Add --version flag to headless server in crates/plix-server/src/bin/plix-server-headless.rs
- [x] T037 [P] [US2] Create scripts/package/server_linux.sh packaging script
- [x] T038 [P] [US2] Create scripts/package/server_windows.ps1 packaging script
- [x] T039 [P] [US2] Create scripts/package/server_macos.sh packaging script
- [x] T040 [P] [US2] Create deploy/scripts/run_server.sh launcher script
- [x] T041 [P] [US2] Create deploy/scripts/run_server.ps1 launcher script for Windows
- [x] T042 [P] [US2] Create scripts/ci/smoke_headless_server.sh for server smoke tests
- [x] T043 [P] [US2] Create scripts/ci/smoke_headless_server.ps1 for Windows server smoke tests

**Checkpoint**: Headless server complete - admins can deploy production-ready servers

---

## Phase 5: User Story 3 - CI/CD Pipeline Builds Release Artifacts (Priority: P2)

**Goal**: CI automatically builds, packages, and uploads artifacts for all platforms on release

**Independent Test**: Trigger CI workflow, verify 6 artifacts produced with correct naming

### Implementation for User Story 3

- [x] T044 [US3] Create .github/workflows/release.yml with matrix strategy for all platforms
- [x] T045 [US3] Add build job for client (Windows/Linux/macOS) in .github/workflows/release.yml
- [x] T046 [US3] Add build job for server headless (Windows/Linux/macOS) in .github/workflows/release.yml
- [x] T047 [US3] Add packaging step calling OS-specific scripts in .github/workflows/release.yml
- [x] T048 [US3] Add smoke test step for client bundles in .github/workflows/release.yml
- [x] T049 [US3] Add smoke test step for server bundles in .github/workflows/release.yml
- [x] T050 [US3] Add checksum generation step (SHA256) in .github/workflows/release.yml
- [x] T051 [US3] Add artifact upload step with naming convention in .github/workflows/release.yml
- [x] T052 [US3] Add release job to create GitHub release on tag push in .github/workflows/release.yml
- [x] T053 [US3] Configure cargo caching via Swatinem/rust-cache in .github/workflows/release.yml

**Checkpoint**: CI/CD complete - releases are automated and reproducible

---

## Phase 6: User Story 4 - Admin Deploys Server via Docker (Priority: P3)

**Goal**: Admins can deploy headless server using Docker with volume persistence

**Independent Test**: Build Docker image, run with exposed ports, verify volume persistence

### Implementation for User Story 4

- [x] T054 [US4] Create deploy/docker/Dockerfile for headless server image
- [x] T055 [US4] Configure non-root user in Dockerfile in deploy/docker/Dockerfile
- [x] T056 [US4] Configure volume mounts for /data/world and /data/mods in deploy/docker/Dockerfile
- [x] T057 [US4] Create deploy/docker/docker-compose.yml for easy deployment
- [x] T058 [US4] Add Docker build/run instructions to docs/server/headless-deploy.md

**Checkpoint**: Docker deployment complete - cloud-native deployment available

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Documentation and final validation

- [x] T059 [P] Create docs/release/client-packaging.md with build and CEF bundling instructions
- [x] T060 [P] Create docs/release/ci-artifacts.md documenting CI artifacts and retrieval
- [x] T061 [P] Create docs/server/headless-deploy.md with VM and Docker deployment guides
- [x] T062 [P] Add systemd service example to docs/server/headless-deploy.md
- [ ] T063 Run cargo fmt --all and verify no formatting issues
- [ ] T064 Run cargo clippy --all and fix any warnings
- [ ] T065 Run cargo test --all and verify all tests pass
- [ ] T066 Validate quickstart.md scenarios work end-to-end

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-6)**: All depend on Foundational phase completion
  - US1 and US2 are both P1 priority - can run in parallel
  - US3 depends on packaging scripts from US1/US2 (but CI config can start)
  - US4 depends on server binary from US2
- **Polish (Phase 7)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational - No dependencies on other stories
- **User Story 2 (P1)**: Can start after Foundational - No dependencies on other stories
- **User Story 3 (P2)**: Requires packaging scripts from US1/US2 to be functional
- **User Story 4 (P3)**: Requires headless binary from US2

### Within Each User Story

- BuildInfo types must exist before version display
- Packaging scripts depend on binary being buildable
- Smoke tests depend on packaging scripts

### Parallel Opportunities

**Phase 1 (Setup):**
- T002, T003, T004, T006, T007 can all run in parallel

**Phase 2 (Foundational):**
- T011, T012 can run in parallel

**Phase 3 (US1):**
- T018, T019, T020 (packaging scripts) can run in parallel
- T024, T025 (smoke scripts) can run in parallel

**Phase 4 (US2):**
- T037, T038, T039 (packaging scripts) can run in parallel
- T040, T041 (run scripts) can run in parallel
- T042, T043 (smoke scripts) can run in parallel

**Phase 7 (Polish):**
- T059, T060, T061, T062 (docs) can all run in parallel

---

## Parallel Example: User Story 1

```bash
# Phase 1: After T014-T017 complete (client version display), launch packaging in parallel:
Task: "Create scripts/package/client_linux.sh packaging script"
Task: "Create scripts/package/client_windows.ps1 packaging script"
Task: "Create scripts/package/client_macos.sh packaging script"

# After packaging scripts done, launch smoke tests in parallel:
Task: "Create scripts/package/smoke_client_bundle.sh for client bundle validation"
Task: "Create scripts/package/smoke_client_bundle.ps1 for Windows client validation"
```

## Parallel Example: User Story 2

```bash
# After T026-T036 complete (headless binary), launch packaging in parallel:
Task: "Create scripts/package/server_linux.sh packaging script"
Task: "Create scripts/package/server_windows.ps1 packaging script"
Task: "Create scripts/package/server_macos.sh packaging script"

# Run scripts and smoke tests in parallel:
Task: "Create deploy/scripts/run_server.sh launcher script"
Task: "Create deploy/scripts/run_server.ps1 launcher script"
Task: "Create scripts/ci/smoke_headless_server.sh for server smoke tests"
Task: "Create scripts/ci/smoke_headless_server.ps1 for Windows server smoke tests"
```

---

## Implementation Strategy

### MVP First (User Stories 1 + 2)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1 (Client packaging)
4. Complete Phase 4: User Story 2 (Headless server)
5. **STOP and VALIDATE**: Test both stories independently
6. Deploy/demo if ready - players can play and admins can host

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Players can download client
3. Add User Story 2 → Test independently → Admins can deploy servers
4. Add User Story 3 → Test independently → CI automation works
5. Add User Story 4 → Test independently → Docker deployment available
6. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 (Client packaging)
   - Developer B: User Story 2 (Headless server)
3. After US1+US2:
   - Developer A: User Story 3 (CI/CD)
   - Developer B: User Story 4 (Docker)
4. Stories complete and integrate independently

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- shadow-rs provides compile-time build info embedding (research.md decision)
- Exit codes follow data-model.md specification (0, 1, 2, 64-68)
- Bundle layouts follow contracts/packaging.md specification
