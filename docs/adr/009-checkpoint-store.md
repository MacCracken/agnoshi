# ADR-009: The Checkpoint Store (2.0.3)

**Status:** Accepted (2026-09-26, for 2.0.3)
**Serves:** [ADR-008](008-nl-exec-contract.md) — the approval-gated tiers it defers (roadmap 2.0.x) must
not run a REMOVE or MOVE until what it destroys can be put back.

## Context

`src/checkpoint.cyr` dates from the v1.0 port and had never been in a build: it called seven stdlib
helpers that no longer exist (`fs_copy`, `fs_mkdir_p`, …). The roadmap's 2.0.3 slot mapped them to
cyrius 6.6.6, but porting the calls would not have been enough:

- **Its index lived in memory.** Which backup belonged to which file lasted as long as the process,
  and 2.0.5's own test — `agnsh -c "remove foo/a"`, then `agnsh -c "undo"` — runs in two processes.
- **Backups were named by an in-process counter** that restarted at 0 in every process, so each run
  overwrote the last one's files.
- **It followed symlinks** both when saving and when restoring, and kept no permission bits.
- **It fell back to a fixed `/tmp/agnoshi-checkpoints`**, which any user could create first.

Two things were settled first:

- **Where the store lives** (ruled 2026-09-26): the roadmap said `$HOME/.agnoshi/checkpoints/`.
  ADR-008 had since put the report folder under XDG state, and XDG lists undo history as state data.
- **What "the last change" means**: one command may remove many files, and `undo` should put back
  the whole command, not its last file.

## Decision

### 1. A store on disk, in a private state folder

- **Location:** host `$XDG_STATE_HOME/agnoshi/checkpoints/`, by default
  `~/.local/state/agnoshi/checkpoints/`, beside the report folder; agnos `/.agnsh_checkpoints/`.
- **Folder rules:** the report folder's, now shared in `statepaths.cyr` (`state_dir_from`,
  `state_dir_prepare`). It is created 0700. On the host it must be a real directory, not a symlink,
  owned by the user and not writable by others. Without a usable `HOME` it falls back to a
  uid-qualified `/tmp` folder, under the same checks.

### 2. One file per entry, named to sort

- **Names:** `<stamp>-<pid>-<group>-<index>.ckpt`, e.g.
  `20260926T193051.123456789Z-0000073264-000001-0001.ckpt`.
- **Fixed width:** every field is fixed-width, so the names sort in the order they were made and the
  store needs no index.
- **Contents:** each entry is a text header, a blank line, then the saved file's bytes:

  ```
  agnsh-checkpoint 1
  op: remove | move
  path: /abs/path        the file removed, or the source of a move
  to: /abs/path          a move's destination
  mode: 644              when a file was saved
  size: 1234             bytes after the blank line
  time: 2026-09-26T19:30:51Z
  ```

- **Paths:** stored absolute, joined to the working directory where given relative. A path that
  cannot sit on one line — one with a control byte — is refused.
- **Creation:** each entry is created exclusively, 0600 and never through a symlink, and removed
  again if its write fails, so a half-written entry never stays.
- **Reading back:** a file that does not parse as an entry is never restored or pruned.

### 3. A group is one command

- **Grouping:** every entry a command makes shares a group (`ckpt_group_new`, `ckpt_remove`,
  `ckpt_move`).
- **Undo:** `ckpt_undo` restores the newest group, newest entry first.
- **Pruning:** the store keeps the newest 100 groups. Each new group prunes once, before its first
  entry, so a command that saves many files never lists the folder per file and can never prune
  itself.

### 4. What is saved, and what is refused

- **Saved:** a regular file of at most 64 MiB — one about to be removed, or one a move is about to
  overwrite.
- **Refused, with a reason:** a directory, a symlink, a device or FIFO, a larger file, or one that
  cannot be read. The caller decides whether its command runs anyway.
- **Opening the source:** on the host the file is opened read-only, non-blocking and without
  following a symlink, then re-checked on the descriptor, so a file swapped for a FIFO after the check
  cannot hang the shell.
- **A move needs no copy:** undo renames it back. Its destination follows `mv`: into a directory
  when there is one, following a symlink to it.

### 5. A restore never overwrites

- **Only into a free path:** a removed file is recreated only where nothing exists now. A move is
  undone only if its destination is still there and its source is free.
- **How files are recreated:** exclusively and never through a symlink, with the saved permission
  bits minus setuid, setgid and sticky.
- **On failure:** the entry stays, with its reason, so the user can clear the path and undo again.

### 6. agnos

- **Built, not run:** the store compiles on agnos, where its `stat` calls take the path length and
  there are no permission bits and no `fchmod`. It does not run there until approval-gated exec
  calls it.
- **Relative paths:** agnos has no working directory yet (roadmap 2.1.x), so a relative path is
  taken from `/`.

## Consequences

- **New state on disk:** it is bounded — 100 groups, 64 MiB per file — and private.
- **No caller in 2.0.3.** Approval-gated exec (roadmap 2.0.x) checkpoints before every REMOVE and
  MOVE, and the `undo` builtin calls `ckpt_undo`. The store's agnos arm is first exercised there.
- **Directories are not recoverable.** Before an `rm -r` can be approved, approval-gated exec must
  decide what a refused checkpoint means: refuse the command, or say that undo will not restore it.
- **Order is the wall clock's.**
  - agnos's clock has whole-second resolution, so two groups made in the same second by different
    processes sort by pid.
  - A clock set back sorts new groups before old ones, so undo could pick an older group.
- **Cost:** one command's checkpoint of a 4 KiB file against a full store costs ~150 µs
  (`checkpoint/remove_4k`). Listing and sorting the folder is most of it; an insertion sort, first
  written, made it ~386 µs.

## Alternatives considered

- **Port the in-memory manager.** It cannot undo across processes. Rejected.
- **`$HOME/.agnoshi/checkpoints/`** (the roadmap, the v1.0 design). A second convention beside the
  XDG report folder. Rejected by ruling.
- **One journal file for all entries.** agnos ignores `O_APPEND`, and one torn write loses the whole
  history; a file per entry is created, and removed, whole. Rejected.
- **Hard links instead of copies.** They are cheap, but they fail across filesystems, and an in-place
  overwrite (`>`) changes the "saved" content too. Rejected.
- **Save directories recursively.** Unbounded size, symlinks and special files inside, and a partial
  copy to reason about. Deferred: a refusal with a reason is safer now.

## References

- `src/checkpoint.cyr` — the store; `src/statepaths.cyr` — `state_dir_from`, `state_dir_prepare`
- `tests/test_core.tcyr` — `_t_checkpoint`; `tests/bench_core.bcyr` — `checkpoint/remove_4k`
- [ADR-008](008-nl-exec-contract.md) § 2 — the report folder this store sits beside
- [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/latest/) —
  `$XDG_STATE_HOME`, whose examples include "undo history"
