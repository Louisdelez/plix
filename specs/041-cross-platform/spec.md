# Feature Specification: Cross-Platform Client Packaging & Headless Server

**Feature Branch**: `041-cross-platform`
**Created**: 2025-12-19
**Status**: Draft
**Input**: User description: "Cross-Platform (Windows/Linux/macOS packaging client) + Headless Server stable - deliver reproducible multi-platform distribution for client and production-ready headless server"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Player Downloads and Launches Client (Priority: P1)

A player on Windows, Linux, or macOS wants to download the game client and start playing. They download the appropriate bundle for their operating system, extract it, and launch the game with minimal friction (2 clicks maximum).

**Why this priority**: This is the core value proposition - players cannot engage with the game without a working, easy-to-install client. This unlocks the entire player base across all major desktop platforms.

**Independent Test**: Can be fully tested by downloading the appropriate OS bundle, extracting it, and verifying the client launches successfully and displays version information.

**Acceptance Scenarios**:

1. **Given** a Windows user downloads `plix-client-win64-<version>.zip`, **When** they extract and run `plix-client.exe`, **Then** the game launches and displays the main menu with version info visible
2. **Given** a Linux user downloads `plix-client-linux-x86_64-<version>.tar.gz`, **When** they extract and run `./plix-client`, **Then** the game launches and displays the main menu with version info visible
3. **Given** a macOS user downloads `plix-client-macos-<version>.zip`, **When** they extract `Plix.app` and double-click it, **Then** the game launches and displays the main menu with version info visible
4. **Given** a player launches the client, **When** they check version information, **Then** they see semantic version, git SHA, and build date

---

### User Story 2 - Server Administrator Deploys Headless Server (Priority: P1)

A server administrator wants to deploy a production-ready game server on a VM or Docker container. They download the headless server bundle, configure it via TOML files, and run it without any graphical dependencies.

**Why this priority**: Without stable server deployment, multiplayer gameplay is impossible. Server stability and ease of deployment directly impact player experience and community server hosting.

**Independent Test**: Can be fully tested by deploying the headless server on a Linux VM without X11/Wayland, starting it, connecting a client, and verifying graceful shutdown on SIGTERM.

**Acceptance Scenarios**:

1. **Given** an admin downloads `plix-server-headless-linux-x86_64-<version>.tar.gz`, **When** they extract and run `./plix-server-headless`, **Then** the server starts and listens on the configured port
2. **Given** a running headless server, **When** it receives SIGINT or SIGTERM, **Then** it performs graceful shutdown (flushes logs, calls mod shutdown hooks) and exits with code 0
3. **Given** an admin provides an invalid `server.toml`, **When** they attempt to start the server, **Then** it exits with code 1 and displays a clear error message
4. **Given** an admin tries to bind to an already-occupied port, **When** the server fails to bind, **Then** it exits with code 2 and displays a specific error message

---

### User Story 3 - CI/CD Pipeline Builds Release Artifacts (Priority: P2)

A maintainer wants the CI/CD pipeline to automatically build, package, and upload release artifacts for all platforms when code is pushed or a release tag is created.

**Why this priority**: Automated builds ensure reproducibility and reduce manual release effort. This is essential for sustainable project maintenance but can be done manually initially.

**Independent Test**: Can be fully tested by triggering the CI workflow and verifying that artifacts are produced for all 6 targets (3 client + 3 server) with correct naming and content.

**Acceptance Scenarios**:

1. **Given** a push to the release branch, **When** CI completes, **Then** artifacts are uploaded for Windows/Linux/macOS client and server
2. **Given** CI builds a client package, **When** the build completes, **Then** the artifact includes `build_info.json` with version, git SHA, build date, and target platform
3. **Given** CI builds artifacts, **When** they are named, **Then** they follow the pattern `plix-{client|server-headless}-{platform}-<version>.{zip|tar.gz}`

---

### User Story 4 - Admin Deploys Server via Docker (Priority: P3)

A server administrator wants to deploy the game server using Docker for easier orchestration and isolation. They use the provided Dockerfile to build an image and run it with volume mounts for persistence.

**Why this priority**: Docker deployment is a convenience enhancement for cloud-native deployments. The core server functionality works without it.

**Independent Test**: Can be fully tested by building the Docker image, running it with exposed ports, connecting a client, and verifying volume persistence.

**Acceptance Scenarios**:

1. **Given** a Dockerfile in the repository, **When** an admin runs `docker build`, **Then** a working image is created
2. **Given** a built Docker image, **When** an admin runs `docker run -p 7777:7777 plix-server`, **Then** the server starts and accepts connections
3. **Given** a running Docker container, **When** the container is stopped, **Then** data in mounted volumes persists

