#!/bin/sh
# check-fmt.sh -- agnoshi format-drift gate, mirroring the CI step exactly.
#
# ⛔ WHY THIS SCRIPT EXISTS: `cyrius fmt` SILENTLY IGNORES EVERY FILE AFTER THE
# FIRST. Both forms do it, and the rewrite form is the more dangerous of the two:
#
#   cyrius fmt --check a.cyr b.cyr   -> checks ONLY a.cyr, exits 0 if a.cyr is
#                                       clean, no matter how badly b.cyr drifts
#   cyrius fmt a.cyr b.cyr           -> reformats ONLY a.cyr and leaves b.cyr
#                                       untouched, while looking like it worked
#
# A local `cyrius fmt --check src/*.cyr tests/*.tcyr` therefore checks exactly
# one file and reports success — which is how 1.9.10 reached CI with drift in
# tests/test_core.tcyr after a "clean" local sweep. CI has always looped
# per-file and caught it; the gap was between CI and what a developer types.
#
# Every other gate in this repo is a script you can run locally (lint-cstr-str,
# check-coverage, bench-history). Format was the one that still had to be
# hand-typed as a shell loop, so it was the one that got typed wrong. It isn't
# any more: run this, not a bare `cyrius fmt --check`.
#
# ⚠ VERIFIED, NOT ASSUMED: this script's own failure path is exercised by
# `--selftest`, which plants drift in a scratch copy and asserts the gate
# reports it. A gate nobody has watched fail is not a gate — the 1.9.10 audit
# path override (lint Category H) and the coverage gate were both checked this
# way, and this one is too.
#
# Usage:
#   sh scripts/check-fmt.sh              # check every file; exit 1 on drift
#   sh scripts/check-fmt.sh --fix        # reformat the drifting files in place
#   sh scripts/check-fmt.sh --selftest   # prove the gate can fail

set -eu

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

MODE="${1:-check}"

# Same globs as .github/workflows/ci.yml, so local and CI cannot disagree.
files() {
    for f in src/*.cyr tests/*.cyr tests/*.tcyr tests/*.bcyr; do
        [ -f "$f" ] && echo "$f"
    done
}

if [ "$MODE" = "--selftest" ]; then
    # Plant drift in a scratch copy and confirm the per-file check reports it,
    # AND that the multi-file form does not — the blind spot this script exists
    # to route around. If cyrius ever fixes that, this selftest says so.
    tmp="${TMPDIR:-/tmp}/agnsh_fmt_selftest_$$.cyr"
    clean="${TMPDIR:-/tmp}/agnsh_fmt_selftest_clean_$$.cyr"
    printf 'fn _selftest_clean(x) {\n    return 0;\n}\n' > "$clean"
    printf 'fn _selftest_drift(x) {\n        return 0;\n}\n' > "$tmp"

    rc_single=0
    cyrius fmt --check "$tmp" >/dev/null 2>&1 || rc_single=$?
    rc_multi=0
    cyrius fmt --check "$clean" "$tmp" >/dev/null 2>&1 || rc_multi=$?
    rm -f "$tmp" "$clean"

    if [ "$rc_single" -eq 0 ]; then
        echo "check-fmt --selftest: FAIL -- planted drift was not reported."
        echo "  The per-file check no longer detects drift; this gate is blind."
        exit 1
    fi
    echo "check-fmt --selftest: per-file check correctly reported planted drift."

    if [ "$rc_multi" -eq 0 ]; then
        echo "check-fmt --selftest: confirmed -- the multi-file form still"
        echo "  ignores everything after the first file. Keep using this script."
    else
        echo "check-fmt --selftest: NOTE -- the multi-file form now reports drift"
        echo "  too. cyrius appears to have fixed it; this script is still the"
        echo "  gate, but the warning above can be revisited."
    fi
    exit 0
fi

FAIL=0
DRIFTED=""

for f in $(files); do
    if ! cyrius fmt --check "$f" >/dev/null 2>&1; then
        DRIFTED="$DRIFTED $f"
        FAIL=1
    fi
done

if [ "$FAIL" -eq 0 ]; then
    echo "check-fmt: clean ($(files | wc -l | tr -d ' ') files)"
    exit 0
fi

if [ "$MODE" = "--fix" ]; then
    for f in $DRIFTED; do
        # One invocation per file — see the banner. A single `cyrius fmt $DRIFTED`
        # would reformat only the first and leave the rest, reporting success.
        cyrius fmt "$f"
        echo "formatted: $f"
    done
    # Re-check, because "the formatter ran" is not the same claim as "the file
    # is now canonical".
    for f in $DRIFTED; do
        if ! cyrius fmt --check "$f" >/dev/null 2>&1; then
            echo "check-fmt: FAIL -- $f still drifts after formatting"
            exit 1
        fi
    done
    echo "check-fmt: all drift fixed"
    exit 0
fi

echo "check-fmt: drift detected"
for f in $DRIFTED; do
    echo ""
    cyrius fmt --check "$f" 2>&1 || true
done
echo ""
echo "Fix with:  sh scripts/check-fmt.sh --fix"
echo "⚠ Do NOT run 'cyrius fmt <a> <b> ...' -- it only touches the first file."
exit 1
