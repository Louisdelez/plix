# Quickstart: Minimal Native UI

**Feature**: 005-minimal-ui-native
**Date**: 2025-12-15

## Prerequisites

- Rust 1.75+ (stable)
- Cargo workspace set up
- plix-client builds successfully (`cargo build -p plix-client`)

## New Dependencies

Add to `crates/plix-client/Cargo.toml`:

```toml
[dependencies]
toml = "0.8"
dirs = "5.0"
```

## Development Setup

1. **Switch to feature branch**:
   ```bash
   git checkout 005-minimal-ui-native
   ```

2. **Verify build**:
   ```bash
   cargo build --workspace
   ```

3. **Run tests**:
   ```bash
   cargo test --workspace
   ```

## Key Files to Modify/Create

### New Files

| File | Purpose |
|------|---------|
| `crates/plix-client/src/config.rs` | Config loading, saving, GameConfig struct |
| `crates/plix-client/src/ui/crosshair.rs` | Crosshair rendering |
| `crates/plix-client/src/ui/menu.rs` | Pause menu, settings menu, state machine |
| `crates/plix-client/tests/config_test.rs` | Config persistence tests |

### Modified Files

| File | Changes |
|------|---------|
| `crates/plix-client/src/main.rs` | Add MenuState, handle ESC, integrate config |
| `crates/plix-client/src/input.rs` | Add Action enum, extend Key enum, rebindable input |
| `crates/plix-client/src/render/camera.rs` | Add `set_fov()` method |
| `crates/plix-client/src/render/engine.rs` | Add fullscreen toggle support |
| `crates/plix-client/src/ui/mod.rs` | Export new modules |

## Testing During Development

### Manual Testing

1. **Crosshair**:
   ```bash
   cargo run -p plix-client -- --server 127.0.0.1:7777 --name Test
   ```
   - Verify crosshair at screen center
   - Resize window, verify crosshair stays centered

2. **Pause Menu**:
   - Press ESC, verify menu opens
   - Verify cursor released
   - Press ESC again, verify resume
   - Verify crosshair hidden when paused

3. **Settings**:
   - Open pause menu, select Settings
   - Adjust sensitivity, verify mouse feel changes
   - Adjust FOV, verify view changes
   - Toggle fullscreen
   - Restart, verify settings persist

4. **Keybinds**:
   - Open keybinds settings
   - Rebind Forward to a different key
   - Verify movement uses new key
   - Try binding to a key already in use (conflict)
   - Verify swap behavior

### Automated Tests

```bash
# Config tests
cargo test -p plix-client config_

# All tests
cargo test --workspace

# Clippy
cargo clippy --workspace

# Format check
cargo fmt --check
```

## Config File Location

During development, config is saved to:
- Linux: `~/.config/plix/config.toml`
- To test fresh config: `rm ~/.config/plix/config.toml`

## Debugging Tips

1. **View config changes**:
   ```bash
   cat ~/.config/plix/config.toml
   ```

2. **Reset to defaults**:
   ```bash
   rm ~/.config/plix/config.toml
   ```

3. **Test corrupted config**:
   ```bash
   echo "invalid toml {{" > ~/.config/plix/config.toml
   cargo run -p plix-client -- --server 127.0.0.1:7777
   # Should use defaults, log warning
   ```

4. **Debug menu state**:
   Window title shows current state (Paused, Settings, etc.)

## Implementation Order

1. **Phase 2.1**: Config infrastructure (load/save)
2. **Phase 2.2**: Crosshair rendering
3. **Phase 2.3**: Pause menu + state machine
4. **Phase 2.4**: Settings menu (sensitivity, FOV, fullscreen, audio)
5. **Phase 2.5**: Keybind system
6. **Phase 2.6**: Integration (apply config on load)
7. **Phase 2.7**: Validation (all tests pass)

## Common Issues

### Issue: Config not saving

- Check directory permissions: `ls -la ~/.config/`
- Check for error logs in terminal output
- Verify `dirs` crate returns correct path

### Issue: Crosshair not visible

- Check `should_show_crosshair()` returns true
- Verify MenuState is None
- Check render order (crosshair after 3D scene)

### Issue: Cursor not releasing on pause

- Check `release_cursor()` called
- Check winit cursor grab mode
- Try different CursorGrabMode (Locked vs Confined)

### Issue: Settings not applying

- Verify `set_sensitivity()` / `set_fov()` called
- Check value clamping
- Add debug logging to confirm values

## Reference Commands

```bash
# Build and run client
cargo run -p plix-client -- --server 127.0.0.1:7777 --name Dev

# Run with debug logging
RUST_LOG=plix=debug cargo run -p plix-client -- --server 127.0.0.1:7777

# Start server for testing
cargo run -p plix-server -- --addr 127.0.0.1:7777
```
