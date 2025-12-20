# CLI Command Contracts

**Binary**: `plix-mod-cli` (or `plix mod` subcommand)
**Version**: 0.1.0

## Command: `plix mod new`

Creates a new mod project from a template.

### Synopsis
```
plix mod new <name> [OPTIONS]
```

### Arguments
| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<name>` | String | Yes | Mod project name (becomes mod ID) |

### Options
| Option | Short | Type | Default | Description |
|--------|-------|------|---------|-------------|
| `--template` | `-t` | String | `chat-filter` | Template to use |
| `--output` | `-o` | Path | `./<name>` | Output directory |

### Templates
| Name | Description |
|------|-------------|
| `chat-filter` | Chat event handler with cancel example |
| `world-query` | World raycast and AABB query example |
| `timers-net` | Timer and network message example |

### Exit Codes
| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Invalid arguments |
| 2 | Template not found |
| 3 | Output directory exists |
| 4 | IO error |

### Example
```bash
plix mod new my-mod --template chat-filter
# Creates: ./my-mod/
#   ├── Cargo.toml
#   ├── mod.toml
#   ├── src/lib.rs
#   └── build.sh
```

---

## Command: `plix mod build`

Compiles a mod project to WASM.

### Synopsis
```
plix mod build [OPTIONS]
```

### Options
| Option | Short | Type | Default | Description |
|--------|-------|------|---------|-------------|
| `--release` | `-r` | Flag | true | Build in release mode |
| `--target` | | String | `wasm32-unknown-unknown` | Build target |
| `--manifest-path` | | Path | `./Cargo.toml` | Path to Cargo.toml |

### Exit Codes
| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Compilation failed |
| 2 | Missing Rust toolchain |
| 3 | Missing wasm target |

### Output
- `target/wasm32-unknown-unknown/release/<name>.wasm`

### Example
```bash
cd my-mod
plix mod build
# Output: target/wasm32-unknown-unknown/release/my_mod.wasm
```

---

## Command: `plix mod pack`

Creates a `.plixmod` bundle from a built mod.

### Synopsis
```
plix mod pack [OPTIONS]
```

### Options
| Option | Short | Type | Default | Description |
|--------|-------|------|---------|-------------|
| `--output` | `-o` | Path | `./<id>-<version>.plixmod` | Output file |
| `--manifest` | `-m` | Path | `./mod.toml` | Mod manifest |
| `--wasm` | `-w` | Path | auto-detect | WASM binary path |
| `--assets` | `-a` | Path | `./assets` | Assets directory |
| `--unsigned` | | Flag | false | Skip signing (dev only) |

### Validation
- Manifest is valid TOML with required fields
- WASM binary exists and has required exports
- Total size ≤ 10 MB
- Deterministic output (same inputs = same hash)

### Exit Codes
| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Manifest error |
| 2 | WASM not found |
| 3 | Size exceeded (> 10 MB) |
| 4 | Missing exports |
| 5 | IO error |

### Output Format
```
<id>-<version>.plixmod (ZIP archive)
├── mod.toml
├── mod.wasm
└── assets/ (optional)
```

### Example
```bash
plix mod pack
# Output: my-mod-1.0.0.plixmod
# Prints: Packed my-mod v1.0.0 (1.2 MB, sha256: abc123...)
```

---

## Command: `plix mod validate`

Validates a mod bundle without installing.

### Synopsis
```
plix mod validate <bundle> [OPTIONS]
```

### Arguments
| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | Path | Yes | Path to `.plixmod` file |

### Options
| Option | Short | Type | Default | Description |
|--------|-------|------|---------|-------------|
| `--strict` | `-s` | Flag | false | Fail on warnings |
| `--json` | | Flag | false | JSON output |

### Checks
| Check | Code | Description |
|-------|------|-------------|
| Manifest valid | E001 | mod.toml parses correctly |
| Required fields | E002 | id, name, version, api_version present |
| WASM present | E003 | mod.wasm exists in bundle |
| Exports present | E004 | mod_init, mod_on_event, mod_shutdown |
| Size limit | E005 | Bundle ≤ 10 MB |
| API version | E006 | api_version ≤ current |
| Capabilities valid | E007 | All capabilities are known |

### Exit Codes
| Code | Meaning |
|------|---------|
| 0 | Valid (no errors) |
| 1 | Validation errors |
| 2 | File not found |
| 3 | Not a valid bundle |

### Output (default)
```
Validating my-mod-1.0.0.plixmod...
✓ Manifest valid
✓ Required fields present
✓ WASM binary present
✓ Required exports found
✓ Size OK (1.2 MB / 10 MB)
✓ API version compatible (1)
✓ Capabilities valid

Result: PASS
```

### Output (--json)
```json
{
  "valid": true,
  "errors": [],
  "warnings": [],
  "info": {
    "id": "my-mod",
    "version": "1.0.0",
    "size": 1258291,
    "sha256": "abc123..."
  }
}
```

---

## Command: `plix mod install`

Installs a mod bundle to the local cache.

### Synopsis
```
plix mod install <bundle> [OPTIONS]
```

### Arguments
| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | Path | Yes | Path to `.plixmod` file |

### Options
| Option | Short | Type | Default | Description |
|--------|-------|------|---------|-------------|
| `--local` | `-l` | Flag | true | Install to local cache |
| `--cache` | `-c` | Path | platform default | Cache directory |
| `--force` | `-f` | Flag | false | Overwrite existing |

### Cache Locations
| Platform | Default Path |
|----------|--------------|
| Linux | `~/.local/share/plix/mods/` |
| macOS | `~/Library/Application Support/plix/mods/` |
| Windows | `%APPDATA%\plix\mods\` |

### Exit Codes
| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Validation failed |
| 2 | Already installed (use --force) |
| 3 | IO error |

### Example
```bash
plix mod install my-mod-1.0.0.plixmod --local
# Installed my-mod v1.0.0 to ~/.local/share/plix/mods/my-mod/1.0.0/
```

---

## Global Options

Available on all commands:

| Option | Short | Description |
|--------|-------|-------------|
| `--help` | `-h` | Show help |
| `--version` | `-V` | Show version |
| `--verbose` | `-v` | Increase verbosity |
| `--quiet` | `-q` | Suppress output |

## Environment Variables

| Variable | Description |
|----------|-------------|
| `PLIX_MOD_CACHE` | Override cache directory |
| `PLIX_MOD_TEMPLATES` | Override templates directory |
| `PLIX_MOD_DEV` | Enable dev mode (allows --unsigned) |
