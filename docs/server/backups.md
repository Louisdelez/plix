# Server Backup Guide

This guide covers backup strategies for Plix servers.

## Automatic Backups

Plix automatically creates backups before migrations:

- Backups are stored in `~/.local/share/plix/backups/`
- Each backup is timestamped (ISO 8601 format)
- SHA-256 checksums verify backup integrity
- Rolling window keeps the 3 most recent backups

### Backup File Format

Backup files are named:
```
{original_filename}.bak.{timestamp}
```

Example:
```
config.toml.bak.2025-01-15T10-30-00
```

### Verifying Backups

Backups include SHA-256 checksums logged during creation:

```
INFO Backup created successfully source=config.toml backup=backups/config.toml.bak.2025-01-15T10-30-00 size=1234 checksum=abc123...
```

To manually verify:
```bash
sha256sum backups/config.toml.bak.2025-01-15T10-30-00
```

## Manual Backups

### What to Back Up

Critical server data:

| Path | Description |
|------|-------------|
| `~/.local/share/plix/worlds/` | World and chunk data |
| `~/.config/plix/server.toml` | Server configuration |
| `mods/` | Installed mod files |
| `arenas/` | Custom arena definitions |

### Backup Script Example

```bash
#!/bin/bash
# Simple Plix server backup script

BACKUP_DIR="/path/to/backups"
DATE=$(date +%Y-%m-%d_%H-%M-%S)
SERVER_DIR="$HOME/.local/share/plix"
CONFIG_DIR="$HOME/.config/plix"

# Create backup directory
mkdir -p "$BACKUP_DIR"

# Create compressed archive
tar -czf "$BACKUP_DIR/plix-backup-$DATE.tar.gz" \
    "$SERVER_DIR/worlds" \
    "$CONFIG_DIR" \
    ./mods \
    ./arenas

# Keep only last 7 daily backups
find "$BACKUP_DIR" -name "plix-backup-*.tar.gz" -mtime +7 -delete

echo "Backup complete: $BACKUP_DIR/plix-backup-$DATE.tar.gz"
```

## Restoring from Backup

### Automatic Backup Restoration

If a migration fails, you can restore from the automatic backup:

```bash
# List available backups
ls ~/.local/share/plix/backups/

# Copy backup over current file
cp ~/.local/share/plix/backups/config.toml.bak.2025-01-15T10-30-00 ~/.config/plix/config.toml
```

### Full Restoration

To restore from a manual backup:

1. Stop the server
2. Extract the backup archive
3. Copy files to their original locations
4. Restart the server

```bash
# Stop server
pkill plix-server

# Extract backup
tar -xzf plix-backup-2025-01-15.tar.gz -C /tmp/restore

# Restore files
cp -r /tmp/restore/worlds/* ~/.local/share/plix/worlds/
cp /tmp/restore/config/* ~/.config/plix/

# Restart server
./plix-server
```

## Backup Best Practices

1. **Regular Backups**: Run daily backups during low-activity periods
2. **Off-site Storage**: Copy backups to remote storage
3. **Test Restores**: Periodically verify backups can be restored
4. **Pre-Upgrade Backups**: Always backup before upgrading Plix versions
5. **Monitor Disk Space**: Ensure sufficient space for backup retention

## Backup Retention

Default automatic backup retention:
- **3 backups** per file type (config, saves)
- Oldest backups automatically deleted when limit exceeded

For production servers, consider:
- Daily backups retained for 7 days
- Weekly backups retained for 4 weeks
- Monthly backups retained for 12 months

## Disaster Recovery

### Complete Server Loss

If you lose the entire server:

1. Deploy new server instance
2. Install Plix binaries
3. Restore configuration from backup
4. Restore world data from backup
5. Verify configuration matches new environment
6. Restart server

### Corrupted World Data

If world data becomes corrupted:

1. Stop the server
2. Remove corrupted world directory
3. Restore from most recent valid backup
4. Players may lose some recent progress

### Configuration Issues

If server won't start due to config errors:

1. Check logs for specific error messages
2. Restore config from backup
3. Or delete config and let server regenerate defaults
