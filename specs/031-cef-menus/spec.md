# Feature Specification: CEF Menus (Main Menu / Settings / Server Browser)

**Feature Branch**: `031-cef-menus`
**Created**: 2025-12-18
**Status**: Draft
**Input**: User description: "Implement the main menu, settings, and server browser UI as HTML/CSS/JS rendered via the optional CEF shell"

## Overview

This feature implements the main menu, settings, and server browser UI as HTML/CSS/JS rendered via the optional CEF shell (Feature 030). The UI supports navigation, configuration editing, server list refresh/filter, favorites management, and connect flow, while preserving correct input focus/capture and providing a native UI fallback when CEF is unavailable.

**Dependencies**:
- Feature 030 - CEF UI Shell (optional CEF OSR + input focus + texture render)
- Feature 026 - Server Browser v1 (master list fetch + connect)
- Feature 025 - Account Identity (display name/profile)

**Key Constraint**: This feature must be optional. If CEF is disabled or unavailable, the native UI (Feature 005) remains the fallback.

## Clarifications

### Session 2025-12-18

- Q: Which settings are exposed in the CEF settings UI? → A: Match native UI settings exactly (sensitivity, FOV, fullscreen, audio mute, keybinds)
- Q: What are the string length limits for sanitization? → A: Standard limits (server name 64 chars, search 32 chars, display name 32 chars)

## User Scenarios & Testing

### User Story 1 - Main Menu in CEF (Priority: P1)

As a player, I can use an HTML main menu to start playing, open server browser, open settings, or quit the game.

**Why this priority**: The main menu is the entry point to the game. Without it, players cannot access any other functionality. It establishes the CEF UI foundation that all other screens depend on.

**Independent Test**: Launch the game with CEF enabled and verify the main menu renders with all navigation buttons functional. Each button should navigate to its target screen or exit the game cleanly.

**Acceptance Scenarios**:

1. **Given** CEF is enabled and initialized successfully, **When** the game starts, **Then** the HTML main menu displays with Play, Servers, Settings, and Quit buttons
2. **Given** the main menu is displayed, **When** the player clicks "Servers", **Then** the server browser screen opens
3. **Given** the main menu is displayed, **When** the player clicks "Settings", **Then** the settings screen opens
4. **Given** the main menu is displayed, **When** the player clicks "Quit", **Then** the client exits cleanly without errors
5. **Given** CEF is disabled or unavailable, **When** the game starts, **Then** the native UI main menu (Feature 005) displays instead

---

### User Story 2 - Settings in CEF (Priority: P1)

As a player, I can view and change settings in an HTML UI and save them persistently. The settings exposed match the native UI exactly: sensitivity, FOV, fullscreen, audio mute, and keybinds.

**Why this priority**: Settings are essential for players to customize their experience. This story validates the bidirectional data flow between the HTML UI and the game engine.

**Independent Test**: Open settings, modify values, save, restart the game, and verify settings persisted correctly.

**Acceptance Scenarios**:

1. **Given** the settings page is opened, **When** it loads, **Then** current configuration values are displayed accurately
2. **Given** the player has modified a setting value, **When** they click Save/Apply, **Then** the configuration is validated and persisted
3. **Given** the player enters an invalid value (e.g., sensitivity outside allowed range), **When** they click Save, **Then** an error message is displayed and the invalid value is not saved
4. **Given** settings have been saved, **When** the game is restarted, **Then** the saved settings are loaded and applied

---

### User Story 3 - Server Browser in CEF (Priority: P1)

As a player, I can view servers from the master list, search/filter them, manage favorites, and connect to a server.

**Why this priority**: Server browsing is the primary way players find and join games. It demonstrates complex UI interactions and data flow from external sources.

**Independent Test**: Open server browser, refresh the list, apply filters, favorite a server, and connect to verify the complete flow works.

**Acceptance Scenarios**:

1. **Given** the server browser is opened, **When** the player clicks Refresh, **Then** the server list is fetched from the master server and displayed
2. **Given** the server list is displayed, **When** the player types in the search field, **Then** servers are filtered by name, tags, or region matching the search term
3. **Given** a server is displayed in the list, **When** the player clicks the favorite toggle, **Then** the server is added/removed from favorites and the change persists across sessions
4. **Given** a server is selected, **When** the player clicks Connect, **Then** a connection attempt is made and status/progress is displayed
5. **Given** the master server is unreachable, **When** refresh is attempted, **Then** a clear error message is displayed

---

### User Story 4 - Input Focus and Capture (Priority: P2)

As a player, when the CEF menus are open and focused, gameplay input is blocked so I can interact with the UI without unintended game actions.

**Why this priority**: Correct input handling prevents frustrating user experience issues like accidental movements while typing. This builds on the input focus system from Feature 030.

**Independent Test**: Open a menu, type in a text field, press movement keys, and verify no game movement occurs while the UI has focus.

**Acceptance Scenarios**:

1. **Given** a CEF menu is open and focused, **When** the player types on the keyboard, **Then** input goes to the UI and not to game controls
2. **Given** a CEF menu is open, **When** the player presses ESC, **Then** the menu closes or navigates back one level
3. **Given** a CEF menu is closed, **When** the player moves, **Then** game movement resumes normally

---

### User Story 5 - Data Flow and Safety (Priority: P2)

As a developer, I want a safe, minimal, versioned bridge between the HTML/JS UI and the game engine so that the UI cannot cause security issues or break the game.

**Why this priority**: Security and stability are critical for a production system. This ensures the architecture is robust before adding more features.

**Independent Test**: Verify all UI actions go through typed messages, external network access is blocked from JS, and malformed inputs are handled gracefully.

**Acceptance Scenarios**:

1. **Given** the UI needs game data, **When** it makes a request, **Then** the request goes through the typed message bridge (not direct function calls)
2. **Given** JavaScript code in the UI, **When** it attempts to fetch external URLs, **Then** the request is blocked
3. **Given** a UI message contains invalid or oversized data, **When** it is received by the game, **Then** an error is returned without crashing
4. **Given** the UI sends a message, **When** an error occurs, **Then** the error is returned with an explicit code and user-friendly message

---

### Edge Cases

- What happens when the master server returns an empty server list? (Display "No servers found" message)
- What happens when server list contains servers with incompatible protocol versions? (Display version mismatch indicator, allow viewing but warn on connect)
- What happens when connection to a server times out? (Display timeout error with retry option)
- What happens when favorites file is corrupted? (Reset to empty favorites with warning)
- What happens when the player rapidly clicks buttons? (Debounce actions, prevent duplicate requests)
- What happens when CEF crashes mid-session? (Trigger fallback to native UI with notification)

## Requirements

### Functional Requirements

- **FR-001**: System MUST display HTML menus only when CEF is enabled and initialized successfully
- **FR-002**: System MUST load UI assets from local package (assets/ui/*) only, not from external sources
- **FR-003**: System MUST provide a bidirectional message bridge between JavaScript and the game engine supporting request/response and push events
- **FR-004**: System MUST fetch server list through the game's networking layer (not JavaScript fetch)
- **FR-005**: System MUST use existing configuration persistence and validation for settings
- **FR-006**: System MUST persist favorites locally (in the same directory as profile/config) and share them with the native server browser if applicable
- **FR-007**: System MUST route connect requests from the CEF UI through the same connection path as the native UI
- **FR-008**: System MUST display loading indicators during async operations (server list fetch, connect attempt)
- **FR-009**: System MUST display clear error messages for: master unreachable, empty server list, version incompatible, connection timeout/failure
- **FR-010**: System MUST support ESC key to close menus or navigate back
- **FR-011**: System MUST continue to work with native UI when CEF is disabled (no regressions)
- **FR-012**: System MUST block external network requests from JavaScript code
- **FR-013**: System MUST sanitize and length-limit all strings before display (server name: 64 chars, search query: 32 chars, display name: 32 chars)
- **FR-014**: System MUST return errors with explicit codes and user-friendly messages

### Non-Functional Requirements

- **NFR-001**: UI operations MUST NOT block the render loop (bridge calls are async/budgeted)
- **NFR-002**: Server list data MUST only be pushed on refresh or when changed (no per-frame JSON updates)
- **NFR-003**: UI code MUST be structured (components/pages) and documented for maintainability

### Key Entities

- **BridgeMessage**: Represents a message between JS and the game engine; contains id (for request/response correlation), type (message type), payload (data), optional error
- **ServerEntry**: Represents a server in the browser list; contains name, address, region, tags, player count, max players, protocol version, favorite status
- **GameConfig**: Existing configuration entity extended with UI-relevant settings (sensitivity, FOV, fullscreen, audio, keybinds)
- **Favorites**: Collection of favorited server addresses persisted locally

## Success Criteria

### Measurable Outcomes

- **SC-001**: Players can navigate from main menu to any screen (servers, settings) and back in under 2 seconds per transition
- **SC-002**: Server list refresh completes and displays results within 3 seconds on standard network conditions
- **SC-003**: Settings changes are persisted and verified on restart 100% of the time (no data loss)
- **SC-004**: Favorites persist across game restarts 100% of the time
- **SC-005**: Native UI fallback activates automatically when CEF is unavailable, with no manual intervention required
- **SC-006**: All error conditions display user-friendly messages (no raw technical errors shown to players)
- **SC-007**: UI interactions do not cause frame drops below 60fps during normal operation

## Out of Scope (v1)

- Remote web browsing (all content is local)
- External network requests from JavaScript
- Complex animations or performance-intensive effects
- Account login/authentication UI
- In-game HUD conversion to CEF
- Full modding/theming system

## Assumptions

- Feature 030 (CEF UI Shell) provides functional texture rendering, input routing, and fallback detection
- Feature 026 (Server Browser v1) provides the master server fetch API and connection logic
- Feature 025 (Account Identity) provides display name and profile loading
- The existing configuration system (GameConfig) is extensible for UI settings
- Players have standard keyboard and mouse input devices
