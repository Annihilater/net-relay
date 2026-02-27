#!/bin/bash
# Upgrade net-relay binary while preserving config
# Usage: sudo ./upgrade-service.sh [install_dir]
#
# Run this from the new release package directory.
# It will replace the binary and restart the service
# without touching your config.toml.

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Must run as root
if [ "$(id -u)" -ne 0 ]; then
    echo -e "${RED}Error: This script must be run as root (use sudo)${NC}"
    exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PACKAGE_DIR="$(dirname "$SCRIPT_DIR")"
INSTALL_DIR="${1:-/opt/net-relay}"
SERVICE_NAME="net-relay"
USER_NAME="net-relay"
GROUP_NAME="net-relay"

echo -e "${BLUE}╔══════════════════════════════════════╗${NC}"
echo -e "${BLUE}║     Net-Relay Service Upgrade        ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════╝${NC}"
echo ""

# Check binary exists in package
if [ ! -f "${PACKAGE_DIR}/net-relay" ]; then
    echo -e "${RED}Error: Binary not found at ${PACKAGE_DIR}/net-relay${NC}"
    exit 1
fi

# Check target install dir exists
if [ ! -d "${INSTALL_DIR}" ]; then
    echo -e "${RED}Error: Install directory not found: ${INSTALL_DIR}${NC}"
    echo -e "If this is a fresh install, use install-service.sh instead."
    exit 1
fi

# Show version info
echo -e "  Package binary:   ${PACKAGE_DIR}/net-relay"
echo -e "  Install location: ${INSTALL_DIR}/net-relay"
echo ""

# Get new version
NEW_VERSION=$("${PACKAGE_DIR}/net-relay" --version 2>/dev/null || echo "unknown")
echo -e "  New version: ${GREEN}${NEW_VERSION}${NC}"

# Get current version
if [ -f "${INSTALL_DIR}/net-relay" ]; then
    OLD_VERSION=$("${INSTALL_DIR}/net-relay" --version 2>/dev/null || echo "unknown")
    echo -e "  Current version: ${YELLOW}${OLD_VERSION}${NC}"
fi
echo ""

# Stop service
echo -e "${GREEN}[1/4] Stopping service...${NC}"
if systemctl is-active --quiet "${SERVICE_NAME}" 2>/dev/null; then
    systemctl stop "${SERVICE_NAME}"
    echo -e "  ${GREEN}✓ Service stopped${NC}"
else
    echo -e "  ${YELLOW}Service was not running${NC}"
fi

# Backup current binary
echo -e "${GREEN}[2/4] Backing up current binary...${NC}"
if [ -f "${INSTALL_DIR}/net-relay" ]; then
    cp "${INSTALL_DIR}/net-relay" "${INSTALL_DIR}/net-relay.bak"
    echo -e "  ${GREEN}✓ Backup saved to ${INSTALL_DIR}/net-relay.bak${NC}"
fi

# Copy new binary
echo -e "${GREEN}[3/4] Installing new binary...${NC}"
cp "${PACKAGE_DIR}/net-relay" "${INSTALL_DIR}/net-relay"
chmod 755 "${INSTALL_DIR}/net-relay"
chown "${USER_NAME}:${GROUP_NAME}" "${INSTALL_DIR}/net-relay"

# Copy new example config for reference
if [ -f "${PACKAGE_DIR}/config.example.toml" ]; then
    cp "${PACKAGE_DIR}/config.example.toml" "${INSTALL_DIR}/config.example.toml"
    chown "${USER_NAME}:${GROUP_NAME}" "${INSTALL_DIR}/config.example.toml"
fi

# Update frontend if present
if [ -d "${PACKAGE_DIR}/frontend" ]; then
    cp -r "${PACKAGE_DIR}/frontend" "${INSTALL_DIR}/frontend"
    chown -R "${USER_NAME}:${GROUP_NAME}" "${INSTALL_DIR}/frontend"
    echo -e "  ${GREEN}✓ Frontend updated${NC}"
fi

echo -e "  ${GREEN}✓ Binary updated${NC}"

# Restart service
echo -e "${GREEN}[4/4] Starting service...${NC}"
systemctl daemon-reload
systemctl start "${SERVICE_NAME}"

sleep 2

if systemctl is-active --quiet "${SERVICE_NAME}"; then
    echo -e "  ${GREEN}✓ Service is running${NC}"
else
    echo -e "  ${RED}✗ Service failed to start, rolling back...${NC}"
    if [ -f "${INSTALL_DIR}/net-relay.bak" ]; then
        cp "${INSTALL_DIR}/net-relay.bak" "${INSTALL_DIR}/net-relay"
        systemctl start "${SERVICE_NAME}"
        echo -e "  ${YELLOW}Rolled back to previous version${NC}"
    fi
    echo -e "  Check logs: journalctl -u ${SERVICE_NAME} -n 50 --no-pager"
    exit 1
fi

# Cleanup backup
rm -f "${INSTALL_DIR}/net-relay.bak"

echo ""
echo -e "${GREEN}✓ Upgrade complete!${NC}"
echo -e "  Check status: systemctl status ${SERVICE_NAME}"
echo -e "  Follow logs:  journalctl -u ${SERVICE_NAME} -f"
echo ""
