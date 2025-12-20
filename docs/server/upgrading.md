# Server Upgrade Guide

This guide covers upgrading Plix servers between versions.

## Before Upgrading

### Pre-Upgrade Checklist

1. **Read Release Notes**: Check for breaking changes
2. **Backup Data**: Create manual backup of all server data
3. **Check Compatibility**: Verify mod compatibility with new version
4. **Plan Downtime**: Notify players of maintenance window
5. **Test Environment**: Test upgrade on non-production server first

### Version Compatibility

Plix follows semantic versioning:

| Upgrade Type | Example | Compatibility |
|--------------|---------|---------------|
| Patch (1.0.0 → 1.0.1) | Bug fixes | Fully compatible, no migration |
| Minor (1.0 → 1.1) | New features | Backward compatible |
| Major (1.x → 2.0) | Breaking changes | May require manual migration |

## Standard Upgrade Process

### Step 1: Stop the Server

```bash
# Graceful shutdown
pkill -SIGTERM plix-server

# Wait for shutdown (check logs)
tail -f ~/.local/share/plix/logs/server.log
```

### Step 2: Backup Current Installation

```bash
# Create timestamped backup
DATE=$(date +%Y%m%d_%H%M%S)
tar -czf plix-backup-$DATE.tar.gz \
    ~/.local/share/plix/worlds \
    ~/.config/plix \
    ./plix-server
```

### Step 3: Download New Version

```bash
# Download new release
wget https://github.com/your-org/plix/releases/download/v1.1.0/plix-1.1.0-linux-x64.tar.gz

# Verify checksum
sha256sum -c SHA256SUMS

# Extract
tar -xzf plix-1.1.0-linux-x64.tar.gz
```

### Step 4: Replace Binaries

```bash
# Replace binaries
cp plix-1.1.0/plix-server ./plix-server

# Verify version
./plix-server --version
```

### Step 5: Start Server

```bash
# Start server - migrations run automatically
./plix-server
```

The server will:
1. Detect configuration version
2. Create automatic backup
3. Run necessary migrations
4. Log migration progress
5. Start normally

### Step 6: Verify

```bash
# Check logs for migration success
grep -i "migration" ~/.local/share/plix/logs/server.log

# Verify server is running
./plix-client --server localhost:7777 --headless
```

## Migration Details

### Automatic Migrations

When the server starts, it automatically:

1. **Detects Versions**: Reads `config_version` from config files
2. **Creates Backups**: Stores backup with SHA-256 checksum
3. **Runs Migrations**: Applies transformations sequentially
4. **Updates Version**: Writes new version to config

### Migration Logging

Monitor migration progress in logs:

```
INFO Starting migration file=config.toml from=0.0.0 to=1.0.0 steps=1
DEBUG Migration step from=0.0.0 to=1.0.0 description="Add config_version field"
INFO Migration completed successfully file=config.toml from=0.0.0 to=1.0.0 migrations=1
```

### Dry Run Mode

Test migrations without making changes:

```bash
./plix-server --migrate-dry-run
```

This shows what would happen without modifying files.

## Troubleshooting

### Migration Failed

If migration fails:

1. Check error message in logs
2. Restore from automatic backup:
   ```bash
   cp ~/.local/share/plix/backups/config.toml.bak.* ~/.config/plix/config.toml
   ```
3. Report issue with error details

### Version Mismatch

If you see "Data version newer than engine":

- You're running an older server version than the data
- Upgrade to at least the version that created the data
- Never downgrade after migration

### Mod Compatibility Issues

If mods fail to load after upgrade:

1. Check mod version requirements in manifest
2. Update mods to compatible versions
3. Contact mod authors for updates
4. Temporarily disable incompatible mods

## Rollback Procedure

If you need to rollback to previous version:

1. **Stop Server**
   ```bash
   pkill plix-server
   ```

2. **Restore Backup**
   ```bash
   tar -xzf plix-backup-YYYYMMDD_HHMMSS.tar.gz
   ```

3. **Restore Binaries**
   ```bash
   cp backup/plix-server ./plix-server
   ```

4. **Start Server**
   ```bash
   ./plix-server
   ```

**Warning**: Rolling back after data migration may cause data loss. Only rollback if the backup is from before the migration.

## Major Version Upgrades

For major version upgrades (e.g., 1.x → 2.0):

1. Read migration guide in release notes
2. Test on staging environment
3. Follow any manual migration steps
4. Allow extra downtime for extended migrations
5. Have rollback plan ready

## Automated Upgrades

For automated deployments:

```bash
#!/bin/bash
# Example upgrade script

set -e

VERSION=$1
BACKUP_DIR="/backups"

# Pre-flight checks
./plix-server --version

# Backup
./scripts/backup.sh

# Download and verify
wget "https://releases.plix.example.com/v$VERSION/plix-server"
sha256sum --check "plix-server.sha256"

# Upgrade
chmod +x plix-server
systemctl restart plix-server

# Health check
sleep 5
curl -f http://localhost:8080/health || exit 1

echo "Upgrade to v$VERSION complete"
```
