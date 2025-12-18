# Feature Specification: Patch Updater & Launcher

**Feature Branch**: `029-patch-launcher`
**Created**: 2025-12-18
**Status**: Draft
**Input**: User description: "Lightweight launcher to verify local plix client version, compare to remote version, download updates if needed, and launch the game with the correct version to avoid client/server mismatches and simplify distribution."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Automatic Update Check and Launch (Priority: P1)

As a player, I want to launch the game without worrying about updates, so I can play immediately with the correct version.

**Why this priority**: This is the core value proposition - players should never encounter version mismatch errors or manually manage updates. This enables seamless gaming experience.

**Independent Test**: Launch the launcher, observe it checks version, downloads updates if needed, then starts the game. The player can verify they join servers without version errors.

**Acceptance Scenarios**:

1. **Given** the local version matches the remote version, **When** the player launches the launcher, **Then** the game starts directly within 5 seconds
2. **Given** the local version is older than the remote version, **When** the player launches the launcher, **Then** the launcher downloads updates, verifies integrity, and launches the updated game
3. **Given** no local version exists (fresh install), **When** the player launches the launcher, **Then** the launcher downloads the full client and launches it
4. **Given** the update server is unreachable, **When** the player launches the launcher with a valid local version, **Then** the game launches with the existing version (offline mode)
5. **Given** the update server is unreachable, **When** the player launches the launcher with no local version, **Then** the launcher displays a clear error message explaining the issue

---

### User Story 2 - Version Verification and Integrity (Priority: P1)

As a player, I want the launcher to verify my game files are complete and correct, so I never experience crashes or corruption issues.

**Why this priority**: Security and reliability are critical - launching corrupted or incomplete files could crash the game or cause undefined behavior.

**Independent Test**: Corrupt a game file locally, launch the launcher, observe it detects the mismatch via checksum and re-downloads only the corrupted file.

**Acceptance Scenarios**:

1. **Given** all local files match remote checksums, **When** the launcher verifies integrity, **Then** verification passes and the game launches
2. **Given** one or more local files have incorrect checksums, **When** the launcher verifies integrity, **Then** only the corrupted files are re-downloaded
3. **Given** a download fails mid-transfer, **When** the launcher attempts to launch, **Then** it refuses to launch and displays a retry option
4. **Given** a partially downloaded update exists, **When** the launcher restarts, **Then** it resumes or restarts the download cleanly

---

### User Story 3 - Progress Feedback (Priority: P2)

As a player, I want to see clear progress during updates, so I know what's happening and how long it will take.

**Why this priority**: User experience - players need feedback to understand the launcher state and estimate wait times.

**Independent Test**: Trigger an update, observe status messages and progress indicators showing current operation.

**Acceptance Scenarios**:

1. **Given** the launcher is checking version, **When** user observes the interface, **Then** they see "Checking for updates..." message
2. **Given** an update is downloading, **When** user observes the interface, **Then** they see download progress (percentage, size downloaded/total)
3. **Given** files are being verified, **When** user observes the interface, **Then** they see "Verifying files..." message
4. **Given** the game is launching, **When** user observes the interface, **Then** they see "Launching..." message

---

### User Story 4 - Developer Release Publishing (Priority: P2)

As a developer, I want to publish a new release easily, so players automatically receive updates without manual intervention.

**Why this priority**: Enables the update ecosystem - developers need a simple way to publish versions that players automatically receive.

**Independent Test**: Create a new manifest file with updated version info, upload files to the update server, observe that launchers detect and download the new version.

**Acceptance Scenarios**:

1. **Given** a new manifest is published with updated version, **When** players launch their launcher, **Then** they automatically receive the update
2. **Given** a manifest contains file checksums, **When** the launcher downloads files, **Then** it verifies each file against the manifest
3. **Given** a manifest lists multiple files, **When** the launcher updates, **Then** it downloads all required files

---

### User Story 5 - Server Admin Version Control (Priority: P3)

As a server admin, I want players to arrive with compatible client versions, so the server doesn't have to reject incompatible players.

**Why this priority**: Complements automatic updates - the launcher prevents version mismatches before they reach the server.

**Independent Test**: Configure server with minimum version requirement, have a player with older launcher attempt to connect (if they bypassed launcher), verify server rejects them.

**Acceptance Scenarios**:

1. **Given** the client version matches server requirements, **When** the player connects, **Then** connection succeeds
2. **Given** the client version is incompatible (player bypassed launcher), **When** the player connects, **Then** the server rejects with a clear version error message

---

### User Story 6 - CLI Operations (Priority: P3)

As a power user, I want to control the launcher via command line, so I can automate or script launcher operations.

**Why this priority**: Enables automation and advanced use cases for technical users.

**Independent Test**: Run launcher with --check flag, observe it outputs version status without launching. Run with --update flag, observe it updates without launching.

**Acceptance Scenarios**:

1. **Given** the user runs `plix-launcher --check`, **When** command completes, **Then** it outputs current local version, remote version, and whether update is needed
2. **Given** the user runs `plix-launcher --update`, **When** command completes, **Then** it downloads and installs updates without launching the game
3. **Given** the user runs `plix-launcher --launch`, **When** command completes, **Then** it launches the game without checking for updates
4. **Given** the user runs `plix-launcher --dry-run`, **When** command completes, **Then** it shows what would happen without making changes

---

### Edge Cases

- What happens when the user has no internet connection but a valid local version? → Launch with existing version (offline mode)
- What happens when the user has no internet and no local version? → Display clear error, cannot proceed
- What happens when disk space is insufficient for update? → Detect before download, display clear error
- What happens when download is interrupted (network drop, user closes launcher)? → Resume or restart cleanly on next launch
- What happens when manifest file is corrupted or invalid? → Reject manifest, display error, do not update
- What happens when a user tries to downgrade? → Not supported in v1, launcher ignores older remote versions
- What happens when multiple launcher instances run simultaneously? → Only one instance should run, detect and warn

