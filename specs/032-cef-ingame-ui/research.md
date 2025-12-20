# Research: CEF In-Game UI (HUD, Chat, Scoreboard)

**Feature**: 032-cef-ingame-ui
**Date**: 2025-12-18

## Research Topics

### 1. Input Focus State Machine Extension

**Question**: How should input focus states be extended for chat typing vs scoreboard viewing?

**Decision**: Extend `InputFocus` enum to include `ChatTyping` state while keeping `ScoreboardViewing` as a separate overlay flag (not a focus state).

**Rationale**:
- Chat requires exclusive keyboard capture (blocks gameplay input)
- Scoreboard is read-only overlay (doesn't affect input routing)
- Existing `Game`/`CefUI` distinction in `InputFocus` already handles menu vs gameplay
- Adding `ChatTyping` variant creates clear three-state model: `Game` → `ChatTyping` → `Game`
- Scoreboard visibility is a boolean flag, not a focus state

**Alternatives Considered**:
1. Add both `ChatTyping` and `ScoreboardViewing` as focus states - Rejected: scoreboard doesn't capture input
2. Use single `CefUI` state for all UI - Rejected: menu UI vs in-game UI have different input behaviors
3. Separate focus system for in-game - Rejected: adds complexity, existing system is extensible

**Implementation**:
```rust
pub enum InputFocus {
    Game,        // Gameplay input active
    CefUI,       // Menu UI active (from F030/F031)
    ChatTyping,  // Chat input active, gameplay blocked
}
```

### 2. Chat Protocol Extension

**Question**: How should chat messages be added to the client-server protocol?

**Decision**: Extend `ClientMessage` and `ServerMessage` enums with chat variants; add `GameEvent::ChatReceived` for broadcasts.

**Rationale**:
- Follows existing protocol patterns (messages.rs in plix-common)
- Server-authoritative: client sends text, server validates and broadcasts
- Reuses existing reliable message delivery (GameEvent pattern)
- Minimal protocol changes (2 new client messages, 1 new server event)

**Alternatives Considered**:
1. Separate UDP channel for chat - Rejected: reliability needed, existing reliable events suffice
2. Chat as separate service - Rejected: over-engineering for MVP
3. Piggyback on snapshot - Rejected: chat is event-driven, not state

**Implementation**:
```rust
// Client → Server
pub enum ClientMessage {
    // ... existing ...
    ChatSend { text: String },
}

// Server → Client (via GameEvent)
pub enum GameEvent {
    // ... existing ...
    ChatReceived {
        sender_id: PlayerId,
        sender_name: String,
        text: String,
        kind: ChatMessageKind,
        timestamp: u64,
    },
}

pub enum ChatMessageKind {
    Player,  // Normal player message
    System,  // Server announcement, join/leave
}
```

### 3. HUD State Publishing Strategy

**Question**: How should HUD data be published to the CEF UI without impacting frame rate?

**Decision**: Use throttled publisher with 15 Hz base rate and change-detection for HP.

**Rationale**:
- 15 Hz is sufficient for human perception of numeric values
- HP changes need immediate feedback (damage taken) - trigger instant update
- RTT and FPS can update at lower frequency (statistical values)
- Avoids per-frame JSON serialization overhead

**Alternatives Considered**:
1. Per-frame updates - Rejected: unnecessary serialization overhead
2. Change-only updates - Rejected: RTT changes constantly, need periodic baseline
3. Separate timers per value - Rejected: adds complexity vs single throttle

**Implementation**:
```rust
pub struct HudStatePublisher {
    last_publish: Instant,
    last_state: HudState,
    min_interval: Duration,  // 66ms = ~15 Hz
}

impl HudStatePublisher {
    pub fn maybe_publish(&mut self, current: &HudState) -> Option<HudState> {
        let now = Instant::now();
        let hp_changed = current.hp != self.last_state.hp;
        let interval_elapsed = now.duration_since(self.last_publish) >= self.min_interval;

        if hp_changed || interval_elapsed {
            self.last_publish = now;
            self.last_state = current.clone();
            Some(current.clone())
        } else {
            None
        }
    }
}
```

### 4. Bridge Message Types for In-Game UI

**Question**: What new bridge message types are needed for in-game UI?

**Decision**: Add 8 new message types following Feature 031 patterns.

**Rationale**:
- Extends existing `MessageType` enum and handlers
- Maintains versioned, typed contract
- Separates concerns (HUD state push vs chat send request)

**UI → Game Messages**:
| Type | Payload | Purpose |
|------|---------|---------|
| `ChatSend` | `{ text: string }` | Send chat message to server |
| `ChatOpen` | `{}` | Notify game that chat input opened |
| `ChatClose` | `{}` | Notify game that chat input closed |
| `ChatClear` | `{}` | Clear local chat history |

**Game → UI Messages (Push)**:
| Type | Payload | Purpose |
|------|---------|---------|
| `HudState` | `{ hp, max_hp, rtt_ms, fps? }` | HUD data update |
| `ChatMessage` | `{ author, text, kind, timestamp }` | Received chat message |
| `ChatToast` | `{ author, text }` | Toast notification when chat closed |
| `ScoreboardState` | `{ server_name, rows: [...] }` | Scoreboard data |
| `UiConfig` | `{ cefHudEnabled, keybinds }` | UI configuration |

### 5. Scoreboard Data Source

**Question**: Where does scoreboard data come from?

**Decision**: Derive from existing `WorldSnapshot.match_state.player_scores` and `WorldSnapshot.players`.

**Rationale**:
- Data already exists in snapshots (name, kills, deaths)
- Ping is available per-player from RTT tracking
- No new server messages needed for basic scoreboard
- Team info available from existing match state

**Implementation**:
```rust
pub struct ScoreboardClient {
    visible: bool,
    cached_state: Option<ScoreboardState>,
}

impl ScoreboardClient {
    pub fn update_from_snapshot(&mut self, snapshot: &WorldSnapshot, ping_map: &HashMap<PlayerId, u32>) {
        if !self.visible { return; }  // Don't update when hidden

        let rows: Vec<ScoreboardRow> = snapshot.match_state.player_scores.iter()
            .map(|ps| ScoreboardRow {
                name: ps.name.clone(),
                ping_ms: ping_map.get(&ps.player_id).copied().unwrap_or(0),
                score: Some(ps.kills),
                kills: Some(ps.kills),
                deaths: Some(ps.deaths),
                team: /* derive from snapshot */,
            })
            .take(64)  // Cap at 64 players
            .collect();

        self.cached_state = Some(ScoreboardState {
            server_name: snapshot.match_state.arena_name.clone(),
            rows,
        });
    }
}
```

### 6. Native Fallback Strategy

**Question**: How extensive should native fallback UI be?

**Decision**: Minimal but functional: text-based chat input/log, simple scoreboard list, reuse existing native HUD.

**Rationale**:
- Native HUD already exists (Feature 005) - just ensure it's used when CEF HUD disabled
- Native chat: simple text input field + scrollable log using existing UI rendering
- Native scoreboard: simple list rendering using existing text/rect primitives
- Parity on functionality, not aesthetics

**Alternatives Considered**:
1. Full feature parity with styled native UI - Rejected: too much effort, defeats CEF purpose
2. No fallback (require CEF) - Rejected: violates accessibility requirement
3. Partial fallback (HUD only) - Rejected: chat/scoreboard are essential

**Implementation Approach**:
- `ChatNative`: Text input line + VecDeque<ChatMessage> log + basic rendering
- `ScoreboardNative`: Simple list with name | ping | score columns
- Runtime switch based on `CefShell::should_fallback()` status

### 7. Toast Notification Behavior

**Question**: How should chat toast notifications work?

**Decision**: Rust-side sends `ChatToast` push to UI when chat is closed; UI handles fade animation.

**Rationale**:
- Rust knows chat open/close state (via `ChatOpen`/`ChatClose` messages)
- UI handles animation (CSS/JS natural for fades)
- Toast queue of 3 with auto-dismiss after 3 seconds

**Implementation**:
- Game sends `ChatToast { author, text }` when message received and chat is closed
- UI JS maintains toast queue, renders with CSS animation
- Toast does not steal focus (click-through)

## Dependencies Validated

| Dependency | Version | Purpose | Status |
|------------|---------|---------|--------|
| serde_json | ^1.0 | Bridge serialization | ✅ Already in use |
| wgpu | 23.0 | Rendering (existing) | ✅ Already in use |
| winit | existing | Input handling | ✅ Already in use |

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| CEF stability issues | Medium | Medium | Native fallback always available |
| Input focus race conditions | Low | High | Explicit state machine, tests for transitions |
| Chat spam/abuse | Medium | Low | Client rate limit + server authority |
| Performance impact from HUD updates | Low | Medium | Throttled publisher, profiling |

## Open Questions Resolved

All questions from Technical Context resolved. No NEEDS CLARIFICATION remaining.

## Next Steps

Proceed to Phase 1: Design & Contracts
- Generate data-model.md with entity definitions
- Generate contracts/ with bridge message schemas
- Generate quickstart.md with development setup
