#!/bin/sh
# uninstall.sh -- remove agnsh
# Usage: sudo sh scripts/uninstall.sh [PREFIX]

set -e

PREFIX="${1:-/usr/local}"
BINDIR="$PREFIX/bin"
DATADIR="$PREFIX/share/agnoshi"
MANDIR="$PREFIX/share/man/man1"

rm -f "$BINDIR/agnsh"
rm -rf "$DATADIR"
rm -f "$MANDIR/agnsh.1"

echo "Uninstalled agnsh from $PREFIX"
