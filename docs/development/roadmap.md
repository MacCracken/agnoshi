# Development Roadmap

**Forward-facing only.** Everything on this page is open work. Shipped work lives in
[`CHANGELOG.md`](../../CHANGELOG.md) — that is the source of truth for what has landed.
Items leave this file when they ship; they are not marked done and kept.

> **Buckets are named by content, not by version** — assign a version at cut time.
> File:line references below were verified against `src/` on 2026-08-29.

---

## v1.9.x hardening arc — carry-over from the 2026-08-29 P(-1)

**This is the immediate next work.** The 1.9.1 sweep fixed 35 finding-clusters and left these
deliberately, each with a reason. Full context per item in
[`docs/audit/2026-08-29-pminus1.md`](../audit/2026-08-29-pminus1.md). Slices are ordered by
severity, not by convenience. (1.9.2 exec-audit, 1.9.3 exec-path error handling,
1.9.4 state-file/error-output hygiene and 1.9.5 parser shadowing have shipped — see the CHANGELOG. The untested agnos half of
1.9.2/1.9.3 is recorded below as verification debt.)

### Verification debt — the agnos exec surface has never been executed

⚠ **Not a feature slice. This is the standing gap behind everything 1.9.2 and 1.9.3 changed**, and
it has now carried forward twice, so it is recorded as debt rather than folded into another slice
where it can quietly vanish again.

The exec audit surface (1.9.2) and the exec-path error handling (1.9.3) both landed with their
host-reachable halves **executed and asserted** — the `run` path end-to-end, and the five pure
parsers that 1.9.3 hoisted out of the `#ifdef`. Everything else lives behind
`#ifdef CYRIUS_TARGET_AGNOS` and is **compile-verified on all three targets and code-reviewed, but
has never been run**:

- pipeline / redirect / background-job audit records (launch line + matching outcome)
- the probe-before-validate reordering in `sh_try_bareword_launch`
- the two sentinel collapses (a failed redirect or pipeline no longer reads as "not mine")
- the three `exec_redirect#62` arm-return checks
- the pre-spawn job-table capacity check

**What would close it** — an agnos smoke run on iron asserting, for `cmd1 | cmd2`, `cmd > file` and
`prog &`: a `launched` record followed by a matching outcome record; `> /.agnsh_audit.log` refused
and recorded as `denied`; a 9th background job refused *without* a stray child; and a deliberately
missing binary in a pipeline stage reported once rather than silently retried down the NL path.

The host smoke suite (`scripts/smoke-test.sh`) already has the shape to copy — it exercises the
binary and parses the resulting audit log. What is missing is a target to run it on.

### Carried from 1.9.4 — the history cap is hardcoded and disagrees with the config

The interactive loop constructs its history with a hardcoded cap of **1000**, while
`ShellConfig_default` declares **10000**. `config.cyr` is not in the binary's include graph, so the
10000 is dead code and 1000 is the real limit — but the two disagree, and the live one is not the
configurable one. Decide which is right, make it the single source, and wire the config value in
(or delete the dead field). Small; it was noticed while fixing the load path, not fixed there,
because "make the config real" is a different change from "stop eating the history file".

### Carried from 1.9.5 — no MEMORY_INFO intent, and a dispatch-ordering decision

**No memory answer.** `show memory usage` no longer emits `df -h` (a disk report), but it routes to
SYSTEM_INFO → `uname -a`, which does not report memory either. There is no MEMORY_INFO tag and no
`free -h` translator. That is a missing capability, not a shadowing bug, which is why 1.9.5 fixed
the routing and stopped there. Small: one IntentTag, one parser arm, one translator.

**The ordering question.** The dispatch runs broad keyword matchers before specific ones
(`parse_show_commands` 1st, `parse_file_ops` 2nd, `parse_admin_ops` 5th, `parse_state_queries` 8th),
and 1.9.5 fixed **four** shadowing bugs that all came from that single property — each one a
guard bolted onto the broad parser to make it decline. Four from one cause is evidence that the
ordering is the defect and the guards are symptom management.

Worth deciding deliberately rather than waiting for the fifth: either reorder specific-before-broad
(and re-verify every existing parse, which is why it was not done inside a bug-fix slice), or
accept guard-by-guard and write that down as the chosen posture so the next person does not
re-litigate it. Note the guards are not free — 1.9.5 measured one at +31% on `parse/list_files`
before it was optimised down to +5%.

### 1.9.6 — Measured optimization

All six carry a named benchmark; none is speculative.

- **Why `parse/shell_cmd` is 4× the next slowest parse** — SHELL_COMMAND is the terminal
  fall-through, so it pays all 79 preceding match attempts. A cheap discriminator (first byte /
  token count) before the NL cascade is the headline win.
