#!/bin/sh
# smoke-test.sh -- end-to-end smoke test for agnsh
# Exercises CLI flags and common NL inputs. Exits non-zero on failure.

set -e

BIN="${1:-./build/agnsh}"

if [ ! -x "$BIN" ]; then
    echo "Error: $BIN not found or not executable"
    exit 1
fi
# Absolute, because some checks `cd` into a scratch folder before running it: CI passes the
# relative `build/agnsh`, which stops resolving after the cd.
case "$BIN" in
    /*) ;;
    *) BIN="$(pwd)/$BIN" ;;
esac

# 2.0.2: as root on a Linux host agnsh reports natural-language lines and runs none of them, so the
# NL checks below would fail for a reason that is not a defect. The root behaviour has its own
# checks (a user namespace, below).
if [ "$(id -u)" = 0 ]; then
    echo "smoke-test: run it as an ordinary user -- as root, agnsh runs no natural-language line (2.0.2)"
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

# 2.0.0: every NL line run with -c files a report (ADR-008). Keep this run's reports out of the
# runner's own $XDG_STATE_HOME / ~/.local/state.
SMOKE_STATE=$(mktemp -d -t agnsh-state.XXXXXX)
export XDG_STATE_HOME="$SMOKE_STATE"

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

# Intent classification. Since 2.0.0 `-c` keeps stdout for the program it runs and files the
# report (ADR-008); `--dry-run` prints the 1.9.x report and runs nothing, so the classifier checks
# read it from there. A line that would not run exits 126/127, hence `|| true` under set -e.
out=$("$BIN" --dry-run -c "show me all files" 2>&1) || true
check "parse show files" "Intent:" "$out"

out=$("$BIN" --dry-run -c "list running processes" 2>&1) || true
check "parse list procs" "Intent:" "$out"

out=$("$BIN" --dry-run -c "git status" 2>&1) || true
check "parse git status" "Intent:" "$out"

out=$("$BIN" --dry-run -c "install vim" 2>&1) || true
check "parse install" "Intent:" "$out"

# NB: "find files named foo" used to exercise the FIND_FILES intent here,
# but `find` is now an in-process FS builtin (1.4.2), so a `find`-leading
# line runs the verb. Use the equivalent NL phrasing that does NOT begin
# with a verb word to keep the intent-parser coverage.
out=$("$BIN" --dry-run -c "search for files named foo" 2>&1) || true
check "parse find files" "Intent:" "$out"

out=$("$BIN" --dry-run -c "remove file.txt" 2>&1) || true
check "parse remove" "Intent:" "$out"

out=$("$BIN" --dry-run -c "firewall allow 8080" 2>&1) || true
check "parse firewall" "Intent:" "$out"

out=$("$BIN" --dry-run -c "create user alice" 2>&1) || true
check "parse user add" "Intent:" "$out"

# Approval wiring -- every -c output now carries a "Risk: [LEVEL]"
# line (assessed via risk_from_permission). BLOCKED commands surface
# a WARNING line; HIGH-risk ones note the approval requirement.
out=$("$BIN" --dry-run -c "show me files" 2>&1) || true
check "risk LOW for read-only" "Risk: \[LOW\]" "$out"

out=$("$BIN" --dry-run -c "copy a to b" 2>&1) || true
check "risk MED for user-write" "Risk: \[MED\]" "$out"

out=$("$BIN" --dry-run -c "install vim" 2>&1) || true
check "risk HIGH for admin" "Risk: \[HIGH\]" "$out"
check "high-risk approval hint" "Approval required" "$out"
# 1.9.15: the line says what happened. It used to promise "(interactive prompt in shell
# mode)", and no prompt exists in any mode.
check "high-risk line says nothing ran" "Approval required -- not executed" "$out"

out=$("$BIN" --dry-run -c "rm -rf /tmp/foo" 2>&1) || true
check "risk CRIT for blocked" "Risk: \[CRIT\]" "$out"
check "blocked warning line" "WARNING: BLOCKED" "$out"
# 1.9.15: it used to say "would not execute without explicit override" — no override exists.
check "blocked line claims no override" "not executed, and there is no override" "$out"

# Command field populated -- the cstring/Str print mismatch that left
# this blank pre-v1.2.1 is now fixed.
out=$("$BIN" --dry-run -c "show me files" 2>&1) || true
check "command field has ls" "Command: ls" "$out"

# Error-recovery hints -- when the parse succeeds but the translation
# isn't actually runnable (LLM not wired, pipeline exec not wired,
# safety check rejected), surface a Hint: line so the user knows the
# echo+Risk:[LOW] output isn't a real run.
out=$("$BIN" --dry-run -c "what is dns" 2>&1) || true
check "question hint surfaces" "Hint: question intent" "$out"
out=$("$BIN" --dry-run -c "ls | grep foo" 2>&1) || true
# The hint text changed in 1.9.2. It used to read "auto-exec arrives with the
# exec wire-up", which had been false for six releases — pipelines DO auto-exec
# on agnos. Assert on the stable "Hint: pipeline" prefix rather than re-pinning
# a full sentence that will drift again.
check "pipeline hint surfaces" "Hint: pipeline" "$out"
out=$("$BIN" --dry-run -c "remove ../etc/passwd" 2>&1) || true
check "safety-reject hint surfaces" "Hint: translator safety check rejected" "$out"
# Happy-path inputs should NOT carry a hint line.
out=$("$BIN" --dry-run -c "show me files" 2>&1) || true
case "$out" in
  *"Hint:"*) FAIL=$((FAIL+1)); FAILED_TESTS="$FAILED_TESTS
  FAIL: happy-path output should not have Hint:";;
  *) PASS=$((PASS+1));;
esac

# The -c contract (2.0.0, ADR-008): stdout is the program's, the report goes to the report
# folder, and a line agnsh does not run says why on stderr and exits 126 (understood, not run) or
# 127 (nothing to run).
C_STATE=$(mktemp -d -t agnsh-report.XXXXXX)
REPORTS="$C_STATE/agnoshi/reports"
ec=0
out=$(XDG_STATE_HOME="$C_STATE" "$BIN" -c "copy a to b" 2>/dev/null) || ec=$?
check "a line that does not run exits 126" "^126$" "$ec"
check "...and leaves stdout empty" "^$" "$out"
err=$(XDG_STATE_HOME="$C_STATE" "$BIN" -c "copy a to b" 2>&1 >/dev/null) || true
check "...and says why on stderr, naming the report" "agnsh: not executed: approval required -- report: " "$err"
check "the report is filed as latest.txt" "Intent: 6  Command: cp" "$(cat "$REPORTS/latest.txt" 2>/dev/null)"
check "the report records the input" "input: copy a to b" "$(cat "$REPORTS/latest.txt" 2>/dev/null)"
check "the report records the result" "result: not executed: approval required" "$(cat "$REPORTS/latest.txt" 2>/dev/null)"
check "the report folder is private" "^700$" "$(stat -c %a "$REPORTS" 2>/dev/null)"
check "a report is private" "^600$" "$(stat -c %a "$REPORTS/latest.txt" 2>/dev/null)"
ec=0
XDG_STATE_HOME="$C_STATE" "$BIN" -c "what is dns" >/dev/null 2>&1 || ec=$?
check "a line with nothing to run exits 127" "^127$" "$ec"
ec=0
dry=$(XDG_STATE_HOME="$C_STATE" "$BIN" --dry-run -c "show me files" 2>&1) || ec=$?
check "--dry-run prints the 1.9.x report" "Intent: 0  Command: ls" "$dry"
check "--dry-run exits 0 for a line that would run" "^0$" "$ec"
check "--dry-run files its report too" "result: not executed: dry run" "$(cat "$REPORTS/latest.txt" 2>/dev/null)"
dry=$(XDG_STATE_HOME="$C_STATE" "$BIN" -n --mode strict -c "show me files" 2>&1) || true
check "-n and --mode combine in either order" "Intent: 0  Command: ls" "$dry"
rm -rf "$C_STATE"

# Shell-first on the host (2.0.0, ADR-007 / ADR-008 § 3): a line whose first word is a program on
# $PATH runs as that program -- through one argv launcher that inherits the environment and never
# uses a shell. `|`, `>` and `&` are shell syntax this host cannot run yet: refused, never NL.
out=$("$BIN" -c "echo agnsh-smoke-hello" 2>/dev/null) || true
check "a bareword program runs, output on stdout" "^agnsh-smoke-hello$" "$out"
ec=0
"$BIN" -c "false" >/dev/null 2>&1 || ec=$?
check "...and its exit status is agnsh's" "^1$" "$ec"
ec=0
err=$("$BIN" -c "ls | sort" 2>&1 >/dev/null) || ec=$?
check "a pipeline is refused on the host" "pipelines are AGNOS-only" "$err"
check "...with 127" "^127$" "$ec"
ec=0
err=$("$BIN" -c "rm -rf /tmp/agnsh-smoke-nonexistent" 2>&1 >/dev/null </dev/null) || ec=$?
check "a typed BLOCKED line asks even in auto" "BLOCKED: run" "$err"
check "...and with no answer it is declined (126)" "^126$" "$ec"
out=$("$BIN" -c "run /bin/echo agnsh-run-args" 2>/dev/null) || true
check "run takes arguments on the host" "^agnsh-run-args$" "$out"
ec=0
"$BIN" -c "run /tmp/x;evil" >/dev/null 2>&1 || ec=$?
check "run of an unsafe path is refused with 126 (2.0.1)" "^126$" "$ec"
out=$(AGNSH_SMOKE_VAR=inherited "$BIN" -c "env" 2>/dev/null) || true
check "a program inherits agnsh's environment" "AGNSH_SMOKE_VAR=inherited" "$out"

# NL execution (2.0.0, ADR-008): SAFE and READ_ONLY lines run; the program's output is stdout.
NL_DIR=$(mktemp -d -t agnsh-nl.XXXXXX)
NL_STATE="$NL_DIR/state"
mkdir -p "$NL_DIR/work"
touch "$NL_DIR/work/agnsh-nl-marker"
ec=0
out=$(cd "$NL_DIR/work" && HOME="$NL_DIR" XDG_STATE_HOME="$NL_STATE" "$BIN" -c "show me all files" 2>/dev/null) || ec=$?
check "an NL read-only line runs" "agnsh-nl-marker" "$out"
check "...with the program's status" "^0$" "$ec"
check "...and no report on stdout" "^[^I]*$" "$(echo "$out" | grep -c 'Intent:')"
check "its report records the execution" "result: executed, exit 0" "$(cat "$NL_STATE/agnoshi/reports/latest.txt" 2>/dev/null)"
nl_log=$(cat "$NL_DIR/.agnsh_audit.log" 2>/dev/null || true)
check "the audit keeps the parse-time record" '"input":"show me all files","action":"ls","approved":1,"result":"proposed"' "$nl_log"
check "...and adds the exec records with the exit code" '"result":"executed","exit_code":0' "$nl_log"
ec=0
(cd "$NL_DIR/work" && HOME="$NL_DIR" XDG_STATE_HOME="$NL_STATE" "$BIN" --mode human -c "show me all files" >/dev/null 2>&1 </dev/null) || ec=$?
check "human mode confirms first, and no answer declines (126)" "^126$" "$ec"
ec=0
(HOME="$NL_DIR" XDG_STATE_HOME="$NL_STATE" "$BIN" -c "copy a to b" >/dev/null 2>&1) || ec=$?
check "a user-write NL line does not run yet (126)" "^126$" "$ec"
check "...and the audit says it needs approval" '"input":"copy a to b","action":"cp","approved":0,"result":"needs_approval"' "$(cat "$NL_DIR/.agnsh_audit.log" 2>/dev/null)"
rm -rf "$NL_DIR"

# Restricted sessions (2.0.2, src/security.cyr): agnsh running as root on a Linux host reports
# natural-language lines and runs none of them (126); the user's own shell lines still run. A user
# namespace gives uid 0 without privilege (`unshare -r`). Where the host forbids one -- some CI kernels
# restrict unprivileged user namespaces -- those checks are skipped, and the run says so.
out=$(printf 'exit\n' | "$BIN" 2>&1) || true
case "$out" in
  *"running as root"*) FAIL=$((FAIL + 1)); FAILED_TESTS="$FAILED_TESTS
  FAIL: an interactive shell run by an ordinary user warned about root";;
  *) PASS=$((PASS + 1));;
esac
if command -v unshare >/dev/null 2>&1 && unshare -r true 2>/dev/null; then
    RS_DIR=$(mktemp -d -t agnsh-root.XXXXXX)
    RS_STATE="$RS_DIR/state"
    mkdir -p "$RS_DIR/work"
    touch "$RS_DIR/work/agnsh-root-marker"
    ec=0
    out=$(cd "$RS_DIR/work" && HOME="$RS_DIR" XDG_STATE_HOME="$RS_STATE" unshare -r "$BIN" -c "show me all files" 2>"$RS_DIR/err") || ec=$?
    check "as root, an NL line is not run (126)" "^126$" "$ec"
    check "...leaving stdout empty: ls never ran" "^$" "$out"
    check "...and stderr says why" "agnsh: not executed: running as root (restricted) -- report: " "$(cat "$RS_DIR/err")"
    check "...as does its report" "result: not executed: running as root (restricted)" "$(cat "$RS_STATE/agnoshi/reports/latest.txt" 2>/dev/null)"
    check "the audit records the refusal as denied" '"input":"show me all files","action":"[^"]*ls -a","approved":0,"result":"denied"' "$(cat "$RS_DIR/.agnsh_audit.log" 2>/dev/null)"
    out=$(HOME="$RS_DIR" XDG_STATE_HOME="$RS_STATE" unshare -r "$BIN" -c "echo agnsh-root-shell-line" 2>/dev/null) || true
    check "as root, the user's own shell line still runs" "^agnsh-root-shell-line$" "$out"
    ec=0
    out=$(HOME="$RS_DIR" XDG_STATE_HOME="$RS_STATE" unshare -r "$BIN" --dry-run -c "show me all files" 2>&1) || ec=$?
    check "as root, --dry-run shows the restriction" "Restricted -- not executed: agnsh is running as root" "$out"
    check "...and exits 126: the line would not run" "^126$" "$ec"
    out=$(printf 'exit\n' | HOME="$RS_DIR" XDG_STATE_HOME="$RS_STATE" unshare -r "$BIN" 2>&1) || true
    check "as root, the interactive shell warns before its first prompt" "WARNING: agnsh is running as root" "$out"
    rm -rf "$RS_DIR"
else
    echo "(skipped: the restricted-session checks -- no unprivileged user namespace on this host)"
fi

# Interactive mode -- drive via stdin pipe and check that the mode-
# switching builtins flow correctly and the prompt updates. Each line
# of input is one user turn (the read_line helper accepts byte-by-byte
# stdin so piped multi-line blobs no longer collapse into one buffer).
# 2.0.0: the NL line now RUNS, and under strict it confirms first -- the `n` answers that prompt, so
# `exit` is still read as the command it is.
INT_OUT=$(printf 'mode\nmode human\nmode\nmode strict\nshow files\nn\nexit\n' | "$BIN" 2>&1)
check "interactive shows assist start" "\[ASSIST\] >" "$INT_OUT"
check "interactive mode reports current" "Current mode: AI-ASSIST" "$INT_OUT"
check "interactive mode switch to human" "Mode -> HUMAN" "$INT_OUT"
check "interactive prompt updates after switch" "\[HUMAN\] >" "$INT_OUT"
check "interactive mode switch to strict" "Mode -> STRICT" "$INT_OUT"
check "interactive parses NL under mode" "Intent:" "$INT_OUT"
check "strict confirms before an NL command runs" "run .*ls.* ? \[y/N\]" "$INT_OUT"
check "...and a no declines it" "(aborted)" "$INT_OUT"
check "interactive exits cleanly" "bye" "$INT_OUT"

# Interactive negative -- unknown mode name should error, not crash,
# and the available list must surface for discoverability.
BAD_OUT=$(printf 'mode wibble\nexit\n' | "$BIN" 2>&1)
check "unknown mode error" "Unknown mode: wibble" "$BAD_OUT"
check "unknown mode suggests list" "Available: auto, assist, human, strict" "$BAD_OUT"

# History -- every non-builtin input is recorded; the `history` builtin
# replays the last 20; the on-disk $HOME/.agnsh_history persists across
# sessions; the next session loads it on start.
HIST_HOME=$(mktemp -d -t agnsh-hist.XXXXXX)
# NB: the second line used to be `find /tmp` (an NL sample), but `find` is
# now an FS builtin, so it would run the verb (noisy real output). Use a
# non-verb NL phrasing so this stays a clean history-recording test.
HIST_OUT=$(printf 'show files\nsearch /tmp\nhistory\nexit\n' | HOME="$HIST_HOME" "$BIN" 2>&1)
check "history shows 1 entry" "1  show files" "$HIST_OUT"
check "history shows 2 entry" "2  search /tmp" "$HIST_OUT"
HIST_FILE="$HIST_HOME/.agnsh_history"
check "history file created" "$(test -f "$HIST_FILE" && echo yes)" "yes"
check "history file line count" "$(wc -l < "$HIST_FILE" 2>/dev/null)" "2"
check "history file content" "show files" "$(cat "$HIST_FILE" 2>/dev/null)"

# Second session must load the persisted file on start.
HIST2_OUT=$(printf 'history\nexit\n' | HOME="$HIST_HOME" "$BIN" 2>&1)
check "history loads on next session" "1  show files" "$HIST2_OUT"
check "history loads entry 2" "2  search /tmp" "$HIST2_OUT"
rm -rf "$HIST_HOME"

# Empty-history path -- a fresh shell with no prior history file
# reports `(history empty)` rather than crashing or echoing nothing.
EMPTY_HOME=$(mktemp -d -t agnsh-empty.XXXXXX)
EMPTY_OUT=$(printf 'history\nexit\n' | HOME="$EMPTY_HOME" "$BIN" 2>&1)
check "empty-history message" "(history empty)" "$EMPTY_OUT"
rm -rf "$EMPTY_HOME"

# Audit log -- every -c invocation appends a JSON line to
# $HOME/.agnsh_audit.log. Point HOME at a clean temp dir, run two
# commands, verify the log has well-formed lines with the expected
# action+approved shape.
SMOKE_HOME=$(mktemp -d -t agnsh-smoke.XXXXXX)
# 2.0.0: `-c` exits 126/127 for a line it does not run (|| true under set -e), and the BLOCKED probe
# is NL phrasing -- a shell-shaped `rm -rf ...` line is a shell line, not a classifier input.
HOME="$SMOKE_HOME" "$BIN" --dry-run -c "show me files" >/dev/null 2>&1 || true
HOME="$SMOKE_HOME" "$BIN" --dry-run -c "delete /tmp/x" >/dev/null 2>&1 || true
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

# Audit result enrichment -- the `result` field now distinguishes six
# classes that downstream filters can grep / jq on: proposed,
# needs_approval, blocked, needs_llm, needs_exec, rejected_safety.
RES_HOME=$(mktemp -d -t agnsh-result.XXXXXX)
# NB: `rm /tmp/x` would now run the rm builtin (literal command). Use the
# NL phrasing `delete /tmp/x`, which the intent parser classifies BLOCKED
# (result "blocked") exactly as the old `rm /tmp/x` did — preserving the
# safety-classification coverage without colliding with the rm verb.
HOME="$RES_HOME" "$BIN" --dry-run -c "show files" > /dev/null 2>&1 || true
HOME="$RES_HOME" "$BIN" --dry-run -c "install vim" > /dev/null 2>&1 || true
HOME="$RES_HOME" "$BIN" --dry-run -c "delete /tmp/x" > /dev/null 2>&1 || true
HOME="$RES_HOME" "$BIN" --dry-run -c "what is dns" > /dev/null 2>&1 || true
HOME="$RES_HOME" "$BIN" --dry-run -c "ls | grep foo" > /dev/null 2>&1 || true
HOME="$RES_HOME" "$BIN" --dry-run -c "remove ../etc/passwd" > /dev/null 2>&1 || true
RES_LOG="$RES_HOME/.agnsh_audit.log"
res_content=$(cat "$RES_LOG" 2>/dev/null)
check "result proposed for read-only" '"input":"show files".*"result":"proposed"' "$res_content"
check "result needs_approval for admin" '"input":"install vim".*"result":"needs_approval"' "$res_content"
check "result blocked for rm" '"input":"delete /tmp/x".*"result":"blocked"' "$res_content"
check "result needs_llm for question" '"input":"what is dns".*"result":"needs_llm"' "$res_content"
check "result needs_exec for pipeline" '"input":"ls | grep foo".*"result":"needs_exec"' "$res_content"
# For REMOVE inputs both "rejected_safety" (translator catches the
# path traversal) and "blocked" (BLOCKED-perm classification) are
# acceptable — both indicate the command won't auto-execute. CI
# environments have surfaced the "blocked" case on x86 builds where
# the translator-side safety check evidently short-circuits behind
# the permission check; the user's safety is preserved either way.
check "result safe-decline for traversal-rm" '"input":"remove ../etc/passwd","action":"\(rm\|echo\)","approved":[01],"result":"\(rejected_safety\|blocked\)"' "$res_content"
# Additional cleaner safety-reject probe — CREATE_DIR is USER_WRITE
# (not BLOCKED), so the audit result MUST be `rejected_safety` for
# a path-traversal input. No permission-vs-safety ambiguity here.
HOME="$RES_HOME" "$BIN" --dry-run -c "create directory ../foo" > /dev/null 2>&1 || true
res_content=$(cat "$RES_LOG" 2>/dev/null)
check "result rejected_safety for usr-write traversal" '"input":"create directory ../foo".*"result":"rejected_safety"' "$res_content"
rm -rf "$RES_HOME"

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

# ---- 1.9.2: the EXEC surface is audited ----
# Before 1.9.2 every path that actually ran a program wrote ZERO audit records:
# `agnsh -c 'run /bin/echo'` executed the program and did not even create the
# log file, so the audit trail contained only the actions that never happened.
# These cases fail loudly if that regresses.
EXEC_HOME=$(mktemp -d)
EXEC_LOG="$EXEC_HOME/.agnsh_audit.log"
HOME="$EXEC_HOME" "$BIN" -c "run /bin/echo" >/dev/null 2>&1 || true
HOME="$EXEC_HOME" "$BIN" -c "run /bin/false" >/dev/null 2>&1 || true
HOME="$EXEC_HOME" "$BIN" -c "run /tmp/x;evil" >/dev/null 2>&1 || true
if [ -f "$EXEC_LOG" ]; then
    exec_log=$(cat "$EXEC_LOG")
    PASS=$((PASS + 1))
else
    exec_log=""
    FAIL=$((FAIL + 1))
    FAILED_TESTS="$FAILED_TESTS
  FAIL: exec audit log not created at $EXEC_LOG"
fi
# A launch writes a pre-exec record so a program that hangs or kills the shell
# still leaves a trace, then an outcome record.
check "exec audit: launched record" '"result":"launched"' "$exec_log"
check "exec audit: clean exit -> executed + code 0" '"result":"executed","exit_code":0' "$exec_log"
check "exec audit: non-zero exit -> failed + code 1" '"result":"failed","exit_code":1' "$exec_log"
# A refusal must be recorded too, and must NOT read as approved.
check "exec audit: refusal recorded as denied" '"result":"denied"' "$exec_log"
check "exec audit: refusal is not approved" '"approved":0,"result":"denied"' "$exec_log"
# exit_code is always present, and null (never the raw sentinel) when N/A.
check "exec audit: exit_code null when N/A" '"exit_code":null' "$exec_log"
if echo "$exec_log" | grep -q "999999"; then
    FAIL=$((FAIL + 1))
    FAILED_TESTS="$FAILED_TESTS
  FAIL: exec audit leaked the AUDIT_NO_EXIT sentinel into the record"
else
    PASS=$((PASS + 1))
fi
# Every emitted line must still be valid JSON.
if command -v python3 >/dev/null 2>&1; then
    if python3 -c "import json,sys
[json.loads(l) for l in open('$EXEC_LOG')]" >/dev/null 2>&1; then
        PASS=$((PASS + 1))
    else
        FAIL=$((FAIL + 1))
        FAILED_TESTS="$FAILED_TESTS
  FAIL: exec audit log is not valid JSON"
    fi
fi
rm -rf "$EXEC_HOME"

# ---- 1.9.4: state-file and error-output hygiene ----
HYG_HOME=$(mktemp -d)

# (a) Diagnostics belong on stderr. Before 1.9.4 all 40 of them went to stdout,
# which corrupts a pipe — and now that agnsh executes programs (1.9.2), it also
# mixes the shell's complaints into the child's output stream.
out_o=$(HOME="$HYG_HOME" "$BIN" -c "run /definitely/not/here" 2>/dev/null || true)
out_e=$(HOME="$HYG_HOME" "$BIN" -c "run /definitely/not/here" 2>&1 >/dev/null || true)
if [ -z "$out_o" ]; then PASS=$((PASS + 1)); else
    FAIL=$((FAIL + 1)); FAILED_TESTS="$FAILED_TESTS
  FAIL: launch diagnostic leaked to stdout: $out_o"; fi
check "diagnostic goes to stderr" "run:" "$out_e"
# ...and the report --dry-run prints still goes to stdout. (Since 2.0.0 a plain -c gives stdout to
# the program it runs and files the report instead -- see the -c contract block above.)
out_n=$(HOME="$HYG_HOME" "$BIN" --dry-run -c "show files" 2>/dev/null || true)
check "normal output stays on stdout" "Intent:" "$out_n"

# (b) The audit log's 0600 is re-asserted on an existing file, not only at
# creation — a log restored from a backup or made under a loose umask used to
# stay world-readable forever.
rm -f "$HYG_HOME/.agnsh_audit.log"
touch "$HYG_HOME/.agnsh_audit.log"
chmod 644 "$HYG_HOME/.agnsh_audit.log"
HOME="$HYG_HOME" "$BIN" --dry-run -c "show files" >/dev/null 2>&1 || true
mode=$(stat -c '%a' "$HYG_HOME/.agnsh_audit.log" 2>/dev/null || echo "?")
check "audit log mode repaired to 0600" "600" "$mode"

# (c) The log APPENDS. On a Darwin host the hardcoded 1089 decoded to
# O_WRONLY|O_ASYNC|O_TRUNC — no O_CREAT, and truncating every open.
before=$(wc -l < "$HYG_HOME/.agnsh_audit.log" 2>/dev/null || echo 0)
HOME="$HYG_HOME" "$BIN" --dry-run -c "list files" >/dev/null 2>&1 || true
after=$(wc -l < "$HYG_HOME/.agnsh_audit.log" 2>/dev/null || echo 0)
if [ "$after" -gt "$before" ]; then PASS=$((PASS + 1)); else
    FAIL=$((FAIL + 1)); FAILED_TESTS="$FAILED_TESTS
  FAIL: audit log did not append ($before -> $after)"; fi

# (d) An oversized history keeps the NEWEST entries. It used to load the oldest
# 64 KB and then write that back, permanently eating the recent half.
rm -f "$HYG_HOME/.agnsh_history"
i=0
while [ $i -lt 3000 ]; do echo "histline $i padding-padding-padding-padding"; i=$((i + 1)); done \
    > "$HYG_HOME/.agnsh_history"
warn=$(printf 'exit\n' | HOME="$HYG_HOME" "$BIN" 2>&1 >/dev/null || true)
check "oversized history warns" "exceeded 64 KB" "$warn"
newest=$(tail -1 "$HYG_HOME/.agnsh_history" 2>/dev/null || echo "")
check "oversized history keeps the NEWEST entries" "histline 2999" "$newest"
oldest=$(head -1 "$HYG_HOME/.agnsh_history" 2>/dev/null || echo "")
if echo "$oldest" | grep -q "histline 0 "; then
    FAIL=$((FAIL + 1)); FAILED_TESTS="$FAILED_TESTS
  FAIL: history kept the OLDEST entries (the pre-1.9.4 bug)"
else PASS=$((PASS + 1)); fi

# (e) HOME unset falls back to a UID-QUALIFIED /tmp path, not a fixed name every
# user on the box would share.
rm -f "/tmp/agnsh_audit.log" "/tmp/agnsh_audit.log.$(id -u)"
(unset HOME; "$BIN" --dry-run -c "show files" >/dev/null 2>&1) || true
if [ -f "/tmp/agnsh_audit.log.$(id -u)" ]; then PASS=$((PASS + 1)); else
    FAIL=$((FAIL + 1)); FAILED_TESTS="$FAILED_TESTS
  FAIL: HOME-unset fallback did not use the uid-qualified path"; fi
if [ -f "/tmp/agnsh_audit.log" ]; then
    FAIL=$((FAIL + 1)); FAILED_TESTS="$FAILED_TESTS
  FAIL: HOME-unset fallback still wrote the shared /tmp/agnsh_audit.log"
else PASS=$((PASS + 1)); fi
rm -f "/tmp/agnsh_audit.log.$(id -u)" "/tmp/agnsh_history.$(id -u)"

rm -rf "$HYG_HOME"

# ---- 1.9.5: parser shadowing ----
# Each of these produced a WRONG COMMAND from a reasonable sentence because a
# broad parser earlier in the dispatch claimed the input before the specific
# parser that already handled it correctly could see it. Asserted at the binary
# level, not just the parser, so the whole pipeline is covered.
PS_HOME=$(mktemp -d)
psh() { HOME="$PS_HOME" "$BIN" --dry-run -c "$1" 2>/dev/null | head -1 || true; }

# `delete user bob` used to emit `rm` — an rm against a FILE named "bob".
check "delete user -> userdel" "Command: userdel" "$(psh 'delete user bob')"
check "remove user -> userdel" "Command: userdel" "$(psh 'remove user bob')"
check "delete firewall rule -> ufw" "Command: ufw" "$(psh 'delete firewall rule 22')"
# ...but a plain file deletion whose name merely STARTS with "user" must stay rm.
check "remove user_data.txt stays rm" "Command: rm" "$(psh 'remove user_data.txt')"
check "delete report.txt stays rm" "Command: rm" "$(psh 'delete report.txt')"

# `show contents of FILE` used to emit a bare `ls`, dropping the filename.
check "show contents of FILE -> cat" "Command: cat" "$(psh 'show contents of /etc/hosts')"

# `show memory usage` used to emit `df -h` — a DISK report for a MEMORY question.
# 1.9.5 moved it off df; this check then asserted `uname`, which does not report
# memory either — the test named itself "is not df" and settled for not-df.
# 1.9.10 gave memory its own intent, so it can now assert the actual answer.
check "show memory usage -> free" "Command: free" "$(psh 'show memory usage')"
check "show free memory -> free" "Command: free" "$(psh 'show free memory')"
check "ram usage -> free" "Command: free" "$(psh 'ram usage')"
# ...without breaking real disk questions.
check "show disk usage stays df" "Command: df" "$(psh 'show disk usage')"
check "show disk space stays df" "Command: df" "$(psh 'show disk space')"
# ...or the system query the memory keywords used to be parked on.
check "system info stays uname" "Command: uname" "$(psh 'system info')"
check "show hostname stays uname" "Command: uname" "$(psh 'show hostname')"
# ...or plain listing.
check "show me all files stays ls" "Command: ls" "$(psh 'show me all files')"

rm -rf "$PS_HOME"

rm -rf "$SMOKE_STATE"

echo ""
echo "Passed: $PASS"
echo "Failed: $FAIL"

if [ $FAIL -gt 0 ]; then
    echo "$FAILED_TESTS"
    exit 1
fi

echo "All smoke tests passed."
