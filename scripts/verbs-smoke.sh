#!/bin/sh
# verbs-smoke.sh -- end-to-end smoke test for agnsh's in-process FS verbs
# (Track B, 1.4.2). Exercises ls/cat/cp/mv/rm/mkdir/rmdir/touch/echo/wc/
# find/grep as BUILTINS (not external binaries) against a real temp dir on
# the HOST, asserting correct stdout / side effects. Each verb is driven
# via `agnsh -c "<verb> ..."`, which routes the line through the same
# dispatch_fs_verb hook the interactive loop uses.
#
# Exits non-zero on any failure.

set -u

BIN="${1:-./build/agnsh}"

if [ ! -x "$BIN" ]; then
    echo "Error: $BIN not found or not executable"
    exit 1
fi

PASS=0
FAIL=0
FAILED_TESTS=""

# check NAME EXPECTED ACTUAL  -- ACTUAL must CONTAIN EXPECTED (grep -F)
check() {
    name="$1"; expected="$2"; actual="$3"
    if printf '%s' "$actual" | grep -qF "$expected"; then
        PASS=$((PASS + 1))
    else
        FAIL=$((FAIL + 1))
        FAILED_TESTS="$FAILED_TESTS
  FAIL: $name
    expected to contain: $expected
    got:                 $actual"
    fi
}

# check_eq NAME EXPECTED ACTUAL  -- exact match
check_eq() {
    name="$1"; expected="$2"; actual="$3"
    if [ "$actual" = "$expected" ]; then
        PASS=$((PASS + 1))
    else
        FAIL=$((FAIL + 1))
        FAILED_TESTS="$FAILED_TESTS
  FAIL: $name
    expected exactly: [$expected]
    got:              [$actual]"
    fi
}

# check_absent NAME NEEDLE ACTUAL -- ACTUAL must NOT contain NEEDLE
check_absent() {
    name="$1"; needle="$2"; actual="$3"
    if printf '%s' "$actual" | grep -qF "$needle"; then
        FAIL=$((FAIL + 1))
        FAILED_TESTS="$FAILED_TESTS
  FAIL: $name
    should NOT contain: $needle
    got:                $actual"
    else
        PASS=$((PASS + 1))
    fi
}

echo "=== agnsh FS-verb smoke test ==="
echo "Binary: $BIN"
echo ""

TMP=$(mktemp -d -t agnsh-verbs.XXXXXX)
trap 'rm -rf "$TMP"' EXIT INT TERM

# ---------------------------------------------------------------
# echo  (stdout + redirect)
# ---------------------------------------------------------------
out=$("$BIN" -c "echo hello world" 2>&1)
check_eq "echo stdout" "hello world" "$out"

out=$("$BIN" -c "echo -n no newline" 2>&1)
check_eq "echo -n suppresses newline" "no newline" "$out"

"$BIN" -c "echo file contents here > $TMP/echo1.txt" >/dev/null 2>&1
check_eq "echo > FILE writes file" "file contents here" "$(cat "$TMP/echo1.txt" 2>/dev/null)"

# ---------------------------------------------------------------
# cat
# ---------------------------------------------------------------
out=$("$BIN" -c "cat $TMP/echo1.txt" 2>&1)
check_eq "cat reads file" "file contents here" "$out"

# cat of a missing file errors to stderr, nonzero
out=$("$BIN" -c "cat $TMP/does-not-exist" 2>&1)
check "cat missing file errors" "cat: $TMP/does-not-exist: No such file" "$out"

# ---------------------------------------------------------------
# cp
# ---------------------------------------------------------------
"$BIN" -c "cp $TMP/echo1.txt $TMP/copy1.txt" >/dev/null 2>&1
check_eq "cp duplicates content" "file contents here" "$(cat "$TMP/copy1.txt" 2>/dev/null)"
check_eq "cp left source intact" "file contents here" "$(cat "$TMP/echo1.txt" 2>/dev/null)"

