# Feature Specification: CEF Media Embeds (YouTube / Twitch / Spotify)

**Feature Branch**: `033-cef-embeds`
**Created**: 2025-12-18
**Status**: Draft
**Input**: User description: "Optional, secure media embeds panel for YouTube, Twitch, and Spotify with whitelist security, rate limiting, and proper focus management"

## Overview

Enable players to display media embeds (YouTube, Twitch, Spotify) within a controlled CEF panel overlay:
- **Optional**: Disabled by default if desired, fully toggleable per-provider
- **Secure**: Strict domain whitelist, no free navigation, no user-injected HTML
- **Non-blocking**: Gameplay continues; player chooses when to focus the panel
- **Fallback-compatible**: Graceful degradation when CEF is unavailable

This feature is for **social/comfort use** (music, streams, videos) and has **no gameplay authority**.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - YouTube Video Playback (Priority: P1)

As a player, I want to load a YouTube video via URL while in-game without leaving the game, so I can watch content during gameplay or downtime.

**Why this priority**: YouTube is the most common embed use case. Core value is watching videos while playing. This validates the entire embed panel system.

**Independent Test**: Can be fully tested by opening panel, pasting a YouTube URL, verifying video loads and plays. Delivers immediate value as a standalone feature.

**Acceptance Scenarios**:

1. **Given** I am in-game with embeds enabled, **When** I press the panel toggle keybind (F8), **Then** the embed panel becomes visible
2. **Given** the panel is visible, **When** I enter a valid YouTube URL (youtube.com or youtu.be), **Then** the video loads in the embed player
3. **Given** a video is playing, **When** I unfocus the panel and return to gameplay, **Then** the video continues playing and gameplay inputs work normally
4. **Given** the panel is visible, **When** I enter an invalid URL, **Then** I see error EEMB001 with a user-friendly message
5. **Given** the panel is focused, **When** I press Escape, **Then** I return to gameplay without closing the panel (video continues)

---

### User Story 2 - Twitch Stream Viewing (Priority: P2)

As a player, I want to watch a Twitch stream (and optionally its chat) while in-game, so I can follow live content during gameplay.

**Why this priority**: Twitch is the second most common streaming platform. Extends the embed system to live content. Chat integration adds social value.

**Independent Test**: Can be tested by loading a Twitch channel, verifying stream plays, optionally enabling/disabling chat overlay.

**Acceptance Scenarios**:

1. **Given** embeds and Twitch provider are enabled, **When** I enter a Twitch channel URL or name, **Then** the live stream loads in the embed player
2. **Given** Twitch chat is enabled in config, **When** a stream is loaded, **Then** the chat panel is visible alongside the video
3. **Given** Twitch chat is disabled in config, **When** a stream is loaded, **Then** only the video player is shown without chat
4. **Given** Twitch provider is disabled, **When** I attempt to load a Twitch URL, **Then** I see error EEMB002 (provider disabled)

---

### User Story 3 - Provider and Feature Toggles (Priority: P2)

As a player (or server admin), I want to disable embeds entirely or disable specific providers, so I can control performance, security, or focus during competitive play.

**Why this priority**: Essential for parental controls, competitive integrity, and performance management. Ensures the feature doesn't negatively impact gameplay when unwanted.

**Independent Test**: Can be tested by toggling config flags and verifying panel/provider availability changes accordingly.

**Acceptance Scenarios**:

1. **Given** `ui.cef_embeds=false`, **When** I press the panel toggle keybind, **Then** nothing happens (or a brief "embeds disabled" message appears)
2. **Given** embeds enabled but `ui.cef_embeds_youtube=false`, **When** I enter a YouTube URL, **Then** I see error EEMB002 (provider disabled)
3. **Given** embeds enabled, **When** I view settings, **Then** I can toggle individual providers (YouTube, Twitch, Spotify)
4. **Given** a server enforces embeds disabled, **When** I connect, **Then** my local embeds preference is overridden and embeds are unavailable

