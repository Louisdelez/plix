# Tasks: CEF Menus (Main Menu / Settings / Server Browser)

**Feature**: 031-cef-menus
**Generated**: 2025-12-18
**Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md)

## Task Summary

| Phase | Tasks | Description |
|-------|-------|-------------|
| 1 | T001-T003 | UI Assets & App Shell |
| 2 | T004-T007 | Bridge JS↔Rust |
| 3 | T008-T010 | Main Menu |
| 4 | T011-T015 | Settings |
| 5 | T016-T018 | Server Browser Data |
| 6 | T019-T024 | Server Browser UI |
| 7 | T025-T027 | Favorites |
| 8 | T028-T029 | Input/Focus |
| 9 | T030-T032 | Security & Robustness |
| 10 | T033-T037 | Tests |
| 11 | T038-T039 | Polish |
| 12 | T040-T042 | Documentation |

## Dependency Graph

```
T001 ─┬─► T002 ─► T003 ─┬─► T004 ─► T005 ─► T006 ─► T007
      │                 │
      │                 └─► T008 ─► T009 ─► T010
      │
      └─► T011 ─► T012 ─► T013 ─► T014 ─► T015
                                          │
T016 ─► T017 ─► T018 ─────────────────────┼─► T019 ─► T020 ─► T021 ─► T022 ─► T023 ─► T024
                                          │
T025 ─► T026 ─► T027 ──────────────────────┘

T028 ─► T029 (depends on T010)

T030 ─► T031 ─► T032 (depends on T007)

T033 ─► T034 ─► T035 ─► T036 ─► T037 (depends on T032)

T038 ─► T039 (depends on T037)

T040 ─► T041 ─► T042 (depends on T039)
```

## Parallel Execution Opportunities

The following task groups can be executed in parallel:

1. **After T003**: T004-T007 (Bridge) || T008-T010 (Main Menu base)
2. **After T007**: T011-T015 (Settings) || T016-T018 (Server Data) || T025-T027 (Favorites)
3. **After T018+T027**: T019-T024 (Server Browser UI)
4. **After T010**: T028-T029 (Input/Focus)

---

## Phase 1 – UI Assets & App Shell

### T001 [P1] [US1] Create assets/ui/ directory structure for CEF menus
**File**: `assets/ui/`

Create the base directory structure for UI assets:
- `assets/ui/index.html` - App shell with router mount point
- `assets/ui/app.js` - Main application entry point (empty)
- `assets/ui/styles.css` - Global styles (empty)
- `assets/ui/pages/` - Directory for page modules
- `assets/ui/components/` - Directory for reusable components

**Acceptance**: Directory structure exists, index.html loads in browser without errors.

- [ ] T001 Create assets/ui/ directory structure

---

### T002 [P1] [US1] Implement index.html app shell with hash router
**File**: `assets/ui/index.html`

Create the HTML shell that:
- Declares DOCTYPE and viewport meta
- Links styles.css
- Contains `<div id="app"></div>` mount point
- Loads app.js as module
- Includes basic loading indicator

**Acceptance**: index.html renders loading state, no console errors.

**Depends on**: T001

- [ ] T002 Implement index.html app shell

---

### T003 [P1] [US1] Implement app.js with hash-based SPA router
**File**: `assets/ui/app.js`

Implement the main application logic:
- Hash-based router listening to `hashchange` event
- Route mapping: `#/` → main, `#/settings` → settings, `#/servers` → servers
- `renderPage(route)` function that swaps content
- `window.plix` bridge stub for development
- Initial route detection on load

**Acceptance**: Navigating to `#/settings` swaps page content, back button works.

**Depends on**: T002

- [ ] T003 Implement app.js with hash router

---

## Phase 2 – Bridge JS↔Rust

### T004 [P2] [US5] Create bridge/mod.rs dispatcher module
**File**: `crates/plix-client/src/ui_cef/bridge/mod.rs`