out=$("$BIN" -c "cp $TMP/missing-src $TMP/dst" 2>&1)
check "cp missing source errors" "cp: $TMP/missing-src: No such file" "$out"

# ---------------------------------------------------------------
# mv  (rename within one mount)
# ---------------------------------------------------------------
"$BIN" -c "mv $TMP/copy1.txt $TMP/moved1.txt" >/dev/null 2>&1
check_eq "mv created destination" "file contents here" "$(cat "$TMP/moved1.txt" 2>/dev/null)"
check_eq "mv removed source" "" "$(cat "$TMP/copy1.txt" 2>/dev/null)"

out=$("$BIN" -c "mv $TMP/missing-mv $TMP/dst" 2>&1)
check "mv missing source errors" "mv: $TMP/missing-mv: rename failed" "$out"

# ---------------------------------------------------------------
# touch
# ---------------------------------------------------------------
"$BIN" -c "touch $TMP/touched.txt" >/dev/null 2>&1
check_eq "touch creates empty file" "yes" "$(test -f "$TMP/touched.txt" && echo yes)"
check_eq "touched file is empty" "0" "$(wc -c < "$TMP/touched.txt" 2>/dev/null | tr -d ' ')"

# ---------------------------------------------------------------
# wc
# ---------------------------------------------------------------
printf 'one two three\nfour five\n' > "$TMP/wc.txt"
out=$("$BIN" -c "wc $TMP/wc.txt" 2>&1)
# 2 lines, 5 words, 24 bytes
check "wc line count" "2" "$out"
check "wc word count" "5" "$out"
check "wc byte count" "24" "$out"
check "wc shows filename" "$TMP/wc.txt" "$out"

# ---------------------------------------------------------------
# mkdir / rmdir
# ---------------------------------------------------------------
"$BIN" -c "mkdir $TMP/subdir" >/dev/null 2>&1
check_eq "mkdir creates directory" "yes" "$(test -d "$TMP/subdir" && echo yes)"

"$BIN" -c "rmdir $TMP/subdir" >/dev/null 2>&1
check_eq "rmdir removes directory" "gone" "$(test -d "$TMP/subdir" || echo gone)"

out=$("$BIN" -c "rmdir $TMP/no-such-dir" 2>&1)
check "rmdir missing dir errors" "rmdir: $TMP/no-such-dir: cannot remove directory" "$out"

# ---------------------------------------------------------------
# rm  (incl. dir-operand hint + missing-file error)
# ---------------------------------------------------------------
"$BIN" -c "touch $TMP/to-remove.txt" >/dev/null 2>&1
"$BIN" -c "rm $TMP/to-remove.txt" >/dev/null 2>&1
check_eq "rm deletes file" "gone" "$(test -f "$TMP/to-remove.txt" || echo gone)"

out=$("$BIN" -c "rm $TMP/no-such-file" 2>&1)
check "rm missing file errors" "rm: $TMP/no-such-file: No such file" "$out"

mkdir -p "$TMP/adir"
out=$("$BIN" -c "rm $TMP/adir" 2>&1)
check "rm on directory suggests rmdir" "is a directory (use rmdir)" "$out"
check_eq "rm did not remove the directory" "yes" "$(test -d "$TMP/adir" && echo yes)"

# ---------------------------------------------------------------
# ls  (plain + -l + file operand)
# ---------------------------------------------------------------
mkdir -p "$TMP/lsdir"
printf 'aaa' > "$TMP/lsdir/alpha.txt"
printf 'bbbbb' > "$TMP/lsdir/beta.txt"
mkdir -p "$TMP/lsdir/gamma"

out=$("$BIN" -c "ls $TMP/lsdir" 2>&1)
check "ls lists alpha" "alpha.txt" "$out"
check "ls lists beta" "beta.txt" "$out"
check "ls lists gamma" "gamma" "$out"
# No bare "." or ".." line in the listing (skipped by dir_read_names).
dotcount=$(printf '%s\n' "$out" | grep -cxE '\.|\.\.')
check_eq "ls hides . and .. entries" "0" "$dotcount"

