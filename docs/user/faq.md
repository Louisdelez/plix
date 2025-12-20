# Frequently Asked Questions

## General

### What is Plix?
Plix is a multiplayer voxel game platform featuring combat, building, and destruction in a 3D block-based world.

### What platforms does Plix support?
Plix runs on:
- Windows 10/11 (64-bit)
- macOS 11+ (Intel and Apple Silicon)
- Linux (Ubuntu 22.04+, other modern distributions)

### Is Plix free?
Plix is open-source software released under the MIT license. You can download, modify, and host servers for free.

### What are the system requirements?
See [Installation Guide](installation.md#system-requirements) for detailed requirements.

## Gameplay

### How do I join a server?
1. Launch `plix-client`
2. Press Escape to open the menu
3. Select "Servers" to browse
4. Select a server and press Enter

Or use the command line: `./plix-client --server 1.2.3.4:7777`

### How do I host my own server?
See [Server Installation](../server/installation.md) for complete instructions.

Basic quick start:
```bash
./plix-server --port 7777 --arena test_arena
```

### What game modes are available?
- **FFA (Free-for-All)**: Everyone fights everyone
- **TDM (Team Deathmatch)**: Team-based combat
- **CTF (Capture the Flag)**: Capture enemy flag and defend yours
- **BR Lite (Battle Royale)**: Shrinking zone, last survivor wins
- **Training**: Practice against bots

### How does combat work?
- Left-click to attack
- Melee attacks deal damage to nearby players
- All players have 100 HP
- HUD shows health and combat feedback

### How do I build and destroy blocks?
- Left-click: Remove block you're looking at
- Right-click: Place block adjacent to what you're looking at
- Range limit: 5 blocks

## Technical

### Why is my ping high?
- Connect to servers geographically closer to you
- Check your internet connection
- Close bandwidth-heavy applications

### The game crashes on startup
1. Update your graphics drivers
2. Verify Vulkan 1.2 support on your GPU
3. Check logs in `~/.config/plix/logs/`
4. Try launching from command line to see error messages

### I can't connect to a server
- Verify the server address is correct
- Check if the server is running
- Ensure UDP port 7777 isn't blocked by firewall
- Try connecting with `--log-level debug` for details

### How do I report a bug?
Open an issue at: https://github.com/your-org/plix/issues

Include:
- Plix version (`plix-client --version`)
- Operating system
- Steps to reproduce
- Relevant log output

### Where are my settings stored?
| Platform | Path |
|----------|------|
| Linux | `~/.config/plix/config.toml` |
| macOS | `~/Library/Application Support/plix/config.toml` |
| Windows | `%APPDATA%\plix\config.toml` |

### Can I use mods?
Plix supports mods through the mod API. See [Modding Overview](../modding/overview.md).

Mods require:
- Server support (mods must be enabled by server)
- Compatible mod API version

## Server Administration

### How do I configure my server?
Server configuration can be done via:
1. Command-line flags
2. Environment variables (prefix: `PLIX_`)
3. Configuration file

See [Server Configuration](../server/configuration.md).

### How do I register my server in the browser?
Use the `--master-url` flag:
```bash
./plix-server --master-url http://master.plix.example.com \
              --server-name "My Server" \
              --region "eu-west"
```

### How do I back up player data?
Player data is stored in `~/.local/share/plix/worlds/`. Back up this directory regularly.

The server creates automatic backups before migrations.

### How do I update my server?
See [Server Upgrading](../server/upgrading.md) for migration procedures.

Key steps:
1. Stop the server
2. Back up data
3. Replace binaries
4. Migrations run automatically on startup

## Modding

### Can I create mods for Plix?
Yes! See [Modding Getting Started](../modding/getting-started.md).

### What can mods do?
Mods can:
- Subscribe to game events
- Modify world state
- Add custom gameplay logic
- Schedule timers

Mods run in a sandboxed WASM environment.

### Where do I put mods?
Mods go in the server's mods directory. See [Server Mods](../server/mods.md).

## Contributing

### How can I contribute to Plix?
See [CONTRIBUTING.md](https://github.com/your-org/plix/blob/main/CONTRIBUTING.md).

Ways to contribute:
- Report bugs
- Submit pull requests
- Improve documentation
- Help other users

### What's the development roadmap?
See [ROADMAP.md](https://github.com/your-org/plix/blob/main/ROADMAP.md) for planned features.

### Where can I get help?
- GitHub Issues: Bug reports and feature requests
- GitHub Discussions: General questions and community chat
