# Research: CEF UI Shell

**Date**: 2025-12-18
**Feature**: 030-cef-ui-shell
**Purpose**: Evaluate CEF Rust binding options and determine implementation approach

## Executive Summary

After evaluating available CEF Rust bindings and alternatives, the recommended approach is to:

1. **Primary**: Attempt integration with [cef-ui](https://github.com/hytopiagg/cef-ui) (most comprehensive)
2. **Fallback**: Create minimal FFI wrapper around CEF C API if existing bindings prove inadequate
3. **Alternative**: Consider [Servo](https://servo.org/) as future option (pure Rust, but less mature embedding)

A spike is mandatory before committing to any approach due to maturity concerns with all options.

## Research Questions

### Q1: Which Rust CEF bindings exist and what is their status?

**Findings**:

| Binding | Status | CEF Version | OSR Support | Notes |
|---------|--------|-------------|-------------|-------|
| [cef-ui](https://github.com/hytopiagg/cef-ui) | Active (WIP) | 121.3.15 | Unknown | Most comprehensive, proper Rust types |
| [rust-cef](https://github.com/Julusian/rust-cef) | **Archived** (Nov 2023) | 75.0.13 | Unknown | No longer maintained |
| [cef-rs (space-07)](https://github.com/space-07/cef-rs) | Unmaintained | 116.0.22 | Unknown | Personal project, no support |
| [cef-rs (dylanede)](https://github.com/dylanede/cef-rs) | Stale | Old | Unknown | Not recently updated |
| [cef-sys](https://crates.io/crates/cef-sys) | Failed build | - | - | Raw bindgen, docs.rs build failed |

**Conclusion**: No mature, production-ready CEF Rust binding exists. All options are work-in-progress or abandoned.

### Q2: Does any binding support Off-Screen Rendering (OSR)?

**Findings**: None of the evaluated bindings explicitly document OSR support. The CEF C API supports OSR (windowless mode), but Rust bindings may not wrap these APIs yet.

**Implication**: The spike must verify OSR capability regardless of which binding is chosen.

### Q3: What is the CEF C API approach?

**CEF Architecture**:
- CEF provides a C API (`cef_capi.h`) in addition to C++
- The C API uses reference counting (CefRefPtr pattern)
- OSR requires implementing `CefRenderHandler` with `OnPaint` callback
- `OnPaint` provides BGRA pixel buffer (not RGBA)

**FFI Wrapper Approach**:
```rust
// Minimal FFI types needed for OSR:
// - cef_browser_host_create_browser_sync
// - cef_browser_settings_t (windowless_frame_rate)
// - cef_window_info_t (windowless_rendering_enabled = true)
// - CefRenderHandler::OnPaint (buffer, width, height, dirtyRects)
```

**Estimated Effort**: 2-3 weeks for minimal FFI wrapper supporting OSR only.

### Q4: Are there pure-Rust alternatives to CEF?

**[Servo](https://servo.org/)** - Rust web engine:

| Aspect | Status |
|--------|--------|
| Language | Pure Rust |
| Maturity | Production transition in progress (2024) |
| Embedding API | WebView API available |
| OSR Support | Unknown (needs investigation) |
| Binary Size | Potentially smaller than CEF |
| Platforms | Linux, Windows, macOS, Android |

**Advantages**:
- Pure Rust = better integration
- No C/C++ interop complexity
- Actively developed (Linux Foundation Europe)

**Disadvantages**:
- Less mature than CEF
- Fewer documented embedding examples
- May lack full web compatibility

**Recommendation**: Servo is a future option worth tracking, but CEF is safer for MVP due to wider adoption.

## Decision: Implementation Approach

### Chosen Approach: Spike-First Evaluation

**Rationale**: Given the immaturity of all CEF Rust bindings, a spike is mandatory before committing.

**Spike Plan**:

1. **Week 1**: Attempt [cef-ui](https://github.com/hytopiagg/cef-ui) integration
   - Build on Linux x86_64
   - Test OSR initialization
   - Test paint callback

2. **Week 1 (parallel)**: Prepare FFI wrapper skeleton
   - Generate bindings with bindgen
   - Implement CefRefPtr wrapper

3. **Week 2**: Based on Week 1 results
   - If cef-ui works → proceed with it
   - If cef-ui fails → switch to FFI wrapper

**Decision Gate**: End of spike must produce:
- Working CEF initialization in OSR mode
- Paint callback receiving BGRA buffer
- Clean shutdown without crashes

### Alternatives Considered

| Alternative | Rejected Because |
|-------------|------------------|
| Wait for mature binding | No timeline, blocks feature indefinitely |
| Use electron/tauri | Too heavyweight, different architecture |
| Use Servo now | Embedding maturity uncertain |
| Skip CEF feature | User requested CEF specifically |

## Technical Details

### CEF Off-Screen Rendering Flow

```
1. Create CefApp with CefRenderHandler implementation
2. Initialize CEF with windowless settings
3. Create browser with windowless_rendering_enabled = true
4. CefRenderHandler::OnPaint called when content changes
5. OnPaint receives: buffer (BGRA), width, height, dirtyRects
6. Convert BGRA → RGBA (or use BGRA texture format)
7. Upload to wgpu texture
```

### wgpu Integration Points

```rust
// Texture format options:
// - wgpu::TextureFormat::Bgra8Unorm (matches CEF, no conversion)
// - wgpu::TextureFormat::Rgba8Unorm (requires BGRA→RGBA swizzle)

// Texture update path:
// 1. CEF OnPaint → BGRA buffer
// 2. queue.write_texture() → GPU upload
// 3. Render as fullscreen quad in UI pass
```

### Platform Considerations

| Platform | CEF Subprocess | Notes |
|----------|----------------|-------|
| Linux | Required | Separate helper process |
| Windows | Required | Separate .exe |
| macOS | Required + Bundle | App bundle structure required |

For MVP, focus on Linux/Windows. macOS is out of scope.

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| cef-ui doesn't support OSR | Medium | High | FFI wrapper as fallback |
| CEF init crashes on exit | Medium | Medium | Proper ref counting, testing |
| Paint callback performance | Low | Medium | Dirty rect optimization |
| CEF binary size (~100-200MB) | Certain | Low | Document clearly, optional feature |

## Next Steps

1. Create spike branch `spike/030-cef-binding`
2. Test cef-ui crate with OSR
3. Document findings
4. Proceed to implementation or FFI wrapper

## Sources

- [cef-ui (hytopiagg)](https://github.com/hytopiagg/cef-ui) - Most comprehensive Rust CEF binding
- [rust-cef (Julusian)](https://github.com/Julusian/rust-cef) - Archived, no longer maintained
- [cef-rs (space-07)](https://github.com/space-07/cef-rs) - Personal unmaintained project
- [cef-sys crate](https://crates.io/crates/cef-sys) - Raw bindgen bindings
- [Servo](https://servo.org/) - Pure Rust web engine, CEF alternative
- [Servo GitHub](https://github.com/servo/servo) - Source and documentation
- [wgpu](https://wgpu.rs/) - Rust GPU library used for rendering
