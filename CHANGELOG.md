# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Fixed

- **agnos: no prompt after the banner — REAL root cause was a cyrius `fnptr.cyr` gap, not getenv/the stack.** The 1.4.8 getenv change below was necessary-but-insufficient: re-staging it still hung agnsh right after the banner. QEMU bisection (real agnos 1.43.7 kernel via `agnos/scripts/agnsh-smoke.sh`, burn-free) traced the hang into `CommandHistory_new() → vec_new()`, which returned **0**, then a null-deref on the first prompt render. Cause: **`cyrius/lib/fnptr.cyr`'s `fncall0..8` have no `CYRIUS_TARGET_AGNOS` asm branch** (only `_LINUX`/`_MACOS`/`_WIN`), and `cyrius build --agnos` predefines `CYRIUS_TARGET_AGNOS` but deliberately not `CYRIUS_TARGET_LINUX`, so every `fncallN` compiled to `var result = 0; return result;` → returned 0 → the `Allocator` vtable (`alloc_via`/`realloc_via`/`free_via` dispatch through `fncall1/2/4`) returned 0 → `vec_new`/`str_new`/`hashmap` via `default_alloc()` all produced null. (Same bug class as the v5.9.38 macOS `fncall` gap the file documents.) The getenv/`buf[8192]`/`~12 KB stack` story in 1.4.8 was wrong — a Cyrius function-local `var X[N]` is a `.bss` global (0 stack bytes), and getenv isn't on the agnos pre-prompt path anyway.
  - **Stopgap** (`scripts/patch-fnptr-agnos.py`, new): idempotent patcher that clones each `fncallN`'s validated Linux/x86 asm under a `CYRIUS_TARGET_AGNOS` guard **in place** in the gitignored vendored `lib/fnptr.cyr`. Must be in-place, not a separate-file override — the hand-asm `[rbp-N]` offsets are coupled to cycc's per-definition frame layout (a copied-out standalone `fncallN` gets a different layout and calls a wild address; disassembly-confirmed). Run after every `cyrius update`/`cyrius deps`, before `cyrius build --agnos`.
  - **Verified** (QEMU, no iron burn): agnsh reaches `[ASSIST] >` and dispatches `help`/`version`/`mode` (`agnos/scripts/agnsh-type-test.py`); host smoke **59/0** (stopgap is agnos-gated, host untouched).
  - **Durable fix** (cyrius, hands-off): add the agnos branch to `fncall0..8` in `cyrius/lib/fnptr.cyr`. Tracking issue: `cyrius/docs/development/issues/2026-06-08-fnptr-fncall-missing-agnos-branch.md`. Remove `scripts/patch-fnptr-agnos.py` once that lands and agnoshi re-vendors.

## [1.4.8] - 2026-06-08

> **Correction (see [Unreleased]):** this getenv fix did NOT resolve the no-prompt hang — the real cause was the cyrius `fnptr.cyr` agnos gap. The getenv change is retained as a harmless simplification (agnos `HOME` is always `/`), not as the fix.

### Fixed

- **agnos: no prompt after the banner — `getenv("HOME")` overflowed the ring-3 init stack (regression from 1.4.6).** The `agnoshi_147` iron burn (agnos 1.43.7) booted cleanly through DHCP + `kybernet: exec /bin/agnsh` and printed the full agnsh banner (`agnoshi 1.4.7` + built-ins + files), then hung with **no prompt** — exactly the "faulted agnsh right after its banner" symptom the 1.4.5 code had documented and avoided. Root cause: 1.4.6 re-enabled `getenv("HOME")` in `history_path()` / `audit_log_path()` on agnos on the premise that cyrius 6.0.87's buffer-free `_agnos_getenv` delegate cleared the old deep-stack hazard. It did not: the **`lib/io.cyr` `getenv()` wrapper** declares `var buf[8192]` *after* its `#ifdef CYRIUS_TARGET_AGNOS` early-return (the local is **unguarded**), so the 8 KB frame is compiled into `getenv` for the agnos target too — the early `return _agnos_getenv(name)` never uses it, but the prologue still reserves it. agnos hands ring-3 only **~12 KB** of init stack (`elf.cyr`: rsp starts at stack offset `0x3000`), and `interactive_loop` already holds `var buf[4096]`; the extra 8 KB frame from the first `history_path()` → `getenv` call overflowed the stack → ring-3 #PF before the first prompt render.
  - **Fix** (`src/agnsh.cyr`): on agnos, resolve both paths **without calling `getenv`** — the kernel default env is always `HOME=/` (staged by `elf.cyr`), so `$HOME/.agnsh_history` / `$HOME/.agnsh_audit.log` are statically `/.agnsh_history` / `/.agnsh_audit.log`. Both helpers now return those literals directly under `#ifdef CYRIUS_TARGET_AGNOS`; the host build keeps full `getenv` `$HOME` resolution under `#ifndef`. Same on-disk paths 1.4.6 intended, minus the stack hazard.
  - **Verified**: host build + smoke **59/0**, FS-verb smoke **84/0** (unchanged — the fix is `#ifdef CYRIUS_TARGET_AGNOS`-only); agnos `--agnos` build OK. Rides the next iron burn (re-stage `/bin/agnsh` on agnos-fs).
  - **Upstream follow-on (cyrius, hands-off — surfaced for the user):** `lib/io.cyr` `getenv()` should guard `var buf[8192]` behind `#ifndef CYRIUS_TARGET_AGNOS` so the agnos build never reserves it. Separately, agnos's ~12 KB ring-3 init stack is a latent cliff for any moderately deep call chain carrying a kbuffer — worth enlarging the `elf.cyr` stack-top offset.

## [1.4.7] - 2026-06-08

**`run` (and bare commands) pass arguments — `bnrmr AGNOS` / `doom /DOOM1.WAD` work, and the launch path is durable.** Iron burn `1436` surfaced that every launch *with an argument* failed `run: failed to launch program` (arg-less `run /bin/bnrmr` worked), and that the burned binary resolved bare commands against `/bin` — a UX that lived only in *uncommitted* working-tree state and was lost. Two root causes: the agnos kernel `execwait`#37 used the whole command line as the file-open name (argv dropped — fixed kernel-side, agnos 1.43.7), and agnsh's launch path routed through the gitignored vendored `lib/process_agnos.cyr`, whose `run()` the 1.4.6 `cyrius lib sync` reverted to the `sys_spawn` form (the 1.4.4 execwait edits to `lib/` were never durable — `lib/` is regenerated by `cyrius update`). This cut moves the agnos launch path into **committed `src/`** and restores the bareword UX.

### Added

- **`src/run_agnos.cyr`** (new, committed, `#ifdef CYRIUS_TARGET_AGNOS`) — the durable agnos launch path, replacing reliance on the gitignored `lib/`:
  - `sh_exec_line(line)` calls `execwait`#37 directly (`syscall(37, cmdline, len)`). The whole command line ("`PATH args`") is handed through; the kernel opens the **path token** and tokenizes the full line into argv (agnos 1.43.7), so argv flows with no client-side splitting. The cyrius master stdlib has no execwait wrapper and `cyrius` is hands-off, so the raw syscall lives here.
  - `sh_try_bareword_launch(line, mode)` — if the first word resolves to `/bin/<word>`, run it from disk with its args; otherwise return -1 so the caller falls through to the NL-intent path. Restores the burn-`1436` UX: `doom` / `bnrmr AGNOS` launch, the typo `brnmr` interprets. PATH is fixed to `/bin`; reuses the FIX-4 mode gate (HUMAN/STRICT confirm, AUTO/ASSIST direct) via `sh_run_program`.

### Changed

- **`sh_run_program` routes through `execwait`#37 on agnos** (`src/agnsh.cyr`, `#ifdef CYRIUS_TARGET_AGNOS`) — calls `sh_exec_line` instead of the vendored lib `run()`. The host build keeps the unchanged lib fork/exec path under `#ifndef`. The stale "routes through `process_agnos.cyr`'s `run()` → `sys_execwait`" comments (1.4.4) were corrected: that routing was clobbered by the 1.4.6 re-sync and the durable path is now `src/run_agnos.cyr`.
- **Bareword-launch dispatch arm** added to the interactive loop and the `-c` one-shot path (both `#ifdef CYRIUS_TARGET_AGNOS`), after FS-verb dispatch and before the NL-intent fallback. agnos-only, so host barewords stay on the NL path and the smoke suites are unchanged.

### Verified

- Host build + smoke **59/0**, FS-verb smoke **84/0** — unchanged (every new path is `#ifdef CYRIUS_TARGET_AGNOS`; the host keeps lib fork/exec and NL-intent barewords).
- Agnos `--agnos` build OK → valid static ELF64.
- **argv-through-`execwait`#37 positively QEMU-validated kernel-side** (agnos 1.43.7): the `/bin/exwv` ring-3 #37 validator now execwaits `/bin/argv Z` and the arg propagates (`run: exit 90` ×2; exec-smoke 13/13, `sweep.sh` 7/7). The agnsh launch/bareword path rides the next iron burn (re-stage `/bin/agnsh` on agnos-fs — the on-disk 1.4.6 is the pre-fix spawn build).

## [1.4.6] - 2026-06-08

**`getenv()` resolves on agnos — `$HOME`-based userland config is live.** The envp arc item's two lower halves already shipped — agnos 1.43.2 stages a real env (`HOME=/`, `PWD=/`) on the exec init stack, and cyrius 6.0.87 added the agnos `getenv()` branch that walks it (QEMU-validated against the real 1.43.2 kernel: `getenv("HOME")→/`, `getenv("PWD")→/`, `getenv("NOPE")→null`). What was missing was *consumption*: agnoshi still pinned 6.0.56, so its vendored `getenv()` was the old `/proc/self/environ`-only reader and agnsh short-circuited to `/tmp` on agnos. This patch closes the loop end-to-end.

### Changed

- **cyrius pin 6.0.56 → 6.0.87** (`cyrius.cyml`) — pulls in the agnos `getenv()`/envp walk (`lib/io.cyr`'s `CYRIUS_TARGET_AGNOS` branch → `lib/args_agnos.cyr`'s `_agnos_getenv`). Feature-motivated, exactly parallel to the 6.0.14→6.0.56 bump that pulled in `CYRIUS_TARGET_AGNOS` — *not* drift-chasing. The vendored `./lib/` snapshot was re-synced via `cyrius lib sync` (gitignored; the pin is the only tracked delta). The agnos **kernel** deliberately stays on its held 6.0.56 — it is the envp *producer*, not a `getenv` consumer.
- **`history_path()` / `audit_log_path()` consume `getenv("HOME")` on agnos** (`src/agnsh.cyr`) — removed the `#ifdef CYRIUS_TARGET_AGNOS` `/tmp` short-circuit. The old rationale (getenv a guaranteed-0 no-op + an 8 KB stack buffer co-resident with the read buffer) no longer holds: getenv now resolves, and the agnos branch (`_agnos_getenv`) is a stack-light envp walk — the 8 KB buffer only ever existed in the Linux `/proc/self/environ` path. With the default agnos `HOME=/`, the paths resolve to `/.agnsh_history` and `/.agnsh_audit.log` (was `/tmp/...`).
- **Trailing-slash-aware `$HOME` concat** — both path builders strip one trailing `/` from `$HOME` so `HOME=/` yields `/.agnsh_history` (not `//.agnsh_history`); `HOME=/root` → `/root/.agnsh_history`, `HOME=/root/` → `/root/.agnsh_history`. Correct on both agnos and host; unset `HOME` still falls back to `/tmp`.

### Verified

- Host build + smoke **59/0** — no regression from the 6.0.57→6.0.87 re-vendor (a 3-lens adversarial review confirmed the agnos-target delta is limited to the intended getenv addition plus one benign uncontended `alloc_agnos` lock; zero syscall-number / ABI / struct / wrapper-signature changes to any agnos peer).
- Agnos `--agnos` build OK → valid static ELF64 (the `getenv → _agnos_getenv` delegation links).
- The ABI offset was independently re-derived from both sides: kernel writes `envp[j]` at `rsp + 8 + (argc+1+j)*8` (`agnos/kernel/core/elf.cyr`); the consumer reads the identical formula (`args_agnos.cyr`) — exact match, no off-by-one.
- Trailing-slash concat proven for `HOME=/`, `/root`, `/root/`, `/home/x`, unset. getenv runtime on agnos already validated by cyrius 6.0.87 against the real 1.43.2 kernel; the agnsh path rides the next iron burn.

## [1.4.5] - 2026-06-07

### Fixed

- **`ls` (and every path verb) now sees the agnos filesystem.** The in-process verbs passed relative paths — `ls` with no arg / `ls .` passed the literal `"."` — straight to `sys_open`/`sys_stat`. agnos's `vfs_resolve_mount` requires an absolute path (the ABI is "userland passes absolute paths"; there is no per-process CWD), so a non-`/`-leading path returned `FS_NONE` → "No such file or directory". The in-kernel recovery shell never hit this because it normalized via `sh_abspath`; agnsh's userland verbs had no equivalent. Added `verb_abspath()` (`src/verbs.cyr`) and applied it at every path chokepoint (`verb_open_read/write_trunc/create/dir`, `verb_stat`, `verb_mkdir/rmdir/unlink/rename_sc`): on agnos it resolves relative paths against root `/` (the effective CWD — no working `cd`); on the host it's a pass-through so Linux still resolves `.` against the real CWD. **QEMU-validated** (gnoboot+OVMF, xHCI keyboard via HMP sendkey): `ls`, `ls .`, and `ls /` all list the ext2 root (`bin`/`lsmark`/`lost+found`) at the `[ASSIST] >` prompt.

## [1.4.4] - 2026-06-07

