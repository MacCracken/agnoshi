# Scripting with agnsh

Agnsh is primarily interactive but supports one-shot mode via `-c` for scripting. **2.0.0 changed
what `-c` does** — see [Migrating from 1.x](#migrating-from-1x) — and
[ADR-008](../adr/008-nl-exec-contract.md) is the contract.

## One-shot

```bash
agnsh -c "show running processes"
```

`-c` **runs** the line, and stdout is the program's:

- A **shell line** runs as the program it names: a bareword found on `$PATH` (`agnsh -c "ls -la"`)
  or `run /abs/path ARGS…`. On AGNOS, `/bin/<name>`, `cmd1 | cmd2`, `cmd > file` and `prog &` too;
  on the Linux host those three are refused — they are shell syntax the host cannot run yet, never
  natural language.
- A **natural-language line** is classified, and runs if its translation is SAFE or READ_ONLY
  (`ls`, `cat`, `grep`, `find`, `ps`, `df`, `free`, `git status`, …). USER_WRITE, SYSTEM_WRITE and
  ADMIN translations are not run yet (approval-gated execution is roadmap 2.0.x); BLOCKED never is.
- A typed line the classifier calls **BLOCKED** (`rm -rf`, `dd`, `mkfs`, `chmod`, `chown`, `shred`)
  asks `[y/N]` on stderr in every mode — `-c` with nothing on stdin declines it.
- `--mode human` / `--mode strict` confirm every launch first; `-c` defaults to `auto`.

agnsh's own words — notices, confirmation prompts, `run: exit N` — go to **stderr**, so
`agnsh -c "..." 2>/dev/null` is exactly the program's output.

### The report

For every natural-language line, agnsh files a report of what it understood and did — the
`Intent:` / `Command:` / `Risk:` lines, plus the input, mode, time and result:

```
input: show running processes
mode: AI-AUTO
time: 2026-09-24T03:55:31Z
Intent: 9  Command: ps
  Risk: [LOW]
result: executed, exit 0
```

It lives in the **report folder**, newest 100 kept, as `<YYYYMMDDTHHMMSSZ>-<pid>.txt` plus a copy in
`latest.txt`:

| target | report folder |
|---|---|
| Linux host | `$XDG_STATE_HOME/agnoshi/reports/`, default `~/.local/state/agnoshi/reports/` |
| AGNOS | `/.agnsh_reports/` |

The folder is 0700 and each report 0600 (a report holds the command line). On the host agnsh refuses
a folder that is a symlink, belongs to someone else, or is writable by others, and says so on stderr
— the command still runs. When a line does not run, stderr names the reason and the report:

```
agnsh: not executed: approval required -- report: /home/you/.local/state/agnoshi/reports/20260924T035531Z-4242.txt
```

### Classify without running: `--dry-run`

```bash
agnsh --dry-run -c "remove old.log"      # or -n
```

`--dry-run` runs **nothing** — no program, no pipeline, no power verb — and prints the report block
on stdout, exactly what `-c` printed before 2.0.0. It files the report too.

### Exit status

| outcome | status |
|---|---|
| the program ran | its exit status (128 + N if signal N killed it) |
| understood, not run: approval required, BLOCKED, declined at a prompt, refused arguments, a restricted session (root on a Linux host, 2.0.2) | 126 |
| nothing to run: not understood, a question, no such program, it could not be launched | 127 |
| usage error | 1 |

Under `--dry-run` the status is 0 for a line that *would* run, and 126 / 127 as above otherwise.

## Migrating from 1.x

| 1.x | 2.0.0 |
|---|---|
| `agnsh -c "$cmd" \| grep -q 'Intent:'` (classify) | `agnsh --dry-run -c "$cmd" \| grep -q 'Intent:'` |
| the report on stdout | stdout is the program's; the report is in the report folder (`latest.txt`) |
| exit 0 for any parse | the program's status, or 126 / 127 when nothing ran |
| a host `run` child got an empty environment | it inherits agnsh's (`PATH`, `HOME`, `LANG`, …) |
| `ls -la` on the host went to the NL parser | it runs `ls` (shell-first) |

A line whose first word is a program is that program's command line: on a host with Go installed,
`agnsh -c "go to /tmp"` runs `go`. Phrase natural language so it does not start with a program's
name, or use `--dry-run` to see how a line is read.

## Piping Input

```bash
echo "show disk usage" | agnsh
```

Agnsh reads lines from stdin in interactive mode — and since 2.0.0 runs them, as `-c` does. Use
`exit` or close the pipe to terminate. A confirmation prompt (`human` / `strict`, or a BLOCKED line)
reads its answer from the same stdin.

## Audit Log Processing

The audit log is newline-delimited JSON, parseable with `jq`:

```bash
# Count commands per mode
jq -r '.mode' ~/.agnsh_audit.log | sort | uniq -c

# Find denied operations
jq -c 'select(.approved == 0)' ~/.agnsh_audit.log

# Find BLOCKED classifications (all times someone tried a dangerous cmd)
jq -c 'select(.result == "blocked")' ~/.agnsh_audit.log

# Timeline for a specific user
jq -c 'select(.user == "alice")' ~/.agnsh_audit.log

# Everything that actually RAN, with its exit status
jq -c 'select(.result == "executed" or .result == "failed")
       | {timestamp, action, exit_code}' ~/.agnsh_audit.log

# A launch with no matching outcome = a program that hung or took the box down
jq -rc 'select(.result == "launched") | .input' ~/.agnsh_audit.log
```

## Shell Integration

### Bash/Zsh aliases

```bash
alias ask='agnsh -c'          # runs what it may
alias explain='agnsh -n -c'   # classifies only
ask "show system info"
explain "remove old.log"
```

### Conditional execution

```bash
# Run agnsh only if we detect NL keywords
if echo "$1" | grep -qE '\b(show|find|list|install)\b'; then
    agnsh -c "$1"
else
    sh -c "$1"
fi
```

### Smoke test before deploy

```bash
# Verify agnsh can classify expected commands -- without running them
for cmd in "show running processes" "git status" "show disk usage"; do
    if ! agnsh --dry-run -c "$cmd" | grep -q 'Intent:'; then
        echo "REGRESSION: agnsh failed on: $cmd" >&2
        exit 1
    fi
done
```

## In CI

```yaml
# .github/workflows/example.yml (snippet)
- name: Install agnsh
  run: |
    curl -sL https://github.com/MacCracken/agnoshi/releases/latest/download/agnsh > /usr/local/bin/agnsh
    chmod +x /usr/local/bin/agnsh

- name: Smoke test
  run: agnsh --version
```

## Programmatic Access

The report block — on `--dry-run`'s stdout, and inside every report file — is line-oriented text:

```
Intent: <tag>  Command: <cmd>
  Risk: [LOW|MED|HIGH|CRIT]
  [WARNING: BLOCKED -- not executed, and there is no override]
  [Approval required -- not executed (no approval prompt in this build)]
  [Hint: <why this translation is not a runnable command>]
```

A report file adds `input:`, `mode:` and `time:` lines before the block and a `result:` line after
it: `executed, exit N`, or `not executed: <reason>`.

For structured output, parse `~/.agnsh_audit.log` — every `-c` invocation appends JSON lines with
`timestamp`, `user`, `mode`, `input`, `action`, `approved` (0/1), `result`, and `exit_code` (JSON
`null` when the record describes something that never ran).

`result` comes from one of two **disjoint** sets, so a single `select` separates "what the shell
decided" from "what it did":

- parse-time: `proposed`, `needs_approval`, `blocked`, `needs_llm`, `needs_exec`,
  `rejected_safety`. Since 2.0.0 an understood USER_WRITE line is `needs_approval` (it does not run
  without approval); `proposed` means SAFE or READ_ONLY, and such a line then runs.
- exec-time (1.9.2): `launched`, `executed`, `failed`, `error`, `denied` — written by every launch,
  including a natural-language line that runs (2.0.0), so one NL line that runs leaves a
  `proposed`, a `launched` and its outcome.

⚠ **`error` vs `failed` is the distinction worth knowing**: `error` means the launch never happened
(missing binary, exec refused) and carries no useful exit code; `failed` means the child ran to
completion and returned non-zero. Until 1.9.10 the **host** build blurred them — a missing binary
went through fork/exec, so the fork succeeded and the shell logged `launched` followed by `failed`
with `exit_code: 127` for a program that never started. It now logs a single `error`. agnos was never
affected.

⚠ **A file that exists but is not executable is still reported as `failed`** by `run` (with the
child's `126`), because the fork/exec path gets no feedback from the failed exec. A bareword or an
NL command is found by the `$PATH` lookup, which only accepts files the user can execute. Stated so
a script reading these labels knows the edge it has.

Downstream:

```sh
# Find all parser-rejected inputs in this session
jq 'select(.result == "rejected_safety") | .input' < ~/.agnsh_audit.log

# Count commands by result class
jq -r '.result' < ~/.agnsh_audit.log | sort | uniq -c
```
