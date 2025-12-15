# Feature Specification: Minimal Native UI

**Feature Branch**: `005-minimal-ui-native`
**Created**: 2025-12-15
**Status**: Draft
**Input**: User description: "Provide a minimal native UI layer (no CEF) to make the game playable and configurable: crosshair, pause menu, mouse sensitivity, keybinds, FOV, fullscreen toggle, and audio on/off."

## Clarifications

### Session 2025-12-15

- Q: When a player tries to bind a key that's already in use, what should happen? → A: Warn + Swap - Show conflict warning, then swap bindings if user confirms

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Crosshair Display (Priority: P1) 🎯 MVP

As a player, I see a crosshair at screen center so I can aim when interacting with blocks and engaging in combat.

**Why this priority**: Without a crosshair, players cannot accurately aim for block placement/removal or combat. This is the most fundamental UI element for gameplay.

**Independent Test**: Launch windowed client, verify crosshair appears at screen center during gameplay, disappears when menu is open.

**Acceptance Scenarios**:

1. **Given** the game is running and not paused, **When** I look at the screen, **Then** I see a crosshair centered on the display
2. **Given** the pause menu is open, **When** I look at the screen, **Then** the crosshair is hidden
3. **Given** I resize the window, **When** the window size changes, **Then** the crosshair remains centered

---

### User Story 2 - Pause Menu Navigation (Priority: P1) 🎯 MVP

As a player, I can pause the game with ESC to access a menu without disconnecting from the server, and resume gameplay when ready.

**Why this priority**: Players need to pause to access settings, take breaks, or quit gracefully. This is essential for a playable game.

**Independent Test**: Press ESC during gameplay, verify menu appears, mouse is released, inputs are blocked, network stays connected. Press Resume or ESC again to return to gameplay.

**Acceptance Scenarios**:

1. **Given** I am playing the game with mouse captured, **When** I press ESC, **Then** the pause menu appears, mouse cursor is visible and can interact with menu items, and gameplay inputs are blocked
2. **Given** the pause menu is open, **When** I click Resume (or press ESC), **Then** the menu closes, mouse is recaptured, and gameplay resumes
3. **Given** the pause menu is open, **When** I click Quit to Desktop, **Then** the client exits cleanly
4. **Given** the pause menu is open for 60 seconds, **When** I check the network connection, **Then** I remain connected to the server

---

### User Story 3 - Mouse Sensitivity Setting (Priority: P2)

As a player, I can adjust mouse sensitivity so that look controls feel comfortable for my preferences.

**Why this priority**: Mouse sensitivity is highly personal and affects playability. Players often need to adjust this immediately after launching.

**Independent Test**: Open Settings, adjust sensitivity slider/control, verify mouse look speed changes immediately in-game.

**Acceptance Scenarios**:

1. **Given** I open the Settings menu, **When** I adjust the mouse sensitivity control, **Then** the change applies immediately to mouse look
2. **Given** I have changed sensitivity, **When** I restart the game, **Then** my sensitivity setting is preserved
3. **Given** I adjust sensitivity to minimum, **When** I move the mouse, **Then** camera rotation is slower than default
4. **Given** I adjust sensitivity to maximum, **When** I move the mouse, **Then** camera rotation is faster than default

---

### User Story 4 - Field of View Setting (Priority: P2)

As a player, I can adjust my field of view (FOV) to see more or less of the game world according to my preference.

**Why this priority**: FOV affects comfort (motion sickness) and competitive advantage. Players expect to customize this.

**Independent Test**: Open Settings, adjust FOV slider, verify the view angle changes immediately.

**Acceptance Scenarios**:

1. **Given** I open the Settings menu, **When** I adjust the FOV control between 60 and 110 degrees, **Then** the view angle changes immediately
2. **Given** I have changed FOV to 90, **When** I restart the game, **Then** FOV is still 90
3. **Given** I set FOV to 60, **When** I look around, **Then** the view appears more zoomed in
4. **Given** I set FOV to 110, **When** I look around, **Then** the view appears wider

---

### User Story 5 - Fullscreen Toggle (Priority: P2)

As a player, I can toggle between windowed and fullscreen mode for optimal viewing.

**Why this priority**: Fullscreen is standard for games and affects performance and immersion.

**Independent Test**: Open Settings, toggle fullscreen option, verify window mode changes.

**Acceptance Scenarios**:

1. **Given** the game is in windowed mode, **When** I enable fullscreen in Settings, **Then** the game switches to fullscreen/borderless mode
2. **Given** the game is in fullscreen mode, **When** I disable fullscreen in Settings, **Then** the game returns to windowed mode
3. **Given** I enable fullscreen, **When** I restart the game, **Then** fullscreen mode is preserved

---

### User Story 6 - Keybind Customization (Priority: P3)

As a player, I can rebind controls so I can use my preferred key layout.

**Why this priority**: While important for accessibility and preference, default controls are functional. This can be deferred after core UI works.

**Independent Test**: Open Settings, select a keybind to change, press a new key, verify the action now uses the new key.

**Acceptance Scenarios**:

1. **Given** I open the keybinds settings, **When** I select "Forward" and press a new key, **Then** the Forward action is bound to that key
2. **Given** I have rebound Forward to a different key, **When** I press that key in-game, **Then** my character moves forward
3. **Given** I try to bind a key already used by another action, **When** I attempt the rebind, **Then** the system shows a conflict warning and upon confirmation swaps the bindings between the two actions
4. **Given** I have customized keybinds, **When** I restart the game, **Then** my keybinds are preserved
5. **Given** the default keybinds, **When** I view the keybinds settings, **Then** I see bindings for: Forward, Backward, Left, Right, Jump, Attack, Place Block, Remove Block, and Pause

---

### User Story 7 - Audio Mute Toggle (Priority: P3)

