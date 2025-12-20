# Tasks: CEF In-Game UI (HUD, Chat, Scoreboard)

**Input**: Design documents from `/specs/032-cef-ingame-ui/`
**Prerequisites**: plan.md, spec.md, data-model.md, contracts/bridge-messages.md

**Tests**: Not explicitly requested in specification. Test tasks omitted.

**Organization**: Tasks grouped by user story for independent implementation.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: US1=HUD Display, US2=Chat Communication, US3=Chat Input Focus, US4=Chat Commands, US5=Scoreboard Display, US6=Native Fallback

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization, feature flags, configuration wiring

- [ ] T001 Add ui.cef_hud, ui.cef_chat, ui.cef_scoreboard config flags in crates/plix-client/src/config.rs
- [ ] T002 [P] Add ui.debug_bridge config flag for bridge message logging in crates/plix-client/src/config.rs
- [ ] T003 [P] Create assets/ui/ingame/ directory structure for web overlay assets
- [ ] T004 [P] Add ChatMessageKind enum and ChatMessage protocol types in crates/plix-common/src/chat.rs
- [ ] T005 Export chat module from crates/plix-common/src/lib.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

### Bridge Protocol Extensions

- [ ] T006 Add ChatSend, ChatOpen, ChatClose, ChatClear to MessageType enum in crates/plix-client/src/ui_cef/bridge/messages.rs
- [ ] T007 Add HudState, ChatMessage, ChatToast, ScoreboardState, UiConfig push types in crates/plix-client/src/ui_cef/bridge/messages.rs
- [ ] T008 Add ECHAT001, ECHAT002, ECHAT003 error codes to BridgeError in crates/plix-client/src/ui_cef/bridge/messages.rs
- [ ] T009 [P] Add ChatSend validation handler in crates/plix-client/src/ui_cef/bridge/handlers.rs
- [ ] T010 [P] Add ChatOpen/ChatClose handlers in crates/plix-client/src/ui_cef/bridge/handlers.rs
- [ ] T011 [P] Add ChatClear handler in crates/plix-client/src/ui_cef/bridge/handlers.rs

### Server Protocol Extensions

- [ ] T012 Add ClientMessage::ChatSend variant in crates/plix-common/src/protocol/messages.rs
- [ ] T013 Add GameEvent::ChatReceived variant in crates/plix-common/src/protocol/messages.rs

### Input Focus State Machine

- [ ] T014 Add ChatTyping variant to InputFocus enum in crates/plix-client/src/ui_cef/input.rs
- [ ] T015 Add is_chat_typing(), give_chat_focus(), release_chat_focus() methods in crates/plix-client/src/ui_cef/input.rs
- [ ] T016 Add focus recovery logic for alt-tab/window focus loss in crates/plix-client/src/ui_cef/input.rs

### In-Game Overlay Module Structure

- [ ] T017 Create crates/plix-client/src/ui_cef/ingame/mod.rs with IngameOverlay coordinator struct
- [ ] T018 [P] Create crates/plix-client/src/ui_cef/ingame/hud.rs with HudStatePublisher struct skeleton
- [ ] T019 [P] Create crates/plix-client/src/ui_cef/ingame/chat.rs with ChatClient struct skeleton
- [ ] T020 [P] Create crates/plix-client/src/ui_cef/ingame/scoreboard.rs with ScoreboardClient struct skeleton
- [ ] T021 Export ingame module from crates/plix-client/src/ui_cef/mod.rs

### Web Overlay Base

- [ ] T022 Create assets/ui/ingame/overlay.html with HUD/Chat/Scoreboard container divs
- [ ] T023 [P] Create assets/ui/ingame/overlay.css with base styles and transparency
- [ ] T024 [P] Create assets/ui/ingame/overlay.js with window.plix.send/onMessage bus and handler registry

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - HUD Display (Priority: P1) 🎯 MVP

**Goal**: Display HP, ping/RTT, FPS overlay with web crosshair when CEF HUD enabled

**Independent Test**: Join game with CEF HUD enabled, verify HP bar updates on damage, verify ping reflects latency, verify crosshair renders in center

### Implementation for User Story 1