## Requirements *(mandatory)*

### Functional Requirements

**Version Management**

- **FR-001**: Launcher MUST be a separate application from the plix game client
- **FR-002**: Launcher MUST store local version information in a version file or metadata
- **FR-003**: Launcher MUST fetch remote version manifest from a configurable HTTP URL
- **FR-004**: Launcher MUST compare local version against remote version using semantic versioning
- **FR-005**: Launcher MUST NOT support downgrades (ignore remote versions older than local)

**Update Process**

- **FR-006**: Launcher MUST download only files that differ from local versions
- **FR-007**: Launcher MUST verify downloaded files using SHA256 checksums before applying
- **FR-008**: Launcher MUST use atomic file replacement (download to temp, then move)
- **FR-009**: Launcher MUST handle network errors gracefully with configurable retry (3 retries, 5 second delay)
- **FR-010**: Launcher MUST have configurable download timeout (default: 30 seconds per file)
- **FR-011**: Launcher MUST refuse to launch if any game files fail integrity verification

**Manifest Format**

- **FR-012**: Manifest MUST include: current version string, list of files with URLs, SHA256 checksums, file sizes
- **FR-013**: Manifest MUST be in TOML or JSON format (developer choice)
- **FR-014**: Manifest MAY include protocol compatibility version for future use

**Local Directory Structure**

- **FR-015**: Launcher MUST manage a dedicated directory structure:
  - `plix/versions/` - stored version downloads
  - `plix/current/` - active version (symlink or copy)
  - `plix/launcher/` - launcher data and config
  - `plix/logs/` - log files
- **FR-016**: Launcher MUST clearly identify the active version
- **FR-017**: Launcher MUST allow manual rollback by removing `current/` directory

**Game Launch**

- **FR-018**: Launcher MUST start the plix client binary after successful update/verification
- **FR-019**: Launcher MUST pass CLI arguments to the game client if provided
- **FR-020**: Launcher MUST support configurable behavior: exit after launch or stay open

**User Interface**

- **FR-021**: Launcher MUST display status messages: checking, downloading, verifying, launching
- **FR-022**: Launcher MUST display download progress (percentage, bytes)
- **FR-023**: Launcher MUST display clear error messages for all failure scenarios
- **FR-024**: Launcher MUST NOT require a heavy UI framework (no CEF, no embedded browser)

**CLI Support**

- **FR-025**: Launcher MUST support `--check` flag to check version without updating or launching
- **FR-026**: Launcher MUST support `--update` flag to update without launching
- **FR-027**: Launcher MUST support `--launch` flag to launch without checking updates
- **FR-028**: Launcher MUST support `--dry-run` flag to simulate operations without changes

**Security & Reliability**

- **FR-029**: Launcher MUST verify checksums for all downloaded files (SHA256)
- **FR-030**: Launcher MUST NOT require elevated privileges (no admin/root)
- **FR-031**: Launcher MUST prevent launching partially installed versions
- **FR-032**: Launcher MUST ensure only one instance runs at a time

**Offline Support**

- **FR-033**: Launcher MUST work offline if a valid local version exists
- **FR-034**: Launcher MUST display appropriate message when offline without local version

**Logging & Debug**

- **FR-035**: Launcher MUST log operations to stdout and optionally to file
- **FR-036**: Launcher MUST include timestamps and log levels in log output
- **FR-037**: Launcher MUST support verbose mode for debugging

**Platform Support**

- **FR-038**: Launcher MUST support Linux (x86_64, aarch64)
- **FR-039**: Launcher MUST support Windows (x86_64)

### Key Entities

- **Version**: Semantic version string (e.g., "1.2.3"), represents a specific release of the client
- **Manifest**: Remote document describing the current release - version, files, checksums, URLs
- **Local State**: Tracks installed version, file checksums, last update time
- **Update File**: A downloadable artifact (binary, asset, config) with associated checksum and URL

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Players can launch the game in under 5 seconds when already up-to-date
- **SC-002**: Players can download and apply a 100MB update in under 2 minutes on a 10Mbps connection
- **SC-003**: 100% of launched game sessions have verified file integrity (no corrupted launches)
- **SC-004**: Version mismatch server rejections drop to near-zero for launcher users
- **SC-005**: Launcher works offline with existing installation (offline play enabled)
- **SC-006**: Launcher binary size is under 10MB (lightweight requirement)
- **SC-007**: Launcher supports both Linux and Windows platforms
- **SC-008**: All downloads are verified with SHA256 checksums before use
- **SC-009**: Failed or interrupted downloads do not corrupt the local installation
- **SC-010**: Clear error messages are displayed for all failure scenarios (no silent failures)

## Assumptions

- Update server is a simple HTTP server (static files, GitHub Releases, or CDN)
- No authentication required for downloading updates
- Manifest format will be TOML (consistent with other plix config files)
- Default manifest URL will be configurable via launcher config file
- The launcher will be distributed separately from the game client
- Semantic versioning is used for all version comparisons
- No delta/binary patching - full file replacement for changed files
- No mod management or custom content in v1
- No cryptographic signatures beyond checksums in v1
- Rollback is manual (delete current/) - no automatic rollback in v1

## Out of Scope

- Delta/binary differential patching (complex, not needed for v1)
- User authentication
- Mod management
- Automatic rollback on failure
- Advanced cryptographic signatures (GPG, code signing)
- macOS support (can be added later)
- Self-update of the launcher itself
- P2P/torrent distribution
- In-launcher news/announcements
