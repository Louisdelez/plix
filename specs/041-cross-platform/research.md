# Research: Cross-Platform Packaging & Headless Server

**Feature**: 041-cross-platform | **Date**: 2025-12-19

## 1. Build Info Embedding (Rust Compile-Time)

### Decision
Use **shadow-rs** for comprehensive build info embedding.

### Rationale
- Provides 40+ compile-time constants (VERSION, GIT_COMMIT, BUILD_TIME, GIT_BRANCH, RUST_VERSION)
- No runtime dependencies; all embedded at compile time
- Automatic rebuild triggering via build.rs when git state changes
- Seamless integration with clap CLI (already used in plix-server, plix-launcher)
- Production-tested in large projects (GreptimeDB)

### Implementation
```rust
// Cargo.toml
[dependencies]
shadow-rs = "0.32"

[build-dependencies]
shadow-rs = "0.32"

// build.rs
fn main() {
    shadow_rs::new().build().expect("Failed to build shadow-rs");
}

// main.rs
mod shadow { include!(concat!(env!("OUT_DIR"), "/shadow.rs")); }

// Usage
println!("plix-server v{} ({}) built {}",
    shadow::PKG_VERSION, shadow::SHORT_COMMIT, shadow::BUILD_TIME);
```

### Alternatives Considered
| Alternative | Trade-off |
|-------------|-----------|
| Manual build.rs + git | Requires custom parsing, no auto-rebuild |
| git-version crate | Lighter but lacks build timestamp |
| compile-time crate | Only timestamps, insufficient for full metadata |

---

## 2. macOS .app Bundle Creation

### Decision
Use **cargo-bundle** for automated .app structure + **rcodesign** (pure Rust) for signing.

### Rationale
- cargo-bundle: Automatically creates correct .app bundle structure (Contents/MacOS, Resources, Frameworks)
- Reads config from Cargo.toml `[package.metadata.bundle]` section
- rcodesign: Pure Rust implementation, runs on Linux/Windows CI runners
- No proprietary Apple tools required (unlike xcrun-based signing)

### Implementation
```toml
# Cargo.toml
[package.metadata.bundle]
name = "Plix"
identifier = "com.plix.game-client"
icon = ["assets/icons/icon.icns"]
resources = ["assets/arenas", "assets/ui"]
```

### macOS Bundle Structure
```
Plix.app/Contents/
├── MacOS/
│   └── plix-client
├── Frameworks/
│   └── (CEF frameworks if needed)
├── Resources/
│   ├── assets/
│   └── icon.icns
└── Info.plist
```

### Alternatives Considered
| Alternative | Trade-off |
|-------------|-----------|
| Manual script bundling | 300+ lines, error-prone |
| fruitbasket | Runtime bundling, adds overhead |
| Tauri/Iced | Heavy UI framework dependency |

---

## 3. CEF Runtime Bundling

### Decision
Implement platform-specific automated download via build.rs from Spotify CDN.

### Rationale
- CEF requires strict filesystem organization per platform
- Pre-built binaries available from cef-builds.spotifycdn.com
- Download at build time, not checked into git (~500MB per platform)
- Matches existing `cef-ui` feature flag pattern in plix-client

### Implementation Pattern
```rust
// build.rs (when cef-ui feature enabled)
let cef_url = match target {
    "x86_64-apple-darwin" => "https://cef-builds.spotifycdn.com/cef_binary_127_macosx64.tar.gz",
    "x86_64-pc-windows-msvc" => "https://cef-builds.spotifycdn.com/cef_binary_127_windows64.tar.gz",
    "x86_64-unknown-linux-gnu" => "https://cef-builds.spotifycdn.com/cef_binary_127_linux64.tar.gz",
    _ => panic!("Unsupported target"),
};
download_and_extract_cef(&cef_url);
```

### Platform-Specific Requirements
| Platform | CEF Location | Notes |
|----------|--------------|-------|
| macOS | `Frameworks/Chromium Embedded Framework.framework/` | Requires helper apps |
| Linux | `./cef/` relative to binary | Set LD_LIBRARY_PATH |
| Windows | Same directory as .exe | DLLs in PATH |

### Alternatives Considered
| Alternative | Trade-off |
|-------------|-----------|
| Static git bundling | 500MB+ repo, version management complexity |
| Runtime download | Poor UX for users, requires network |
| WebView alternatives | Platform-inconsistent, loses CEF features |

---

## 4. Cross-Compilation vs Native CI

### Decision
**Hybrid approach**: Native runners for primary targets (Win/Mac/Linux x64), cross-compile for ARM.

### Rationale
- macOS signing/notarization requires macOS runner
- CEF framework linking needs native platform
- Native compilation is simpler and more reliable
- Cross-compilation for ARM saves CI costs (~$60/month vs $100+)

### CI Matrix Configuration
```yaml
matrix:
  include:
    - os: ubuntu-latest
      target: x86_64-unknown-linux-gnu
      compile: native
    - os: windows-latest
      target: x86_64-pc-windows-msvc
      compile: native
    - os: macos-latest
      target: x86_64-apple-darwin
      compile: native
    - os: macos-latest
      target: aarch64-apple-darwin
      compile: native  # Apple Silicon native support
```