- Case-insensitive compare loops never `break` on mismatch — every probe pays the full needle
  length at every offset (`sanitize.cyr`).
- `analyze_command_permission` runs up to 75 full `streq` calls (two `strlen` walks each) with no
  first-byte gate.
- `input_has_word` characterises the needle, then `is_word_prefix` repeats the identical
  `strlen` + trim. ⚠ **Now has a measured price tag**: 1.9.5 added two keyword guards on the
  LIST_FILES path and paid **+31%** on `parse/list_files` for them (2.464us → 3.23us), reduced to
  +5% only by gating them behind a shorter needle. Every guard the parser gains pays this twice
  over, so fixing the double scan makes future correctness fixes cheaper, not just this one faster.
- `is_safe_path` walks the path twice with two separate `strlen` calls where one fused pass
  answers both predicates.
- `get_command_basename` scans forward to find the *last* slash instead of scanning back.

### 1.9.7 — Test reachability

- ✅ *(largely closed in 1.9.3)* `run_agnos.cyr`'s **pure** parsers — `sh_scan_trailing_amp`,
  `_sh_find_pipe`, `_sh_find_redirect`, `_sh_bin_segment`, `_sh_path_segment` — were hoisted out
  of the `#ifdef` and now have host unit coverage (the "host-compilable extraction" option was
  the one taken). What remains untestable on the host is everything that needs a syscall:
  `_sh_bin_probe` and the launchers themselves — see the verification-debt entry above.
- **The coverage gate's denominator omits five modules that are in the binary today**, so the
  reported figure overstates real coverage.
- ✅ *(closed in 1.9.2)* Smoke now asserts the exec audit end-to-end: log created, launch/outcome pair present, refusal recorded with `approved:0`, `exit_code` null rather than the sentinel when inapplicable, and every line valid JSON. Still missing: the same assertions on the **agnos-only** launchers — see the 1.9.3 carry-over.

### 1.9.8 — Latent defects in the non-compiled modules

Not reachable today; each detonates on wire-up. Fix with the corresponding wire-up slice, or
sooner if cheap.

- **`prompt.cyr:16` — `PromptContext_new` hands a 256-byte buffer to `uname(2)`, which writes
  390.** 134-byte overflow; the sharpest of these.
- `session.cyr:31` / `checkpoint.cyr` — `str_cat(getenv-cstring, Str)`: a wild-pointer `memcpy`,
  a hard SIGSEGV the lint shield cannot see (it matches only literal arguments).
- `session.cyr` — `Session_handle_builtin` compares Str with cstring `streq`, so no builtin ever
  matches. *(The 1.9.1 boundary fix makes `cmd` a real cstring, so this is now half-closed; the
  args side still expects Str.)*
- `session.cyr:33` — hands `CommandHistory_new` a Str where every consumer expects a cstring.
- `checkpoint.cyr` wire-up is blocked by **7 removed stdlib symbols**, not the "1 deferred
  MEDIUM" the roadmap claimed.
- `session.cyr`'s raw `SYS_CHDIR` / `SYS_GETCWD` are **compile errors on agnos**, not the runtime
  `-38` stub previously assumed.
- `prompt.cyr:111` — passes a Str to the cstring-typed `is_safe_branch_name`, so the git-branch
  escape-injection guard is skipped.
- `audit.cyr:121` — `audit_format_table` passes a variable-held cstring to Str-typed
  `str_builder_add`.
- `aliases.cyr:23` — calls `map_del`; the 6.5.36 stdlib renamed it `map_delete`.
- `interpreter.cyr:489` — `str_split(Str, " | ")` reads 8 bytes past the 4-byte separator literal
  as a length.
- **`prompt.cyr`'s git parent-walk deferral is unblocked** — `path_dirname(Str)` exists in the
  6.5.36 stdlib.

### Not scheduled — needs a decision, not a slice

- **The `>` scan is unanchored**, so a natural-language sentence containing `>` is diverted into
  the redirect path. Fixing it means deciding where NL ends and shell syntax begins, which is a
  product question rather than a bug.

---

## Bucket 1 — Exec on the NL path, module wire-ups, hoosh/LLM

Two headlines: **execution on the natural-language path**, and **the LLM client**. Everything else
here is either a module wire-up that was deferred at port time or a hardening item. The order below
is the suggested slice path — grouped by dependency so the next non-blocked slice is pickable
without re-deriving the graph.

### Dependency map

