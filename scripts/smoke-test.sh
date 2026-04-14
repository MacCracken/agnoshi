#!/bin/sh
# smoke-test.sh -- end-to-end smoke test for agnsh
# Exercises CLI flags and common NL inputs. Exits non-zero on failure.

set -e

BIN="${1:-./build/agnsh}"

if [ ! -x "$BIN" ]; then
    echo "Error: $BIN not found or not executable"
    exit 1
fi

PASS=0
FAIL=0
FAILED_TESTS=""

check() {
    name="$1"
    expected="$2"
    actual="$3"
    if echo "$actual" | grep -q "$expected"; then
        PASS=$((PASS + 1))
    else
        FAIL=$((FAIL + 1))
        FAILED_TESTS="$FAILED_TESTS
  FAIL: $name
    expected: $expected
    got:      $actual"
    fi
}

echo "=== agnsh smoke test ==="
echo "Binary: $BIN"
echo ""

# --version
out=$("$BIN" --version 2>&1)
check "version" "agnoshi" "$out"

out=$("$BIN" -v 2>&1)
check "version (-v)" "agnoshi" "$out"

# --help
out=$("$BIN" --help 2>&1)
check "help contains usage" "Usage" "$out"
check "help lists -c" "command" "$out"
check "help lists version" "version" "$out"

out=$("$BIN" -h 2>&1)
check "help (-h)" "Usage" "$out"

# -c COMMAND: intent classification
out=$("$BIN" -c "show me all files" 2>&1)
check "parse show files" "Intent:" "$out"

out=$("$BIN" -c "list running processes" 2>&1)
check "parse list procs" "Intent:" "$out"

out=$("$BIN" -c "git status" 2>&1)
check "parse git status" "Intent:" "$out"

out=$("$BIN" -c "install vim" 2>&1)
check "parse install" "Intent:" "$out"

out=$("$BIN" -c "find files named foo" 2>&1)
check "parse find files" "Intent:" "$out"

out=$("$BIN" -c "remove file.txt" 2>&1)
check "parse remove" "Intent:" "$out"

out=$("$BIN" -c "firewall allow 8080" 2>&1)
check "parse firewall" "Intent:" "$out"

out=$("$BIN" -c "create user alice" 2>&1)
check "parse user add" "Intent:" "$out"

# Error handling
out=$("$BIN" -c 2>&1) || true
check "error on missing -c arg" "Error\|Usage\|required" "$out"

out=$("$BIN" --bogus-flag 2>&1) || true
check "error on bad flag" "Usage" "$out"

# Exit codes
"$BIN" --version >/dev/null 2>&1
ec=$?
if [ $ec -eq 0 ]; then PASS=$((PASS+1)); else FAIL=$((FAIL+1)); FAILED_TESTS="$FAILED_TESTS
  FAIL: --version exit code was $ec"; fi

"$BIN" --help >/dev/null 2>&1
ec=$?
if [ $ec -eq 0 ]; then PASS=$((PASS+1)); else FAIL=$((FAIL+1)); FAILED_TESTS="$FAILED_TESTS
  FAIL: --help exit code was $ec"; fi

# Binary attributes
if file "$BIN" | grep -q "statically linked"; then
    PASS=$((PASS+1))
else
    FAIL=$((FAIL+1))
    FAILED_TESTS="$FAILED_TESTS
  FAIL: binary not statically linked"
fi

SIZE=$(stat -c%s "$BIN" 2>/dev/null || stat -f%z "$BIN" 2>/dev/null)
if [ -n "$SIZE" ] && [ "$SIZE" -lt 524288 ]; then
    PASS=$((PASS+1))
else
    FAIL=$((FAIL+1))
    FAILED_TESTS="$FAILED_TESTS
  FAIL: binary size $SIZE > 512KB limit"
fi

echo ""
echo "Passed: $PASS"
echo "Failed: $FAIL"

if [ $FAIL -gt 0 ]; then
    echo "$FAILED_TESTS"
    exit 1
fi

echo "All smoke tests passed."