---

### User Story 4 - Spotify Playback (Priority: P3 - LATER)

As a player, I want to play Spotify tracks/playlists in the embed panel, so I can listen to music while gaming.

**Why this priority**: Spotify has DRM and licensing complexities. Marked as LATER - API and toggle should exist but implementation can be stubbed.

**Independent Test**: Would be tested by loading a Spotify embed URL. For now, should return EEMB002 (provider disabled).

**Acceptance Scenarios**:

1. **Given** Spotify provider is enabled (future), **When** I enter a Spotify URL, **Then** the Spotify embed player loads
2. **Given** Spotify provider is disabled (default), **When** I enter a Spotify URL, **Then** I see error EEMB002 (provider disabled)

---

### Edge Cases

- What happens when the user pastes a URL from an unsupported domain?
  → Navigation blocked, error EEMB003 logged, user sees "domain not allowed" message
- What happens when the user spams load requests?
  → Rate limited to 1 action per 2 seconds, excess requests silently dropped with optional UI feedback
- What happens when CEF is unavailable or crashes?
  → Embed feature gracefully unavailable, no crash propagation to game
- What happens when the user alt-tabs while embed is focused?
  → Focus returns to gameplay state on window focus loss (no stuck focus)
- What happens when an embed tries to navigate away from the whitelisted domain?
  → Navigation blocked and logged, current content preserved
- What happens when the panel is visible but not focused?
  → No input captured by embed; mouse clicks pass through transparent areas; video continues playing

## Requirements *(mandatory)*

### Functional Requirements

#### Panel & Display

- **FR-001**: System MUST provide a toggleable overlay panel for media embeds, activated via configurable keybind (default: F8)
- **FR-002**: Panel MUST support at least 1 embed slot (multi-slot is optional/future)
- **FR-003**: Panel MUST NOT interrupt gameplay when visible but unfocused
- **FR-004**: Panel position and transparency SHOULD be configurable (simple implementation acceptable)

#### Provider Support

- **FR-005**: System MUST support YouTube embeds (iframe player via youtube-nocookie.com preferred)
- **FR-006**: System MUST support Twitch embeds (player with optional chat)
- **FR-007**: System MUST stub Spotify provider with disabled-by-default toggle (full implementation LATER)
- **FR-008**: System MUST normalize user input (URLs or direct IDs) to canonical embed URLs

#### Input Handling

- **FR-009**: System MUST define two input states: `Gameplay` (no embed focus) and `EmbedFocus` (panel receives input)
- **FR-010**: Escape key MUST return focus to gameplay without necessarily closing the panel
- **FR-011**: System MUST handle window focus loss by returning to `Gameplay` state (no stuck focus)
- **FR-012**: Keybind (F8 default) MUST toggle panel visibility; panel focus is a separate action
- **FR-012a**: Clicking anywhere on the visible panel MUST give it focus (transition to `EmbedFocus` state)

#### Security (MANDATORY)

- **FR-013**: System MUST implement strict domain whitelist:
  - YouTube: `youtube.com`, `www.youtube.com`, `youtu.be`, `youtube-nocookie.com`
  - Twitch: `twitch.tv`, `player.twitch.tv`, `www.twitch.tv`
  - Spotify (future): `open.spotify.com`
- **FR-014**: System MUST block all navigation attempts to non-whitelisted domains
- **FR-015**: System MUST log all blocked navigation attempts for debugging
- **FR-016**: System MUST NOT provide an address bar or free navigation capability
- **FR-017**: System MUST NOT inject user-provided HTML into the DOM (URLs treated as data only)
- **FR-018**: System MUST NOT execute eval() or dynamic script injection from external sources
- **FR-019**: System MUST enforce rate limiting (max 1 load action per 2 seconds)

#### Configuration