Create the bridge module structure:
- `pub mod messages;`
- `pub mod handlers;`
- `pub mod serialize;`
- `BridgeDispatcher` struct with message queue
- `dispatch(&mut self, json: &str) -> Option<String>` entry point
- Message routing based on `type` field

**Acceptance**: Module compiles, dispatcher routes test message to handler.

**Depends on**: T003

- [ ] T004 Create bridge/mod.rs dispatcher

---

### T005 [P2] [US5] Define bridge message types in messages.rs
**File**: `crates/plix-client/src/ui_cef/bridge/messages.rs`

Define serde types per contracts/bridge-messages.md:
- `BridgeRequest` { id, msg_type, payload }
- `BridgeResponse` { id, msg_type, ok, payload, error }
- `BridgeError` { code, message }
- `MessageType` enum: Handshake, GetConfig, SetConfig, FetchServers, ToggleFavorite, Connect, Quit
- `PushType` enum: ConnectionStatus, FavoritesUpdated

**Acceptance**: All types serialize/deserialize correctly with serde_json.

**Depends on**: T004

- [ ] T005 Define bridge message types

---

### T006 [P2] [US5] Implement JSON serialization utilities
**File**: `crates/plix-client/src/ui_cef/bridge/serialize.rs`

Implement serialization helpers:
- `parse_request(json: &str) -> Result<BridgeRequest, BridgeError>`
- `serialize_response(response: &BridgeResponse) -> String`
- `serialize_push<T: Serialize>(push_type: &str, payload: &T) -> String`
- Error handling for malformed JSON (EBRG002)

**Acceptance**: Round-trip serialization preserves all fields.

**Depends on**: T005

- [ ] T006 Implement JSON serialization utilities

---

### T007 [P2] [US5] Implement Handshake handler with version check
**File**: `crates/plix-client/src/ui_cef/bridge/handlers.rs`

Implement the handshake handler:
- `handle_handshake(id: &str, payload: Value) -> BridgeResponse`
- Extract `bridge_version` from payload
- Check compatibility (accept 1.x, reject 2.x+)
- Return supported version and display name on success
- Return EBRG001 error on version mismatch

**Acceptance**: Handshake with version "1.0" succeeds, version "2.0" returns error.

**Depends on**: T006

- [ ] T007 Implement Handshake handler

---

## Phase 3 – Main Menu

### T008 [P1] [US1] Create main menu page component
**File**: `assets/ui/pages/main.js`

Implement the main menu page:
- Export `render()` function returning HTML string
- Play button (disabled in v1, placeholder)
- Servers button → navigates to `#/servers`
- Settings button → navigates to `#/settings`
- Quit button → sends Quit message via bridge
- Display player name from handshake response

**Acceptance**: All buttons render, navigation works, Quit sends message.

**Depends on**: T003, T007

- [ ] T008 Create main menu page

---

### T009 [P1] [US1] Implement Quit handler in Rust
**File**: `crates/plix-client/src/ui_cef/bridge/handlers.rs`

Add Quit message handling:
- `handle_quit(id: &str) -> BridgeResponse`
- Set game exit flag via shared state
- Return success response
- Game loop checks flag and exits cleanly

**Acceptance**: Clicking Quit in UI exits game without errors.

**Depends on**: T008

- [ ] T009 Implement Quit handler

---

### T010 [P1] [US1] Wire main menu to bridge and test navigation
**File**: `assets/ui/app.js`, `crates/plix-client/src/ui_cef/bridge/mod.rs`

Complete main menu integration:
- Register `window.plix.send()` in CEF shell
- Register `window.plix.onMessage()` callback
- Main menu sends Handshake on load
- Display player name from response
- Verify all navigation paths work

**Acceptance**: Main menu displays player name, all buttons functional.

**Depends on**: T009

- [ ] T010 Wire main menu to bridge

---

## Phase 4 – Settings

