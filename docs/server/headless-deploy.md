# Headless Server Deployment Guide

This guide covers deploying the Plix headless server for production use.

## Quick Start

### From Binary Bundle

1. Download the appropriate bundle for your platform:
   - `plix-server-headless-linux-x86_64-{version}.tar.gz`
   - `plix-server-headless-win64-{version}.zip`
   - `plix-server-headless-macos-{version}.zip`

2. Extract and run:

```bash
# Linux/macOS
tar xzf plix-server-headless-linux-x86_64-0.1.0.tar.gz
cd plix-server-headless-linux-x86_64-0.1.0
cp configs/examples/server.toml ./server.toml
./run_server.sh server.toml
```

```powershell
# Windows
Expand-Archive plix-server-headless-win64-0.1.0.zip
cd plix-server-headless-win64-0.1.0
Copy-Item configs\examples\server.toml .\server.toml
.\run_server.ps1 server.toml
```

### From Docker

```bash
docker run -d \
    -p 7777:7777/udp \
    -v plix-world:/data/world \
    -v plix-mods:/data/mods \
    -v $(pwd)/config:/config:ro \
    plix-server-headless:latest
```

## Configuration

### server.toml

```toml
[network]
bind_address = "0.0.0.0"
port = 7777
max_players = 32

[game]
tick_rate = 30
arena = "ffa_small"

[server]
name = "My Plix Server"
motd = "Welcome!"

[logging]
level = "info"

[persistence]
autosave_interval_secs = 300
shutdown_timeout_secs = 5
```

### CLI Arguments

```
plix-server-headless [OPTIONS]

Options:
  -c, --config <PATH>        Path to configuration file
      --bind <IP>            IP address to bind (default: 0.0.0.0)
      --port <PORT>          UDP port (default: 7777)
      --max-players <N>      Max players (default: 32)
      --arena <NAME>         Arena to load (default: ffa_small)
      --tick-rate <HZ>       Server tick rate (default: 30)
      --log-level <LEVEL>    Log level (default: info)
      --shutdown-timeout <S> Graceful shutdown timeout (default: 5)
      --validate             Validate config and exit
  -h, --help                 Print help
  -V, --version              Print version
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `PLIX_CONFIG` | Config file path | - |
| `PLIX_LOG_LEVEL` | Log verbosity | `info` |
| `PLIX_PORT` | Override server port | `7777` |
| `RUST_BACKTRACE` | Enable backtraces | `0` |

## Exit Codes

| Code | Name | Description |
|------|------|-------------|
| 0 | Success | Clean shutdown |
| 1 | GeneralError | Unspecified runtime error |
| 2 | Misuse | Invalid CLI arguments |
| 64 | BindFailed | Port already in use |
| 65 | AssetLoadFailed | Missing arena files |
| 66 | PersistenceError | World save/load failure |
| 67 | NetworkError | Socket creation failed |
| 68 | ShutdownTimeout | Forced exit after timeout |

## Signal Handling

The server handles signals for graceful shutdown:

- **SIGINT** (Ctrl+C): Initiates graceful shutdown
- **SIGTERM**: Initiates graceful shutdown (Docker, systemd)

Shutdown process:
1. Stop accepting new connections
2. Notify connected players
3. Save world state
4. Close network sockets
5. Exit with code 0

If shutdown takes longer than `shutdown_timeout_secs`, the server force-exits with code 68.

## Linux VM Deployment

### Systemd Service

Create `/etc/systemd/system/plix-server.service`:

```ini
[Unit]
Description=Plix Game Server
After=network.target
Wants=network-online.target

[Service]
Type=simple
User=plix
Group=plix
WorkingDirectory=/opt/plix
ExecStart=/opt/plix/plix-server-headless --config /opt/plix/server.toml
Restart=on-failure
RestartSec=5

# Security hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/plix/data
PrivateTmp=true

# Resource limits
LimitNOFILE=65535
MemoryMax=2G
CPUQuota=200%

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
sudo systemctl daemon-reload
sudo systemctl enable plix-server
sudo systemctl start plix-server
sudo systemctl status plix-server
```

### Firewall Configuration

```bash
# UFW (Ubuntu)
sudo ufw allow 7777/udp comment "Plix game server"

# firewalld (RHEL/CentOS)
sudo firewall-cmd --permanent --add-port=7777/udp
sudo firewall-cmd --reload

# iptables
sudo iptables -A INPUT -p udp --dport 7777 -j ACCEPT
```

## Docker Deployment

### Building the Image

```bash
cd /path/to/plix
docker build -t plix-server-headless -f deploy/docker/Dockerfile.headless .
```

### Running with Docker

```bash
# Basic run
docker run -d \
    --name plix-server \
    -p 7777:7777/udp \
    plix-server-headless

# With persistence and config
docker run -d \
    --name plix-server \
    -p 7777:7777/udp \
    -v plix-world:/data/world \
    -v plix-mods:/data/mods \
    -v $(pwd)/config:/config:ro \
    -e PLIX_LOG_LEVEL=info \
    --restart unless-stopped \
    plix-server-headless
```

### Docker Compose

```yaml
version: "3.8"

services:
  plix-server:
    image: plix-server-headless:latest
    container_name: plix-server
    restart: unless-stopped
    ports:
      - "7777:7777/udp"
    volumes:
      - plix-world:/data/world
      - plix-mods:/data/mods
      - ./config:/config:ro
    environment:
      - PLIX_LOG_LEVEL=info

volumes:
  plix-world:
  plix-mods:
```

Start:

```bash
docker compose up -d
docker compose logs -f
```

## Monitoring

### Log Output

Logs are written to stdout in JSON format (when `RUST_LOG` includes JSON):

```bash
# View logs
docker logs -f plix-server

# Or with systemd
journalctl -u plix-server -f
```

### Health Checks

The server supports health checks via CLI:

```bash
# Check if binary runs
./plix-server-headless --help

# Validate config without starting
./plix-server-headless --config server.toml --validate
```

Docker health check:

```bash
docker inspect --format='{{.State.Health.Status}}' plix-server
```

### Metrics

Server metrics are available via tracing spans. Configure a tracing subscriber for Prometheus, Jaeger, or other backends.

## Troubleshooting

### Port Already in Use (Exit 64)

```bash
# Find what's using the port
lsof -i :7777
# or
netstat -tulpn | grep 7777

# Kill the process or use a different port
./plix-server-headless --port 7778
```

### Missing Assets (Exit 65)

```bash
# Verify assets directory
ls -la assets/arenas/

# Specify custom path
./plix-server-headless --assets-dir /path/to/assets
```

### Permission Denied

```bash
# Make binary executable
chmod +x plix-server-headless

# Check file ownership
ls -la plix-server-headless

# On Linux, ensure UDP port > 1024 or run as root
```

### Docker Container Exits Immediately

```bash
# Check logs
docker logs plix-server

# Run interactively to debug
docker run -it --rm plix-server-headless --validate
```

### Graceful Shutdown Not Working

Ensure the container receives signals properly:

```bash
# Docker run with proper signal handling
docker run -d --init plix-server-headless

# Or use exec form in Dockerfile (already configured)
ENTRYPOINT ["/app/plix-server-headless"]
```