```
[1] security.cyr wire ────────────────┐   (host-only value; agnos compiles the rule out)
                                      ▼
[2] session.cyr wire  →  cd / pwd    [5] exec on the NL path  →  [6] approval-gated exec
    (host-only; agnos has no cwd)         (program exec already                │
                                           ships — see slice)                  ▼
[4] checkpoint.cyr wire  ─────────────────────────────────────────────→  [7] `undo` builtin

[12] hoosh client (src/llm.cyr)  →  [13] LLM streaming (QUESTION + revision)

(independent, can land any time)
[3]  ui.cyr dead-stub cleanup      — tiny; NOT a ui.cyr wire-up any more
[8]  completion.cyr stdlib sweep   →  [9] raw-mode line editor (agnos: upstream-blocked)
[10] O_NOFOLLOW hardening          — three write paths; agnos needs a kernel bit
[11] ark install-path reconciliation — release-side only, no source impact
```

**Suggested first bite**: slice 3 (dead-stub cleanup) for a five-minute win, or slice 1
(`security.cyr` wire-up) for the smallest real slice. Slice 5 is the highest-value item on the page.

**Two-target reality check.** `src/run_agnos.cyr` is wholly `#ifdef CYRIUS_TARGET_AGNOS`. Bareword
launch, pipelines, background jobs and `>` redirection therefore exist **only on the agnos target**;
the Linux host build's sole exec path is `run /abs/path` through `lib/process.cyr`. Several slices
below split along that line — read the per-slice scope notes rather than assuming parity.

---

### Slice 1 — Wire `src/security.cyr` into the agnsh binary
**Deps**: none. **Risk**: low. **Bite size**: small.

- `security.cyr` is absent from `src/agnsh.cyr`'s include graph. The only file that includes it is
  `src/main.cyr`, a **dead legacy entry that nothing builds** (`cyrius.cyml` sets
  `entry = "src/agnsh.cyr"`; CI and release both build only that).
- Add `include "src/security.cyr"`, construct `var sec = SecurityContext_new(0);` in `main()` after
  `alloc_init()` / `args_init()`, and thread it toward the exec slices (5/6) that will query it.
- **Scope note — this is host-only value now.** CHANGELOG 1.8.4 compiled the `uid == 0 → restricted`
  rule *out* under `CYRIUS_TARGET_AGNOS` ("agnos is single-owner, uid 0 is the normal state"). On
  agnos the root-warning is intentionally dead, so the user-visible payoff is Linux/macOS only.
- The module is maintained but test-only by policy (CHANGELOG 1.3.0 states it explicitly); 1.8.2
  fixed a `verify_sudo_path` stack smash and a root-warning write truncation in it.
- **Verify**: non-root → clean exit, no warning; root → `WARNING: Shell running as root` on stderr.
  Watch the capacity + coverage gates (security.cyr brings ~9 fns and a 64 KB `/etc/passwd` buffer).

### Slice 2 — Wire `src/session.cyr` into agnsh + fix 5 deferred MEDIUM findings
**Deps**: slice 1. **Risk**: medium. **Bite size**: medium-large.

- There is **no `cd` and no `pwd` builtin today**. Typing `cd /tmp` parses to `IntentTag.CHANGE_DIR`,
  prints `Intent:` / `Command:` / `Risk:` — and never chdir's. That is the user-visible gap.
- Either adopt `Session_run_interactive`, or (lighter touch, recommended) keep agnsh's own loop and
  reach in for the cd/mode/history dispatch only.
- ✅ **The five deferred MEDIUM audit findings are CLOSED (1.9.1)** — fixed in place rather than
  waiting on this wire-up. `SYS_CHDIR(str_data(dir))` ×2 and `sys_chmod(str_data(dir))` now use
  `str_cstr()`; the three discarded `SYS_GETCWD` returns go through one checked
  `session_cwd_or_unknown()` helper that returns `"?"` with a stderr warning. Lint Category G now
  guards the `str_data`-into-a-path-syscall class. **Do not re-derive these — they are done.**
- Still open in this module (see the v1.9.x arc, slice 1.9.8): `str_cat(getenv-cstring, Str)` at
  `session.cyr:31` is a wild-pointer memcpy, and the `cd` block treats args as Str while the
  1.9.1 boundary fix makes `split_command_line` emit cstrings — reconcile both at wire-up.
- **Scope note — a working `cd` is host-only.** `lib/syscalls_x86_64_agnos.cyr:517` stubs
  `sys_chdir` to `-38` ("agnos ring-3 has no cwd concept and no chdir number"), and agnos has no
  per-process CWD at all — which is why `verb_abspath` resolves relative paths against `/` there.
  Decide up front whether the builtin is host-only or gets an agnos-side story.

