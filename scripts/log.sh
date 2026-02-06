#!/bin/bash
# View net-relay logs

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
LOG_FILE="${PROJECT_DIR}/logs/net-relay.log"

# Colors
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

if [ ! -f "$LOG_FILE" ]; then
    echo -e "${YELLOW}Log file not found: $LOG_FILE${NC}"
    echo "Start the server first with: ./scripts/start.sh"
    exit 1
fi

# Parse arguments
case "${1:-}" in
    -f|--follow)
        echo -e "${YELLOW}Following log file (Ctrl+C to exit)...${NC}"
        tail -f "$LOG_FILE"
        ;;
    -n|--lines)
        LINES="${2:-50}"
        tail -n "$LINES" "$LOG_FILE"
        ;;
    --clear)
        echo -n > "$LOG_FILE"
        echo "Log file cleared."
        ;;
    -h|--help)
        echo "Usage: $0 [OPTION]"
        echo ""
        echo "Options:"
        echo "  -f, --follow    Follow log output (like tail -f)"
        echo "  -n, --lines N   Show last N lines (default: 50)"
        echo "  --clear         Clear log file"
        echo "  -h, --help      Show this help"
        echo ""
        echo "Default: Show last 50 lines"
        ;;
    *)
        tail -n 50 "$LOG_FILE"
        ;;
esac
