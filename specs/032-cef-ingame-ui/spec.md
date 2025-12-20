# Feature Specification: CEF In-Game UI (HUD, Chat, Scoreboard)

**Feature Branch**: `032-cef-ingame-ui`
**Created**: 2025-12-18
**Status**: Draft
**Input**: User description: "Add in-game UI rendered via CEF (HTML/CSS/JS) as overlay texture, with optional web HUD, in-game chat, and hold-to-show scoreboard"

## Overview

This feature implements in-game UI elements rendered via CEF as transparent overlays during gameplay:
- **Web HUD** (optional): Displays HP, ping/RTT, FPS, and crosshair (replaces native crosshair when enabled)
- **Chat**: Full text chat with input, history, system messages, and basic commands
- **Scoreboard**: Hold-to-show player list with stats (ping, score, kills, deaths, team)

The feature maintains input focus separation (chat captures keyboard, scoreboard doesn't), provides native UI fallback when CEF is unavailable, and follows the server-authoritative model (UI is never source of truth).

**Dependencies**:
- Feature 030 - CEF UI Shell (OSR texture rendering, input routing, fallback detection)
- Feature 031 - CEF Menus (bridge protocol foundation, message typing)
- Feature 005 - Minimal UI Native (fallback HUD, crosshair)

**Key Constraints**:
- CEF overlay must not impact gameplay framerate (limit update frequency)
- Input handling must be robust (no "stuck focus" or "input leak" scenarios)
- All chat content validated both client and server side (size limits, rate limits)
- No dynamic JS execution from server data (no eval, no remote code)

## Clarifications

### Session 2025-12-18

- Q: How should incoming chat messages be notified when chat is closed? → A: Show a small toast notification (last message preview) that auto-fades
- Q: Should web HUD render the crosshair when CEF HUD is enabled? → A: Yes, web HUD renders crosshair (replaces native crosshair when CEF enabled)

## User Scenarios & Testing

### User Story 1 - HUD Display (Priority: P1)

As a player, I can see my health, ping, and optionally FPS overlaid on the gameplay screen without opening any menu.

**Why this priority**: The HUD provides essential real-time information players need during gameplay. Without HP visibility, players cannot make tactical decisions. This is the most fundamental in-game UI element.

**Independent Test**: Join a game with CEF HUD enabled, verify HP bar updates when taking damage, verify ping value reflects actual connection latency, verify overlay doesn't obstruct gameplay.

**Acceptance Scenarios**:

1. **Given** CEF HUD is enabled in settings, **When** the player is in-game, **Then** the HUD overlay displays HP (value + bar), ping/RTT, and optionally FPS
2. **Given** the player takes damage, **When** their HP decreases, **Then** the HUD updates within 100ms to show the new HP value
3. **Given** network conditions change, **When** latency increases, **Then** the ping display updates to reflect current RTT
4. **Given** CEF HUD is disabled in settings, **When** the player is in-game, **Then** the native HUD (Feature 005) displays instead
5. **Given** CEF HUD is enabled, **When** the overlay renders, **Then** gameplay behind the HUD elements remains visible (transparency)

---

### User Story 2 - Chat Communication (Priority: P1)

As a player, I can open a chat window, type messages, send them to other players, and see messages from others in chronological order.

**Why this priority**: Chat is essential for multiplayer coordination and social interaction. It enables team communication and is a core multiplayer feature expected in any online game.

**Independent Test**: Open chat, type a message, send it, verify it appears in your history, have another player send a message, verify it appears in your history.

**Acceptance Scenarios**:

1. **Given** the player is in-game, **When** they press Enter, **Then** the chat input opens and keyboard focus moves to the text field
2. **Given** chat is open, **When** the player types a message and presses Enter, **Then** the message is sent to the server and appears in chat history
3. **Given** chat is open, **When** the player presses Escape, **Then** the chat closes without sending and gameplay input resumes
4. **Given** another player sends a message, **When** the server broadcasts it, **Then** the message appears in the local chat history with author and timestamp
5. **Given** a system event occurs (player join/leave, server message), **When** the event is broadcast, **Then** a system message appears in chat history
6. **Given** chat is closed, **When** a new message arrives, **Then** a brief notification appears (fade in/out) without opening the chat

---

### User Story 3 - Chat Input Focus (Priority: P1)

As a player, when typing in chat, my keyboard input goes only to the chat and not to game controls, preventing accidental movement or actions.

**Why this priority**: Input isolation is critical for usability. Without it, typing "wasd" would move the player, making chat unusable. This is a blocking issue for chat functionality.

**Independent Test**: Open chat, type movement keys (WASD), verify player does not move. Close chat, press movement keys, verify player moves normally.

**Acceptance Scenarios**:

1. **Given** chat is open, **When** the player presses W/A/S/D, **Then** those characters appear in the text field and no player movement occurs
2. **Given** chat is open, **When** the player clicks outside the chat area, **Then** chat closes and input returns to gameplay
3. **Given** chat was just closed, **When** the very next frame processes input, **Then** gameplay input is active (no frame delay)
4. **Given** the game loses window focus (alt-tab) while chat is open, **When** focus returns, **Then** chat state is preserved correctly (no stuck focus)

---

### User Story 4 - Chat Commands (Priority: P2)

As a player, I can type commands in chat to get help or clear my local history.

**Why this priority**: Basic commands improve usability but are not essential for core communication. Players can use chat without commands.

**Independent Test**: Type "/help" and verify help text appears locally. Type "/clear" and verify chat history is cleared.

**Acceptance Scenarios**:

1. **Given** chat is open, **When** the player types "/help" and presses Enter, **Then** a help message appears locally listing available commands
2. **Given** chat history has messages, **When** the player types "/clear" and presses Enter, **Then** the local chat history is cleared
3. **Given** the player types an unknown command (e.g., "/foo"), **When** they press Enter, **Then** the command is forwarded to the server (server decides how to handle)

---

### User Story 5 - Scoreboard Display (Priority: P1)

As a player, I can hold a key (TAB) to see a list of all players in the match with their ping and stats.

**Why this priority**: Scoreboard is essential for competitive awareness. Players need to know who is in the match, team compositions, and current standings. It's a core feature of multiplayer games.

**Independent Test**: Hold TAB, verify scoreboard appears with player list. Release TAB, verify scoreboard hides. Verify player data (names, ping) is accurate.

**Acceptance Scenarios**:

1. **Given** the player is in-game, **When** they press and hold TAB, **Then** the scoreboard overlay appears
2. **Given** the scoreboard is visible, **When** the player releases TAB, **Then** the scoreboard hides
3. **Given** the scoreboard is visible, **When** other players join or leave, **Then** the scoreboard updates to reflect current players
4. **Given** a team-based game mode, **When** the scoreboard displays, **Then** players are grouped by team
5. **Given** an FFA game mode, **When** the scoreboard displays, **Then** players are shown in a single list (sorted by score if available)
6. **Given** the scoreboard is visible, **When** the player presses movement keys, **Then** player movement still works (scoreboard doesn't capture input)

---

### User Story 6 - Native UI Fallback (Priority: P2)

As a player with CEF disabled, I can still use chat and scoreboard through a simplified native UI.

**Why this priority**: Ensures the game remains playable without CEF. Not all systems support CEF, and players should have a complete experience regardless.

**Independent Test**: Disable CEF, join a game, verify native chat opens on Enter, verify native scoreboard appears on TAB hold.

**Acceptance Scenarios**:

1. **Given** CEF is disabled/unavailable, **When** the player presses Enter, **Then** a native text input opens for chat
2. **Given** CEF is disabled, **When** the player holds TAB, **Then** a native scoreboard displays (simpler but functional)
3. **Given** CEF is disabled, **When** the game starts, **Then** the native HUD (HP + crosshair) displays automatically
4. **Given** CEF becomes unavailable mid-session (crash), **When** detected, **Then** the system falls back to native UI with a brief notification

---

### Edge Cases

- What happens when chat message exceeds 200 characters? (Truncate input, prevent sending oversized messages)
- What happens when chat history exceeds 100 messages? (Remove oldest messages to maintain limit)
- What happens when player sends messages too fast? (Client-side rate limit 1 msg/500ms, server enforces anti-spam)
- What happens when scoreboard has more than 64 players? (Cap display at 64, no pagination in v1)
- What happens when player name contains special characters? (Sanitize for display, escape HTML entities)
- What happens when network disconnects while chat is open? (Queue messages locally, send on reconnect or discard with notification)
- What happens when HUD update rate exceeds limit? (Throttle to 10-20 Hz, update only on change)
- What happens when TAB is remapped to another action? (Use configured key from keybinds, not hardcoded TAB)

## Requirements

### Functional Requirements

#### HUD

- **FR-001**: System MUST display web HUD overlay when CEF is enabled and ui.cef_hud setting is true
- **FR-002**: System MUST display HP as both numeric value and visual bar
- **FR-003**: System MUST display current ping/RTT in milliseconds
- **FR-004**: System SHOULD display FPS (optional, configurable)
- **FR-005**: System MUST update HUD data at 10-20 Hz maximum, or on value change
- **FR-006**: System MUST render HUD with transparency (alpha channel preserved)
- **FR-006b**: System MUST render crosshair in web HUD, replacing native crosshair when CEF HUD is enabled

#### Chat

- **FR-007**: System MUST open chat input when configured key is pressed (default: Enter)
- **FR-008**: System MUST close chat when Escape is pressed without sending
- **FR-009**: System MUST send message when Enter is pressed with non-empty input
- **FR-010**: System MUST support text selection, cursor movement, and basic clipboard (copy/paste)
- **FR-011**: System MUST limit message length to 200 characters
- **FR-012**: System MUST enforce client-side rate limit of 1 message per 500ms
- **FR-013**: System MUST send messages to server for broadcast (server is authoritative for distribution)
- **FR-014**: System MUST display received messages with author, text, kind (player/system), and timestamp
- **FR-015**: System MUST maintain local scrollable history of last 100 messages
- **FR-016**: System MUST display system messages (player join/leave, server announcements)
- **FR-017**: System MUST implement "/help" command showing available commands locally
- **FR-018**: System MUST implement "/clear" command clearing local history
- **FR-019**: System MUST forward unrecognized commands to server
- **FR-019b**: System MUST display incoming messages as auto-fading toast notifications (showing message preview) when chat is closed

#### Scoreboard

- **FR-020**: System MUST display scoreboard while configured key is held (default: TAB)
- **FR-021**: System MUST hide scoreboard when key is released
- **FR-022**: System MUST display server name (or "Match" if unavailable)
- **FR-023**: System MUST display player list with: name, ping
- **FR-024**: System MUST display score, kills, deaths when provided by server
- **FR-025**: System MUST group players by team when team data is available
- **FR-026**: System MUST NOT capture gameplay input while scoreboard is visible
- **FR-027**: System MUST update scoreboard data only when visible (no updates when hidden)

#### Bridge Protocol

- **FR-028**: System MUST use typed, versioned messages between JS and Rust (same philosophy as Feature 031)
- **FR-029**: System MUST support UI→Game messages: ui.ready, chat.send, chat.open, chat.close
- **FR-030**: System MUST support Game→UI messages: hud.state, chat.message, scoreboard.state, ui.config
- **FR-031**: System MUST validate message payloads (size limits, required fields)
- **FR-032**: System MUST log errors without crashing when UI is not ready or messages are malformed
- **FR-033**: System MUST NOT execute dynamic JavaScript from server data (no eval, no Function constructor)

#### Input Focus

- **FR-034**: System MUST block gameplay input (movement, actions) when chat is open
- **FR-035**: System MUST restore gameplay input immediately when chat closes
- **FR-036**: System MUST handle focus correctly across alt-tab and window focus changes
- **FR-037**: System MUST prevent "stuck focus" states (always have a clear focus owner)

#### Fallback

- **FR-038**: System MUST provide native chat UI when CEF is disabled/unavailable
- **FR-039**: System MUST provide native scoreboard UI when CEF is disabled/unavailable
- **FR-040**: System MUST use native HUD (Feature 005) when CEF HUD is disabled
- **FR-041**: System MUST detect CEF failure and automatically switch to native fallback

### Key Entities

- **HudState**: Player's current HP, max HP, RTT in ms, optional FPS; pushed to UI at throttled rate
- **ChatMessage**: Contains author (string), text (string), kind (player/system), timestamp (u64); received from server and stored locally
- **ScoreboardRow**: Contains player name, ping_ms, optional score, kills, deaths, team identifier; pushed when scoreboard visible
- **ScoreboardState**: Contains server_name and array of ScoreboardRow; sent to UI on request
- **UiConfig**: Contains cefHudEnabled boolean, keybind mappings; sent on ui.ready
- **InputFocusState**: Tracks whether UI or gameplay owns input; prevents focus conflicts

## Success Criteria

### Measurable Outcomes

- **SC-001**: HUD updates visible within 100ms of HP/ping changes during gameplay
- **SC-002**: Chat messages sent appear in sender's history within 200ms (local round-trip)
- **SC-003**: Chat messages from other players appear within 500ms of server broadcast
- **SC-004**: Scoreboard appears within 100ms of key press
- **SC-005**: Scoreboard disappears within 50ms of key release
- **SC-006**: Zero input leak incidents (no gameplay actions while typing in chat) during testing
- **SC-007**: Zero stuck focus incidents (always recoverable to gameplay) during testing
- **SC-008**: Native fallback activates within 1 second of CEF failure detection
- **SC-009**: HUD overlay does not cause frame drops below 60fps during normal gameplay
- **SC-010**: Chat history retains 100 messages without memory growth issues

## Out of Scope (v1)

- Private/whisper messages (team chat only if server supports, otherwise global only)
- Chat channels or rooms
- Rich text formatting in chat (bold, colors, links)
- Killfeed or death notifications
- Minimap or radar
- Inventory/hotbar UI
- Chat emotes or reactions
- Voice chat indicators
- Chat moderation UI (mute/block players)
- Custom chat themes or skins

## Assumptions

- Feature 030 (CEF UI Shell) provides working OSR texture rendering at game resolution
- Feature 031 (CEF Menus) establishes bridge protocol that can be extended for in-game messages
- Feature 005 (Native UI) provides functional HP display and crosshair for fallback
- Server already broadcasts chat messages to connected clients (or protocol can be extended)
- Server provides player list with ping data for scoreboard (or protocol can be extended)
- Keybinds are configurable via existing settings system (Feature 005)
- The game maintains an authoritative tick rate where chat/scoreboard can integrate