### Slice 3 — Dead-stub cleanup in `src/agnsh.cyr` (**not** a `ui.cyr` wire-up)
**Deps**: none. **Risk**: very low. **Bite size**: tiny.

- **Do not wire `ui.cyr` in.** The banner, `help`, `mode`, `history`, `clear` and goodbye all live
  natively inside `agnsh.cyr` and are strictly more current than `ui.cyr`'s v1.0-era text —
  `src/ui.cyr:43`'s `ui_show_help` still advertises an `undo` builtin that does not exist.
  Re-including it would risk exactly the double-printing the original slice warned about.
- What genuinely remains: the two silent no-op stubs at `src/agnsh.cyr:31-32` —
  `fn ui_show_error(m) { return 0; }` and `fn ui_show_warning(m) { return 0; }`. Delete them, or give
  them real stderr bodies.
- They are currently unreachable (their only callers, `src/aliases.cyr` and `src/session.cyr`, are
  outside the include graph) — but they are a **live footgun**: the first in-graph caller silently
  discards its error message. Slice 2 wires `session.cyr` in, so do this before or with it.
- Also stale and worth clearing: the comment at `agnsh.cyr:24-26` still says ui.cyr is "queued for
  the v1.2.1 interactive-shell wire-up".

### Slice 4 — Wire `src/checkpoint.cyr` into agnsh + fix 1 deferred MEDIUM
**Deps**: slice 3. **Risk**: low. **Bite size**: small.

- `checkpoint.cyr` is absent from the include graph; there is no checkpointing in the binary today.
- ✅ **Both halves of the original finding are CLOSED.** The HIGH unchecked return was fixed in
  v1.3.1; the MEDIUM `sys_chmod(str_data(dir), 448)` null-termination bug was fixed in 1.9.1
  (`str_cstr`), and the literal-length over-read in the chmod warning was fixed with it.
- ✅ **Lint Category G shipped in 1.9.1** — `str_data(...)` into a path-taking syscall. It names
  the path-taking calls explicitly so the legitimate `sys_write(fd, str_data(s), len)` ptr+len
  shape is not matched. Next free letter is now **H**.
- What remains here is the wire-up itself, which is blocked on 7 removed stdlib symbols (see the
  v1.9.x arc, slice 1.9.8) — a bigger job than the "1 deferred MEDIUM" this slice used to claim.

### Slice 5 — Execute on the natural-language path
**Deps**: slice 1. **Risk**: medium-high. **Bite size**: medium.

- **Program execution already works — through a different mechanism than this slice proposed.**
  agnsh executes real programs today via `src/run_agnos.cyr`: `run /abs/path` (1.4.3), bareword
  `/bin/<word>` launch (1.4.7), kriya/owl file-verb delegation (1.5.0), background `prog &` (1.6.0),
  env inheritance (1.7.0), two-stage pipelines (1.8.0/1.8.1, streaming 1.8.8), `cmd > file` (1.8.3),
  scheduler-friendly foreground (1.8.6).
- **What did NOT ship is this slice's actual acceptance criterion: the NL path still only proposes.**
  `print_intent_result` (`src/agnsh.cyr:187`) prints `Intent:` / `Command:` / `Risk:` / `Hint:`, calls
  `audit_one_shot`, and **never execs — on either target**. `agnsh -c "show files"` still does not run
  `ls`.
- **The originally proposed mechanism is dead.** `execute_command` lives in `src/security.cyr:136`,
  which is not in the include graph; its only callers are in the uncompiled `session.cyr`. Route
  through `sh_exec_line_sched` / `sh_exec_line` (`run_agnos.cyr`) on agnos and `lib/process.cyr`'s
  `run()` on the host instead — wiring security.cyr in is no longer the cheapest path.
- Remaining work:
  1. In `print_intent_result` and the interactive NL fallback, exec when `perm == SAFE || READ_ONLY`.
  2. Thread the child's exit code into `audit_one_shot` (`agnsh.cyr:163`) and extend
     `classify_audit_result` (`agnsh.cyr:129`) with `"executed"` (rc==0) / `"failed"` (rc>0) /
     `"error"` (rc<0). Leave the five parse-time labels alone. Delete the stale TODO at
     `agnsh.cyr:169` that still promises this.
  3. Decide the stdout/stderr split so child stdout stays clean for piping (recommend metadata to
     stderr, child output passes through stdout).
  4. Delete the now-false `Hint: pipeline intent -- auto-exec arrives with the exec wire-up`
     (`agnsh.cyr:250`) — pipelines *do* auto-exec on agnos.
  5. Decide whether NL exec is agnos-only, since the host build has no bareword/pipeline/redirect exec.

