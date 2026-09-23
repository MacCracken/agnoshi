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
agnsh --version    # "agnoshi 1.9.9"
agnsh --help       # usage summary
man agnsh          # full reference
```

## First Commands

Drop into the interactive shell with no arguments. The prompt carries the
current mode (default is `[ASSIST]`):

```
$ agnsh
agnoshi 1.9.9
AI-native shell -- type a natural-language command, or 'exit' to quit.
Built-ins: help, version, mode, history, clear, exit, reboot, poweroff, halt
Run: run /abs/path   (bareword / pipeline / redirect launching is AGNOS-only)

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

| Mode | Description | Confirms a program launch? |
|------|-------------|----------------------------|
| `human` | You drive; agnsh still classifies and audits | **Yes** |
| `assist` (default) | Parses and reports the translation | No |
| `auto` | Same, without the assist framing | No |
| `strict` | Most conservative posture | **Yes** |

Change mode interactively with `mode <name>`, or start with `--mode <name>`
(an unrecognised name is an error — it will not silently fall back).

⚠ **What the mode does NOT change today.** No mode hands you a raw shell, and no
mode disables classification — every mode classifies, reports risk and writes an
audit record. What `human` / `strict` add is a **confirmation prompt before a
program launch** (`run`, or an AGNOS bareword). There is no interactive approval
prompt for permission tiers yet: a SYSTEM_WRITE or ADMIN command prints
`Approval required` and is not executed, because **the natural-language path
does not execute at all** (see below).

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
{"timestamp":"2026-08-30T00:05:41Z","user":"user","mode":"AI-ASSIST","input":"show me all files in /tmp","action":"ls","approved":1,"result":"proposed","exit_code":null}
```

`result` comes from one of **two disjoint sets**, so a single `jq select` can
separate what the shell *decided* from what it *did*:

**Parse-time** (nothing ran): `proposed` · `needs_approval` (HIGH-risk) ·
`blocked` · `needs_llm` (question; no LLM wired) · `needs_exec` ·
`rejected_safety` (a translator refused path traversal, shell metacharacters, a
leading-dash argument, …).

**Exec-time** (something ran, or was refused before running): `launched` ·
`executed` · `failed` · `error` · `denied`. These carry `exit_code`; the
parse-time ones carry `exit_code: null`.

A `launched` line is written **before** the child starts, so a program that hangs
or takes the machine down still leaves a trace — a `launched` with no matching
outcome is the signal.

```bash
jq 'select(.result == "executed" or .result == "failed")' ~/.agnsh_audit.log   # what ran
jq 'select(.result == "rejected_safety")'                ~/.agnsh_audit.log   # what to rephrase
```

All fields are JSON-escaped and UTF-8 validated — crafted input cannot forge
entries, nor make the log unparseable.

## What actually executes

⚠ **The natural-language path does not execute anything.** Typing
`show me all files` prints the translation, its risk, and writes an audit
record — it does not run `ls`. That is why runnable inputs log
`result: proposed`.

What *does* execute:

- **Any host or AGNOS**: `run /abs/path` — validated, mode-gated, audited.
- **AGNOS only**: bareword `/bin/<name> [args]`, two-stage pipelines
  `cmd1 | cmd2`, output redirection `cmd > file`, and background jobs `prog &`.

Wiring execution into the NL path is roadmap 1.10.x — NL exec.

## Undo — not available yet

`src/checkpoint.cyr` implements checkpoint-before-destructive-op and an `undo`
builtin, but it is **not compiled into the binary**: there is no `undo` command
and no `~/.agnoshi/checkpoints/` directory. Do not rely on rollback. (Its wire-up
is blocked on seven stdlib symbols that no longer exist.)

## Next Steps

- `docs/guides/writing-intents.md` — add new NL intents
- `docs/guides/security-model.md` — deep dive on the permission system
- `docs/architecture/overview.md` — module map, data flow
- `docs/adr/` — architectural decision records
