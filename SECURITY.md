# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 2.1.x   | Yes       |
| < 2.1   | No        |

## Reporting a Vulnerability

If you discover a security vulnerability, please report it responsibly:

1. **Do not** open a public issue
2. Email security details to the maintainers
3. Include steps to reproduce if possible
4. Allow reasonable time for a fix before disclosure

## Security Model

Agnoshi enforces defense in depth:

### Command classification (6-tier)
Every command is classified before execution:
- **SAFE** — no state change (cd, echo, help)
- **READ_ONLY** — queries (ls, cat, ps)
- **USER_WRITE** — user files (cp, mv, mkdir, a plain rm; asks first since 2.1.3)
- **SYSTEM_WRITE** — system paths (asks first)
- **ADMIN** — asks first, then runs as `sudo -n` on a Linux host (2.1.3)
- **BLOCKED** — never allowed for AI (rm -rf, dd, mkfs, chmod, chown, shred)

Classification uses **basename extraction** so `/usr/bin/dd`, `./rm`,
`../bin/chmod` cannot bypass the blocklist.

### Approval workflow (2.1.3)

Since 2.1.3 ([ADR-011](docs/adr/011-approval-gated-exec.md)), a natural-language line in the
USER_WRITE, SYSTEM_WRITE or ADMIN tier **asks first, in every mode**, and runs only on a yes.

- **The prompt:** it goes to stderr, as `  ⚠ [MED] approve: <line> ? [y/N]`.
  - Control bytes in the line are stripped before it is shown (the H5 mitigation), so a crafted file
    name cannot redraw what you are approving.
  - `y` or `Y` approves. Anything else declines, including a bare Enter.
  - With nothing on stdin (EOF), there is no answer and nothing runs.
  - On a Linux host, no answer within 300 seconds times out and declines.
- **The prompt stands in for the mode gate:** `human` and `strict` ask once, not twice.
- **What it covers:**
  - A plain remove is USER_WRITE; a recursive one stays BLOCKED.
  - A remove or move is **checkpointed before the prompt**, and a file that cannot be checkpointed
    refuses the command unasked (see *Checkpoint/undo*).
  - On the host, an approved ADMIN line runs as `sudo -n` (see *Privilege escalation*).
- **BLOCKED still never runs** (`WARNING: BLOCKED -- not executed, and there is no override`). In a
  restricted session — agnsh as root on a Linux host — no natural-language line runs at all, so
  nothing is asked.
- **`agnsh -c`:** the answer is read from stdin, so a script that gives none gets 126, as before.
- **Audit:** each prompt leaves a record — `approved` before the launch, `denied`, or `timed_out`.

Until 2.1.3 these tiers printed `Approval required -- not executed` and never ran. The prompt
(`ApprovalManager_request`) was compiled in with no caller.

### Execution — what can actually run

Since 2.0.0 ([ADR-008](docs/adr/008-nl-exec-contract.md)):

- **Shell lines.** On a Linux host: a bareword program found on `$PATH` (only its
  *absolute* entries are searched — an empty or relative entry, which would mean
  the current directory, never is) and `run /abs/path ARGS…`. On AGNOS: bareword
  `/bin/<name>`, `run`, two-stage pipelines, `>` redirection and `prog &`. A
  `|`, `>` or `&` line on the host is refused, never read as natural language.
- **Natural language.** A SAFE or READ_ONLY translation runs through the same
  launcher as a typed `run` of the same command. Nothing else runs.
- **Restricted sessions** (2.0.2). agnsh running as root on a Linux host (uid or
  euid 0) reports every natural-language line and runs **none** of them, whatever
  the tier or mode: `-c` exits 126 (`running as root (restricted)`), the
  interactive shell warns before its first prompt, and the audit log records the
  refusal as `denied`. The user's own shell lines still run. AGNOS, which has no
  Unix uid model, is never restricted.
- **Every launch** is an argument vector — never `/bin/sh -c` — after the line
  passes `is_safe_path` (no traversal, no shell metacharacter). `human` and
  `strict` confirm first, and **a typed line the classifier calls BLOCKED asks
  `[y/N]` in every mode**, `auto` included; with no answer (`-c` with nothing on
  stdin) it is declined. The AI never runs a BLOCKED command; a human may, after
  an explicit yes.
- Host programs inherit agnsh's environment (until 2.0.0 the host `run` passed an
  empty one), as AGNOS programs always have.
- On AGNOS a pipeline stage starts with fds 0/1/2 and nothing else of the shell's
  (`SPAWN_F_CLEANFD`, 2.0.1) — not even the other end of its own pipe, which is
  what let a producer outlive its consumer and wedge the shell until 2.0.1.

Every launch and every refusal is recorded in the audit log (see below).

### Input sanitization
All user-controlled strings flowing to syscalls must pass validation:
- `is_safe_arg` — rejects shell metacharacters (`; | & $ ( ) < > ` ` \n`)
- `is_safe_path` — rejects path traversal (`..`) and metacharacters
- `is_valid_pid` — rejects PIDs <= 0 or > Linux maximum
- `is_safe_username` — only `[a-zA-Z0-9_-]`
- `is_safe_branch_name` — only `[a-zA-Z0-9_/.-]`
- `is_safe_commit_message` — rejects leading `-` (flag injection)

### Checkpoint/undo (2.1.3 / 2.1.4)

