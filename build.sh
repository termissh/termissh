#!/usr/bin/env bash
# termissh local build script
# Run this on Linux/WSL (with cargo installed) to build the Linux binary.
# For Windows binary: run `cargo build --release` in PowerShell/CMD.
# For both platforms at once: push to GitHub and use the CI workflow.
set -euo pipefail

BINARY_NAME="termissh"
TARGET="x86_64-unknown-linux-musl"
DIST_DIR="dist"

echo "[*] Building $BINARY_NAME for $TARGET..."

if ! command -v cargo &>/dev/null; then
    echo "[!] cargo not found. Install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

if ! command -v musl-gcc &>/dev/null; then
    echo "[!] musl-tools not found. Install it:"
    echo "    sudo apt-get install musl-tools   # Debian/Ubuntu"
    echo "    sudo pacman -S musl               # Arch"
    exit 1
fi

if ! rustup target list --installed | grep -q "$TARGET"; then
    echo "[*] Adding rustup target $TARGET..."
    rustup target add "$TARGET"
fi

cargo build --release --target "$TARGET"

mkdir -p "$DIST_DIR"
cp "target/$TARGET/release/$BINARY_NAME" "$DIST_DIR/$BINARY_NAME-linux-x86_64"

echo "[+] Done: $DIST_DIR/$BINARY_NAME-linux-x86_64"
