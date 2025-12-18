#!/usr/bin/env bash
# =============================================================================
# Plix Server Docker Compose Wrapper Script
# Feature 028: Dedicated Server Packaging
# =============================================================================
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Script directory (for relative paths)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMPOSE_DIR="${SCRIPT_DIR}/../docker"

# Default values
COMMAND="up"
PROFILE=""
DETACH=false
BUILD=false

# Help message
show_help() {
    cat << EOF
Usage: $(basename "$0") [COMMAND] [OPTIONS]

Docker Compose wrapper for Plix server infrastructure.

Commands:
  up              Start services (default)
  down            Stop and remove services
  logs            View service logs
  status          Show service status
  restart         Restart services
  build           Build images only

Options:
  --server-only   Start only game server (default behavior)
  --with-master   Start server + master (profile: full)
  --master-only   Start only master server (profile: master)
  --build         Rebuild images before starting
  --detach        Run in background (for 'up' command)
  -h, --help      Show this help message

Examples:
  $(basename "$0")                        # Start server in foreground
  $(basename "$0") --detach               # Start server in background
  $(basename "$0") --with-master --detach # Start full stack in background
  $(basename "$0") logs                   # View logs
  $(basename "$0") down                   # Stop everything
EOF
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        up|down|logs|status|restart|build)
            COMMAND="$1"
            shift
            ;;
        --server-only)
            PROFILE=""
            shift
            ;;
        --with-master)
            PROFILE="full"
            shift
            ;;
        --master-only)
            PROFILE="master"
            shift
            ;;
        --build)
            BUILD=true
            shift
            ;;
        --detach)
            DETACH=true
            shift
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            echo -e "${RED}Error: Unknown option $1${NC}"
            show_help
            exit 1
            ;;
    esac
done

# Check Docker Compose is available
if ! command -v docker &> /dev/null; then
    echo -e "${RED}Error: Docker not found. Please install Docker.${NC}"
    exit 2
fi

# Change to compose directory
cd "${COMPOSE_DIR}"

# Build compose command
COMPOSE_CMD="docker compose"

if [[ -n "$PROFILE" ]]; then
    COMPOSE_CMD+=" --profile ${PROFILE}"
fi

# Execute command
case $COMMAND in
    up)
        echo -e "${GREEN}Starting Plix services...${NC}"
        if [[ -n "$PROFILE" ]]; then
            echo -e "  Profile: ${PROFILE}"
        else
            echo -e "  Profile: server-only (default)"
        fi

        COMPOSE_OPTS=""
        if [[ "$BUILD" == "true" ]]; then
            COMPOSE_OPTS+=" --build"
        fi
        if [[ "$DETACH" == "true" ]]; then
            COMPOSE_OPTS+=" -d"
        fi

        $COMPOSE_CMD up $COMPOSE_OPTS

        if [[ "$DETACH" == "true" ]]; then
            echo -e "${GREEN}✓ Services started in background${NC}"
            echo -e "  View logs:  $(basename "$0") logs"
            echo -e "  Status:     $(basename "$0") status"
            echo -e "  Stop:       $(basename "$0") down"
        fi
        ;;
    down)
        echo -e "${YELLOW}Stopping Plix services...${NC}"
        $COMPOSE_CMD down
        echo -e "${GREEN}✓ Services stopped${NC}"
        ;;
    logs)
        $COMPOSE_CMD logs -f
        ;;
    status)
        echo -e "${GREEN}Plix Service Status:${NC}"
        $COMPOSE_CMD ps
        ;;
    restart)
        echo -e "${YELLOW}Restarting Plix services...${NC}"
        $COMPOSE_CMD restart
        echo -e "${GREEN}✓ Services restarted${NC}"
        ;;
    build)
        echo -e "${GREEN}Building Plix images...${NC}"
        $COMPOSE_CMD build
        echo -e "${GREEN}✓ Images built${NC}"
        ;;
esac
