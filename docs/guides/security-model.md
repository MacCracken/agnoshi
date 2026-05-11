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

User-controlled strings reaching `execve` must pass `is_safe_arg`
(cstring-typed) or `safe_arg_in_str` (Str-typed; for parser-extracted
values). The two-variant split is required because Cyrius distinguishes
`cstring` (null-terminated) and `Str` (fat-pointer) types and the wrong
helper on the wrong type causes silent runtime fallthroughs — see
[ADR-006](../adr/006-cstr-str-dispatch-discipline.md) for the discipline.

```cyrius
# Cstring caller (permissions module compares against literal cmd names):
if (is_safe_arg(arg_cstring) == 0) { return translate_unknown(intent); }

# Str caller (translator-side; receives parser-extracted Str):
if (safe_arg_in_str(load64(intent + 8)) == 0) { return translate_unknown(intent); }
```

Rejected characters: `; | & $ ( ) < > ` ` \n`

For paths, `is_safe_path` / `safe_path_in_str` additionally rejects `..`
(traversal). v1.3.1's CI lint shield (`scripts/lint-cstr-str.sh`)
enforces the cstring-vs-Str dispatch at lint time — calling
`is_safe_path(Str)` would have silently routed every NL filesystem op
to `translate_unknown` since v1.0; v1.3.0 slice 3 caught it.

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

**Known LOW-severity hardening deferred to v1.4.0** (per `docs/audit/2026-05-11-pminus1.md`):

- **Symlink races on state-file open**. `~/.agnsh_audit.log` and
  `~/.agnsh_history` are opened without `O_NOFOLLOW`. An attacker who
  can place a symlink at one of those paths *before* agnsh runs can
  redirect the write. Requires pre-existing attacker write access to
  `$HOME` — unusual on single-user systems; possible on shared-NFS or
  container setup phases. Mitigation: add `O_NOFOLLOW` to the open
  flags (value differs per arch — `0o400000` on x86_64,
  `0o100000` on aarch64-generic — needs a per-arch constant +
  `lib/io.cyr` API extension).
- **chmod-failure logging**. v1.3.1 added a stderr warning when
  `sys_chmod` returns non-zero on the history / checkpoint paths. If
  chmod silently fails the file stays at the umask default (typically
  0644), leaking history to other users on a multi-user system. The
  warning is the operator-visible signal.

## Forward Shield (v1.3.1)

The v1.3.1 P(-1) audit added `scripts/lint-cstr-str.sh` — a 14-pattern
CI gate covering five bug categories: Str-typed fns with cstring arg
(1st position × 5 + 2nd position × 3), cross-arch-broken raw syscalls
(SYS_OPEN / CHMOD / STAT × 3), static-buffer escape via `str_from(&buf)`
(× 2), unchecked `sys_chmod` return (× 1). Together they catch the
seven distinct bug variants that surfaced over v1.2.0/v1.3.0 at lint
time — each previously took a probe / SIGSEGV / first-use crash to
discover. See [ADR-006](../adr/006-cstr-str-dispatch-discipline.md) for
the operational rules, and [`docs/audit/2026-05-11-pminus1.md`](../audit/2026-05-11-pminus1.md)
for the full audit pass.

## Incident Response

If you suspect the audit log has been tampered with:

1. Check file modification times vs the `timestamp` fields
2. Verify file permissions (should be 0600)
3. JSON-parse each line — malformed lines indicate tampering or disk error
4. Compare against system audit (`auditd`) if available

## Reporting Vulnerabilities

See SECURITY.md — email the maintainers rather than opening a public issue.