### T011 [P1] [US2] Create menus/config.rs module
**File**: `crates/plix-client/src/ui_cef/menus/config.rs`

Create settings handler module:
- `GameConfigUI` struct matching data-model.md
- `KeybindMap` struct with all action bindings
- `to_ui_config(config: &GameConfig) -> GameConfigUI`
- `from_ui_config(ui: &GameConfigUI) -> Result<GameConfig, BridgeError>`

**Acceptance**: Conversion functions work correctly in both directions.

**Depends on**: T007

- [ ] T011 Create menus/config.rs module

---

### T012 [P1] [US2] Implement GetConfig handler
**File**: `crates/plix-client/src/ui_cef/menus/config.rs`

Implement GetConfig:
- `handle_get_config(id: &str, config: &GameConfig) -> BridgeResponse`
- Convert GameConfig to GameConfigUI
- Return as JSON payload
- Handle missing config gracefully

**Acceptance**: GetConfig returns current settings in correct format.

**Depends on**: T011

- [ ] T012 Implement GetConfig handler

---

### T013 [P1] [US2] Implement SetConfig handler with validation
**File**: `crates/plix-client/src/ui_cef/menus/config.rs`

Implement SetConfig:
- `handle_set_config(id: &str, payload: Value, config: &mut GameConfig) -> BridgeResponse`
- Parse GameConfigUI from payload
- Validate ranges: sensitivity 0.0001-0.01, FOV 60-110
- Check keybind conflicts
- Save to config file on success
- Return ECFG001 on validation failure, ECFG002 on save failure

**Acceptance**: Valid config saves, invalid config returns appropriate error.

**Depends on**: T012

- [ ] T013 Implement SetConfig handler

---

### T014 [P1] [US2] Create settings page UI
**File**: `assets/ui/pages/settings.js`

Implement settings page:
- Export `render()` and `init()` functions
- Send GetConfig on page load
- Display form with all settings (sensitivity slider, FOV slider, checkboxes, keybind buttons)
- Live preview for sensitivity/FOV changes
- Save button sends SetConfig
- Cancel button returns to main menu
- Error display area for validation messages

**Acceptance**: Settings display correctly, changes save and persist.

**Depends on**: T013

- [ ] T014 Create settings page UI

---

### T015 [P1] [US2] Implement keybind editing UI
**File**: `assets/ui/pages/settings.js`, `assets/ui/components/keybind.js`

Implement keybind editor:
- Create keybind.js component for individual binding
- Click to edit mode, press key to capture
- Display current binding (e.g., "W", "Space", "LMB")
- Conflict detection with swap confirmation dialog
- Support keyboard keys and mouse buttons
- ESC cancels edit mode

**Acceptance**: Keybinds can be changed, conflicts detected and resolved.

**Depends on**: T014

- [ ] T015 Implement keybind editing UI

---

## Phase 5 – Server Browser Data

### T016 [P1] [US3] Create menus/servers.rs module
**File**: `crates/plix-client/src/ui_cef/menus/servers.rs`

Create server browser handler module:
- `ServerEntryUI` struct matching data-model.md
- `to_server_entry(server: &ServerInfo, favorites: &[String]) -> ServerEntryUI`
- Protocol version compatibility check
- Placeholder for master server fetch integration

**Acceptance**: ServerEntryUI correctly maps from ServerInfo.

**Depends on**: T007

- [ ] T016 Create menus/servers.rs module

---

### T017 [P1] [US3] Implement FetchServers handler
**File**: `crates/plix-client/src/ui_cef/menus/servers.rs`

Implement FetchServers:
- `handle_fetch_servers(id: &str, server_browser: &ServerBrowser, favorites: &[String]) -> BridgeResponse`
- Call existing server browser fetch API (Feature 026)
- Convert results to ServerEntryUI list
- Mark favorites based on address match
- Return ESRV001 on master unreachable, ESRV002 on empty list

**Acceptance**: FetchServers returns server list from master server.

**Depends on**: T016