out=$("$BIN" -c "ls -l $TMP/lsdir" 2>&1)
check "ls -l marks file size for alpha" "3  alpha.txt" "$out"
check "ls -l marks file size for beta" "5  beta.txt" "$out"
check "ls -l marks directory gamma" "d " "$out"

# ls of a single file prints just that path
out=$("$BIN" -c "ls $TMP/lsdir/alpha.txt" 2>&1)
check_eq "ls of a file prints the file" "$TMP/lsdir/alpha.txt" "$out"

# ls of a missing path errors
out=$("$BIN" -c "ls $TMP/no-such-dir" 2>&1)
check "ls missing path errors" "ls: $TMP/no-such-dir: No such file or directory" "$out"

# ---------------------------------------------------------------
# find  (recursive walk + -name suffix filter)
# ---------------------------------------------------------------
mkdir -p "$TMP/findtree/sub"
printf 'x' > "$TMP/findtree/top.cyr"
printf 'y' > "$TMP/findtree/sub/deep.cyr"
printf 'z' > "$TMP/findtree/sub/other.txt"

out=$("$BIN" -c "find $TMP/findtree" 2>&1)
check "find prints root" "$TMP/findtree" "$out"
check "find descends to top.cyr" "$TMP/findtree/top.cyr" "$out"
check "find descends into sub" "$TMP/findtree/sub" "$out"
check "find reaches deep.cyr" "$TMP/findtree/sub/deep.cyr" "$out"
check "find reaches other.txt" "$TMP/findtree/sub/other.txt" "$out"

out=$("$BIN" -c "find $TMP/findtree -name *.cyr" 2>&1)
check "find -name matches top.cyr" "$TMP/findtree/top.cyr" "$out"
check "find -name matches deep.cyr" "$TMP/findtree/sub/deep.cyr" "$out"
check_absent "find -name excludes other.txt" "other.txt" "$out"

# ---------------------------------------------------------------
# grep  (single + multi-file with prefix)
# ---------------------------------------------------------------
printf 'apple\nbanana\napricot\ncherry\n' > "$TMP/g1.txt"
out=$("$BIN" -c "grep ap $TMP/g1.txt" 2>&1)
check "grep matches apple" "apple" "$out"
check "grep matches apricot" "apricot" "$out"
check_absent "grep excludes banana" "banana" "$out"
check_absent "grep excludes cherry" "cherry" "$out"

printf 'apple pie\n' > "$TMP/g2.txt"
out=$("$BIN" -c "grep apple $TMP/g1.txt $TMP/g2.txt" 2>&1)
check "grep multi-file prefixes g1" "$TMP/g1.txt:apple" "$out"
check "grep multi-file prefixes g2" "$TMP/g2.txt:apple pie" "$out"

out=$("$BIN" -c "grep zzz $TMP/no-such-grep" 2>&1)
check "grep missing file errors" "grep: $TMP/no-such-grep: No such file" "$out"

# ---------------------------------------------------------------
# Round-trip: echo > , cat, cp, ls, wc, rm, mkdir, rmdir
# (the task's manual sanity sequence, asserted)
# ---------------------------------------------------------------
RT="$TMP/rt"
mkdir -p "$RT"
"$BIN" -c "echo roundtrip > $RT/f" >/dev/null 2>&1
check_eq "round-trip echo>cat" "roundtrip" "$("$BIN" -c "cat $RT/f" 2>&1)"
"$BIN" -c "cp $RT/f $RT/g" >/dev/null 2>&1
check_eq "round-trip cp" "roundtrip" "$("$BIN" -c "cat $RT/g" 2>&1)"
out=$("$BIN" -c "ls $RT" 2>&1)
check "round-trip ls sees f" "f" "$out"
check "round-trip ls sees g" "g" "$out"
out=$("$BIN" -c "wc $RT/f" 2>&1)
check "round-trip wc bytes" "10" "$out"   # "roundtrip\n" = 10 bytes
"$BIN" -c "rm $RT/f" >/dev/null 2>&1
"$BIN" -c "rm $RT/g" >/dev/null 2>&1
check_eq "round-trip rm cleared dir" "" "$(ls -A "$RT" 2>/dev/null)"
"$BIN" -c "mkdir $RT/d" >/dev/null 2>&1
check_eq "round-trip mkdir" "yes" "$(test -d "$RT/d" && echo yes)"
"$BIN" -c "rmdir $RT/d" >/dev/null 2>&1
check_eq "round-trip rmdir" "gone" "$(test -d "$RT/d" || echo gone)"

