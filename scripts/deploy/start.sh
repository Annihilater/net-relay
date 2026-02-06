#!/bin/bash
# Start net-relay in background mode
# Usage: ./scripts/start.sh

set -e

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# Go up one level to get the installation directory
INSTALL_DIR="$(dirname "$SCRIPT_DIR")"
BINARY="${INSTALL_DIR}/net-relay"
CONFIG_FILE="${INSTALL_DIR}/config.toml"
LOG_DIR="${INSTALL_DIR}/logs"
LOG_FILE="${LOG_DIR}/net-relay.log"
PID_FILE="${INSTALL_DIR}/net-relay.pid"

cd "$INSTALL_DIR"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Create logs directory
mkdir -p "$LOG_DIR"

# Check if already running
if [ -f "$PID_FILE" ]; then
    PID=$(cat "$PID_FILE")
    if ps -p "$PID" > /dev/null 2>&1; then
        echo -e "${YELLOW}net-relay is already running (PID: $PID)${NC}"
        exit 0
    else
        rm -f "$PID_FILE"
    fi
fi

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

echo -e "${GREEN}🚀 Starting net-relay in background...${NC}"

# Start in background with logging
export RUST_LOG="${RUST_LOG:-info}"
nohup "$BINARY" -c "$CONFIG_FILE" >> "$LOG_FILE" 2>&1 &
PID=$!
echo $PID > "$PID_FILE"

sleep 1

if ps -p $PID > /dev/null 2>&1; then
    echo -e "${GREEN}✓ net-relay started successfully${NC}"
    echo -e "  PID:      $PID"
    echo -e "  Log file: $LOG_FILE"
    echo -e ""
    echo -e "  Use './scripts/log.sh -f' to follow logs"
    echo -e "  Use './scripts/status.sh' to check status"
    echo -e "  Use './scripts/stop.sh' to stop"
else
    echo -e "${RED}✗ Failed to start net-relay${NC}"
    rm -f "$PID_FILE"
    echo -e "Check logs: $LOG_FILE"
    exit 1
fi