---

### Edge Cases

- What happens when a client bundle is missing required runtime files (CEF)?
  - Client displays a clear error message indicating missing dependencies and exits gracefully
- How does the server handle shutdown during active mod operations?
  - Server waits up to 5 seconds for mods to complete shutdown hooks before force-terminating
- What happens when the client version mismatches expected asset version?
  - Client warns about potential asset mismatch but allows user to proceed
- How does the system handle network failure during server startup?
  - Server retries binding once, then exits with code 2 if still failing

## Requirements *(mandatory)*

### Functional Requirements

**Client Packaging**

- **FR-001**: System MUST produce client bundles for Windows (x64), Linux (x86_64), and macOS (x64/ARM universal)
- **FR-002**: Each client bundle MUST include the executable, assets folder, required runtime libraries (CEF), and default configuration
- **FR-003**: Client bundles MUST include a `build_info.json` file containing semantic version, git SHA, build timestamp (UTC), and target platform
- **FR-004**: Client MUST display version information (semver + git SHA) in the UI or console log on startup
- **FR-005**: Windows client bundle MUST be packaged as a `.zip` file containing the executable and all required DLLs
- **FR-006**: Linux client bundle MUST be packaged as a `.tar.gz` file containing the executable and required shared libraries
- **FR-007**: macOS client bundle MUST be packaged as a `.zip` file containing a valid `.app` bundle structure with frameworks in `Frameworks/`
- **FR-008**: Packaging scripts MUST validate that all required files exist before creating the archive

**Headless Server**

- **FR-009**: System MUST provide a headless server binary that runs without any graphical or audio dependencies
- **FR-010**: Headless server MUST handle SIGINT and SIGTERM (Unix) and Ctrl-C (Windows) for graceful shutdown
- **FR-011**: Server MUST invoke mod shutdown hooks and flush logs before exiting during graceful shutdown
- **FR-012**: Server MUST use defined exit codes: 0 (normal), 1 (config invalid), 2 (port bind failed), 3 (fatal runtime error)
- **FR-013**: Server MUST validate configuration files (`server.toml`, `server_mods.toml`) at startup and provide clear error messages for invalid config
- **FR-014**: Server bundle MUST include example configuration files, documentation, and platform-appropriate run scripts (`run_server.sh`, `run_server.ps1`)
- **FR-015**: Server MUST have a shutdown timeout of 5 seconds before force-terminating

**CI/CD**

- **FR-016**: CI workflow MUST build client and server for all three platforms using a matrix strategy
- **FR-017**: CI workflow MUST upload artifacts with consistent naming: `plix-{type}-{platform}-<version>.{ext}`
- **FR-018**: CI workflow MUST use cargo caching to improve build times
- **FR-019**: CI MUST run smoke tests verifying server start/stop, config validation, and port bind failure handling

**Docker (Optional)**

- **FR-020**: Repository SHOULD include a Dockerfile for building a headless server image
- **FR-021**: Docker image SHOULD expose the game port and support volume mounts for mods cache and world data

### Key Entities

- **Client Bundle**: Platform-specific distributable package containing executable, assets, runtimes, and metadata
- **Server Bundle**: Platform-specific distributable package containing headless executable, example configs, docs, and run scripts
- **Build Info**: Metadata structure containing version (semver), commit SHA, build timestamp, and target triple
- **Exit Codes**: Standardized server exit status indicating shutdown reason (0=normal, 1=config error, 2=bind error, 3=fatal)

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Players can download, extract, and launch the client on any supported platform in under 2 minutes
- **SC-002**: Server administrators can deploy a headless server on a fresh Linux VM in under 5 minutes
- **SC-003**: CI pipeline produces all 6 release artifacts (3 client + 3 server) in under 30 minutes
- **SC-004**: Headless server completes graceful shutdown within 5 seconds of receiving termination signal
- **SC-005**: 100% of smoke tests pass in CI before artifacts are uploaded
- **SC-006**: Client bundles for all platforms include all required runtime dependencies with zero missing files
- **SC-007**: Server exits with correct exit code for each error condition (config invalid=1, bind failed=2)
- **SC-008**: Build info is correctly embedded and displayable in both client and server

## Assumptions

- CEF runtime files are available and can be bundled (location documented per-platform)
- The existing build system (cargo) supports cross-compilation or CI uses native runners per platform
- GitHub Actions is the CI/CD platform (based on existing `.github/workflows/` presence)
- Version information is derived from git tags (semver) and commit SHA
- Players have standard permissions to run executables on their systems (no elevated privileges required)
- Server deployments target standard Linux distributions (Ubuntu/Debian), Windows Server, or macOS
- Docker deployments use Linux containers
