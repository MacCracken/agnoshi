# Development Roadmap

> **Open work only, and forward-facing.** Anything shipped is removed from this file — what landed
> and why lives in [`CHANGELOG.md`](../../CHANGELOG.md). This file answers one question: *what
> next, in what order, against what gate.*
>
> **Pinned release arcs.** Each arc is a minor version, and each slot in it names the patch it is
> planned to land in. Slots are ordered by dependency, so the next unblocked slot is the lowest open
> number. ⚠ An unplanned release — a toolchain bump, a response to an upstream issue — takes the next
> free patch number and pushes the planned slots down by one (1.9.11 and 1.9.12 both did, and their
> pinned work is still open below). Renumber the arc in the same change.
>
> **Cite a slot by arc and title** — `roadmap 1.10.x — NL exec` — never by patch number alone:
> titles survive a renumber. References name functions and files, not line numbers.
>
> Verified against `src/` and the sibling repos on **2026-09-23** (tree at 1.9.16, cyrius 6.6.6).
> Every upstream gate was re-checked at the 6.6.6 pin (1.9.13); § Gated lists what is still open.

## How this file is organised

| Section | Answers | Use it when |
|---|---|---|
| **Arc sequence** | *What ships next?* | Picking up work |
| **1.10.x → 1.12.x** | *What is in each release, and when is it done?* | Planning a slice |
| **Open decisions** | *What must be ruled before a slot can start?* | Before starting a slot that names one |
| **Gated** | *Why isn't this moving?* | Asking why an item is not in an arc |
| **Demand-gated backlog** / **2.0.0** | *What is deliberately not scheduled?* | Before adding something that looks missing |
| **Standing** | *What must I re-check every time?* | Bumping the toolchain pin, cutting a release |

## Arc sequence

| Arc | Theme | Next up | Gate |
|---|---|---|---|
| **1.10.x** | NL execution — the natural-language path runs what it proposes | **1.10.0** — NL exec, SAFE / READ_ONLY | a ruling (§ Open decisions — NL exec contract) |
| **1.11.x** | Interactive shell — `cd`, an rc file, a line editor | **1.11.0** — `cd` / `pwd` | host: none; agnos pieces gated |
| **1.12.x** | hoosh / LLM — answer questions, suggest commands | **1.12.0** — hoosh client (host) | host: none; agnos: loopback TCP |

1.10.x is the headline: **agnoshi's premise is that natural language becomes execution, and today
the NL path only proposes** — `agnsh -c "show files"` prints `Command: ls` and runs nothing. 1.11.x
and 1.12.x are independent of each other and may swap on demand; 1.12.2's suggested commands run
through 1.10.x's exec path, so that one slot does follow it.

⚠ **The two-target reality**, which every slot below splits along: `src/run_agnos.cyr` is mostly
`#ifdef CYRIUS_TARGET_AGNOS`, so bareword launch, pipelines, background jobs and `>` redirection
exist **only on agnos**. The Linux host's sole exec path is `run /abs/path` through `lib/process.cyr`.
Read each slot's target notes before estimating — several are half the size on one target and gated
on the other.

---

## 1.10.x — NL execution

The natural-language path starts doing what it proposes, one permission tier at a time, with
rollback in place before anything destructive runs. Slots are ordered so that nothing destructive
executes before checkpointing exists.

### 1.10.0 — NL exec: SAFE and READ_ONLY intents

- **Program execution already exists — through a different mechanism than the NL path.** `run /abs/path`,
  bareword `/bin/<word>` launch, kriya/owl file verbs, `prog &`, env inheritance, two-stage pipelines
  and `cmd > file` all run today via `src/run_agnos.cyr`. What does not run is the NL path:
  `print_intent_result` prints `Intent:` / `Command:` / `Risk:` / `Hint:`, calls `audit_one_shot`, and
  never execs, on either target.
- Exec when `perm == SAFE || READ_ONLY`, from both `print_intent_result` and the interactive NL
  fallback: `sh_exec_line_sched` on agnos, `run()` on the host. (`execute_command` in `security.cyr`
  is not the route — that module is not compiled in.)