- [ ] T017 Implement FetchServers handler

---

### T018 [P1] [US3] Implement Connect handler
**File**: `crates/plix-client/src/ui_cef/menus/servers.rs`

Implement Connect:
- `handle_connect(id: &str, payload: Value) -> BridgeResponse`
- Extract address from payload
- Validate address format
- Initiate connection via existing connect API (Feature 026)
- Return immediate success (connection progress via push events)
- Implement ConnectionStatus push event sending

**Acceptance**: Connect initiates connection, status updates sent via push.

**Depends on**: T017

- [ ] T018 Implement Connect handler

---

## Phase 6 – Server Browser UI

### T019 [P1] [US3] Create server browser page structure
**File**: `assets/ui/pages/servers.js`

Implement server browser page skeleton:
- Export `render()` and `init()` functions
- Search input field (max 32 chars)
- Filter controls (region, mode tags)
- Server list container
- Refresh button
- Back button → main menu
- Loading and error state areas

**Acceptance**: Page structure renders, navigation works.

**Depends on**: T018, T027

- [ ] T019 Create server browser page structure

---

### T020 [P1] [US3] Implement server list component
**File**: `assets/ui/components/list.js`

Create reusable server list component:
- `ServerList` class with `render(servers)` method
- Row template: name, region, players/max, tags, favorite toggle, connect button
- Click row to select (highlight)
- Double-click to connect
- Sort by name, players, or region
- Empty state message

**Acceptance**: Server list displays correctly, interactions work.

**Depends on**: T019

- [ ] T020 Implement server list component

---

### T021 [P1] [US3] Implement client-side search and filter
**File**: `assets/ui/pages/servers.js`

Implement filtering:
- 300ms debounce on search input
- Filter by name (case-insensitive substring)
- Filter by region (dropdown)
- Filter by tags (multi-select)
- Show/hide incompatible servers toggle
- Show favorites only toggle
- Filter count indicator

**Acceptance**: Filters work correctly, debounce prevents spam.

**Depends on**: T020

- [ ] T021 Implement search and filter

---

### T022 [P1] [US3] Implement refresh flow with loading state
**File**: `assets/ui/pages/servers.js`

Implement refresh:
- Refresh button sends FetchServers
- Disable button during fetch (prevent double-click)
- Show loading spinner
- Update list on response
- Show error message on failure
- Auto-refresh on page open (optional)

**Acceptance**: Refresh fetches new data, loading states display correctly.

**Depends on**: T021

- [ ] T022 Implement refresh flow

---

### T023 [P1] [US3] Implement connect flow with status display
**File**: `assets/ui/pages/servers.js`, `assets/ui/components/modal.js`

Implement connection:
- Connect button sends Connect message
- Show connecting modal with server name
- Handle ConnectionStatus push events
- Display progress: "Connecting...", "Connected!", "Failed: reason"
- Cancel button during connection attempt
- Close modal on success (game will switch to playing state)

**Acceptance**: Connection flow shows status, handles success and failure.

**Depends on**: T022

- [ ] T023 Implement connect flow

---

### T024 [P1] [US3] Display protocol compatibility indicators
**File**: `assets/ui/components/list.js`

Implement compatibility display:
- Show warning icon for incompatible servers
- Tooltip explaining version mismatch
- Dim incompatible servers in list
- Block connect with error for incompatible servers
- Show "Compatible" badge for matching versions

**Acceptance**: Incompatible servers clearly indicated, connect blocked.

**Depends on**: T023

- [ ] T024 Display protocol compatibility

---

## Phase 7 – Favorites

### T025 [P1] [US3] Create menus/favorites.rs persistence module
**File**: `crates/plix-client/src/ui_cef/menus/favorites.rs`

Create favorites persistence:
- `Favorites` struct: version, favorites vec
- `load_favorites() -> Favorites` from ~/.config/plix/favorites.toml
- `save_favorites(favorites: &Favorites) -> Result<(), BridgeError>`
- Handle missing file (create empty)
- Handle corrupt file (reset with warning, log error)
- File format version = 1

