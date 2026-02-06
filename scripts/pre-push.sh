#!/bin/bash
# Pre-push check script - Run this before pushing to ensure CI will pass

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}🔍 Running pre-push checks...${NC}\n"

# 1. Format check
echo -e "${YELLOW}[1/4] Checking code format...${NC}"
if cargo fmt --check; then
    echo -e "${GREEN}✓ Format check passed${NC}\n"
else
    echo -e "${RED}✗ Format check failed. Run 'cargo fmt' to fix.${NC}"
    exit 1
fi

# 2. Clippy
echo -e "${YELLOW}[2/4] Running clippy...${NC}"
if cargo clippy -- -D warnings; then
    echo -e "${GREEN}✓ Clippy check passed${NC}\n"
else
    echo -e "${RED}✗ Clippy found issues${NC}"
    exit 1
fi

# 3. Build check
echo -e "${YELLOW}[3/4] Checking build...${NC}"
if cargo build --release; then
    echo -e "${GREEN}✓ Build check passed${NC}\n"
else
    echo -e "${RED}✗ Build failed${NC}"
    exit 1
fi

# 4. Tests
echo -e "${YELLOW}[4/4] Running tests...${NC}"
if cargo test; then
    echo -e "${GREEN}✓ All tests passed${NC}\n"
else
    echo -e "${RED}✗ Tests failed${NC}"
    exit 1
fi

echo -e "${GREEN}════════════════════════════════════════${NC}"
echo -e "${GREEN}✓ All checks passed! Safe to push.${NC}"
echo -e "${GREEN}════════════════════════════════════════${NC}"
