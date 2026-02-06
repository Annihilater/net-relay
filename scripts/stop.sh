#!/bin/bash
# Stop net-relay

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
PID_FILE="${PROJECT_DIR}/net-relay.pid"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${YELLOW}Stopping net-relay...${NC}"

# Try PID file first
if [ -f "$PID_FILE" ]; then
    PID=$(cat "$PID_FILE")
    if ps -p "$PID" > /dev/null 2>&1; then
        kill "$PID" 2>/dev/null
        sleep 1
        if ps -p "$PID" > /dev/null 2>&1; then
            echo -e "${YELLOW}Process still running, sending SIGKILL...${NC}"
            kill -9 "$PID" 2>/dev/null
        fi
    fi
    rm -f "$PID_FILE"
fi

# Also try pkill as fallback
if pgrep -x "net-relay" > /dev/null; then
    pkill -x "net-relay" 2>/dev/null || true
    sleep 1
fi

if pgrep -x "net-relay" > /dev/null; then
    echo -e "${RED}✗ Failed to stop net-relay${NC}"
    exit 1
else
    echo -e "${GREEN}✓ net-relay stopped${NC}"
fi