**Acceptance**: Favorites load and save correctly, corruption handled.

**Depends on**: T007

- [ ] T025 Create favorites.rs persistence

---

### T026 [P1] [US3] Implement ToggleFavorite handler
**File**: `crates/plix-client/src/ui_cef/menus/favorites.rs`

Implement ToggleFavorite:
- `handle_toggle_favorite(id: &str, payload: Value, favorites: &mut Favorites) -> BridgeResponse`
- Extract address from payload
- Add if not present, remove if present
- Save to file
- Return new is_favorite status
- Send FavoritesUpdated push event

**Acceptance**: Toggle adds/removes favorites, persists to file.

**Depends on**: T025

- [ ] T026 Implement ToggleFavorite handler

---

### T027 [P1] [US3] Wire favorites to server list UI
**File**: `assets/ui/pages/servers.js`, `assets/ui/components/list.js`

Integrate favorites:
- Favorite toggle button in server row (star icon)
- Click sends ToggleFavorite message
- Update UI optimistically
- Handle FavoritesUpdated push (sync if changed externally)
- Sort favorites to top option
- "Favorites only" filter

**Acceptance**: Favorites toggle works, persists across sessions.

**Depends on**: T026

- [ ] T027 Wire favorites to server list

---

## Phase 8 – Input/Focus

### T028 [P2] [US4] Implement ESC key handling for menu navigation
**File**: `assets/ui/app.js`, `crates/plix-client/src/ui_cef/input.rs`

Implement ESC handling:
- ESC on sub-page → navigate back to main menu
- ESC on main menu → close CEF overlay (return to game if in-game)
- Prevent ESC from propagating to game when in menu
- Handle ESC during keybind edit (cancel edit, not navigate)

**Acceptance**: ESC navigates back correctly in all contexts.

**Depends on**: T010

- [ ] T028 Implement ESC key handling

---

### T029 [P2] [US4] Verify input focus blocks game controls
**File**: `crates/plix-client/src/ui_cef/input.rs`

Verify and fix input isolation:
- When CEF has focus, game input is blocked
- Typing in search field doesn't move character
- Mouse clicks on UI don't fire weapons
- Focus returns to game when menu closes
- Test with all input types (keyboard, mouse, scroll)

**Acceptance**: UI interaction doesn't trigger game actions.

**Depends on**: T028

- [ ] T029 Verify input focus isolation

---

## Phase 9 – Security & Robustness

### T030 [P2] [US5] Implement string sanitization and length limits
**File**: `crates/plix-client/src/ui_cef/bridge/serialize.rs`

Implement sanitization:
- `sanitize_string(s: &str, max_len: usize) -> String`
- Truncate to max length
- Remove control characters
- Escape HTML entities for display
- Apply limits: server name 64, search 32, display name 32
- Apply to all strings in responses

**Acceptance**: Oversized strings truncated, control chars removed.

**Depends on**: T007

- [ ] T030 Implement string sanitization

---

### T031 [P2] [US5] Block external network requests from JS
**File**: `crates/plix-client/src/ui_cef/config.rs`

Configure CEF to block external requests:
- Verify CEF request handler blocks non-local URLs
- Log blocked request attempts
- Allow only file:// and local asset URLs
- Block fetch(), XMLHttpRequest to external domains
- Test with attempted external fetch

**Acceptance**: External network requests fail with logged warning.

**Depends on**: T030

- [ ] T031 Block external network requests

---

### T032 [P2] [US5] Handle malformed messages gracefully
**File**: `crates/plix-client/src/ui_cef/bridge/mod.rs`

Implement robust error handling:
- Catch JSON parse errors → EBRG002
- Catch unknown message types → EBRG003
- Catch missing required fields → EBRG002
- Catch payload type mismatches → EBRG002
- Never crash on malformed input
- Log errors with details for debugging

