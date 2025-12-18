# Tasks: Patch Updater & Launcher

**Input**: Design documents from `/specs/029-patch-launcher/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Unit tests for core functionality (manifest parsing, checksum verification, version comparison).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

```text
crates/plix-launcher/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── config.rs
│   ├── manifest/
│   ├── version/
│   ├── download/
│   ├── install/
│   ├── launch/
│   └── ui/
└── tests/
```

---

## Phase 1: Setup (Shared Infrastructure) ✅ COMPLETE

**Purpose**: Create plix-launcher crate and basic project structure

- [x] T001 Create crates/plix-launcher/Cargo.toml with workspace dependencies (reqwest, serde, toml, sha2, semver, clap, tracing, dirs-next, thiserror)
- [x] T002 Add plix-launcher to workspace members in /Cargo.toml
- [x] T003 [P] Create minimal crates/plix-launcher/src/main.rs with clap CLI skeleton
- [x] T004 [P] Create crates/plix-launcher/src/lib.rs exporting public modules
- [x] T005 Test: verify `cargo build -p plix-launcher` succeeds

---

## Phase 2: Foundational (Blocking Prerequisites) ✅ COMPLETE

**Purpose**: Core types, directory structure, and error handling that ALL user stories depend on

**CRITICAL**: No user story work can begin until this phase is complete

- [x] T006 Create error types in crates/plix-launcher/src/error.rs (LauncherError enum with thiserror)
- [x] T007 [P] Create Manifest struct in crates/plix-launcher/src/manifest/mod.rs (version, protocol_version, files[], release_date)
- [x] T008 [P] Create ManifestFile struct in crates/plix-launcher/src/manifest/mod.rs (path, url, size, sha256, executable)
- [x] T009 [P] Create LocalState struct in crates/plix-launcher/src/version/local.rs (installed_version, last_update, file_checksums)
- [x] T010 [P] Create LauncherConfig struct in crates/plix-launcher/src/config.rs (manifest_url, stay_open, timeout_seconds, max_retries, verbose)
- [x] T011 Create directory structure helper in crates/plix-launcher/src/install/structure.rs (data_dir, config_dir, versions_dir, current_dir, logs_dir)
- [x] T012 Implement config loading/saving with atomic writes in crates/plix-launcher/src/config.rs
- [x] T013 Implement LocalState loading/saving in crates/plix-launcher/src/version/local.rs
- [x] T014 [P] Add unit test for Manifest parsing (valid/invalid TOML) in crates/plix-launcher/tests/manifest_test.rs
- [x] T015 [P] Add unit test for LocalState persistence in crates/plix-launcher/tests/state_test.rs

**Checkpoint**: Foundation ready - core types and directory management functional

---

## Phase 3: User Story 1 - Automatic Update Check and Launch (Priority: P1) MVP ✅ COMPLETE

**Goal**: Player launches the game without worrying about updates - version check, download if needed, launch game

**Independent Test**: Launch launcher, observe it checks version, downloads updates if needed, then starts the game

### Implementation for User Story 1

- [x] T016 [US1] Implement manifest fetch with HTTP GET and timeout in crates/plix-launcher/src/manifest/fetch.rs
- [x] T017 [US1] Implement manifest TOML parsing and validation in crates/plix-launcher/src/manifest/validate.rs
- [x] T018 [US1] Implement semver version comparison in crates/plix-launcher/src/version/compare.rs
- [x] T019 [US1] Implement UpdateStatus enum (UpToDate, UpdateAvailable, FreshInstall, Offline) in crates/plix-launcher/src/version/mod.rs
- [x] T020 [US1] Implement version check logic (local vs remote comparison) in crates/plix-launcher/src/version/mod.rs
- [x] T021 [US1] Implement file download with retry (3 attempts, 5s delay) in crates/plix-launcher/src/download/file.rs
- [x] T022 [US1] Implement download to temp directory in crates/plix-launcher/src/download/mod.rs
- [x] T023 [US1] Implement atomic file installation (temp → versions/{ver}/) in crates/plix-launcher/src/install/atomic.rs
- [x] T024 [US1] Implement current/ symlink/copy update in crates/plix-launcher/src/install/mod.rs
- [x] T025 [US1] Implement game binary launch in crates/plix-launcher/src/launch/mod.rs
- [x] T026 [US1] Implement platform-specific launch (Linux chmod+exec, Windows .exe) in crates/plix-launcher/src/launch/platform.rs
- [x] T027 [US1] Implement CLI argument passthrough to game in crates/plix-launcher/src/launch/mod.rs
- [x] T028 [US1] Implement offline mode fallback (launch existing if fetch fails) in crates/plix-launcher/src/main.rs
- [x] T029 [US1] Implement main flow orchestration (check → download → install → launch) in crates/plix-launcher/src/main.rs
- [x] T030 [US1] Add unit test for version comparison in crates/plix-launcher/tests/version_test.rs (tests in version/compare.rs)

**Checkpoint**: User Story 1 complete - full update + launch flow functional

---

## Phase 4: User Story 2 - Version Verification and Integrity (Priority: P1) ✅ COMPLETE

**Goal**: Launcher verifies game files are complete and correct via SHA256 checksums

**Independent Test**: Corrupt a game file locally, launch launcher, observe re-download of corrupted file only

### Implementation for User Story 2

- [x] T031 [US2] Implement SHA256 checksum calculation in crates/plix-launcher/src/download/checksum.rs
- [x] T032 [US2] Implement file integrity verification (compare local vs manifest checksums) in crates/plix-launcher/src/download/checksum.rs
- [x] T033 [US2] Implement selective file download (only files with checksum mismatch) in crates/plix-launcher/src/download/mod.rs (via find_files_to_update)
- [x] T034 [US2] Implement download rejection on checksum failure in crates/plix-launcher/src/download/file.rs
- [x] T035 [US2] Implement partial install prevention (refuse launch if verification fails) in crates/plix-launcher/src/install/mod.rs
- [x] T036 [US2] Implement temp directory cleanup on failure in crates/plix-launcher/src/download/mod.rs (cleanup_temp)
- [x] T037 [US2] Add unit test for SHA256 checksum (valid/invalid) in crates/plix-launcher/tests/checksum_test.rs (tests in download/checksum.rs)

**Checkpoint**: User Story 2 complete - file integrity verification fully functional

---

## Phase 5: User Story 3 - Progress Feedback (Priority: P2) ✅ COMPLETE

**Goal**: Player sees clear progress during updates (checking, downloading, verifying, launching)

**Independent Test**: Trigger an update, observe status messages and progress indicators

### Implementation for User Story 3

- [x] T038 [US3] Create UpdatePhase enum (CheckingVersion, Downloading, Verifying, Installing, Complete, Failed) in crates/plix-launcher/src/ui/mod.rs
- [x] T039 [US3] Create DownloadProgress struct (file_path, bytes_downloaded, total_bytes, speed_bps) in crates/plix-launcher/src/ui/mod.rs
- [x] T040 [US3] Implement console output for status messages in crates/plix-launcher/src/ui/console.rs (tracing-based)
- [x] T041 [US3] Implement download progress display (percentage, MB/MB) in crates/plix-launcher/src/ui/console.rs (basic tracing)
- [x] T042 [US3] Implement clear error message display in crates/plix-launcher/src/ui/console.rs (tracing::error)
- [x] T043 [US3] Integrate progress callbacks into download flow in crates/plix-launcher/src/download/file.rs

**Checkpoint**: User Story 3 complete - users see clear feedback during operations

---

## Phase 6: User Story 4 - Developer Release Publishing (Priority: P2)

**Goal**: Developers can publish releases easily via manifest file

**Independent Test**: Create manifest, upload files, observe launcher detects and downloads new version

### Implementation for User Story 4

- [ ] T044 [P] [US4] Create example manifest.toml at deploy/launcher/manifest.toml.example
- [ ] T045 [P] [US4] Create manifest generation script at deploy/scripts/generate-manifest.sh
- [ ] T046 [US4] Document manifest format in docs/LAUNCHER.md (fields, validation rules, examples)
- [ ] T047 [US4] Add manifest URL configuration to launcher.toml in crates/plix-launcher/src/config.rs

**Checkpoint**: User Story 4 complete - developers can publish updates via manifest

---

## Phase 7: User Story 5 - Server Admin Version Control (Priority: P3)

**Goal**: Server admins can ensure players arrive with compatible versions

**Independent Test**: Server rejects incompatible client versions

### Implementation for User Story 5

- [ ] T048 [US5] Document client version reporting to server in docs/LAUNCHER.md
- [ ] T049 [US5] Verify existing protocol includes version in Connect message (plix-common)
- [ ] T050 [US5] Document server-side version validation in docs/DEDICATED_SERVER.md

**Checkpoint**: User Story 5 complete - version compatibility documented

---

## Phase 8: User Story 6 - CLI Operations (Priority: P3)

**Goal**: Power users can control launcher via command line flags

**Independent Test**: Run launcher with --check, --update, --launch, --dry-run flags

### Implementation for User Story 6

- [ ] T051 [US6] Implement --check flag (check version only, no download/launch) in crates/plix-launcher/src/main.rs
- [ ] T052 [US6] Implement --update flag (download and install, no launch) in crates/plix-launcher/src/main.rs
- [ ] T053 [US6] Implement --launch flag (launch without checking updates) in crates/plix-launcher/src/main.rs
- [ ] T054 [US6] Implement --dry-run flag (simulate without changes) in crates/plix-launcher/src/main.rs
- [ ] T055 [US6] Implement --verbose flag (detailed logging) in crates/plix-launcher/src/main.rs
- [ ] T056 [US6] Implement --manifest-url flag (override config) in crates/plix-launcher/src/main.rs
- [ ] T057 [US6] Implement --version and --help flags in crates/plix-launcher/src/main.rs
- [ ] T058 [US6] Implement exit codes (0=success, 1=error, 2=network, 3=checksum, 4=install, 5=launch, 10=already running)

**Checkpoint**: User Story 6 complete - full CLI interface functional

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Logging, robustness, packaging, documentation

### Logging & Observability

- [ ] T059 [P] Implement structured logging with tracing in crates/plix-launcher/src/main.rs
- [ ] T060 [P] Implement log file output to ~/.local/share/plix/logs/launcher.log
- [ ] T061 [P] Add timestamps and log levels to all log output

### Robustness & Security

- [ ] T062 Implement single instance lock file in crates/plix-launcher/src/main.rs
- [ ] T063 Implement network timeout handling (30s default) in crates/plix-launcher/src/manifest/fetch.rs
- [ ] T064 Implement graceful shutdown on SIGINT/SIGTERM in crates/plix-launcher/src/main.rs
- [ ] T065 Verify no elevated privileges required (no admin/root)

### Packaging & Integration

- [ ] T066 [P] Update deploy/scripts/release-local.sh to include plix-launcher binary
- [ ] T067 [P] Create launcher-specific release archive script at deploy/scripts/release-launcher.sh
- [ ] T068 Document launcher distribution in docs/LAUNCHER.md

### Documentation

- [ ] T069 [P] Create docs/LAUNCHER.md with full usage guide
- [ ] T070 [P] Add Quick Start section to docs/LAUNCHER.md
- [ ] T071 [P] Add Troubleshooting section to docs/LAUNCHER.md (offline, checksum errors, network issues)
- [ ] T072 [P] Add Manifest Format Reference section to docs/LAUNCHER.md

### Final Validation

- [ ] T073 Run cargo fmt --all -- --check
- [ ] T074 Run cargo clippy --all-targets -p plix-launcher
- [ ] T075 Verify cargo test -p plix-launcher passes
- [ ] T076 Manual validation: fresh install (no local version)
- [ ] T077 Manual validation: update from version N to N+1
- [ ] T078 Manual validation: offline launch with existing version
- [ ] T079 Verify plix-client can still be launched directly (dev mode, without launcher)

**Checkpoint**: Feature complete - all stories implemented, tested, documented

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup - BLOCKS all user stories
- **User Stories (Phase 3-8)**: All depend on Foundational phase completion
  - US1 (P1) and US2 (P1): Should complete first (core functionality)
  - US3 (P2) and US4 (P2): Can start after US1/US2 complete
  - US5 (P3) and US6 (P3): Can start after US1/US2 complete
- **Polish (Phase 9)**: Can start after US1/US2, continues through completion

### User Story Dependencies

- **User Story 1 (P1)**: After Foundational - Core update and launch flow
- **User Story 2 (P1)**: After Foundational - Can run in parallel with US1 (checksum module independent)
- **User Story 3 (P2)**: After US1 - Integrates with download flow
- **User Story 4 (P2)**: After Foundational - Independent (documentation + scripts)
- **User Story 5 (P3)**: After Foundational - Documentation only
- **User Story 6 (P3)**: After US1 - CLI modes depend on core flow

### Parallel Opportunities

**Within Phase 1 (Setup)**:
- T003, T004 can run in parallel (different files)

**Within Phase 2 (Foundational)**:
- T007, T008, T009, T010 can run in parallel (different struct definitions)
- T014, T015 can run in parallel (different test files)

**Within US4**:
- T044, T045 can run in parallel (different files)

**Within Phase 9**:
- T059, T060, T061 can run in parallel (logging tasks)
- T066, T067 can run in parallel (different scripts)
- T069, T070, T071, T072 can run in parallel (different doc sections)

---

## Parallel Example: User Story 1 and 2

```bash
# After Foundational phase completes:

# US1 and US2 can start in parallel (different modules):
# US1 focuses on: manifest/fetch.rs, version/, download/file.rs, install/, launch/
# US2 focuses on: download/checksum.rs

# Within US1, T016 and T017 can run in parallel (fetch.rs vs validate.rs)
```

---

## Implementation Strategy

### MVP First (User Story 1 + 2 Only)

1. Complete Phase 1: Setup (5 tasks)
2. Complete Phase 2: Foundational (10 tasks)
3. Complete Phase 3: User Story 1 (15 tasks) → **Core flow works**
4. Complete Phase 4: User Story 2 (7 tasks) → **Integrity verification works**
5. **STOP and VALIDATE**: Test update + launch + checksum verification
6. Deploy/demo if ready

### Incremental Delivery

1. Setup + Foundational → Crate builds
2. Add User Story 1 → Update and launch works
3. Add User Story 2 → Integrity verification works
4. Add User Story 3 → Progress feedback
5. Add User Story 4 → Developer tooling
6. Add User Story 5 → Documentation
7. Add User Story 6 → CLI operations
8. Polish → Logging, packaging, docs

### Single Developer Strategy

Execute in order:
1. Phase 1 (Setup): 5 tasks
2. Phase 2 (Foundational): 10 tasks
3. Phase 3 (US1): 15 tasks → **MVP ready**
4. Phase 4 (US2): 7 tasks
5. Phase 5 (US3): 6 tasks
6. Phase 6 (US4): 4 tasks
7. Phase 7 (US5): 3 tasks
8. Phase 8 (US6): 8 tasks
9. Phase 9 (Polish): 21 tasks

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story
- Each user story is independently testable after US1 foundation
- Unit tests included for core functionality (manifest, checksum, version)
- Manual validation required for end-to-end flows
- Commit after each task or logical group
- Total: 79 tasks across 9 phases
