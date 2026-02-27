#!/bin/bash
# Uninstall net-relay systemd service
# Usage: sudo ./uninstall-service.sh [install_dir]
#
# This script will:
#   1. Stop and disable the systemd service
#   2. Remove the service file
#   3. Optionally remove the installation directory
#   4. Optionally remove the system user

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

INSTALL_DIR="${1:-/opt/net-relay}"
SERVICE_NAME="net-relay"
SERVICE_FILE="/etc/systemd/system/${SERVICE_NAME}.service"
USER_NAME="net-relay"

echo -e "${BLUE}╔══════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Net-Relay Service Uninstallation    ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════╝${NC}"
echo ""

# --- Step 1: Stop and disable service ---
echo -e "${GREEN}[1/3] Stopping and disabling service...${NC}"
if systemctl is-active --quiet "${SERVICE_NAME}" 2>/dev/null; then
    systemctl stop "${SERVICE_NAME}"
    echo -e "  ${GREEN}✓ Service stopped${NC}"
else
    echo -e "  ${YELLOW}Service is not running${NC}"
fi

if systemctl is-enabled --quiet "${SERVICE_NAME}" 2>/dev/null; then
    systemctl disable "${SERVICE_NAME}"
    echo -e "  ${GREEN}✓ Service disabled${NC}"
else
    echo -e "  ${YELLOW}Service is not enabled${NC}"
fi

# --- Step 2: Remove service file ---
echo -e "${GREEN}[2/3] Removing service file...${NC}"
if [ -f "$SERVICE_FILE" ]; then
    rm -f "$SERVICE_FILE"
    systemctl daemon-reload
    echo -e "  ${GREEN}✓ Service file removed${NC}"
else
    echo -e "  ${YELLOW}Service file not found${NC}"
fi

# --- Step 3: Ask about files and user ---
echo -e "${GREEN}[3/3] Cleanup...${NC}"

# Ask to remove install dir
if [ -d "$INSTALL_DIR" ]; then
    echo -e ""
    read -p "  Remove installation directory ${INSTALL_DIR}? [y/N] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        rm -rf "$INSTALL_DIR"
        echo -e "  ${GREEN}✓ Installation directory removed${NC}"
    else
        echo -e "  ${YELLOW}Installation directory kept at ${INSTALL_DIR}${NC}"
    fi
fi

# Ask to remove user
if id "$USER_NAME" &>/dev/null; then
    echo -e ""
    read -p "  Remove system user '${USER_NAME}'? [y/N] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        userdel "$USER_NAME" 2>/dev/null || true
        echo -e "  ${GREEN}✓ User '${USER_NAME}' removed${NC}"
    else
        echo -e "  ${YELLOW}User '${USER_NAME}' kept${NC}"
    fi
fi

echo ""
echo -e "${GREEN}✓ Net-relay service uninstalled${NC}"
echo ""
