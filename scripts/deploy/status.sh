#!/bin/bash
# Check net-relay status
# Usage: ./scripts/status.sh

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# Go up one level to get the installation directory
INSTALL_DIR="$(dirname "$SCRIPT_DIR")"
PID_FILE="${INSTALL_DIR}/net-relay.pid"
CONFIG_FILE="${INSTALL_DIR}/config.toml"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}═══════════════════════════════════════${NC}"
echo -e "${CYAN}       Net-Relay Status${NC}"
echo -e "${CYAN}═══════════════════════════════════════${NC}"
echo ""

# Check if running
RUNNING=false
PID=""

if [ -f "$PID_FILE" ]; then
    PID=$(cat "$PID_FILE")
    if ps -p "$PID" > /dev/null 2>&1; then
        RUNNING=true
    else
        rm -f "$PID_FILE"
        PID=""
    fi
fi

# Fallback: check by process name
if [ "$RUNNING" = false ]; then
    if pgrep -x "net-relay" > /dev/null; then
        PID=$(pgrep -x "net-relay" | head -1)
        RUNNING=true
        echo -e "${YELLOW}(Process running without PID file)${NC}"
    fi
fi

if [ "$RUNNING" = true ]; then
    echo -e "Status:  ${GREEN}● Running${NC} (PID: $PID)"
    
    # Get process info
    if command -v ps > /dev/null; then
        UPTIME=$(ps -o etime= -p "$PID" 2>/dev/null | xargs)
        MEM=$(ps -o rss= -p "$PID" 2>/dev/null | awk '{printf "%.1f MB", $1/1024}')
        [ -n "$UPTIME" ] && echo -e "Uptime:  $UPTIME"
        [ -n "$MEM" ] && echo -e "Memory:  $MEM"
    fi
else
    echo -e "Status:  ${RED}● Stopped${NC}"
fi

echo ""

# Try to read ports from config
SOCKS_PORT=1080
HTTP_PORT=8080
API_PORT=3000

if [ -f "$CONFIG_FILE" ]; then
    # Try to extract ports from config (basic parsing)
    SOCKS_PORT=$(grep -E "^\s*socks_port\s*=" "$CONFIG_FILE" 2>/dev/null | head -1 | sed 's/.*=\s*//' | tr -d ' "' || echo "1080")
    HTTP_PORT=$(grep -E "^\s*http_port\s*=" "$CONFIG_FILE" 2>/dev/null | head -1 | sed 's/.*=\s*//' | tr -d ' "' || echo "8080")
    API_PORT=$(grep -E "^\s*api_port\s*=" "$CONFIG_FILE" 2>/dev/null | head -1 | sed 's/.*=\s*//' | tr -d ' "' || echo "3000")
    
    # Use defaults if empty
    [ -z "$SOCKS_PORT" ] && SOCKS_PORT=1080
    [ -z "$HTTP_PORT" ] && HTTP_PORT=8080
    [ -z "$API_PORT" ] && API_PORT=3000
fi

echo -e "${CYAN}Listening Ports:${NC}"

check_port() {
    local name=$1
    local port=$2
    
    if command -v lsof > /dev/null; then
        if lsof -i ":$port" -sTCP:LISTEN 2>/dev/null | grep -q "net-relay"; then
            echo -e "  $name: ${GREEN}:$port ✓${NC}"
        else
            echo -e "  $name: ${RED}:$port ✗${NC}"
        fi
    elif command -v ss > /dev/null; then
        if ss -tlnp 2>/dev/null | grep -q ":$port "; then
            echo -e "  $name: ${GREEN}:$port ✓${NC}"
        else
            echo -e "  $name: ${RED}:$port ✗${NC}"
        fi
    elif command -v netstat > /dev/null; then
        if netstat -tlnp 2>/dev/null | grep -q ":$port "; then
            echo -e "  $name: ${GREEN}:$port ✓${NC}"
        else
            echo -e "  $name: ${RED}:$port ✗${NC}"
        fi
    else
        echo -e "  $name: :$port (cannot verify)"
    fi
}

check_port "SOCKS5" "$SOCKS_PORT"
check_port "HTTP  " "$HTTP_PORT"
check_port "API   " "$API_PORT"

echo ""
echo -e "${CYAN}═══════════════════════════════════════${NC}"

if [ "$RUNNING" = true ]; then
    echo -e "Dashboard: ${GREEN}http://localhost:${API_PORT}${NC}"
fi