- [ ] T025 [US1] Implement HudState struct with hp, max_hp, rtt_ms, fps, crosshair_visible fields in crates/plix-client/src/ui_cef/ingame/hud.rs
- [ ] T026 [US1] Implement HudStatePublisher with throttling logic (15 Hz + immediate on HP change) in crates/plix-client/src/ui_cef/ingame/hud.rs
- [ ] T027 [US1] Add maybe_publish() method that returns Option<HudState> based on throttle/change in crates/plix-client/src/ui_cef/ingame/hud.rs
- [ ] T028 [US1] Implement serialize_hud_state() to create BridgePush for HudState in crates/plix-client/src/ui_cef/bridge/serialize.rs
- [ ] T029 [US1] Integrate HudStatePublisher into game loop (collect HP/RTT/FPS, publish to bridge) in crates/plix-client/src/ui_cef/ingame/mod.rs
- [ ] T030 [US1] Add ui.cef_hud gating (skip publish if disabled) in crates/plix-client/src/ui_cef/ingame/mod.rs
- [ ] T031 [P] [US1] Implement HUD module in assets/ui/ingame/overlay.js with HP bar, value, RTT, FPS display
- [ ] T032 [P] [US1] Style HUD elements (HP bar, values, positioning) in assets/ui/ingame/overlay.css
- [ ] T033 [US1] Implement crosshair rendering (centered div/CSS) in assets/ui/ingame/overlay.js
- [ ] T034 [US1] Disable native crosshair when cef_hud_enabled in crates/plix-client/src/ui/crosshair.rs
- [ ] T035 [US1] Send UiConfig push on ui.ready with cefHudEnabled, cefCrosshairEnabled, keybinds in crates/plix-client/src/ui_cef/ingame/mod.rs

**Checkpoint**: HUD should display HP/RTT/FPS with web crosshair, native crosshair disabled when CEF HUD on

---

## Phase 4: User Story 2 - Chat Communication (Priority: P1)

**Goal**: Send/receive chat messages, display in scrollable history, show toast when chat closed

**Independent Test**: Open chat, type message, send, verify appears in history. Have another player send, verify appears.

### Implementation for User Story 2

- [ ] T036 [US2] Implement ChatMessage struct (id, author, text, kind, timestamp) in crates/plix-client/src/ui_cef/ingame/chat.rs
- [ ] T037 [US2] Implement ChatHistory with VecDeque<ChatMessage>, max 100, add/evict logic in crates/plix-client/src/ui_cef/ingame/chat.rs
- [ ] T038 [US2] Implement ChatClient struct (history, is_open, pending_text, last_send_time, rate_limit_ms) in crates/plix-client/src/ui_cef/ingame/chat.rs
- [ ] T039 [US2] Implement send() method with 200 char validation, 500ms rate limit in crates/plix-client/src/ui_cef/ingame/chat.rs
- [ ] T040 [US2] Implement receive() method that adds to history and triggers toast if chat closed in crates/plix-client/src/ui_cef/ingame/chat.rs
- [ ] T041 [US2] Implement serialize_chat_message() for ChatMessage push in crates/plix-client/src/ui_cef/bridge/serialize.rs
- [ ] T042 [US2] Implement serialize_chat_toast() for ChatToast push in crates/plix-client/src/ui_cef/bridge/serialize.rs
- [ ] T043 [US2] Handle ChatSend bridge message (validate, send ClientMessage::ChatSend to server) in crates/plix-client/src/ui_cef/bridge/handlers.rs
- [ ] T044 [US2] Handle GameEvent::ChatReceived (call ChatClient.receive(), push to UI) in crates/plix-client/src/ui_cef/ingame/mod.rs
- [ ] T045 [US2] Implement server-side chat broadcast in crates/plix-server/src/game.rs (add ChatSend handling, broadcast ChatReceived)
- [ ] T046 [P] [US2] Implement Chat module in assets/ui/ingame/overlay.js with history list, scroll, message rendering
- [ ] T047 [P] [US2] Style chat history (message list, author, timestamp, kind styling) in assets/ui/ingame/overlay.css
- [ ] T048 [US2] Implement toast notification component (fade-in/out, 3s duration, queue of 3) in assets/ui/ingame/overlay.js
- [ ] T049 [US2] Style toast notifications (positioning, fade animation) in assets/ui/ingame/overlay.css

