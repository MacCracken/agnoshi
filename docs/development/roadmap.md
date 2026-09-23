# Development Roadmap

**Forward-facing only.** Everything on this page is open work. Shipped work lives in
[`CHANGELOG.md`](../../CHANGELOG.md) — that is the source of truth for what has landed.
Items leave this file when they ship; they are not marked done and kept.

> **The v1.9.x arc's remaining slices are version-pinned** (1.9.11 – 1.9.12). The three buckets
> below it are named by content and get a version at cut time — the arc is a finite, ordered piece
> of work; the buckets are not.
> References name **functions and files, not line numbers** — line numbers drift
> with every edit and were already stale five refs out of seven by 1.9.9.
> Verified against `src/` on 2026-08-29 (tree at 1.9.10).

---

## Moving the cyrius pin to 6.6.5

Nothing breaks at this bump. cyrius 6.6.5 is not tagged yet, and nothing below can land against the
pin until it is. The pin is 6.6.2 today, and this section lists only what 6.6.5 itself changes.

- [ ] `scripts/check-fmt.sh` can be deleted. It exists because `cyrius fmt` ignored every file after
      the first; from 6.6.5 `fmt` takes 1..N files and checks each one. Replace the *Format check*
      step in `.github/workflows/ci.yml` (it runs the script, and its comment explains why) with
      `cyrius fmt --check src/*.cyr tests/*.cyr tests/*.tcyr tests/*.bcyr`, the script's own globs.
      The script's `--selftest` already prints a NOTE when the multi-file form starts reporting
      drift. See the cyrius CHANGELOG [6.6.5] entry "The whole CLI took flags as file names, dropped
      flags written after the operand, and dropped extra operands".
- [ ] At the bump, re-run `cyrius deps` — the aarch64 syscall peer moved SYS_UNLINKAT 35 → 263, so
      an un-re-vendored peer's sys_unlink would run nanosleep.

---

## v1.9.x hardening arc — remaining slices

The numbered slices **1.9.1 – 1.9.10 have shipped** (see `CHANGELOG.md`); they are gone from this
file per the forward-only rule. What the arc still owes is below, **pinned to versions** rather
than left as undated carry-overs. Full context for every item:
[`docs/audit/2026-08-29-pminus1.md`](../audit/2026-08-29-pminus1.md).

