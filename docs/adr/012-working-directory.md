# ADR-012: The Working Directory — `cd` and `pwd` (2.2.0)

**Status:** Accepted (2026-11-02, for 2.2.0)

## Context

agnsh had no `cd` and no `pwd`. A typed `cd /tmp` fell through to the natural-language parser, which
printed `Intent: 4  Command: cd`, and then a hint that `cd` "arrives in a later release". Nothing
changed directory: `cd` in a child process changes only the child.

The two targets differ at the root:

- **Linux host:** the kernel keeps a working directory per process. `chdir(2)` and `getcwd(2)` do
  the work.
- **agnos:** there is no cwd syscall, and none is to be added (agnos userland ABI § 3.2): "CWD is
  userland-owned — agnsh tracks its own CWD and passes absolute paths to every syscall." A relative
  name is rooted at `/`.

The roadmap asked one question before anything was promised on agnos: do kriya and owl resolve
relative operands against `$PWD`? At 2.2.0 they do not.
- kriya reads `$PWD` only for its own `pwd -L`.
- owl requires an absolute path on agnos.

## Decision

### 1. `cd` and `pwd` are builtins (`src/cwd.cyr`)

- **`cd [DIR]`:**
  - no argument, or `~`, goes to `$HOME`;
  - `~/x` goes to `$HOME/x`;
  - `-` goes to the directory before the last `cd`, and prints it;
  - anything else is that directory, relative to the working directory unless it starts with `/`.
  - Two arguments are refused. A failure exits 1 and says why on stderr: no such directory, not a
    directory, permission denied.
- **`pwd`** prints the working directory.
- **Where:** both work in the interactive shell and under `-c`, where `cd DIR` only says whether DIR
  can be entered, as `sh -c` does. Both are in `is_builtin` and the help.
- **The natural-language CHANGE_DIR** (`switch to /tmp`) runs the same builtin, recorded as
  `executed` or `failed`. It used to be reported and not run.

### 2. On the host, the kernel's directory

- `cd` is `chdir(2)`, and `pwd` is `getcwd(2)`.
- Once agnsh has changed directory, a child it launches gets `PWD` and `OLDPWD` as they are now. Its
  environment otherwise comes from agnsh's start (`env_block`), whose `PWD` would be stale.
- Before any `cd`, the environment passes through as it was, so a logical `PWD` through a symlink
  survives.

### 3. On agnos, agnsh's own

- **The working directory** is kept in `cwd.cyr`. It is seeded from `$PWD` (kybernet's seed is `/`)
  and normalized; `.` and `..` are resolved, never above `/`.
- **`cd` checks that its target is a directory** (`is_dir`) before moving.
- **agnsh resolves its own relative paths against it**, so it hands every syscall and program an
  absolute path:
  - a `cd` target;
  - a `>` target, resolved before the audit-log and history guards compare it;
  - every path argument of a natural-language translation (`path_arg_cstr`), so the program and a
    checkpoint of the same line see the same file;
  - an `ls`, `find` or `grep -r` with no path, which gets the working directory, not `.`;
  - a checkpoint's relative path (`ckpt_abs`, which used to root it at `/`).
- **Children get `PWD` and `OLDPWD`** (`sh_build_env_blob`). The inherited ones are dropped and the
  tracked ones passed instead, within the kernel's 16-entry, 1,024-byte cap.

### 4. A program's own operands are the program's

agnsh does not rewrite the arguments of a typed program line. It cannot tell `grep pattern` from
`rm file`. So on agnos, `cd /tmp` then a typed `ls notes` still lists `/notes`, until kriya (and
owl) resolve relative operands against `$PWD` there. That is § Gated in the roadmap, and agnsh's
exported `PWD` is what they would read.

## Consequences

- **+** The shell has a working directory on both targets, and natural language can move it.
- **+** On agnos, every path agnsh itself resolves is absolute and consistent: a natural-language
  `remove notes.txt` checkpoints and removes the same file.
- **−** On agnos, typed program lines do not follow `cd` until kriya and owl read `$PWD`.
- **−** The host's `pwd` is physical (getcwd), not the logical path a shell keeps through a symlink.
  The `PWD` children get after a `cd` is physical too.
- **Neutral:** `go to DIR` typed into the shell runs a program named `go`, when there is one on
  `$PATH` (ADR-007: a first word that names a program is a shell line). `switch to DIR` and `cd DIR`
  do not.

## Alternatives considered

- **Rewrite relative-looking arguments of kriya verbs on agnos.** Rejected: agnsh cannot tell a path
  from a pattern or a message, and a wrong guess would silently change what a command does.
- **Ask agnos for a cwd syscall.** Rejected by agnos's ABI: the cwd is userland's.
- **Adopt `session.cyr`'s `cd`.** Rejected: it treats its arguments as `Str` where the shell's are
  cstrings, and the module needs UI bodies that do not exist (roadmap note).

## References

- `src/cwd.cyr`; `src/translate.cyr` (`path_arg_cstr`); `src/nlexec.cyr` (`nl_cd`);
  `src/run_host.cyr` (`host_environ_envp`); `src/run_agnos.cyr` (`sh_build_env_blob`, the `>` target);
  `src/checkpoint.cyr` (`ckpt_abs`).
- agnos `docs/development/agnos-userland-abi.md` § 3.2 (the userland-owned cwd); kriya
  `src/cmd/pwd.cyr`; owl `src/vcs.cyr` (`_vcs_abspath`).
- Tests: `tests/test_core.tcyr` (`_t_cwd`), `scripts/smoke-test.sh` (the cd/pwd block),
  `scripts/agnos-qemu-test.py` § 1c.
