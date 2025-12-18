# Quickstart: CEF UI Shell

**Feature**: 030-cef-ui-shell
**Date**: 2025-12-18

## Overview

This guide explains how to use the optional CEF UI shell feature in plix-client.

## Prerequisites

- plix-client built with `cef-ui` feature
- CEF binaries present in distribution (bundled with game)
- Feature 005 (minimal-ui-native) as fallback

## Configuration

### Client Config File

Add to `~/.config/plix/config.toml`:

```toml
[ui]
cef_enabled = true          # Enable CEF UI (default: true)
cef_devtools = false        # Enable Chrome DevTools (default: false)
cef_initial_page = "index.html"  # Initial HTML page to load
cef_frame_rate = 60         # CEF frame rate limit (1-120)
```

## CLI Flags

| Flag | Description |
|------|-------------|
| `--cef-ui` | Enable CEF UI (overrides config) |
| `--no-cef-ui` | Disable CEF UI (overrides config) |
| `--cef-devtools` | Enable Chrome DevTools |

### Examples

```bash
# Run with CEF UI enabled (default if available)
./plix-client

# Run with CEF UI disabled (use native UI)
./plix-client --no-cef-ui

# Run with CEF DevTools enabled (for debugging)
./plix-client --cef-devtools
```

## Input Controls

### Focus Behavior

| Action | Result |
|--------|--------|
| Click on UI area | CEF UI receives focus |
| Press Escape | Focus returns to game |
| Click outside UI | Focus returns to game |

When CEF UI has focus:
- Mouse events go to CEF (clicks, scrolls, movement)
- Keyboard events go to CEF (typing, shortcuts)
- Game controls are disabled

When game has focus:
- All input goes to gameplay
- CEF UI is visible but not interactive

## Developer Tools

### Reload UI

Press **F6** to reload the current HTML page (when enabled).

### Chrome DevTools

1. Launch with `--cef-devtools`
2. Open Chrome and navigate to `chrome://inspect`
3. Click "inspect" on the plix CEF instance
4. Use standard Chrome DevTools for debugging

### Console Logging

CEF JavaScript console logs are forwarded to the game log:

```
[INFO] [CEF] Console: Hello from JavaScript!
[ERROR] [CEF] Console: Uncaught TypeError: ...
```

## HTML/CSS/JS Development

### Asset Location

HTML files are located in:
```
assets/ui/
├── index.html      # Initial page
├── styles/         # CSS files
├── scripts/        # JavaScript files
└── images/         # UI images
```

### Restrictions

- Only local files allowed (no external URLs)
- No network requests from CEF
- Single viewport only

### Example HTML

```html
<!DOCTYPE html>
<html>
<head>
    <title>Plix UI</title>
    <style>
        body {
            margin: 0;
            font-family: sans-serif;
            background: rgba(0, 0, 0, 0.5);
            color: white;
        }
        .menu {
            padding: 20px;
        }
        button {
            display: block;
            width: 200px;
            padding: 10px;
            margin: 10px 0;
            background: #4a90d9;
            border: none;
            color: white;
            cursor: pointer;
        }
        button:hover {
            background: #5ba0e9;
        }
    </style>
</head>
<body>
    <div class="menu">
        <h1>Main Menu</h1>
        <button onclick="alert('Resume')">Resume</button>
        <button onclick="alert('Settings')">Settings</button>
        <button onclick="alert('Quit')">Quit</button>
    </div>
</body>
</html>
```

## Fallback Behavior

If CEF is unavailable, the game automatically falls back to native UI:

| Scenario | Behavior |
|----------|----------|
| `cef_enabled = false` | Use native UI |
| `--no-cef-ui` flag | Use native UI |
| CEF init fails | Use native UI, log warning |
| CEF crashes at runtime | Switch to native UI, log error |
| CEF binaries missing | Use native UI, log warning |

The native UI (Feature 005) provides the same menu functionality.

## Performance

- Target: <2ms frame time overhead at 1080p
- CEF rendering pauses when game is minimized
- Texture updates synchronized with game frame rate

## Troubleshooting

### CEF doesn't initialize

Check:
1. CEF binaries are present in game directory
2. `cef_enabled` is `true` in config
3. No `--no-cef-ui` flag
4. Check logs for initialization errors

### UI not responding to input

Check:
1. Click on UI area to give it focus
2. Check if focus indicator shows "CEF UI"
3. Press Escape and try clicking again

### Performance issues

Try:
1. Reduce `cef_frame_rate` in config
2. Simplify HTML/CSS/JS
3. Disable DevTools if enabled

### DevTools not working

1. Ensure `--cef-devtools` flag is used
2. Check that CEF initialized successfully
3. Try `http://localhost:9222` directly