**Checkpoint**: Chat messages send/receive working, toast shows when chat closed

---

## Phase 5: User Story 3 - Chat Input Focus (Priority: P1)

**Goal**: Block gameplay input when typing in chat, restore on close, no stuck focus

**Independent Test**: Open chat, type WASD, verify no player movement. Close chat, verify movement works.

### Implementation for User Story 3

- [ ] T050 [US3] Handle ChatOpen message to transition focus to ChatTyping in crates/plix-client/src/ui_cef/bridge/handlers.rs
- [ ] T051 [US3] Handle ChatClose message to transition focus to Game in crates/plix-client/src/ui_cef/bridge/handlers.rs
- [ ] T052 [US3] Block gameplay input dispatch when InputFocus::ChatTyping in crates/plix-client/src/input.rs
- [ ] T053 [US3] Route keyboard events to CEF when InputFocus::ChatTyping in crates/plix-client/src/input.rs
- [ ] T054 [US3] Handle Enter key to open chat (send ChatOpen, transition focus) in crates/plix-client/src/input.rs
- [ ] T055 [US3] Handle Escape key to close chat (send ChatClose, transition focus) in crates/plix-client/src/input.rs
- [ ] T056 [US3] Handle click-outside-chat to close (coordinate with UI) in crates/plix-client/src/ui_cef/ingame/mod.rs
- [ ] T057 [US3] Add stuck-focus recovery (Escape always returns to Game, window focus loss resets) in crates/plix-client/src/ui_cef/input.rs
- [ ] T058 [P] [US3] Implement chat input field (text entry, cursor, selection) in assets/ui/ingame/overlay.js
- [ ] T059 [P] [US3] Style chat input field (focus state, background) in assets/ui/ingame/overlay.css
- [ ] T060 [US3] Send ChatOpen on input focus, ChatClose on blur/Escape in assets/ui/ingame/overlay.js
- [ ] T061 [US3] Send ChatSend on Enter with non-empty text in assets/ui/ingame/overlay.js

**Checkpoint**: Typing in chat doesn't move player, closing restores movement, no stuck states

---

## Phase 6: User Story 4 - Chat Commands (Priority: P2)

**Goal**: /help shows local help, /clear clears history, unknown commands forward to server

**Independent Test**: Type /help, verify help text. Type /clear, verify history cleared.

### Implementation for User Story 4

- [ ] T062 [US4] Implement handle_command() method in ChatClient (parse /, route locally or forward) in crates/plix-client/src/ui_cef/ingame/chat.rs
- [ ] T063 [US4] Implement /help command (add system message with command list) in crates/plix-client/src/ui_cef/ingame/chat.rs
- [ ] T064 [US4] Implement /clear command (call history.clear()) in crates/plix-client/src/ui_cef/ingame/chat.rs
- [ ] T065 [US4] Forward unknown commands to server via ChatSend in crates/plix-client/src/ui_cef/ingame/chat.rs
- [ ] T066 [US4] Call handle_command() before send() if text starts with / in crates/plix-client/src/ui_cef/bridge/handlers.rs

**Checkpoint**: /help and /clear work locally, other /commands go to server

---

## Phase 7: User Story 5 - Scoreboard Display (Priority: P1)

**Goal**: Hold TAB to show player list with ping/stats, release to hide, no input capture

**Independent Test**: Hold TAB, verify scoreboard shows. Release, verify hides. Move while TAB held.

### Implementation for User Story 5

