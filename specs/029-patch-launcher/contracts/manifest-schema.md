# Manifest Schema Contract

**Feature**: 029-patch-launcher
**Version**: 1.0
**Date**: 2025-12-18

## Overview

The manifest is a TOML file hosted on an HTTP server that describes a game release. The launcher fetches this manifest to determine if updates are available and what files to download.

## Schema Definition

### Root Object

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `manifest_version` | Integer | Yes | Schema version (must be `1`) |
| `version` | String | Yes | Game version in semver format (e.g., `"1.2.3"`) |
| `protocol_version` | Integer | No | Network protocol version for compatibility |
| `release_date` | Integer | Yes | Unix timestamp (seconds since epoch) |
| `files` | Array | Yes | List of files in this release |
| `release_notes_url` | String | No | URL to release notes |

### File Object

Each entry in the `files` array:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `path` | String | Yes | Relative path within installation directory |
| `url` | String | Yes | Full download URL (HTTP or HTTPS) |
| `size` | Integer | Yes | File size in bytes |
| `sha256` | String | Yes | SHA256 checksum (64 hex characters, lowercase) |
| `executable` | Boolean | No | If true, set executable permission (Unix). Default: `false` |

## Validation Rules

### Manifest Level

1. `manifest_version` MUST equal `1`
2. `version` MUST be valid semantic version (per semver.org)
3. `release_date` MUST be positive integer
4. `files` MUST contain at least one entry

### File Level

1. `path` MUST be relative (no leading `/` or `..` components)
2. `path` MUST use forward slashes (`/`) as separators
3. `url` MUST be valid HTTP or HTTPS URL
4. `size` MUST be positive integer
5. `sha256` MUST be exactly 64 lowercase hexadecimal characters
6. `path` MUST be unique within the manifest

## Example: Complete Manifest

```toml
# Plix Release Manifest
# Generated: 2025-12-18

manifest_version = 1
version = "1.3.0"
protocol_version = 1
release_date = 1734480000
release_notes_url = "https://github.com/plix/plix/releases/tag/v1.3.0"

# Game binary
[[files]]
path = "plix-client"
url = "https://releases.plix.example/v1.3.0/linux-x86_64/plix-client"
size = 47448064
sha256 = "a1b2c3d4e5f67890abcdef1234567890a1b2c3d4e5f67890abcdef1234567890"
executable = true

# Arena definitions
[[files]]
path = "assets/arenas/test_arena.toml"
url = "https://releases.plix.example/v1.3.0/assets/arenas/test_arena.toml"
size = 2048
sha256 = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"

[[files]]
path = "assets/arenas/ctf_arena.toml"
url = "https://releases.plix.example/v1.3.0/assets/arenas/ctf_arena.toml"
size = 3072
sha256 = "fedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321"
```

## Example: Minimal Manifest

```toml
manifest_version = 1
version = "1.0.0"
release_date = 1734480000

[[files]]
path = "plix-client"
url = "https://releases.plix.example/v1.0.0/plix-client"
size = 45000000
sha256 = "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
executable = true
```

## Error Responses

When manifest validation fails, the launcher MUST display clear error messages:

| Error | Message |
|-------|---------|
| Invalid TOML syntax | "Manifest file is corrupted or invalid" |
| Missing required field | "Manifest missing required field: {field}" |
| Invalid version format | "Invalid version format: {version}" |
| Invalid checksum format | "Invalid checksum for file: {path}" |
| Empty files array | "Manifest contains no files" |
| Unsupported manifest version | "Unsupported manifest version: {version}" |

## Platform Variations

The manifest URL may include platform identifiers:

```
https://releases.plix.example/v1.3.0/linux-x86_64/manifest.toml
https://releases.plix.example/v1.3.0/linux-aarch64/manifest.toml
https://releases.plix.example/v1.3.0/windows-x86_64/manifest.toml
```

Platform detection is done by the launcher before fetching.

## Versioning

### Manifest Version 1 (Current)

- Initial format as described above
- Full file replacement (no delta patches)

### Future Versions

Future manifest versions may add:
- Delta patch support (`patch_url`, `patch_sha256`)
- File compression info
- Dependencies between files
- Asset packs / DLC

Launchers MUST reject manifests with `manifest_version > 1` until they support higher versions.

## HTTP Requirements

### Request

```
GET /manifest.toml HTTP/1.1
Host: releases.plix.example
User-Agent: plix-launcher/1.0
Accept: text/plain
```

### Response (Success)

```
HTTP/1.1 200 OK
Content-Type: text/plain; charset=utf-8
Content-Length: 1234
Cache-Control: max-age=60

manifest_version = 1
...
```

### Response (Error)

| Status | Meaning |
|--------|---------|
| 404 | Manifest not found |
| 500+ | Server error |
| Timeout | Network issue |

Launcher should treat any non-200 response as "offline mode eligible".
