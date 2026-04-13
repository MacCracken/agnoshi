#!/bin/sh
CC="${1:-./build/cc3}"
echo "=== agnoshi tests ==="
cat src/main.cyr | "$CC" > /tmp/agnoshi_test && chmod +x /tmp/agnoshi_test && /tmp/agnoshi_test
echo "exit: $?"
rm -f /tmp/agnoshi_test
