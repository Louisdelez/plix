# Contract: Deployment Scripts

**Feature**: 028-dedicated-server-packaging
**Type**: Bash Script Specifications

## Script Locations

```
deploy/
├── scripts/
│   ├── build.sh          # Build Docker image(s)
│   ├── run.sh            # Run server container
│   ├── compose.sh        # Docker Compose wrapper
│   └── release-local.sh  # Create release archive
```

## build.sh

**Purpose**: Build Docker images for plix-server and/or plix-master

**Interface**:
```bash
./deploy/scripts/build.sh [OPTIONS]

Options:
  --server      Build plix-server image only (default if no option)
  --master      Build plix-master image only
  --all         Build both images
  --tag TAG     Image tag (default: latest)
  --no-cache    Build without cache
  -h, --help    Show help
```

**Exit Codes**:
- 0: Success
- 1: Build failed
- 2: Docker not found

## run.sh

**Purpose**: Run plix-server container with common options

**Interface**:
```bash
./deploy/scripts/run.sh [OPTIONS]

Options:
  --name NAME       Server name (default: Plix Server)
  --mode MODE       Game mode: ffa, tdm, ctf, br_lite (default: ffa)
  --port PORT       UDP port (default: 7777)
  --players N       Max players (default: 16)
  --master URL      Master server URL (enables registration)
  --detach          Run in background
  --volume PATH     Mount data volume (default: ./data)
  -h, --help        Show help
```

**Exit Codes**:
- 0: Success (or container started in detach mode)
- 1: Container failed to start
- 2: Docker not found
- 3: Port already in use

## compose.sh

**Purpose**: Docker Compose wrapper with profile shortcuts

**Interface**:
```bash
./deploy/scripts/compose.sh [COMMAND] [OPTIONS]

Commands:
  up              Start services (default)
  down            Stop and remove services
  logs            View service logs
  status          Show service status

Options:
  --server-only   Start only game server (default)
  --with-master   Start server + master
  --master-only   Start only master server
  --build         Rebuild images before starting
  --detach        Run in background
  -h, --help      Show help
```

**Exit Codes**:
- 0: Success
- 1: Docker Compose failed
- 2: Docker not found

## release-local.sh

**Purpose**: Create distributable release archive (non-Docker)

**Interface**:
```bash
./deploy/scripts/release-local.sh [OPTIONS]

Options:
  --version VER   Version string (default: git describe or 0.0.0)
  --output DIR    Output directory (default: ./release)
  --target TARGET Build target (default: x86_64-unknown-linux-gnu)
  -h, --help      Show help
```

**Output**:
```
release/
└── plix-server-<version>-linux-x86_64.tar.gz
└── plix-server-<version>-linux-x86_64.tar.gz.sha256
```

**Exit Codes**:
- 0: Success
- 1: Build failed
- 2: Cargo not found
- 3: Archive creation failed

## Common Requirements

All scripts:
- Use `#!/usr/bin/env bash`
- Set `set -euo pipefail`
- Check for required dependencies
- Support `-h` / `--help`
- Use colored output (if terminal supports)
- Print success/failure messages

## Error Messages

| Code | Message Template |
|------|------------------|
| 1 | "Error: {operation} failed. See output above." |
| 2 | "Error: {tool} not found. Please install {tool}." |
| 3 | "Error: Port {port} is already in use." |

## Example Usage

```bash
# Build and run quick test
./deploy/scripts/build.sh
./deploy/scripts/run.sh --name "Test Server" --mode ffa

# Full stack with master
./deploy/scripts/build.sh --all
./deploy/scripts/compose.sh up --with-master --detach

# Create release for distribution
./deploy/scripts/release-local.sh --version 1.0.0
```
