# Security Model Deep Dive

This guide explains how agnsh protects the user and system.

## Threat Model

**In scope:**
- User accidentally running destructive commands
- AI misinterpreting intent and running dangerous commands
- Command injection via malicious natural language input
- Audit log tampering
- Privilege escalation via crafted inputs
- Terminal escape sequences manipulating approval UI
- Path traversal
- Symlink attacks on state files

**Out of scope:**
- Kernel exploits (covered by AGNOS at the kernel layer)
- Physical access to the machine
- Malicious Cyrius compiler (covered by Cyrius bootstrap chain)
- Compromised LLM gateway (runs as a separate process; agnsh treats its
  output as untrusted)

## Defense Layers

### 1. Classification

Every command maps to one of six `PermissionLevel` values. The classifier
in `src/permissions.cyr:analyze_command_permission` uses
**basename extraction** via `src/sanitize.cyr:get_command_basename`:

```
/usr/bin/dd    -> basename("dd")    -> BLOCKED
./rm           -> basename("rm")    -> ADMIN (or BLOCKED with dangerous flags)
../bin/chmod   -> basename("chmod") -> BLOCKED
ls             -> basename("ls")    -> READ_ONLY
```

This prevents path-based bypass of the blocklist.

### 2. Argument Sanitization

User-controlled strings reaching `execve` must pass `is_safe_arg`:

```cyrius
if (is_safe_arg(user_input) == 0) { return translate_unknown(intent); }
```

Rejected characters: `; | & $ ( ) < > ` ` \n`

For paths, `is_safe_path` additionally rejects `..` (traversal).

### 3. rm Flag Parsing

The rm classifier parses every argument to detect dangerous flags even
when combined or reordered:

- Long form: `--recursive`, `--force`, `--no-preserve-root`
- Short form: per-character scan for `r`, `f`, `R` in any `-` prefixed arg
- `--` (end-of-flags marker): always flagged as dangerous

### 4. Approval UI Hardening

The approval prompt displays the command via `print_str_safe`, which strips
control characters (< 0x20 except space, and 0x7F). A malicious command
cannot inject ANSI escape sequences to clear the screen, fake approval, or
hide the real command.

### 5. Audit Log Integrity

`AuditEntry_to_json` manually builds the JSON using `json_escape` for every
string field. Quotes, backslashes, newlines, tabs, and control chars are
all escaped. Crafted input cannot terminate a string early and inject
fake fields.

### 6. Checkpoint / Undo

Destructive operations are backed up to `$HOME/.agnoshi/checkpoints/`
(mode 0700, NOT `/tmp`) before execution:

- `rm file.txt` → `cp file.txt ~/.agnoshi/checkpoints/N_file.txt` before unlink
- `mv a b` → records source/dest mapping

`undo` pops the last checkpoint and restores. Auto-prune keeps 100 most
recent entries.

### 7. Privilege Escalation

The `SecurityContext`:

- Checks **effective UID** (catches setuid binaries where `uid != 0` but
  `euid == 0`)
- Sudo presence and **root ownership** re-verified at escalation time, not
  just at shell init (catches post-init tampering)
- Child processes inherit a **whitelist environment** (PATH, HOME, LANG,
  TERM only) — no `LD_PRELOAD`, `LD_LIBRARY_PATH`, `SUDO_*`

### 8. Terminal Input Paths

- Git branch name from `.git/HEAD` passes through `is_safe_branch_name`
  and `strip_control_chars` before prompt display
- Commit messages checked for leading `-` (flag injection)
- Usernames from `/etc/passwd` pass `is_safe_username` regex

## File Permissions

| File | Mode | Why |
|------|------|-----|
| `~/.agnsh_history` | 0600 | Contains command history — may reveal secrets |
| `~/.agnsh_audit.log` | 0600 | Forensic record — tampering breaks investigations |
| `~/.agnoshi/checkpoints/` | 0700 | Contains backed-up file contents from `rm` |
| `/usr/local/bin/agnsh` | 0755 | Binary — exec, not writable by users |

## What Can Still Go Wrong

**Things agnsh cannot prevent:**

- User explicitly choosing `mode human` and running `rm -rf /` directly
  (agnsh steps out of the way)
- User installing a malicious alias that expands to dangerous commands
  (mitigated by rejecting aliases with metacharacters, but still: caveat
  user)
- Race conditions between permission check and execution (TOCTOU) if the
  filesystem is mutated by another process. Full TOCTOU protection would
  require inode-locking at the kernel layer.

## Incident Response

If you suspect the audit log has been tampered with:

1. Check file modification times vs the `timestamp` fields
2. Verify file permissions (should be 0600)
3. JSON-parse each line — malformed lines indicate tampering or disk error
4. Compare against system audit (`auditd`) if available

## Reporting Vulnerabilities

See SECURITY.md — email the maintainers rather than opening a public issue.