### Slice 6 — Approval-gated exec for higher permissions
**Deps**: slice 5. **Risk**: high (interactive UI, sudo escalation). **Bite size**: medium.

- For `USER_WRITE` / `SYSTEM_WRITE` / `ADMIN`, call `ApprovalManager_request(am, cmd, args, argc, risk)`
  before executing. Today the approval gate prints "Approval required" and nothing executes, so the
  gate is untested against a real exec.
- `ApprovalManager_request` reads a single char via `syscall(SYS_READ, 0, &buf, 63)` — fine
  interactively, unusable in `-c` (no stdin). For `-c`, decline by default (current behavior).
- ADMIN routes through `execute_with_privileges` (prepends `sudo -n`, re-verifies via
  `verify_sudo_path`). The TOCTOU window is documented in ADR-006.
- BLOCKED stays blocked — `WARNING: BLOCKED` is final, no approval path.
- Audit labels: `"approved_executed"` (or `"executed"` with `approved=1`), `"denied"`, `"timed_out"`.

### Slice 7 — `undo` builtin
**Deps**: slice 6 (checkpoint must run *before* exec). **Risk**: low. **Bite size**: small-medium.

- ⚠ **There is a live inconsistency to fix here.** `src/commands.cyr` — which **is** in the binary —
  already reports `undo` as a builtin (`is_builtin("undo") == 1`, plus a description string). But
  `agnsh.cyr` has no `undo` dispatch and `session.cyr:129` (the implementation) is not compiled in.
  So the binary advertises a builtin it does not implement; typing `undo` falls through to the NL
  parser. Either implement it here or stop claiming it.
- Checkpoint before each REMOVE/MOVE exec via `CheckpointManager_checkpoint(cm, intent)` into
  `$HOME/.agnoshi/checkpoints/`; add `undo` to the builtin dispatch calling `CheckpointManager_undo`.
- Auto-prune keeps the most recent 100 entries.
- **Test**: tempdir round-trip — `mkdir foo; touch foo/a; agnsh -c "remove foo/a"; agnsh -c "undo"`.

### Slice 8 — `src/completion.cyr` stdlib sweep (pre-flight for slice 9)
**Deps**: none. **Risk**: low. **Bite size**: small-medium.

- Never swept. The module has been touched twice ever: the v1.0 port, and a v1.8.5 edit that
  `vec_push`'d `"reboot"`/`"poweroff"`/`"halt"` as literals **into an unlinked module** (that edit
  produces no user-visible completion — do not read CHANGELOG 1.8.5 as completion progress).
- ⚠ **Both gates this slice would lean on return a FALSE GREEN — do not trust them as proof:**
  - `cyrius check src/completion.cyr` prints `ok` while emitting 8 `undefined function` warnings
    (`vec_new`, `vec_push`, `alloc`, `str_len`, `vec_len`, `vec_get`, `str_starts_with`, `streq`).
    Nothing was actually type-checked.
  - `sh scripts/lint-cstr-str.sh src` prints `clean` only because Categories A/B match a **literal**
    `"` argument; variable-carried cstrings are invisible to it.
- The ADR-005/006 defect is real and specific: `CompletionEngine_new` pushes cstring **literals**
  (`completion.cyr:10-63`), then `completion_search_vec:92` calls `str_starts_with(item, prefix)` —
  declared `(s: Str, prefix: Str)` with `str_len = load64(s+8)`, i.e. reading 8 bytes past a cstring
  literal as a length — while `:96` calls cstring-typed `streq` on the **same** vec elements. Same
  contradiction in `CompletionEngine_complete_contextual` (`streq` at `:114`/`:121` vs `str_len` at
  `:118`/`:122`).
- The sweep must (a) settle **one** type for the five vecs, and (b) verify by other means than the
  two false-green gates above.
- Produces no live-binary change — it exists so slice 9 isn't also a bug-discovery slice.

### Slice 9 — Raw-mode line editor (tab completion, arrow keys)
**Deps**: slice 8. **Risk**: medium-high. **Bite size**: large.

- Nothing exists: zero `termios` / `tcgetattr` / `tcsetattr` / `ICANON` hits in `src/`, no Tab
  (byte 9) handling, no arrow-key parsing. `history` prints a list; it is not arrow-key recall.
- ⚠ **The two targets need different mechanisms, and agnos is upstream-blocked.**
  - **Host**: the described termios path (`ICANON`/`ECHO` off, `VMIN=1`/`VTIME=0`) is valid.
  - **agnos**: the kernel owns line discipline — CHANGELOG 1.4.1 moved `read_line` to a *single*
    line-buffered `read(fd 0)` with kernel echo ("canonical-lite"), the opposite direction. Its Notes
    say a richer line editor "would want raw keystrokes again; that returns when agnos's future
    multithreading arc lets ring 3 run with IF=1 and O1 can go back to RAW." The agnos raw-input
    primitive would be `kbscan #42`, **not** `tcgetattr`. So on agnos this is **blocked on a kernel
    capability**, not merely unstarted.
