#!/bin/bash
# Dev script - run net-relay in foreground mode for development

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
CONFIG_FILE="${PROJECT_DIR}/config.toml"
BINARY="${PROJECT_DIR}/target/release/net-relay"

cd "$PROJECT_DIR"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}🚀 Starting net-relay in development mode...${NC}"

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

# Stop any existing instance
if pgrep -x "net-relay" > /dev/null; then
    echo -e "${YELLOW}Stopping existing net-relay instance...${NC}"
    pkill -x net-relay || true
    sleep 1
fi

echo -e "${GREEN}Running: $BINARY -c $CONFIG_FILE${NC}"
echo ""

# Run in foreground
exec "$BINARY" -c "$CONFIG_FILE"
