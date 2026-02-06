#!/bin/bash
# View net-relay logs
# Usage: ./scripts/log.sh [OPTIONS]
#   -f, --follow    Follow log output (like tail -f)
#   -n, --lines N   Show last N lines (default: 50)
#   --clear         Clear log file
#   -h, --help      Show help

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# Go up one level to get the installation directory
INSTALL_DIR="$(dirname "$SCRIPT_DIR")"
LOG_FILE="${INSTALL_DIR}/logs/net-relay.log"

# Colors
RED='\033[0;31m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

show_help() {
    echo "Usage: $0 [OPTION]"
    echo ""
    echo "Options:"
    echo "  -f, --follow    Follow log output (like tail -f)"
    echo "  -n, --lines N   Show last N lines (default: 50)"
    echo "  --clear         Clear log file"
    echo "  -h, --help      Show this help"
    echo ""
    echo "Default: Show last 50 lines"
}

if [ ! -f "$LOG_FILE" ]; then
    echo -e "${YELLOW}Log file not found: $LOG_FILE${NC}"
    echo "Start the server first with: ./scripts/start.sh"
    exit 1
fi

# Parse arguments
case "${1:-}" in
    -f|--follow)
        echo -e "${CYAN}═══════════════════════════════════════${NC}"
        echo -e "${CYAN}  Following log file (Ctrl+C to exit)${NC}"
        echo -e "${CYAN}═══════════════════════════════════════${NC}"
        echo ""
        tail -f "$LOG_FILE"
        ;;
    -n|--lines)
        LINES="${2:-50}"
        echo -e "${CYAN}═══════════════════════════════════════${NC}"
        echo -e "${CYAN}  Last $LINES lines of log${NC}"
        echo -e "${CYAN}═══════════════════════════════════════${NC}"
        echo ""
        tail -n "$LINES" "$LOG_FILE"
        ;;
    --clear)
        echo -n > "$LOG_FILE"
        echo -e "Log file cleared: $LOG_FILE"
        ;;
    -h|--help)
        show_help
        ;;
    *)
        echo -e "${CYAN}═══════════════════════════════════════${NC}"
        echo -e "${CYAN}  Last 50 lines of log${NC}"
        echo -e "${CYAN}═══════════════════════════════════════${NC}"
        echo ""
        tail -n 50 "$LOG_FILE"
        ;;
esac