- **Watch out**: raw mode breaks the existing `read_line` path and needs a restore-on-exit (trap
  signals) or a crash leaves the terminal unusable.

### Slice 10 — `O_NOFOLLOW` hardening on file writes
**Deps**: none. **Risk**: low. **Bite size**: small.

- ✅ **Two of the three write paths are CLOSED (1.9.1).** `src/audit.cyr` and `src/history.cyr`
  now open with `O_NOFOLLOW` via `file_open`, and the history file is created 0600 at open rather
  than chmod'd afterwards. Verified by pre-placing symlinks at both paths: the decoy is untouched.
  ⚠ Note the constant is **131072 on BOTH arches** — there is no per-arch split, and the value
  this roadmap and `security-model.md` used to publish for aarch64 (`0o100000` = 32768) is
  `O_LARGEFILE`. Do not reintroduce a per-arch `#ifdef`.
- **Still open: the third path.** `src/run_agnos.cyr:504` — the `>` redirect target open (`0x301`),
  a symlink-TOCTOU named in 1.8.3's own Deferred section. This one genuinely needs upstream work:
  agnos has **no `AO_NOFOLLOW` bit at all**, so it requires a kernel change threading it into
  `ext2_path_lookup` before the client can OR it in. `file_open` currently drops the bit on agnos
  (graceful degrade, not a fix).

### Slice 11 — ark install-path reconciliation
**Deps**: none. **Risk**: low (release-side only). **Bite size**: small.

- The divergence this slice exists to close is fully intact: `scripts/install.sh` (unmodified since
  2026-04-13) installs `/usr/local/bin/agnsh` + `/usr/local/share/man/man1/agnsh.1` +
  `/usr/local/share/agnoshi/`, while the zugot recipe
  (`~/Repos/zugot/marketplace/agnoshi.cyml`) declares `groups = ["tool","shell","ai","cyrius"]` and
  installs `$PKG/usr/bin/agnsh`. `/usr/local/bin` vs `/usr/bin`.
- Reconcile against the ark `--group shell` layout; update both sides. May need `/etc/agnoshi/` for
  system config (none exists today — `ShellConfig` is built in code).
- Still live despite the project's agnos tilt: `release.yml` ships x86_64 + aarch64 Linux artifacts
  and the zugot recipe consumes them.

### Slice 12 — hoosh client wire-up (`src/llm.cyr`)
**Deps**: external hoosh modernization. **Risk**: medium. **Bite size**: medium.

- `src/llm.cyr` **has never existed** in repo history. The binary has **no network capability at
  all**: the include graph carries no `lib/net.cyr`, `lib/tls.cyr`, `lib/json.cyr` or HTTP — so a
  call to `127.0.0.1:8088` is physically impossible today. `src/config.cyr` defines
  `LLM_BASE_URL_DATA` but is itself not in the include graph (dead v1.0-era constant).
- ⚠ **CHANGELOG 1.8.4 is not an LLM ship.** "AI stays enabled on agnos" removes the
  `uid == 0 → restricted` heuristic in `security.cyr` under `CYRIUS_TARGET_AGNOS`. It changes a
  policy predicate — no transport, no client, no prompt path — and is doubly moot since
  `security.cyr` isn't compiled in. Do not read it as progress here.
- Port prompt-injection sanitization (`sanitize_llm_input`) from the deleted `rust-old/src/llm.rs`
  via git history: `git log --all --diff-filter=D -- rust-old/src/llm.rs`.
- JSON: `lib/json.cyr` no longer exists standalone — cyrius 6.2.25 folded it into the `bayan` distlib
  (`bayan_json_*`), which is why v1.7.1 dropped `json` from `[deps] stdlib`. Plan for `bayan`.
- Premise is still live: `~/Repos/hoosh` exists; the "away from hoosh" note is only a v2.0.0
  speculation.

### Slice 13 — LLM streaming (QUESTION + revision workflow)
**Deps**: slice 12. **Risk**: medium. **Bite size**: medium.

- The echo this slice exists to replace is still shipped behavior: `translate_question`
  (`src/translate.cyr:523`) returns `Translation_new("echo", …, "Question -- needs LLM", …)`, reached
  live via `interpreter.cyr:585`. The user-facing hint is still printed at `agnsh.cyr:248`
  ("LLM streaming arrives in a later slice").