Two items that arose during the arc are **not** listed here because they are not hardening work:
the `checkpoint.cyr` stdlib blocker moved into Bucket 1 Slice 4 (it is that slice's real blocker),
and `prompt.cyr`'s git parent-walk moved to Bucket 2 (UX). The buffer-scope rule the arc learned
the hard way now lives in `CONTRIBUTING.md` § Code Standards, where a contributor will actually
meet it.

### 1.9.11 — agnos iron verification *(gated on hardware, not on effort)*

⚠ **The standing debt of the whole arc.** The exec audit surface (1.9.2) and the exec-path error
handling (1.9.3) both landed with their host-reachable halves executed and asserted — the `run`
path end-to-end, and the five pure parsers 1.9.3 hoisted out of the `#ifdef`. Everything else lives
behind `#ifdef CYRIUS_TARGET_AGNOS` and is **compile-verified on all three targets and
code-reviewed, but has never been run**:

- pipeline / redirect / background-job audit records (launch line + matching outcome)
- the probe-before-validate reordering in `sh_try_bareword_launch`
- the two sentinel collapses (a failed redirect or pipeline no longer reads as "not mine")
- the three `exec_redirect#62` arm-return checks
- the pre-spawn job-table capacity check

This is pinned to a version rather than left open-ended, but it is **gated on access to agnos iron**
— it cannot be closed by effort alone, and that is why it has carried across three slices.

**What would close it** — an agnos smoke run asserting, for `cmd1 | cmd2`, `cmd > file` and
`prog &`: a `launched` record followed by a matching outcome; `> /.agnsh_audit.log` refused and
recorded as `denied`; a 9th background job refused *without* a stray child; and a deliberately
missing binary in a pipeline stage reported once rather than silently retried down the NL path.
`scripts/smoke-test.sh` already has the shape to copy — it runs the binary and parses the resulting
audit log. What is missing is a target to run it on.

Since 1.9.7 the debt is also visible in CI output: `scripts/check-coverage.sh` reports the
agnos-only function count (**18**) separately from the gated host-reachable figure.

### 1.9.12 — the dispatch-ordering decision *(a decision, then possibly a slice)*

**Two symptoms, one question, asked twice.**

1.9.5 fixed **four** parser-shadowing bugs that all came from a single property: the dispatch runs
broad keyword matchers before specific ones (`parse_show_commands` 1st, `parse_file_ops` 2nd,
`parse_admin_ops` 5th, `parse_state_queries` 8th). Each fix was a guard bolted onto the broad parser
to make it decline. Four from one cause is evidence that the **ordering** is the defect and the
guards are symptom management — and the guards are not free: 1.9.5 measured one at **+31%** on
`parse/list_files` before optimising it down to +5%.

1.9.6 then made every probe much cheaper (`parse/shell_cmd` −42%) without changing the shape:
SHELL_COMMAND is still the terminal fall-through, so a plain shell line still pays ~79 keyword
probes and is still 4× the next-slowest parse. The remaining win there is a discriminator that
recognises "this is not natural language" **before** entering the cascade.

⇒ Those are the same question. A pre-cascade discriminator changes *which parser claims an input*,
so it is a correctness change wearing a performance costume — which is exactly why neither slice
attempted it.

**Decide first, then implement.** Either reorder specific-before-broad and re-verify every existing
parse, or accept guard-by-guard and write that down as the chosen posture so the next person does
not re-litigate it. Whichever is chosen, record it as an ADR — this is the kind of call ADRs exist
for.

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

### Slice 2 — Wire `src/session.cyr` into agnsh (gives the shell `cd`)
**Deps**: slice 1. **Risk**: medium. **Bite size**: medium-large.

- There is **no `cd` and no `pwd` builtin today**. Typing `cd /tmp` parses to `IntentTag.CHANGE_DIR`,
  prints `Intent:` / `Command:` / `Risk:` — and never chdir's. That is the user-visible gap.
- Either adopt `Session_run_interactive`, or (lighter touch, recommended) keep agnsh's own loop and
  reach in for the cd/mode/history dispatch only.
- **Current state of the module** (so this is not re-derived): its `str_data`-into-a-syscall and
  discarded-`getcwd` defects are already fixed, it now uses the shared `history_path()` resolver,
  and it compiles on agnos. What is left for the wire-up itself is the `cd` block's arg handling —
  it treats args as `Str`, while `split_command_line` emits cstrings — and deciding how `cd`
  behaves on a target with no cwd.
- **Scope note — a working `cd` is host-only.** `lib/syscalls_x86_64_agnos.cyr:517` stubs
  `sys_chdir` to `-38` ("agnos ring-3 has no cwd concept and no chdir number"), and agnos has no
  per-process CWD at all — which is why `verb_abspath` resolves relative paths against `/` there.
  Decide up front whether the builtin is host-only or gets an agnos-side story.

### Slice 3 — Dead-stub cleanup in `src/agnsh.cyr` (**not** a `ui.cyr` wire-up)
**Deps**: none. **Risk**: very low. **Bite size**: tiny.

- **Do not wire `ui.cyr` in.** The banner, `help`, `mode`, `history`, `clear` and goodbye all live
  natively inside `agnsh.cyr` and are strictly more current than `ui.cyr`'s v1.0-era text —
  `src/ui.cyr`'s `ui_show_help` still advertises an `undo` builtin that does not exist.
  Re-including it would risk exactly the double-printing the original slice warned about.
- What genuinely remains: the two silent no-op stubs `ui_show_error` / `ui_show_warning` near the top of `src/agnsh.cyr` —
  `fn ui_show_error(m) { return 0; }` and `fn ui_show_warning(m) { return 0; }`. Delete them, or give
  them real stderr bodies.
- They are currently unreachable (their only callers, `src/aliases.cyr` and `src/session.cyr`, are
  outside the include graph) — but they are a **live footgun**: the first in-graph caller silently
  discards its error message. Slice 2 wires `session.cyr` in, so do this before or with it.
- Also stale and worth clearing: the comment at `agnsh.cyr:24-26` still says ui.cyr is "queued for
  the v1.2.1 interactive-shell wire-up".

### Slice 4 — Wire `src/checkpoint.cyr` into agnsh
**Deps**: slice 3. **Risk**: medium. **Bite size**: medium — see the blocker below.

⛔ **THE REAL BLOCKER IS 7 MISSING STDLIB SYMBOLS, not the "1 deferred MEDIUM" this slice was
named for** (1.9.1 closed that MEDIUM in place). `checkpoint.cyr` calls `fs_basename`, `fs_copy`,
`fs_exists`, `fs_is_dir`, `fs_mkdir_p`, `fs_remove` and `fs_rename` — all seven verified **absent
from the entire 6.5.36 snapshot** (1.9.8). This slice is therefore a re-implementation against the
current stdlib (`file_exists`, `file_open`, `xunlink`, `path_dirname`, …), not a wire-up, and it
should be sized accordingly. It also gates Slice 7 (`undo`).

- `checkpoint.cyr` is absent from the include graph; there is no checkpointing in the binary today.
- Its own `str_data`-into-`sys_chmod` defect and the literal-length over-read beside it are already
  fixed, so the module's remaining work is the stdlib re-implementation above plus the wire-up.
- The `str_data`-into-a-path-syscall class is guarded by lint **Category G**; the next free
  category letter is **H**.

### Slice 5 — Execute on the natural-language path
**Deps**: slice 1. **Risk**: medium-high. **Bite size**: medium.

- **Program execution already works — through a different mechanism than this slice proposed.**
  agnsh executes real programs today via `src/run_agnos.cyr`: `run /abs/path` (1.4.3), bareword
  `/bin/<word>` launch (1.4.7), kriya/owl file-verb delegation (1.5.0), background `prog &` (1.6.0),
  env inheritance (1.7.0), two-stage pipelines (1.8.0/1.8.1, streaming 1.8.8), `cmd > file` (1.8.3),
  scheduler-friendly foreground (1.8.6).
- **What did NOT ship is this slice's actual acceptance criterion: the NL path still only proposes.**
  `print_intent_result` (`src/agnsh.cyr`) prints `Intent:` / `Command:` / `Risk:` / `Hint:`, calls
  `audit_one_shot`, and **never execs — on either target**. `agnsh -c "show files"` still does not run
  `ls`.
- **The originally proposed mechanism is dead.** `execute_command` lives in `src/security.cyr`,
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

### Slice 10 — symlink-proof the `>` redirect target *(needs an agnos kernel change)*
**Deps**: an agnos `AO_NOFOLLOW` bit. **Risk**: low client-side. **Bite size**: small once unblocked.

One write path remains unprotected: `sh_run_redirect`'s target open in `src/run_agnos.cyr`, the `>` redirect target open (`0x301`)
— a symlink-TOCTOU named in 1.8.3's own Deferred section. The shell's two state files are already
covered.

⛔ **This is genuinely upstream work.** agnos has **no `AO_NOFOLLOW` bit at all**, so it needs a
kernel change threading one into `ext2_path_lookup` before the client can OR it in. `file_open`
currently drops the bit on agnos — a graceful degrade, not a fix.

⚠ When implementing, note the Linux constant is **131072 on BOTH arches**. There is no per-arch
split; the `0o100000` = 32768 value this roadmap and `security-model.md` once published for aarch64
is `O_LARGEFILE`, and shipping it would have left the race open on half the release binaries. Do
not reintroduce a per-arch `#ifdef`.

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
  (`src/translate.cyr`) returns `Translation_new("echo", …, "Question -- needs LLM", …)`, reached
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
  `is_shell_metachar` (`src/sanitize.cyr`) takes a single byte and **no mode argument** — it and
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
- **Git branch in the prompt from a repo SUBDIRECTORY.** `prompt.cyr` only checks the immediate
  cwd's `.git/HEAD`, so the branch disappears the moment you `cd` into a subdirectory. The v1.0
  parent-walk used `fs_parent`, which no longer exists — but `path_dirname(path: Str)` ships in
  `lib/fs.cyr:57` and does exactly the job, so **the blocker is gone** (verified 1.9.8). Not written
  then because `prompt.cyr` is outside the binary's include graph: new logic added there cannot be
  executed or tested. Do it whenever the prompt module is wired in.
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

## Moving the cyrius pin to 6.6.6

Current pin: `cyrius = "6.6.2"` (`cyrius.cyml`).

agnoshi appends to two logs — the command history at `src/history.cyr:240` and the audit trail
at `src/audit.cyr:102`, both through `file_open` with the named `OPEN_WRONLY | OPEN_CREAT |
OPEN_APPEND` constants that `src/sanitize.cyr` splits per target. That is precisely the shape
6.6.6 fixes on Windows, where a PE build's `O_APPEND` overwrote from offset 0 instead of
appending. **agnoshi has no PE target**, so it was never exposed: `src/` declares
`CYRIUS_TARGET_AGNOS` and `CYRIUS_TARGET_MACOS` only, CI is `ubuntu-latest` throughout, and
`release.yml` ships x86_64 + aarch64 with no `windows-*` runner. If a Windows target is ever
added, 6.6.6 is the floor and these two logs are the first thing to re-verify.

Worth noting alongside: `src/sanitize.cyr:22-26` already records the *macOS* version of this
same class of bug — the hardcoded `1089` that decodes on Darwin as
`O_WRONLY|O_ASYNC|O_TRUNC`, truncating the audit log on every open. That was fixed here by
splitting the constants per target. The Windows bug 6.6.6 fixes is the same failure one layer
down, in the compiler rather than in the caller, which is why no source change is needed for
it: agnoshi was already passing the right flags, and the PE backend was ignoring them.

Otherwise this is a plain pin bump. What was checked, and found empty:

- No other `O_APPEND` / `O_TRUNC` sites outside the vendored `lib/`; `src/history.cyr:234-239`
  is prose, and the remaining opens are read-only (`src/history.cyr:44`, `:62`,
  `src/run_agnos.cyr:558`, `:790`).
- None of item 3's new compile errors have sites: no `async fn`, no `operator` fn, no
  `ret2`/`rethi`, no SIMD intrinsics, and no struct- or vector-typed parameter or `var`
  declaration anywhere in `src/`. The 19 structs (`CommandHistory`, `Session`, `Intent`,
  `AuditEntry`, `AuditLogger`, …) are accessor-style over heap offsets and are never passed or
  assigned by value, so item 5's by-value-struct-param deep copy is a no-op too.
- No top-level bare `{` blocks (item 4), no duplicated global declarations (item 6), no
  `regression_*` call sites of its own (item 8 — the two `include` hits are
  `lib/regression.cyr` pulling `lib/regression_agnos.cyr`), and no own `vec_*` function
  colliding with the 14 names `lib/vec.cyr` exports, so the new transitive
  `lib/assert.cyr` → `lib/vec.cyr` include is inert (item 9).
- 16 `file_exists` / `file_read_all` call sites gain nothing here — that change is PE-side.

After bumping, verify: the full `.tcyr` suite passes per-file, and run the shell long enough
to write several history and audit records across two sessions, confirming both files
accumulate rather than restart.