- **FR-020**: System MUST provide `ui.cef_embeds` toggle (master enable/disable)
- **FR-021**: System MUST provide per-provider toggles: `ui.cef_embeds_youtube`, `ui.cef_embeds_twitch`, `ui.cef_embeds_spotify`
- **FR-022**: System SHOULD provide optional toggles: `ui.cef_embeds_autoplay` (default: off), `ui.cef_embeds_chat` (Twitch chat)
- **FR-023**: All toggle defaults: embeds=on, youtube=on, twitch=on, spotify=off, autoplay=off

#### Bridge Protocol (JS ↔ Rust)

- **FR-024**: Bridge MUST support UI→Game messages: `EmbedOpenPanel`, `EmbedClosePanel`, `EmbedFocus`, `EmbedUnfocus`, `EmbedLoad`, `EmbedStop`
- **FR-025**: Bridge MUST support Game→UI messages: `EmbedState`, `EmbedError`, `UiConfig` (with embed settings)
- **FR-026**: Error codes MUST include: EEMB001 (invalid URL), EEMB002 (provider disabled), EEMB003 (blocked domain)
- **FR-027**: Bridge messages MUST use the existing versioned protocol from Feature 031

#### Debug & Observability

- **FR-028**: System MUST log embed bridge messages when `ui.debug_bridge` is enabled
- **FR-029**: System MUST log security events (blocked navigations, rate limiting)
- **FR-030**: System SHOULD support debug UI showing current provider, embed URL, and focus state

### Key Entities

- **EmbedPanel**: The overlay container (visible state, focused state, position, transparency)
- **EmbedSlot**: A single embed instance (provider, content URL, playback state)
- **EmbedProvider**: Enum of supported providers (YouTube, Twitch, Spotify) with their whitelist domains
- **EmbedConfig**: User preferences for embed feature (toggles, keybind, autoplay, chat visibility)
- **EmbedError**: Error type with code (EEMB001-003) and user-friendly message

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Players can load and watch a YouTube video within 5 seconds of entering a valid URL
- **SC-002**: Panel toggle (F8) responds within 100ms with visible state change
- **SC-003**: Focus transitions (panel ↔ gameplay) complete within 50ms with no input leakage
- **SC-004**: 100% of navigation attempts to non-whitelisted domains are blocked and logged
- **SC-005**: Rate limiting correctly enforces 1 action per 2 seconds without false positives
- **SC-006**: Zero stuck-focus states occur during normal operation (verified via automated testing)
- **SC-007**: Embed panel has zero impact on gameplay frame rate when hidden (within measurement error)
- **SC-008**: When panel is visible but unfocused, frame rate impact is less than 5%
- **SC-009**: All provider toggles correctly enable/disable their respective functionality
- **SC-010**: Graceful degradation when CEF unavailable (no crashes, clean error message)

## Scope Boundaries

### In Scope

- YouTube video embed (iframe player)
- Twitch stream embed (with optional chat)
- Spotify stub (API ready, implementation deferred)
- Domain whitelist security
- Rate limiting
- Focus state management
- Configuration toggles
- Debug logging

### Out of Scope

- Authentication/login for any provider
- DRM/advanced Spotify integration
- Party/synchronized watching
- Persistent overlays during gameplay
- Content downloading or caching
- Monetization, ads, or tracking
- Multiple simultaneous embed slots (future consideration)
- Audio mixing/volume control (simple mute toggle acceptable)

## Clarifications

### Session 2025-12-18

- Q: How does the player enter focus mode to interact with the embed panel? → A: Click anywhere on the visible panel to give it focus

## Assumptions

- CEF shell from Feature 030 is available and functional
- Bridge protocol from Feature 031 is stable and can be extended
- Twitch embeds will work with `parent=localhost` or equivalent local domain
- YouTube embeds via youtube-nocookie.com provide adequate privacy
- Players understand they need to manually focus the panel to interact with it
