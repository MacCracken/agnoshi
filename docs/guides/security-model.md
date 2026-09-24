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
narrow, gap. Tracked for the approval wire-up (roadmap 2.0.x — approval-gated exec).

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

⛔ **On agnos, none of that held before 1.9.14.** agnos ignores `AO_APPEND`
(`ext2_open` starts every file at position 0), and the writer relied on it, so
each record was written at offset 0 over the previous one. Measured in QEMU on
agnos 1.57.5 with `scripts/agnos-qemu-test.py`: a session that wrote six
records left one complete record on disk — the last — plus the torn tails of
longer ones. The writer now seeks to the end on agnos before writing, and refuses to
write if the seek fails (a lost record is recoverable; one written over the log
is not). Linux hosts were never affected — `O_APPEND` is honoured there.

### 6. Checkpoint / Undo — ⚠ NOT IN THE SHIPPED BINARY

`src/checkpoint.cyr` implements backup-before-destructive-op, `undo` and a
100-entry auto-prune, but it is **not in the binary's include graph**, there is
no `undo` builtin, and no `~/.agnoshi/checkpoints/` directory is ever created.
**Do not rely on any rollback guarantee.** It calls seven stdlib helpers that
no longer exist, so its wire-up is a re-implementation (roadmap 2.0.x — checkpointing).

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
mode it had, indefinitely. The history save repairs an existing file's mode the
same way. Since 1.9.15 both repairs go through the **open descriptor**
(`fchmod`): a path chmod issued after the `O_NOFOLLOW` open followed a symlink
swapped in between the two calls, re-permissioning whatever it pointed at.
agnos has no permission bits, so there is nothing to repair there.

⚠ **The audit log's path is overridable in-process, by design.**
`audit_path_override_set` (`src/statepaths.cyr`, 1.9.10) redirects it. It exists
because the audit writers resolve their own destination — so before it, a test
had no file to read back and the six functions that write the security log had
**no assertion of any kind**. They now have assertions on record *content*.

The reason this is not a hole, stated rather than assumed: **nothing in the shell
calls the setter.** Cyrius has no dynamic dispatch, no reflection and no
call-by-name, so a function with no call site is unreachable from any
input-driven path — an attacker with control of a command line cannot get to it.
`scripts/lint-cstr-str.sh` **Category H** fails the build if any file under
`src/` other than the definition site so much as names it, which is what stops a
future edit from quietly turning a test hook into a runtime redirect. The
override is per-process and cannot persist between runs.

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
- **Typed shell lines skip the permission tiers.** Since 2.0.0 the
  natural-language path executes (SAFE and READ_ONLY only), so its
  classification is enforced. A user who types a program's name or
  `run /abs/path` is gated by `is_safe_path`, the mode confirmation, and — for a
  line the classifier calls BLOCKED — a confirmation in every mode (ADR-008). The
  other tiers do not gate typed lines: doing so before approval exists would
  stop agnos's kriya file verbs. Approval-gated exec is roadmap 2.0.x.
- Race conditions between permission check and execution (TOCTOU) if the
  filesystem is mutated by another process. Full TOCTOU protection would
  require inode-locking at the kernel layer.

**Hardening items from the 2026-05-11 audit — current status** (that audit
deferred them "to v1.4.0"; both were in fact closed in the 1.9.x arc):

- **Symlink races on state-file open** — ✅ **CLOSED on x86_64 in 1.9.1;
  on aarch64 and agnos only in 1.9.13.**
  `~/.agnsh_audit.log` and `~/.agnsh_history` are opened with
  `O_NOFOLLOW`, and the history file is created 0600 **at open** rather
  than being chmod'd afterwards (the old create-then-chmod sequence was
  itself a race). Exploit before the fix was worse than originally
  described: `sys_chmod` followed the symlink too, so a pre-placed link
  meant a *write plus re-permission* of an arbitrary user-owned file —
  and for the history file, which opens `O_TRUNC`, a lossy **rewrite** of it.
  ⛔ **The 1.9.1 "correction" that stood here was itself wrong, and it
  shipped.** Before 1.9.1 this guide said `O_NOFOLLOW` "differs per arch —
  `0o400000` on x86_64, `0o100000` on aarch64". That was right: x86_64 uses
  the `asm-generic/fcntl.h` set, and arm64 overrides four flags in
  `arch/arm64/include/uapi/asm/fcntl.h`, where `O_NOFOLLOW` is `0o100000`
  (32768) and `0o400000` (131072) is `O_LARGEFILE`. 1.9.1 replaced that
  with "neither x86_64 nor aarch64 overrides it", pinned 131072 for both,
  and said not to reintroduce a per-arch split — so every aarch64 release
  from 1.9.1 to 1.9.12 opened both state files with `O_LARGEFILE` and **no**
  `O_NOFOLLOW`, and the race stayed fully open there. Reproduced under
  `qemu-aarch64` in 1.9.13 (a planted symlink was appended to, truncated
  and chmod'd 0600) and fixed by taking the stdlib's per-target
  `O_NOFOLLOW` (cyrius ≥ 6.6.4). CI now runs both test suites on aarch64
  under qemu-user, including a planted-symlink test. On **agnos**,
  `file_open` maps the bit to `AO_NOFOLLOW` since cyrius 6.6.4, which the
  kernel honours from agnos 1.56.53; before that it was dropped. The last
  write path without it, the `>` redirect target's raw open in
  `run_agnos.cyr`, gained `AO_NOFOLLOW` in 1.9.14: in QEMU on agnos 1.57.5,
  `echo pwned > /redir-link` wrote through a planted symlink into its target
  before the fix and is refused (`cannot open redirect target`) after it.
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
