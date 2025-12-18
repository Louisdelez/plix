# Feature Specification: CEF UI Shell (Optional)

**Feature Branch**: `030-cef-ui-shell`
**Created**: 2025-12-18
**Status**: Draft
**Input**: User description: "Optional CEF-based UI shell for rendering HTML/CSS/JS UI as GPU texture. This is a technical foundation only - not a full UI system."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Display HTML UI in Game (Priority: P1)

As a developer, I want to render HTML/CSS content as a GPU texture inside the game, so I can create rich UI elements using web technologies.

**Why this priority**: This is the core value proposition - enabling web-based UI rendering inside the game engine. Without this, CEF integration has no purpose.

**Independent Test**: Load a simple HTML page (e.g., a styled menu), render it to texture, display it in the game viewport. Verify the HTML renders correctly with CSS styling applied.

**Acceptance Scenarios**:

1. **Given** CEF is enabled and a valid HTML file exists, **When** the game loads, **Then** the HTML content renders as a GPU texture visible in the viewport
2. **Given** an HTML page with CSS styling, **When** rendered via CEF, **Then** all CSS properties (fonts, colors, layouts) display correctly
3. **Given** an HTML page with JavaScript, **When** rendered via CEF, **Then** JavaScript executes and modifies the DOM as expected
4. **Given** the game window resizes, **When** the resize event occurs, **Then** the CEF texture updates to match the new dimensions

---

### User Story 2 - Input Focus Handling (Priority: P1)

As a player, I want to interact with HTML UI elements using keyboard and mouse, so I can click buttons, type in inputs, and navigate menus.

**Why this priority**: UI without input is useless. Players must be able to interact with rendered HTML for it to serve as a functional UI system.

**Independent Test**: Display an HTML form with buttons and text inputs. Click a button, type in a text field, verify the interactions register correctly in the HTML/JS layer.

**Acceptance Scenarios**:

1. **Given** a CEF UI is displayed with a clickable button, **When** the player clicks the button, **Then** the button's onclick handler fires
2. **Given** a CEF UI is displayed with a text input, **When** the player types on keyboard, **Then** text appears in the input field
3. **Given** game input and CEF input could conflict, **When** CEF has focus, **Then** keyboard input goes to CEF (not game controls)
4. **Given** the player presses Escape or clicks outside UI, **When** focus should return to game, **Then** CEF releases input focus

---

### User Story 3 - Optional/Fallback Mode (Priority: P2)

As a player on a system without CEF support, I want the game to work without CEF, so I can still play even if CEF is unavailable or disabled.

**Why this priority**: CEF should enhance, not block. The game must remain playable without CEF, using fallback native UI if available.

**Independent Test**: Launch the game with `--no-cef-ui` flag or on a system without CEF libraries. Verify the game starts normally and uses fallback UI.

**Acceptance Scenarios**:

1. **Given** CEF is disabled via config or flag, **When** the game launches, **Then** the game starts without CEF and uses native UI fallback
2. **Given** CEF libraries are missing or fail to initialize, **When** the game attempts to start CEF, **Then** it gracefully falls back to native UI
3. **Given** a runtime error occurs in CEF, **When** the error is detected, **Then** the game continues running with native UI fallback
4. **Given** the user has configured `cef_enabled = false`, **When** the game reads config, **Then** CEF is not initialized

---

### User Story 4 - Engine Integration (Priority: P2)

As a developer, I want CEF integrated cleanly with the wgpu rendering pipeline, so I can composite HTML UI over the 3D game world without performance issues.

**Why this priority**: Integration quality determines whether CEF is usable in production. Poor integration (flickering, latency, performance) makes the feature unusable.

**Independent Test**: Run the game with CEF UI overlay at 60fps. Measure frame time impact, verify no visual artifacts or flickering, confirm UI updates synchronize with render frames.

**Acceptance Scenarios**:

1. **Given** CEF is rendering a moderately complex UI, **When** measuring frame time, **Then** CEF adds less than 2ms per frame at 1080p
2. **Given** CEF texture needs updating, **When** the CEF frame is ready, **Then** texture upload happens without stalling the render thread
3. **Given** the game is rendering 3D content with CEF overlay, **When** displaying both, **Then** CEF content composites correctly with proper alpha blending
4. **Given** the game resolution changes, **When** resize occurs, **Then** CEF texture resizes appropriately without memory leaks

---

### User Story 5 - Debug and Development (Priority: P3)

As a developer, I want to use CEF DevTools to debug HTML/CSS/JS, so I can develop and troubleshoot UI issues efficiently.

**Why this priority**: Developer experience - without debugging tools, developing complex UIs becomes extremely difficult.

**Independent Test**: Launch with `--cef-devtools` flag, connect to the DevTools port, inspect DOM elements, set JavaScript breakpoints.

**Acceptance Scenarios**:

1. **Given** the game is launched with `--cef-devtools`, **When** connecting to the DevTools port, **Then** Chrome DevTools shows the rendered page
2. **Given** DevTools is connected, **When** inspecting DOM elements, **Then** elements are highlighted in the CEF viewport
3. **Given** DevTools is connected, **When** modifying CSS live, **Then** changes reflect immediately in the CEF viewport
4. **Given** JavaScript errors occur, **When** DevTools is connected, **Then** errors appear in the DevTools console

---

### Edge Cases

- What happens when CEF crashes mid-game? → Detect crash, log error, switch to native fallback, continue game
- What happens when HTML page has infinite loop? → CEF runs in separate process, game remains responsive, may need timeout/kill
- What happens when CEF texture exceeds GPU memory? → Limit texture size to reasonable bounds (e.g., 4K max)
- What happens when HTML references external URLs? → v1 restricts to local files only (no network requests from CEF)
- What happens with multiple CEF views simultaneously? → Out of scope for v1, single viewport only
- What happens when game is minimized? → Pause CEF rendering to save resources

## Requirements *(mandatory)*

### Functional Requirements

**CEF Integration**

- **FR-001**: System MUST use Chromium Embedded Framework (CEF) for HTML/CSS/JS rendering
- **FR-002**: System MUST render CEF content off-screen (not in a separate window)
- **FR-003**: System MUST provide CEF rendered frames as a GPU texture compatible with wgpu
- **FR-004**: System MUST support loading local HTML files from the assets directory
- **FR-005**: System MUST NOT allow CEF to load external URLs (security restriction for v1)
- **FR-006**: System MUST initialize CEF in a separate subprocess (standard CEF architecture)

**Input Handling**

- **FR-007**: System MUST forward mouse events (move, click, scroll) to CEF when UI has focus
- **FR-008**: System MUST forward keyboard events to CEF when UI has focus
- **FR-009**: System MUST use click-to-focus: mouse click on CEF UI area gives CEF input focus
- **FR-010**: System MUST support Escape key to release CEF focus (return to game)
- **FR-011**: System MUST prevent game input processing while CEF has focus

**Fallback & Optional**

- **FR-012**: CEF integration MUST be optional (compile-time feature flag)
- **FR-013**: System MUST gracefully handle CEF initialization failure
- **FR-014**: System MUST fall back to Feature 005 native UI system when CEF is disabled or unavailable
- **FR-015**: System MUST support runtime disable via configuration (`cef_enabled = false`)
- **FR-016**: System MUST log CEF availability status on startup

**Rendering Integration**

- **FR-017**: CEF texture MUST be composited over the 3D game world (overlay mode)
- **FR-018**: CEF texture MUST support alpha transparency for non-rectangular UI
- **FR-019**: System MUST update CEF texture synchronously with render frame
- **FR-020**: System MUST resize CEF viewport when game window resizes
- **FR-021**: CEF rendering MUST NOT block the main render thread

**Configuration**

