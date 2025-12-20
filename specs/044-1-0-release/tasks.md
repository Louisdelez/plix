# Tasks: 1.0 Release

**Input**: Design documents from `/specs/044-1-0-release/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3, US4)

---

## Phase 1: Setup (Version Infrastructure)

**Purpose**: Establish version constants and types as foundation for all stories

- [x] T001 Update workspace version to 1.0.0 in Cargo.toml
- [x] T002 [P] Create version module in crates/plix-common/src/version.rs with ProtocolVersion struct
- [x] T003 [P] Add PROTOCOL_VERSION constant (major: 1, minor: 0) in crates/plix-common/src/version.rs
- [x] T004 [P] Add is_compatible_with() method to ProtocolVersion in crates/plix-common/src/version.rs
- [x] T005 [P] Add MOD_API_VERSION constant (1, 0) in crates/plix-mod-core/src/lib.rs
- [x] T006 [P] Add CONTENT_SCHEMA_VERSION constant in crates/plix-server/src/content/mod.rs
- [x] T007 Export version module from crates/plix-common/src/lib.rs
- [x] T008 Unit tests for version compatibility checking in crates/plix-common/src/version.rs

---

## Phase 2: Foundational (Migration Framework)

**Purpose**: Core migration infrastructure that MUST be complete before user stories can be validated

**⚠️ CRITICAL**: US2 (Migration) depends on this phase

- [x] T009 Create migration module directory crates/plix-common/src/migration/
- [x] T010 [P] Create backup module in crates/plix-common/src/migration/backup.rs
- [x] T011 [P] Implement create_backup() function with SHA-256 checksum in crates/plix-common/src/migration/backup.rs
- [x] T012 Implement rotate_backups() function keeping 3 most recent in crates/plix-common/src/migration/backup.rs
- [x] T013 [P] Create migration trait in crates/plix-common/src/migration/mod.rs
- [x] T014 [P] Implement MigrationEngine struct in crates/plix-common/src/migration/mod.rs
- [x] T015 Implement run_migrations() with sequential N→N+1 execution in crates/plix-common/src/migration/mod.rs
- [x] T016 Add MigrationError and MigrationResult types in crates/plix-common/src/migration/mod.rs
- [x] T017 [P] Create config migration module in crates/plix-common/src/migration/config.rs
- [x] T018 Export migration module from crates/plix-common/src/lib.rs
- [x] T019 Unit tests for backup rotation (verify 3 kept, oldest deleted) in crates/plix-common/src/migration/backup.rs
- [x] T020 Unit tests for migration idempotence (run twice = same result) in crates/plix-common/src/migration/mod.rs

**Checkpoint**: Migration framework ready - user story implementation can begin

---

## Phase 3: User Story 1 - Install and Play Without Issues (Priority: P1) 🎯 MVP

**Goal**: Players can install Plix 1.0, launch without errors, and complete tutorial quest

**Independent Test**: Download release artifacts → install → connect to server → complete tutorial quest → all systems function

### Implementation for User Story 1

- [x] T021 [US1] Add version display to client startup log in crates/plix-client/src/main.rs
- [x] T022 [US1] Add version display to server startup log in crates/plix-server/src/main.rs
- [x] T023 [P] [US1] Add --version CLI flag to plix-client in crates/plix-client/src/main.rs
- [x] T024 [P] [US1] Add --version CLI flag to plix-server in crates/plix-server/src/main.rs
- [x] T025 [P] [US1] Add --version CLI flag to plix-tools in crates/plix-tools/src/main.rs
- [x] T026 [US1] Verify no experimental features enabled by default (audit feature flags) in Cargo.toml
- [x] T027 [US1] Create user installation docs in docs/user/installation.md
- [x] T028 [P] [US1] Create getting started guide in docs/user/getting-started.md
- [x] T029 [P] [US1] Create settings documentation in docs/user/settings.md
- [x] T030 [P] [US1] Create FAQ in docs/user/faq.md
- [x] T031 [US1] Create smoke test script (launch server + client + complete quest) in scripts/smoke-test.sh
- [ ] T032 [US1] Run smoke test on Linux and verify PASS

**Checkpoint**: User Story 1 complete - New players can install and play

---

## Phase 4: User Story 2 - Update Server Without Data Loss (Priority: P1)

**Goal**: Server admins can upgrade from v0.x to v1.0 without losing player data

**Independent Test**: Run v0.x server with test data → upgrade to v1.0 → verify all data preserved

### Implementation for User Story 2

- [x] T033 [US2] Add config_version field to ServerConfig in crates/plix-server/src/config.rs
- [x] T034 [P] [US2] Add config_version field to GameConfig (client) in crates/plix-client/src/config.rs
- [x] T035 [US2] Implement V0ToV1ConfigMigration in crates/plix-common/src/migration/config.rs
- [ ] T036 [US2] Add save_version field to player save data in crates/plix-server/src/session.rs
- [x] T037 [P] [US2] Create save migration module in crates/plix-common/src/migration/save.rs
- [x] T038 [US2] Implement V0ToV1SaveMigration for quest/inventory data in crates/plix-common/src/migration/save.rs
- [x] T039 [US2] Integrate migration engine into server startup in crates/plix-server/src/main.rs
- [x] T040 [US2] Add --migrate and --migrate-dry-run CLI flags in crates/plix-server/src/main.rs
- [x] T041 [US2] Add --validate-content CLI flag in crates/plix-server/src/main.rs
- [x] T042 [US2] Log migration progress with tracing::info in crates/plix-common/src/migration/mod.rs
- [x] T043 [US2] Handle migration errors with clear messages and recovery instructions
- [x] T044 [P] [US2] Create server backup documentation in docs/server/backups.md
- [x] T045 [P] [US2] Create server upgrade guide in docs/server/upgrading.md
- [x] T046 [US2] Integration test: migrate v0 config → verify fields preserved
- [x] T047 [US2] Integration test: migrate v0 save → verify quest/inventory data

**Checkpoint**: User Story 2 complete - Servers can safely upgrade

---

## Phase 5: User Story 3 - Develop Stable Mods Against Documented API (Priority: P2)

**Goal**: Mod developers have stable, documented API with clear version compatibility

**Independent Test**: Follow modding docs → create hello-mod → load on v1.0 server → verify mod works

### Implementation for User Story 3

- [x] T048 [US3] Add stability status comments to public API in crates/plix-mod-core/src/lib.rs
- [x] T049 [US3] Mark experimental APIs with #[deprecated] attribute in crates/plix-mod-core/src/lib.rs
- [x] T050 [US3] Add mod version compatibility check to mod loader in crates/plix-mod-runtime-wasm/src/lib.rs
- [x] T051 [US3] Create compatibility matrix documentation in docs/release/compatibility.md
- [x] T052 [P] [US3] Create modding overview in docs/modding/overview.md (sdk-v1.md)
- [x] T053 [P] [US3] Create hello-mod tutorial in docs/modding/getting-started.md (sdk-v1.md)
- [x] T054 [P] [US3] Create SDK reference documentation in docs/modding/sdk-reference.md (sdk-v1.md)
- [x] T055 [P] [US3] Create stability policy documentation in docs/modding/stability.md (stability.rs)
- [x] T056 [P] [US3] Create mod compatibility guide in docs/modding/compatibility.md (publishing.md)
- [ ] T057 [US3] Unit test: v1.0 mod loads on v1.0 engine
- [ ] T058 [US3] Unit test: v1.0 mod loads on v1.1 engine (minor upgrade)
- [ ] T059 [US3] Unit test: v2.0 mod rejected on v1.x engine with clear error

**Checkpoint**: User Story 3 complete - Mod developers have stable API

---

## Phase 6: User Story 4 - Contribute to the Project (Priority: P3)

**Goal**: Contributors have clear guidelines for participating in the project

**Independent Test**: New contributor reads docs → forks repo → makes change → submits PR following guidelines

### Implementation for User Story 4

- [x] T060 [P] [US4] Create LICENSE file with MIT license text in LICENSE
- [x] T061 [P] [US4] Create README.md with project vision, status, and quickstart in README.md
- [x] T062 [P] [US4] Create CONTRIBUTING.md with PR rules and commit format in CONTRIBUTING.md
- [x] T063 [P] [US4] Create CODE_OF_CONDUCT.md (Contributor Covenant) in CODE_OF_CONDUCT.md
- [x] T064 [P] [US4] Create SECURITY.md with vulnerability disclosure process in SECURITY.md
- [x] T065 [P] [US4] Create ROADMAP.md with v1.x and v2.0 planning in docs/roadmap.md
- [ ] T066 [US4] Create issue templates in .github/ISSUE_TEMPLATE/
- [ ] T067 [US4] Create pull request template in .github/PULL_REQUEST_TEMPLATE.md
- [x] T067a [US4] Create SUPPORT.md with help resources in SUPPORT.md
- [x] T067b [US4] Create MAINTAINERS.md with maintainer info in MAINTAINERS.md

**Checkpoint**: User Story 4 complete - Contributors have clear guidelines

---

## Phase 7: Release Automation & Validation

**Purpose**: Ensure reproducible, verified release process

### Release Artifacts

- [x] T068 [P] Add SHA-256 checksum generation to release workflow in .github/workflows/release.yml
- [x] T069 [P] Add GPG signature verification step to release workflow in .github/workflows/release.yml
- [x] T070 Create release manifest generation script in .github/workflows/release.yml (inline)
- [x] T071 Add release_manifest.json generation to CI in .github/workflows/release.yml
- [ ] T072 [P] Create checksum verification docs in docs/release/verification.md
- [ ] T073 [P] Create GPG signing documentation in docs/release/signing.md

### Release Documentation

- [ ] T074 [P] Create CHANGELOG.md for v1.0.0 in CHANGELOG.md
- [ ] T075 [P] Create migration guide in docs/release/migration-guide.md
- [ ] T076 [P] Create known limitations document in docs/release/known-issues.md

### Server Documentation

- [ ] T077 [P] Create server installation guide in docs/server/installation.md
- [ ] T078 [P] Create server configuration reference in docs/server/configuration.md
- [ ] T079 [P] Create server mods documentation in docs/server/mods.md
- [ ] T080 [P] Create server security documentation in docs/server/security.md

---

## Phase 8: Final Validation & Release

**Purpose**: Quality gate and release execution

### Quality Gate

- [ ] T081 Run cargo fmt --all --check and fix any issues
- [ ] T082 Run cargo clippy --all-targets -- -D warnings and fix any warnings
- [ ] T083 Run cargo test --all and verify 100% pass
- [ ] T084 Run smoke test on Windows and verify PASS
- [ ] T085 Run smoke test on macOS and verify PASS
- [ ] T086 Verify version displays correctly in all locations (client, server, CLI)
- [ ] T087 Verify all documentation links resolve (no broken references)

### Release Execution

- [ ] T088 Create GPG-signed tag v1.0.0 with release notes
- [ ] T089 Push tag and verify CI builds all platform artifacts
- [ ] T090 Verify SHA256SUMS file generated and matches artifacts
- [ ] T091 Create GitHub Release with artifacts and checksums
- [ ] T092 Verify release download and checksum verification works
- [ ] T093 Update tasks.md status to DONE

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies - start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1 version types
- **Phase 3 (US1)**: Depends on Phase 1 completion
- **Phase 4 (US2)**: Depends on Phase 2 migration framework
- **Phase 5 (US3)**: Depends on Phase 1 version constants
- **Phase 6 (US4)**: No code dependencies - can run in parallel with US1-US3
- **Phase 7 (Release)**: Depends on Phase 1-6 completion
- **Phase 8 (Validation)**: Depends on Phase 7 completion

### User Story Dependencies

- **US1 (Install & Play)**: Foundational only - independent
- **US2 (Migration)**: Depends on migration framework (Phase 2)
- **US3 (Mod API)**: Foundational only - independent
- **US4 (Contribute)**: Completely independent - can run anytime

### Parallel Opportunities

**Phase 1 (after T001):**
```
T002 + T003 + T004 + T005 + T006 (all different files)
```

**Phase 2:**
```
T010 + T013 + T017 (all different files)
```

**Phase 3 (US1 docs):**
```
T027 + T028 + T029 + T030 (all different doc files)
```

**Phase 4 (US2):**
```
T034 + T037 + T044 + T045 (client config + save module + docs)
```

**Phase 5 (US3 docs):**
```
T052 + T053 + T054 + T055 + T056 (all different doc files)
```

**Phase 6 (US4 - all parallel):**
```
T060 + T061 + T062 + T063 + T064 + T065 (all different root files)
```

**Phase 7:**
```
T068 + T069 (different workflow sections)
T072 + T073 + T074 + T075 + T076 + T077 + T078 + T079 + T080 (all different docs)
```

---

## Implementation Strategy

### MVP First (US1 Only)

1. Complete Phase 1: Setup (T001-T008)
2. Complete Phase 3: User Story 1 (T021-T032)
3. **STOP and VALIDATE**: Smoke test passes
4. Can demo/ship with basic installation working

### Full Release

1. Complete all phases in order
2. US4 can run in parallel anytime
3. US1-US3 should complete before Phase 7
4. Phase 8 validates everything before release

### Suggested MVP Scope

**Minimum Viable Release** (can ship after):
- Phase 1: Setup ✓
- Phase 2: Foundational (migration framework) ✓
- Phase 3: US1 - Install & Play ✓
- Phase 4: US2 - Migration ✓

This delivers:
- Working installation
- Safe upgrades for existing servers
- Version displayed everywhere
- Basic user documentation

---

## Task Summary

| Phase | Description | Task Count | Parallelizable |
|-------|-------------|------------|----------------|
| 1 | Setup | 8 | 5 |
| 2 | Foundational | 12 | 6 |
| 3 | US1: Install & Play | 12 | 6 |
| 4 | US2: Migration | 15 | 4 |
| 5 | US3: Mod API | 12 | 6 |
| 6 | US4: Contribute | 8 | 6 |
| 7 | Release Automation | 13 | 10 |
| 8 | Validation & Release | 13 | 0 |
| **Total** | | **93** | **43** |

---

## Notes

- [P] tasks = different files, no dependencies
- [USn] label maps task to specific user story
- Verify smoke tests pass at each checkpoint
- Commit after each task or logical group
- All documentation should be in English
- Stop at any checkpoint to validate story independently
