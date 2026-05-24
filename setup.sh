#!/bin/bash
set -e

echo "🎵 Music Player - Setup Check"
echo "=============================="
echo ""

# Check for required tools
echo "1. Checking dependencies..."

if ! command -v yt-dlp &> /dev/null; then
    echo "   ❌ yt-dlp not found. Install with: pip install yt-dlp"
    exit 1
fi
echo "   ✓ yt-dlp found: $(yt-dlp --version)"

if ! command -v ffplay &> /dev/null; then
    echo "   ❌ ffplay not found. Install with: sudo apt-get install ffmpeg"
    exit 1
fi
echo "   ✓ ffplay found"

if ! command -v cargo &> /dev/null; then
    echo "   ❌ Cargo not found. Install Rust from https://rustup.rs/"
    exit 1
fi
echo "   ✓ Cargo found: $(cargo --version)"

echo ""
echo "2. Building project..."
cargo build --release

echo ""
echo "✅ All dependencies installed!"
echo ""
echo "To start the player, run:"
echo "  cargo run --release"
echo ""
echo "Or directly:"
echo "  ./target/release/music-player"
