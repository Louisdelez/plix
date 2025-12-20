# Getting Started with Plix

Welcome to Plix! This guide will help you get into the game quickly.

## Launching the Game

```bash
# Start the client
./plix-client

# Connect to a specific server
./plix-client --server 192.168.1.10:7777 --name "YourName"
```

## Controls

### Movement
| Action | Default Key |
|--------|-------------|
| Move Forward | W |
| Move Backward | S |
| Strafe Left | A |
| Strafe Right | D |
| Jump | Space |
| Crouch | Ctrl |

### Combat
| Action | Default Key |
|--------|-------------|
| Attack | Left Mouse Button |
| Place Block | Right Mouse Button |

### Interface
| Action | Default Key |
|--------|-------------|
| Pause Menu | Escape |
| Toggle Debug Overlay | F3 |

All keybinds can be customized in Settings > Keybinds.

## Basic Gameplay

### Connecting to a Server

1. Launch `plix-client`
2. Open the Pause Menu (Escape)
3. Select "Servers" to browse available servers
4. Select a server and press Enter to connect

Or connect directly via command line:
```bash
./plix-client --server myserver.example.com:7777
```

### Game Modes

Plix supports multiple game modes:

- **Free-for-All (FFA)**: Every player for themselves
- **Team Deathmatch (TDM)**: Team-based combat
- **Capture the Flag (CTF)**: Capture the enemy flag
- **Battle Royale Lite**: Shrinking zone, last player standing
- **Training**: Practice against bots

### Building and Destruction

- **Left Mouse Button**: Remove blocks you're looking at
- **Right Mouse Button**: Place blocks adjacent to what you're looking at

The crosshair shows your aim point. Blocks can only be placed/removed within 5 units.

### Combat

- Attack with Left Mouse Button
- Players have 100 health points
- Melee attacks deal damage on hit
- Watch for hit confirmation indicators in the HUD

### HUD Elements

The window title displays:
- FPS: Current frames per second
- Ping: Network latency to server
- HP: Your health
- ID: Your player ID
- Phase: Current match phase

## Player Profile

Your display name is stored in `~/.config/plix/profile.toml`. Edit this file or use the command line:

```bash
./plix-client --name "NewName"
```

## Tips for New Players

1. **Practice movement first** - Get comfortable with WASD and mouse look
2. **Use cover** - Build blocks to create defensive positions
3. **Watch your health** - Damage taken shows briefly in the HUD
4. **Check your ping** - High latency affects combat responsiveness
5. **Explore the arena** - Know spawn points and key locations

## Next Steps

- [Settings](settings.md) - Customize controls, graphics, and audio
- [FAQ](faq.md) - Common questions answered
- [Server Installation](../server/installation.md) - Host your own server
