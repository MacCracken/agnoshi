# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 1.9.x   | Yes       |
| < 1.9   | No        |

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
- **USER_WRITE** — user files (cp, mv, touch)
- **SYSTEM_WRITE** — system paths (requires approval)
- **ADMIN** — requires sudo (requires approval)
- **BLOCKED** — never allowed for AI (rm -rf, dd, mkfs, chmod, chown, shred)

Classification uses **basename extraction** so `/usr/bin/dd`, `./rm`,
`../bin/chmod` cannot bypass the blocklist.

### Approval workflow — ⚠ NOT IN THE SHIPPED BINARY YET

**What ships today**: every translated command is classified into a permission
tier and its risk is reported. Since 2.0.0 a SAFE or READ_ONLY translation
**runs** (see *Execution*, below); a USER_WRITE, SYSTEM_WRITE or ADMIN one prints
`Approval required -- not executed (no approval prompt in this build)` and a
BLOCKED one prints `WARNING: BLOCKED -- not executed, and there is no override`.
**Neither prompts, and neither runs**: with no approval prompt, "requires
approval" means "does not run". Until 1.9.15 the HIGH-risk line promised an
"interactive prompt in shell mode" that no mode had.

**What does not ship**: `ApprovalManager_request` and the escape-stripped
approval display (`print_str_safe`, the H5 mitigation) live in
`src/approval.cyr` / `src/sanitize.cyr` but their only caller is
`src/session.cyr`, which is **not in `src/agnsh.cyr`'s include graph**. Wiring
them in is roadmap 2.0.x — approval-gated exec.

### Execution — what can actually run

Since 2.0.0 ([ADR-008](docs/adr/008-nl-exec-contract.md)):

- **Shell lines.** On a Linux host: a bareword program found on `$PATH` (only its
  *absolute* entries are searched — an empty or relative entry, which would mean
  the current directory, never is) and `run /abs/path ARGS…`. On AGNOS: bareword
  `/bin/<name>`, `run`, two-stage pipelines, `>` redirection and `prog &`. A
  `|`, `>` or `&` line on the host is refused, never read as natural language.
- **Natural language.** A SAFE or READ_ONLY translation runs through the same
  launcher as a typed `run` of the same command. Nothing else runs.
- **Every launch** is an argument vector — never `/bin/sh -c` — after the line
  passes `is_safe_path` (no traversal, no shell metacharacter). `human` and
  `strict` confirm first, and **a typed line the classifier calls BLOCKED asks
  `[y/N]` in every mode**, `auto` included; with no answer (`-c` with nothing on
  stdin) it is declined. The AI never runs a BLOCKED command; a human may, after
  an explicit yes.
- Host programs inherit agnsh's environment (until 2.0.0 the host `run` passed an
  empty one), as AGNOS programs always have.

Every launch and every refusal is recorded in the audit log (see below).

### Input sanitization
All user-controlled strings flowing to syscalls must pass validation:
- `is_safe_arg` — rejects shell metacharacters (`; | & $ ( ) < > ` ` \n`)
- `is_safe_path` — rejects path traversal (`..`) and metacharacters
- `is_valid_pid` — rejects PIDs <= 0 or > Linux maximum
- `is_safe_username` — only `[a-zA-Z0-9_-]`
- `is_safe_branch_name` — only `[a-zA-Z0-9_/.-]`
- `is_safe_commit_message` — rejects leading `-` (flag injection)

### Checkpoint/undo — ⚠ NOT IN THE SHIPPED BINARY YET

`src/checkpoint.cyr` implements backup-before-destructive-op and `undo`, but it
is **not in the binary's include graph** and there is no `undo` builtin. Do not
rely on any rollback guarantee. (It calls seven stdlib helpers that no longer
exist, so its wire-up is a re-implementation — roadmap 2.0.x — checkpointing.)

### Audit log
Every action is logged as a JSON line to `~/.agnsh_audit.log`. All fields
are JSON-escaped (C4 mitigation) so crafted input cannot forge entries.

⛔ **On AGNOS the log only accumulates from 1.9.14.** agnos ignores `O_APPEND`, so
before 1.9.14 each record overwrote the previous one and a session left roughly
its last record behind (measured in QEMU; see
`docs/guides/security-model.md` § Audit Log Integrity). Linux hosts were not
affected.

### Privilege escalation — ⚠ NOT IN THE SHIPPED BINARY YET

There is **no privilege-escalation path in the binary at all**: nothing invokes
`sudo`. The `euid == 0` restricted-mode check, the sudo path/root-ownership
re-verification (`verify_sudo_path`) and the environment whitelist
(`build_safe_env`) exist in `src/security.cyr` / `src/sanitize.cyr` but have no
caller in the include graph — `build_safe_env` has no caller anywhere.

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
- Checkpoint directory: mode 0700 — *in the unwired module; see above*

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
  natural-language line may run (SAFE and READ_ONLY only), and a typed BLOCKED
  line confirms in every mode. ⚠ Approval *workflows* do not ship — see above.
- **ASI03 (Insecure Integration)**: ⚠ **not implemented** — there is no sandbox
  and no dotfile protection in the binary.
