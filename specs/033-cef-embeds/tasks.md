# Tasks: CEF Media Embeds (YouTube / Twitch / Spotify)

**Input**: Design documents from `/specs/033-cef-embeds/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/bridge-messages.md

**Tests**: Not explicitly requested in specification. Test tasks omitted.

**Organization**: Tasks grouped by user story for independent implementation.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: US1=YouTube, US2=Twitch, US3=Toggles, US4=Spotify (stub)

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization, config wiring, directory structure

- [X] T001 Add embed config fields to crates/plix-client/src/ui_cef/config.rs (cef_embeds, cef_embeds_youtube, cef_embeds_twitch, cef_embeds_spotify, cef_embeds_autoplay, cef_embeds_chat, cef_embeds_twitch_parent)
- [X] T002 [P] Create assets/ui/embeds/ directory structure
- [X] T003 [P] Create assets/ui/embeds/embeds.html with panel skeleton (header, input, iframe slot, error zone)
- [X] T004 [P] Create assets/ui/embeds/embeds.css with panel styles (visibility toggle, focus indicator)
- [X] T005 [P] Create assets/ui/embeds/embeds.js with bridge bus skeleton (window.plix.send, onMessage)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

### Bridge Protocol Extensions

- [X] T006 Add MessageType variants (EmbedOpenPanel, EmbedClosePanel, EmbedFocus, EmbedUnfocus, EmbedLoad, EmbedStop) in crates/plix-client/src/ui_cef/bridge/messages.rs
- [X] T007 Add PushType variants (EmbedState, EmbedError) in crates/plix-client/src/ui_cef/bridge/messages.rs
- [X] T008 Add BridgeError helpers for EEMB001-004 in crates/plix-client/src/ui_cef/bridge/messages.rs
- [X] T009 [P] Add embed payload structs (EmbedLoadPayload, EmbedStatePayload) in crates/plix-client/src/ui_cef/bridge/serialize.rs
- [X] T010 [P] Add embed message handlers dispatch in crates/plix-client/src/ui_cef/bridge/mod.rs

### Input Focus State Machine

- [X] T011 Add EmbedFocus variant to InputFocus enum in crates/plix-client/src/ui_cef/input.rs
- [X] T012 Add is_embed_focus(), give_embed_focus(), release_embed_focus() methods in crates/plix-client/src/ui_cef/input.rs
- [X] T013 Add focus recovery logic for window focus loss in crates/plix-client/src/ui_cef/input.rs

### Embeds Module Structure

- [X] T014 Create crates/plix-client/src/ui_cef/embeds/mod.rs with EmbedsManager struct skeleton
- [X] T015 [P] Create crates/plix-client/src/ui_cef/embeds/config.rs with EmbedConfig struct
- [X] T016 [P] Create crates/plix-client/src/ui_cef/embeds/provider.rs with EmbedProvider enum and whitelist constants
- [X] T017 [P] Create crates/plix-client/src/ui_cef/embeds/normalizer.rs with URL normalization trait skeleton
- [X] T018 Export embeds module from crates/plix-client/src/ui_cef/mod.rs

### JS Bridge Integration

- [X] T019 Implement embed message routing in assets/ui/embeds/embeds.js (send EmbedLoad, EmbedStop, receive EmbedState, EmbedError)
- [X] T020 Implement UiConfig handler with embedsEnabled, providersEnabled in assets/ui/embeds/embeds.js

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - YouTube Video Playback (Priority: P1) 🎯 MVP

**Goal**: Load and watch YouTube videos via URL while in-game

**Independent Test**: Open panel with F8, paste YouTube URL, verify video loads and plays. Return to gameplay with Escape.

### Implementation for User Story 1

- [X] T021 [US1] Implement YouTube URL parsing (watch?v=, youtu.be/, shorts/) in crates/plix-client/src/ui_cef/embeds/normalizer.rs
- [X] T022 [US1] Implement YouTube canonical URL generation (youtube-nocookie.com/embed/{id}) in crates/plix-client/src/ui_cef/embeds/normalizer.rs
- [X] T023 [US1] Add YouTube whitelist domains to provider.rs (youtube.com, www.youtube.com, youtu.be, youtube-nocookie.com, www.youtube-nocookie.com)
- [X] T024 [US1] Implement EmbedsManager.open_panel() and close_panel() in crates/plix-client/src/ui_cef/embeds/mod.rs
- [X] T025 [US1] Implement EmbedsManager.load() with provider validation in crates/plix-client/src/ui_cef/embeds/mod.rs
- [X] T026 [US1] Implement EmbedsManager.stop() in crates/plix-client/src/ui_cef/embeds/mod.rs
- [X] T027 [US1] Implement EmbedState push on state change in crates/plix-client/src/ui_cef/embeds/mod.rs
- [X] T028 [US1] Handle F8 keybind for panel toggle in crates/plix-client/src/input.rs
- [X] T029 [US1] Implement click-to-focus (EmbedFocus on panel click) via JS EmbedFocus message in assets/ui/embeds/embeds.js
- [X] T030 [US1] Handle Escape key to unfocus (release_embed_focus) in crates/plix-client/src/input.rs
- [X] T031 [P] [US1] Implement iframe src update on EmbedState in assets/ui/embeds/embeds.js
- [X] T032 [P] [US1] Implement error display (EEMB001) in assets/ui/embeds/embeds.js
- [X] T033 [US1] Verify gameplay input blocked when EmbedFocus, unblocked otherwise in crates/plix-client/src/input.rs

**Checkpoint**: YouTube playback should be fully functional and testable independently

---

## Phase 4: User Story 2 - Twitch Stream Viewing (Priority: P2)

**Goal**: Watch Twitch streams (and optionally chat) while in-game

**Independent Test**: Load Twitch channel URL, verify stream plays. Toggle chat visibility.

### Implementation for User Story 2

- [X] T034 [US2] Implement Twitch URL parsing (twitch.tv/<channel>, player.twitch.tv) in crates/plix-client/src/ui_cef/embeds/normalizer.rs
- [X] T035 [US2] Implement Twitch canonical URL generation (player.twitch.tv/?channel={}&parent={}) in crates/plix-client/src/ui_cef/embeds/normalizer.rs
- [X] T036 [US2] Add Twitch whitelist domains to provider.rs (twitch.tv, www.twitch.tv, player.twitch.tv)
- [X] T037 [US2] Add twitch_parent config usage in URL generation in crates/plix-client/src/ui_cef/embeds/normalizer.rs
- [X] T038 [P] [US2] Add Twitch chat iframe support (optional, gated by cef_embeds_chat) in assets/ui/embeds/embeds.js
- [X] T039 [P] [US2] Add Twitch chat CSS layout (side-by-side or stacked) in assets/ui/embeds/embeds.css
- [X] T040 [US2] Handle Twitch provider disabled (EEMB002) in crates/plix-client/src/ui_cef/embeds/mod.rs

**Checkpoint**: Twitch streaming should be fully functional and testable independently

---

## Phase 5: User Story 3 - Provider and Feature Toggles (Priority: P2)

**Goal**: Enable/disable embeds and individual providers via config

**Independent Test**: Toggle ui.cef_embeds=false, verify F8 does nothing. Toggle provider, verify EEMB002.

### Implementation for User Story 3

- [X] T041 [US3] Implement config gating in EmbedsManager.open_panel() (check cef_embeds) in crates/plix-client/src/ui_cef/embeds/mod.rs
- [X] T042 [US3] Implement provider gating in EmbedsManager.load() (check provider-specific toggle) in crates/plix-client/src/ui_cef/embeds/mod.rs
- [X] T043 [US3] Add disabled feedback (brief system message or no-op) when embeds disabled in crates/plix-client/src/ui_cef/embeds/mod.rs
- [X] T044 [US3] Expose provider toggles in UiConfig push (embedsEnabled, providersEnabled map) in crates/plix-client/src/ui_cef/embeds/mod.rs
- [X] T045 [P] [US3] Show/hide provider options in UI based on providersEnabled in assets/ui/embeds/embeds.js

**Checkpoint**: All toggles should work correctly

---

## Phase 6: User Story 4 - Spotify Stub (Priority: P3 - LATER)

**Goal**: Stub Spotify provider (API ready, disabled by default)

**Independent Test**: Enter Spotify URL, verify EEMB002 (provider disabled).

### Implementation for User Story 4

- [X] T046 [US4] Implement Spotify URL parsing (open.spotify.com/...) stub in crates/plix-client/src/ui_cef/embeds/normalizer.rs
- [X] T047 [US4] Add Spotify whitelist domain (open.spotify.com) to provider.rs (for future use)
- [X] T048 [US4] Ensure Spotify returns EEMB002 when spotify_enabled=false (default) in crates/plix-client/src/ui_cef/embeds/mod.rs

**Checkpoint**: Spotify stub ready without blocking release

---

## Phase 7: Security (CEF Navigation Guard)

**Purpose**: Block all unauthorized navigation, popups, downloads

- [X] T049 Create crates/plix-client/src/ui_cef/embeds/navigation_guard.rs with whitelist check function
- [X] T050 Implement domain extraction and whitelist matching in navigation_guard.rs
- [X] T051 Hook CEF OnBeforeBrowse to call navigation guard in crates/plix-client/src/ui_cef/mod.rs (or shell)
- [X] T052 Block and log non-whitelisted navigation (return EEMB003) in navigation_guard.rs
- [X] T053 [P] Block popups (window.open) in CEF shell
- [X] T054 [P] Block downloads in CEF shell
- [X] T055 [P] Block file:// URLs in navigation_guard.rs

**Checkpoint**: No unauthorized navigation possible

---

## Phase 8: Rate Limiting & Robustness

**Purpose**: Anti-spam and edge case handling

### Rate Limiting

- [X] T056 Add last_load_at: Option<Instant> to EmbedsManager state in crates/plix-client/src/ui_cef/embeds/mod.rs
- [X] T057 Implement 2s cooldown check in EmbedsManager.load() in crates/plix-client/src/ui_cef/embeds/mod.rs
- [X] T058 Return EEMB004 on rate limit violation in crates/plix-client/src/ui_cef/embeds/mod.rs
- [X] T059 [P] Add rate limit UI feedback (brief message or disable button) in assets/ui/embeds/embeds.js

### Edge Cases & Robustness

- [X] T060 Handle alt-tab/window focus loss (auto-unfocus) in crates/plix-client/src/ui_cef/input.rs
- [X] T061 Handle window resize (panel remains functional) in assets/ui/embeds/embeds.css
- [X] T062 Handle UI reload (EmbedsManager sends EmbedState resync) in crates/plix-client/src/ui_cef/embeds/mod.rs
- [X] T063 Handle F8 spam (debounce or stable toggle) in crates/plix-client/src/input.rs
- [X] T064 Implement load-while-hidden behavior (auto-open panel) in crates/plix-client/src/ui_cef/embeds/mod.rs

**Checkpoint**: Feature is robust under stress

---

## Phase 9: Fallback & CEF OFF

**Purpose**: Graceful degradation when CEF unavailable

- [X] T065 Check CefShell availability before opening panel in crates/plix-client/src/ui_cef/embeds/mod.rs
- [X] T066 Display system message "Embeds unavailable (CEF disabled)" when CEF OFF in crates/plix-client/src/ui_cef/embeds/mod.rs
- [X] T067 Ensure F8 does not crash when CEF unavailable in crates/plix-client/src/input.rs

**Checkpoint**: No crashes when CEF disabled

---

## Phase 10: Polish & Documentation

**Purpose**: Debug logging, documentation, final validation

### Debug & Observability

- [X] T068 Add debug logging for embed bridge messages when ui.debug_bridge enabled in crates/plix-client/src/ui_cef/embeds/mod.rs
- [X] T069 [P] Add security event logging (blocked navigations, rate limits) in navigation_guard.rs
- [X] T070 [P] Add debug state display in UI (provider, embed_url, focused) in assets/ui/embeds/embeds.js

### Documentation

- [X] T071 Create docs/feature-033.md with toggles, whitelist, URL formats, errors, focus behavior

### Final Validation (DoD)

- [X] T072 Validate: F8 show/hide panel without crash
- [X] T073 Validate: Click panel => focus; Escape => unfocus; no input leak
- [X] T074 Validate: YouTube URLs load correctly (watch, youtu.be, shorts)
- [X] T075 Validate: Twitch URLs load correctly (channel, player URL)
- [X] T076 Validate: Whitelist blocks all non-allowed domains (EEMB003)
- [X] T077 Validate: Rate limiting works (EEMB004)
- [X] T078 Validate: Provider toggles work (EEMB002)
- [X] T079 Validate: CEF OFF => graceful fallback

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies - can start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1 - BLOCKS all user stories
- **Phases 3-6 (User Stories)**: All depend on Phase 2 completion
- **Phase 7 (Security)**: Can run in parallel with Phases 3-6 (different files)
- **Phase 8 (Robustness)**: Depends on Phases 3-6
- **Phase 9 (Fallback)**: Depends on Phase 3
- **Phase 10 (Polish)**: Depends on all previous phases

### User Story Dependencies

- **US1 (YouTube)**: Foundational only - no cross-story dependencies (MVP)
- **US2 (Twitch)**: Foundational only - independent of US1
- **US3 (Toggles)**: Foundational only - tests all providers
- **US4 (Spotify)**: Foundational only - stub independent

### Parallel Opportunities

**Within Phase 1 (Setup)**:
```
T002, T003, T004, T005 - parallel (different files)
```

**Within Phase 2 (Foundational)**:
```
T009, T010 - parallel (serialize, handlers)
T015, T016, T017 - parallel (config, provider, normalizer)
```

**Within Phase 3 (US1: YouTube)**:
```
T031, T032 - parallel (JS files)
```

**Across User Stories (with team)**:
```
After Phase 2:
  Developer A: US1 (YouTube) - MVP
  Developer B: US2 (Twitch)
  Developer C: Phase 7 (Security)
Then:
  Developer A: US3 (Toggles)
  Developer B: US4 (Spotify stub)
  Developer C: Phase 8 (Robustness)
```

---

## Summary

| Phase | Description | Task Count | Parallel Tasks |
|-------|-------------|------------|----------------|
| 1 | Setup | 5 | 4 |
| 2 | Foundational | 15 | 6 |
| 3 | US1: YouTube (P1) 🎯 MVP | 13 | 2 |
| 4 | US2: Twitch (P2) | 7 | 2 |
| 5 | US3: Toggles (P2) | 5 | 1 |
| 6 | US4: Spotify (P3) | 3 | 0 |
| 7 | Security | 7 | 3 |
| 8 | Robustness | 9 | 1 |
| 9 | Fallback | 3 | 0 |
| 10 | Polish | 12 | 2 |
| **Total** | | **79** | **21** |

**MVP Scope**: Phase 1 + Phase 2 + Phase 3 (US1: YouTube) = 33 tasks
**Core Feature**: + Phase 4 + Phase 5 + Phase 7 = 55 tasks
**Full Feature**: All phases = 79 tasks
