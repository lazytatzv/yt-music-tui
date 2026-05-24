#!/bin/bash
set -e

# Colors for a fashionable output
PINK='\033[0;35m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${PINK}
 ❯  Y T - T U I   I N S T A L L E R
══════════════════════════════════════
${NC}"

# Function to check dependency
check_dep() {
    if ! command -v $1 &> /dev/null; then
        echo -e "   ${RED}❌ $1 not found.${NC}"
        return 1
    fi
    echo -e "   ${GREEN}✓ $1 found${NC} (${CYAN}$($1 --version | head -n 1)${NC})"
    return 0
}

echo -e "${BLUE}1. Scanning system environment...${NC}"

MISSING=0

# Check for Cargo
if ! check_dep cargo; then
    echo -e "      ${YELLOW}Please install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh${NC}"
    MISSING=1
fi

# Check for mpv
if ! check_dep mpv; then
    echo -e "      ${YELLOW}Please install mpv: sudo apt install mpv (or brew install mpv)${NC}"
    MISSING=1
fi

# Check for yt-dlp
if ! check_dep yt-dlp; then
    echo -e "      ${YELLOW}Please install yt-dlp: sudo apt install yt-dlp (or brew install yt-dlp)${NC}"
    MISSING=1
fi

if [ $MISSING -eq 1 ]; then
    echo -e "\n   ${RED}Installation aborted. Please install the missing dependencies above and try again.${NC}"
    exit 1
fi

echo -e "\n${BLUE}2. Constructing YT-TUI Studio...${NC}"
echo -e "   ${CYAN}This may take a moment while we optimize the binaries.${NC}"

cargo build --release

echo -e "\n${PINK}
 ❯  I N S T A L L A T I O N   C O M P L E T E
══════════════════════════════════════════════
${NC}"

echo -e "   ${GREEN}Success!${NC} The studio is now ready."
echo -e "\n   ${BLUE}To launch the player:${NC}"
echo -e "      ${YELLOW}./target/release/yt-tui${NC}"

echo -e "\n   ${BLUE}Pro tip:${NC}"
echo -e "      You can create a symlink to use it from anywhere:"
echo -e "      ${CYAN}sudo ln -s $(pwd)/target/release/yt-tui /usr/local/bin/yt-tui${NC}"

echo -e "\n${PINK}   Enjoy your aesthetic audio journey. ✦${NC}\n"
