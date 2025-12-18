# plix Development Guidelines

Auto-generated from all feature plans. Last updated: 2025-12-14

## Active Technologies
- Rust 1.75+ (stable channel only) + glam (math), bincode (serialization), wgpu (rendering), tokio (async) (003-combat-visible)
- N/A (in-memory state only, no persistence) (003-combat-visible)
- Rust 1.75+ (stable channel only) + glam (math), bincode (serialization), wgpu (rendering), tokio (async), winit (input) (004-block-interaction)
- Rust 1.75+ (stable channel only per constitution) + tokio (async), bincode (serialization), glam (math), wgpu (client rendering) (006-match-flow)
- In-memory only for MVP (ban list clears on restart per spec) (007-anti-cheat-hardening)
- Rust 1.75+ (stable channel only per constitution) + glam (math), bincode (serialization), tokio (async), wgpu (client rendering) (008-movement-polish)
- N/A (in-memory state only) (008-movement-polish)
- Rust 1.75+ (stable channel only per constitution) + tokio (async), bincode (serialization), glam (math), wgpu (client rendering), tracing (logging) (010-logging-metrics)
- N/A (in-memory metrics only) (010-logging-metrics)
- Rust 1.75+ (stable channel only per constitution) + wgpu 23.0 (rendering), glam (math), bincode (serialization), tokio (async) (011-chunked-world)
- In-memory chunked HashMap (client-side); arena still loads from TOML server-side (011-chunked-world)
- Rust 1.75+ (stable channel only per constitution) + plix-common (chunk types), plix-client (ChunkManager, meshing), tracing (metrics) (012-world-edit-optimization)
- Rust 1.75+ (stable channel only per constitution) + `noise-rs` (noise generation), `glam` (math), existing plix-common types (013-procedural-generation)
- N/A (in-memory chunk generation, no persistence in this feature) (013-procedural-generation)
- Rust 1.75+ (stable channel only per constitution) + serde + bincode (existing), tokio (existing async runtime), tracing (existing logging) (014-world-persistence)
- File system with per-world directories (`~/.local/share/plix/worlds/`) (014-world-persistence)
- Rust 1.75+ (stable channel only per constitution) + plix-common (types, world, chunk), plix-server (game loop), bincode (serialization), glam (math) (015-block-physics)
- N/A (in-memory event queue, block state in existing ChunkedWorld) (015-block-physics)
- N/A (in-memory state only, no persistence required for match state) (016-tdm-arena)
- Rust 1.75+ (stable channel only per constitution) + plix-arena (arena loading), plix-server (match logic), plix-common (types, protocol), glam (math), bincode (serialization), tokio (async) (017-ffa-arena)
- N/A (in-memory state only, arena definitions in TOML files) (017-ffa-arena)
- Rust 1.75+ (stable channel only per constitution) + plix-common (types, protocol), plix-server (match state, game logic), plix-arena (zone definitions), glam (math/Vec3), bincode (serialization), tokio (async) (018-ctf-mode)
- N/A (in-memory state only - flag states, scores reset on match end) (018-ctf-mode)
- Rust 1.75+ (stable channel only per constitution) + plix-common (types, protocol, math), plix-server (match state, game loop), plix-arena (arena loading), glam (Vec3), bincode (serialization), tokio (async), tracing (logging) (019-br-lite)
- N/A (in-memory state only - zone, alive roster, loot reset on match end) (019-br-lite)
- Rust 1.75+ (stable channel only per constitution) + plix-common (types, protocol), plix-server (match state, game loop), plix-arena (arena loading), glam (Vec3), bincode (serialization), tokio (async), tracing (logging) (020-training-mode)
- N/A (in-memory state only - bots, stats reset on match end/disconnect) (020-training-mode)
- Rust 1.75+ (stable channel only per constitution) + plix-common (types, protocol), plix-server (game loop, match state), plix-client (UI), glam (math), bincode (serialization), serde (derive), tokio (async) (021-inventory-hotbar)
- N/A (in-memory state only, no persistence in v1) (021-inventory-hotbar)
- Rust 1.75+ (stable channel only per constitution) + glam (math), bincode (serialization), tokio (async), existing plix-common/plix-server crates (022-weapons-items-v1)
- N/A (in-memory state only - projectiles, cooldowns, recoil state) (022-weapons-items-v1)
- Rust 1.75+ (stable channel only per constitution) + plix-common (types, inventory, protocol), plix-server (game loop, session), plix-client (console commands) (023-crafting-lite)
- N/A (in-memory state only - crafting state resets on match end) (023-crafting-lite)
- Rust 1.75+ (stable channel only per constitution) + plix-common (types, inventory, protocol), plix-server (game loop, session, match_state), plix-client (console commands), Hotbar (Feature 021), Crafting (Feature 023) (024-economy-lite)
- N/A (in-memory state only - balances reset on match end) (024-economy-lite)
- Rust 1.75+ (stable channel only per constitution) + serde (serialization), toml (profile format), tracing (logging), bincode (protocol) (025-account-identity)
- Client: `~/.config/plix/profile.toml` (XDG compliant); Server: in-memory only (no persistence) (025-account-identity)
- Rust 1.75+ (stable channel only per constitution) + plix-common (types, server_browser), plix-client (console, server_browser, profile), reqwest (HTTP), serde/toml (serialization), rand (tie-breaking) (027-matchmaking-v1)
- `~/.config/plix/profile.toml` (extends Feature 025 profile with `[matchmaking]` section) (027-matchmaking-v1)
- Rust 1.83+ (stable channel only per constitution) + reqwest (HTTP, blocking mode for simplicity), serde/toml (config), sha2 (checksums), clap (CLI), tracing (logging), dirs-next (platform paths) (029-patch-launcher)
- File system - `~/.local/share/plix/` for game data, `~/.config/plix/` for launcher config (029-patch-launcher)
- Rust 1.75+ (stable channel only per constitution) + CEF (binding TBD via spike), wgpu (existing), winit (existing), clap (CLI), toml/serde (config) (030-cef-ui-shell)
- N/A (in-memory state only) (030-cef-ui-shell)
- Rust 1.75+ (stable channel only per constitution) + HTML5/CSS3/ES6 JavaScript + plix-client (rendering, config, server_browser), plix-common (types, protocol), serde_json (bridge serialization), Feature 030 (CEF shell), Feature 026 (server browser), Feature 025 (account identity) (031-cef-menus)
- ~/.config/plix/favorites.toml (local file, TOML format, shared with native UI) (031-cef-menus)

- Rust 1.75+ (stable channel only per constitution) (002-voxel-game-platform)

## Project Structure

```text
src/
tests/
```

## Commands

cargo test [ONLY COMMANDS FOR ACTIVE TECHNOLOGIES][ONLY COMMANDS FOR ACTIVE TECHNOLOGIES] cargo clippy

## Code Style

Rust 1.75+ (stable channel only per constitution): Follow standard conventions

## Recent Changes
- 031-cef-menus: Added Rust 1.75+ (stable channel only per constitution) + HTML5/CSS3/ES6 JavaScript + plix-client (rendering, config, server_browser), plix-common (types, protocol), serde_json (bridge serialization), Feature 030 (CEF shell), Feature 026 (server browser), Feature 025 (account identity)
- 031-cef-menus: Added Rust 1.75+ (stable channel only per constitution) + HTML5/CSS3/ES6 JavaScript + plix-client (rendering, config, server_browser), plix-common (types, protocol), serde_json (bridge serialization), Feature 030 (CEF shell), Feature 026 (server browser), Feature 025 (account identity)
- 030-cef-ui-shell: Added Rust 1.75+ (stable channel only per constitution) + CEF (binding TBD via spike), wgpu, winit, clap, toml/serde


<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
