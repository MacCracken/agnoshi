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

# Approval wiring -- every -c output now carries a "Risk: [LEVEL]"
# line (assessed via risk_from_permission). BLOCKED commands surface
# a WARNING line; HIGH-risk ones note the approval requirement.
out=$("$BIN" -c "show me files" 2>&1)
check "risk LOW for read-only" "Risk: \[LOW\]" "$out"

out=$("$BIN" -c "copy a to b" 2>&1)
check "risk MED for user-write" "Risk: \[MED\]" "$out"

out=$("$BIN" -c "install vim" 2>&1)
check "risk HIGH for admin" "Risk: \[HIGH\]" "$out"
check "high-risk approval hint" "Approval required" "$out"

out=$("$BIN" -c "rm -rf /tmp/foo" 2>&1)
check "risk CRIT for blocked" "Risk: \[CRIT\]" "$out"
check "blocked warning line" "WARNING: BLOCKED" "$out"

# Command field populated -- the cstring/Str print mismatch that left
# this blank pre-v1.2.1 is now fixed.
out=$("$BIN" -c "show me files" 2>&1)
check "command field has ls" "Command: ls" "$out"

# Interactive mode -- drive via stdin pipe and check that the mode-
# switching builtins flow correctly and the prompt updates. Each line
# of input is one user turn (the read_line helper accepts byte-by-byte
# stdin so piped multi-line blobs no longer collapse into one buffer).
INT_OUT=$(printf 'mode\nmode human\nmode\nmode strict\nshow files\nexit\n' | "$BIN" 2>&1)
check "interactive shows assist start" "\[ASSIST\] >" "$INT_OUT"
check "interactive mode reports current" "Current mode: AI-ASSIST" "$INT_OUT"
check "interactive mode switch to human" "Mode -> HUMAN" "$INT_OUT"
check "interactive prompt updates after switch" "\[HUMAN\] >" "$INT_OUT"
check "interactive mode switch to strict" "Mode -> STRICT" "$INT_OUT"
check "interactive parses NL under mode" "Intent:" "$INT_OUT"
check "interactive exits cleanly" "bye" "$INT_OUT"

# Interactive negative -- unknown mode name should error, not crash,
# and the available list must surface for discoverability.
BAD_OUT=$(printf 'mode wibble\nexit\n' | "$BIN" 2>&1)
check "unknown mode error" "Unknown mode: wibble" "$BAD_OUT"
check "unknown mode suggests list" "Available: auto, assist, human, strict" "$BAD_OUT"

# Audit log -- every -c invocation appends a JSON line to
# $HOME/.agnsh_audit.log. Point HOME at a clean temp dir, run two
# commands, verify the log has well-formed lines with the expected
# action+approved shape.
SMOKE_HOME=$(mktemp -d -t agnsh-smoke.XXXXXX)
HOME="$SMOKE_HOME" "$BIN" -c "show me files" >/dev/null 2>&1
HOME="$SMOKE_HOME" "$BIN" -c "rm -rf /tmp/x" >/dev/null 2>&1
LOG="$SMOKE_HOME/.agnsh_audit.log"
if [ -f "$LOG" ]; then
    PASS=$((PASS+1))
else
    FAIL=$((FAIL+1))
    FAILED_TESTS="$FAILED_TESTS
  FAIL: audit log not created at $LOG"
fi
log_content=$(cat "$LOG" 2>/dev/null)
check "audit log has approved=1 for ls" '"action":"ls","approved":1' "$log_content"
check "audit log has approved=0 for rm" '"action":"rm","approved":0' "$log_content"
lines=$(wc -l < "$LOG" 2>/dev/null || echo 0)
check "audit log lines match invocations" "2" "$lines"
rm -rf "$SMOKE_HOME"

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
