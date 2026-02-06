#!/bin/bash
# Dev script - run net-relay in foreground mode
# Usage: ./scripts/dev.sh

set -e

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# Go up one level to get the installation directory
INSTALL_DIR="$(dirname "$SCRIPT_DIR")"
BINARY="${INSTALL_DIR}/net-relay"
CONFIG_FILE="${INSTALL_DIR}/config.toml"
LOG_DIR="${INSTALL_DIR}/logs"

cd "$INSTALL_DIR"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Create logs directory
mkdir -p "$LOG_DIR"

# Check if binary exists
if [ ! -f "$BINARY" ]; then
    echo -e "${RED}Error: Binary not found: $BINARY${NC}"
    exit 1
fi

# Check if config exists, if not copy from example
if [ ! -f "$CONFIG_FILE" ]; then
    if [ -f "${INSTALL_DIR}/config.example.toml" ]; then
        echo -e "${YELLOW}Config file not found, copying from example...${NC}"
        cp "${INSTALL_DIR}/config.example.toml" "$CONFIG_FILE"
    else
        echo -e "${RED}Error: Config file not found: $CONFIG_FILE${NC}"
        exit 1
    fi
fi

# Stop any existing instance
PID_FILE="${INSTALL_DIR}/net-relay.pid"
if [ -f "$PID_FILE" ]; then
    PID=$(cat "$PID_FILE")
    if ps -p "$PID" > /dev/null 2>&1; then
        echo -e "${YELLOW}Stopping existing net-relay instance (PID: $PID)...${NC}"
        kill "$PID" 2>/dev/null || true
        sleep 1
    fi
    rm -f "$PID_FILE"
fi

echo -e "${GREEN}🚀 Starting net-relay in foreground (development mode)...${NC}"
echo -e "  Binary:  $BINARY"
echo -e "  Config:  $CONFIG_FILE"
echo ""

# Run in foreground with environment logging
export RUST_LOG="${RUST_LOG:-info}"
exec "$BINARY" -c "$CONFIG_FILE"