- Thread the child's exit code into the audit record with the exec-surface labels that already exist
  (`executed` / `failed` / `error`); leave the parse-time labels alone. Update the `proposed` note in
  `src/audit.cyr`, which cites this slot.
- **Shell first on the host** ([ADR-007](../adr/007-input-classification.md), decisions 1–2): a line
  whose first word resolves on `PATH` runs as that program, and a line with `|` or `>` is shell syntax —
  an error, never natural language, when a stage is not a program. agnos already dispatches this way
  (`/bin/<word>`); the host sends every line to the NL parser today. ⚠ About a third of the rows in
  `docs/examples/common-commands.md` open with a word that is a program on a typical host (`git` ×13,
  `find`, `kill`, `groups`): under the lookup they run it. The table's scope note already says so;
  keep it true.
- ⚠ **Open decision first** (§ Open decisions): which modes execute, what happens to the `-c` stdout
  contract, and how the host's NL exec reaches a program.
- Enabler new in cyrius 6.6.x: `proc_set_timeout_ms` (host only) gives NL exec a command timeout.

**Done when** `agnsh -c "show files"` runs `ls` in the modes ADR'd to run it, and the audit record
carries its exit code. Re-run the benchmarks — this is a new code path.

### 1.10.1 — Wire `security.cyr`

- Absent from `src/agnsh.cyr`'s include graph; its only includer is the dead legacy `src/main.cyr`.
  Include it and construct `SecurityContext_new(0)` in `main()` after `alloc_init()` / `args_init()`.
- It carries what 1.10.3 needs for ADMIN: `execute_with_privileges` (prepends `sudo -n`) and
  `verify_sudo_path` (re-verifies at escalation; TOCTOU window documented in ADR-006).
- **Host-only user-visible value**: the `uid == 0 → restricted` warning is compiled out on agnos
  (single-owner; uid 0 is normal there, CHANGELOG 1.8.4).
- Watch the capacity and coverage gates: ~9 functions and a 64 KB `/etc/passwd` buffer.

### 1.10.2 — Checkpointing, re-implemented against the current stdlib

`src/checkpoint.cyr` calls seven `fs_*` helpers the stdlib no longer has — absent from the 6.5.36
snapshot (1.9.8) and re-verified absent at 6.6.6. Six have equivalents; one does not:

| `checkpoint.cyr` calls | cyrius 6.6.6 |
|---|---|
| `fs_mkdir_p` | `xmkdir_p` (`lib/io.cyr`) |
| `fs_rename` | `file_rename` (`lib/io.cyr`, has an agnos arm) |
| `fs_exists` | `file_exists` (`lib/io.cyr`) |
| `fs_remove` | `xunlink` / `xrmdir` (`lib/io.cyr`) |
| `fs_is_dir` | `is_dir` (`lib/fs.cyr`, takes a `Str`) |
| `fs_basename` | `path_basename` (`lib/fs.cyr`, takes a `Str`) |
| `fs_copy` | **none** — write it (read + write loop; there is no copy or sendfile helper) |

Two of the equivalents take a `Str`: honour ADR-006 at that boundary. Checkpoints go to
`$HOME/.agnoshi/checkpoints/`, auto-pruned to the newest 100. No exec change in this slot.

### 1.10.3 — Approval-gated exec: USER_WRITE, SYSTEM_WRITE, ADMIN

- `ApprovalManager_request` is compiled in and has no caller. Call it before executing these tiers.
  It reads fd 0 directly, so `-c` (no stdin) declines by default, as today.
- Checkpoint before every REMOVE / MOVE exec (1.10.2).
- ADMIN routes through `execute_with_privileges` (1.10.1). BLOCKED stays blocked — `WARNING: BLOCKED`
  is final, with no approval path.
- Audit labels: `approved` + outcome, `denied`, `timed_out`.
- Settle the power-verb ruling first (§ Open decisions) so every confirmation follows one policy.
- ADR the approval-vs-execute split. `docs/examples/server-hardening.md`'s do-not-deploy banner comes
  down only when `strict` mode actually gates.

