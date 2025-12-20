# Research: CEF Media Embeds

**Feature**: 033-cef-embeds
**Date**: 2025-12-18

## Research Tasks

### 1. YouTube Embed URL Format

**Question**: What is the canonical embed URL format for YouTube videos?

**Decision**: Use `https://www.youtube-nocookie.com/embed/{VIDEO_ID}` as the canonical embed URL.

**Rationale**:
- `youtube-nocookie.com` is YouTube's privacy-enhanced mode (no tracking cookies)
- Standard embed format supported by all browsers
- Parameters: `autoplay=0&controls=1` for MVP defaults

**Alternatives Considered**:
- `youtube.com/embed/` - Works but has tracking cookies
- `youtube.com/watch?v=` - Not embeddable in iframe (X-Frame-Options blocks)

**URL Parsing Patterns**:
- `youtube.com/watch?v={ID}` → extract `v` query param
- `youtu.be/{ID}` → extract path segment
- `youtube.com/shorts/{ID}` → extract path segment after `/shorts/`
- `youtube.com/embed/{ID}` → already in embed format, extract ID

### 2. Twitch Embed URL Format

**Question**: What is the canonical embed URL format for Twitch streams?

**Decision**: Use `https://player.twitch.tv/?channel={CHANNEL}&parent={PARENT}` for live streams.

**Rationale**:
- Official Twitch embed player URL
- `parent` parameter is **required** by Twitch for security (must match embedding domain)
- For local CEF, use `parent=localhost` or the file:// origin handler

**Alternatives Considered**:
- `twitch.tv/{channel}` - Not embeddable (requires full page)
- Third-party embed services - Rejected for security/reliability

**URL Parsing Patterns**:
- `twitch.tv/{CHANNEL}` → extract channel name from path
- `player.twitch.tv/?channel={CHANNEL}` → extract channel query param

**Twitch Chat Embed** (optional):
- `https://www.twitch.tv/embed/{CHANNEL}/chat?parent={PARENT}`
- Separate iframe if chat enabled

### 3. Twitch Parent Domain Configuration

**Question**: How to configure the required `parent` parameter for Twitch embeds in CEF?

**Decision**: Use configurable `ui.cef_embeds_twitch_parent` with default `localhost`.

**Rationale**:
- CEF loads local HTML files, origin varies by configuration
- `localhost` works for most local development scenarios
- Configurable allows users to adjust if needed

**Alternatives Considered**:
- Hardcoded `localhost` - Less flexible
- Dynamic detection from CEF - Complex, unreliable across platforms

### 4. CEF Navigation Interception

**Question**: How to intercept and block navigation in CEF iframes?

**Decision**: Use CEF's request handler (`OnBeforeResourceLoad` or `OnBeforeBrowse`) to intercept all navigation.

**Rationale**:
- CEF provides hooks at multiple levels (browser, frame, resource)
- `OnBeforeBrowse` fires for all navigation including iframe src changes
- Can return `RV_CANCEL` to block unauthorized navigation

**Implementation Pattern**:
```rust
// In CefShell or navigation guard
fn on_before_browse(&self, url: &str, is_redirect: bool) -> NavigationDecision {
    if self.whitelist.is_allowed(url) {
        NavigationDecision::Allow
    } else {
        tracing::warn!(url = %url, "Blocked navigation to non-whitelisted domain");
        NavigationDecision::Block
    }
}
```

**Alternatives Considered**:
- JavaScript-only interception - Bypassable, not secure
- Content Security Policy - Can help but not sufficient alone

### 5. Rate Limiting Strategy

**Question**: How to implement client-side rate limiting for embed load requests?

**Decision**: Use `Instant`-based cooldown tracker in `EmbedsManager`.

**Rationale**:
- Simple, no external dependencies
- Per-action tracking (not global)
- 2-second cooldown per spec

**Implementation Pattern**:
```rust
struct RateLimiter {
    last_action: Option<Instant>,
    cooldown: Duration,
}

impl RateLimiter {
    fn try_action(&mut self) -> Result<(), EmbedError> {
        let now = Instant::now();
        if let Some(last) = self.last_action {
            if now.duration_since(last) < self.cooldown {
                return Err(EmbedError::rate_limited());
            }
        }
        self.last_action = Some(now);
        Ok(())
    }
}
```

### 6. Input Focus State Integration

**Question**: How to integrate `EmbedFocus` with existing input focus system?

**Decision**: Extend `InputFocus` enum in `ui_cef/input.rs` with `EmbedFocus` variant.

**Rationale**:
- Consistent with existing `ChatTyping` variant (Feature 032)
- Single source of truth for input focus state
- Existing `blocks_gameplay()` pattern works

**Implementation Pattern**:
```rust
pub enum InputFocus {
    Game,
    ChatTyping,
    EmbedFocus,  // NEW
}

impl InputFocus {
    pub fn blocks_gameplay(&self) -> bool {
        matches!(self, Self::ChatTyping | Self::EmbedFocus)
    }

    pub fn is_embed_focus(&self) -> bool {
        matches!(self, Self::EmbedFocus)
    }
}
```

### 7. Spotify Stub Strategy

**Question**: How to stub Spotify support without blocking YouTube/Twitch release?

**Decision**: Implement full API surface with `provider_disabled` response.

**Rationale**:
- API contract defined and tested
- UI can show "Spotify coming soon" or hide option
- Toggle exists but defaults to `false`
- No dead code, just gated functionality

**Implementation Pattern**:
```rust
impl EmbedProvider {
    pub fn is_enabled(&self, config: &EmbedConfig) -> bool {
        match self {
            Self::YouTube => config.youtube_enabled,
            Self::Twitch => config.twitch_enabled,
            Self::Spotify => config.spotify_enabled, // defaults to false
        }
    }
}
```

## Summary

All technical unknowns resolved. Key decisions:
1. YouTube: `youtube-nocookie.com/embed/{ID}` for privacy
2. Twitch: `player.twitch.tv/?channel={CH}&parent=localhost` (configurable parent)
3. Navigation: CEF `OnBeforeBrowse` hook for whitelist enforcement
4. Rate limiting: `Instant`-based cooldown tracker
5. Focus: Extend existing `InputFocus` enum
6. Spotify: Full API, disabled by default
