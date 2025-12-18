#!/usr/bin/env bash
# =============================================================================
# Plix Server Docker Run Script
# Feature 028: Dedicated Server Packaging
# =============================================================================
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default values
SERVER_NAME="Plix Server"
GAME_MODE="ffa"
PORT="7777"
MAX_PLAYERS="16"
MASTER_URL=""
DETACH=false
VOLUME_PATH=""
TAG="latest"
CONTAINER_NAME="plix-server"

# Help message
show_help() {
    cat << EOF
Usage: $(basename "$0") [OPTIONS]

Run a Plix game server in a Docker container.

Options:
  --name NAME       Server name (default: "Plix Server")
  --mode MODE       Game mode: ffa, tdm, ctf, br_lite (default: ffa)
  --port PORT       UDP port (default: 7777)
  --players N       Max players (default: 16)
  --master URL      Master server URL (enables server browser registration)
  --volume PATH     Mount data volume from host path
  --detach          Run container in background
  --tag TAG         Image tag to use (default: latest)
  --container NAME  Container name (default: plix-server)
  -h, --help        Show this help message

Examples:
  $(basename "$0")                           # Start FFA server
  $(basename "$0") --mode tdm --players 24   # Start TDM server
  $(basename "$0") --detach --volume ./data  # Background with persistence
EOF
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --name)
            SERVER_NAME="$2"
            shift 2
            ;;
        --mode)
            GAME_MODE="$2"
            shift 2
            ;;
        --port)
            PORT="$2"
            shift 2
            ;;
        --players)
            MAX_PLAYERS="$2"
            shift 2
            ;;
        --master)
            MASTER_URL="$2"
            shift 2
            ;;
        --volume)
            VOLUME_PATH="$2"
            shift 2
            ;;
        --detach)
            DETACH=true
            shift
            ;;
        --tag)
            TAG="$2"
            shift 2
            ;;
        --container)
            CONTAINER_NAME="$2"
            shift 2
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

# Check Docker is available
if ! command -v docker &> /dev/null; then
    echo -e "${RED}Error: Docker not found. Please install Docker.${NC}"
    exit 2
fi

# Check if port is in use
if netstat -tuln 2>/dev/null | grep -q ":${PORT} " || ss -tuln 2>/dev/null | grep -q ":${PORT} "; then
    echo -e "${RED}Error: Port ${PORT} is already in use.${NC}"
    exit 3
fi

# Build docker run command
DOCKER_OPTS=()
DOCKER_OPTS+=("--name" "${CONTAINER_NAME}")
DOCKER_OPTS+=("-p" "${PORT}:7777/udp")
DOCKER_OPTS+=("-e" "PLIX_SERVER_NAME=${SERVER_NAME}")
DOCKER_OPTS+=("-e" "PLIX_GAME_MODE=${GAME_MODE}")
DOCKER_OPTS+=("-e" "PLIX_MAX_PLAYERS=${MAX_PLAYERS}")

if [[ -n "$MASTER_URL" ]]; then
    DOCKER_OPTS+=("-e" "PLIX_MASTER_URL=${MASTER_URL}")
    DOCKER_OPTS+=("-e" "PLIX_MASTER_ENABLED=true")
fi

if [[ -n "$VOLUME_PATH" ]]; then
    # Convert to absolute path
    VOLUME_PATH="$(cd "$(dirname "$VOLUME_PATH")" 2>/dev/null && pwd)/$(basename "$VOLUME_PATH")"
    mkdir -p "$VOLUME_PATH"
    DOCKER_OPTS+=("-v" "${VOLUME_PATH}:/data")
fi

if [[ "$DETACH" == "true" ]]; then
    DOCKER_OPTS+=("-d")
fi

# Remove existing container if present
if docker ps -a --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
    echo -e "${YELLOW}Removing existing container '${CONTAINER_NAME}'...${NC}"
    docker rm -f "${CONTAINER_NAME}" > /dev/null
fi

# Run the container
echo -e "${GREEN}Starting Plix server...${NC}"
echo -e "  Name:    ${SERVER_NAME}"
echo -e "  Mode:    ${GAME_MODE}"
echo -e "  Port:    ${PORT}/udp"
echo -e "  Players: ${MAX_PLAYERS}"
if [[ -n "$MASTER_URL" ]]; then
    echo -e "  Master:  ${MASTER_URL}"
fi
if [[ -n "$VOLUME_PATH" ]]; then
    echo -e "  Volume:  ${VOLUME_PATH}"
fi
echo ""

docker run "${DOCKER_OPTS[@]}" "plix-server:${TAG}"

if [[ "$DETACH" == "true" ]]; then
    echo -e "${GREEN}✓ Server started in background${NC}"
    echo -e "  View logs: docker logs -f ${CONTAINER_NAME}"
    echo -e "  Stop:      docker stop ${CONTAINER_NAME}"
fi
