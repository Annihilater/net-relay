#!/bin/bash
# Start net-relay in background mode

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
CONFIG_FILE="${PROJECT_DIR}/config.toml"
BINARY="${PROJECT_DIR}/target/release/net-relay"
LOG_DIR="${PROJECT_DIR}/logs"
LOG_FILE="${LOG_DIR}/net-relay.log"
PID_FILE="${PROJECT_DIR}/net-relay.pid"

cd "$PROJECT_DIR"

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
    echo -e "${YELLOW}Binary not found, building...${NC}"
    cargo build --release
fi

# Check if config exists
if [ ! -f "$CONFIG_FILE" ]; then
    echo -e "${RED}Error: Config file not found: $CONFIG_FILE${NC}"
    exit 1
fi

echo -e "${GREEN}🚀 Starting net-relay in background...${NC}"

# Start in background with logging
nohup "$BINARY" -c "$CONFIG_FILE" >> "$LOG_FILE" 2>&1 &
PID=$!
echo $PID > "$PID_FILE"

sleep 1

if ps -p $PID > /dev/null 2>&1; then
    echo -e "${GREEN}✓ net-relay started successfully (PID: $PID)${NC}"
    echo -e "  Log file: $LOG_FILE"
else
    echo -e "${RED}✗ Failed to start net-relay${NC}"
    rm -f "$PID_FILE"
    exit 1
fi
