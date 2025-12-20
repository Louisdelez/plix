# Security Checklist: Accessibility

**Purpose**: Validate security considerations for accessibility features
**Created**: 2025-12-19
**Feature**: [spec.md](../spec.md)

## Input Validation

- [x] UI Scale clamped to valid range (75-150)
- [x] FOV clamped to valid range (60-110)
- [x] Colorblind preset validated against enum (not arbitrary string)
- [x] Subtitle duration clamped (1000-10000ms)
- [x] Background opacity clamped (0-100)
- [x] Action names validated against Action enum
- [x] Key names validated against Key enum

## Config Persistence

- [x] Config file written atomically (temp file + rename)
- [x] Invalid config values clamped, not rejected (denial of service)
- [x] Config path uses platform-appropriate directory
- [x] No path traversal possible in config values

## CEF Bridge Security

- [x] Rebind messages validate action exists
- [x] Rebind messages validate key exists
- [x] No arbitrary code execution via settings
- [x] CSS filter values are constants, not user-provided
- [x] CSS class names are constants, not user-provided

## Console Command Security

- [x] Commands validate all parameters
- [x] No shell injection possible (not using system shell)
- [x] Error messages don't leak internal paths
- [x] Invalid input rejected with safe defaults

## Privacy

- [x] No keybinding data sent to server
- [x] No accessibility preferences sent to server
- [x] All settings stored locally only
- [x] No analytics or telemetry for accessibility usage

## Accessibility as Security

- [x] Accessibility features cannot be used to cheat
- [x] High contrast doesn't reveal hidden game elements
- [x] Colorblind modes use standard simulation (not advantage)
- [x] UI scale doesn't provide zoom advantage (affects UI only)

## Notes

- Accessibility features are purely client-side
- No server interaction required for any setting
- All validation happens before persistence
- CSS filters use SVG filter IDs, not arbitrary filter strings