# ---------------------------------------------------------------
# Safety gate: a dangerous / pipeline / traversal verb line must NOT
# run the verb -- it falls through to the NL/intent classifier (which
# emits "Intent:"), preserving the shell's safety surface.
# ---------------------------------------------------------------
out=$("$BIN" -c "rm -rf $TMP/findtree" 2>&1)
check "safety: rm -rf falls through to intent" "Intent:" "$out"
check_eq "safety: rm -rf did NOT delete the tree" "yes" "$(test -d "$TMP/findtree" && echo yes)"

out=$("$BIN" -c "cat $TMP/../etc/hostname" 2>&1)
check "safety: traversal falls through to intent" "Intent:" "$out"

out=$("$BIN" -c "ls | grep foo" 2>&1)
check "safety: pipeline falls through to intent" "Intent:" "$out"

# ---------------------------------------------------------------
# FIX 1 (data loss): `cp f f` must REFUSE before any open -- the old
# O_TRUNC-before-read zeroed an identical src/dst. The file must survive
# byte-for-byte and the verb must error with "same file".
# ---------------------------------------------------------------
printf 'do not truncate me\n' > "$TMP/selfcp.txt"
out=$("$BIN" -c "cp $TMP/selfcp.txt $TMP/selfcp.txt" 2>&1)
check "FIX1: cp f f errors as same file" "are the same file" "$out"
check_eq "FIX1: cp f f left file intact (NOT truncated)" "do not truncate me" "$(cat "$TMP/selfcp.txt" 2>/dev/null)"

# ---------------------------------------------------------------
# FIX 2 (inverted safety): `rm -i FILE` must NOT delete FILE -- the old
# code treated -i as a literal operand-that-errors while still unlinking
# the real operand. It must now reject -i as an unknown option and unlink
# nothing. Same for -v.
# ---------------------------------------------------------------
printf 'keep\n' > "$TMP/keep.txt"
out=$("$BIN" -c "rm -i $TMP/keep.txt" 2>&1)
check "FIX2: rm -i reports unrecognized option" "rm: unrecognized option '-i'" "$out"
check_eq "FIX2: rm -i did NOT delete keep.txt" "yes" "$(test -f "$TMP/keep.txt" && echo yes)"
out=$("$BIN" -c "rm -v $TMP/keep.txt" 2>&1)
check "FIX2: rm -v reports unrecognized option" "rm: unrecognized option '-v'" "$out"
check_eq "FIX2: rm -v did NOT delete keep.txt" "yes" "$(test -f "$TMP/keep.txt" && echo yes)"

# ---------------------------------------------------------------
# FIX 3 (leading-dash operands): `mkdir -p d` must NOT create a directory
# literally named "-p" (nor "d") -- it must reject -p as an unknown
# option. Same class for touch/wc/rmdir/cp/mv.
# ---------------------------------------------------------------
out=$("$BIN" -c "mkdir -p $TMP/dpdir" 2>&1)
check "FIX3: mkdir -p reports unrecognized option" "mkdir: unrecognized option '-p'" "$out"
check_eq "FIX3: mkdir -p did NOT create a literal '-p'" "no" "$(test -e "$TMP/-p" && echo yes || echo no)"
check_eq "FIX3: mkdir -p did NOT create the dir operand" "no" "$(test -d "$TMP/dpdir" && echo yes || echo no)"
out=$("$BIN" -c "touch -x $TMP/tch" 2>&1)
check "FIX3: touch -x reports unrecognized option" "touch: unrecognized option '-x'" "$out"
check_eq "FIX3: touch -x created nothing" "no" "$(test -e "$TMP/tch" && echo yes || echo no)"

