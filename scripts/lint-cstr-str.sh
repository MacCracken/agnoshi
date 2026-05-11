#!/bin/sh
# lint-cstr-str.sh -- static analyzer for Cyrius cstring/Str type
# mismatches that the toolchain can't catch.
#
# Background. The Cyrius 4.5 → 5.10 stdlib refactor split many string
# helpers into Str-typed and cstring-typed variants without unifying
# the type system, so a function signature like
# `fn str_cat(a: Str, b: Str): Str` will happily accept a cstring
# literal — it compiles, and the runtime reads `load64(s+8)` (a Str
# length) off the cstring's address (garbage). The bug surfaces as a
# silent runtime fallthrough (helper returns 0 / never matches) or a
# SIGSEGV on the next allocation. Five separate variants have bitten
# agnoshi across v1.2.0 / v1.3.0:
#
#   1. `str_len("cstr")` — sanitize CI helpers (v1.2.0 slice 1)
#   2. `str_sub(s, start, end)` end→length semantics drift (v1.2.0 s1)
#   3. `str_cat("cstr", *)` — slices 7 of v1.2.0 + 8 of v1.3.0
#   4. `str_cat(*, "cstr")` — slice 2 of v1.3.0 (the audit module)
#   5. `is_safe_path(Str)` cstring-vs-Str mismatch (v1.3.0 slice 3) —
#      this one had silently routed every NL filesystem op to
#      `translate_unknown` since v1.0
#
# Each one was discovered the hard way (probe / SIGSEGV / first-use
# crash). The script greps for the antipatterns we know about and
# fails CI when one appears.
#
# Escape hatch. A trailing `# lint:cstr-ok` comment on the offending
# line marks an intentional use. Use sparingly — these are real bugs.
#
# Usage: sh scripts/lint-cstr-str.sh [src_dir...]
#   defaults to src/. Exits 0 on clean, 1 on any hit.
#
# Pattern categories (see below for the literal grep regexes):
#   A. Str-typed-arg fn called with a cstring LITERAL
#   B. Str-typed-arg fn called with a cstring LITERAL as second arg
#   C. cross-arch-broken raw syscalls
#
# Category B uses a tighter regex to avoid matching `str_cat(a, b)` where
# both are vars; we only flag `str_cat(*, "literal-here")` shapes.
# Category C catches `syscall(SYS_OPEN, ...)`, `syscall(SYS_CHMOD, ...)`,
# `syscall(SYS_STAT, ...)` — all defined in x86_64's table but missing
# in aarch64's (aarch64 uses openat / fchmodat / no-bare-stat). The
# v1.3.0 closeout caught these via the CI cross-build failure; this
# linter catches them at lint time.
#
# What it does NOT catch.
# - Tainted Str values flowing into cstring-typed fns (would need
#   data-flow analysis; Cyrius 5.10.x's type-warning hint catches some
#   of these — see `streq(Str, Str)` in security.cyr slice 5)
# - Custom Str-typed fns defined elsewhere; the list below is the
#   stdlib set we depend on
# - str_sub end-vs-length semantic drift (a semantic bug, not a type
#   mismatch; covered by the existing `str_substr` migration)

set -e

SRC_DIRS="${@:-src}"
FAIL=0
HITS=""

