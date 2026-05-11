# Getting Started with agnsh

## Install

### From source (requires Cyrius toolchain)

```bash
git clone https://github.com/MacCracken/agnoshi.git
cd agnoshi
cyrius build src/agnsh.cyr build/agnsh
sudo sh scripts/install.sh
```

The install script places:
- `/usr/local/bin/agnsh` — the binary (mode 755)
- `/usr/local/share/agnoshi/` — README, CHANGELOG, LICENSE
- `/usr/local/share/man/man1/agnsh.1` — man page

### Verify

```bash
agnsh --version    # "agnoshi 1.3.1"
agnsh --help       # usage summary
man agnsh          # full reference
```

## First Commands

Drop into the interactive shell with no arguments. The prompt carries the
current mode (default is `[ASSIST]`):

```
$ agnsh
agnoshi 1.3.1
AI-native shell -- type a natural-language command, or 'exit' to quit.
Built-ins: help, version, mode, history, clear, exit

[ASSIST] > show me all files in /tmp
Intent: 0  Command: ls
  Risk: [LOW]

[ASSIST] > install vim
Intent: 14  Command: apt
  Risk: [HIGH]
  Approval required (interactive prompt in shell mode)

[ASSIST] > rm -rf /tmp/foo
Intent: 8  Command: rm
  Risk: [CRIT]
  WARNING: BLOCKED -- would not execute without explicit override

[ASSIST] > mode strict
Mode -> STRICT

[STRICT] > history
  1  show me all files in /tmp
  2  install vim
  3  rm -rf /tmp/foo
  4  mode strict

[STRICT] > exit
bye
```

Or run a single command:

```bash
agnsh -c "install vim"
```

The output carries `Intent: <tag>  Command: <cmd>` plus a `Risk: [LOW|MED|HIGH|CRIT]` line and, when appropriate, a `Hint:` line explaining why a particular input isn't directly runnable (LLM-routed question, pipeline without an exec wire-up yet, or a translator safety-check rejection).

## Operating Modes

Agnsh has four modes:

| Mode | Description | Approval |
|------|-------------|----------|
| `human` | Direct shell, no AI | Never (you just run commands) |
| `assist` (default) | AI parses and suggests; risky ops ask | For SYSTEM_WRITE, ADMIN, BLOCKED |
| `auto` | AI runs safe commands; escalates elevated ops | Only for elevated ops |
| `strict` | Every command requires approval | Always |

Change mode interactively with `mode <name>`.

## Permission Levels

Every command is classified before execution:

- **SAFE** — cd, echo, help (no state change)
- **READ_ONLY** — ls, cat, ps, grep (read-only)
- **USER_WRITE** — cp, mv, touch, mkdir (modifies user files)
- **SYSTEM_WRITE** — writes to /etc, /usr, /bin (requires approval)
- **ADMIN** — apt, systemctl, kill, iptables (requires sudo + approval)
- **BLOCKED** — rm -rf, dd, mkfs, chmod (never allowed for AI)

The classifier uses **basename extraction**, so `/usr/bin/dd` is still
BLOCKED — you can't hide dangerous commands behind absolute paths.

## Audit Log

Every action is recorded as a JSON line in `~/.agnsh_audit.log`:

```json
{"timestamp":"2026-05-11T18:00:00Z","user":"user","mode":"AI-ASSIST","input":"show files","action":"ls","approved":1,"result":"proposed"}
```

The `result` field is one of six labels — `proposed` (auto-runnable), `needs_approval` (HIGH-risk; SYSTEM_WRITE / ADMIN), `blocked` (BLOCKED-perm), `needs_llm` (question; LLM not yet wired), `needs_exec` (pipeline; exec not yet wired), `rejected_safety` (translator caught path traversal / shell metachars / leading-dash arg / invalid pid). Downstream filters can `jq 'select(.result == "rejected_safety")'` to find inputs that need rephrasing.

All fields are JSON-escaped — crafted input cannot forge entries.

## Undo (v1.4.0)

Destructive operations (`rm`, `mv`) are designed to be checkpointed to
`~/.agnoshi/checkpoints/` before execution, with an `undo` builtin to
restore the most recent operation. The `src/checkpoint.cyr` module is in
place but the wire-up to actual exec lands in v1.4.0 — today's `-c` and
interactive modes *propose* translations without executing them. The
audit log shows `result:proposed` for runnable inputs to reflect that.

## Next Steps

- `docs/guides/writing-intents.md` — add new NL intents
- `docs/guides/security-model.md` — deep dive on the permission system
- `docs/architecture/overview.md` — module map, data flow
- `docs/adr/` — architectural decision records
