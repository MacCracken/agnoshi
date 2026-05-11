#!/bin/sh
# check-coverage.sh -- agnoshi test-coverage gate
#
# Cyrius doesn't ship line-coverage instrumentation (no probe inserter,
# no .gcov equivalent), so we measure function coverage: every top-level
# `fn` defined in src/*.cyr should be referenced by at least one assertion
# in tests/test_core.tcyr or tests/test_security.tcyr. The threshold is
# the v1.2.0 roadmap target (>=80%). CI fails below threshold.
#
# Excludes (intentionally not counted toward coverage):
# - `fn main()` and `fn print_*` entry/io scaffolding in src/agnsh.cyr
# - The 4 cc3-era stubs in src/agnsh.cyr (ui_show_error, ui_show_warning,
#   chrono_now_rfc3339) — placeholder shims for modules not pulled into
#   the binary's include graph; they get covered when src/main.cyr is
#   wired in at 1.2.x.
# - The duplicate-helper block in src/main.cyr (legacy pre-port entry,
#   never linked into the agnsh binary; queued for removal).
#
# Usage: sh scripts/check-coverage.sh [threshold-percent]
#   threshold defaults to 80.

set -e

THRESHOLD="${1:-80}"

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# Scope: modules linked into the agnsh binary (per src/agnsh.cyr's
# include list). Modules reserved for the deferred main.cyr full
# wire-up (session, ui, prompt, checkpoint, audit, history, aliases,
# completion, output, security, approval, config) are excluded from
# the denominator — they have their own follow-up coverage slot in
# the 1.2.x interactive-shell work.
IN_BINARY_FILES="src/sanitize.cyr src/mode.cyr src/permissions.cyr src/intent.cyr src/commands.cyr src/translate.cyr src/interpreter.cyr"

ALL_FNS=$(awk '
    /^fn [A-Za-z_][A-Za-z0-9_]*\(/ {
        match($0, /^fn [A-Za-z_][A-Za-z0-9_]*/)
        print substr($0, RSTART+3, RLENGTH-3)
    }
' $IN_BINARY_FILES 2>/dev/null | grep -v '^$' | sort -u)

# Exclude entry / scaffolding fns from the denominator.
EXCLUDE_RE='^(main|print_usage|print_version|print_intent_result|ui_show_error|ui_show_warning|chrono_now_rfc3339)$'

COUNTED_FNS=$(echo "$ALL_FNS" | grep -vE "$EXCLUDE_RE" || true)

TOTAL=0
TESTED=0
UNTESTED=""

for fn in $COUNTED_FNS; do
    TOTAL=$((TOTAL + 1))
    if grep -qwE "$fn" tests/test_core.tcyr tests/test_security.tcyr 2>/dev/null; then
        TESTED=$((TESTED + 1))
    else
        UNTESTED="$UNTESTED $fn"
    fi
done

if [ "$TOTAL" -eq 0 ]; then
    echo "ERROR: no fns discovered in src/ — coverage check broken"
    exit 1
fi

PERCENT=$(( (TESTED * 100) / TOTAL ))

echo "agnoshi test coverage (fn-level):"
echo "  tested:   $TESTED / $TOTAL ($PERCENT%)"
echo "  threshold: ${THRESHOLD}%"

if [ -n "$UNTESTED" ]; then
    echo "  untested fns:"
    for fn in $UNTESTED; do
        echo "    - $fn"
    done
fi

if [ "$PERCENT" -lt "$THRESHOLD" ]; then
    echo "FAIL: coverage $PERCENT% < $THRESHOLD% threshold"
    exit 1
fi

echo "OK: coverage gate passed"
