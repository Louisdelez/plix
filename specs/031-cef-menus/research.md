# Research: CEF Menus

**Feature**: 031-cef-menus
**Date**: 2025-12-18

## Research Topics

### 1. JS↔Rust Bridge Pattern for CEF

**Decision**: Use JSON-based message passing with typed request/response pattern and correlation IDs.

**Rationale**:
- JSON is universally supported in both JS and Rust (serde_json)
- Correlation IDs enable async request/response without blocking
- Typed messages provide safety and documentation
- Push events support server-initiated updates (e.g., connection status)

**Alternatives Considered**:
- **Direct FFI bindings**: Rejected - too complex, unsafe, and tight coupling
- **MessagePack/CBOR**: Rejected - JSON is sufficient for UI messages, human-readable for debugging
- **WebSocket-style protocol**: Rejected - unnecessary complexity for in-process communication

**Implementation Pattern**:
```
JS → Rust: window.plix.send({ id: "req-1", type: "GetConfig", payload: {} })
Rust → JS: window.plix.onMessage({ id: "req-1", ok: true, payload: {...} })
Rust → JS (push): window.plix.onMessage({ id: null, type: "ConnectionStatus", payload: {...} })
```

### 2. CEF JavaScript Binding Mechanism

**Decision**: Use CEF's native JavaScript binding through `CefV8Handler` or equivalent in the chosen Rust CEF wrapper.

**Rationale**:
- CEF provides built-in mechanism for exposing native functions to JS
- Allows synchronous registration of `window.plix` object at page load
- More reliable than URL interception or custom schemes for message passing

**Alternatives Considered**:
- **Custom URL scheme interception**: Rejected - async-only, harder to correlate responses
- **File polling**: Rejected - inefficient, adds latency
- **WebSocket local server**: Rejected - unnecessary complexity, port conflicts possible

**Implementation Notes**:
- Register `window.plix.send()` as native function that queues messages to Rust
- Implement `window.plix.onMessage()` callback registration
- Ensure binding happens before page scripts execute

### 3. Favorites Persistence Format

**Decision**: Use TOML file at `~/.config/plix/favorites.toml` with simple address list.

**Rationale**:
- TOML is already used for other plix config (consistency)
- Simple format, human-editable
- Shared location allows native UI to read same favorites
- Serde integration already available

**Alternatives Considered**:
- **JSON**: Acceptable but less consistent with existing plix config
- **SQLite**: Overkill for simple address list
- **Binary format**: No benefit, harder to debug

**Format**:
```toml
version = 1
favorites = [
  "192.168.1.10:7777",
  "game.example.com:7777"
]
```

### 4. SPA Routing Pattern

**Decision**: Use hash-based routing (`#/settings`, `#/servers`) with vanilla JS.

**Rationale**:
- No external dependencies (no React/Vue/framework)
- Hash routing works with file:// protocol and CEF local pages
- Simple implementation: listen to `hashchange`, swap page content
- Matches minimal scope requirement (IX. Scoping)

**Alternatives Considered**:
- **History API routing**: Rejected - requires server configuration, complex with file://
- **Full SPA framework (React/Vue)**: Rejected - unnecessary complexity, large bundle size
- **Server-side rendering**: N/A - no server, local assets only

**Implementation Pattern**:
```javascript
window.addEventListener('hashchange', () => {
  const route = window.location.hash.slice(1) || '/';
  renderPage(route);
});
```

### 5. Error Code System

**Decision**: Use structured error codes with categories and user-friendly messages.

**Rationale**:
- Enables consistent error handling across UI
- Allows localization of messages in the future
- Facilitates logging and debugging

**Error Code Format**:
```
E[CATEGORY][NUMBER]: [User-friendly message]

Categories:
- ECON: Connection errors (ECON001 = timeout, ECON002 = refused, etc.)
- ECFG: Config errors (ECFG001 = validation failed, ECFG002 = save failed)
- ESRV: Server browser errors (ESRV001 = master unreachable, ESRV002 = empty list)
- EBRG: Bridge errors (EBRG001 = invalid message, EBRG002 = version mismatch)
```

### 6. Debounce and Rate Limiting

**Decision**: Implement client-side debounce for search (300ms) and button click debounce (500ms).

**Rationale**:
- Prevents accidental double-clicks and spam
- Reduces unnecessary bridge traffic
- Standard UX pattern for search inputs

**Implementation**:
- Search input: 300ms debounce before sending filter to list
- Buttons (Connect, Refresh): Disable during operation, re-enable on completion
- No server-side rate limiting needed (local operations)

### 7. Bridge Version Compatibility

**Decision**: Include `bridge_version` field in initial handshake, fail gracefully on mismatch.

**Rationale**:
- Prevents cryptic errors when UI assets are outdated
- Allows explicit upgrade path
- Simple semver-style versioning (major.minor)

**Implementation**:
- JS sends `{ type: "Handshake", payload: { bridge_version: "1.0" } }` on load
- Rust responds with `{ ok: true/false, payload: { supported_version: "1.0" } }`
- On mismatch: JS displays "UI version incompatible" and game falls back to native UI

## Summary

All technical decisions align with constitution principles:
- Simple JSON messaging (V. Code Quality, IX. Scoping)
- No external dependencies (VIII. Open Source)
- Documented protocol (VI. Technical Standards)
- Async operations (II. Performance)
- Graceful fallback (VII. Player Experience)

No NEEDS CLARIFICATION items remain. Ready for Phase 1 design.
