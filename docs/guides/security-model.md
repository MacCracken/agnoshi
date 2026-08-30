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
(traversal). Both were fused into a single pass in 1.9.6 — the verdict is
unchanged, they are just no longer four traversals of the same string.

⚠ **The lint shield does not "enforce the cstring-vs-Str dispatch"** — it cannot.
`scripts/lint-cstr-str.sh` matches seven specific textual antipatterns (A–G):
literal arguments to Str-typed helpers, cross-arch-broken raw syscalls,
static-buffer escape, unchecked `sys_chmod`, `strlen` inside an `_in_str` body,
and `str_data()` handed to a path-taking syscall. Its known blind spot is a
cstring carried in a **variable**, which is invisible to the literal-matching
categories — that is precisely how the dead SHELL_COMMAND classifier (1.9.1)
and the `audit_format_table` defect (1.9.8) both passed a clean lint run. **A
green shield is not proof.**

### 3. rm Flag Parsing

The rm classifier parses every argument to detect dangerous flags even
when combined or reordered:

- Long form: `--recursive`, `--force`, `--no-preserve-root`
- Short form: per-character scan for `r`, `f`, `R` in any `-` prefixed arg
- `--` (end-of-flags marker): always flagged as dangerous

### 4. Approval UI Hardening — ⚠ NOT IN THE SHIPPED BINARY

`print_str_safe` (which does strip control characters) is called only from
`ApprovalManager_request` in `src/approval.cyr`, whose only caller is
`src/session.cyr` — **not in the binary's include graph**. There is no approval
prompt at runtime.

⚠ **The confirmation that DOES ship is not hardened.** In `human` / `strict`
mode a program launch is confirmed by `verb_confirm` (`src/sanitize.cyr`), which
writes the action string straight to the terminal with no stripping. A path
containing an ESC byte can therefore style or reposition the confirmation text.
The path itself has already passed `is_safe_path`, which rejects shell
metacharacters and traversal but **not** control bytes — so this is a real, if
narrow, gap. Tracked for the approval wire-up (Bucket 1 Slice 6).

### 5. Audit Log Integrity

`AuditEntry_to_json` manually builds the JSON using `json_escape` for every
string field. Quotes, backslashes, newlines, tabs, and control chars are
all escaped. Crafted input cannot terminate a string early and inject
fake fields.

⚠ **Escaping alone was not enough.** Until 1.9.1 bytes ≥ 0x80 were copied through
unvalidated, so a single high byte made the record invalid UTF-8 and therefore
invalid JSON — and on a whole-file parse one poisoned line takes the entire log
down with it, which an actor whose own entry would read `rejected_safety` could
trigger deliberately. UTF-8 is now **validated**: well-formed sequences pass
byte-exact (so `café` stays `café` rather than becoming mojibake), invalid bytes
become U+FFFD, and overlongs and surrogates are rejected.

**What is recorded.** Two disjoint label sets, so one `select` separates them:

- *parse-time* (the shell's decision): `proposed`, `needs_approval`, `blocked`,
  `rejected_safety`, `needs_llm`, `needs_exec`
- *exec-time* (what it actually did, 1.9.2): `launched`, `executed`, `failed`,
  `error`, `denied`, each with an `exit_code` field (JSON `null` when none applies)

A `launched` record is written **before** the child starts, so a program that
hangs, kills the shell, or reboots the machine still leaves a trace — a
`launched` with no matching outcome *is* the signal. Refusals are recorded too,
with `"approved":0`. `>` refuses to truncate the audit log or the history file.

### 6. Checkpoint / Undo — ⚠ NOT IN THE SHIPPED BINARY

`src/checkpoint.cyr` implements backup-before-destructive-op, `undo` and a
100-entry auto-prune, but it is **not in the binary's include graph**, there is
no `undo` builtin, and no `~/.agnoshi/checkpoints/` directory is ever created.
**Do not rely on any rollback guarantee.** Its wire-up is blocked on seven
stdlib symbols that no longer exist (roadmap Bucket 1 Slice 4).

### 7. Privilege Escalation — ⚠ NOT IN THE SHIPPED BINARY

**agnsh never escalates privileges.** Nothing in the binary invokes `sudo`. The
`SecurityContext` euid check and the sudo path/root-ownership re-verification
live in `src/security.cyr`, which is not in the include graph.

### 7b. Child environment — what actually happens

Neither target uses the documented whitelist (`build_safe_env` exists in
`src/sanitize.cyr` and has **no caller anywhere**). The two real behaviours:

- **Host** (`run /abs/path`): the child gets an **empty environment** —
  `lib/process.cyr`'s `_exec3` passes a NULL `envp`. Stronger than a whitelist
  for `LD_PRELOAD` purposes, since nothing is inherited at all.
- **AGNOS**: the child **inherits agnsh's entire environment**, deliberately —
  `sh_build_env_blob` walks agnsh's own envp and passes it on, clamped to
  ≤1024 B / ≤16 entries. This is the 1.7.0 env-inheritance feature. Today that
  environment is the kernel seed (`HOME=/`, `PWD=/`), so there is little to
  inherit — but it is inheritance, not filtering.

### 8. Terminal Input Paths

- Commit messages checked for leading `-` (flag injection). ✅ **Active** — and
  note this guard was *inert* until 1.9.1: it was cstring-typed while the parser
  handed it a `Str`, so `git commit -m -oh-no-a-flag` was accepted. It now uses
  the ADR-006 `_in_str` twin.
- ⚠ Git branch name from `.git/HEAD` — the guard exists
  (`safe_branch_name_in_str`, itself fixed in 1.9.8 for the same Str/cstring
  reason) but its only caller is `src/prompt.cyr`, which is **not compiled**.
  The live prompt renders no branch at all.
- ⚠ Usernames from `/etc/passwd` — `is_safe_username`'s only caller is
  `src/security.cyr`, **not compiled**.

## File Permissions

| File | Mode | Why |
|------|------|-----|
| `~/.agnsh_history` | 0600 | Contains command history — may reveal secrets |
| `~/.agnsh_audit.log` | 0600 | Forensic record — tampering breaks investigations |

As of 1.9.4 both are created **0600 at open** (not chmod'd afterwards — that
sequence was itself a race), opened `O_NOFOLLOW`, and the audit log's mode is
**re-asserted on every open** rather than only at creation: a log made under a
looser umask, restored from a backup, or copied into place used to keep whatever
mode it had, indefinitely.

⚠ **When `$HOME` is unset** these fall back to `/tmp/agnsh_history.<uid>` and
`/tmp/agnsh_audit.log.<uid>`. The uid qualifier (1.9.4 — they were previously
fixed, shared names) removes the collision between users on a multi-user host and
the pre-creation race in which another user creates the file first and thereby
owns your audit trail. **It does not make `/tmp` a safe home for an audit log**:
a same-uid process is unaffected, and the directory is still world-writable. Treat
the fallback as degraded operation for a broken environment, not a supported
configuration.
| `~/.agnoshi/checkpoints/` | *(n/a)* | ⚠ Never created — `checkpoint.cyr` is not compiled |
| `/usr/local/bin/agnsh` | 0755 | Binary — exec, not writable by users |

## What Can Still Go Wrong

**Things agnsh cannot prevent:**

- ⚠ **A caveat about `mode human`**: it does **not** hand the user a raw shell
  and does not disable classification. What it changes is that program launches
  gain a confirmation prompt (`mode_needs_confirm` covers `human` and `strict`).
  Every mode still classifies, still reports risk, and still audits.
- Alias expansion — ⚠ `src/aliases.cyr` is **not compiled**, so neither the risk
  nor the metacharacter mitigation described in earlier revisions of this guide
  exists today.
- **The natural-language path does not execute**, so its classification is
  advisory. A user who types a raw `run /abs/path`, or an AGNOS bareword, is
  gated by `is_safe_path` and the mode confirmation — not by the permission
  tier. Closing that is roadmap Bucket 1 slices 5 and 6.
- Race conditions between permission check and execution (TOCTOU) if the
  filesystem is mutated by another process. Full TOCTOU protection would
  require inode-locking at the kernel layer.

**Hardening items from the 2026-05-11 audit — current status** (that audit
deferred them "to v1.4.0"; both were in fact closed in the 1.9.x arc):

- **Symlink races on state-file open** — ✅ **CLOSED in 1.9.1.**
  `~/.agnsh_audit.log` and `~/.agnsh_history` are now opened with
  `O_NOFOLLOW`, and the history file is created 0600 **at open** rather
  than being chmod'd afterwards (the old create-then-chmod sequence was
  itself a race). Exploit before the fix was worse than originally
  described: `sys_chmod` followed the symlink too, so a pre-placed link
  meant a *write plus re-permission* of an arbitrary user-owned file.
  ⚠ **Correction to the value published here before 1.9.1.** This guide
  said `O_NOFOLLOW` "differs per arch — `0o400000` on x86_64,
  `0o100000` on aarch64-generic". That is wrong. `asm-generic/fcntl.h`
  defines `O_NOFOLLOW` as `(1 << 17)` = **131072**, and neither x86_64
  nor aarch64 overrides it (32-bit **arm** does — the likely source of
  the confusion). `0o100000` = 32768 is **`O_LARGEFILE`**: an
  implementer following the old text would have opened the audit log
  with `O_LARGEFILE` on the aarch64 release artifact and left the race
  fully intact there. There is **no per-arch split**; do not reintroduce
  one. On agnos the bit is dropped rather than miscompiled (`file_open`
  masks only the `AO_*` bits it knows), so the agnos side still needs a
  kernel `AO_NOFOLLOW` — tracked in the roadmap.
- **chmod-failure logging** — ✅ **superseded in 1.9.4.** The chmod is no longer
  the primary protection: a new history file is created **0600 at open**, and the
  audit log's mode is re-asserted on every open. A failed chmod now only matters
  for a file that already existed with looser permissions, and it still emits an
  operator-visible stderr warning in both cases.

## Forward Shield (v1.3.1)

The v1.3.1 P(-1) audit added `scripts/lint-cstr-str.sh`. It now covers **seven**
bug categories (A–G) — Category F (`strlen` inside an `_in_str` body) landed in
v1.3.3 and Category G (`str_data()` handed to a path-taking syscall) in 1.9.1.
The next free letter is **H**.

⚠ **Read the blind spot before trusting a green run**: categories A/B match only
a **literal** argument, so a cstring carried in a *variable* is invisible. That
is exactly how the dead SHELL_COMMAND classifier (1.9.1) and the
`audit_format_table` defect (1.9.8) both survived a clean lint.

The original five categories: Str-typed fns with cstring arg
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
