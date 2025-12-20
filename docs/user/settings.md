# Plix Settings

This guide covers all configurable settings in Plix.

## Accessing Settings

1. Press Escape to open the Pause Menu
2. Select "Settings"
3. Use Up/Down arrows to navigate, Left/Right to adjust values
4. Press Escape to go back (changes save automatically)

## Mouse Settings

### Sensitivity
- **Range**: 0.0001 - 0.01
- **Default**: 0.003
- **Adjustment step**: 0.0005

Higher sensitivity means faster mouse movement. Adjust until aiming feels comfortable.

## Graphics Settings

### Field of View (FOV)
- **Range**: 60 - 110 degrees
- **Default**: 90
- **Adjustment step**: 5 degrees

Higher FOV shows more peripheral vision but may cause distortion at extreme values.

### Fullscreen
- **Options**: On / Off
- **Default**: Off

Toggle between fullscreen and windowed mode. Uses borderless fullscreen on the primary monitor.

## Audio Settings

### Mute
- **Options**: On / Off
- **Default**: Off

Toggles all game audio. Use when you need to focus or are in a call.

## Keybinds

Access through Settings > Keybinds.

### Rebinding Keys

1. Navigate to the action you want to rebind
2. Press Enter to start capture
3. Press the new key or mouse button
4. If the key conflicts with another action, confirm the swap

### Available Actions

| Action | Default Binding |
|--------|-----------------|
| Move Forward | W |
| Move Backward | S |
| Strafe Left | A |
| Strafe Right | D |
| Jump | Space |
| Crouch | Left Ctrl |
| Attack | Left Mouse Button |
| Place Block | Right Mouse Button |

### Mouse Button Support

You can bind actions to:
- Left Mouse Button
- Right Mouse Button
- Middle Mouse Button
- Mouse Button 4 (side button)
- Mouse Button 5 (side button)

## Configuration File

Settings are stored in `~/.config/plix/config.toml`:

```toml
config_version = "1.0.0"
sensitivity = 0.003
fov_degrees = 90.0
fullscreen = false
audio_muted = false

[keybinds]
forward = "W"
backward = "S"
strafe_left = "A"
strafe_right = "D"
jump = "Space"
crouch = "LeftCtrl"
attack = "MouseLeft"
place_block = "MouseRight"

[ui]
enabled = true
devtools = false
```

### Manual Editing

You can edit the config file directly. Changes take effect on next launch.

### Resetting to Defaults

Delete or rename your config file to reset all settings:

```bash
rm ~/.config/plix/config.toml
```

## Command Line Overrides

Some settings can be overridden via command line:

```bash
# Force windowed mode
./plix-client --no-fullscreen

# Set custom player name
./plix-client --name "MyName"

# Set log verbosity
./plix-client --log-level debug

# Enable CEF DevTools (for UI debugging)
./plix-client --cef-devtools
```

## Performance Tips

### For Low-End Systems
- Reduce FOV to 75
- Play in windowed mode at lower resolution
- Close background applications

### For High Refresh Rate Monitors
- The game runs at your monitor's refresh rate
- Consider higher sensitivity for faster response

## Troubleshooting

### Settings not saving
- Check file permissions on `~/.config/plix/`
- Ensure the config file isn't read-only

### Keybind not working
- Check for conflicts in the keybinds menu
- Some keys may be reserved by the system

### Mouse feels off
- Adjust sensitivity in small increments
- Try enabling/disabling fullscreen mode