Since 2.1.3 the checkpoint store (`src/checkpoint.cyr`,
[ADR-009](docs/adr/009-checkpoint-store.md)) runs before every approved natural-language remove or
move:
- **A remove** saves each file it names.
- **A move** records the move, and saves a file it would overwrite.
- **What is refused:** anything the store cannot save refuses the command — a directory, a symlink,
  a device or FIFO, a file over 64 MiB, an unreadable file, or an unusable store folder. Nothing
  destructive runs without its checkpoint.
- **Declined or failed commands** leave no checkpoint behind.

**`undo`** (2.1.4) restores the newest group: each removed file, recreated from its checkpoint, and
each move, renamed back. It never overwrites. An entry whose path is taken stays in the store with
its reason, and `undo` exits 1. Two gaps:
- `cp` onto an existing file is not checkpointed.
- Typed shell lines (`run`, barewords) are not checkpointed: they are your own commands, not the AI's.

What the store guarantees:
- a private folder (below);
- each saved file 0600, created exclusively and never through a symlink;
- regular files only, up to 64 MiB;
- a restore that never overwrites what is at a path now.

### Audit log
Every action is logged as a JSON line to `~/.agnsh_audit.log`. All fields
are JSON-escaped (C4 mitigation) so crafted input cannot forge entries.

⛔ **On AGNOS the log only accumulates from 1.9.14.** agnos ignores `O_APPEND`, so
before 1.9.14 each record overwrote the previous one and a session left roughly
its last record behind (measured in QEMU; see
`docs/guides/security-model.md` § Audit Log Integrity). Linux hosts were not
affected.

### Privilege escalation (2.1.3)

Since 2.1.3 an **approved ADMIN natural-language line** on a Linux host runs as `<sudo> -n <line>`
(`security_sudo_line`, `src/security.cyr`):
- **sudo is re-verified** as a root-owned binary at the moment of escalation (`verify_sudo_path`), not
  trusted from the session's start.
- **`-n` means sudo never asks for a password.** A sudo that needs one fails, and the outcome record
  says so.
- **The line runs through the one host launcher**, with agnsh's environment, never a shell.
- **Nothing else invokes sudo:** typed shell lines run as you.
- **A restricted session** (agnsh as root) runs no natural-language line, so it never escalates.
- **agnos has no sudo and no uid model**, so an approved ADMIN line there runs as agnsh does.

The environment whitelist (`build_safe_env`, `src/sanitize.cyr`) still has no caller anywhere. Until
2.1.3 the escalation code (`execute_with_privileges`) had no caller and launched through a second,
empty-environment launcher; that is gone.

⚠ **Children inherit agnsh's environment on AGNOS, by design** — the opposite of
a whitelist (`sh_build_env_blob`, `src/run_agnos.cyr`). Any `LD_PRELOAD`-style
protection is a future property, not a current one.

### File permissions

- History file: mode **0600 at open**, opened `O_NOFOLLOW`; a file that already
  existed with a looser mode is repaired through the **open descriptor**
  (`fchmod`, since 1.9.15 — before that a path chmod after close, which a
  symlink swapped in after the open could redirect)
- Audit log: mode 0600, opened `O_NOFOLLOW`, and its mode is **re-asserted on
  every open**, through the open descriptor since 1.9.15, so a file restored
  from a backup cannot stay world-readable. On agnos there are no permission
  bits to set.
- Report folder (2.0.0): `$XDG_STATE_HOME/agnoshi/reports/` (default
  `~/.local/state/agnoshi/reports/`; `/.agnsh_reports/` on AGNOS), mode 0700, each
  report 0600 and created exclusively with `O_NOFOLLOW`. On the host a folder that
  is a symlink, belongs to another user or is writable by group or others is
  refused and no report is written.
- Checkpoint store (2.0.3; written since 2.1.3): `$XDG_STATE_HOME/agnoshi/checkpoints/`
  (default `~/.local/state/agnoshi/checkpoints/`; `/.agnsh_checkpoints/` on
  AGNOS), under the report folder's rules — 0700, and on the host refused unless
  it is a real directory you own that nobody else can write; each entry 0600.

⚠ When `$HOME` is unset the history and audit log fall back to `/tmp/<name>.<uid>`
(the report folder to `/tmp/agnoshi-reports.<uid>`, which the ownership check
above guards). The uid qualifier
prevents collision between users, but `/tmp` remains world-writable: treat that
fallback as degraded operation, not a supported configuration.

## Security Audit

See `docs/audit/` for audit reports. The 2026-04-13 audit surfaced 21
findings (5 critical, 7 high, 9 medium) — all resolved before v1.0.0.

Security regression tests in `tests/test_security.tcyr` exercise every
finding and must continue to pass.

## OWASP Alignment

⚠ Alignment is stated against the CURRENT binary; items resting on unwired
modules are marked.

- **ASI01 (Prompt Injection)**: ⚠ **not applicable yet** — there is no LLM
  integration in the binary (no `src/llm.cyr`, no network stack compiled in), so
  nothing is sent to a model and nothing is sanitized for one.
- **ASI02 (Unauthorized Actions)**: permission tiers and basename classification
  ship and work (`/usr/bin/dd` → `dd` → BLOCKED). Since 2.0.0 they decide what a
  natural-language line may run, and since 2.1.3 the USER_WRITE, SYSTEM_WRITE and ADMIN tiers run
  only after an approval. A typed BLOCKED line confirms in every mode.
- **ASI03 (Insecure Integration)**: ⚠ **not implemented** — there is no sandbox
  and no dotfile protection in the binary.
