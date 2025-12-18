# Manual Test Checklist: CEF Menus

**Feature**: 031-cef-menus
**Date**: 2025-12-18

## Prerequisites

- Build client with CEF enabled: `cargo build -p plix-client --features cef-ui`
- Have a test server running (optional, for connection tests)

## Test Cases

### Main Menu Navigation

- [ ] **TC-001**: Launch game with CEF enabled
  - Expected: Main menu displays with Play, Servers, Settings, Quit buttons
  - Verify player name is displayed

- [ ] **TC-002**: Click "Servers" button
  - Expected: Server browser page opens
  - Back button returns to main menu

- [ ] **TC-003**: Click "Settings" button
  - Expected: Settings page opens
  - Cancel button returns to main menu

- [ ] **TC-004**: Click "Quit" button
  - Expected: Game exits cleanly without errors

- [ ] **TC-005**: Press ESC on main menu
  - Expected: Menu closes (returns to game if in-game, or no effect at startup)

### Settings Page

- [ ] **TC-010**: Settings load correctly
  - Expected: Current config values displayed (sensitivity, FOV, fullscreen, audio, keybinds)

- [ ] **TC-011**: Modify sensitivity slider
  - Expected: Value updates in real-time, stays within 0.0001-0.01 range

- [ ] **TC-012**: Modify FOV slider
  - Expected: Value updates in real-time, stays within 60-110 range

- [ ] **TC-013**: Toggle fullscreen checkbox
  - Expected: Checkbox state changes

- [ ] **TC-014**: Toggle audio muted checkbox
  - Expected: Checkbox state changes

- [ ] **TC-015**: Click keybind button
  - Expected: Button shows "Press key..." and highlights
  - Press key: Keybind updates
  - Press ESC: Edit cancelled

- [ ] **TC-016**: Keybind conflict detection
  - Expected: Assign same key to two actions → keys swap

- [ ] **TC-017**: Save settings
  - Expected: "Settings saved successfully" message
  - Restart game and verify settings persisted

- [ ] **TC-018**: Invalid sensitivity value (programmatic)
  - Expected: Error message "Sensitivity must be between..."

- [ ] **TC-019**: Press ESC on settings page
  - Expected: Returns to main menu (unsaved changes lost)

### Server Browser Page

- [ ] **TC-020**: Server browser loads
  - Expected: Page displays with search, filters, refresh button

- [ ] **TC-021**: Click Refresh button
  - Expected: Loading spinner shown, server list populated

- [ ] **TC-022**: Search servers
  - Expected: List filters by name (300ms debounce)

- [ ] **TC-023**: Region filter
  - Expected: List filters by selected region

- [ ] **TC-024**: Favorites only filter
  - Expected: Only favorited servers shown

- [ ] **TC-025**: Click server row
  - Expected: Row highlights, Connect button enables

- [ ] **TC-026**: Double-click server row
  - Expected: Connection attempt starts

- [ ] **TC-027**: Toggle favorite (star icon)
  - Expected: Star toggles, persists across refresh

- [ ] **TC-028**: Connect to server
  - Expected: Modal shows "Connecting...", then "Connected!" or error

- [ ] **TC-029**: Connect to incompatible server
  - Expected: Error modal "Version mismatch"

- [ ] **TC-030**: Empty server list
  - Expected: "No servers found" message

- [ ] **TC-031**: Master server unreachable
  - Expected: Error message displayed

- [ ] **TC-032**: Press ESC on server browser
  - Expected: Returns to main menu

### Input Focus Isolation

- [ ] **TC-040**: Type in search field
  - Expected: Text appears in field, no game character movement

- [ ] **TC-041**: Click in UI area
  - Expected: No weapon fire or game actions

- [ ] **TC-042**: Mouse movement over UI
  - Expected: No camera rotation

### CEF Fallback

- [ ] **TC-050**: Launch without cef-ui feature
  - Expected: Native UI (Feature 005) displays instead

- [ ] **TC-051**: CEF initialization failure
  - Expected: Automatic fallback to native UI

### Persistence

- [ ] **TC-060**: Favorites persist across sessions
  - Expected: Favorite servers still marked after restart

- [ ] **TC-061**: Config persists across sessions
  - Expected: Modified settings still applied after restart

- [ ] **TC-062**: Corrupt favorites.toml
  - Expected: Reset to empty, warning logged

- [ ] **TC-063**: Corrupt config.toml
  - Expected: Reset to defaults, warning logged

### Performance

- [ ] **TC-070**: UI interaction frame rate
  - Expected: No drops below 60fps during normal UI use

- [ ] **TC-071**: Server list with 100+ servers
  - Expected: Smooth scrolling and filtering

### Error Handling

- [ ] **TC-080**: Malformed bridge message
  - Expected: Error response, no crash

- [ ] **TC-081**: Unknown message type
  - Expected: EBRG003 error returned

- [ ] **TC-082**: Connection timeout
  - Expected: ECON001 error displayed

## Test Results

| Test ID | Status | Notes |
|---------|--------|-------|
| TC-001  | [ ]    |       |
| TC-002  | [ ]    |       |
| TC-003  | [ ]    |       |
| ...     | ...    |       |

## Notes

- All tests should be run on Linux (primary platform)
- Test both with and without --cef-devtools flag
- Verify console logs for any warnings or errors
