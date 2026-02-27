#!/bin/bash
# Install net-relay as a systemd service
# Usage: sudo ./install-service.sh [install_dir]
#
# This script will:
#   1. Create a dedicated 'net-relay' system user
#   2. Copy files to /opt/net-relay (or specified directory)
#   3. Install and enable the systemd service
#   4. Start the service
#
# After installation, manage with:
#   systemctl start|stop|restart|status net-relay
#   journalctl -u net-relay -f

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

# Determine script location (where the release package is)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# The release package root is one level up from scripts/
PACKAGE_DIR="$(dirname "$SCRIPT_DIR")"

INSTALL_DIR="${1:-/opt/net-relay}"
SERVICE_NAME="net-relay"
SERVICE_FILE="/etc/systemd/system/${SERVICE_NAME}.service"
USER_NAME="net-relay"
GROUP_NAME="net-relay"

echo -e "${BLUE}╔══════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   Net-Relay Systemd Service Setup    ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════╝${NC}"
echo ""

# --- Step 1: Create system user ---
echo -e "${GREEN}[1/5] Creating system user '${USER_NAME}'...${NC}"
if id "$USER_NAME" &>/dev/null; then
    echo -e "  ${YELLOW}User '${USER_NAME}' already exists, skipping${NC}"
else
    useradd --system --no-create-home --shell /usr/sbin/nologin "$USER_NAME"
    echo -e "  ${GREEN}✓ User '${USER_NAME}' created${NC}"
fi

# --- Step 2: Create install directory and copy files ---
echo -e "${GREEN}[2/5] Installing files to ${INSTALL_DIR}...${NC}"
mkdir -p "${INSTALL_DIR}/logs"

# Copy binary
if [ -f "${PACKAGE_DIR}/net-relay" ]; then
    cp "${PACKAGE_DIR}/net-relay" "${INSTALL_DIR}/net-relay"
    chmod 755 "${INSTALL_DIR}/net-relay"
    echo -e "  ${GREEN}✓ Binary copied${NC}"
else
    echo -e "  ${RED}✗ Binary not found at ${PACKAGE_DIR}/net-relay${NC}"
    exit 1
fi

# Copy config (don't overwrite existing)
if [ ! -f "${INSTALL_DIR}/config.toml" ]; then
    if [ -f "${PACKAGE_DIR}/config.toml" ]; then
        cp "${PACKAGE_DIR}/config.toml" "${INSTALL_DIR}/config.toml"
        echo -e "  ${GREEN}✓ config.toml copied${NC}"
    elif [ -f "${PACKAGE_DIR}/config.example.toml" ]; then
        cp "${PACKAGE_DIR}/config.example.toml" "${INSTALL_DIR}/config.toml"
        echo -e "  ${YELLOW}✓ config.example.toml copied as config.toml (please review and edit)${NC}"
    fi
else
    echo -e "  ${YELLOW}config.toml already exists, not overwriting${NC}"
fi

# Copy example config for reference
if [ -f "${PACKAGE_DIR}/config.example.toml" ]; then
    cp "${PACKAGE_DIR}/config.example.toml" "${INSTALL_DIR}/config.example.toml"
fi

# Copy frontend if exists
if [ -d "${PACKAGE_DIR}/frontend" ]; then
    cp -r "${PACKAGE_DIR}/frontend" "${INSTALL_DIR}/frontend"
    echo -e "  ${GREEN}✓ Frontend files copied${NC}"
fi

# Set ownership
chown -R "${USER_NAME}:${GROUP_NAME}" "${INSTALL_DIR}"
echo -e "  ${GREEN}✓ Ownership set to ${USER_NAME}:${GROUP_NAME}${NC}"

# --- Step 3: Install systemd service ---
echo -e "${GREEN}[3/5] Installing systemd service...${NC}"

# Generate service file with correct paths
cat > "$SERVICE_FILE" <<EOF
[Unit]
Description=Net-Relay Proxy Server
Documentation=https://github.com/yourusername/net-relay
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=${USER_NAME}
Group=${GROUP_NAME}

# Paths
ExecStart=${INSTALL_DIR}/net-relay -c ${INSTALL_DIR}/config.toml
WorkingDirectory=${INSTALL_DIR}

# Restart policy
Restart=on-failure
RestartSec=5
StartLimitIntervalSec=60
StartLimitBurst=5

# Environment
Environment=RUST_LOG=info

# Logging - use journal
StandardOutput=journal
StandardError=journal
SyslogIdentifier=net-relay

# Security hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
PrivateTmp=true
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectControlGroups=true
ReadWritePaths=${INSTALL_DIR}/logs

# Resource limits
LimitNOFILE=65535
LimitNPROC=4096

[Install]
WantedBy=multi-user.target
EOF

echo -e "  ${GREEN}✓ Service file installed to ${SERVICE_FILE}${NC}"

# --- Step 4: Enable and start ---
echo -e "${GREEN}[4/5] Enabling service...${NC}"
systemctl daemon-reload
systemctl enable "${SERVICE_NAME}"
echo -e "  ${GREEN}✓ Service enabled (will start on boot)${NC}"

echo -e "${GREEN}[5/5] Starting service...${NC}"
# Stop existing service if running
systemctl stop "${SERVICE_NAME}" 2>/dev/null || true
systemctl start "${SERVICE_NAME}"

sleep 2

if systemctl is-active --quiet "${SERVICE_NAME}"; then
    echo -e "  ${GREEN}✓ net-relay is running${NC}"
else
    echo -e "  ${RED}✗ net-relay failed to start${NC}"
    echo -e "  Check logs with: journalctl -u ${SERVICE_NAME} -n 50 --no-pager"
    exit 1
fi

# --- Summary ---
echo ""
echo -e "${BLUE}══════════════════════════════════════${NC}"
echo -e "${GREEN}✓ Installation complete!${NC}"
echo -e "${BLUE}══════════════════════════════════════${NC}"
echo ""
echo -e "  Install dir:  ${INSTALL_DIR}"
echo -e "  Config file:  ${INSTALL_DIR}/config.toml"
echo -e "  Log dir:      ${INSTALL_DIR}/logs"
echo -e "  Service file: ${SERVICE_FILE}"
echo ""
echo -e "  ${YELLOW}Management commands:${NC}"
echo -e "    systemctl start   ${SERVICE_NAME}    # Start"
echo -e "    systemctl stop    ${SERVICE_NAME}    # Stop"
echo -e "    systemctl restart ${SERVICE_NAME}    # Restart"
echo -e "    systemctl status  ${SERVICE_NAME}    # Status"
echo -e "    journalctl -u ${SERVICE_NAME} -f     # Follow logs"
echo ""
echo -e "  ${YELLOW}To edit config:${NC}"
echo -e "    sudo vim ${INSTALL_DIR}/config.toml"
echo -e "    sudo systemctl restart ${SERVICE_NAME}"
echo ""