- **FR-022**: Configuration MUST include `cef_enabled` boolean (default: true if available)
- **FR-023**: Configuration MUST include `cef_devtools` boolean (default: false)
- **FR-024**: Configuration MUST include `cef_initial_page` string (path to initial HTML)
- **FR-025**: Configuration MUST be in the existing client config file

**CLI Support**

- **FR-026**: System MUST support `--cef-ui` flag to enable CEF (override config)
- **FR-027**: System MUST support `--no-cef-ui` flag to disable CEF (override config)
- **FR-028**: System MUST support `--cef-devtools` flag to enable DevTools

**Debug & Development**

- **FR-029**: System MUST support CEF DevTools when enabled via flag/config
- **FR-030**: System MUST log CEF subprocess startup and shutdown
- **FR-031**: System MUST log JavaScript errors from CEF to the game log

**Performance**

- **FR-032**: CEF frame rate SHOULD match game frame rate (or be configurable)
- **FR-033**: CEF texture upload MUST use efficient GPU transfer (no CPU readback)
- **FR-034**: System MUST pause CEF rendering when game is minimized/unfocused

### Non-Functional Requirements

- **NFR-001**: CEF integration MUST add less than 2ms frame time at 1080p with typical UI
- **NFR-002**: CEF subprocess memory SHOULD be limited to reasonable bounds (e.g., 256MB)
- **NFR-003**: CEF binaries MUST be distributable (proper CEF licensing compliance)

### Key Entities

- **CefShell**: Main integration component - manages CEF subprocess, texture updates, input routing
- **CefTexture**: GPU texture containing the latest CEF rendered frame, updated each frame
- **CefConfig**: Configuration struct for CEF options (enabled, devtools, initial_page)
- **InputFocus**: State machine tracking whether game or CEF should receive input

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: HTML content renders correctly in the game viewport (visual verification)
- **SC-002**: CSS styling (fonts, colors, flexbox layouts) renders correctly
- **SC-003**: JavaScript executes and modifies the DOM as expected
- **SC-004**: Mouse clicks trigger HTML button onclick handlers
- **SC-005**: Keyboard input populates HTML text fields when CEF has focus
- **SC-006**: Game runs normally when CEF is disabled or unavailable
- **SC-007**: CEF adds less than 2ms per frame at 1080p resolution
- **SC-008**: DevTools can connect and inspect the rendered page
- **SC-009**: No memory leaks after 1 hour of continuous operation with UI
- **SC-010**: CEF crashes are caught and logged without crashing the game

## Clarifications

### Session 2025-12-18

- Q: Which Rust CEF binding approach should be used? → A: Evaluate at planning phase (spike first to assess binding maturity)
- Q: How does the player give focus to CEF UI? → A: Mouse click on CEF UI area (click-to-focus)
- Q: What is the native UI fallback when CEF unavailable? → A: Use existing Feature 005 native UI system
- Q: How should CEF binaries be distributed? → A: Bundle with main game download

## Assumptions

- CEF Rust binding approach to be determined via planning spike (options: existing binding like cef-rs, or minimal FFI wrapper)
- Feature 005 (minimal-ui-native) is a prerequisite - provides fallback UI when CEF unavailable
- CEF binaries will be bundled with main game download (not separate optional download)
- Only local HTML files are loaded (no network requests from CEF in v1)
- Single CEF viewport/view for v1 (no multiple simultaneous views)
- wgpu is the rendering backend (already established in the codebase)
- CEF subprocess architecture is used (not single-process mode)
- Initial HTML page is a simple test/placeholder for v1

## Out of Scope

- Full UI system built on CEF (this is just the technical foundation)
- Multiple simultaneous CEF viewports
- Network requests from CEF (e.g., loading remote pages)
- CEF audio output (game has its own audio system)
- Bidirectional JS/Rust communication (beyond simple events)
- Custom CEF extensions or plugins
- CEF cache/storage persistence
- macOS support (can be added later)
- Self-updating CEF binaries