**Acceptance**: Any malformed message returns error, no crashes.

**Depends on**: T031

- [ ] T032 Handle malformed messages

---

## Phase 10 – Tests

### T033 [P2] [US5] Write bridge message routing tests
**File**: `crates/plix-client/tests/ui_cef/bridge_test.rs`

Test bridge dispatcher:
- Test Handshake routing
- Test GetConfig routing
- Test SetConfig routing
- Test FetchServers routing
- Test unknown type returns EBRG003
- Test malformed JSON returns EBRG002

**Acceptance**: All routing tests pass.

**Depends on**: T032

- [ ] T033 Write bridge routing tests

---

### T034 [P2] [US2] Write config handler tests
**File**: `crates/plix-client/tests/ui_cef/config_test.rs`

Test config handlers:
- Test GetConfig returns valid JSON
- Test SetConfig with valid values
- Test SetConfig with invalid sensitivity
- Test SetConfig with invalid FOV
- Test keybind conflict detection

**Acceptance**: All config tests pass.

**Depends on**: T033

- [ ] T034 Write config handler tests

---

### T035 [P2] [US3] Write favorites persistence tests
**File**: `crates/plix-client/tests/ui_cef/favorites_test.rs`

Test favorites:
- Test load empty favorites
- Test save and reload favorites
- Test toggle add
- Test toggle remove
- Test corrupt file recovery
- Test max favorites limit (if applicable)

**Acceptance**: All favorites tests pass.

**Depends on**: T034

- [ ] T035 Write favorites tests

---

### T036 [P2] [US3] Write server browser handler tests
**File**: `crates/plix-client/tests/ui_cef/servers_test.rs`

Test server handlers:
- Test FetchServers success path
- Test FetchServers with empty list
- Test FetchServers with master unreachable
- Test Connect with valid address
- Test Connect with invalid address
- Test protocol compatibility marking

**Acceptance**: All server browser tests pass.

**Depends on**: T035

- [ ] T036 Write server browser tests

---

### T037 [P2] Manual UI test checklist
**File**: `specs/031-cef-menus/test-checklist.md`

Document manual test procedures:
- Main menu navigation test
- Settings modify and save test
- Settings persist across restart test
- Server browser refresh test
- Server browser filter test
- Favorites toggle and persist test
- Connect success and failure test
- Input focus isolation test
- ESC navigation test
- CEF fallback test (disable CEF, verify native UI)

**Acceptance**: Checklist documented, all items pass manual testing.

**Depends on**: T036

- [ ] T037 Create manual test checklist

---

## Phase 11 – Polish

### T038 [P1] Apply consistent styling across all pages
**File**: `assets/ui/styles.css`

Finalize UI styling:
- Consistent color scheme (match game aesthetic)
- Button hover/active states
- Input field styling
- Loading spinner animation
- Error message styling (red background)
- Success message styling (green)
- Responsive layout within expected window sizes
- Font consistency

**Acceptance**: All pages have consistent, polished appearance.

**Depends on**: T037

- [ ] T038 Apply consistent styling

---

### T039 [P1] Implement button debounce and loading states
**File**: `assets/ui/components/button.js`

Create button component:
- Debounce click events (500ms)
- Disable during async operation
- Show loading spinner inside button
- Re-enable on operation complete
- Visual feedback for disabled state
- Apply to Refresh, Connect, Save buttons

**Acceptance**: No double-clicks, clear loading feedback.

**Depends on**: T038

- [ ] T039 Implement button debounce

---

## Phase 12 – Documentation

### T040 Update quickstart.md with final implementation details
**File**: `specs/031-cef-menus/quickstart.md`

Update developer documentation:
- Verify all file paths are correct
- Add any new message types
- Update debugging tips based on development
- Add common issues and solutions
- Update build commands if changed

**Acceptance**: Quickstart reflects actual implementation.

**Depends on**: T039

