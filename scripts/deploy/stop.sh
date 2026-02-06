#!/bin/bash
# Stop net-relay
# Usage: ./scripts/stop.sh

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# Go up one level to get the installation directory
INSTALL_DIR="$(dirname "$SCRIPT_DIR")"
PID_FILE="${INSTALL_DIR}/net-relay.pid"

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
        echo "Sending SIGTERM to PID $PID..."
        kill "$PID" 2>/dev/null
        
        # Wait for graceful shutdown (up to 5 seconds)
        for i in {1..5}; do
            if ! ps -p "$PID" > /dev/null 2>&1; then
                break
            fi
            sleep 1
        done
        
        # Force kill if still running
        if ps -p "$PID" > /dev/null 2>&1; then
            echo -e "${YELLOW}Process still running, sending SIGKILL...${NC}"
            kill -9 "$PID" 2>/dev/null
            sleep 1
        fi
    fi
    rm -f "$PID_FILE"
fi

# Also try pkill as fallback
if pgrep -x "net-relay" > /dev/null; then
    echo "Found additional net-relay processes, stopping..."
    pkill -x "net-relay" 2>/dev/null || true
    sleep 1
fi

if pgrep -x "net-relay" > /dev/null; then
    echo -e "${RED}✗ Failed to stop net-relay${NC}"
    echo "Remaining processes:"
    pgrep -xl "net-relay"
    exit 1
else
    echo -e "${GREEN}✓ net-relay stopped${NC}"
fi
