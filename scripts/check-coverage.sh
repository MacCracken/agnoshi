#!/bin/sh
# check-coverage.sh -- agnoshi test-coverage gate
#
# Cyrius doesn't ship line-coverage instrumentation (no probe inserter, no
# .gcov equivalent), so we measure FUNCTION coverage: every top-level `fn`
# compiled into the agnsh binary should be referenced by at least one assertion
# in tests/test_core.tcyr or tests/test_security.tcyr.
#
# ⛔ THE DENOMINATOR IS DERIVED, NOT LISTED. Until 1.9.7 this script carried a
# hardcoded seven-module list and a comment claiming the rest were "reserved for
# the deferred main.cyr wire-up". That stopped being true as modules were wired
# in one by one: approval, audit, history and run_agnos all became part of the
# binary while the gate went on ignoring them, and src/agnsh.cyr itself was
# never counted at all. The gate reported 84% against a real figure of 69% —
# it was measuring the modules it already knew were well covered.
#
# The include list is now read out of src/agnsh.cyr, so wiring a module in
# automatically puts it in scope and the two can no longer drift apart.
#
# ── Two numbers, because one would lie either way ──
# GATED: functions reachable from a host test binary. This is what the
#   threshold applies to.
# AGNOS-ONLY: functions inside `#ifdef CYRIUS_TARGET_AGNOS`. They are absent
#   from a host build, so a host test CANNOT reach them — counting them in the
#   gated denominator would punish the suite for a platform boundary. But they
#   DO ship on agnos, untested, so silently dropping them would hide exactly
#   the gap the roadmap tracks as verification debt. They are reported
#   separately and loudly instead.
#
# Excluded from the denominator (entry scaffolding, not library code):
#   main / _entry / _agnos_entry / print_* / interactive_loop / read_line
#   and the ui_show_* + chrono_now_rfc3339 shims in src/agnsh.cyr.
#
# Usage: sh scripts/check-coverage.sh [threshold-percent]   (default 80)

set -e

THRESHOLD="${1:-80}"

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

ENTRY="src/agnsh.cyr"

# Derive the module set from the entry's own include list, plus the entry.
IN_BINARY_FILES=$(grep -oE '^include "src/[^"]+"' "$ENTRY" \
    | sed 's/^include "//; s/"$//' | tr '\n' ' ')
IN_BINARY_FILES="$IN_BINARY_FILES $ENTRY"

# Emit "<file> <fn> <host|agnos>" for every top-level fn, tracking whether the
# definition sits inside a CYRIUS_TARGET_AGNOS block.
FN_TABLE=$(awk '
    FNR == 1 { depth = 0; agnos = 0 }
    /^[ \t]*#ifdef[ \t]+CYRIUS_TARGET_AGNOS/ { depth++; agnos = 1; next }
    /^[ \t]*#ifdef/ || /^[ \t]*#ifndef/ { if (agnos) depth++; next }
    /^[ \t]*#endif/ { if (agnos) { depth--; if (depth <= 0) agnos = 0 } next }
    /^fn [A-Za-z_][A-Za-z0-9_]*\(/ {
        match($0, /^fn [A-Za-z_][A-Za-z0-9_]*/)
        name = substr($0, RSTART + 3, RLENGTH - 3)
        print FILENAME " " name " " (agnos ? "agnos" : "host")
    }
' $IN_BINARY_FILES 2>/dev/null)

EXCLUDE_RE='^(main|_entry|_agnos_entry|print_usage|print_version|print_intent_result|interactive_loop|read_line|ui_show_error|ui_show_warning|chrono_now_rfc3339)$'

TOTAL=0
TESTED=0
UNTESTED=""
AGNOS_TOTAL=0
AGNOS_UNTESTED=""

# Field 2 = fn name, field 3 = host|agnos.
echo "$FN_TABLE" | sort -u -k2,2 | while read -r _f _n _s; do
    :
done

for row in $(echo "$FN_TABLE" | sort -u -k2,2 | awk '{print $2 ":" $3}'); do
    fn=${row%:*}
    scope=${row#*:}
    echo "$fn" | grep -qE "$EXCLUDE_RE" && continue
    if [ "$scope" = "agnos" ]; then
        AGNOS_TOTAL=$((AGNOS_TOTAL + 1))
        if ! grep -qwE "$fn" tests/test_core.tcyr tests/test_security.tcyr 2>/dev/null; then
            AGNOS_UNTESTED="$AGNOS_UNTESTED $fn"
        fi
        continue
    fi
    TOTAL=$((TOTAL + 1))
    if grep -qwE "$fn" tests/test_core.tcyr tests/test_security.tcyr 2>/dev/null; then
        TESTED=$((TESTED + 1))
    else
        UNTESTED="$UNTESTED $fn"
    fi
done

if [ "$TOTAL" -eq 0 ]; then
    echo "ERROR: no fns discovered in the include graph — coverage check broken"
    exit 1
fi

PERCENT=$(( (TESTED * 100) / TOTAL ))

echo "agnoshi test coverage (fn-level):"
echo "  modules in scope: $(echo $IN_BINARY_FILES | wc -w) (derived from $ENTRY)"
echo "  host-reachable:   $TESTED / $TOTAL ($PERCENT%)"
echo "  threshold:        ${THRESHOLD}%"

if [ -n "$UNTESTED" ]; then
    echo "  untested (host-reachable):"
    for fn in $UNTESTED; do echo "    - $fn"; done
fi

AGNOS_UNTESTED_N=$(echo $AGNOS_UNTESTED | wc -w)
if [ "$AGNOS_TOTAL" -gt 0 ]; then
    echo ""
    echo "  agnos-only fns (absent from a host build, NOT gated): $AGNOS_TOTAL"
    echo "    of which untested: $AGNOS_UNTESTED_N"
    echo "    These need an agnos smoke run on iron, not a host unit test."
    echo "    Tracked as verification debt in docs/development/roadmap.md."
fi

if [ "$PERCENT" -lt "$THRESHOLD" ]; then
    echo ""
    echo "FAIL: host-reachable coverage $PERCENT% < $THRESHOLD% threshold"
    exit 1
fi

echo ""
echo "OK: coverage gate passed"