### 1.10.4 — `undo`

- `CheckpointManager_undo` behind a new `undo` builtin. `commands.cyr` stopped advertising `undo` in
  1.9.1, when the audit found nothing behind it — add it to `is_builtin` and the dispatch together.
- **Test**: a tempdir round trip — `mkdir foo; touch foo/a; agnsh -c "remove foo/a"; agnsh -c "undo"`.

**Arc closeout**: re-run the benchmarks; README's "Not shipped yet", `SECURITY.md`'s four
NOT-IN-THE-BINARY sections and `security-model.md` all change with this arc.

---

## 1.11.x — Interactive shell

The shell as a daily driver. The host halves are unblocked; the agnos halves are agnsh-owned work or
sit on userland components, not on the kernel.

### 1.11.0 — `cd` / `pwd`

There is no `cd` and no `pwd` builtin. `cd /tmp` parses to `IntentTag.CHANGE_DIR`, prints its
proposal, and never changes directory.

- **Host**: native builtins in `agnsh.cyr`'s loop (lighter than adopting `Session_run_interactive`
  from `session.cyr`, whose `cd` block also treats args as `Str` where `split_command_line` emits
  cstrings).
- **agnos — agnsh-owned, permanently.** agnos's roadmap lists a userland-owned cwd among its standing
  boundaries: no chdir/getcwd syscall, by design. The 6.6.6 peer stubs `sys_chdir` to -38. So agnsh
  tracks its own cwd, resolves its own relative paths against it (a `>` target must be absolute
  today), and exports `PWD` through the env blob `sh_build_env_blob` hands to `#37` / `#43`. Children
  currently inherit kybernet's `PWD=/`. ⚠ Check whether kriya and owl resolve relative operands against
  `$PWD` before promising `cd` semantics to them.
- If `session.cyr` code is reused, its `ui_show_*` calls need real bodies first: agnsh has no stubs
  for them (the two dead `return 0` ones were deleted in 1.9.15), so the build fails until they exist.

### 1.11.1 — `.agnshrc`

A startup file sourced at launch (default mode, aliases, history size), named to match the `.agnsh_*`
state files. Unblocked on both targets: `$HOME` resolves on agnos (1.43.2 staged `envp`;
`lib/io.cyr`'s `getenv` branches to `_agnos_getenv`), where `HOME=/` puts it at `/.agnshrc`.
⚠ An rc file is configuration an attacker may be able to write: decide whether it may only set values
or may also run commands, open it `O_NOFOLLOW` like the state files, and say what it cannot do.

### 1.11.2 — `completion.cyr` stdlib sweep

The pre-flight for 1.11.4; no live-binary change.

- `CompletionEngine_new` pushes cstring literals, then `completion_search_vec` calls `str_starts_with`
  (Str-typed, reads `load64(s+8)` as a length) on the same elements that `streq` treats as cstrings.
  `CompletionEngine_complete_contextual` has the same contradiction. Settle **one** type for the five vecs.
- ⛔ **Both gates return a false green on this file.** `cyrius check src/completion.cyr` prints `ok`
  over eight `undefined function` warnings, and `scripts/lint-cstr-str.sh` matches only literal
  arguments. Verify by other means.
- The v1.8.5 `vec_push` of `reboot` / `poweroff` / `halt` went into this unlinked module: it produces no
  completion today.

### 1.11.3 — Raw-mode line editor (host)

- Nothing exists: no termios, no Tab handling, no arrow keys; `history` prints a list.
- Host: `ICANON`/`ECHO` off, `VMIN=1`/`VTIME=0`, arrow-key history recall, and a restore on every exit
  path including signals — a crash in raw mode leaves the terminal unusable.
- agnos is § Gated: the terminal's line discipline belongs to puka, which drops arrow-key sequences
  precisely because "agnsh has no line editor". The route there is a raw mode in puka, not the kernel.

### 1.11.4 — Tab completion, wired

1.11.2's engine behind 1.11.3's editor (host).

### 1.11.5 — The prompt: git branch from any subdirectory

`prompt.cyr` finds `.git/HEAD` only in the immediate cwd. The v1.0 parent walk used `fs_parent`,
which no longer exists; `path_dirname` (`lib/fs.cyr`) does the job. `prompt.cyr` is outside the
include graph, so this is its wire-up — needs 1.11.0 for a cwd worth walking.

Rides this arc when there is demand: **`docs <cmd>`**, a raw man-page viewer (the human half of the
man-page pair; `explain` is 1.12.2), and **history fuzzy search** (after 1.11.3).

---

## 1.12.x — hoosh / LLM

The host client is unblocked: hoosh 2.6.10's modernization has shipped, and the stdlib carries the
transport. On agnos the client is § Gated on loopback TCP.

### 1.12.0 — hoosh client, host (`src/llm.cyr`)

- `src/llm.cyr` has never existed, and the binary has no network code at all.
- hoosh 2.6.10 serves an **OpenAI-compatible** API on `127.0.0.1:8088`; bearer auth optional; requests
  must carry `Content-Length`.
- Transport: `lib/http.cyr` is HTTP/1.0 **GET-only** and cannot make the call. The bundled sandhi has
  `sandhi_http_post` and `sandhi_http_stream`; JSON is bayan's 69 `bayan_json_*` functions (standalone
  `json.cyr` was folded into bayan at cyrius 6.2.25). Add both to `[deps] stdlib`; `net` is already there.
