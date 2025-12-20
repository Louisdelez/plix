# Plix

**Multiplayer Voxel Game Platform**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.83%2B-orange.svg)](https://www.rust-lang.org/)

Plix is an open-source multiplayer voxel game platform built in Rust. It features a server-authoritative architecture, full modding support, and cross-platform compatibility.

## Features

- **Multiplayer**: 8-16 players per server with low-latency networking
- **Voxel World**: Procedural terrain generation with chunked streaming
- **Combat System**: Melee and ranged weapons with server-authoritative hit detection
- **Game Modes**: TDM, FFA, CTF, Battle Royale, and Training modes
- **Quest System**: NPCs, dialogues, and campaign progression
- **Modding**: WASM-sandboxed mod runtime with stable API
- **Accessibility**: High contrast mode, colorblind filters, custom keybinds
- **Cross-Platform**: Windows, Linux, and macOS support

## Quick Start

### Prerequisites

- Rust 1.83+ (stable)
- Cargo

### Build

```bash
cargo build --release
```

### Run Server

```bash
./target/release/plix-server --port 7777
```

### Run Client

```bash
./target/release/plix-client --server 127.0.0.1:7777 --name Player1
```

## Downloads

Pre-built binaries are available on the [Releases](https://github.com/Louisdelez/plix/releases) page.

Verify downloads with SHA-256 checksums:
```bash
sha256sum -c SHA256SUMS --ignore-missing
```

## Documentation

| Guide | Description |
|-------|-------------|
| [Installation](docs/user/installation.md) | How to install and run Plix |
| [Getting Started](docs/user/getting-started.md) | First steps and tutorial |
| [Server Setup](docs/server/headless-deploy.md) | Running a dedicated server |
| [Modding SDK](docs/modding/sdk-v1.md) | Creating mods for Plix |
| [Roadmap](docs/roadmap.md) | Future development plans |

## Project Structure

```
crates/
  plix-client/          # Game client with CEF UI
  plix-server/          # Authoritative game server
  plix-common/          # Shared types, protocol, content
  plix-tools/           # CLI tools and utilities
  plix-mod-core/        # Mod API definitions
  plix-mod-runtime-wasm/# WASM sandbox for mods
  plix-mod-sdk/         # SDK for mod developers
  plix-mod-cli/         # Mod development CLI
```

## Architecture

- **Server-Authoritative**: All game state validated on server
- **Client Prediction**: Responsive local movement with reconciliation
- **60 Hz Tick Rate**: Smooth gameplay at 60 updates/second
- **WASM Sandbox**: Mods run in isolated WebAssembly environment
- **CEF UI**: Modern HTML/CSS/JS user interface

## Contributing

We welcome contributions! Please read:

- [Contributing Guide](CONTRIBUTING.md) - How to contribute
- [Code of Conduct](CODE_OF_CONDUCT.md) - Community guidelines
- [Security Policy](SECURITY.md) - Reporting vulnerabilities

### Development

```bash
# Run tests
cargo test --workspace

# Check formatting
cargo fmt --all -- --check

# Run linter
cargo clippy --all-targets
```

## Community

- **Issues**: [GitHub Issues](https://github.com/Louisdelez/plix/issues)
- **Discussions**: [GitHub Discussions](https://github.com/Louisdelez/plix/discussions)
- **Security**: security@plix.dev

## License

Plix is licensed under the [MIT License](LICENSE).

---

Built with Rust and wgpu
