# ADR-011: Approval-Gated Execution (2.1.3)

**Status:** Accepted (2026-11-02, for 2.1.3)
**Builds on:** [ADR-008](008-nl-exec-contract.md), whose § 1 deferred these tiers, and
[ADR-009](009-checkpoint-store.md), the store this ADR gives its first caller.

## Context

Through 2.1.2, a natural-language line in the USER_WRITE, SYSTEM_WRITE or ADMIN tier stopped at
`Approval required -- not executed`. The pieces to run one were compiled in, with no caller:

- **`ApprovalManager_request`**, the prompt. Its only caller was `session.cyr`, which is outside the
  binary. It also had four faults:
  - it wrote to stdout;
  - it read fd 0 past `read_line`'s carry buffer;
  - it reported EOF as a timeout;
  - it offered a `[m]odify` that nothing implemented.
- **The checkpoint store** (ADR-009).
- **`execute_with_privileges`**, which ran `sudo -n` through `execute_command` → lib `exec_vec`. That
  is an empty environment and a second host launcher, which ADR-008 § 3 rules out.

Four questions were ruled first (2026-11-02):

1. NL `remove` was BLOCKED whether recursive or not, so a checkpoint before a remove would never have
   run. **A plain remove is approvable; a recursive one stays BLOCKED.**
2. **A destructive command whose files cannot be checkpointed is refused.** It is not run with a
   warning.
3. **ADMIN runs through `sudo -n` after the approval** on the host.
4. **A typed `reboot` / `poweroff` / `halt` keeps no confirmation** (agnos). This closes the roadmap's
   open decision on power verbs: they are the user's own lines, not the AI's.

## Decision

### 1. Which lines ask

- **USER_WRITE:**
  - `mkdir`, `cp`, `mv`;
  - the git verbs: commit, branch, push, pull, checkout, merge, stash;
  - since this release, a plain `rm`. A recursive `rm` stays BLOCKED: the store saves files, not
    directories.
- **SYSTEM_WRITE:** no translator produces it today.
- **ADMIN:** `kill`, `apt`, `ufw`, `useradd`, `userdel`, `passwd`, `groupadd`, `systemctl`.
- **These never ask, because they never run:**
  - BLOCKED;
  - the SHELL_COMMAND fall-through, which is not a program;
  - a translation routed to an MCP tool;
  - every line in a restricted session — root on a Linux host runs no NL line (ADR-008 § 1, 2.0.2).

### 2. The prompt

- **Every mode asks:** the AI proposes, and the human approves. The prompt stands in for the mode
  gate, so `human` and `strict` ask once, not twice.
- **Where it goes:** stderr, as every confirmation since 2.0.0:
  `  ⚠ [MED] approve: <line> -- checkpointed first ? [y/N]`. The note appears only when something was
  checkpointed, and an ADMIN line shows its `sudo -n` prefix.
- **The answer** is one line through `read_line`. `y` or `Y` approves; anything else, including a
  bare Enter, declines.
  - EOF is **no answer**, a decline that says so.
  - On the host, no line within the manager's `timeout_seconds` (300) is **timed out**.
  - agnos has no poll, so there the prompt waits as long as the read does.

### 3. The order: escalate, checkpoint, ask, run

1. **ADMIN on the host** becomes `<sudo> -n <line>` (`security_sudo_line`). sudo is re-verified as
   root-owned at this moment (`verify_sudo_path`), and `-n` makes it fail rather than ask for a
   password that agnsh could not show.
   - The line runs through the one host launcher (`host_exec`), with agnsh's environment.
   - `execute_command` and `execute_with_privileges` are deleted.
   - agnos has no sudo and no uid model, so an approved ADMIN line there runs as agnsh does.
2. **A REMOVE or MOVE is checkpointed before the prompt**, so nobody is asked about a line that will
   not run.
   - A remove saves each file it names (`ckpt_remove`).
   - A move records the move, and saves a file it would overwrite (`ckpt_move`).
   - Anything the store refuses refuses the command, unasked: a directory, a symlink, a device or
     FIFO, a file over 64 MiB, an unreadable file, or an unusable store.
3. **The prompt** (§ 2).
4. **An `approved` record**, then the launch (`sh_launch_ungated`) with its own `launched` and
   outcome records.

A command that is declined, or that runs and exits non-zero, leaves no checkpoint behind
(`ckpt_group_drop`). Otherwise `undo` would try to restore a file that was never removed.

### 4. `-c` and `--dry-run`

- **`-c`:** the prompt is on stderr, and the answer is read from stdin. With nothing on stdin
  (`< /dev/null`, a closed pipe) the line is not run: **126**, `approval required, no answer`. So a
  script that never fed stdin keeps the status it had.
- **`--dry-run`** gives no approval, so a gated line stays **126**, as in 2.1.2. Only a line that would
  run without asking is 0.

### 5. Audit

- **The parse-time record** stays `needs_approval`.
- **The exec records:**
  - `approved`, then `launched`, then the outcome (`executed`, `failed` or `error`);
  - `denied`, for a decline, no answer, a file that cannot be checkpointed, or a sudo that cannot be
    used;
  - `timed_out`, for no answer in time.
- `approved` and `timed_out` are new: the exec vocabulary goes from five labels to seven. `approved`
  sets the record's approved flag to 1, and `timed_out` and `denied` set it to 0. The two vocabularies
  stay disjoint.

### 6. Exit status (ADR-008 § 4)

| outcome | status |
|---|---|
| approved and run | the program's status |
| declined, no answer, timed out, cannot checkpoint, cannot escalate | **126** |
| the program cannot be found or launched | **127** |

## Consequences

- **+** The premise holds: a natural-language line that changes something runs, with a human's yes.
- **+** Nothing destructive runs without its checkpoint, and nothing is asked about that would then
  be refused.
- **+** One host launcher again: the sudo path no longer has its own.
- **−** `undo` is the next slot (roadmap 2.1.x — `undo`). Until then a checkpoint exists, but no
  command restores it; `ckpt_undo` is compiled in, uncalled. *(2.1.4: the `undo` builtin,
  `nl_undo`, restores the newest group.)*
- **−** `cp` onto an existing file is not checkpointed. The store has no operation for an overwrite
  that leaves a new file in place.
- **−** A sudo that needs a password fails under `-n`, and the outcome record says `failed` with
  sudo's status. agnsh never collects a password.
- **−** An approved ADMIN line on agnos runs without privilege separation. agnos has no uid model to
  separate with.

## Alternatives considered

- **Ask only in `human` and `strict`.** Rejected: `auto` would run an AI-proposed `rm` unasked.
- **Ask, then checkpoint.** Rejected: a file that cannot be checkpointed would be refused after the
  user's yes.
- **Keep the mode gate too.** Rejected: `human` would ask twice for one line.
- **Keep `[m]odify`.** Dropped: nothing implemented it, and editing a proposal is a line editor's job
  (roadmap 2.2.x).

## References

- `src/nlexec.cyr` (`nl_launch_gated`, `nl_checkpoint`), `src/approval.cyr`
  (`ApprovalManager_request`), `src/security.cyr` (`security_sudo_line`), `src/checkpoint.cyr`
  (`ckpt_group_drop`, `ckpt_dir_override_set`), `src/run_agnos.cyr` (`sh_launch_ungated`),
  `src/translate.cyr` (`translate_remove`).
- Tests: `tests/test_core.tcyr` (`_t_gated`), `scripts/smoke-test.sh` (the approved and declined
  removes, the refused directory), `scripts/agnos-qemu-test.py` (§ 1b, the store's first run on
  agnos).