As a player, I can mute and unmute game audio with a simple toggle.

**Why this priority**: Audio may not yet be implemented; this provides the settings infrastructure and placeholder.

**Independent Test**: Open Settings, toggle audio on/off, verify setting persists.

**Acceptance Scenarios**:

1. **Given** audio is enabled, **When** I toggle audio off, **Then** the setting shows audio is muted
2. **Given** audio is muted, **When** I toggle audio on, **Then** the setting shows audio is enabled
3. **Given** I have muted audio, **When** I restart the game, **Then** audio remains muted

---

### Edge Cases

- What happens when the player presses ESC rapidly multiple times? Menu should toggle cleanly without flickering.
- How does the system handle pressing ESC while already in a settings sub-menu? Should return to main pause menu.
- What happens if the config file is corrupted or missing? System loads defaults and recreates the file.
- What happens when a keybind is unbound (removed)? The action becomes unusable until rebound.
- What happens when fullscreen fails (unsupported resolution)? System remains in windowed mode and logs a warning.
- What happens when sensitivity is set outside the valid range via config editing? Value is clamped to valid range on load.

## Requirements *(mandatory)*

### Functional Requirements

#### Crosshair

- **FR-001**: System MUST display a crosshair at screen center during active gameplay
- **FR-002**: System MUST hide the crosshair when any menu is open
- **FR-003**: Crosshair MUST remain centered when window is resized

#### Pause Menu

- **FR-004**: Pressing ESC MUST toggle the pause menu open/closed
- **FR-005**: When pause menu is open, system MUST release mouse cursor for menu interaction
- **FR-006**: When pause menu is open, system MUST block gameplay inputs (movement, attack, block edits)
- **FR-007**: When pause menu is open, system MUST maintain network connection to server
- **FR-008**: Pause menu MUST include options: Resume, Settings, Quit to Desktop
- **FR-009**: Resume option MUST close menu and recapture mouse cursor
- **FR-010**: Quit to Desktop option MUST cleanly exit the client

#### Settings - Sensitivity

- **FR-011**: System MUST provide a control to adjust mouse sensitivity
- **FR-012**: Sensitivity changes MUST apply immediately without restart
- **FR-013**: Sensitivity setting MUST be persisted to config file

#### Settings - FOV

- **FR-014**: System MUST provide a control to adjust FOV between 60 and 110 degrees
- **FR-015**: FOV changes MUST apply immediately without restart
- **FR-016**: FOV setting MUST be persisted to config file

#### Settings - Fullscreen

- **FR-017**: System MUST provide a toggle for fullscreen/windowed mode
- **FR-018**: Fullscreen toggle MUST take effect immediately
- **FR-019**: Fullscreen setting MUST be persisted to config file

#### Settings - Keybinds

- **FR-020**: System MUST allow rebinding of: Forward, Backward, Left, Right, Jump, Attack, Place Block, Remove Block, Pause
- **FR-021**: System MUST handle duplicate key bindings by showing a conflict warning and swapping bindings upon user confirmation
- **FR-022**: Keybind changes MUST apply immediately without restart
- **FR-023**: Keybind settings MUST be persisted to config file

#### Settings - Audio

- **FR-024**: System MUST provide a master audio on/off toggle
- **FR-025**: Audio setting MUST be persisted to config file
- **FR-026**: If audio system is not implemented, the setting MUST still exist and persist

#### Config Persistence

- **FR-027**: System MUST load configuration from a config file on startup
- **FR-028**: If config file is missing, system MUST use default values
- **FR-029**: System MUST save configuration when settings are changed
- **FR-030**: Config file MUST be stored in a standard location (e.g., ~/.config/plix/ on Linux)
- **FR-031**: System MUST handle corrupted config files by falling back to defaults

#### Compatibility

- **FR-032**: UI features MUST NOT affect headless server mode
- **FR-033**: Pause state MUST NOT cause network disconnection

### Key Entities

- **GameConfig**: Represents all persistent user settings including sensitivity, FOV, fullscreen preference, keybinds, and audio mute state
- **MenuState**: Represents current UI state (Playing, Paused, Settings, KeybindEdit)
- **Keybind**: Maps an action name to a key code, with all rebindable actions tracked

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Players can identify the center of their aim within 1 second of gameplay start (crosshair visible)
- **SC-002**: Players can pause and resume gameplay within 2 seconds total interaction time
- **SC-003**: Settings changes take effect within 100ms of user action (immediate feedback)
- **SC-004**: Configuration persists across 100% of normal restarts
- **SC-005**: Pause menu can remain open for at least 5 minutes without network disconnection
- **SC-006**: All 9 rebindable actions can be customized and saved
- **SC-007**: Fullscreen toggle completes within 2 seconds
- **SC-008**: Headless server mode continues to function without UI code interference
- **SC-009**: All existing tests continue to pass (`cargo test --workspace`)

## Assumptions

- The crosshair will be a simple visual (lines or dot) that does not require complex rendering
- Audio system may not be implemented yet; the audio toggle is a placeholder setting
- Config file format will be TOML for readability and Rust ecosystem compatibility
- Default keybinds follow standard FPS conventions (WASD, Space, LMB/RMB)
- The pause menu uses simple text-based rendering, not a full GUI framework
- Window management is handled through the existing winit integration

## Scope Boundaries

### In Scope

- Crosshair rendering
- Pause menu with Resume/Settings/Quit
- Settings for: sensitivity, FOV, fullscreen, keybinds, audio mute
- Config file persistence

### Out of Scope

- CEF or web-based UI
- Fancy UI frameworks or animations
- Localization/internationalization
- Account/login UI
- Server browser
- Graphics quality settings beyond FOV
- Audio volume slider (only mute toggle)
- In-game HUD beyond crosshair
