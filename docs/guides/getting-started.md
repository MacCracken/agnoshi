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
agnsh --version    # "agnoshi 1.0.0"
agnsh --help       # usage summary
man agnsh          # full reference
```

## First Commands

Drop into the interactive shell with no arguments:

```
$ agnsh
agnoshi 1.0.0 -- AI-native shell
Type a natural language command or 'exit' to quit.

> show me all files in /tmp
Intent: 0
  Command: ls
  Permission: 1

> git status
Intent: 22
  Command: git
  Permission: 1

> exit
bye
```

Or run a single command:

```bash
agnsh -c "install vim"
```

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
{"timestamp":"2026-04-13T18:00:00Z","user":"alice","mode":"assist","input":"show files","action":"ls","approved":1,"result":"success"}
```

All fields are JSON-escaped — crafted input cannot forge entries.

## Undo

Destructive operations (`rm`, `mv`) are checkpointed to
`~/.agnoshi/checkpoints/` before execution. Type `undo` in the interactive
shell to restore the most recent operation.

The checkpoint dir is owner-only (0700) and auto-prunes to the 100 most
recent entries.

## Next Steps

- `docs/guides/writing-intents.md` — add new NL intents
- `docs/guides/security-model.md` — deep dive on the permission system
- `docs/architecture/overview.md` — module map, data flow
- `docs/adr/` — architectural decision records