- Revision workflow: on UNKNOWN intent that looks like NL, query the LLM with input + recent history
  + cwd + last exit code (the v1.0-era `suggest_command_with_context` shape; its only trace is in the
  pre-1.0 Rust changelog — the source is gone).
- New audit label `"llm_suggested"` (a 7th result class — `agnsh.cyr:156` still returns `"needs_llm"`;
  `"llm_suggested"` appears nowhere). ADR it if the vocabulary change proves architectural.
- **Test**: with hoosh mocked, verify output appears progressively rather than buffered to completion.

### Closeout checklist (when this bucket cuts)

- Bump `VERSION` via `sh scripts/version-bump.sh <new>` — it also syncs the `VERSION_STR` banner
  literal in `src/agnsh.cyr`, which CI's version-consistency gate enforces.
- `CHANGELOG.md` entry for the release.
- **Delete the shipped slices from this file** (do not annotate them as done).
- `docs/doc-health.md` row refresh.
- ADR slot if earned — candidates: the approval-vs-execute split, or the LLM result-class addition.
- Verify the zugot recipe version at `~/Repos/zugot/marketplace/agnoshi.cyml`.

### Notes for the next agent

- **Highest value on this page is slice 5** — the NL path is agnoshi's whole premise and it still
  only proposes.
- **Cheapest real win is slice 3** (two dead stubs), then slice 1.
- **Parallelism**: slices 8, 10, 11 are independent and interleave well with the 1→7 sequence.
- **Don't skip slice 8 before slice 9** — and don't trust `cyrius check` or the lint shield to prove
  completion.cyr is clean; both false-green on it (see slice 8).
- **Check the target split before estimating.** `run_agnos.cyr` is agnos-only, `sys_chdir` is
  host-only, `AO_NOFOLLOW` doesn't exist on agnos, raw mode is kernel-blocked on agnos. Several
  slices are half the size you'd guess on one target and blocked on the other.
- **Honor ADR-006** at any new Str/cstring boundary: explicit `_in_str` suffix, per-arch syscall
  wrappers, `str_clone` for static-buf escape, every cstring path NUL-terminated.
- **Watch the lint shield** (`scripts/lint-cstr-str.sh`) as modules come online; new patterns land as
  new categories (next free letter is **G** — F is taken by the v1.3.3 rule).
- **Benchmarks**: rerun `sh scripts/bench-history.sh` after slice 5 and slice 13 — both add genuinely
  new code paths. Note 6.5.x subtracts a measured clock-read floor, so rows before/after v1.9.0 are
  not directly comparable.
- **Coverage**: every wire-up grows the in-binary fn denominator. Add `test_core` anchors for new
  modules' pure-logic fns; the 80% gate is CI-enforced.

---

## Bucket 2 — Demand-gated

Open these as demand warrants; none is scheduled.

### Systems features

- **Redirection lane beyond `cmd > file`** (single-target truncate already works on agnos):
  - **`>>` append, `<` stdin-from-file, `2>` stderr, and multiple/combined redirects.** Today
    `_sh_find_redirect` returns the first byte-62 offset with no handling for any of these; each
    safely fails the `is_safe_path` check on the residual `>` rather than misbehaving. Globbing also
    still open.
  - **Port redirection to the Linux host build.** Both dispatch sites sit inside
    `#ifdef CYRIUS_TARGET_AGNOS`, so `cmd > file` does **not** work on the host — it is still
    rejected by `is_safe_path`.
  - **Three tracked low-severity hardening items** from CHANGELOG 1.8.3 §Deferred, none of which have
    any code in `src/`: symlink-TOCTOU (needs a kernel `AO_NOFOLLOW` bit in `ext2_path_lookup` plus
    the client OR-ing it into the `0x301` open — see slice 10); robust kernel-side clear of the
    one-shot `exec_redirect#62` on every `execwait#37` early-return (the client-side `_sh_bin_probe`
    only covers the common missing-binary case, for both the redirect and pipeline paths); and a
    shared writable-target denylist closing the `> /bin/agnsh` / `> /boot/agnos` foot-gun.
- **Metacharacter pass-through in `human` mode (user-flagged 2026-07-07).** `human` mode is the
  explicit "agnsh steps out of the way, user owns the risk" mode, so blanket rejection of `;` `|` `&`
  `$()` backtick `<` `>` is more paternalistic than the mode intends. Let standard shell symbols
  through in **`human` mode only**, keeping reject-all for `auto`/`assist`/`strict`. Today
  `is_shell_metachar` (`src/sanitize.cyr:165`) takes a single byte and **no mode argument** — it and
  all four wrappers (`has_shell_metachars`, `has_shell_metachars_cstr`, `is_safe_arg`,
  `is_safe_path`) are mode-blind, and every call site passes no mode, so this needs a threaded mode
  parameter, not just a branch. **Must surface a threat warning on Linux targets** — there,
  metachar → `execve` pass-through is a real injection vector (audit C2); the agnos model is more
  contained. Structural operators agnsh parses itself (`|`, `>`) are unaffected.