- [ ] T040 Update quickstart.md

---

### T041 Document all error codes in contracts/bridge-messages.md
**File**: `specs/031-cef-menus/contracts/bridge-messages.md`

Verify error code documentation:
- All implemented error codes documented
- Each code has clear description
- Examples of when each error occurs
- Error message format consistent
- Add any new codes discovered during implementation

**Acceptance**: All error codes documented with examples.

**Depends on**: T040

- [ ] T041 Document error codes

---

### T042 Final review and cleanup
**File**: Multiple

Final review tasks:
- Remove any TODO comments
- Remove debug logging (or gate behind feature flag)
- Verify all tests pass
- Verify clippy passes with no warnings
- Verify cargo fmt applied
- Update CLAUDE.md if needed
- Create PR description summarizing changes

**Acceptance**: Code is clean, all checks pass, ready for merge.

**Depends on**: T041

- [ ] T042 Final review and cleanup

---

## Checklist Summary

### Phase 1 – UI Assets & App Shell
- [x] T001 [P1] [US1] Create assets/ui/ directory structure
- [x] T002 [P1] [US1] Implement index.html app shell
- [x] T003 [P1] [US1] Implement app.js with hash router

### Phase 2 – Bridge JS↔Rust
- [x] T004 [P2] [US5] Create bridge/mod.rs dispatcher
- [x] T005 [P2] [US5] Define bridge message types
- [x] T006 [P2] [US5] Implement JSON serialization utilities
- [x] T007 [P2] [US5] Implement Handshake handler

### Phase 3 – Main Menu
- [x] T008 [P1] [US1] Create main menu page
- [x] T009 [P1] [US1] Implement Quit handler
- [x] T010 [P1] [US1] Wire main menu to bridge

### Phase 4 – Settings
- [x] T011 [P1] [US2] Create menus/config.rs module
- [x] T012 [P1] [US2] Implement GetConfig handler
- [x] T013 [P1] [US2] Implement SetConfig handler
- [x] T014 [P1] [US2] Create settings page UI
- [x] T015 [P1] [US2] Implement keybind editing UI

### Phase 5 – Server Browser Data
- [x] T016 [P1] [US3] Create menus/servers.rs module
- [x] T017 [P1] [US3] Implement FetchServers handler
- [x] T018 [P1] [US3] Implement Connect handler

### Phase 6 – Server Browser UI
- [x] T019 [P1] [US3] Create server browser page structure
- [x] T020 [P1] [US3] Implement server list component
- [x] T021 [P1] [US3] Implement search and filter
- [x] T022 [P1] [US3] Implement refresh flow
- [x] T023 [P1] [US3] Implement connect flow
- [x] T024 [P1] [US3] Display protocol compatibility

### Phase 7 – Favorites
- [x] T025 [P1] [US3] Create favorites.rs persistence
- [x] T026 [P1] [US3] Implement ToggleFavorite handler
- [x] T027 [P1] [US3] Wire favorites to server list

### Phase 8 – Input/Focus
- [x] T028 [P2] [US4] Implement ESC key handling
- [x] T029 [P2] [US4] Verify input focus isolation

### Phase 9 – Security & Robustness
- [x] T030 [P2] [US5] Implement string sanitization
- [x] T031 [P2] [US5] Block external network requests
- [x] T032 [P2] [US5] Handle malformed messages

### Phase 10 – Tests
- [x] T033 [P2] [US5] Write bridge routing tests
- [x] T034 [P2] [US2] Write config handler tests
- [x] T035 [P2] [US3] Write favorites tests
- [x] T036 [P2] [US3] Write server browser tests
- [x] T037 [P2] Manual test checklist

### Phase 11 – Polish
- [x] T038 [P1] Apply consistent styling
- [x] T039 [P1] Implement button debounce

### Phase 12 – Documentation
- [x] T040 Update quickstart.md
- [x] T041 Document error codes
- [x] T042 Final review and cleanup