- [ ] T067 [US5] Implement ScoreboardRow struct (name, ping_ms, score, kills, deaths, team) in crates/plix-client/src/ui_cef/ingame/scoreboard.rs
- [ ] T068 [US5] Implement ScoreboardState struct (server_name, rows Vec<ScoreboardRow>) in crates/plix-client/src/ui_cef/ingame/scoreboard.rs
- [ ] T069 [US5] Implement ScoreboardClient struct (visible, cached_state, last_update) in crates/plix-client/src/ui_cef/ingame/scoreboard.rs
- [ ] T070 [US5] Implement show()/hide() methods that set visible flag in crates/plix-client/src/ui_cef/ingame/scoreboard.rs
- [ ] T071 [US5] Implement update_from_snapshot() that builds ScoreboardState from WorldSnapshot (max 64 rows) in crates/plix-client/src/ui_cef/ingame/scoreboard.rs
- [ ] T072 [US5] Implement serialize_scoreboard_state() for ScoreboardState push in crates/plix-client/src/ui_cef/bridge/serialize.rs
- [ ] T073 [US5] Push ScoreboardState only when visible (2-5 Hz or on change) in crates/plix-client/src/ui_cef/ingame/mod.rs
- [ ] T074 [US5] Handle TAB key press to show scoreboard (call show(), push state) in crates/plix-client/src/input.rs
- [ ] T075 [US5] Handle TAB key release to hide scoreboard (call hide()) in crates/plix-client/src/input.rs
- [ ] T076 [US5] Verify scoreboard does NOT block gameplay input (movement/actions work while TAB held) in crates/plix-client/src/input.rs
- [ ] T077 [P] [US5] Implement Scoreboard module in assets/ui/ingame/overlay.js with table/list rendering
- [ ] T078 [P] [US5] Style scoreboard (table, rows, team grouping, header) in assets/ui/ingame/overlay.css
- [ ] T079 [US5] Implement team grouping logic (group by team if present, else single list sorted by score) in assets/ui/ingame/overlay.js

**Checkpoint**: TAB shows scoreboard with player list, release hides, gameplay continues during

---

## Phase 8: User Story 6 - Native UI Fallback (Priority: P2)

**Goal**: When CEF disabled/unavailable, native chat and scoreboard work with same keybinds

**Independent Test**: Disable CEF, verify Enter opens native chat, TAB shows native scoreboard.

### Implementation for User Story 6

- [ ] T080 [US6] Create crates/plix-client/src/ui/chat_native.rs with NativeChatClient struct
- [ ] T081 [US6] Implement native chat input field (simple text entry) in crates/plix-client/src/ui/chat_native.rs
- [ ] T082 [US6] Implement native chat history display (scrollable text log) in crates/plix-client/src/ui/chat_native.rs
- [ ] T083 [US6] Create crates/plix-client/src/ui/scoreboard_native.rs with NativeScoreboard struct
- [ ] T084 [US6] Implement native scoreboard rendering (name | ping | score list) in crates/plix-client/src/ui/scoreboard_native.rs
- [ ] T085 [US6] Export NativeChatClient, NativeScoreboard from crates/plix-client/src/ui/mod.rs
- [ ] T086 [US6] Add CEF fallback detection and routing in crates/plix-client/src/ui_cef/ingame/mod.rs
- [ ] T087 [US6] Route chat/scoreboard to native when CefShell::should_fallback() in crates/plix-client/src/ui_cef/ingame/mod.rs
- [ ] T088 [US6] Ensure native HUD (Feature 005) displays when CEF HUD disabled in crates/plix-client/src/ui/hud.rs
- [ ] T089 [US6] Add CEF crash detection with auto-fallback in crates/plix-client/src/ui_cef/mod.rs

**Checkpoint**: With CEF OFF, chat/scoreboard/HUD all work via native UI

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Edge cases, documentation, validation

### Edge Cases & Robustness

- [ ] T090 [P] Handle alt-tab window focus loss (reset focus to Game if stuck in ChatTyping) in crates/plix-client/src/ui_cef/input.rs
- [ ] T091 [P] Handle window resize (overlay follows resolution) in crates/plix-client/src/ui_cef/ingame/mod.rs
- [ ] T092 [P] Handle UI not ready (queue messages with timeout, drop after 100ms) in crates/plix-client/src/ui_cef/ingame/mod.rs
- [ ] T093 [P] Handle server reconnect (preserve chat history, refresh scoreboard) in crates/plix-client/src/ui_cef/ingame/mod.rs
- [ ] T094 Sanitize player names in scoreboard (escape HTML entities) in assets/ui/ingame/overlay.js
- [ ] T095 Sanitize chat message text (escape HTML entities) in assets/ui/ingame/overlay.js