# Helper: grep a pattern, filter comments, filter the `# lint:cstr-ok`
# escape hatch, report each hit.
scan() {
    label="$1"
    pattern="$2"
    found=$(grep -nE "$pattern" $SRC_DIRS/*.cyr 2>/dev/null | \
            grep -v '^[^:]*:[0-9]*:\s*#' | \
            grep -v 'lint:cstr-ok' || true)
    if [ -n "$found" ]; then
        HITS="$HITS
$label:
$found"
        FAIL=1
    fi
}

# All patterns use `\b` word anchors so e.g. `cstr_starts_with` (a
# custom cstring-typed helper in permissions.cyr) doesn't match the
# `str_starts_with` regex.
#
# Coverage rationale — only flag stdlib fns we've confirmed are BUGGY
# when passed a cstring. Cyrius dispatches several Str-typed fns to
# their `_cstr` counterpart automatically (`strlen` → `strlen_str`,
# `str_contains` → `str_contains_cstr`, `str_eq` → `str_eq_cstr`,
# `str_split` → `str_split_cstr`) so those calls are safe even though
# the signature is typed Str. Fns WITHOUT a `_cstr` overload are the
# ones that bite:
#   str_len    — Str-only primitive (use `strlen` for cstring)
#   str_data   — Str-only primitive
#   str_cat    — no str_cat_cstr in lib/str.cyr (verified)
#   str_starts_with — no str_starts_with_cstr (verified)
#   str_ends_with   — no str_ends_with_cstr (verified)

# Category A — first-arg cstring literal.
scan "str_len(cstring)"        '\bstr_len\("'
scan "str_data(cstring)"       '\bstr_data\("'
scan "str_cat(cstring, *)"     '\bstr_cat\("'
scan "str_starts_with(cstring, *)" '\bstr_starts_with\("'
scan "str_ends_with(cstring, *)"   '\bstr_ends_with\("'

# Category B — second-arg cstring literal. `\b` word-anchor in front
# of the fn name keeps `cstr_starts_with` and other custom helpers
# from getting flagged.
scan "str_cat(*, cstring)"     '\bstr_cat\([^"]+,\s*"'
scan "str_starts_with(*, cstring)" '\bstr_starts_with\([^"]+,\s*"'
scan "str_ends_with(*, cstring)"   '\bstr_ends_with\([^"]+,\s*"'

# Category C — raw syscalls that work on x86_64 but break on aarch64
# (which has no bare SYS_OPEN/CHMOD/STAT — uses openat/fchmodat/stat-
# layout-shifted). Use the lib/io.cyr wrappers (`sys_open`, `sys_chmod`,
# `sys_stat`) — both arches export them. Direct `syscall(SYS_*, ...)`
# for these three is a portability failure.
scan "syscall(SYS_OPEN) — aarch64-broken" 'syscall\(\s*SYS_OPEN\b'
scan "syscall(SYS_CHMOD) — aarch64-broken" 'syscall\(\s*SYS_CHMOD\b'
scan "syscall(SYS_STAT) — aarch64-broken"  'syscall\(\s*SYS_STAT\b'

# Category D — static-buffer escape via str_from. `var buf[N]` in
# Cyrius is STATIC DATA, not stack — two calls to the same fn share
# the backing memory. `str_from(&buf)` wraps the buffer in a Str
# fat-pointer that BORROWS the data; if the resulting Str outlives
# the function (returned, stored in a long-lived struct, pushed into
# a vec), every subsequent call to the same fn clobbers every prior
# Str's data. Use `str_clone(str_from(&buf))` to deep-copy.
#
# Bit history.cyr in slice 7 of v1.3.0 (`CommandHistory_add` borrowed
# interactive_loop's `&buf`; every history entry's data aliased to
# whatever was last typed). Audit P(-1) sweep in v1.3.1 slice 3 found
# five dormant copies of the same shape in ui/prompt/session.
scan "str_from(&buf) escape via return" 'return\s+str_from\(&'
scan "str_from(&buf) escape via store"  'store64\([^,]+,\s*str_from\(&'

if [ $FAIL -eq 0 ]; then
    echo "lint-cstr-str: clean (no Str/cstring antipatterns in $SRC_DIRS/)"
    exit 0
fi

echo "lint-cstr-str: FAIL"
echo "$HITS"
echo ""
echo "Each hit is a known bug-class antipattern. Wrap cstring literals"
echo "in str_from() before passing to Str-typed fns; use sys_open /"
echo "sys_chmod / sys_stat wrappers instead of bare syscalls. If a"
echo "specific call site is intentional, add a trailing"
echo "  # lint:cstr-ok"
echo "comment on that line."
exit 1
