# Tasks: Dedicated Server Packaging

**Input**: Design documents from `/specs/028-dedicated-server-packaging/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: No automated tests requested. Validation is manual (Docker build/run verification).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

```text
deploy/
├── docker/
│   ├── Dockerfile           # plix-server
│   ├── Dockerfile.master    # plix-master
│   └── docker-compose.yml
├── scripts/
│   ├── build.sh
│   ├── run.sh
│   ├── compose.sh
│   └── release-local.sh
└── config/
    └── server.toml.example

docs/
└── DEDICATED_SERVER.md

rust-toolchain.toml          # Repo root
```

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Version pinning and project structure for reproducible builds

- [x] T001 Create rust-toolchain.toml at repo root with Rust 1.75.0 pinned
- [x] T002 Verify Cargo.lock is committed and document "do not delete" policy
- [x] T003 [P] Create deploy/ directory structure per plan.md

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Docker image foundation that MUST be complete before ANY user story can be implemented

**CRITICAL**: No user story work can begin until this phase is complete

- [x] T004 Create multi-stage Dockerfile at deploy/docker/Dockerfile (builder stage)
- [x] T005 Add runtime stage to deploy/docker/Dockerfile with debian:bookworm-slim pinned
- [x] T006 Add non-root user (plix:1000) to Dockerfile runtime stage
- [x] T007 [P] Add EXPOSE 7777/udp and ENTRYPOINT to Dockerfile
- [x] T008 [P] Create Dockerfile.master at deploy/docker/Dockerfile.master
- [ ] T009 Test docker build locally for both images

**Checkpoint**: Docker images build successfully - user story implementation can now begin

---

## Phase 3: User Story 1 - Quick Server Deployment (Priority: P1)

**Goal**: Deploy a plix game server in 2 commands with default FFA mode

**Independent Test**: Run `docker build` and `docker run plix-server`, verify server starts in FFA mode within 30 seconds

### Implementation for User Story 1

- [x] T010 [US1] Set default CMD args for FFA mode in deploy/docker/Dockerfile
- [x] T011 [US1] Copy assets/arenas/ into Docker image at /app/assets
- [x] T012 [US1] Set environment defaults (PLIX_ASSETS_DIR, RUST_LOG) in Dockerfile
- [x] T013 [P] [US1] Create deploy/scripts/build.sh for Docker image building
- [x] T014 [P] [US1] Create deploy/scripts/run.sh for simple container execution
- [x] T015 [US1] Add HEALTHCHECK (pgrep plix-server) to Dockerfile
- [ ] T016 [US1] Manual validation: docker run without config starts FFA server

**Checkpoint**: User Story 1 complete - server deploys in 2 commands with FFA defaults

---

## Phase 4: User Story 2 - Runtime Configuration (Priority: P1)

**Goal**: Configure server via environment variables and CLI flags without rebuilding

**Independent Test**: Start server with PLIX_* env vars and verify settings take effect

### Implementation for User Story 2

- [x] T017 [US2] Add config.rs module to crates/plix-server/src/ for TOML parsing
- [x] T018 [US2] Implement --config flag in crates/plix-server/src/main.rs
- [x] T019 [US2] Implement PLIX_* environment variable reading in config.rs
- [x] T020 [US2] Implement config priority: CLI > ENV > file > defaults in config.rs
- [x] T021 [P] [US2] Create deploy/config/server.toml.example with all options documented
- [x] T022 [P] [US2] Create deploy/config/server-ffa.toml example config
- [x] T023 [P] [US2] Create deploy/config/server-tdm.toml example config
- [x] T024 [P] [US2] Create deploy/config/server-ctf.toml example config
- [x] T025 [P] [US2] Create deploy/config/server-br.toml example config
- [x] T026 [US2] Define /data structure in Dockerfile (/data/config, /data/worlds, /data/logs)
- [ ] T027 [US2] Manual validation: mount config volume, verify overrides work

**Checkpoint**: User Story 2 complete - configuration works via env/CLI/file with correct priority

---

## Phase 5: User Story 3 - Reproducible Builds (Priority: P2)

**Goal**: Builds produce identical binaries across machines and time

**Independent Test**: Build Docker image on two machines, compare binary checksums

### Implementation for User Story 3

- [x] T028 [US3] Pin debian:bookworm-slim to specific tag in Dockerfile
- [x] T029 [US3] Add SOURCE_DATE_EPOCH for reproducible timestamps in Dockerfile
- [x] T030 [US3] Document Cargo.lock usage and update policy in docs/DEDICATED_SERVER.md
- [x] T031 [US3] Document rust-toolchain.toml update procedure in docs/DEDICATED_SERVER.md

**Checkpoint**: User Story 3 complete - builds are reproducible with pinned versions

---

## Phase 6: User Story 4 - Multi-Service Stack (Priority: P2)

**Goal**: Deploy plix-server + plix-master together via docker-compose

**Independent Test**: Run `docker-compose up`, verify both services start and communicate

### Implementation for User Story 4

- [x] T032 [US4] Create docker-compose.yml at deploy/docker/docker-compose.yml
- [x] T033 [US4] Add plix-server service with ports, volumes, environment
- [x] T034 [US4] Add plix-master service with profile "full" or "master"
- [x] T035 [US4] Configure plix-network bridge network
- [x] T036 [P] [US4] Create deploy/scripts/compose.sh wrapper script
- [x] T037 [P] [US4] Create .env.example at deploy/docker/.env.example
- [ ] T038 [US4] Manual validation: docker-compose up --profile full

**Checkpoint**: User Story 4 complete - full stack deploys with compose

---

## Phase 7: User Story 5 - Data Persistence (Priority: P3)

**Goal**: World data and logs persist across container restarts with volumes

**Independent Test**: Start server with volume, make changes, restart, verify data persists

### Implementation for User Story 5

- [x] T039 [US5] Ensure /data/worlds directory exists and is writable by plix user
- [x] T040 [US5] Ensure /data/logs directory exists (optional file logging)
- [x] T041 [US5] Document volume mounts in docker-compose.yml comments
- [x] T042 [US5] Verify server starts without volumes (ephemeral mode)
- [ ] T043 [US5] Manual validation: mount /data volume, restart, verify persistence

**Checkpoint**: User Story 5 complete - data persists with volumes, ephemeral mode works

---

## Phase 8: User Story 6 - Non-Docker Deployment (Priority: P3)

**Goal**: Deploy from release archive (tar.gz) without Docker

**Independent Test**: Extract archive, run binary, verify server starts with config

### Implementation for User Story 6

- [x] T044 [US6] Create deploy/scripts/release-local.sh for archive creation
- [x] T045 [US6] Generate SHA256 checksum file in release script
- [x] T046 [US6] Include assets/arenas/ in release archive
- [x] T047 [US6] Include server.toml.example in release archive
- [x] T048 [US6] Include README excerpt in release archive
- [ ] T049 [US6] Manual validation: extract archive, run binary

**Checkpoint**: User Story 6 complete - non-Docker deployment works

---

## Phase 9: Documentation & Polish

**Purpose**: Admin documentation and cross-cutting concerns

### Documentation

- [x] T050 [P] Create docs/DEDICATED_SERVER.md with full deployment guide
- [x] T051 [P] Add Quick Start (2 commands) section to DEDICATED_SERVER.md
- [x] T052 [P] Add Configuration Reference section to DEDICATED_SERVER.md
- [x] T053 [P] Add Troubleshooting section (ports, NAT, firewall, logs)
- [x] T054 [P] Add Log Management section (RUST_LOG, Docker logging drivers)
- [x] T055 [P] Add Game Mode Examples section (FFA, TDM, CTF, BR Lite)

### Observability

- [x] T056 Verify all logs go to stdout/stderr (no internal rotation)
- [x] T057 Document RUST_LOG usage in DEDICATED_SERVER.md

### Final Validation

- [x] T058 Run cargo fmt --all -- --check
- [x] T059 Run cargo clippy --all-targets
- [x] T060 Verify cargo run -p plix-server still works (non-regression)
- [ ] T061 Verify assets found in Docker runtime
- [ ] T062 Final documentation review

**Checkpoint**: Feature complete - all documentation and validation done

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup - BLOCKS all user stories
- **User Stories (Phase 3-8)**: All depend on Foundational phase
  - US1 (P1) and US2 (P1): Should complete first
  - US3 (P2) and US4 (P2): Can run in parallel after US1/US2
  - US5 (P3) and US6 (P3): Can run in parallel after US3/US4
- **Documentation (Phase 9)**: Can start after US1, continues through completion

### User Story Dependencies

- **User Story 1 (P1)**: After Foundational - No dependencies on other stories
- **User Story 2 (P1)**: After Foundational - Adds config.rs (new Rust code)
- **User Story 3 (P2)**: After Foundational - Documentation only
- **User Story 4 (P2)**: After Foundational - Requires Dockerfile complete
- **User Story 5 (P3)**: After Foundational - Dockerfile volume setup
- **User Story 6 (P3)**: After Foundational - Independent release script

### Parallel Opportunities

**Within Phase 1 (Setup)**:
- T003 can run in parallel with T001, T002

**Within Phase 2 (Foundational)**:
- T007, T008 can run in parallel after T006

**Within US1**:
- T013, T014 can run in parallel (different scripts)

**Within US2**:
- T021-T025 can all run in parallel (different config files)

**Within US4**:
- T036, T037 can run in parallel (different files)

**Within Phase 9**:
- T050-T055 can all run in parallel (different doc sections)

---

## Parallel Example: User Story 2 Config Files

```bash
# Launch all config example files together:
Task: "Create deploy/config/server.toml.example"
Task: "Create deploy/config/server-ffa.toml"
Task: "Create deploy/config/server-tdm.toml"
Task: "Create deploy/config/server-ctf.toml"
Task: "Create deploy/config/server-br.toml"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T003)
2. Complete Phase 2: Foundational (T004-T009)
3. Complete Phase 3: User Story 1 (T010-T016)
4. **STOP and VALIDATE**: Docker run with FFA default works
5. Deploy/demo if ready

### Incremental Delivery

1. Setup + Foundational → Docker builds
2. Add User Story 1 → 2-command deployment works
3. Add User Story 2 → Configuration works
4. Add User Story 3 → Reproducible builds documented
5. Add User Story 4 → Docker Compose stack works
6. Add User Story 5 → Persistence works
7. Add User Story 6 → Non-Docker deployment works
8. Documentation polish → Feature complete

### Single Developer Strategy

Execute in order:
1. Phase 1 (Setup): 3 tasks
2. Phase 2 (Foundational): 6 tasks
3. Phase 3 (US1): 7 tasks → **MVP ready**
4. Phase 4 (US2): 11 tasks
5. Phase 5 (US3): 4 tasks
6. Phase 6 (US4): 7 tasks
7. Phase 7 (US5): 5 tasks
8. Phase 8 (US6): 6 tasks
9. Phase 9 (Polish): 13 tasks

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story
- Each user story is independently testable
- Manual validation required (no automated Docker tests)
- Commit after each task or logical group
- Total: 62 tasks across 9 phases
