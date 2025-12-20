# Testing Checklist: Accessibility

**Purpose**: Validate testing requirements for accessibility features
**Created**: 2025-12-19
**Feature**: [spec.md](../spec.md)

## Unit Tests

### Keybindings

- [ ] `Action::all()` returns all rebindable actions
- [ ] `Key::from_keycode()` maps winit keys correctly
- [ ] `Keybinds::action_for_key()` detects conflicts
- [ ] `Keybinds::swap()` exchanges bindings correctly
- [ ] `Keybinds::ensure_all_actions_bound()` fills missing
- [ ] Default keybinds match WASD+Mouse layout

### Accessibility Config

- [ ] `AccessibilityConfig::default()` has correct values
- [ ] `AccessibilityConfig::validate()` clamps ui_scale (75-150)
- [ ] `ColorblindPreset::css_class()` returns correct classes
- [ ] `SubtitleSize::font_size_px()` returns correct values
- [ ] `AudioEvent::subtitle_text()` returns non-empty strings

### Config Persistence

- [ ] GameConfig serializes with `[accessibility]` section
- [ ] Deserialize missing `[accessibility]` uses defaults
- [ ] Invalid values clamped on load (not rejected)

## Integration Tests

### Keybinding Integration

- [ ] Rebind action in CEF UI, verify config updated
- [ ] Conflict detected when binding used key
- [ ] Swap resolves conflict correctly
- [ ] Reset to defaults clears all custom bindings
- [ ] Bindings persist across game restart

### Visual Accessibility Integration

- [ ] UI scale changes affect CEF element sizes
- [ ] FOV slider updates camera projection
- [ ] High contrast adds CSS class to root
- [ ] Colorblind preset applies correct CSS filter
- [ ] Settings apply on startup from config

### Subtitle Integration

- [ ] Enable subtitles, trigger audio event, verify display
- [ ] Size change affects subtitle text size
- [ ] Background opacity changes visibility
- [ ] Subtitles auto-dismiss after duration

### Native Fallback Integration

- [ ] `/rebind forward up` changes Forward binding
- [ ] `/rebind list` shows all bindings
- [ ] `/ui_scale 125` sets ui_scale to 125
- [ ] `/colorblind protanopia` sets preset
- [ ] Invalid commands show helpful error

## Manual Testing

### Keyboard Testing

- [ ] Test all letter keys (A-Z)
- [ ] Test number keys (0-9)
- [ ] Test function keys (F1-F12)
- [ ] Test special keys (Space, Enter, Escape, Tab)
- [ ] Test mouse buttons (LMB, RMB, MMB)

### Visual Testing

- [ ] UI Scale 75% - elements visibly smaller
- [ ] UI Scale 150% - elements visibly larger
- [ ] High Contrast - borders and text more visible
- [ ] Protanopia - red/green shift visible
- [ ] Deuteranopia - red/green shift visible (different)
- [ ] Tritanopia - blue/yellow shift visible

### Edge Cases

- [ ] Try to unbind Pause action (should warn or block)
- [ ] Set ui_scale to 0 via console (should clamp)
- [ ] Set invalid colorblind preset (should ignore)
- [ ] Rapid key presses during capture (should take first)

## Notes

- Unit tests focus on Rust types and validation
- Integration tests require CEF or native fallback running
- Manual testing required for visual verification
- All tests should run in CI where possible