### Alternatives Considered
| Alternative | Trade-off |
|-------------|-----------|
| Pure cross-compilation | Cannot sign macOS, CEF linking issues |
| All native runners | $100+/month, slower CI queue |
| Self-hosted runners | Maintenance burden, hardware cost |

---

## 5. Signal Handling (Headless Server)

### Decision
Use **tokio::signal::ctrl_c()** with broadcast channel coordination.

### Rationale
- Async-aware, doesn't block event loop
- Works on Windows (Ctrl+C) and Unix (SIGINT/SIGTERM)
- Already partially implemented in plix-server (lib.rs:506-516)
- Broadcast channels enable graceful multi-task shutdown

### Implementation Pattern
```rust
tokio::select! {
    _ = tokio::signal::ctrl_c() => {
        info!("Shutdown signal received");
        let _ = shutdown_tx.send(true);
        self.shutdown().await;
        return Ok(());
    }
    // ... other branches
}
```

### Alternatives Considered
| Alternative | Trade-off |
|-------------|-----------|
| signal-hook + tokio | More explicit but requires unsafe |
| Dedicated signal task | Decoupled but adds complexity |

---

## 6. Graceful Shutdown with Timeout

### Decision
Two-phase shutdown: (1) Signal all tasks, (2) Wait with 5s timeout, (3) Force exit.

### Rationale
- Ensures state persistence (world save) before exit
- Notifies connected clients of shutdown
- Prevents hanging on unresponsive mods
- Already partially implemented in plix-server

### Implementation Pattern
```rust
async fn graceful_shutdown_with_timeout(timeout_secs: u64) {
    let deadline = Instant::now() + Duration::from_secs(timeout_secs);

    // Phase 1: Notify tasks
    let _ = shutdown_tx.send(true);

    // Phase 2: Wait with timeout
    tokio::select! {
        _ = tokio::time::sleep_until(deadline) => {
            warn!("Shutdown timeout exceeded, forcing exit");
            std::process::exit(1);
        }
        _ = wait_for_tasks() => {
            info!("Clean shutdown complete");
        }
    }
}
```

---

## 7. Exit Code Conventions

### Decision
Standard Unix codes + application-specific codes 64-78.

### Rationale
- Industry standard for CLI applications
- Scriptable (CI, systemd, Docker can react to codes)
- Already used in plix-perf (exit 1 on validation failure)

### Exit Code Scheme
| Code | Meaning | Example |
|------|---------|---------|
| 0 | Success | Clean shutdown |
| 1 | General error | Runtime error, validation failure |
| 2 | Misuse | Invalid CLI arguments |
| 64 | Bind failed | Port already in use |
| 65 | Asset load failed | Missing arena file |
| 66 | Persistence error | World save failed |

---

## 8. Config Validation Patterns

### Decision
Structured validation with error context and remediation hints.

### Rationale
- Fail fast before complex initialization
- Clear error messages reduce support burden
- Already patterned in plix-mod-cli (check_rust_toolchain, check_wasm_target)

### Implementation Pattern
```rust
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Port validation failed: {0}")]
    InvalidPort(String),

    #[error("Arena not found at {0}\n\nHint: List arenas with --list-arenas")]
    ArenaNotFound(String),
}

impl ServerConfig {
    pub fn validate(&self) -> Result<(), Vec<ConfigError>> {
        let mut errors = Vec::new();
        if self.port == 0 { errors.push(ConfigError::InvalidPort("cannot be 0".into())); }
        // ... more validations
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}
```

---

## 9. GitHub Actions Release Workflow

### Decision
Swatinem/rust-cache v2 + 3-tier smoke tests + separate release job.

### Rationale
- rust-cache already in use, provides 40-60% build time savings
- Smoke tests catch linking/dependency issues in <2 minutes
- Separate release job waits for all builds, creates single release

### Smoke Test Tiers
```yaml
# Tier 1: Binary existence (30s)
- run: ls -lh target/${{ matrix.target }}/release/plix-*

# Tier 2: Execution (1-2min)
- run: timeout 5 ./plix-server --help || true

# Tier 3: Version/Integrity (1min)
- run: |
    ./plix-server --version
    sha256sum target/${{ matrix.target }}/release/plix-* > checksums.txt
```

### Artifact Naming Convention
```
plix-client-linux-x86_64-v0.1.0.tar.gz
plix-client-windows-x64-v0.1.0.zip
plix-client-macos-x64-v0.1.0.zip
plix-server-headless-linux-x86_64-v0.1.0.tar.gz
```

---

## Summary Table

| Topic | Decision | Implementation Effort |
|-------|----------|----------------------|
| Build Info | shadow-rs | 2-3 hours |
| macOS Bundles | cargo-bundle + rcodesign | 4-6 hours |
| CEF Bundling | Automated download via build.rs | 8-12 hours |
| CI Strategy | Hybrid native + selective cross | 6-8 hours |
| Signal Handling | tokio::signal::ctrl_c() | 2-3 hours |
| Graceful Shutdown | Two-phase with timeout | 3-4 hours |
| Exit Codes | Standard Unix + app-specific | 1-2 hours |
| Config Validation | Structured errors with hints | 2-3 hours |
| GitHub Actions | rust-cache + smoke tests | 4-6 hours |

**Total estimated implementation**: 32-47 hours