- Port `sanitize_llm_input` (prompt-injection sanitization) from the deleted Rust tree:
  `git log --all --diff-filter=D -- rust-old/src/llm.rs`.
- Measure the size and capacity cost; this is the biggest include-graph jump since the port.

### 1.12.1 — Streaming answers to QUESTION intents

- `translate_question` still returns an `echo` stub and `print_intent_result` still prints "LLM
  streaming arrives in a later slice". Replace both.
- sandhi's SSE parser ends at `data: [DONE]`; show output progressively (test with hoosh mocked).
- A new audit result class (`llm_suggested` or `answered`) — the seventh; the six-class vocabulary has
  been stable since v1.3.0. ADR it.

### 1.12.2 — Suggestions for UNKNOWN input, and `explain <cmd>`

- On UNKNOWN input that reads as natural language, ask hoosh with the input, recent history, cwd and
  last exit code. The suggestion runs only through 1.10.x's exec and approval path.
- `explain <cmd>`: the assist layer reads the man page and explains it in context (pairs with `docs`).

---

## Open decisions — rule before the slot that needs them

| Decision | Needed by | The question |
|---|---|---|
| **NL exec contract** | 1.10.0 | (1) Which modes execute — `run` already confirms in `human`/`strict` and runs directly in `auto`/`assist` (`mode_needs_confirm`); reuse that, or say why NL differs. (2) The `-c` report is on **stdout** today and `scripting.md` documents piping it; moving it to stderr so child output stays clean is **breaking** — keep it, flag it, or make it 2.0.0. (3) The host: ADR-007 gives it a `PATH` lookup for shell lines in this slot — does NL exec launch through the same lookup, or only by absolute path via `run()`? |
| **Power verbs and confirmation** | 1.10.3 | Should a typed `reboot` / `poweroff` / `halt` require the mode confirm that `run` does? They match by exact `streq` before classification, so today they never reach `is_admin_command`. Left for an operator ruling in 1.9.11. |
| **Metacharacter pass-through in `human` mode** | any redirection-lane work | Let `;` `\|` `&` `$()` `<` `>` through in `human` only (user-flagged 2026-07-07). `is_shell_metachar` and its four wrappers are mode-blind with no mode parameter, so this threads one. On Linux it must warn: metachar → `execve` is a real injection vector (audit C2). |
| **Install location** | the next zugot recipe bump | `scripts/install.sh` → `/usr/local/bin`; the zugot recipe (ark) → `/usr/bin`; agnos images → `/bin/agnsh`. The first two are FHS-correct as a pair (local build vs package), so the likely ruling is "no change" — but the recipe ships no man page, which is a real gap. |