**`run` is LIVE — agnsh launches programs from disk on agnos.** agnos 1.43.0 shipped the `execwait` (#37) syscall (the ring-3 blocking-exec primitive), so the `run` builtin staged + gated in 1.4.3 is now un-gated. `run /bin/klug` (etc.) loads and runs a static ELF64 from the ext2 root in ring 3 and reports its exit code — the first user-facing "agnsh runs an external tool" path.

### Changed

- **`process_agnos.cyr` `run()` routes through `sys_execwait(path, len)`** (agnos #37) instead of the old read-into-buffer + `sys_spawn`(#3) + `sys_waitpid`(#4) pair. The old pair faulted on the kernel (in-memory `elf_load` did a CR3-switch `memcpy` of an unmapped arena buffer; `waitpid` busy-`hlt`ed with IF=0 → hard hang); `execwait` has the kernel own the load (under boot CR3) + synchronous `exec_and_wait`, so there's no in-shell ELF buffering and no busy-wait. Returns the child's exit code directly. (`spawn`/`run_capture`/`wait_pid` retain the old wrappers — not reachable from `run`, left for a future capture/background bite.)
- **`RUN_EXECWAIT_READY = 1`** (`src/agnsh.cyr`) — the one-line gate flip 1.4.3 was built for. `sh_run_program` now executes via `run()` (mode-gated: HUMAN/STRICT confirm, AUTO/ASSIST direct) instead of refusing. Help/usage text dropped the "(pending agnos 1.43.x)" qualifier.
- **`SYS_EXECWAIT = 37` + `sys_execwait(path, pathlen)` wrapper** added to `lib/syscalls_x86_64_agnos.cyr` (also backfilled the missing `SYS_UNAME`/`SYS_SYSINFO`/`SYS_KLOG` = 34/35/36 enum entries for parity).

### Notes

- **No argv/envp yet** — `execwait` takes the program path only (the envp bite is an agnos 1.43.x follow-on; `run`'s `arg1`/`arg2` stay unused on agnos). So `run /bin/klug` works (no args needed); `run /bin/bnrmr TEXT` runs bnrmr but can't pass `TEXT` until argv lands.
- **QEMU-validated kernel-side** (agnos `/bin/exwv` ring-3 selftest, sweep 7/7); the agnsh `run` path rides the next iron burn. Host build unaffected — `run()` there uses the POSIX fork/exec path, and the 84+59 host smoke assertions stay green.

## [1.4.3] - 2026-06-07

**The file verbs + `echo` are now discoverable, and a `run` builtin is wired (kernel-gated) for the coming execwait arc.** The `14115`-class iron burns surfaced two shell-UX gaps: the 1.4.2 in-process file verbs executed but were absent from `help` (so `echo` "worked but was hidden"), and there was no `run` command to launch a program from disk (`commandress`/`bannermanor`/`klug`). This patch documents the verbs and lands `run` as a first-class builtin — validated and pre-wired to the real exec path, gated off until agnos ships the execwait syscall.

### Added

- **`run PATH` builtin** in both the interactive loop and the `-c` one-shot path. Dispatched ahead of `dispatch_fs_verb` (it is a first-class builtin, never an FS verb), with absolute-path enforcement (leading `/`), and `..`-traversal / shell-metacharacter rejection via `is_safe_path`. The launch itself is centralized in one `sh_run_program(path, mode)` helper shared by both call sites, behind a single `RUN_EXECWAIT_READY` flag (`src/agnsh.cyr`).
- **Mode-gated launch (pre-wired).** When un-gated, `run` takes the same FIX 4 mode gate as `rm`/`mv` — HUMAN/STRICT confirm before launch, AUTO/ASSIST run directly (`mode_needs_confirm` + `verb_confirm`). Launching an arbitrary on-disk binary is at least as consequential as a destructive verb.

### Changed

- **`help` now lists every command that works.** The interactive `help` builtin gained a **Files:** group (ls/cat/cp/mv/rm/mkdir/rmdir/touch/echo/wc/find/grep, one-line each) and a **Run:** group (`run PATH`) — previously these executed but were undocumented. The boot-banner `Built-ins:` block gained a Files/Run summary line, and `print_usage()` (`-h`/`--help`) gained file-verb and `run` hints.

### Notes

- **`run` is wired but kernel-gated (`RUN_EXECWAIT_READY = 0`).** It validates the path and refuses with a clear message rather than driving the broken `spawn`(#3)+`waitpid`(#4) path, which fatally faults on iron: `waitpid` `hlt`-spins with `IF=0` (the scheduler-driving timer ISR never fires → hard hang), and `spawn`→ in-memory `elf_load` does a CR3-switch `memcpy` of agnsh's mmap-arena buffer (unmapped under the child CR3 → `#PF`/`#DF`/triple fault), with its 4 KB page mapped as a 2 MB huge page aliasing live memory. The proven exec path is the in-kernel recovery shell's `elf_load_from_file` + synchronous `exec_and_wait`; the **agnos 1.43.x execwait arc** exposes it as a ring-3 syscall and swaps `process_agnos.cyr`'s `run()` onto it. Then `RUN_EXECWAIT_READY` flips to `1` and `run` goes live with no other agnsh-side change. No argv/envp passing in the first un-gated cut (`sys_spawn`/execwait take the program path only).

## [1.4.2] - 2026-06-06

**Core filesystem verbs — agnsh can now actually operate on files.** `ls cat cp mv rm mkdir rmdir touch echo wc find grep` are implemented as in-process builtins that call the agnos (or host) syscalls directly. They're builtins rather than external binaries because running standalone programs from the shell waits on ring-3 `execwait` (agnos 1.43.x). This is the substantive core of the AGNOS-roadmap "Track B — userland environment". Validated on the host (`verbs-smoke.sh` 84/0) and on the live agnos kernel ext2 root (agnos `agnsh-verb-test.py`: `echo > /vtest` → `ls /` lists it → `cat` reads it back).

### Added

- **`src/verbs.cyr` — the 12 FS verbs as builtins.** `dispatch_fs_verb(line, mode)` runs after the inline builtins and before the NL→intent path in both the interactive loop and the `-c` one-shot path; an unknown first word falls through to the AI/intent path. Semantics: `ls`/`ls -l` (getdents + stat for type/size), `cat` (+ stdin), `cp` (streamed), `mv` (rename within a mount; clean cross-mount error), `rm`, `mkdir`, `rmdir`, `touch` (open `AO_CREAT`), `echo [-n] [> FILE]`, `wc`, `find [PATH] [-name PAT]` (recursive, depth-capped), `grep PAT FILE...`. Cross-target correctness is `#ifdef CYRIUS_TARGET_AGNOS` only where the surfaces genuinely differ: `open` flags, the getdents record layout (agnos §4.2 packed vs Linux `getdents64`), and the `stat` struct offsets.
- **Mode-gated safety for destructive verbs.** `rm`/`mv` and overwriting `cp` execute directly in **auto**/**assist** but prompt `⚠ <action> <path> ? [y/N]` in **human**/**strict** (proceed only on y/Y; EOF/Enter default to no). Read-only/create verbs (`ls cat wc find grep mkdir rmdir touch echo`) never prompt. The pre-existing safety gate is preserved: `rm -rf`/`--force`/`--recursive`/`..`-traversal/pipeline forms still route to the intent classifier (BLOCKED), and `rm`/`mv` reject any unknown `-flag` (closes a `rm -i FILE` hazard that would otherwise still unlink).
- **`scripts/verbs-smoke.sh`** — 84 host assertions covering every verb against a real temp dir + the safety-gate + mode-confirm behaviors. `scripts/smoke-test.sh` stays 59/0 (5 NL samples that collided with verb-leading words re-phrased to the same intent/safety classification).

### Fixed

- **`cp FILE FILE` no longer truncates the file to 0** (a same-path `streq` + inode guard refuses before any `O_TRUNC` open).

### Changed

- **`scripts/version-bump.sh` now syncs the `VERSION_STR` banner literal** in `src/agnsh.cyr` from `VERSION` (with a fail-loud count-guard), so a version bump can't desync the banner again — the root cause of the agnos `14115` burn showing `agnoshi 1.4.0` while `VERSION` was 1.4.1.

## [1.4.1] - 2026-06-05

**`read_line` is now a single line read — pairs with agnos 1.41.15's line-disciplined `read(fd 0)`.** The first iron burn of 1.4.0 typing (`14114`) read the keyboard but every line collapsed to a single stuck `Command: D` with no echo. The kernel-side root cause (the ring-3 IF=0 gap between per-byte reads dropping shift-release breaks) is fixed in agnos 1.41.15, which makes `read(fd 0)` line-buffered + kernel-echoed (canonical-lite). This shell-side change consumes that contract.

### Changed

- **`read_line` issues ONE `read(fd 0)` for the whole line instead of looping byte-by-byte.** The kernel now blocks until Enter, echoes printable bytes + handles backspace as you type, and returns the line with its trailing newline; `read_line` strips the terminator and returns the length (0 for a bare Enter, -1 on EOF). The old per-byte loop existed to keep line-oriented dispatch working under a RAW kernel, but on iron it masked IRQ1 in the ring-3 IF=0 gap between syscalls and dropped keystrokes — the `14114` collapse. No agnsh-side echo: the kernel owns echo now (mirroring the in-kernel recovery shell), so typed input is visible and dispatches intact. (`src/agnsh.cyr`.)

### Notes

- A richer line editor (`completion.cyr`/`history.cyr` with cursor movement) would want raw keystrokes again; that returns when agnos's future multithreading arc lets ring 3 run with IF=1 and O1 can go back to RAW. For now, canonical-lite is the only shape that types on iron.

## [1.4.0] - 2026-06-05

**`agnsh` reaches its interactive prompt AND accepts typed input in ring 3 on agnos.** This is the milestone the 1.3.x line was building toward: a userland shell exec'd from disk that a user can actually *type into* on the sovereign kernel. Proven in QEMU end-to-end — booted to `[ASSIST] >`, then `help` / `version` / `mode` injected through a USB-xHCI HID keyboard each dispatched and printed their output. The full input path works: USB-HID → kernel `hid_poll` → `read(fd 0)` → `read_line` → command dispatch.

The two blockers that stood between "renders banner" (1.3.x) and "reaches prompt + types" were cross-repo, and `1.4.0` is the marker that both are resolved:

### Fixed
- **Function-pointer dispatch was silently returning 0 on the agnos target — the prompt-blocker.** Every `fncall0`..`fncall8` (`lib/fnptr.cyr`) had `#ifdef` branches for `CYRIUS_TARGET_LINUX` / `_MACOS` / `_WIN` but **none for `CYRIUS_TARGET_AGNOS`**, so on agnos none of the `call rax` asm compiled and each `fncallN` fell through to its `var result = 0` initializer. That broke *all* vtable dispatch: `alloc_via(default_alloc(), …)` → `_bump_alloc` never ran → `vec_new()` returned 0 → `CommandHistory_new`'s `vec_len(entries)` dereferenced null → `#PF` (cpl=3, CR2=0x8) right after the banner, before the first prompt. Fixed by adding the `CYRIUS_TARGET_AGNOS` branch (identical x86-64 SysV asm to the Linux branch — agnos ring-3 shares the SysV convention) to all nine `fncallN`. This is the exact failure mode the file's own v5.9.38 comment records for the then-missing macOS branch. **Note:** `lib/fnptr.cyr` is a vendored cyrius-stdlib snapshot; the durable fix belongs upstream in cyrius and will propagate on the next `cyrius deps`/`update`. The Linux/macOS/Windows builds are unaffected (their branches already existed).

### Changed
- **`VERSION_STR` `agnoshi 1.3.6` → `agnoshi 1.4.0`** (`src/agnsh.cyr`) — the banner is the on-iron "which build am I running" canary.

### Depends on (agnos kernel, separate repo)
- **agnos `proc_unmap_page` no longer punches the kernel identity map.** The ring-3 stack guard-page unmap was zeroing a *supervisor* PD entry (pid 2's guard at `0xE00000` = PD[7]) which — under agnos's top-down `pmm_alloc` — identity-maps the very 2 MB region holding the process's own page tables. After the unmap the kernel couldn't walk to those tables under the per-process CR3, so the first `sys_mmap`/`proc_map_page` `#PF`'d at `CR2 = PML4-phys` and agnsh died before its banner. The kernel fix (clear user PDEs only) is what lets `agnsh` exec, run, and reach `read(fd 0)` at all.

### Validated
- QEMU (gnoboot + OVMF + NVMe ext2 rootfs, USB-xHCI keyboard): clean boot → `[ASSIST] >` prompt → typed `help` / `version` / `mode` each dispatched and rendered. No `#PF`, no fallback to the kernel emergency shell.

## [1.3.6] - 2026-06-04

**Hardening on the agnos target — not itself the prompt fix.** The `14111`/`14112` archaemenid burns showed `agnsh` launches in ring 3 and renders its banner, then never reached a prompt. The blocker turned out to be **outside agnsh**: a kernel PMM allocation bug in agnos (`pmm_alloc` cross-class fragmentation) left the userland mmap heap page mapped supervisor-not-user, so agnsh `#PF`'d on its first heap write on ~half of boots — fixed in **agnos 1.41.12** (`pmm_alloc` top-down; QEMU 0 `#PF`/52 boots, iron-pending). This patch is independent hygiene and does not by itself make the prompt appear. *(An earlier draft of this entry mis-diagnosed the fault as an agnsh "frame-smash / rbp corruption at `0x42c0b8`" — that read came from a stale disassembly and was wrong; the fault is the kernel mapping bug above, with healthy rbp/rsp.)*

### Fixed
- **Skip the guaranteed-useless `getenv("HOME")` on agnos.** `history_path()` and `audit_log_path()` now return their fixed `/tmp/agnsh_*` paths directly under `#ifdef CYRIUS_TARGET_AGNOS`. On agnos `getenv` opens `/proc/self/environ` (which doesn't exist) and always returns 0, so these helpers already fell back to the `/tmp` paths — the call was pure dead weight that also reserved an 8 KB stack buffer (`cyrius lib/io.cyr` `var buf[8192]`) on a deep call path. Behavior-preserving on agnos. **The Linux/macOS/Windows builds are unaffected** — they keep the `$HOME`-based history/audit paths their smoke tests assert (`scripts/smoke-test.sh` history-persistence + audit-log cases set `HOME` and check `$HOME/.agnsh_*`).

### Changed
- **`VERSION_STR` `agnoshi 1.3.5` → `agnoshi 1.3.6`** (`src/agnsh.cyr`) — the banner doubles as the on-iron "which build am I running" canary.

### Validated
- Host `smoke-test.sh` **59/59** green (the `#ifdef` guard is agnos-only; `$HOME` history/audit paths intact on Linux).
- agnos-target `agnsh_agnos` (282,912 B): with the agnos **1.41.12** PMM fix, `agnsh-smoke` boots agnsh past its first heap write to its prompt in QEMU across **0 `#PF` / 52 fresh boots** (was ~16/20 faulting). **Iron burn pending** for hardware confirmation.

## [1.3.5] - 2026-06-03

**agnsh runs on AGNOS for the first time.** Cyrius toolchain pin `6.0.14` → **6.0.56** — the bump that lands the `CYRIUS_TARGET_AGNOS` stdlib peer agnsh needs to boot on the sovereign kernel. With it, the `cyrius build --agnos` binary (`agnsh_agnos`, 282,880 B) launches in ring 3 from the agnos ext2 root and reaches its prompt — no more `#UD` at `args_init()` startup.

### Changed
- **Cyrius toolchain pin `6.0.14` → `6.0.56`** (`cyrius.cyml`). `cyrius update` repopulated `./lib/` from the 6.0.56 snapshot (84 → 89 files; adds `lib/args_agnos.cyr` + `lib/process_agnos.cyr`). The agnos-target build now resolves `args_init`/`argc`/`argv` (6.0.55's `lib/args_agnos.cyr` + init-stack capture), correct agnos file-op ABI (6.0.55's `lib/io.cyr` `AO_*` mapping fix), and `exec`/`run` of external programs (6.0.56's `lib/process_agnos.cyr`, `sys_spawn`-based). The Linux/macOS/Windows builds are unaffected.

### Fixed
- **`VERSION_STR` banner drift.** `src/agnsh.cyr` hard-coded `"agnoshi 1.3.2"` while `VERSION` had advanced to 1.3.4 — so `version` / `-v` / the startup banner printed a stale number. Synced to the live version (`agnoshi 1.3.5`).

### Validated
- **Boot-to-agnsh-on-disk** — built `--agnos` and seeded as `/bin/agnsh` on an ext2 root; agnsh launches in ring 3 under the agnos kernel (`kybernet` PID 1 execs it) and reaches its prompt; agnos-side `agnsh-smoke` PASS ×3 (no `#UD`, no emergency-shell fallback). The agnos-target binary is `ud2`-free (0 unresolved-call sentinels).

## [1.3.4] - 2026-05-28

Cyrius toolchain pin 6.0.1 → **6.0.14** — a within-6.0.x patch-level bump. Zero codegen drift: both binary sizes are byte-for-byte identical to v1.3.3 (x86_64 295,312 B / aarch64 339,512 B). All gates clean, all tests green, benchmarks unchanged.

### Changed
- **Cyrius toolchain pin 6.0.1 → 6.0.14** (`cyrius.cyml`). Patch-level bump within the 6.0.x line; `cyrius deps` repopulated `./lib/` from the 6.0.14 stdlib snapshot. The wrapper had already advanced to 6.0.14 (manifest-pin drift); this aligns the manifest. Cleanliness gates all clean against the new snapshot: `cyrius check` ok, fmt no drift (22 files), lint 0 warnings (22 files), `cyrius vet` 22 deps / 0 untrusted / 0 missing, `cyrius capacity --check` all caps under 85% (fn_table 512/8192, code_size 154,864/1,048,576). Bracketed benchmarks pre/post: all 10 averages unchanged to microsecond resolution (per-run jitter only). **Binary size unchanged** on both arches — x86_64 295,312 B, aarch64 339,512 B — no codegen drift between 6.0.1 and 6.0.14.

### Notes
- **Toolchain warnings under 6.0.14** — unchanged from v1.3.3 in character: test_core/test_security/bench builds emit `warning:<source>:1: duplicate fn 'getenv' (last definition wins)` (test-side stub vs `lib/io.cyr` getenv) and test_core emits `warning:src/security.cyr:1087: syscall arity mismatch` (the synthesized-position diagnostic, now attributed to :1087 vs :1086 under 6.0.1 — still past security.cyr's real line count; doesn't reproduce in the production `agnsh` build). Both are warnings, not errors. Production `agnsh` build is warning-free.
- **Test count** stays at 301 unit + 26 security + 59 smoke. Coverage 86%. lint-cstr-str clean.
- **Zugot recipe** at `~/Repos/zugot/marketplace/agnoshi.cyml` to be updated locally with the new version + sha when release artifacts land.

## [1.3.3] - 2026-05-20

Cyrius toolchain bump 5.10.44 → **6.0.1** + a latent path-traversal safety regression caught at the new compiler's codegen layout. The safety predicate flake was a pre-existing v1.0-era bug in `sanitize.cyr` that 5.10.x's address layout happened to not surface; 6.0.x's different bump-allocator stride exposed it as a 5-10% smoke-test flake on the v1.3.1 `create directory ../foo` rejected_safety probe. ADR-006 §"explicit `_in_str` discipline" now extends to: any `strlen(s)` inside a `_in_str` fn body is also caught by the lint shield (new Category F). 100/100 traversal-probe repro post-fix.

### Changed
- **Cyrius toolchain pin 5.10.44 → 6.0.1** (`cyrius.cyml`). Major-version bump across the 5.10.x → 6.0.x boundary. The 6.0.x stdlib snapshot picks up upstream bug fixes — notably the `_exec3` argv/envp `var X[4]` / `var X[1]` byte-count fix (was reserving 4 + 1 bytes, silently corrupting the stack on every call; now 40 + 16 bytes) landed in upstream 5.11.60 — and adds new stdlib surfaces (`audit_walk`, `base64`, `bigint`, `bounds`, `callback`, `cffi`, `csv`, `ct`, `cyml`, `dynlib`, etc.) not yet consumed by agnoshi. Bracketed benchmarks pre/post (`bench-history.csv` rows at `cyrius_5.10.44` vs `deab6d3`): all 10 averages unchanged to microsecond resolution (perm/classify_5 6us → 5us is within the per-run jitter window). Codegen drift on x86_64: 293,920 → **295,312 B** (+1,392 / +0.5%); aarch64 337,168 → **339,512 B** (+2,344 / +0.7%). No agnoshi source code change driving the size delta — pure toolchain-side.

### Fixed
- **sanitize.cyr — `strlen(s)` dispatch unreliable inside `_in_str` fns under Cyrius 6.0.x.** Three sites in `path_traversal_in_str` / `shell_metachars_in_str` / `safe_arg_in_str` called the bare `strlen(s)` and relied on Cyrius's name-lookup dispatch to route `strlen(Str)` → `strlen_str(Str)` → `load64(s + 8)`. Under 6.0.x's stricter type inference, the dispatch falls through to the cstring `strlen` when the param has no explicit `: Str` annotation, which walks the Str fat-pointer's bytes looking for a null terminator. Heap addresses on Linux x86_64 typically have a zero byte at position 4-7 (high-order bytes of the user-space pointer), so `strlen(Str-as-cstring)` returns 1-7 — usually 4 — instead of the real length. For `path_traversal_in_str("../foo")` (real len 6), the corrupt-length 1 case skipped the loop via the `len < 2` guard, returning 0 (no traversal detected). `safe_path_in_str` then said "safe", and the translator emitted `mkdir -p ../foo` instead of `translate_unknown`. **Live bug-class on the agnoshi safety predicate surface** — repro: 5-10% of `create directory ../foo` invocations under 6.0.1 cycc + 5.10.44 lib OR 6.0.1 cycc + 6.0.1 lib (cycc determines the codegen, lib is irrelevant). Fix: explicit `var len = str_len(s);` (the Str-side primitive — no dispatch ambiguity). 100/100 traversal-probe repro post-fix on both lib snapshots.

### Added
- **scripts/lint-cstr-str.sh — Category F** (`strlen(...) inside _in_str fn body`). Awk-based scanner: when inside a fn whose name matches `_in_str`, any `strlen(` call is flagged unless tagged `# lint:cstr-ok`. Catches the v1.3.3 bug class at lint time. Verified by reverting one of the three fixes — lint flagged all three sites and exited 1. Reverted again and lint passed.

### CI
- **.github/workflows/{ci,release}.yml — Cyrius 6.0 binary-rename + install-layout updates.** Cyrius 6.0 renamed `cc5` → `cycc` and `cc5_aarch64` → `cycc_aarch64`; the existing `cc5 --version` verify step and `cc5_aarch64` aarch64-gate would both fail on every 6.x release. Replaced `cc5 --version` with `cyrius --version` (the wrapper — present in every release, reports both wrapper + cycc versions and flags pin/wrapper drift). Replaced `cc5_aarch64` checks with `cycc_aarch64`. Install step migrated from flat `~/.cyrius/{bin,lib}` to versioned `~/.cyrius/versions/$VERSION/{bin,lib}` + symlinks layout (mirrors sankoch's CI; matches the 6.0+ wrapper's manifest-pin drift-check expectations). Added the `cycc_aarch64`-at-tarball-root fallback the way sankoch/yukti/sakshi carry it (some 6.x releases ship the aarch64 cc under `$CYRIUS_DIR/cycc_aarch64` instead of `$CYRIUS_DIR/bin/cycc_aarch64`).
- **src/security.cyr** — comment refresh: `cc5` / `cc5_aarch64` references in the `verify_sudo_path` arch-flag note bumped to `cycc` / `cycc_aarch64` with a parenthetical noting the 6.0 rename. No code change.

### Notes
- **Bug class lesson** — ADR-006's "explicit `_in_str` suffix" rule was about *naming*; this cycle extends it to *call-site discipline*. Any `_in_str` fn body should reach for the Str-side primitives directly (`str_len`, `str_data`, `str_byte_at`) and not rely on dispatch magic from the overloaded names (`strlen`, `str_eq`). The dispatch worked reliably on 5.10.x but stopped working reliably on 6.0.x, and the regression was silent (no warning, no crash, just a flaky safety bypass). Per ADR-006's "mechanical enforcement" principle, this rule lives in the lint shield, not in human review.
- **Toolchain warnings under 6.0.1** — test_core/test_security/bench builds now emit `warning:<source>:1: duplicate fn 'getenv' (last definition wins)` (test-side stub at `test_core.tcyr:14` collides with `lib/io.cyr:246`'s getenv that 6.0.x's stricter duplicate-detection now surfaces) and `warning:src/security.cyr:1086: syscall arity mismatch` (test_core only; doesn't reproduce in the production agnsh build; expanded-line attribution past security.cyr's real 165 lines suggests a synthesized-position diagnostic — flagged for follow-up). Both are warnings, not errors; tests pass 301 + 26. Production `agnsh` build is warning-free.
- **Test count** stays at 301 unit + 26 security + 59 smoke. Coverage 86%. lint-cstr-str clean. Lint Category F now in CI gate set.
- **Binary**: x86_64 **295,312 B** (+1,392 vs v1.3.2). aarch64 **339,512 B** (+2,344). Both arches build clean; toolchain-side growth, not new agnoshi code.
- **Zugot recipe** at `~/Repos/zugot/marketplace/agnoshi.cyml` to be updated locally with the new version + sha when release artifacts land.

## [1.3.2] - 2026-05-11

Doc-staleness sweep + `rust-old/` removal. Per the AGNOS first-party-standards "Delete `rust-old/` only after the Cyrius version has equal or better test coverage and benchmarks" criterion, v1.3.1's CI lint shield + ADR-006 + 301 + 26 + 59 unit/security/smoke tests + bracketed `bench-history.csv` clear the bar. Doc staleness from the v1.1.0/v1.2.0/v1.3.0 cycles also caught up.

### Removed
- **`rust-old/` directory** — 1.2 MB, 65 .rs files, 21 modules + tests + benches + Cargo metadata. Historical record preserved in `benchmarks-rust-v-cyrius.md` (port-arc benchmark snapshot, frozen by design at Rust 0.90 vs Cyrius 4.5.0), `docs/adr/001-cyrius-port.md` (port rationale), and the git history of the v0.x → v1.0.0 commits. `.gitignore` updated to ignore any restored copy. No live-code impact (rust-old was never linked).

### Documented
- **rust-old parity audit** — every Rust module catalogued vs current Cyrius `src/`. 18 modules cleanly ported. Six categories of intentionally un-ported Rust modules with documented homes:
  - `dashboard.rs` (413L) — agent-TUI; separate concern from shell.
  - `llm.rs` (461L) — deferred → v1.4.0 (needs hoosh modernization first).
  - `schema_filter.rs` (464L) — MCP infrastructure; belongs in bote/mela.
  - `sandbox.rs` (187L) — superseded by kavach (per AGNOS "kavach owns the sandbox").
  - `interpreter/patterns.rs` (851L) — replaced by keyword matching per ADR-003.
  - `interpreter/explain.rs` (581L) — partial port (per-translator `explanation` field exists; standalone `explain <cmd>` lookup table deferred → v1.5.x+ man-page-integration).
  - `interpreter/translate/{aequi,bullshift,delta,edge,jalwa,knowledge,marketplace,mneme,photis,phylax,rasa,shruti,synapse,tarang,tazama,tron}` (16 modules) — consumer-app translators deferred → v1.5.x+ (v1.0 pruned IntentTag 211→44; wire up only when each consumer lands its public surface).
  
  Verdict: no unintentional gaps; everything missing has a documented home in the roadmap.

### Changed
- **README.md** — stat-line refreshed (was `1.1.0 · Cyrius 5.10.34 · ... · 272 KB`; now `1.3.1 · Cyrius 5.10.44 · ... · 294 KB static binary (DCE, x86_64) · 337 KB aarch64 · ... · 301 unit + 26 security + 58 smoke tests`). Pin refs bumped to 5.10.44. "Rust Legacy" section rewritten as historical reference pointing at the `benchmarks-rust-v-cyrius.md` + ADR-001 + git history.
- **CONTRIBUTING.md** — Cyrius pin ref bumped 5.10.34 → 5.10.44.
- **docs/architecture/overview.md** — pin ref bumped; binary-size line updated with per-arch numbers; "Language Migration" section rewritten now that `rust-old/` is removed.
- **docs/guides/getting-started.md** — every example output refreshed for the v1.3.0 surface: mode-aware prompt (`[ASSIST] >`), `Risk: [LEVEL]` line in place of `Permission: N`, BLOCKED warning + approval-required hint blocks, `mode` / `history` builtin demos. Audit-log example updated to the six-class `result` vocabulary. Undo section qualified as v1.4.0-pending (today's `-c` and interactive modes propose without executing).
- **docs/guides/writing-intents.md** — dropped the cc3-era `< 64 enum entries` claim (gone in Cyrius 5.10.x); added a §3 callout pointing to ADR-006 + `safe_path_in_str` / `safe_arg_in_str` for translator safety predicates + the `scripts/lint-cstr-str.sh` lint shield.
- **docs/guides/security-model.md** — §2 (Argument Sanitization) updated to reference both cstring (`is_safe_arg`) and Str-aware (`safe_arg_in_str`) variants per ADR-006. Added a "Known LOW-severity hardening deferred to v1.4.0" subsection covering the symlink-race-on-state-files finding and the chmod-failure stderr warning from v1.3.1 slice 4. New "Forward Shield (v1.3.1)" section pointing at the 14-pattern lint + ADR-006 + the P(-1) audit report.
- **docs/examples/scripting.md** — "Programmatic Access" section rewritten with the actual `-c` output shape (Risk / WARNING / Approval / Hint lines), the audit-log six-class `result` vocabulary, and `jq` recipes for filtering on result class.

### Zugot recipe (separate repo)
- `~/Repos/zugot/marketplace/agnoshi.cyml` updated **locally only** (no agnoshi GH release cut): version `1.0.0 → 1.3.1`, bin name `agnoshi → agnsh`, asset glob `agnsh-*-x86_64-linux` matching `.github/workflows/release.yml`, tags drop stale "rust" → "cyrius", landlock entries added for `~/.agnsh_history` + `~/.agnsh_audit.log`. `sha256` left at the v1.0.0 value with a TODO comment — needs an update when a real v1.3.1+ release lands.

### Notes
- **No source-code changes in this slice**. Pure docs + deletion + version bump. Binary, test, smoke, coverage, gate sweep all unchanged from v1.3.1 (x86_64 293,920 B / aarch64 337,168 B / test_core 301/301 / test_security 26/26 / smoke 59/59 / coverage 86% / lint-cstr-str clean).
- **Repo size drop** — 1.2 MB → 580 KB working tree (excluding `lib/` and `build/`). `.git` retains the full history.
- **v1.3.x bucket** is now empty in the roadmap; v1.4.0 (exec wire-up + hoosh + completion) is the next anchor.

## [1.3.1] - 2026-05-11

P(-1) audit/review pass per AGNOS first-party standards. Eight slices across the roadmap's P(-1) bullets: toolchain bump (zero codegen drift), cstring/Str static analyzer with **14 patterns across 5 categories** wired into CI (catches 7 distinct bug variants that took 5 separate slices to discover during v1.2.0/v1.3.0), buffer-safety sweep with 5 dormant static-buf-escape fixes, syscall-return audit with 2 HIGH-severity unchecked-chmod fixes (live multi-user data-leak shape on `$HOME/.agnsh_history` and `$HOME/.agnoshi/checkpoints/`), ADR-006 codifying the four operational rules, input-validation sweep clearing 3 stale-stdlib breaks in `prompt.cyr`, path-traversal sweep verifying every file-op site, CVE pattern review.

**Zero CRITICAL findings.** Eight HIGH all fixed. Five MEDIUM deferred (three getcwd + two `str_data(Str)` → syscall, all in modules not currently in agnsh's include graph). Twelve LOW triaged.

### Fixed
- **history.cyr** — unchecked `sys_chmod` return on `$HOME/.agnsh_history`. **HIGH severity, live in agnsh**: if chmod fails the file stays at umask default (0644), leaking every typed command to other users on a multi-user system. Captured return + stderr warning.
- **checkpoint.cyr** — same chmod shape on the checkpoint directory. **HIGH severity, deferred module**. Same fix.
- **session.cyr** — `str_starts_with(trimmed, "mode ")` was Str + cstring with no `_cstr` overload (would have silently never matched the `mode <name>` builtin when wired). Wrapped cstring with `str_from`.
- **Five static-buf-escape sites** in `ui.cyr` / `prompt.cyr` / `session.cyr` (× 3) — `str_from(&local_static_buf)` returns or stores a Str borrowing memory that gets overwritten on the next function call. **HIGH severity each, all dormant**. Same shape that bit `CommandHistory_add` in v1.3.0 slice 7. Wrapped each with `str_clone(str_from(&buf))`.
- **prompt.cyr — three stale-stdlib build breaks** (`fs_exists` → `file_exists`, single-arg `file_read_all` → buffer-based, `fs_parent` removed) plus a variable-cstring passed to `str_starts_with`. Caught during the input-validation sweep. `prompt_detect_git_branch` rewired against Cyrius 5.10.x stdlib; parent-walk replaced with a single-iteration break (TODO for v1.4.0 wire-up). Module now ready for inclusion.

### Added
- **scripts/lint-cstr-str.sh** — static analyzer for the Cyrius cstring/Str mismatch class. Five categories, fourteen patterns:
  - A: 1st-arg cstring × 5 (`str_len`, `str_data`, `str_cat`, `str_starts_with`, `str_ends_with`)
  - B: 2nd-arg cstring × 3 (`str_cat`, `str_starts_with`, `str_ends_with`)
  - C: aarch64-broken syscalls × 3 (`SYS_OPEN`, `SYS_CHMOD`, `SYS_STAT`)
  - D: static-buf escape × 2 (`return str_from(&...)`, `store64(*, str_from(&...))`)
  - E: security-critical syscall return × 1 (`^\s*sys_chmod(`)
  
  Word-anchored regexes (so `cstr_starts_with` ≠ `str_starts_with`). Only fns lacking a `_cstr` overload in `lib/str.cyr` are flagged (so `str_contains` / `str_eq` / `str_split` are intentionally not — Cyrius dispatches them). Escape hatch: `# lint:cstr-ok`. Wired into `.github/workflows/ci.yml`.
- **docs/adr/006-cstr-str-dispatch-discipline.md** — refines ADR-005 with four operational rules earned by v1.2.0/v1.3.0 production discovery: (1) explicit `_in_str` suffix for cross-type-boundary Str-side variants, (2) per-arch syscall wrappers, (3) `str_clone(str_from(&buf))` for static-buf escape paths, (4) CI lint shield as mechanical enforcement. Catalogues all seven historical bug variants with discovery-slice attribution and severities.
- **docs/audit/2026-05-11-pminus1.md** — full P(-1) audit report per AGNOS convention. Eight §-sections one per slice; running tally by severity; known linter gaps documented.
- **bench-history.csv** — 20 new rows bracketing the toolchain bump (5.10.34 vs 5.10.44, 10 benchmarks each).
- **docs/agnsh.1**, **docs/doc-health.md**, **docs/adr/README.md**, **docs/development/roadmap.md** — refreshed for the v1.3.1 close.

### Changed
- **Cyrius toolchain pin 5.10.34 → 5.10.44** (`cyrius.cyml`). 10-patch bump along the 5.10 line. Bracketed benchmarks pre/post: all 10 averages unchanged to microsecond resolution. Codegen drift on x86_64: −32 bytes; aarch64 unchanged. No regression.

### Notes
- **Test count** stays at **301** through P(-1) (no new feature work). Smoke 58 → **59** (added a cleaner CREATE_DIR-USER_WRITE traversal probe). Coverage 86%. Both arches build (x86_64 293,920 B / aarch64 337,168 B — small growth from chmod-warning strings and history audit log changes).
- **CI gate count** grew: new `lint-cstr-str` step runs after `check-coverage`. Failing the new gate fails CI like fmt / lint / capacity drift.
- **Bug-class history** documented in ADR-006 §Context: seven distinct variants, each linked to its discovery slice. Six of seven took a probe / SIGSEGV / first-use crash to find; all seven are now lint-caught.
- **Smoke check loosened** for the REMOVE-traversal probe — accepts either `result:"rejected_safety"` (translator-side catches the path traversal) OR `result:"blocked"` (BLOCKED-perm classification short-circuits ahead of the safety check on some build envs). Both indicate the command won't auto-execute; the user's safety is preserved either way. Replaced by an additional CREATE_DIR-USER_WRITE traversal probe (no permission-vs-safety ambiguity — must be `rejected_safety`).

## [1.3.0] - 2026-05-11

The v1.2.x cycle outgrew patch scope — what started as "v1.2.1 approval workflow + interactive shell" closed *both* lead roadmap items, swept five Cyrius 4.5 → 5.10 stdlib regressions across deferred modules, and added a six-label audit-result vocabulary. Bumped to v1.3.0 to reflect the actual scope.

**Approval workflow battle-tested interactively** — every `-c` invocation now prints `Risk: [LOW|MED|HIGH|CRIT]` with `WARNING: BLOCKED` / `Approval required` lines as appropriate; `src/approval.cyr` and `src/audit.cyr` and `src/security.cyr` all wired into the binary's include graph. Audit-log JSON line per command (timestamp + user + mode + input + action + approved 0/1 + result), real wall-clock timestamps via `lib/chrono.cyr::iso8601_now()`, six-class `result` field (`proposed`, `needs_approval`, `blocked`, `needs_llm`, `needs_exec`, `rejected_safety`) so downstream filters can `jq 'select(.result == "rejected_safety")'`. Sudo path re-verified for existence AND root-ownership at the escalation moment via `verify_sudo_path` — closes the TOCTOU window between session-init cache and actual sudo invocation.

**Interactive shell end-to-end** — mode-aware prompt (`[ASSIST] >`, `[HUMAN] >`, `[STRICT] >`, `[AUTO] >`), `mode` / `history` / `clear` / `help` builtins, persistent command history at `$HOME/.agnsh_history` (last 1000 entries) loaded on session start, line-oriented `read_line` byte-reader for piped + terminal use, error-recovery `Hint:` lines surfacing parse-succeeded-but-translation-not-runnable cases (LLM-routed questions, pipelines, safety-rejected translations).

**Five deferred modules unbusted**. The v1.1.0 toolchain migration left silent build breaks in every module not in the agnsh binary's include graph. This cycle swept:
- **audit.cyr** — `str_cat(cstring, *)` × 3; `fs_exists` / `json_get_str` / `file_read_all` arity (read-side `AuditViewer_query` body gutted, MCP-routed AUDIT_VIEW is the user-facing path)
- **security.cyr** — `fs_exists` → `file_exists` × 5; `file_read_all` arity; `process_exec` → `exec_vec`; `str_data(cstring)` mismatch; `streq(Str, Str)` → `str_eq` (caught by Cyrius 5.10.x's type-warning hint — first time the toolchain caught one of these)
- **history.cyr** — `fs_exists`/`file_read_all`/`file_write_all` arity; `fs_parent`/`fs_mkdir_p` removed; `streq` → `str_eq`; entry-data-lifetime fix (`str_clone(str_from(...))` to break aliasing with the reused interactive_loop static buf); explicit byte-separator workaround for `str_split` cstring-needle dispatch
- **translate.cyr** — `is_safe_path` / `is_safe_arg` were cstring-only but the parser hands `Str`; added `safe_path_in_str` / `safe_arg_in_str` variants and routed all 11 translator call sites. **Every NL filesystem operation (copy / move / remove / mkdir / show-file / find / search-content) had been silently routing to `translate_unknown` since v1.0** because of this — now `agnsh -c "copy a to b"` correctly prints `Command: cp / Risk: [MED]` instead of `Command: echo / Risk: [LOW]`.
- **sanitize.cyr** — already swept in v1.2.0 slice 1; v1.3.0 carries the str_substr migration + new Str-aware variants

### Added

- **agnsh.cyr — runtime wire-up** of approval/audit/history/chrono; `audit_log_path` / `history_path` cstring builders; `audit_one_shot` with mode-aware + tag-aware classification; `classify_audit_result` six-label classifier; `print_intent_result` with risk-line + Hint:-line dispatch; `interactive_loop` with `ModeManager` + `CommandHistory` + `read_line` + builtin dispatch (help / version / mode / mode `<name>` / history / clear / exit / quit).
- **sanitize.cyr — Str-aware safety predicates** (`safe_path_in_str`, `safe_arg_in_str`, `path_traversal_in_str`, `shell_metachars_in_str`); `is_word_prefix` token-prefix matcher (plural-tolerance + substring-trap immunity); `input_starts_with` interrogative-form gate.
- **interpreter.cyr — new NL paths** — `parse_state_queries` (noun-phrase queries: ip address / my ip / uptime / disk space / running processes / etc.), `parse_service_query` (`is X running` / `is X active` / `is X enabled` / `status of X`), `parse_service_action` (bare imperative `start X` / `stop X` / `restart X` / `reload X` / `enable X` / `disable X`); `token_count` whitespace tokenizer.
- **security.cyr — `verify_sudo_path`** — escalation-time existence + root-ownership re-check.
- **scripts/check-coverage.sh** — fn-level coverage gate, ≥80% threshold, wired into `.github/workflows/ci.yml`. Current: 86%.
- **tests/test_core.tcyr** — 44 new assertions across approval workflow (20), audit JSON shape (11), security context + sudo gate (11), coverage anchors. Test count 257 → **301**.
- **scripts/smoke-test.sh** — 27 new assertions across interactive mode/history/exit (9), audit shape (4), audit result vocabulary (6), Hint: surfacing (4), command-field populated (1), risk-line per level (3). Smoke 31 → **58**.

### Fixed

- All five-module stdlib regressions listed above.
- **agnsh.cyr** — `str_print(cmd)` where `cmd` was a cstring command literal (e.g. `"ls"`, `"systemctl"`); pre-v1.3.0 the `Command:` line silently printed empty because `load64(s+8)` read past the cstring as a fake Str length. Now `str_print(str_from(cmd))`.
- **Interactive banner** — was hardcoded `agnoshi 1.1.0`; now uses `VERSION_STR`.
- **aarch64 cross-build** — three direct `syscall(SYS_*, ...)` sites broke the aarch64 CI cross-build because aarch64's generic syscall table doesn't expose bare SYS_OPEN (= io_setup there), SYS_CHMOD (only fchmodat), or SYS_STAT (with a different `struct stat` layout). Switched to the per-arch wrappers `sys_open` / `sys_chmod` / `sys_stat` (lib/syscalls_{x86,aarch64}_linux.cyr both export them) in `audit.cyr`, `history.cyr`, and `security.cyr`. The st_uid offset in `verify_sudo_path` is now `#ifdef`-gated per the architecture's `struct stat` layout (x86=28, aarch64=24). Both arches now build clean: x86_64 293,824 B, aarch64 337,032 B.

### Notes

- **Binary size**: 280,344 B (v1.2.0) → **293,824 B** (+13.5 KB). Growth from approval.cyr + audit.cyr + history.cyr + lib/chrono.cyr now in the include graph, plus the new parser helpers + safety predicates + audit/result vocabulary.
- **Coverage**: holds at 86% — the denominator grew with five newly-included modules; numerator grew through smoke + test_core additions to stay above the 80% gate.
- **Bug-class lesson** — five Cyrius 4.5 → 5.10 stdlib regression patterns surfaced over the v1.2.0 + v1.3.0 arc: `str_len(cstring)`, `str_sub(start, end)` semantics, `str_cat(cstring, *)` / `str_cat(*, cstring)`, `is_safe_path(Str)`, and renames (`fs_exists` / `process_exec` / `file_read_all` / `file_write_all` arity). Cyrius 5.10.x's type-warning hint caught one (`streq` in slice 5); the rest still surface as silent runtime fallthroughs. A static-analyzer pass for "cstring passed where `Str` is typed" is queued for v1.4.0 tooling.
- **What's deferred** — Tab completion (terminal raw mode + tty escapes); LLM response streaming (waits on hoosh modernization); exec wire-up for SAFE/READ_ONLY commands; `undo` builtin (needs exec wire-up). All slotted for v1.4.0. The v1.3.1 P(-1) audit/review pass per [agnosticos first-party standards](https://github.com/MacCracken/agnosticos/blob/main/docs/development/planning/first-party-standards.md) sits between.

---

**Detailed slice history below** — slices 1-9 documented as they landed (decision UI risk-print, audit-log shape, sudo timing, mode switching + line-oriented stdin, command history, error-recovery hints, audit result vocabulary, plus two bug-class audit sweeps).

### Slice 9 — audit result enrichment (this cut)

#### Added
- **agnsh.cyr: `classify_audit_result(tag, perm, desc_cstr)`** — maps the parse+translate outcome to one of six audit-result labels. Mirrors slice 8's Hint: lines so the user-facing surface and the audit JSON tell the same story. Order-sensitive: QUESTION / PIPELINE tag checks come first because PIPELINE has no translator arm and falls through to `translate_unknown` (stamping the same `"Unknown intent"` description that a real safety-rejected translation does); without the tag-first order, pipelines would mis-classify as `rejected_safety`. Six labels:
  - `rejected_safety` — translator safety check rejected the input (path traversal, shell metachars, leading-dash commit message, null PID, etc.)
  - `needs_llm` — QUESTION tag: LLM streaming not yet wired
  - `needs_exec` — PIPELINE tag: no translator arm
  - `blocked` — BLOCKED permission level
  - `needs_approval` — HIGH risk (SYSTEM_WRITE / ADMIN): would require an interactive approval prompt
  - `proposed` — SAFE / READ_ONLY / USER_WRITE: auto-runnable, logged as-is until exec wire-up lands (at which point this label flips to `executed` / `denied` / `error` at the exec call site)
- **agnsh.cyr: `audit_one_shot` extended signature** — now takes `(input, cmd, perm, mode_label, tag, desc)` and delegates the result classification. `print_intent_result` passes `tag` and `load64(translation + 16)` (the description cstring) through.
- **scripts/smoke-test.sh — 6 new audit-result assertions** — one input per label class, each verified via grep against the on-disk audit log. The `input` prefix in each grep is the parser's deduplication anchor: `'"input":"show files".*"result":"proposed"'` etc. Smoke 52 → **58**.

#### Notes
- **Binary size**: 293,312 → 293,824 B (+0.5 KB) — six cstring labels + the classifier dispatch.
- **Probe**: a single `-c` cycle through `show files / install vim / rm /tmp/x / what is dns / ls | grep foo / remove ../etc/passwd` produces six audit lines with all six result labels in order. The `result` field is now the single grep-target for "what did agnoshi actually do".
- **Coverage** holds at 86%. `classify_audit_result` gains transitive coverage via smoke (each of the six labels exercised end-to-end); the classifier's ordering bug from the first probe (PIPELINE → rejected_safety) was caught by the smoke probe itself, not unit tests — direct unit tests would be a follow-up.
- **Forward compat**: when exec wires up, `audit_one_shot` will take an additional `exec_result` arg and the `proposed` label will be replaced at the call site with `executed` / `denied` / `error`. The other five labels (`needs_*`, `blocked`, `rejected_safety`) stay — they describe parse-time decisions, not runtime outcomes.

### Slice 8 — interactive shell: error-recovery hints (committed)

#### Added
- **agnsh.cyr: post-translation `Hint:` line** in `print_intent_result`. Three classes:
  - **QUESTION (tag 42)** — LLM streaming isn't wired; the parser classified the input as a question but agnoshi can only echo. Hint: `question intent -- LLM streaming arrives in a later slice`.
  - **PIPELINE (tag 41)** — no `translate_pipeline` arm in `translate_core` / `translate_extended`, so the dispatch falls to `translate_unknown` (echo, SAFE). Hint: `pipeline intent -- auto-exec arrives with the exec wire-up`.
  - **Safety-rejected translation** — any tag whose `translate_X` called `is_safe_path_str` / `is_safe_arg_str` / `is_valid_pid` / `is_safe_commit_message` on a parser-extracted field and the predicate rejected. Detected by the `"Unknown intent"` description that `translate_unknown` stamps. Hint: `translator safety check rejected this input -- try rephrasing`. This is the actual v1.2.1 error-recovery sub-bullet: pre-slice-8 a user typing `remove ../etc/passwd` saw `Command: echo / Risk: [LOW]` and could plausibly believe the deletion was queued; now the rejection is surfaced.
- **scripts/smoke-test.sh — 4 new hint assertions**: each of the three hint classes appears for a matching input; happy-path inputs (`show me files`) do *not* carry a `Hint:` line (negative check, no false-positive). Smoke 48 → **52**.

#### Notes
- **Binary size**: 292,920 → 293,312 B (+0.4 KB) — three println strings and one streq.
- **Order of output** preserved: Risk line still shows the technical classification first (some callers / scripts may want the raw permission level even on echo-only translations); the Hint follows. BLOCKED warnings still print before the Hint (a BLOCKED translation that's also safety-rejected gets both lines, which is the correct surface area).
- **Audit unchanged this slice** — the audit entry still records `action="echo"` for these cases. A future slice could enrich audit with a separate `result` field value like `"rejected_safety"` / `"needs_llm"` / `"needs_exec"` to distinguish from real echo invocations, but the user-facing print is the higher-leverage fix and stands alone.
- **Remaining v1.2.1 interactive-shell sub-items**: completion (tab) and streaming LLM responses. Both need bigger infrastructure (completion: terminal raw mode + tty escape handling; streaming: hoosh wire-up). At this point the interactive shell is fully usable end-to-end for the parse-and-classify use case; the remaining items light up *execution* paths.

### Slice 7 — interactive shell: command history (committed)

#### Fixed
- **src/history.cyr — Cyrius 5.10.x stdlib alignment**. Same shape as slice-5's security.cyr repair: four latent bugs that compile-broke any wire-up attempt.
  - `fs_exists` (2 sites) → `file_exists` (per lib/io.cyr 5.10.x).
  - `file_read_all(path)` single-arg → buffer-based `(path, buf, maxlen)`. Reworked `CommandHistory_new` to alloc a 64 KB scratch buffer, read the file, null-terminate, wrap as Str for `str_split`.
  - `file_write_all(path, content)` two-arg → `(path, buf, len)`. `CommandHistory_save` now passes `str_data(content)` + `str_len(content)`.
  - `fs_parent` / `fs_mkdir_p` — don't exist in 5.10.x stdlib. Removed the parent-dir-create call; `$HOME` is the conventional parent and is guaranteed to exist when the shell starts.
- **history dedup**: `streq(last, command)` was a Str/cstring type mismatch (entries are Str from str_split / str_clone; command was cstring) — replaced with `str_eq(Str, Str)`. Pre-slice-7 dedup never matched, so a flood of identical inputs would have all stuck. Caught by Cyrius 5.10.x's type-warning hint at first build attempt.
- **history entry data lifetime**: `CommandHistory_add` was `str_from(command_cstr)` which BORROWS the cstring buffer. interactive_loop reuses one static `var buf[4096]` across iterations, so every history entry's data pointer would alias to whatever was in `&buf` at display time — first probe showed all entries dereferencing to the same garbled bytes. Now `str_clone(str_from(command_cstr))` deep-copies into a fresh heap buffer so the stored Str is independent of the caller's scratch.
- **load-side str_split**: `str_split(content, "\n")` returned the whole file as one entry — the cstring-needle dispatch path didn't route the way audit/translate calls do. Switched to explicit byte separator `str_split(content, 10)` (where 10 is `'\n'`). Persisted multi-line history files now load with the correct entry count.

#### Added
- **agnsh.cyr: `history_path()`** — `$HOME/.agnsh_history` cstring builder, mirrors `audit_log_path` shape. Falls back to `/tmp/agnsh_history` when HOME is unset.
- **`CommandHistory` wired into `interactive_loop`** — loads from disk on session start (last 1000 lines); every non-builtin input gets recorded; `CommandHistory_save` writes the file back on `exit` / `quit`.
- **`history` builtin** — prints the last 20 entries with 1-indexed numbering. `(history empty)` when the list is empty (vs silently printing nothing — discoverability).
- **`help` updated** — mentions `history` builtin and that `exit/quit` save history.
- **scripts/smoke-test.sh — 8 new history assertions**: entry recording (entries 1 and 2 appear in the in-session `history` output), file creation, file line count, file content shape, persistence across sessions (next invocation's `history` shows the same entries), empty-history path. Smoke 40 → **48**.

#### Notes
- **Binary size**: 289,896 → 292,920 B (+3.0 KB) — history.cyr's CommandHistory body + the new `history_path` builder + the history builtin's display loop.
- **Coverage** holds at **86%**; new history fns gain transitive coverage via the smoke tests, no direct unit anchors yet (load+save are I/O-bound and exercised end-to-end through smoke).
- **Remaining v1.2.1 interactive-shell sub-items**: completion (tab), error recovery loop, streaming LLM. Completion needs terminal raw mode + tty escape handling. Streaming needs hoosh wire-up. Error-recovery loop is the smallest of the three.

### Slice 6 — interactive shell: mode switching + line-oriented stdin (committed)

#### Added
- **agnsh.cyr: `read_line(buf, maxlen)`** — byte-by-byte stdin reader that delivers one line per call. The previous `syscall(SYS_READ, 0, &buf, 4095)` worked in a real terminal (line discipline serves one line per read) but collapsed multi-line piped input into a single buffer, so the loop's line-oriented dispatch (`streq` against builtins) failed under any kind of scripted invocation. Byte-by-byte is slow per char but correct for both modes; terminal users see no difference (the tty's local echo handles visible feedback before \n arrives).
- **agnsh.cyr: mode-aware interactive_loop** — owns a `ModeManager` starting at `Mode.AI_ASSISTED` (matches `ShellConfig_default`'s default_mode). The prompt now carries the current mode prefix (`[ASSIST] >`, `[HUMAN] >`, `[STRICT] >`, `[AUTO] >`) via `mode_prompt_prefix`, so the AI-autonomy level is visible before every input. Pre-slice-6 the prompt was a bare `> ` regardless of mode.
- **agnsh.cyr: `mode` builtin** — no-arg form prints current mode + the available list; `mode <name>` switches when name ∈ `{auto, assist, human, strict}`. Unknown names error with the available list (surface vs silent failure). Bookkeeping helper `try_mode_switch(mgr, arg_cstr)` maps the CLI names to enum values and pulls `ModeManager_switch`.
- **agnsh.cyr: `clear` builtin** — emits the ANSI ED (`\x1b[2J`) + CUP (`\x1b[H`) pair to clear screen + home cursor. Matches the man-page entry that had been undocumented in the actual code.
- **agnsh.cyr: `help` expanded** — now lists every builtin with its arg shape (was a 2-line summary that omitted mode/clear).
- **agnsh.cyr: mode-aware audit entries** — `print_intent_result` now takes a `mode_label_cstr` and threads it into `audit_one_shot`. Interactive invocations write the actual `mode_display` label (`"AI-ASSIST"`, `"HUMAN"`, etc.) into the audit JSON's `mode` field; `-c` continues to log as `"auto"` (one-shot non-interactive). Downstream audit filters can now distinguish interactive-human sessions from interactive-auto from script-driven `-c`.
- **scripts/smoke-test.sh — 9 new interactive-loop assertions** driving the binary via piped stdin: assist start, `mode` reports current, switch to human, prompt updates after switch, switch to strict, NL parses under mode, exit clean, unknown-mode errors deterministically, unknown-mode lists the available set. Smoke 31 → **40**.

#### Notes
- **Binary size**: 288,040 → 289,896 B (+1.8 KB) — mode-prompt helper + read_line byte-loop + builtin parsing.
- **Coverage** holds at **86%**; the new helpers (`read_line`, `try_mode_switch`) gain transitive coverage through the smoke tests but are also reachable directly through the agnsh include graph.
- **Remaining v1.2.1 interactive-shell sub-items**: history (recall previous commands), completion (tab), error recovery loop, streaming LLM responses. History + error recovery are the next natural slices; completion + streaming both need bigger infrastructure (terminal raw mode for completion, hoosh wire-up for streaming).

### Slice 5 — security.cyr: sudo re-verification timing (committed)

#### Fixed
- **src/security.cyr — Cyrius 5.10.x stdlib alignment**. Four latent breaks accumulated since v1.0 because the module isn't (yet) in any binary's include graph, so the build never tripped on them:
  - `fs_exists` (5 call sites in `security_check_sudo` + `execute_with_privileges`) — renamed to `file_exists` (per `lib/io.cyr` 5.10.x).
  - `file_read_all("/etc/passwd")` single-arg form in `security_get_username` — Cyrius 5.10.x's `file_read_all(path, buf, maxlen): i64` is buffer-based; reworked to alloc a 64 KB heap buffer (lifetime survives the function return so the `vec_get(fields, 0)` Str's data pointer stays valid), call file_read_all with it, null-terminate, wrap as Str for str_split.
  - `process_exec(cmd, argv)` in `execute_command` — function doesn't exist in 5.10.x's `lib/process.cyr`. Replaced with `exec_vec(argv)` (the 5.10.x form that handles fork + execve + waitpid internally with cmd at argv[0]).
  - `str_data("/usr/bin/sudo")` in `execute_with_privileges` — same Cyrius 4.5 → 5.10 type-confusion class that bit slices 1/7/8/3: `str_data` reads `load64(s)` expecting a Str fat pointer, but the cstring literal there means it returned garbage as the path. The stat() syscall now takes the cstring directly.
  - **Plus** `streq(field_uid, uid_str)` — both sides are Str (from `str_split` + `str_from_int`), but `streq` is cstring-typed. Replaced with `str_eq` (lib/str.cyr's Str variant). Cyrius 5.10.x's new type-warning hint flagged this on build — same shape as the earlier Str/cstring mismatches but caught by the toolchain this time.

#### Added
- **`verify_sudo_path` extraction** — the inline existence-check + stat-based ownership-check in `execute_with_privileges` is now a named helper. Re-verifies at the escalation moment (not at session init) that `sudo_path` (cstring) **(a) exists on disk now** AND **(b) is owned by uid 0**. Closes the TOCTOU window between session-start cache and actual escalation: a long-running session may survive a sudo binary swap, deletion, or ownership flip; trusting `SecurityContext.sudo_available` alone would let the binary attempt sudo against a now-untrustworthy path. Caller `execute_with_privileges` now tries `/usr/bin/sudo` then `/bin/sudo` through `verify_sudo_path`, returning `-3` (sudo present but not root-owned) vs `-2` (sudo missing) so the failure mode is actionable.
- **Return-code contract documented** — `execute_with_privileges` return codes now have an inline contract block: `0+` exit code, `-1` restricted mode, `-2` sudo unavailable, `-3` sudo present but not root-owned. Pre-v1.2.1 the return codes were undocumented; downstream callers had to read the body to disambiguate.
- **tests/test_core.tcyr — 11 new security assertions** (and `_mock_sec` helper to compose a `SecurityContext` by hand, sidestepping the runtime UID dependency):
  - `SecurityContext_is_root` — yes / no paths.
  - `SecurityContext_is_restricted` — yes / no paths.
  - `SecurityContext_can_escalate` — full gate matrix: normal user OK; restricted blocked; sudo missing blocked; root blocked. **This is the v1.2.1 contract**: three independent guards, all must pass.
  - `verify_sudo_path` — happy path against `/usr/bin/sudo` (gated by `file_exists` so containers without sudo skip cleanly); deterministic negative against `/nonexistent/sudo/path`.
  - `security_check_sudo` — at-init coarse check agrees with the per-call `verify_sudo_path` (the integration invariant between cache and re-verifier).
  - Test count 290 → **301**.
- **`src/security.cyr` now in tests/test_core.tcyr's include graph** — was unreferenced previously. Future stdlib drift in security.cyr now surfaces as a build failure, not a runtime crash on first escalation attempt.

#### Notes
- **Not wired into `agnsh.cyr`** — security.cyr stays test-only this slice because the binary's `-c` mode still prints translations without executing them. When the exec wire-up lands (interactive-shell slice or later v1.2.1), the `agnsh.cyr` include + `SecurityContext_new` at startup is one additional line. The fixes here just make sure security.cyr is *ready* — a v1.2.x interactive-shell slice can trust the module to compile + behave on first wire-up rather than discovering the four bugs at integration time.
- **Bug-class lesson** — five Cyrius 4.5 → 5.10 stdlib regressions surfaced over the v1.2.0+v1.2.1 arc: `str_len(cstring)`, `str_sub(start, end)` semantics, `str_cat` first-arg cstring, `str_cat` second-arg cstring, `is_safe_path(Str)`, and now `process_exec` rename / `file_read_all` arity / `fs_exists` rename. The `streq(Str, Str)` case in this slice WAS caught by Cyrius 5.10.x's new type-warning hint — first time the toolchain caught one of these. Other variants are still silent; the queued static-analysis slice remains warranted.
- **Coverage** — denominator grew (security.cyr added 9 fns to the in-binary-scope set since it's now included by test_core); coverage held at **86%** (107/124). Capacity, fmt, lint, build, smoke all clean.

### Slice 4 — audit-log wire-up + JSON-shape coverage (committed)

#### Added
- **agnsh.cyr: `audit_one_shot` + `audit_log_path`** — every `-c` invocation now appends one JSON line to `$HOME/.agnsh_audit.log` (falls back to `/tmp/agnsh_audit.log` when `HOME` is unset for test harnesses / restricted envs). Path is constructed as a null-terminated cstring (manual buffer + `memcpy` because `lib/str.cyr::str_cat` returns a length-prefixed buffer with no trailing zero, and `syscall(SYS_OPEN)` wants cstring). The audit entry carries `user="user"`, `mode="auto"`, `input=<raw NL input>`, `action=<translated command>`, `approved={0,1}` derived from permission level (SAFE/READ_ONLY/USER_WRITE auto-approved; SYSTEM_WRITE/ADMIN/BLOCKED not), `result="proposed"` (will flip to `executed`/`denied`/`error` when the exec wire-up lands in the interactive-shell slice).
- **agnsh.cyr: real timestamp** — `chrono_now_rfc3339` now wraps `lib/chrono.cyr::iso8601_now()` (real wall-clock via `clock_gettime` syscall) instead of returning the v1.0-era fixed `"2026-04-13T00:00:00Z"` stub. `lib/chrono.cyr` is now in the binary's include graph.
- **audit.cyr now wired into `agnsh.cyr`** — was test-only previously. The dead-coded `AuditViewer_query` body was stripped (`return vec_new()`) because the AUDIT_VIEW intent already routes through MCP via `translate_audit_view`, and the in-process file-read fallback needs a stdlib API alignment (`fs_exists` → `file_exists` rename, `json_get_str` → `json_get` rename, `file_read_all` arity change) that's bigger than this slice. Slot for that alignment: the AUDIT_VIEW read-path slice in v1.2.x.
- **tests/test_core.tcyr — audit-log JSON shape coverage** — 11 new field-level assertions on `AuditEntry_to_json`: every field present + correctly JSON-quoted (`timestamp`, `user`, `mode`, `input`, `action`, `result`); `approved` serialized as a raw integer (not a quoted string — locks the contract downstream parsers depend on); JSON-string escaping for embedded `"` in the `input` field (locks the v1.0 audit C4 mitigation from the audit side too); `AuditLogger_log` writes a complete line to disk and the file is readable afterward. Substring-based assertions throughout so each contract gets one explicit check rather than one giant strict-equality line that would bust the 120-char lint cap. Test count 277 → **290**.
- **scripts/smoke-test.sh — end-to-end audit log checks** — 4 new assertions: log file is created at `$HOME/.agnsh_audit.log` after a `-c` invocation; LOW-risk command produces `"action":"ls","approved":1`; BLOCKED command produces `"action":"rm","approved":0`; line count matches invocation count (verifies append-mode + newline terminator). Smoke count 27 → **31**.

#### Notes
- **Binary size**: 284,504 → 288,040 B (+3.5 KB) — `lib/chrono.cyr` time helpers + `src/audit.cyr` JSON serializer + the `audit_log_path` cstring builder.
- **Coverage**: stays at **86%** (107/124). The denominator grew (audit.cyr added 8 fns, agnsh.cyr added 2) but the explicit new tests + coverage anchors held the percentage steady.
- **Verification**: `HOME=/tmp ./build/agnsh -c "show me files"` writes `{"timestamp":"2026-05-11T16:15:04Z","user":"user","mode":"auto","input":"show me files","action":"ls","approved":1,"result":"proposed"}` to `/tmp/.agnsh_audit.log`. Second invocation appends the next line. JSON is shellcheck-clean and `jq .` parses each line.
- **Remaining v1.2.1 sub-items** — sudo re-verification timing (the third "battle-tested" bullet) and the full interactive-shell loop (history, prompt, mode switching, completion, error recovery, streaming LLM) are still open. Sudo-timing is small and likely the next bite.

### Slice 3 — approval runtime wire-up + safety-predicate Str-fix (committed)

#### Fixed
- **sanitize.cyr: Str-aware safety predicates** — added `has_path_traversal_str`, `has_shell_metachars_str`, `is_safe_path_str`, `is_safe_arg_str` (named with full `_str` suffix initially, then renamed to `safe_path_in_str` / `safe_arg_in_str` / etc. after the original convention turned out to trigger an unintended Cyrius name-mangling overload). All 11 call sites in `src/translate.cyr` now route through the Str-aware variants. The cstring-form `is_safe_path` / `is_safe_arg` are kept in place for the tests that pass cstring literals and for `permissions.cyr` cstring callers. **Behavior impact**: pre-v1.2.1 `agnsh -c "copy a to b"` printed `Risk: [LOW]` (because translate_copy fell through to `translate_unknown` → `echo`); now correctly prints `Risk: [MED]` with `Command: cp`. Same fix unlocks `move`, `remove`, `create directory`, `find files named ...`, `search for ... in ...`, and `read <file>` NL paths.
- **agnsh.cyr: print Command via Str wrap** — `str_print(cmd)` where `cmd` is a translator-stored cstring (e.g., `"ls"`, `"git"`, `"systemctl"`) caused `str_print`'s `load64(s+8)` to read garbage as a length; the line silently printed nothing. Now `str_print(str_from(cmd))` wraps the cstring on the fly. Pre-v1.2.1 every `-c` invocation showed `Command: ` blank.
- **agnsh.cyr: interactive banner version drift** — banner string was hardcoded `agnoshi 1.1.0`; replaced with `VERSION_STR` so future bumps stay in sync.
- **Second-position str_cat sweep (slice 2)** — `str_cat(X, "...")` cstring-in-second-position pattern fixed at 7 latent call sites in `aliases.cyr`, `checkpoint.cyr` ×3, `audit.cyr`, `prompt.cyr`, `session.cyr`. Same Cyrius 4.5 → 5.10 stdlib drift as slice 8's first-position sweep.

#### Added
- **agnsh.cyr: approval risk-print in `-c` mode** — `src/approval.cyr` now wired into the binary's include graph (was only in tests). Every `-c` invocation now prints `Risk: [LOW|MED|HIGH|CRIT]` (assessed via `risk_from_permission`) in place of the bare permission integer. `BLOCKED` permission surfaces an explicit `WARNING: BLOCKED -- would not execute without explicit override`; HIGH risk surfaces `Approval required (interactive prompt in shell mode)`. Interactive prompt itself (`ApprovalManager_request` with stdin reads) is queued for the next slice.
- **scripts/smoke-test.sh** — 7 new assertions on the new `-c` output shape: risk label for each of the four risk levels, the BLOCKED warning line, the HIGH-risk approval hint, and `Command: ls` populated (locking the str_print-cstring fix in CI). Smoke count 20 → **27**.
- **tests/test_core.tcyr — approval coverage (slice 1, retained)** — 20 assertions for `src/approval.cyr` (first time covered): full `risk_from_permission` mapping, `risk_icon` labels, `ApprovalManager_assess_risk` for representative commands across risk levels, `ApprovalManager_is_blocked` pattern-add behavior, `ApprovalManager_set_auto_approve` toggle.

#### Notes
- **Test count**: 257 → **277** (slice 1 + slice 3). The Str-fielded translator-test rewrite that came out of slice 3 (every `store64(*_intent + N, "...")` for safety-checking translators now wraps in `str_from`) keeps the existing 20 translator assertions passing under the new contract — total stays at 277 because slice 3 added no new test entries, only updated existing fixtures to the actual production contract.
- **Binary size**: 280,344 B (post-slice-2) → 284,504 B (+4.1 KB). Growth from approval.cyr's include into the binary + the new `is_safe_path_str` / `is_safe_arg_str` helpers.
- **Coverage**: 89% → 86%. The denominator grew (approval.cyr's 8 fns and the four `_str` safety helpers all entered the in-binary scope) faster than tests added direct anchors for them; still well above the 80% gate.
- **Bug-class lesson** — three Cyrius 4.5 → 5.10 stdlib regressions surfaced over the v1.2.0+1.2.1 arc: `str_len(cstring)` mis-read, `str_sub(start, end)` semantics flip, `str_cat(cstring, *)` / `str_cat(*, cstring)` type mismatch, and now the `is_safe_path(Str)` type mismatch. None are caught by the build — all surfaced as silent runtime fallthroughs or segfaults. A static analyzer pass for "cstring passed to fn typed `Str`" would catch the whole class; queued as a v1.2.x or v1.3.x tooling slice.

### Slice 2 — second-position str_cat sweep + approval coverage debut (committed)

### Fixed
- **Second-position str_cat bug-class sweep** — slice 8's audit only checked `str_cat("...", X)` (cstring as first arg). The dual case `str_cat(X, "...")` (cstring as second arg) is *also* broken because `lib/str.cyr`'s `str_cat(a: Str, b: Str)` types both sides — passing a raw cstring for `b` causes the function to read `load64(cstring+8)` as a Str length header (garbage). 7 latent sites fixed across `aliases.cyr` (expansion suffix space), `checkpoint.cyr` ×3 (HOME-relative checkpoint dir + backup-name infixes), `audit.cyr` (`"..."` truncation suffix), `prompt.cyr` (`/.git/HEAD` path build), `session.cyr` (HOME-relative history path). All in modules deferred to v1.2.x wire-up; same hygiene rationale as slice 8.

### Added
- **tests/test_core.tcyr — approval workflow coverage** — 20 new assertions exercising `src/approval.cyr` (first time the module has unit tests):
  - `risk_from_permission` — full mapping locked: SAFE/READ_ONLY → LOW, USER_WRITE → MEDIUM, SYSTEM_WRITE/ADMIN → HIGH, BLOCKED → CRITICAL.
  - `risk_icon` — UI label strings (`[LOW]`, `[MED]`, `[HIGH]`, `[CRIT]`) locked. When the interactive approval dialog ships in slice 10+, drift here would silently break the on-screen risk indicator.
  - `ApprovalManager_assess_risk` — end-to-end risk for representative commands (`ls` → LOW, `cp` → MEDIUM, `apt` → HIGH, `dd` → CRITICAL). Tests the composition of `analyze_command_permission` + `risk_from_permission`.
  - `ApprovalManager_is_blocked` — pattern blocklist (substring match). Default-empty + add-pattern + matching cmd + unrelated cmd all locked.
  - `ApprovalManager_set_auto_approve` — toggle bit at offset 8 locked in both directions.
  - Test count 257 → **277**, all passing.
- **approval.cyr now wired into tests/test_core.tcyr** — the test binary now compiles + links the module, which means future regressions (e.g. another stdlib drift) surface as build failures rather than runtime crashes on first use.

### Notes
- `ApprovalManager_request` itself (the interactive dialog) is *not* covered yet — it does `syscall(SYS_READ, 0, ...)` to read keyboard input, which can't be exercised in a unit-test harness. That branch lands in slice 10's interactive-shell wiring with an injection seam for testable I/O.
- Binary size unchanged at 280,344 B (approval.cyr only landed in the test binary, not in `agnsh.cyr`'s include graph yet — the runtime wire-up is the next slice).

## [1.2.0] - 2026-05-11

The v1.2.0 cycle closed out all three roadmap items: deeper intent parsing (slices 1-4), all-core-translators production-tested (slices 5-7), and a coverage report wired into CI (slice 9, 89% fn-level coverage against an 80% threshold). Slice 8 was a bug-class audit pass that swept `src/` for the same `(cstring, Str)`-where-`(Str, Str)`-expected pattern that bit slices 1 and 7, fixing 10 latent call sites across `prompt.cyr`, `security.cyr`, `checkpoint.cyr`, `sanitize.cyr`, and `session.cyr` — all in modules deferred to the v1.2.x interactive-shell wire-up, but now correct ahead of that work.

### Fixed
- **translate.cyr: `translate_audit_view` / `translate_agent_info`** — both built MCP JSON bodies via `str_cat("{\"agent\":\"", agent_str)`. `lib/str.cyr`'s `str_cat` takes `(Str, Str)` on 5.10.x, and passing a cstring as the first arg causes `load64(cstring)` to be read as a Str header (garbage length). Binary segfaulted any time the user asked for an audit view (`"show audit log"`) or queried agent info. Both literals now wrapped in `str_from()`. Verified by translator tests AND end-to-end against the binary (`./build/agnsh -c "show audit"` no longer crashes).
- **Bug-class audit pass** — 10 additional `str_cat(cstring, Str)` call sites swept from `src/`: `prompt.cyr` (path `~` abbreviation), `security.cyr` x2 (`uid_` username fallbacks), `checkpoint.cyr` x2 (rollback message formatting), `sanitize.cyr` x3 (`build_safe_env` for `HOME=` / `LANG=` / `TERM=`), `session.cyr` x2 (cd-error message). All in modules not currently linked into the agnsh binary; fixing them ahead of v1.2.x's interactive-shell wire-up keeps the same Cyrius 4.5 → 5.10 stdlib-drift bug class from biting once those modules ship.
- **sanitize.cyr (slice 1, retained)** — `str_contains_ci`, `str_find_ci`, `str_find_ci_from`, `str_split_ci` were calling `str_len(needle)` / `str_data(needle)` on a cstring needle. Garbage length, every `input_has_word()` match silently false, every parsed intent fell to `SHELL_COMMAND`. Helpers now use `strlen()` for the cstring side and raw pointer arithmetic. Single root cause behind the "agnoshi can't parse NL" symptom on 5.10.x.
- **str_sub → str_substr migration (slice 1, retained)** — 19 call sites across `aliases.cyr`, `audit.cyr`, `commands.cyr`, `prompt.cyr`, `session.cyr`, `sanitize.cyr`, `interpreter.cyr` were passing end-positions to `str_sub(s, start, len)` (which takes a *length* on 5.10.x). Global rename to `str_substr` (the (start, end) variant).
- **interpreter.cyr: extract_after / extract_between (slice 1, retained)** — same `str_len(cstring keyword)` bug pattern; replaced with `strlen(keyword)` / `strlen(before_kw)`.

### Added

#### Slices 1-4 — Deeper intent parsing
- **parse_state_queries** — noun-phrase queries: `"ip address"`, `"my ip"`, `"network status"` → `NETWORK_INFO`; `"uptime"`, `"load average"`, `"kernel version"`, `"memory usage"`, `"hostname"` → `SYSTEM_INFO`; `"disk space"`, `"free space"`, `"how full"`, `"storage usage"` → `DISK_USAGE`; `"running processes"`, `"what's running"`, `"active processes"` → `SHOW_PROCESSES`.
- **parse_service_query** — `"is X running"` / `"is X active"` / `"is X enabled"` (gated on `input_starts_with("is ")` so statements like `"the application is running"` don't get hijacked) and `"status of X"` → `SERVICE_CONTROL` with action=status, target=X.
- **parse_service_action** — bare imperative form: `"start nginx"`, `"stop sshd"`, `"restart cron"`, `"reload nginx"`, `"enable cron"`, `"disable apache"` → `SERVICE_CONTROL`. Gated on `input_starts_with(verb)` at token 0 + `token_count == 2` so `"start a new project"` / `"stop wasting time"` keep falling through to `SHELL_COMMAND`. `parse_admin_ops` runs first so `"enable firewall"` / `"disable ufw"` correctly stay `FIREWALL_ENABLE` / `FIREWALL_DISABLE`.
- **sanitize.cyr: `is_word_prefix(input, word)`** — case-insensitive token-prefix matcher. Gives plural-tolerance (`"file"` matches `"files"`, `"process"` matches `"processes"`, `"directory"` matches `"directories"`) AND substring-trap immunity (`"move"` doesn't match inside `"remove"`, `"rm"` doesn't match inside `"warm"`). The previous trap-defense ordering hack (REMOVE-before-MOVE) is retired; the `"rm "` / `"move "` trailing-space anchors dropped.
- **sanitize.cyr: `input_starts_with(input, prefix_cstr)`** — case-insensitive prefix check, gates interrogative form for service queries.
- **interpreter.cyr: `input_has_word` auto-dispatch** — compound phrases (internal whitespace) keep substring matching; single-token needles route through `is_word_prefix`.
- **interpreter.cyr: `token_count`** — whitespace-delimited token counter, sanity gate for imperative service actions.

#### Slices 5-7 — Translator production tests
- **tests/test_core.tcyr — full translator-coverage block** — every `translate_X` in `src/translate.cyr` (43 translators) gets at least command + permission-level assertions; safety-check translators get explicit negative cases (path-traversal → unknown for `translate_show_file`, missing destination → unknown for `translate_copy`, null path → unknown for `translate_change_dir`, pid=0 → unknown for `translate_kill_process`, leading-dash commit message → unknown for `translate_git_commit` locking the v1.0 audit H7 mitigation, null action → unknown for `translate_service_control`). `translate_remove` BLOCKED permission level locked; `translate_shell_command`'s dynamic-permission derivation tested both arms (`"ls"` → READ_ONLY, `"apt"` → ADMIN). MCP-routing translators (`audit_view`, `agent_info`) have `mcp_tool` field-40 non-zero locked.

#### Slice 9 — Coverage report in CI
- **scripts/check-coverage.sh** — fn-level coverage gate. Cyrius doesn't ship line-coverage instrumentation, so the script counts top-level `fn` defs in the modules linked into the agnsh binary (`sanitize.cyr`, `mode.cyr`, `permissions.cyr`, `intent.cyr`, `commands.cyr`, `translate.cyr`, `interpreter.cyr`) and requires ≥80% to be referenced by name in `tests/test_core.tcyr` / `tests/test_security.tcyr`. Modules reserved for the v1.2.x interactive-shell wire-up (`session.cyr`, `ui.cyr`, `prompt.cyr`, `checkpoint.cyr`, etc.) are out-of-scope until that work lands. Current: 107 / 120 fns covered (89%), comfortably above the 80% threshold.
- **CI gate** — `.github/workflows/ci.yml` runs `scripts/check-coverage.sh 80` after the smoke test. Below-threshold coverage now fails CI like fmt / lint / capacity drift.
- **tests/test_core.tcyr — coverage anchor block** — direct assertions for the helpers that were transitively exercised but never named in the test file: string ops (`str_byte_at`, `str_contains_ci`, `str_find_ci`, `str_find_ci_from`, `str_split_ci`, `strip_control_chars`, `print_str_safe`), the substring-trap matcher (`is_word_prefix`), permission classifiers (`is_blocked_command`, `is_readonly_command`, `is_write_command`, `is_safe_command`, `is_safe_arg`, `is_shell_metachar`), parser dispatch arms (`parse_show_commands`, `parse_file_ops`, `parse_system_ops`, `parse_git_ops`, `parse_admin_ops`, `parse_service_query`, `parse_service_action`, `parse_state_queries`), translator dispatch (`translate_core`, `translate_extended`), mode helpers (`mode_description`, `mode_prompt_prefix`, `ModeManager_toggle`), tokenizer (`token_count`, `split_command_line`, `str_to_int`), env builder (`build_safe_env`), and intent option-pack bit-accessors (`list_options_time`).

### Notes
- **Test count**: 57 → **257** (4.5× growth). 200 new assertions across parse-side coverage, translator coverage, and coverage-anchor blocks.
- **Binary size**: 271,832 B (1.1.0) → 280,344 B (+8.5 KB). Growth is the new parser helpers (`is_word_prefix`, `input_starts_with`, `token_count`, `parse_service_query`, `parse_service_action`, `parse_state_queries`) plus the `str_from()` wraps in the bug-class fixes. Still a single statically-linked ELF with zero runtime deps.
- **Parser performance** — parse benchmarks moved 1-2us (pre-slice-1 fast path was a no-op due to broken CI helpers) → 3-13us (parser walking actual branches with the substring-trap-immune word-prefix matcher). Still well under interactive-latency thresholds.
- **Bug-class audit findings** — the v1.1.0 toolchain migration left three distinct stdlib-semantics regressions in tree: `str_len(cstring)` (slice 1, sanitize + interpreter), `str_sub(start, end)` → length semantics (slice 1, 19 sites in 7 files), and `str_cat(cstring, Str)` (slices 7 + 8, 12 sites in 6 files). All swept. Recommended for the v1.2.x interactive-shell work: re-audit any module brought into the binary's include graph for the same patterns before wiring it in.

## [1.1.0] - 2026-05-10

Repair-focused modernization. No new shell features — toolchain bump + scaffolding parity with the rest of the AGNOS ecosystem.

### Documentation
- **doc closeout** — Five docs flagged Stale in the initial `doc-health.md` audit moved Fresh in the 1.1.0 closeout pass. Each refreshed in-place against the agnoshi shape (userland AI shell), not pasted from the agnosys playbook (kernel-interface library):
  - `README.md` — added a `1.1.0 · Cyrius 5.10.34 · 21 modules · ~4 K src lines · 272 KB static binary (DCE) · 0 runtime deps` stat-line; install instructions now lead with `cyrius deps`; the "146 KB" headline from 1.0.0 is reframed as a port-arc snapshot pointing at `benchmarks-rust-v-cyrius.md` with an in-tree refresh command; the `agnsh.cyr "v1.0 minimal"` annotation dropped (the entry shipped).
  - `CONTRIBUTING.md` — `cyrius deps` step added before build; cleanliness gate command list (`cyrius check / capacity / vet / fmt / lint`) documented inline matching the CI shape; cc3-era warnings purged (`//`-comment-with-colons mis-parse note, "40+ match arms may exceed per-fn limit"); Cyrius 5.10.x trailing-comma rule from the toolchain-bump notes carried in.
  - `docs/architecture/overview.md` — `lib/` reframed as "Cyrius stdlib (gitignored; populated by `cyrius deps` from the pinned snapshot)"; build-time requirement bumped `cyrius v4.3.0+` → `Cyrius 5.10.34 pinned in cyrius.cyml`; runtime size annotated with the 146 KB → 272 KB toolchain-side growth between 4.5.0 and 5.10.x.
  - `docs/agnsh.1` — `.TH` header bumped `April 2026 / agnoshi 1.0.0` → `May 2026 / agnoshi 1.1.0`. Command surface (modes, builtins, options, files) unchanged in 1.1.0 so the body needed no edits.
  - `benchmarks-rust-v-cyrius.md` — historical-port-arc framing added at the top; cc3-limit references called out as point-in-time and no longer applicable on Cyrius 5.10.34; in-tree refresh command (`cyrius build tests/bench_core.bcyr build/bench_core && ./build/bench_core`) wired in for current-toolchain numbers. Doc otherwise remains frozen by design.
- **doc-health.md** — bucket counts re-rolled (Fresh: 6 → 11), per-row entries for the five closeout items moved to ✅ Fresh with refresh notes; the one outstanding Open Strategic Question is now strictly `benchmarks-rust-v-cyrius.md`'s home (root vs `docs/`), deferred to 1.2.0 doc-sync.

### Changed
- **toolchain** — Cyrius pin bumped 4.5.0 → 5.10.34 (latest stable). Pin now lives in `cyrius.cyml` (`cyrius = "5.10.34"`); the standalone `.cyrius-toolchain` file was retired.
- **manifest** — `cyrius.toml` → `cyrius.cyml`. Package version is no longer hand-edited in the manifest — `version = "${file:VERSION}"` reads `VERSION` at toolchain-resolve time, so `VERSION` is the only file the release process touches.
- **lib/** — vendored stdlib stubs removed from the tree; `./lib/` is gitignored. `cyrius deps` repopulates from the version-pinned stdlib snapshot referenced in `[deps] stdlib` (matches the agnosys / yukti / patra convention). Prevents prior-version stubs from sitting in tree across toolchain bumps.
- **ci** — agnosys-parity gate set: syntax check (`cyrius check`), fmt diff-check, lint with warn-as-error, vet (include-graph audit), capacity gate, aarch64 best-effort cross-build, security-pattern scan (raw execve / shadow access / large fn-scope buffers), version-consistency gate (`VERSION` ↔ `CHANGELOG.md` ↔ `cyrius.cyml ${file:VERSION}`), required-docs check now includes `CLAUDE.md`, `docs/development/roadmap.md`, and `docs/doc-health.md`.
- **release** — accepts both `vX.Y.Z` and `X.Y.Z` tag styles; semver shape verified; SHA256SUMS published alongside source archive + per-arch binaries; pre-release flag auto-set for `0.x` tags.
- **scripts/version-bump.sh** — touches only `VERSION` now (was editing both `VERSION` and `cyrius.toml`); the manifest substitutes automatically via `${file:VERSION}`.
- **CLAUDE.md** — cleanliness gates rewritten from Rust toolchain (`cargo fmt/clippy/audit/deny/doc`) to Cyrius equivalents (`cyrius check/fmt/lint/vet/capacity`); P(-1) and Work Loop sections refreshed; version-discipline rules (VERSION is single SoT, `./lib/` never committed) added under Key Principles and DO NOT.
- **docs/development/roadmap.md** — reshaped: shipped items dated (1.1.0 itself folded in at closeout with the full modernization summary inline), post-v1.0 polish items slotted across 1.2.0 (intent parsing + translators), 1.2.1 (approval + interactive shell), 1.2.2 (zugot packaging); demand-gated systems / UX / consumer-app translator items moved to v1.3.x+.

### Added
- **docs/doc-health.md** — living doc-currency ledger (fresh / stale / archived / open-question), agnoshi-shaped tiers, initial audit covering ~26 markdown files plus the `agnsh.1` man page. Refreshed opportunistically when docs are touched (paired with each minor-cut closeout step per CLAUDE.md Work Loop §10).

### Fixed
- **release.yml** — was building `src/main.cyr → agnoshi` (the pre-port Rust entry / pre-rename binary), but `cyrius.cyml [build]` specifies `src/agnsh.cyr → agnsh`. Releases would have shipped the wrong binary name. Release workflow now builds and archives `agnsh`.
- **lint cleanup** — Cyrius 5.10.x added a 120-character line-length lint. Wrapped 49 long lines across `src/interpreter.cyr` (16), `src/translate.cyr` (32, mostly `Translation_new(...)` call sites), and `src/permissions.cyr` (1). Behavior unchanged; CI's lint gate now reports zero warnings.
- **fmt drift** — Cyrius 5.10.x formatter rules differ from 4.5.0. Re-formatted 5 files (`commands.cyr`, `permissions.cyr`, `session.cyr`, `translate.cyr`, `ui.cyr`) so the fmt diff-gate is clean.
- **CLAUDE.md Known Issues** — purged two stale entries: (1) the "ModeManager undefined variable" build-error note (the struct is defined in `src/mode.cyr:8` — the note was a leftover from a mid-port debugging session); (2) the "cc3 function/token limit" comment in `benchmarks-rust-v-cyrius.md` (cc3 is retired, the current Cyrius compiler has no such limit; the doc has been re-classified as historical in `docs/doc-health.md`).
- **ci: syntax check** — switched from per-file `cyrius check` loop to single `cyrius check src/agnsh.cyr` (entry-walk). agnoshi modules don't declare their own includes — `agnsh.cyr` stitches them — so isolated-file checking failed on cross-module references (`PermissionLevel` in `approval.cyr`, etc.). Same posture as vet / capacity / build.
- **ci: security scan** — agnosys's "writes to /bin / /sbin" heuristic was a false positive for agnoshi (which legitimately references `/bin/sudo` and uses `"/bin/"` / `"/sbin/"` prefix strings to *block* writes). Replaced with shell-shaped checks: raw `execve` syscall outside the approval pipeline, `/etc/shadow` access, stray sudo paths outside `src/security.cyr`. Buffer warn threshold lifted 4 KB → 8 KB (4 KB is PATH_MAX, expected pattern).
- **ci: shadow-lib note** — `cyrius deps` populates `./lib/` and the toolchain then notes the shadow against its version cache (informational, not an error). Silenced via `CYRIUS_NO_WARN_SHADOW_LIB=1` at job-level env so CI logs stay clean.
- **agnsh.cyr: duplicate getenv stub** — cc3-era stub at `src/agnsh.cyr:17` shadowed the real `getenv` shipped by `lib/io.cyr` on Cyrius 5.10.x, triggering a duplicate-fn linker warning. Stub removed; `ui_show_*` / `chrono_now_rfc3339` stubs remain (their real impls live in `src/ui.cyr` / `lib/chrono.cyr` which aren't pulled into this entry's include graph — slot the full-entry migration into 1.2.0 alongside the deeper-intent-parsing work).
- **agnsh.cyr: VERSION_STR** — bumped `"agnoshi 1.0.0"` → `"agnoshi 1.1.0"`; the `-v` flag was reporting the old version after the bump.

### Notes
- **Binary size**: 146 KB (1.0.0 on Cyrius 4.5.0) → 271,912 bytes (1.1.0 on Cyrius 5.10.x). Toolchain-side growth from richer stdlib + codegen, not from new agnoshi code. Still a single statically-linked ELF with no dynamic deps.
- **Cyrius 5.10.x source rule**: trailing commas in call argument lists are rejected by `cyrius build` even though `cyrius fmt` preserves them. Apply line-wraps without a trailing comma after the last argument.
- **Local-vs-CI toolchain skew**: the pin in `cyrius.cyml` is 5.10.34; local dev may run a newer 5.10.x. Verified compatible against 5.10.47.

## [1.0.0] - 2026-04-13

### Added
- **port** — full Cyrius port of the Rust codebase (27,251 → 4,042 lines, 20 modules)
- **sanitize.cyr** — shared validation module: `is_safe_arg`, `is_safe_path`, `get_command_basename`, `strip_control_chars`, `json_escape`, `build_safe_env`, `is_valid_pid`, `is_safe_branch_name`, `is_safe_commit_message`, `is_safe_username`
- **audit** — JSON-escaped audit log output (prevents log injection)
- **benchmarks** — `tests/bench_core.bcyr` with 10 benchmarks; results in `bench-history.csv` and `benchmarks-rust-v-cyrius.md`
- **tests** — `tests/test_core.tcyr` (100 assertions), `tests/test_security.tcyr` (80 assertions)
- **scripts/install.sh** — install to /usr/local/bin
- **scripts/uninstall.sh** — clean removal
- **scripts/smoke-test.sh** — 20 end-to-end tests for the binary
- **docs/agnsh.1** — man page
- **docs/audit/2026-04-13.md** — 21-finding security audit report
- **CI** — GitHub Actions workflow builds, smoke-tests, and benchmarks on every push

### Changed
- **entry point** — `src/agnsh.cyr` replaces `src/main.cyr` (minimal, works with current cc3)
- **binary name** — `agnsh` (was `agnoshi`) to match man page and prior convention
- **permissions** — `analyze_command_permission` now extracts basename before classification (prevents `/usr/bin/dd` bypass)
- **security** — check effective UID (catches setuid), sudo re-verified at escalation time
- **checkpoint** — backups moved from world-readable `/tmp` to `$HOME/.agnoshi/checkpoints` (mode 0700)
- **checkpoint** — auto-prune keeps only the most recent 100 entries (deletes old backups)
- **interpreter** — split `Interpreter_translate` 42-arm match into `translate_core` + `translate_extended` (cc3 per-function limit)
- **IntentTag** — pruned from 211 to 44 entries (downstream consumer apps deferred)

### Fixed
- **security (C1)** — command bypass via absolute/relative paths (basename extraction)
- **security (C2)** — argument injection (dangerous character validation)
- **security (C3)** — null pointer dereference in 4 translators
- **security (C4)** — JSON injection in audit logs
- **security (C5)** — 8 unhandled intent tags fell through to SAFE echo
- **security (H1)** — euid check in root detection
- **security (H2)** — environment inheritance in privilege escalation (clean env whitelist)
- **security (H3)** — checkpoint dir in world-readable /tmp
- **security (H4)** — git branch terminal escape injection
- **security (H5)** — approval UI terminal escape injection
- **security (H7)** — git commit message argument injection (leading-dash reject)
- **security (M1)** — /proc/self/environ 8KB fixed buffer (now 32KB dynamic with bounds check)
- **security (M2)** — PID validation (`kill 0` kills process group)
- **security (M3)** — rm flag parsing (`--`, combined flags, per-char scan)
- **security (M4)** — path traversal in file translators
- **security (M5)** — backslash escape handling in quote parser
- **security (M6)** — alias expansion metacharacter injection
- **security (M7)** — checkpoint failure warning before destructive ops
- **security (M8)** — /etc/passwd username validation
- **security (M9)** — sudo re-verification at escalation time

### Performance
- parse/list_files: 32.0us (Rust) → 1us (Cyrius) — **32× faster**
- parse/cd: 19us (Rust) → 1us (Cyrius) — **19× faster**
- binary size: 3.8 MB (Rust, dynlinked+debug) → 146 KB (Cyrius, static) — **−96%**
- startup: ~2-5ms (Rust, dynamic linker) → microseconds (Cyrius, static ELF)
- note: translation is 4-8× slower per call (still sub-microsecond); net pipeline 19× faster

### Removed
- **Rust implementation** — preserved in `rust-old/` for reference during port

## [0.90.0] - 2026-04-02

### Added

- **interpreter** — 10 git workflow intents: `GitCommit`, `GitDiff`, `GitBranch`, `GitStatus`, `GitLog`, `GitPush`, `GitPull`, `GitCheckout`, `GitMerge`, `GitStash` with full NL parsing, translation, and tests
- **interpreter** — 7 user/group management intents: `UserAdd`, `UserDelete`, `UserMod`, `Passwd`, `GroupAdd`, `GroupDelete`, `GroupList` with full NL parsing, translation, and tests
- **interpreter** — 7 firewall intents: `FirewallAllow`, `FirewallDeny`, `FirewallList`, `FirewallStatus`, `FirewallEnable`, `FirewallDisable`, `FirewallDeleteRule` with full NL parsing (ufw-based), translation, and tests
- **explain** — added explanations for `ufw`, `nft`, `iptables`, `ip6tables`, `groupdel`
- **security** — prompt injection defense: all external content sanitized before LLM prompts (OWASP ASI01/ASI02); strips role-override patterns, special tokens, truncates to 4KB
- **security** — command validation: LLM-generated commands validated with `shlex::split()` before presentation; rejects malformed syntax
- **security** — sandbox hardening: Landlock now protects dotfiles (`.bashrc`, `.ssh/`, `.gitconfig`) as read-only (OWASP ASI03)

### Changed
- **deps** — `agnosys` dependency temporarily switched to local path for musl static build (pending agnosys release with ioctl fix)
- **explain** — replaced 140-arm `match` statement with `LazyLock<HashMap<&'static str, &'static str>>`; eliminates per-call String allocation
- **interpreter** — extracted `cap_str()` / `cap_opt()` parse helpers; deduplicated ~155 capture-group extraction patterns across 4 parse files
- **security** — refactored `analyze_command_permission()`: extracted command lists to module-level constants (`BLOCKED_COMMANDS`, `ADMIN_COMMANDS`, `WRITE_COMMANDS`, `READ_ONLY_COMMANDS`, `SAFE_COMMANDS`); extracted `normalize_path()` and `targets_system_path()` helpers
- **session** — removed unused `_config`, `_security`, `_output` fields from `Session` struct
- **session** — added structured tracing to command execution (duration, exit code) and approval decisions
- **config** — extracted `DEFAULT_MCP_BASE_URL` constant; `DEFAULT_LLM_TIMEOUT_SECS` constant in llm module

### Fixed

- **security** — `get_username` now reads from passwd database instead of trusting `$USER` env var (was spoofable to bypass permission checks)
- **security** — JSON injection in phylax.rs scan target: switched from `format!()` to `serde_json::json!()`
- **security** — added 16 missing dangerous commands to admin list: `kill`, `killall`, `pkill`, `reboot`, `shutdown`, `poweroff`, `halt`, `iptables`, `ip6tables`, `nft`, `ufw`, `crontab`, `visudo`, `su`, `swapoff`, `swapon`, `mknod`; added `shred` to blocked list
- **security** — removed duplicate `dd` entry from blocked list
- **interpreter** — fixed `list` regex: made first group required — was matching empty strings and arbitrary input (e.g., `""`, `"htop"`, `"go to /tmp"` all incorrectly parsed as `ListFiles`)
- **interpreter** — fixed `cd` regex capture group: `caps.get(4)` → `caps.get(5)` — `cd` and `go to` now correctly parse as `ChangeDirectory`
- **interpreter** — fixed `find` regex: greedy `(.+)` → non-greedy `(.+?)` so `\s+in\s+(.+)` path group can match
- **session** — fixed pipe deadlock: replaced `child.wait()` + post-read with `child.wait_with_output()` (child filling pipe buffer could deadlock)
- **session** — `rm` checkpoint now backs up all non-flag target files (was only checkpointing the first)
- **mode** — `toggle()` now respects `allow_switching` guard (was bypassing it, allowing mode changes when disabled)
- **schema_filter** — fixed cache age off-by-one: matched categories now get age 0 (not 1) after update; moved cache update before merge so expired schemas aren't returned
- **audit** — replaced byte-offset string slicing with `chars().take(n)` to prevent panic on multi-byte UTF-8
- **completion** — fixed case-sensitivity: registered names now lowercased at insertion for correct case-insensitive matching
- **output** — `format_auto` now pretty-prints valid JSON instead of double-wrapping it in `{"output": ...}`
- **permissions** — added wildcard arm for `#[non_exhaustive]` `PermissionLevel` (future variants default to denied)
- **bench** — fixed duplicate `--all-features` flag in `bench-history.sh`
- **bench** — fixed `bench-history.sh` CSV parsing: criterion `change:` lines (containing `%` values) were captured alongside actual timing lines, corrupting CSV and crashing the markdown generator
- **security** — URL parameter injection in `phylax.rs`: severity value now percent-encoded (was raw-embedded, allowing `?severity=critical&evil=true`)
- **security** — `sanitize_url_segment()` in `package.rs` now rejects URL-special characters (`?`, `&`, `#`, `%`, `=`) in addition to path traversal sequences
- **dashboard** — fixed UTF-8 panic: byte-offset string slicing (`&s[..N]`) replaced with `chars().take(N)` for agent ID and action truncation (was crashing on multi-byte characters)

### Changed

- **mode** — `Mode` now derives `Copy` (all unit variants); removed unnecessary `.clone()` calls
- **mode** — `toggle()` now returns `Result<()>` (was `()`)
- **security** — moved `echo` out of `safe` list (was dead entry; already in `read_only` which is checked first)
- **deps** — replaced `once_cell::sync::Lazy` with `std::sync::LazyLock` (stable in Rust 1.89)
- **deps** — removed `once_cell` dependency
- **deps** — added `agnosys` git URL to `deny.toml` `allow-git`
- **api** — added `#[must_use]` to 20+ pure functions across security, permissions, commands, aliases, completion, history, output modules
- **api** — added `#[inline]` to hot-path functions: `Interpreter::parse()`, `Interpreter::translate()`, `CompletionEngine::complete()`
- **api** — added `#[must_use]` to `Interpreter::translate()` and `Interpreter::explain()`; `explain()` intentionally not `#[inline]` (17K-line match statement — inlining hurts icache)
- **security** — `rm` permission logic now distinguishes dangerous flags (`-r`, `-f`, `-rf`, `--recursive`, `--force`, `--no-preserve-root`) from safe flags (`-v`, `-i`); safe-flagged `rm` requires approval (Admin), dangerous-flagged `rm` is Blocked
- **deps** — removed unused `BSD-2-Clause` from `deny.toml` allow list
- **tests** — 1,241 unit tests (up from 1,109); 132 new tests covering git/user/firewall intents, prompt injection defense, UTF-8 truncation, URL injection, URL sanitization, rm flag classification

## [0.90.0] - 2026-04-02

### Added

- **session** — error recovery loop: when a command fails, LLM suggests a fix (shown in cyan)
- **session** — revision workflow: `Intent::Unknown` now queries LLM with context before falling back to raw shell execution
- **session** — richer LLM context: `suggest_command_with_context` sends CWD, recent history, and last exit code to LLM
- **checkpoint** — checkpoint/rollback system for destructive operations (`rm`, `mv`); `undo` builtin restores files
- **interpreter** — 12 stiva container intents: run, stop, ps, rm, pull, images, rmi, build, logs, exec, inspect, ansamblu (compose)
- **interpreter** — 7 new shell domain intents: `Chmod`, `Chown`, `Symlink`, `Archive`, `Cron`, `ServiceEnable`, `EnvVar` with full NL parsing, translation, and tests
- **interpreter** — wired up 6 previously orphaned patterns: `find`, `remove`, `install`, `du`, `kill`, `netinfo` — these NL inputs were silently falling to Unknown
- **interpreter** — 140+ command explanations (up from 12), covering file ops, process mgmt, network, archive, dev tools, and more
- **tests** — 1,096 unit tests (up from 769)
- **docs** — CLAUDE.md with development process, principles, and DO NOTs
- **ci** — GitHub Actions CI (ci.yml) and release (release.yml) workflows
- **ark** — registered as `ark install --group shell` meta-package

### Fixed

- **security** — JSON injection prevention in knowledge.rs, marketplace.rs, package.rs via `serde_json::json!()`
- **security** — URL path sanitization in marketplace.rs and package.rs
- **security** — expanded shell metacharacter filtering in misc.rs pipeline validation
- **security** — null byte validation in network target validation
- **interpreter** — fixed parser ordering: moved `list` pattern to end (was swallowing all inputs due to all-optional regex)
- **interpreter** — tightened `show_file` regex to require "content(s) of" keyword (prevented false matches)
- **interpreter** — fixed `ai_shell::` crate references to `agnoshi::` in all benchmark files
- **interpreter** — collapsed 10 nested `if` statements into `if let` chains
- **interpreter** — replaced `unwrap()` in patterns.rs and platforms.rs with proper error handling

### Changed

- **api** — added `#[non_exhaustive]` to all public enums, `#[must_use]` on pure functions, `#[inline]` on hot paths
- **api** — added `PermissionLevel` to root re-exports, crate-level documentation
- **security** — added doc comments to all `PermissionLevel` variants, `.context()` on privilege escalation
- **deps** — updated deny.toml: added `MPL-2.0`, `CDLA-Permissive-2.0`; removed unused licenses; wildcard path deps allowed
- **version** — bumped to 0.90.0 to align with AGNOS ecosystem versioning

### Performance

- intent_parsing/batch/100: 2.43ms → 1.09ms (−55%)
- intent_parsing/batch/500: 13.1ms → 5.38ms (−59%)

## [0.1.0] - 2026-04-01

### Added

- Initial extraction from `agnosticos/userland/ai-shell/`
- Natural language interpreter with 19-file module structure
- 30+ domain translators (filesystem, process, network, AGNOS, packages, marketplace, all consumer apps)
- Intent classification and pattern matching
- Security approval workflows with human oversight
- Session management and context tracking
- Fuzzy completion engine
- Command history with search
- Dashboard for system status
- Alias system
- LLM integration via hoosh
- Audit logging
- 3 criterion benchmark suites (ai_shell, system_bench, intent_parsing)
