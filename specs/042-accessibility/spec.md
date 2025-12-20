# Feature Specification: Accessibility

**Feature Branch**: `042-accessibility`
**Created**: 2025-12-19
**Status**: Draft
**Input**: User description: "Accessibility pass - Keybinding remapping with full ActionMap system, conflict detection, and reset-to-defaults; Visual accessibility options including UI scale, FOV slider, high contrast mode, and colorblind presets; Subtitles system for audio events"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Player Remaps Keybindings (Priority: P1)

A player wants to customize their control scheme to match their preferences or physical needs. They open the settings menu, navigate to the Controls section, and rebind actions to their preferred keys with visual feedback and conflict detection.

**Why this priority**: Core accessibility - players with different physical abilities, keyboard layouts (AZERTY, Dvorak), or preferences cannot play without keybinding customization. This unlocks the game for a much wider audience.

**Independent Test**: Open settings > Controls, rebind "Forward" from W to Up Arrow, verify the change persists after restart and works in-game.

**Acceptance Scenarios**:

1. **Given** a player opens Settings > Controls, **When** they view the keybindings list, **Then** all rebindable actions are displayed with current bindings and display names
2. **Given** a player clicks on an action's binding, **When** they press a new key, **Then** the binding updates to show the new key and enters "listening" state during capture
3. **Given** a player binds a key that's already used, **When** the conflict is detected, **Then** they are shown which action has the conflict and offered to swap bindings
4. **Given** a player has modified keybindings, **When** they click "Reset to Defaults", **Then** all bindings revert to WASD+Mouse defaults
5. **Given** a player changes keybindings, **When** they save and restart the game, **Then** their custom bindings persist from config.toml

---

### User Story 2 - Player Adjusts Visual Accessibility Settings (Priority: P1)

A player with visual impairments or preferences wants to adjust the game's visual presentation. They access Display settings and modify UI scale, FOV, contrast, and colorblind modes with immediate preview.

**Why this priority**: Visual accessibility is essential for players with low vision, colorblindness (~8% of males), or those using non-standard displays. Without these options, the game is unplayable for a significant population.

**Independent Test**: Enable High Contrast mode, verify UI elements have enhanced borders and reduced background opacity. Enable Deuteranopia preset, verify red/green differentiation via CSS filter.

**Acceptance Scenarios**:

1. **Given** a player opens Settings > Display, **When** they adjust the UI Scale slider (75%-150%), **Then** all CEF UI elements scale proportionally with live preview
2. **Given** a player adjusts the FOV slider (60-110 degrees), **When** they change the value, **Then** the camera FOV updates in real-time
3. **Given** a player enables High Contrast mode, **When** the setting is applied, **Then** UI elements show enhanced borders, higher text contrast, and reduced background opacity
4. **Given** a player selects a colorblind preset (Protanopia, Deuteranopia, Tritanopia), **When** applied, **Then** CSS filters adjust colors via hue-rotate/saturate transforms
5. **Given** a player adjusts any visual setting, **When** they save and restart, **Then** settings persist from config.toml and are applied on startup

---

### User Story 3 - Player Enables Subtitles for Game Audio (Priority: P3)

A player who is deaf or hard of hearing, or playing in a noisy environment, wants text captions for important game audio events. They enable subtitles in Audio settings and receive on-screen text for sound cues.

**Why this priority**: Audio accessibility is important but lower priority because the current game has minimal audio events. This provides the foundation for future audio expansion.

**Independent Test**: Enable subtitles, trigger a chat message sound, verify subtitle "[Chat]" appears on screen with configured styling.

**Acceptance Scenarios**:

1. **Given** a player opens Settings > Audio, **When** they enable Subtitles, **Then** the subtitle overlay activates
2. **Given** subtitles are enabled and an audio event fires, **When** the sound plays, **Then** a text caption appears with event description
3. **Given** a player adjusts subtitle size (Small/Medium/Large), **When** applied, **Then** subtitle text scales accordingly
4. **Given** a player adjusts subtitle background opacity, **When** applied, **Then** subtitle background transparency changes for readability
5. **Given** subtitle settings are changed, **When** the player saves and restarts, **Then** settings persist

---

### Edge Cases

- What happens when a player tries to unbind an essential action (e.g., Pause)?
  - Essential actions (Pause) cannot be fully unbound; attempting shows a warning but allows rebinding to a different key
- How does the system handle modifier keys (Ctrl, Shift, Alt)?
  - Modifier keys can be bound as standalone keys for actions, but modifier+key combinations are not supported in v1
- What happens if the colorblind filter causes performance issues?
  - CSS filters are GPU-accelerated; if performance drops below threshold, show warning and suggest disabling
- How does UI scale affect native fallback UI?
  - Native UI (if CEF unavailable) respects ui_scale by adjusting font sizes and element dimensions proportionally
- What happens if config.toml has invalid accessibility values?
  - Values are clamped to valid ranges on load; warnings logged for invalid values

## Requirements *(mandatory)*