---

## Gated — not on the arc sequence

Open, with the trigger outside agnoshi. Each was re-verified at pin 6.6.6.

### agnos kernel and userland

| Gate | Filed as | What it unblocks in agnsh |
|---|---|---|
| `spawn_path` #43 answers -1 for every failure | agnos `2026-09-23-spawn-path-failure-gives-no-reason` | Telling "process table full" from "no such program" in the launch error and the `error` audit record. agnsh launches foreground and background jobs through #43. |
| `spawn_path` arguments cannot contain a space | agnos `2026-09-23-spawn-path-args-cannot-contain-spaces` | Quoted arguments reaching a program on agnos. |
| A child inherits every fd; `exec_redirect` arms one fd; a failed spawn leaves `CH_ENDOW` armed | agnos `2026-09-23-child-inherits-every-fd-and-spawn-arms-leak` | Clean fds in redirected and piped children. |
| A parent cannot end, stop or continue a child | agnos `2026-09-23-parent-cannot-end-stop-or-continue-a-child` | Job control: Ctrl-C to the foreground child, `kill %n`, `fg` / `bg`. |
| Raw keyboard input | puka (line discipline) | 1.11.3's editor on agnos. `kbscan #42` works from ring 3 but loses keys (0–4 of 9 at a 100 ms hold). |
| Loopback TCP, and sockets that hold the CPU | agnos `2026-09-23-tcp-server-cannot-be-loopback-only`, `…-sock-recv-never-reports-eof-after-peer-fin`, `…-sock-send-and-connect-hold-the-cpu` | 1.12.x on agnos. TCP to `127.0.0.1` is dropped; receive never reports EOF (stop on `[DONE]`). Under QEMU a host hoosh is reachable at `10.0.2.2:8088`. |

### Consumer app translators

Wire each only when the app lands a public surface for agnoshi to translate into. **None exists in
the binary**: v1.0.0 pruned `IntentTag` from 211 to 44, and v1.3.2 deleted the Rust translators with
`rust-old/`. Each is a full implementation, not a re-wire — Agnostic, Delta, Edge, Shruti, Tazama,
Rasa, Mneme, Synapse, BullShift, Yeoman, Phylax, T-Ron, Tarang, Jalwa, **Stiva** (its 12 container
intents did not survive the prune), Aequi, Photis.

---

## Demand-gated backlog

Not scheduled. Open on demand.

- **The redirection lane beyond `cmd > file`**: `>>`, `<`, `2>`, combined redirects and globbing
  (each fails `is_safe_path` on the residual `>` today rather than misbehaving); `cmd > file` on the
  Linux host (both dispatch sites are agnos-only); a shared writable-target denylist closing
  `> /bin/agnsh` and `> /boot/agnos`; a kernel-side clear of the one-shot `exec_redirect#62` on every
  `execwait#37` early return.
- Docker CLI syntax → stiva; SSH key management; VPN / proxy intents; systemd timers, sockets and
  dependencies; log rotation; a diff preview before destructive file operations.
- Rich prompt themes; AI-assisted, project-aware completion (after 1.11.4 and 1.12.x).

## 2.0.0 — what would force a major

No scoped work. Candidates:

- The `-c` output contract, if the 1.10.0 decision needs stdout for child output.
- A break in the intent enum, translator shape or session contract (there is no shipped session
  contract yet — `session.cyr` is not in the binary).
- An audit-log format break (would need migration tooling).
- A different LLM transport than hoosh.

**Re-evaluate at the close of 1.10.x.** The trigger this section used to name — "when Bucket 1
closes" — was never going to fire as a single event.

---

## Standing

### The toolchain pin-bump checklist

This replaces the per-version "Moving the cyrius pin to …" sections. ⛔ **Run these; do not recall
them** — the 6.6.6 changelog read exposed a constant agnoshi had hardcoded that meant something else on
one arch, and three gates had opened without anyone noticing.

