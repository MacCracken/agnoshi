# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 1.0.x   | Yes       |
| < 1.0   | No        |

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

### Approval workflow
Risky commands trigger the `ApprovalManager` — a human must approve before
execution. Terminal escape sequences are stripped from the displayed command
(H5 mitigation) so a crafted input cannot spoof the approval prompt.

### Input sanitization
All user-controlled strings flowing to syscalls must pass validation:
- `is_safe_arg` — rejects shell metacharacters (`; | & $ ( ) < > ` ` \n`)
- `is_safe_path` — rejects path traversal (`..`) and metacharacters
- `is_valid_pid` — rejects PIDs <= 0 or > Linux maximum
- `is_safe_username` — only `[a-zA-Z0-9_-]`
- `is_safe_branch_name` — only `[a-zA-Z0-9_/.-]`
- `is_safe_commit_message` — rejects leading `-` (flag injection)

### Checkpoint/undo
Destructive operations (`rm`, `mv`) back up targets to
`$HOME/.agnoshi/checkpoints/` (mode 0700) before execution. `undo` restores
the last operation. Auto-prune keeps the 100 most recent entries.

### Audit log
Every action is logged as a JSON line to `~/.agnsh_audit.log`. All fields
are JSON-escaped (C4 mitigation) so crafted input cannot forge entries.

### Privilege escalation
- `euid == 0` detection forces restricted mode (even for setuid binaries)
- Clean environment whitelist for child processes (no `LD_PRELOAD` inheritance)
- Sudo path and root-ownership re-verified at escalation time, not just init

### File permissions
- History file: mode 0600 (owner read/write only)
- Checkpoint directory: mode 0700 (owner only, in `$HOME` not `/tmp`)
- Audit log: mode 0600

## Security Audit

See `docs/audit/` for audit reports. The 2026-04-13 audit surfaced 21
findings (5 critical, 7 high, 9 medium) — all resolved before v1.0.0.

Security regression tests in `tests/test_security.tcyr` exercise every
finding and must continue to pass.

## OWASP Alignment

- **ASI01 (Prompt Injection)**: external content sanitized before LLM prompts; role-override patterns stripped
- **ASI02 (Unauthorized Actions)**: approval workflows; permission tiers; basename classification prevents path bypass
- **ASI03 (Insecure Integration)**: sandbox (when available) protects dotfiles as read-only
