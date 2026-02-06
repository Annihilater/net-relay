#!/bin/bash
# Restart net-relay
# Usage: ./scripts/restart.sh

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "Restarting net-relay..."
echo ""

"$SCRIPT_DIR/stop.sh"
sleep 1
"$SCRIPT_DIR/start.sh"
