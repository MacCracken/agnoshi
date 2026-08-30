# Scripting with agnsh

Agnsh is primarily interactive but supports one-shot mode via `-c` for scripting.

## One-shot

```bash
agnsh -c "show running processes"
```

The translation report goes to **stdout**; diagnostics and errors go to
**stderr** (since 1.9.4), so `agnsh -c "..." 2>/dev/null | ...` gives you a clean
stream. Exit code is 0 on a successful parse — including for a BLOCKED command,
since classification and execution are separate concerns and nothing is executed
on this path.

## Piping Input

```bash
echo "show disk usage" | agnsh
```

Agnsh reads lines from stdin in interactive mode. Use `exit` or close the
pipe to terminate.

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

### Bash/Zsh alias for NL-first

```bash
alias ask='agnsh -c'
ask "show system info"
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
# Verify agnsh can classify expected commands
for cmd in "show running processes" "git status" "show disk usage"; do
    if ! agnsh -c "$cmd" | grep -q 'Intent:'; then
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

Agnsh `-c` output is line-oriented text:

```
Intent: <tag>  Command: <cmd>
  Risk: [LOW|MED|HIGH|CRIT]
  [WARNING: BLOCKED -- would not execute without explicit override]
  [Approval required (interactive prompt in shell mode)]
  [Hint: <recovery hint when translation isn't directly runnable>]
```

For structured output, parse `~/.agnsh_audit.log` — every `-c` invocation
appends one JSON line with `timestamp`, `user`, `mode`, `input`, `action`,
`approved` (0/1), `result`, and `exit_code` (JSON `null` when the record
describes something that never ran).

`result` comes from one of two **disjoint** sets, so a single `select` separates
"what the shell decided" from "what it did":

- parse-time: `proposed`, `needs_approval`, `blocked`, `needs_llm`,
  `needs_exec`, `rejected_safety`
- exec-time (1.9.2): `launched`, `executed`, `failed`, `error`, `denied`

⚠ **`error` vs `failed` is the distinction worth knowing**: `error` means the
launch never happened (missing binary, exec refused) and carries no useful exit
code; `failed` means the child ran to completion and returned non-zero. Until
1.9.10 the **host** build blurred them — a missing binary went through
fork/exec, so the fork succeeded and the shell logged `launched` followed by
`failed` with `exit_code: 127` for a program that never started. It now logs a
single `error`. agnos was never affected.

⚠ **A file that exists but is not executable is still reported as `failed`**
(with the child's `126`), because the fork/exec path gets no feedback from the
failed exec. Stated so a script reading these labels knows the edge it has.

Downstream:

```sh
# Find all parser-rejected inputs in this session
jq 'select(.result == "rejected_safety") | .input' < ~/.agnsh_audit.log

# Count commands by result class
jq -r '.result' < ~/.agnsh_audit.log | sort | uniq -c
```

For native JSON-on-stdout `-c` output (rather than going through the
audit log), see the roadmap.
