#!/bin/sh
# install.sh -- install agnsh binary to /usr/local/bin
# Usage: sudo sh scripts/install.sh [PREFIX]
#   PREFIX defaults to /usr/local

set -e

PREFIX="${1:-/usr/local}"
BINDIR="$PREFIX/bin"
DATADIR="$PREFIX/share/agnoshi"
MANDIR="$PREFIX/share/man/man1"

# Build if binary doesn't exist
if [ ! -f "build/agnsh" ]; then
    echo "Building agnsh..."
    if ! command -v cyrius >/dev/null 2>&1; then
        echo "Error: cyrius compiler not found in PATH"
        echo "Install from https://github.com/MacCracken/cyrius"
        exit 1
    fi
    mkdir -p build
    cyrius build src/agnsh.cyr build/agnsh
fi

# Verify the binary
if [ ! -x "build/agnsh" ]; then
    echo "Error: build/agnsh is not executable"
    exit 1
fi

echo "Installing agnsh to $BINDIR..."
mkdir -p "$BINDIR" "$DATADIR" "$MANDIR"

cp build/agnsh "$BINDIR/agnsh"
chmod 755 "$BINDIR/agnsh"

# Copy docs
cp README.md "$DATADIR/"
cp CHANGELOG.md "$DATADIR/" 2>/dev/null || true
cp LICENSE "$DATADIR/"

# Install man page if it exists
if [ -f docs/agnsh.1 ]; then
    cp docs/agnsh.1 "$MANDIR/"
    chmod 644 "$MANDIR/agnsh.1"
fi

echo ""
echo "Installation complete."
echo "  Binary: $BINDIR/agnsh"
echo "  Docs:   $DATADIR"
echo ""
echo "Run 'agnsh --version' to verify."
echo "Run 'agnsh --help' for usage."