- Docker compatibility layer — translate docker CLI syntax to stiva commands.
- SSH key management — generate, add, list, agent.
- VPN/proxy configuration intents.
- Systemd timers, sockets, and dependency management.
- Log rotation intents.
- Diff preview before destructive file operations.

### UX features

- **AI-powered completion — project-type-aware suggestions.** Prerequisites are slice 8
  (completion.cyr sweep) then slice 9 (raw-mode line editor); on agnos slice 9 is itself blocked on
  the kernel multithreading arc, since line input there is kernel-owned canonical-lite.
- History fuzzy search (fzf-style) — extends v1.3.0's persistent history. Also depends on a raw-mode
  line editor for interactive incremental search.
- Rich prompt themes — extends the v1.3.0 mode-aware prompt.
- **Man page integration — two faces along the AI/human axis.** `explain <cmd>` (AI-shelled): the
  assist layer reads the man page and explains it in context. `docs <cmd>` (human side): a direct raw
  man-page viewer, no AI. Same source, two front-ends — mirrors the human/AI split behind the
  human-mode metacharacter item above. Neither builtin exists today. **The `docs` half is independent
  and buildable now**; the `explain` half inherits the hoosh/LLM dependency (slices 12–13). This item
  is the recorded home for the deferred `interpreter/explain.rs` standalone lookup table.
- **`.agnshrc` startup config** — a sourced-at-launch rc file (aliases, default mode, env/settings),
  the `.zshrc`/`.bashrc` analog. Today agnsh keeps only *state* dotfiles (`~/.agnsh_history`,
  `~/.agnsh_audit.log`, `~/.agnoshi/checkpoints/`) and reads no config at startup. Name `.agnshrc`,
  matching the `.agnsh_*` prefix.
  **Unblocked but unbuilt.** `$HOME` resolves on both targets: agnos stages `envp` on the exec init
  stack as of 1.43.2, `lib/io.cyr`'s `getenv` branches to `_agnos_getenv`, and agnsh already consumes
  this in production (`sh_build_env_blob`). Caveat: `HOME=/` on agnos, so the rc path resolves to
  `/.agnshrc`; real `PWD`/`getcwd` tracking on agnos remains a separate open gap.

### Consumer app translators (from IntentTag stubs)

Wire up only when the consumer app lands a public surface for agnoshi to translate into — that
gating condition is still the right trigger.

**None of these exist in the binary.** The `IntentTag` enum holds 44 generic shell/git/user/firewall
verbs and zero consumer-app tags — v1.0.0 pruned it from 211 to 44, and v1.3.2 deleted the 16 Rust
translator modules along with `rust-old/`. Each of these is a **full implementation**, not a re-wire.

- Agnostic (QA orchestration)
- Delta (git hosting)
- Edge (fleet management)
- Shruti (DAW)
- Tazama (video editor)
- Rasa (image canvas)
- Mneme (knowledge base)
- Synapse (LLM management)
- BullShift (trading)
- Yeoman (agent orchestration)
- Phylax (threat detection)
- T-Ron (security monitor)
- Tarang (media analysis)
- Jalwa (media player)
- **Stiva (containers)** — full re-implementation, *not* a re-wire. The 12 intents (`run`, `stop`,
  `ps`, `rm`, `pull`, `images`, `rmi`, `build`, `logs`, `exec`, `inspect`, `ansamblu`) were Rust code;
  they did not survive the 211→44 prune and the source is gone. `grep -i stiva src/` returns nothing.
- Aequi (finance)
- Photis (tasks/habits)

---

## Bucket 3 — Future major (2.0.0)

No scoped work yet. Candidates that would justify a major cut:

- Breaking change to the public API surface (intent enum, translator trait shape, session contract).
  Note there is no shipped session contract to break yet — `session.cyr` isn't in the binary (slice 2).
- Audit-log format break (would require migration tooling). The six-class `result` vocabulary has been
  stable since v1.3.0.
- Switch to a different LLM transport (away from hoosh). Downstream of hoosh ever being wired in at
  all — `grep -rn hoosh src/` currently returns zero hits.

**Re-evaluate when Bucket 1 closes.** This re-evaluation is itself outstanding work — it has been
owed since the trigger it originally named fired back in May 2026.