### Functional Requirements

**Keybinding Remapping (US1)**

- **FR-001**: System MUST display all rebindable actions (from existing Action enum) with current key binding and display name
- **FR-002**: System MUST enter "listening" state when user clicks a binding, capturing the next key/mouse input; listening state MUST auto-cancel after 5 seconds of no input
- **FR-003**: System MUST detect binding conflicts and display which action has the conflicting key
- **FR-004**: System MUST offer "Swap" option when conflict detected, exchanging bindings between two actions
- **FR-005**: System MUST provide "Reset to Defaults" button that reverts all keybindings to WASD+Mouse defaults
- **FR-006**: System MUST persist keybinding changes to config.toml under `[keybinds.bindings]` section
- **FR-007**: System MUST apply keybinding changes immediately without requiring restart
- **FR-008**: CEF UI MUST send `rebind_action` message to bridge; native fallback MUST use console command

**Visual Accessibility (US2)**

- **FR-009**: System MUST provide UI Scale slider (75%-150%) affecting all CEF UI elements via CSS transform
- **FR-010**: System MUST provide FOV slider (60-110 degrees) with live camera preview
- **FR-011**: System MUST provide High Contrast toggle adding enhanced borders and increased text contrast
- **FR-012**: System MUST provide colorblind presets: None, Protanopia, Deuteranopia, Tritanopia
- **FR-013**: Colorblind presets MUST be implemented via CSS filters (hue-rotate, saturate) on root element
- **FR-014**: System MUST persist visual settings to config.toml under `[accessibility]` section
- **FR-015**: Visual settings MUST apply on game startup from persisted config
- **FR-016**: All visual changes MUST preview live without requiring save/restart

**Subtitles (US3)**

- **FR-017**: System MUST provide Subtitles toggle in Audio settings
- **FR-018**: System MUST display text captions for defined audio events (chat sounds, hit sounds, etc.)
- **FR-019**: System MUST provide subtitle size options: Small (12px), Medium (16px), Large (20px)
- **FR-020**: System MUST provide subtitle background opacity slider (0%-100%)
- **FR-021**: Subtitles MUST appear in a dedicated overlay region (bottom of screen, above HUD)
- **FR-022**: Subtitle text MUST auto-dismiss after configurable duration (default: 3 seconds)
- **FR-022a**: Subtitle queue MUST be limited to 3 simultaneous entries; when exceeded, oldest subtitle is dropped
- **FR-023**: System MUST persist subtitle settings to config.toml under `[accessibility.subtitles]` section

**Native Fallback**

- **FR-024**: All accessibility features MUST have native fallback when CEF is unavailable
- **FR-025**: Native fallback MUST use console commands: `/rebind <action> <key>`, `/ui_scale <percent>`, `/colorblind <preset>`
- **FR-026**: Native fallback for keybinding rebind MUST display in console with action list and current bindings

### Key Entities

- **AccessibilityConfig**: Configuration struct holding ui_scale, high_contrast, colorblind_preset, subtitle settings
- **ColorblindPreset**: Enum of colorblind modes (None, Protanopia, Deuteranopia, Tritanopia)
- **SubtitleConfig**: Configuration for subtitle size, background opacity, duration, enabled state
- **AudioEvent**: Enum of captionable audio events with display text
- **KeybindConflict**: Struct representing a detected binding conflict between two actions

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All 10 existing Action enum variants can be rebound in Settings > Controls
- **SC-002**: Keybinding conflicts are detected and resolved (swap or cancel) 100% of the time
- **SC-003**: UI Scale slider adjusts all CEF elements from 75% to 150% with no layout breakage
- **SC-004**: FOV changes from 60-110 degrees reflect in-game within 1 frame
- **SC-005**: All 4 colorblind presets apply distinct CSS filter values
- **SC-006**: High Contrast mode increases text luminance delta by at least 20%
- **SC-007**: Subtitle text appears within 100ms of audio event trigger
- **SC-008**: All accessibility settings persist correctly across game restarts
- **SC-009**: Native fallback console commands work when CEF is disabled
- **SC-010**: No accessibility feature causes >5% framerate degradation

## Assumptions

- CEF UI is the primary interface (Feature 030+); native fallback is secondary
- Existing Action and Key enums from config.rs are the source of truth for rebindable actions
- Existing GameConfig persistence via config.toml will be extended for accessibility settings
- CSS filters for colorblind modes are supported in CEF's Chromium version
- Audio events that need subtitles are minimal in current scope (chat, basic SFX)
- No gamepad/controller support in v1 (keyboard+mouse only)
- Modifier key combinations (Ctrl+Key) are deferred to future versions

## Clarifications

### Session 2025-12-19

- Q: What should happen if a player doesn't press any key during keybinding capture? → A: 5-second timeout - auto-cancel if no input, revert to previous binding
- Q: How should the subtitle system handle rapid audio events? → A: Queue limit of 3 - drop oldest subtitle when new one arrives and queue is full
