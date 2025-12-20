# Research: Accessibility

## Technical Decisions

### Decision 1: Keybinding Persistence Strategy

**Options Considered**:
1. Extend existing `[keybinds.bindings]` section in config.toml
2. Create separate `keybinds.toml` file
3. Use JSON for more complex binding structures

**Decision**: Extend existing `[keybinds.bindings]` section

**Rationale**:
- The codebase already has `Keybinds` struct in `crates/plix-client/src/config.rs` with HashMap<Action, Key>
- GameConfig already includes `keybinds: Keybinds` field
- TOML serialization already works via serde
- No migration needed - backward compatible with existing configs

**Reference**: `crates/plix-client/src/config.rs:291-354`

---

### Decision 2: Colorblind Filter Implementation

**Options Considered**:
1. CSS filters (filter: hue-rotate, saturate, contrast)
2. WebGL shader post-processing
3. Pre-computed color palettes with runtime swap

**Decision**: CSS filters on root CEF element

**Rationale**:
- CSS filters are GPU-accelerated in Chromium
- Simple implementation: single CSS rule change
- Standard colorblind simulation values available:
  - Protanopia: `filter: url(#protanopia)` or `hue-rotate(180deg) saturate(2.5)`
  - Deuteranopia: `filter: url(#deuteranopia)` or `hue-rotate(180deg) saturate(1.5)`
  - Tritanopia: `filter: url(#tritanopia)` or `hue-rotate(-60deg) saturate(2.0)`
- Falls back gracefully (no filter = normal vision)
- CEF Chromium supports all filter properties

**Implementation**:
```css
/* Applied to html or body element */
.colorblind-protanopia { filter: url('#protanopia-filter') !important; }
.colorblind-deuteranopia { filter: url('#deuteranopia-filter') !important; }
.colorblind-tritanopia { filter: url('#tritanopia-filter') !important; }

/* SVG filters provide more accurate simulation */
<svg style="display:none">
  <filter id="protanopia-filter">
    <feColorMatrix values="0.567,0.433,0,0,0 0.558,0.442,0,0,0 0,0.242,0.758,0,0 0,0,0,1,0"/>
  </filter>
</svg>
```

---

### Decision 3: UI Scale Implementation

**Options Considered**:
1. CSS `transform: scale()` on root element
2. CSS `zoom` property
3. Dynamic font-size with rem units
4. Viewport meta tag manipulation

**Decision**: CSS `transform: scale()` on CEF container

**Rationale**:
- `transform: scale()` is hardware-accelerated
- Maintains aspect ratio automatically
- Works with existing pixel-based layouts
- No need to refactor all CSS to rem units
- `zoom` has inconsistent browser support (though CEF is consistent)

**Implementation**:
```css
.ui-scale-container {
  transform-origin: top left;
  transform: scale(var(--ui-scale, 1.0));
}
```

Native fallback: Adjust wgpu text rendering scale factor.

---

### Decision 4: High Contrast Mode Implementation

**Options Considered**:
1. Separate high-contrast CSS stylesheet
2. CSS custom properties with contrast theme
3. `prefers-contrast` media query only
4. Filter-based contrast enhancement

**Decision**: CSS custom properties with .high-contrast class

**Rationale**:
- CSS custom properties allow targeted adjustments
- Single class toggle on root element
- Works alongside colorblind filters
- No separate stylesheet to maintain

**Implementation**:
```css
:root {
  --text-primary: #e0e0e0;
  --text-secondary: #a0a0a0;
  --bg-panel: rgba(0, 0, 0, 0.7);
  --border-default: transparent;
}

:root.high-contrast {
  --text-primary: #ffffff;
  --text-secondary: #ffffff;
  --bg-panel: rgba(0, 0, 0, 0.95);
  --border-default: 2px solid #ffffff;
}
```

---

### Decision 5: Subtitle Overlay Architecture

**Options Considered**:
1. Dedicated CEF overlay (new browser instance)
2. Shared HUD overlay with subtitle region
3. Native wgpu text rendering for subtitles

**Decision**: Shared HUD overlay with dedicated subtitle region

**Rationale**:
- Existing `ingame/` overlay already handles HUD elements (Feature 032)
- Subtitles are just another HUD component
- No additional CEF browser overhead
- Consistent styling with other HUD elements

**Implementation**:
- Add `subtitle-container` div to ingame overlay HTML
- Bridge message `subtitle_show { event_id, text, duration_ms }`
- JS manages queue and auto-dismiss timers

---

### Decision 6: Keybinding Capture UX

**Options Considered**:
1. Modal dialog for key capture
2. Inline capture (highlight row, listen)
3. Dedicated rebind screen

**Decision**: Inline capture with visual state change

**Rationale**:
- Less disruptive than modal
- Clearer context (see the action being rebound)
- Pattern used by most games (Minecraft, CS:GO)
- Escape key cancels capture

**UX Flow**:
1. User clicks on key binding cell
2. Cell shows "Press a key..." with pulsing border
3. Next key/mouse input is captured
4. If conflict: show conflict modal with Swap/Cancel options
5. If no conflict: update immediately

---

### Decision 7: Native Fallback Commands

**Options Considered**:
1. Console commands only
2. Minimal TUI menus
3. Command-line arguments

**Decision**: Console commands with help text

**Rationale**:
- Existing console system from Feature 005
- Consistent with other native fallbacks
- No additional UI complexity
- Accessible to power users

**Commands**:
```
/rebind <action> <key>     - Rebind action (e.g., /rebind forward up)
/rebind list               - Show all current bindings
/rebind reset              - Reset all to defaults
/ui_scale <75-150>         - Set UI scale percentage
/fov <60-110>              - Set camera FOV
/colorblind <preset>       - Set colorblind mode (none/protanopia/deuteranopia/tritanopia)
/highcontrast <on/off>     - Toggle high contrast
/subtitles <on/off>        - Toggle subtitles
```

---

## Existing Code Integration Points

### Config System (`crates/plix-client/src/config.rs`)

Current structure to extend:
```rust
pub struct GameConfig {
    pub sensitivity: f32,
    pub fov_degrees: f32,  // Already exists!
    pub fullscreen: bool,
    pub audio_muted: bool,
    pub keybinds: Keybinds,  // Already exists!
    pub ui: CefConfig,
    // ADD: pub accessibility: AccessibilityConfig,
}
```

### Key/Action Enums

Already defined in `config.rs`:
- `Action` enum with 10 variants (Forward, Backward, Left, Right, Jump, Attack, PlaceBlock, RemoveBlock, Pause, ToggleDebugOverlay)
- `Key` enum with all keyboard keys and mouse buttons
- `Keybinds` struct with conflict detection via `action_for_key()`

### CEF Bridge (`crates/plix-client/src/ui_cef/bridge/`)

Existing message types to extend:
- Add `RebindAction { action, key }` to outbound messages
- Add `AccessibilityUpdate { ui_scale, colorblind, high_contrast }` to outbound messages
- Add `SubtitleShow { event_id, text }` to inbound messages (Rust -> JS)

### Camera FOV (`crates/plix-client/src/render/camera.rs`)

FOV already configurable - just need to expose slider and ensure live update works.

---

## Dependencies

- **Existing**: serde, toml, winit (for key capture)
- **No new crates required** - all features use existing infrastructure

---

## Performance Considerations

1. **CSS Filters**: GPU-accelerated, ~1-2% overhead maximum
2. **UI Scale Transform**: GPU-accelerated, negligible overhead
3. **Subtitle Rendering**: Text-only, negligible
4. **Key Capture**: Event-based, no polling

No performance concerns expected.
