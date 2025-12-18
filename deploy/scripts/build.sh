#!/usr/bin/env bash
# =============================================================================
# Plix Server Docker Image Build Script
# Feature 028: Dedicated Server Packaging
# =============================================================================
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default values
TAG="latest"
BUILD_SERVER=true
BUILD_MASTER=false
NO_CACHE=false

# Script directory (for relative paths)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

# Help message
show_help() {
    cat << EOF
Usage: $(basename "$0") [OPTIONS]

Build Docker images for Plix game server and master server.

Options:
  --server      Build plix-server image only (default)
  --master      Build plix-master image only
  --all         Build both server and master images
  --tag TAG     Image tag (default: latest)
  --no-cache    Build without Docker cache
  -h, --help    Show this help message

Examples:
  $(basename "$0")                    # Build server with tag 'latest'
  $(basename "$0") --all --tag v1.0   # Build both with tag 'v1.0'
  $(basename "$0") --master           # Build only master server
EOF
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --server)
            BUILD_SERVER=true
            BUILD_MASTER=false
            shift
            ;;
        --master)
            BUILD_SERVER=false
            BUILD_MASTER=true
            shift
            ;;
        --all)
            BUILD_SERVER=true
            BUILD_MASTER=true
            shift
            ;;
        --tag)
            TAG="$2"
            shift 2
            ;;
        --no-cache)
            NO_CACHE=true
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

# Check Docker is available
if ! command -v docker &> /dev/null; then
    echo -e "${RED}Error: Docker not found. Please install Docker.${NC}"
    exit 2
fi

# Build options
DOCKER_BUILD_OPTS=""
if [[ "$NO_CACHE" == "true" ]]; then
    DOCKER_BUILD_OPTS="--no-cache"
fi

# Build plix-server
if [[ "$BUILD_SERVER" == "true" ]]; then
    echo -e "${YELLOW}Building plix-server:${TAG}...${NC}"
    docker build \
        $DOCKER_BUILD_OPTS \
        -t "plix-server:${TAG}" \
        -f "${REPO_ROOT}/deploy/docker/Dockerfile" \
        "${REPO_ROOT}"

    if [[ $? -eq 0 ]]; then
        echo -e "${GREEN}✓ plix-server:${TAG} built successfully${NC}"
    else
        echo -e "${RED}✗ plix-server build failed${NC}"
        exit 1
    fi
fi

# Build plix-master
if [[ "$BUILD_MASTER" == "true" ]]; then
    echo -e "${YELLOW}Building plix-master:${TAG}...${NC}"
    docker build \
        $DOCKER_BUILD_OPTS \
        -t "plix-master:${TAG}" \
        -f "${REPO_ROOT}/deploy/docker/Dockerfile.master" \
        "${REPO_ROOT}"

    if [[ $? -eq 0 ]]; then
        echo -e "${GREEN}✓ plix-master:${TAG} built successfully${NC}"
    else
        echo -e "${RED}✗ plix-master build failed${NC}"
        exit 1
    fi
fi

echo -e "${GREEN}Build complete!${NC}"
