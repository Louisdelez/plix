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
- 010-logging-metrics: Added Rust 1.75+ (stable channel only per constitution) + tokio (async), bincode (serialization), glam (math), wgpu (client rendering), tracing (logging)
- 008-movement-polish: Added Rust 1.75+ (stable channel only per constitution) + glam (math), bincode (serialization), tokio (async), wgpu (client rendering)
- 007-anti-cheat-hardening: Added Rust 1.75+ (stable channel only per constitution) + tokio (async), bincode (serialization), glam (math)


<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