1. `cyrius.cyml` `cyrius = "…"` is the source of truth; CI reads it from there.
2. **`rm -rf lib && cyrius deps`.** `cyrius deps` never deletes what an earlier pin vendored — the
   local `lib/` had grown to 114 files against CI's 38.
3. Read the cyrius CHANGELOG for **every** version crossed: new compile errors; semantics of the
   stdlib modules in the include graph (string, fmt, alloc, vec, str, syscalls + peers, io, fs, hashmap,
   tagged, args, chrono, process); CLI changes to fmt / lint / check / vet / capacity / build / deps;
   the bench row format (`scripts/bench-history.sh` parses ` avg ` lines); every "consumers must".
4. All CI gates on a clean `git archive` copy first — CI's own starting state — then on the tree,
   including the aarch64 suites under qemu-user and the agnos build. Then
   `python3 scripts/agnos-qemu-test.py`: CI builds the agnos target but cannot run it, and this is the
   repo's only test that does.
5. **Re-verify every gate in § Gated and every upstream claim in the arcs.** A pin bump is when a
   blocker quietly disappears.
6. Benchmarks: five alternating runs per toolchain behind any claim; one `bench-history.csv` row each.
7. Record all three binary sizes. aarch64 DCE NOPs dead functions in place, so its size is not
   comparable with x86_64's.

### The release checklist

- `sh scripts/version-bump.sh X.Y.Z` — `VERSION` plus the `VERSION_STR` literal the binary prints; CI
  fails if they disagree.
- The CHANGELOG entry goes **directly under `## [Unreleased]` at the top**. Tools insert under the first
  `## [Unreleased]` they find: 1.9.12's entry landed 1,400 lines down, under a stray one.
- Delete the shipped slot from this file (do not mark it done), renumber its arc, and move citations:
  `grep -rn "roadmap 1\.\|Bucket\|Slice [0-9]" src docs README.md SECURITY.md`.
- Refresh the touched rows of `docs/doc-health.md`.
- The zugot recipe, `~/Repos/zugot/marketplace/agnoshi.cyml`: version plus the sha256 of the
  `agnsh-X.Y.Z-x86_64-linux` release asset (check it against the release's `SHA256SUMS`), then
  zugot's own `scripts/validate_recipes.py` and a line in zugot's CHANGELOG. Do it every release: it
  drifted from 1.7.0 to 1.9.13 before anyone did.

### Notes for the next agent

- **Open flags are per-arch — spell the symbol, never the number.** x86_64 and arm64 swap
  `O_NOFOLLOW` / `O_LARGEFILE` and `O_DIRECTORY` / `O_DIRECT`. Hardcoding x86's `O_NOFOLLOW` left the
  aarch64 release following symlinks for twelve releases, behind a unit test that pinned the bug. CI
  runs every suite on aarch64 under qemu-user since 1.9.13; keep it that way.
- **In this repo, only `scripts/agnos-qemu-test.py` exercises the agnos paths.** Typing there is about one keystroke a
  second, so scenarios are slow. A background-job test needs a sleeper that waits on a signal (the
  harness creates `/stop`) — a busy-count starved the keyboard and a fixed wall time expired before
  the ninth job was typed; both were tried. An agnos syscall clobbers `rcx, rdx, rsi, rdi, r8–r11`.
- **Honour ADR-006** at every new Str/cstring boundary: `_in_str` suffix, per-arch syscall wrappers,
  `str_clone` for static-buffer escape, every cstring path NUL-terminated.
- **The lint shield's next free category is I** (A–H are taken; H is the 1.9.10 audit-path seam). Its
  blind spot stands: A/B match only literal arguments.
- **Buffer sizing is by scope**: module-scope `var X[N]` is 8N bytes, function-scope is N
  (`CONTRIBUTING.md`). `sh_env_blob[128]` is 1 KB, not 128 bytes.
- **Coverage**: every wire-up grows the fn denominator; add `test_core` anchors for the new module's
  pure functions. The 80% gate is CI-enforced, and agnos-only functions are reported, not gated.
- **Benchmarks** after 1.10.0 and 1.12.1 — both add new code paths.