### Debug & Logging

- [ ] T096 Add bridge debug logging when ui.debug_bridge enabled (log type, payload size) in crates/plix-client/src/ui_cef/bridge/mod.rs
- [ ] T097 Add input focus state logging (transitions, stuck recovery) in crates/plix-client/src/ui_cef/input.rs

### Documentation

- [ ] T098 Create docs/feature-032.md with toggles, keybinds, test scenarios, fallback modes
- [ ] T099 Document bridge message contracts in docs/feature-032.md (reference contracts/bridge-messages.md)

### Final Validation

- [ ] T100 Validate HUD: HP/RTT update, crosshair renders, no double-crosshair, 60fps maintained
- [ ] T101 Validate Chat: open/close/send/recv/history/limits/toast, zero input leak
- [ ] T102 Validate Scoreboard: TAB hold shows, release hides, gameplay continues, team grouping works
- [ ] T103 Validate Bridge: versioned messages, errors logged not crashed, UI not ready handled
- [ ] T104 Validate Fallback: CEF OFF shows native HUD/chat/scoreboard with same keybinds

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies - can start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1 - BLOCKS all user stories
- **Phase 3-8 (User Stories)**: All depend on Phase 2 completion
- **Phase 9 (Polish)**: Depends on all user stories being complete

### User Story Dependencies

- **US1 (HUD Display)**: Foundational only - no cross-story dependencies
- **US2 (Chat Communication)**: Foundational only - no cross-story dependencies
- **US3 (Chat Input Focus)**: Depends on US2 (ChatClient exists)
- **US4 (Chat Commands)**: Depends on US2 and US3 (ChatClient with send flow)
- **US5 (Scoreboard Display)**: Foundational only - no cross-story dependencies
- **US6 (Native Fallback)**: Can start after Foundational, integrates with US1, US2, US5

### Recommended Execution Order

1. **MVP**: Phase 1 → Phase 2 → Phase 3 (US1: HUD) → Deploy/Demo
2. **Core Chat**: → Phase 4 (US2) → Phase 5 (US3) → Deploy/Demo
3. **Full Feature**: → Phase 6 (US4) → Phase 7 (US5) → Phase 8 (US6) → Phase 9

### Parallel Opportunities

**Within Phase 2 (Foundational)**:
```
T009, T010, T011 (handlers) - parallel
T018, T019, T020 (module skeletons) - parallel
T023, T024 (CSS, JS base) - parallel
```

**Within Phase 3 (US1: HUD)**:
```
T031, T032 (UI module, CSS) - parallel
```

**Within Phase 4 (US2: Chat)**:
```
T046, T047 (JS module, CSS) - parallel
```

**Across User Stories (with team)**:
```
After Phase 2:
  Developer A: US1 (HUD)
  Developer B: US2+US3 (Chat core + focus)
  Developer C: US5 (Scoreboard)
Then:
  Developer A: US6 (Fallback)
  Developer B: US4 (Chat commands)
  Developer C: Phase 9 (Polish)
```

---

## Summary

| Phase | User Story | Task Count | Parallel Tasks |
|-------|------------|------------|----------------|
| 1 | Setup | 5 | 3 |
| 2 | Foundational | 19 | 7 |
| 3 | US1: HUD Display (P1) | 11 | 2 |
| 4 | US2: Chat Communication (P1) | 14 | 2 |
| 5 | US3: Chat Input Focus (P1) | 12 | 2 |
| 6 | US4: Chat Commands (P2) | 5 | 0 |
| 7 | US5: Scoreboard Display (P1) | 13 | 2 |
| 8 | US6: Native Fallback (P2) | 10 | 0 |
| 9 | Polish | 15 | 7 |
| **Total** | | **104** | **25** |

**MVP Scope**: Phase 1 + Phase 2 + Phase 3 (US1: HUD Display) = 35 tasks
**Core Feature**: + Phase 4 + Phase 5 (Chat with focus) = 61 tasks
**Full Feature**: All phases = 104 tasks
