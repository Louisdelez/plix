# UX Checklist: Accessibility

**Purpose**: Validate user experience considerations for accessibility features
**Created**: 2025-12-19
**Feature**: [spec.md](../spec.md)

## Keybinding Remapping UX

- [x] Clear visual indication of current bindings
- [x] "Listening" state clearly visible when capturing input
- [x] Conflict detection with actionable resolution (swap/cancel)
- [x] Reset to defaults easily accessible
- [x] Changes apply immediately without restart
- [x] Escape key cancels rebind capture
- [x] All rebindable actions shown with display names

## Visual Accessibility UX

- [x] UI Scale slider with live preview
- [x] FOV slider with live camera update
- [x] High Contrast mode immediately visible
- [x] Colorblind presets have descriptive names
- [x] Settings persist after restart
- [x] No modal dialogs required for basic adjustments

## Subtitle UX

- [x] Clear enable/disable toggle
- [x] Size options intuitive (Small/Medium/Large)
- [x] Background opacity adjustable for readability
- [x] Subtitles positioned to not obstruct gameplay
- [x] Auto-dismiss with reasonable default duration

## Native Fallback UX

- [x] Console commands documented with examples
- [x] `/rebind list` shows all current bindings
- [x] Error messages are helpful (invalid action, key names)
- [x] Commands match feature parity with CEF UI

## Accessibility Meta

- [x] Accessibility settings discoverable in main settings
- [x] Settings grouped logically (Controls, Display, Audio)
- [x] No accessibility feature requires another accessibility feature
- [x] Colorblind presets work with high contrast mode

## Notes

- CEF UI provides primary UX; native console is fallback
- Key capture uses inline editing (less modal, less disruptive)
- All changes preview live to support iterative adjustment