# ---------------------------------------------------------------
# FIX 4 (mode-gated confirm): destructive verbs prompt in HUMAN/STRICT
# and run directly in AUTO/ASSIST. Driven via the -c `--mode NAME` prefix
# with y/n piped to stdin.
#   (a) strict + 'n' piped -> NOT deleted + prints "(aborted)"
#   (b) strict + 'y' piped -> deleted
#   (c) assist -> deleted with NO prompt
# ---------------------------------------------------------------
printf 'gate\n' > "$TMP/gate_n.txt"
out=$(printf 'n\n' | "$BIN" --mode strict -c "rm $TMP/gate_n.txt" 2>&1)
check "FIX4: strict rm + n prints aborted" "(aborted)" "$out"
check "FIX4: strict rm + n shows confirm prompt" "[y/N]" "$out"
check_eq "FIX4: strict rm + n did NOT delete" "yes" "$(test -f "$TMP/gate_n.txt" && echo yes)"

printf 'gate\n' > "$TMP/gate_y.txt"
out=$(printf 'y\n' | "$BIN" --mode strict -c "rm $TMP/gate_y.txt" 2>&1)
check "FIX4: strict rm + y shows confirm prompt" "[y/N]" "$out"
check_eq "FIX4: strict rm + y DID delete" "gone" "$(test -f "$TMP/gate_y.txt" || echo gone)"

printf 'gate\n' > "$TMP/gate_assist.txt"
out=$("$BIN" --mode assist -c "rm $TMP/gate_assist.txt" 2>&1 </dev/null)
check_absent "FIX4: assist rm shows NO prompt" "[y/N]" "$out"
check_eq "FIX4: assist rm deleted with no prompt" "gone" "$(test -f "$TMP/gate_assist.txt" || echo gone)"

# human mode behaves like strict (prompts); auto behaves like assist (direct)
printf 'gate\n' > "$TMP/gate_human.txt"
out=$(printf 'n\n' | "$BIN" --mode human -c "rm $TMP/gate_human.txt" 2>&1)
check "FIX4: human rm + n prints aborted" "(aborted)" "$out"
check_eq "FIX4: human rm + n did NOT delete" "yes" "$(test -f "$TMP/gate_human.txt" && echo yes)"

# cp overwrite is destructive in strict; cp to a NEW dst is not.
printf 'SRC\n' > "$TMP/ov_src.txt"; printf 'OLD\n' > "$TMP/ov_dst.txt"
out=$(printf 'n\n' | "$BIN" --mode strict -c "cp $TMP/ov_src.txt $TMP/ov_dst.txt" 2>&1)
check "FIX4: strict cp-overwrite + n prints aborted" "(aborted)" "$out"
check_eq "FIX4: strict cp-overwrite + n left dst intact" "OLD" "$(cat "$TMP/ov_dst.txt" 2>/dev/null)"
out=$(printf 'n\n' | "$BIN" --mode strict -c "cp $TMP/ov_src.txt $TMP/ov_new.txt" 2>&1)
check_absent "FIX4: strict cp to NEW dst shows NO prompt" "[y/N]" "$out"
check_eq "FIX4: strict cp to NEW dst created it" "SRC" "$(cat "$TMP/ov_new.txt" 2>/dev/null)"

# ---------------------------------------------------------------
echo ""
echo "Passed: $PASS"
echo "Failed: $FAIL"

if [ "$FAIL" -gt 0 ]; then
    echo "$FAILED_TESTS"
    exit 1
fi

echo "All FS-verb smoke tests passed."
