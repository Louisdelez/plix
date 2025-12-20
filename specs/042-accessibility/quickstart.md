# Quickstart: Accessibility Feature Validation

**Feature**: 042-accessibility
**Purpose**: Quick validation scenarios to verify accessibility features work correctly

## Prerequisites

- plix-client builds successfully: `cargo build --release -p plix-client`
- CEF runtime available (or test native fallback)
- No existing `~/.config/plix/config.toml` (or backup existing)

---

## Scenario 1: Keybinding Rebind

**Goal**: Verify keybinding remapping works with persistence.

### Steps

1. Launch client: `./target/release/plix-client`
2. Open Settings > Controls
3. Click on "Forward" binding (currently "W")
4. Press "Up Arrow"
5. Verify binding changes to "Up"
6. Quit and restart client
7. Verify "Forward" is still bound to "Up"

### Expected Result

- Binding cell shows "Press a key..." during capture
- After pressing Up, cell shows "Up Arrow"
- `~/.config/plix/config.toml` contains `Forward = "Up"` in `[keybinds.bindings]`
- Binding persists across restart

### Native Fallback Test

```bash
# In game console
/rebind list              # Shows all bindings
/rebind forward up        # Rebind Forward to Up
/rebind list              # Verify change
```

---

## Scenario 2: Keybind Conflict Resolution

**Goal**: Verify conflict detection and swap functionality.

### Steps

1. Open Settings > Controls
2. Click on "Jump" binding (currently "Space")
3. Press "W" (already bound to Forward)
4. Verify conflict modal appears
5. Click "Swap"
6. Verify Jump is now "W" and Forward is now "Space"

### Expected Result

- Conflict modal shows: "W is already bound to Forward"
- Options: "Swap" and "Cancel"
- After swap: Jump = W, Forward = Space
- Both changes persist to config.toml

---

## Scenario 3: Keybind Capture Timeout

**Goal**: Verify 5-second timeout auto-cancels capture.

### Steps

1. Open Settings > Controls
2. Click on "Attack" binding
3. Wait 5 seconds without pressing any key
4. Verify capture mode exits automatically

### Expected Result

- "Press a key..." shown for ~5 seconds
- Automatically reverts to previous binding display
- No config change

---

## Scenario 4: UI Scale

**Goal**: Verify UI scaling works with live preview.

### Steps

1. Open Settings > Display
2. Find UI Scale slider (default: 100%)
3. Drag to 75%
4. Verify UI elements shrink immediately
5. Drag to 150%
6. Verify UI elements enlarge immediately
7. Save settings

### Expected Result

- All CEF UI elements scale proportionally
- No layout breakage at 75% or 150%
- `~/.config/plix/config.toml` contains `ui_scale = 150` in `[accessibility]`

### Native Fallback Test

```bash
/ui_scale 125     # Set to 125%
/ui_scale 75      # Set to 75%
```

---

## Scenario 5: Colorblind Presets

**Goal**: Verify colorblind filters apply correctly.

### Steps

1. Open Settings > Display
2. Find Colorblind dropdown (default: None)
3. Select "Deuteranopia (Green-blind)"
4. Verify color shift is visible
5. Select "Protanopia (Red-blind)"
6. Verify different color shift
7. Select "None"
8. Verify colors return to normal

### Expected Result

- Each preset applies a distinct CSS filter
- Colors visibly shift when preset is active
- `~/.config/plix/config.toml` contains `colorblind_preset = "deuteranopia"` etc.

### Native Fallback Test

```bash
/colorblind deuteranopia
/colorblind protanopia
/colorblind tritanopia
/colorblind none
```

---

## Scenario 6: High Contrast Mode

**Goal**: Verify high contrast mode enhances visibility.

### Steps

1. Open Settings > Display
2. Find High Contrast toggle (default: off)
3. Enable High Contrast
4. Verify UI shows enhanced borders and brighter text
5. Disable High Contrast
6. Verify UI returns to normal

### Expected Result

- Panel borders become visible (white/bright)
- Text contrast increases
- Background opacity increases
- `~/.config/plix/config.toml` contains `high_contrast = true`

### Native Fallback Test

```bash
/highcontrast on
/highcontrast off
```

---

## Scenario 7: FOV Slider

**Goal**: Verify FOV changes apply to camera in real-time.

### Steps

1. Open Settings > Display
2. Find FOV slider (default: 70)
3. Drag to 60 (narrow FOV)
4. If in-game, verify view narrows
5. Drag to 110 (wide FOV)
6. Verify view widens

### Expected Result

- Camera FOV changes within 1 frame
- No visual glitches during transition
- `~/.config/plix/config.toml` contains `fov_degrees = 110.0`

---

## Scenario 8: Subtitles Enable/Disable

**Goal**: Verify subtitle system toggles correctly.

### Steps

1. Open Settings > Audio
2. Find Subtitles toggle (default: off)
3. Enable Subtitles
4. Trigger a captionable event (e.g., receive chat message)
5. Verify subtitle appears on screen
6. Disable Subtitles
7. Trigger event again
8. Verify no subtitle appears

### Expected Result

- Subtitle overlay appears at bottom of screen
- Text matches event (e.g., "[Chat]")
- Subtitle auto-dismisses after ~3 seconds
- No subtitle when disabled

### Native Fallback Test

```bash
/subtitles on
/subtitles off
```

---

## Scenario 9: Subtitle Queue Limit

**Goal**: Verify max 3 subtitle queue with oldest dropped.

### Steps

1. Enable Subtitles
2. Rapidly trigger 5+ audio events (e.g., break multiple blocks)
3. Observe subtitle display

### Expected Result

- Maximum 3 subtitles displayed simultaneously
- Oldest subtitle dropped when 4th arrives
- All subtitles auto-dismiss after duration

---

## Scenario 10: Config Persistence

**Goal**: Verify all accessibility settings persist across restart.

### Steps

1. Configure all settings to non-default values:
   - UI Scale: 125%
   - High Contrast: On
   - Colorblind: Protanopia
   - Subtitles: On
   - Subtitle Size: Large
   - FOV: 90
2. Quit client
3. Inspect `~/.config/plix/config.toml`
4. Restart client
5. Open Settings, verify all values retained

### Expected Config Section

```toml
[accessibility]
ui_scale = 125
high_contrast = true
colorblind_preset = "protanopia"

[accessibility.subtitles]
enabled = true
size = "large"
background_opacity = 75
duration_ms = 3000
```

---

## Performance Validation

**Goal**: Verify <5% framerate degradation (SC-010).

### Steps

1. Enable FPS counter (F3 or debug overlay)
2. Note baseline FPS in gameplay
3. Enable all accessibility features:
   - High Contrast: On
   - Colorblind: Deuteranopia
   - UI Scale: 150%
   - Subtitles: On
4. Note FPS with all features enabled
5. Calculate degradation: `(baseline - enabled) / baseline * 100`

### Expected Result

- FPS degradation < 5%
- No stuttering or frame drops

---

## Troubleshooting

### Settings Not Persisting

1. Check config path: `~/.config/plix/config.toml`
2. Verify write permissions
3. Check for TOML parse errors in logs

### CEF UI Not Updating

1. Check browser console for JS errors (if devtools enabled)
2. Verify bridge messages in logs (`debug_bridge = true`)
3. Test native fallback to isolate issue

### CSS Filters Not Applying

1. Verify SVG filters embedded in HTML
2. Check `filter` CSS property support
3. Inspect element to see applied styles

### Key Capture Not Working

1. Verify window has focus
2. Check for conflicting key handlers
3. Test with different keys (some may be reserved)
