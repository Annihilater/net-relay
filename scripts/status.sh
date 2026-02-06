#!/bin/bash
# Check net-relay status

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
PID_FILE="${PROJECT_DIR}/net-relay.pid"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}═══════════════════════════════════════${NC}"
echo -e "${CYAN}       Net-Relay Status${NC}"
echo -e "${CYAN}═══════════════════════════════════════${NC}"

# Check if running
if [ -f "$PID_FILE" ]; then
    PID=$(cat "$PID_FILE")
    if ps -p "$PID" > /dev/null 2>&1; then
        echo -e "Status:  ${GREEN}● Running${NC} (PID: $PID)"
        
        # Get process info
        if command -v ps > /dev/null; then
            UPTIME=$(ps -o etime= -p "$PID" 2>/dev/null | xargs)
            MEM=$(ps -o rss= -p "$PID" 2>/dev/null | awk '{printf "%.1f MB", $1/1024}')
            echo -e "Uptime:  $UPTIME"
            echo -e "Memory:  $MEM"
        fi
    else
        echo -e "Status:  ${RED}● Stopped${NC} (stale PID file)"
        rm -f "$PID_FILE"
    fi
elif pgrep -x "net-relay" > /dev/null; then
    PID=$(pgrep -x "net-relay" | head -1)
    echo -e "Status:  ${GREEN}● Running${NC} (PID: $PID, no PID file)"
else
    echo -e "Status:  ${RED}● Stopped${NC}"
fi

echo ""

# Check ports
echo -e "${CYAN}Listening Ports:${NC}"
if command -v lsof > /dev/null; then
    lsof -i :1080 -sTCP:LISTEN 2>/dev/null | grep -q "net-relay" && echo -e "  SOCKS5: ${GREEN}:1080 ✓${NC}" || echo -e "  SOCKS5: ${RED}:1080 ✗${NC}"
    lsof -i :8080 -sTCP:LISTEN 2>/dev/null | grep -q "net-relay" && echo -e "  HTTP:   ${GREEN}:8080 ✓${NC}" || echo -e "  HTTP:   ${RED}:8080 ✗${NC}"
    lsof -i :3000 -sTCP:LISTEN 2>/dev/null | grep -q "net-relay" && echo -e "  API:    ${GREEN}:3000 ✓${NC}" || echo -e "  API:    ${RED}:3000 ✗${NC}"
else
    echo "  (lsof not available)"
fi

echo -e "${CYAN}═══════════════════════════════════════${NC}"
