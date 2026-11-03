# Development Roadmap

> **Open work only, and forward-facing.** Anything shipped is removed from this file — what landed
> and why lives in [`CHANGELOG.md`](../../CHANGELOG.md). This file answers one question: *what
> next, in what order, against what gate.*
>
> **Pinned release arcs.** Each arc is a minor version, and each slot in it names the patch it is
> planned to land in. Slots are ordered by dependency, so the next unblocked slot is the lowest open
> number. ⚠ An unplanned release — a toolchain bump, a response to an upstream issue — takes the next
> free number and pushes the planned slots down. Renumber the arc in the same change.
> - 1.9.11, 1.9.12, 2.0.1 and 2.0.4 each took a patch.
> - 2.1.0 took a minor for the cyrius 6.7.9 pin while 2.0.x still had two open slots. 2.1.1 (the
>   6.7.18 pin) and 2.1.2 (kybernet's console P1) followed.
> - The skipped 2.0.x slots now open 2.1.x at 2.1.3, and the interactive shell and hoosh arcs move
>   down one minor each.
>
> **Cite a slot by arc and title** — `roadmap 2.1.x — approval-gated exec` — never by patch number
> alone: titles survive a renumber. References name functions and files, not line numbers.
>
> Verified against `src/` and the sibling repos on **2026-11-02**. The tree was at 2.1.2 (cyrius
> 6.7.18), with agnos 1.57.14 (itself on cyrius 6.6.20), puka 0.6.25 and hoosh 2.8.1. Every gate was
> re-checked at the 6.7.18 pin, and none opened or closed since 2.0.4.

## How this file is organised

| Section | Answers | Use it when |
|---|---|---|
| **Arc sequence** | *What ships next?* | Picking up work |
| **2.1.x → 2.3.x** | *What is in each release, and when is it done?* | Planning a slice |
| **Open decisions** | *What must be ruled before a slot can start?* | Before starting a slot that names one |
| **Gated** | *Why isn't this moving?* | Asking why an item is not in an arc |
| **Demand-gated backlog** / **3.0.0** | *What is deliberately not scheduled?* | Before adding something that looks missing |
| **Standing** | *What must I re-check every time?* | Bumping the toolchain pin, cutting a release |

## Arc sequence

| Arc | Theme | Next up | Gate |
|---|---|---|---|
| **2.1.x** | NL execution, finished — the approval-gated tiers and `undo` | **2.1.3** — approval-gated exec (the checkpoint store shipped in 2.0.3) | none |
| **2.2.x** | Interactive shell — `cd`, an rc file, a line editor | **2.2.0** — `cd` / `pwd` | host: none; agnos pieces gated |
| **2.3.x** | hoosh / LLM — answer questions, suggest commands | **2.3.0** — hoosh client (host) | none — agnos's socket adapter shipped in cyrius 6.6.7, not yet run by agnsh |

2.1.x is the headline: **agnoshi's premise is that natural language becomes execution.**
- **What has shipped:** 2.0.0, the first tier. SAFE and READ_ONLY lines run, under the `-c` contract
  of [ADR-008](../adr/008-nl-exec-contract.md). It was a major because that contract broke; the arc
  was 1.10.x until the ruling.
- **What 2.1.x adds:** the approval-gated tiers, with rollback in place before anything destructive
  runs.
- **Ordering:** 2.2.x and 2.3.x are independent of each other and may swap on demand. 2.3.2's
  suggested commands run through 2.1.x's exec path, so that one slot does follow it.

⚠ **The two-target reality**, which every slot below splits along.
- **agnos:** `src/run_agnos.cyr` is mostly `#ifdef CYRIUS_TARGET_AGNOS`, so pipelines, background
  jobs and `>` redirection exist only on agnos. Since 2.0.1 agnsh needs agnos **1.57.7** or later
  (`WAIT_BLOCK`; `SPAWN_F_CLEANFD` is 1.57.6), and 1.57.8 for a PTY-hosted shell to wait in the kernel.
- **Linux host:** programs run through `src/run_host.cyr`, a `$PATH` lookup and an argv launcher
  behind `run`, barewords and NL exec. The host refuses `|`, `>` and `&` (2.0.0).
- **Host console sessions:** since 2.1.2, an interactive agnsh started with `/dev/null` on stdin and
  stdout (a kybernet service) runs on `/dev/console`, with no controlling terminal
  ([ADR-010](../adr/010-console-without-ownership.md)).

Read each slot's target notes before estimating: several are half the size on one target and gated
on the other.

---

## 2.1.x — NL execution, finished

The natural-language path does what it proposes, one permission tier at a time, with rollback in
place before anything destructive runs.
- 2.0.0 shipped SAFE and READ_ONLY (see the CHANGELOG).
- 2.0.4 – 2.1.2 went to the cyrius 6.6.20, 6.7.9 and 6.7.18 pins and kybernet's console P1, so this
  arc's slots start at 2.1.3.
- The slots are ordered so that nothing destructive executes before checkpointing exists.
- Standing since 2.0.0, unscheduled: `proc_set_timeout_ms` (host only) could give NL exec a command
  timeout.

### 2.1.3 — Approval-gated exec: USER_WRITE, SYSTEM_WRITE, ADMIN

- **The prompt:** call `ApprovalManager_request` before executing these tiers. It is compiled in
  with no caller; only `session.cyr` calls it, and that module is not in the binary.
  - It reads fd 0 directly, so `-c` (no stdin) declines by default, as today.
  - ⚠ Its prompt writes to **stdout**. Move it to stderr, as `verb_confirm` did in 2.0.0: `-c`'s
    stdout is the program's (ADR-008).
- **Checkpoints:** before every REMOVE / MOVE exec, use the store
  ([ADR-009](../adr/009-checkpoint-store.md), `src/checkpoint.cyr`, compiled in with no caller): one
  `ckpt_group_new(ckpt_dir())` per command, then `ckpt_remove` / `ckpt_move` per file.
  - ⚠ A refusal — a directory (`rm -r`), a symlink, a file over 64 MiB — needs a ruling first: refuse
    the command, or run it and say that undo will not restore that file.
  - ⚠ The store's agnos arm has never run. Verify it in QEMU with the first caller.
- **ADMIN** routes through `execute_with_privileges` (`security.cyr`, compiled in since 2.0.2 with no
  caller).
  - ⚠ It launches through `execute_command` → lib `exec_vec`: an empty environment and a second host
    launcher, which ADR-008 § 3 rules out. Route it through `host_exec` first.
  - A restricted session (root on a Linux host) runs no NL line at all since 2.0.2, and
    `execute_with_privileges` refuses one (-1).
  - BLOCKED stays blocked: `WARNING: BLOCKED` is final, with no approval path.
- **Audit labels:** `approved` + outcome, `denied`, `timed_out`.
- Settle the power-verb ruling first (§ Open decisions), so every confirmation follows one policy.
- ADR the approval-vs-execute split. `docs/examples/server-hardening.md`'s do-not-deploy banner comes
  down only when `strict` mode actually gates.

### 2.1.4 — `undo`

- `ckpt_undo(ckpt_dir())` (ADR-009) goes behind a new `undo` builtin. It restores the newest group
  and returns the lines to print, and how many entries were and were not undone, which is the exit
  status.
- `commands.cyr` stopped advertising `undo` in 1.9.1, when the audit found nothing behind it. Add it
  to `is_builtin` and the dispatch together.
- **Test**: a tempdir round trip — `mkdir foo; touch foo/a; agnsh -c "remove foo/a"; agnsh -c "undo"`.

**Arc closeout**: re-run the benchmarks. README's "Not shipped yet", `SECURITY.md`'s four
NOT-IN-THE-BINARY sections and `security-model.md` all change with this arc.

---

## 2.2.x — Interactive shell

The shell as a daily driver. The host halves are unblocked; the agnos halves are agnsh-owned work or
sit on userland components, not on the kernel.

### 2.2.0 — `cd` / `pwd`

There is no `cd` and no `pwd` builtin. `cd /tmp` parses to `IntentTag.CHANGE_DIR`, prints its
proposal, and never changes directory.

- **Host**: native builtins in `agnsh.cyr`'s loop. That is lighter than adopting
  `Session_run_interactive` from `session.cyr`, whose `cd` block also treats args as `Str` where
  `split_command_line` emits cstrings.
- **agnos — agnsh-owned, permanently.** agnos's roadmap lists a userland-owned cwd among its standing
  boundaries: no chdir/getcwd syscall, by design. The 6.7.18 peer still stubs `sys_chdir` to -38.
  - agnsh tracks its own cwd and resolves its own relative paths against it; a `>` target must be
    absolute today.
  - It exports `PWD` through the env blob `sh_build_env_blob` hands to `#37` / `#43`. Children
    currently inherit kybernet's `PWD=/`.
  - ⚠ Check whether kriya and owl resolve relative operands against `$PWD` before promising `cd`
    semantics to them.
- If `session.cyr` code is reused, its `ui_show_*` calls need real bodies first. agnsh has no stubs
  for them (the two dead `return 0` ones were deleted in 1.9.15), so the build fails until they exist.

### 2.2.1 — `.agnshrc`

A startup file sourced at launch (default mode, aliases, history size), named to match the `.agnsh_*`
state files.
- **Unblocked on both targets.** `$HOME` resolves on agnos: 1.43.2 staged `envp`, and
  `lib/io.cyr`'s `getenv` branches to `_agnos_getenv`. There, `HOME=/` puts the file at `/.agnshrc`.
- ⚠ An rc file is configuration an attacker may be able to write.
  - Decide whether it may only set values, or may also run commands.
  - Open it `O_NOFOLLOW`, like the state files.
  - Say what it cannot do.

### 2.2.2 — `completion.cyr` stdlib sweep

The pre-flight for 2.2.4; no live-binary change.

- `CompletionEngine_new` pushes cstring literals. `completion_search_vec` then calls
  `str_starts_with` on the same elements that `streq` treats as cstrings, and `str_starts_with` is
  Str-typed: it reads `load64(s+8)` as a length. `CompletionEngine_complete_contextual` has the same
  contradiction. Settle **one** type for the five vecs.
- ⛔ **Both gates return a false green on this file.** Verify by other means.
  - `cyrius check src/completion.cyr` prints `ok` over eight `undefined function` warnings.
  - `scripts/lint-cstr-str.sh` matches only literal arguments.
- The v1.8.5 `vec_push` of `reboot` / `poweroff` / `halt` went into this unlinked module, so it
  produces no completion today.

### 2.2.3 — Raw-mode line editor (host)

- Nothing exists: no termios, no Tab handling, no arrow keys; `history` prints a list.
- **Host:** `ICANON`/`ECHO` off, `VMIN=1`/`VTIME=0`, arrow-key history recall, and a restore on every
  exit path, including signals. A crash in raw mode leaves the terminal unusable.
- ⚠ **A console session (2.1.2) has no controlling terminal.** ^C reaches agnsh only as a byte read
  in raw mode, never as SIGINT, and no SIGHUP comes. The restore cannot rely on either signal there.
- **agnos is § Gated.** The terminal's line discipline belongs to puka, which drops arrow-key
  sequences precisely because "agnsh has no line editor". The route there is a raw mode in puka, not
  the kernel.

### 2.2.4 — Tab completion, wired

2.2.2's engine behind 2.2.3's editor (host).

### 2.2.5 — The prompt: git branch from any subdirectory

- `prompt.cyr` finds `.git/HEAD` only in the immediate cwd.
- The v1.0 parent walk used `fs_parent`, which no longer exists; `path_dirname` (`lib/fs.cyr`) does
  the job.
- `prompt.cyr` is outside the include graph, so this is its wire-up. It needs 2.2.0 for a cwd worth
  walking.

Rides this arc when there is demand:
- **`docs <cmd>`**, a raw man-page viewer: the human half of the man-page pair, whose other half is
  `explain` in 2.3.2.
- **History fuzzy search**, after 2.2.3.

---

## 2.3.x — hoosh / LLM

The host client is unblocked: hoosh's modernization has shipped, and the stdlib carries the
transport. On agnos, the socket adapter it waited on shipped in cyrius 6.6.7 (§ Gated → *Opened
upstream*), so the same client can be built `--agnos`; nobody has run it there yet.

### 2.3.0 — hoosh client, host (`src/llm.cyr`)

- `src/llm.cyr` has never existed, and the binary has no network code at all.
- hoosh (2.8.1, re-checked at 2.1.2) serves an **OpenAI-compatible** API on `127.0.0.1:8088`. Bearer
  auth is optional, and requests must carry `Content-Length`.
- **Transport:** `lib/http.cyr` is still HTTP/1.0 **GET-only** at 6.7.18, so it cannot make the call.
  - The bundled sandhi (1.13.7 at 6.7.18) has `sandhi_http_post` and `sandhi_http_stream`.
  - JSON is bayan's 78 `bayan_json_*` functions (bayan 1.11.1 at 6.7.18; 71 at 6.6.20). Standalone
    `json.cyr` was folded into bayan at cyrius 6.2.25.
  - Add both to `[deps] stdlib`; `net` is already there.
- Port `sanitize_llm_input` (prompt-injection sanitization) from the deleted Rust tree:
  `git log --all --diff-filter=D -- rust-old/src/llm.rs`.
- **Measure the size and capacity cost:** this is the biggest include-graph jump since the port.
  `cyrius build` links every `[deps]` module whether anything includes it or not, and DCE removes
  functions, not data, so count both modules' static data as well as their code (2.1.1).

### 2.3.1 — Streaming answers to QUESTION intents

- `translate_question` still returns an `echo` stub, and `print_intent_result` still prints "LLM
  streaming arrives in a later slice". Replace both.
- sandhi's SSE parser ends at `data: [DONE]`. Show output progressively, and test with hoosh mocked.
- Add a new audit result class, `llm_suggested` or `answered`. It is the seventh; the six-class
  vocabulary has been stable since v1.3.0. ADR it.

### 2.3.2 — Suggestions for UNKNOWN input, and `explain <cmd>`

- On UNKNOWN input that reads as natural language, ask hoosh with the input, recent history, cwd and
  last exit code. The suggestion runs only through 2.1.x's exec and approval path.
- `explain <cmd>`: the assist layer reads the man page and explains it in context (pairs with `docs`).

---

## Open decisions — rule before the slot that needs them

| Decision | Needed by | The question |
|---|---|---|
| **Power verbs and confirmation** | 2.1.3 | Should a typed `reboot` / `poweroff` / `halt` require the mode confirm that `run` does? They match by exact `streq` before classification, so today they never reach `is_admin_command`. Left for an operator ruling in 1.9.11. |
| **Metacharacter pass-through in `human` mode** | any redirection-lane work | Let `;` `\|` `&` `$()` `<` `>` through in `human` only (user-flagged 2026-07-07). `is_shell_metachar` and its four wrappers are mode-blind with no mode parameter, so this threads one. On Linux it must warn: metachar → `execve` is a real injection vector (audit C2). |
| **Install location** | the next zugot recipe bump | `scripts/install.sh` → `/usr/local/bin`; the zugot recipe (ark) → `/usr/bin`; agnos images → `/bin/agnsh`. The first two are FHS-correct as a pair (local build vs package), so the likely ruling is "no change" — but the recipe ships no man page, which is a real gap. |

---

## Gated — not on the arc sequence

Open, with the trigger outside agnoshi.
- **Last check:** re-verified at pin 6.7.18 (2.1.2) against agnos 1.57.14 (itself on cyrius 6.6.20),
  puka 0.6.25 and hoosh 2.8.1.
- **Unchanged since 2.0.4:** no gate opened or closed. puka's line discipline still drops arrow-key
  sequences because "agnsh has no line editor" (`puka/src/line_discipline.cyr`).

### kybernet and argonaut

| Gate | Where | What it unblocks |
|---|---|---|
| kybernet's next release, and an argonaut tag that runs agnos-init before agnoshi in BOOT_MINIMAL | kybernet `[Unreleased]`: agnos-init grants `/dev/console` to group 1000, mode 0660. argonaut `[Unreleased]`: agnos-init in `default_services(BOOT_MINIMAL)`, with the minimal agnoshi depending on it | **The console P1's "done when": a BOOT_MINIMAL board started by kybernet reaches an agnoshi prompt on its console, as uid 1000.** agnsh's side shipped in 2.1.2 (ADR-010). Until both land, the default agnoshi exits 1 with `errno 13` on a minimal board, and on a desktop board until kybernet's release. Confirm it in kybernet's QEMU harnesses, which open the console as 1000:1000 (they need sudo to build the image). |

### agnos kernel and userland

| Gate | Filed as | What it unblocks in agnsh |
|---|---|---|
| Raw keyboard input | puka (line discipline) | 2.2.3's editor on agnos. `kbscan #42` works from ring 3 but loses keys (0–4 of 9 at a 100 ms hold). |

### Opened upstream — agnsh's half, not yet scheduled

The kernel halves shipped in agnos 1.57.6 and 1.57.7 (the issues are archived there, resolved), and
the socket adapter in cyrius 6.6.7; what is left is agnsh's. Each lands in an arc on demand.

| Upstream gave | Issue | What agnsh has to do |
|---|---|---|
| 1.57.6: `spawn_path` #43 fails with `-SPAWN_E_*` — −2 table full, −3 no memory, −4 no such program, −5 not an executable, −6 bad arguments | `2026-09-23-spawn-path-failure-gives-no-reason` | Say which in the launch error and the `error` audit record: agnsh still prints one message for all of them. `execwait` #37 — the `>` path — still folds them to −1. |
| 1.57.6: `SPAWN_F_ARGV`, an argv blob whose entries may contain spaces (≤ 1,024 bytes) | `2026-09-23-spawn-path-args-cannot-contain-spaces` | A quoting parser — agnsh has none — then launch through the argv form, which also lifts the 127-byte line cap. |
| 1.57.6: `SPAWN_F_CLEANFD`, multi-pair `exec_redirect`, arms cleared on every #43 / #37 return | `2026-09-23-child-inherits-every-fd-and-spawn-arms-leak` | **Pipeline stages: done in 2.0.1.** The `>` child still inherits agnsh's table — 0/1/2 and its target — because #37 takes no flags; `2>` and `2>&1` wait on the redirection lane (§ Demand-gated backlog). |
| cyrius 6.6.7: the agnos socket adapter — `lib/net.cyr` over `sock_connect` #47, `sock_listen` #56, `sock_accept` #57 and `sock_peer` #106 (agnos 1.57.7's kernel half: a loopback-only listen, `127.0.0.0/8` as a destination, EOF after the peer's FIN, `#47` / `#48` that block only their caller). 6.6.17 adds the portable `sys_socket` / `sys_connect` names; agnos 1.57.14 boots sandhi's HTTP server under 6.6.20 | `2026-09-23-tcp-server-cannot-be-loopback-only` (agnos); `2026-09-25-agnos-sock-peer-spawn-limits-wait-block-kill-tree-peer` (cyrius, resolved 6.6.7) | 2.3.x on agnos: build 2.3.0's client `--agnos` and run it in QEMU against a host hoosh at `10.0.2.2:8088`. agnos's smoke runs sandhi's server; the client side has not been run there from this repo. |
| 1.57.7: `kill` #16 ends (9), stops (19) and continues (18) a child | `2026-09-23-parent-cannot-end-stop-or-continue-a-child` | Job control — `kill %n`, `fg` / `bg`, Ctrl-C to the foreground child — which needs a keypress to reach agnsh while it waits: agnos has no default action for SIGINT. (A killed child's status has read 128 + N since 2.0.1.) |

### Consumer app translators

Wire each only when the app lands a public surface for agnoshi to translate into. **None exists in
the binary**: v1.0.0 pruned `IntentTag` from 211 to 44, and v1.3.2 deleted the Rust translators with
`rust-old/`. Each is a full implementation, not a re-wire: Agnostic, Delta, Edge, Shruti, Tazama,
Rasa, Mneme, Synapse, BullShift, Yeoman, Phylax, T-Ron, Tarang, Jalwa, **Stiva** (its 12 container
intents did not survive the prune), Aequi, Photis.

---

## Demand-gated backlog

Not scheduled. Open on demand.

- **The redirection lane beyond `cmd > file`.** Each of these fails `is_safe_path` on the residual
  `>` today rather than misbehaving:
  - `>>`, `<`, `2>`, combined redirects and globbing;
  - `cmd > file` on the Linux host (both dispatch sites are agnos-only);
  - a shared writable-target denylist closing `> /bin/agnsh` and `> /boot/agnos`.

  The kernel-side clear of an armed `exec_redirect#62` on every early return, which this list used
  to carry, shipped in agnos 1.57.6.
- Docker CLI syntax → stiva; SSH key management; VPN / proxy intents; systemd timers, sockets and
  dependencies; log rotation; a diff preview before destructive file operations.
- **A `--restricted` flag**: restrict an ordinary user's session the way root's is restricted since
  2.0.2. `SecurityContext_new(1)` does this, but nothing calls it that way.
  `docs/examples/server-hardening.md` describes it as a login-shell wrapper for untrusted users.
- **`[deps]` modules nothing includes.** `sakshi` and `net` are listed, so `cyrius build` links them,
  but no file includes either. At 6.7.18 they cost 9,296 B and 288 B of the x86_64 binary (2.1.1).
  `net` earns its place with 2.3.0's transport; `sakshi` would need a caller first.
- Rich prompt themes; AI-assisted, project-aware completion (after 2.2.4 and 2.3.x).

## 3.0.0 — what would force the next major

No scoped work. Candidates:

- A break in the intent enum, translator shape or session contract. There is no shipped session
  contract yet: `session.cyr` is not in the binary.
- An audit-log format break (would need migration tooling).
- A different LLM transport than hoosh.

**Re-evaluate at the close of 2.1.x** (NL execution). The trigger this section once named — "when
Bucket 1 closes" — was never going to fire as a single event.

---

## Standing

### The toolchain pin-bump checklist

This replaces the per-version "Moving the cyrius pin to …" sections. ⛔ **Run these; do not recall
them.** The 6.6.6 changelog read exposed a constant agnoshi had hardcoded that meant something else on
one arch, and three gates had opened without anyone noticing.

1. `cyrius.cyml` `cyrius = "…"` is the source of truth; CI reads it from there.
2. **`rm -rf lib && cyrius deps`.**
   - `cyrius deps` never deletes what an earlier pin vendored; the local `lib/` had grown to 114
     files against CI's 38.
   - `cyrius.lock` is tracked since 2.0.4. A moved pin re-locks on this run, so the lock changes in
     the same diff as the pin. Under an unchanged pin a changed snapshot is refused; do not reach for
     `--relock` without finding out why.
   - Since 2.1.0, CI's *Check the committed lock* (`cyrius deps --locked`) fails a lock the pin no
     longer resolves to, before *Resolve dependencies* can re-lock it.
3. **Read what changed, for every version crossed.**
   - Start with `../cyrius/docs/development/ecosystem-migration.md`, the consumer-facing summary of
     each release. The CHANGELOG behind it is far larger: 6.7.10 – 6.7.18 alone is about 45,000
     lines.
   - Then read the entries it names, for:
     - new compile errors;
     - the semantics of the stdlib modules in the include graph (string, fmt, alloc, vec, str,
       syscalls + peers, io, fs, hashmap, tagged, args, chrono, process);
     - CLI changes to fmt / lint / check / vet / capacity / build / deps;
     - the bench row format (`scripts/bench-history.sh` parses ` avg ` lines);
     - every "consumers must".
4. **Run all CI gates on a clean `git archive` copy first**, which is CI's own starting state, then
   on the tree. Include the aarch64 suites under qemu-user and the agnos build.
   - Run each `ci.yml` step as written, under `bash -e`. That is the runner's shell for a `run:`
     block with no `shell:` key, and it has no pipefail. Read each step's exit status:
     - At 2.0.4, a local loop that grepped lint output for `warn` passed while CI's lint step exited
       1 on a file that no longer compiled.
     - At 2.1.0, a replay under `-eo pipefail` falsely failed the security scan, whose `grep | awk`
       finds nothing on a clean tree.
   - Then run `python3 scripts/agnos-qemu-test.py` and `python3 scripts/agnos-qemu-bench.py`. CI
     builds the agnos target but cannot run it, and these are the repo's only tests that do.
5. **Re-verify every gate in § Gated and every upstream claim in the arcs.** A pin bump is when a
   blocker quietly disappears.
6. Benchmarks: five alternating runs per toolchain behind any claim; one `bench-history.csv` row each.
7. **Record all three binary sizes.**
   - aarch64 DCE NOPs dead functions in place, so its size is not comparable with x86_64's.
   - If a size moves with no code change, build with each installed cyrius
     (`~/.cyrius/versions/<v>/`) to find the release that moved it. 2.1.1's −60 KB was a stdlib
     fold's static buffers, linked because every `[deps]` module is.

### The release checklist

- `sh scripts/version-bump.sh X.Y.Z` — `VERSION` plus the `VERSION_STR` literal the binary prints; CI
  fails if they disagree.
- The CHANGELOG entry goes **directly under `## [Unreleased]` at the top**. Tools insert under the first
  `## [Unreleased]` they find: 1.9.12's entry landed 1,400 lines down, under a stray one.
- Delete the shipped slot from this file (do not mark it done), renumber its arc, and move citations:
  `grep -rn "roadmap [0-9]" src tests docs README.md SECURITY.md`.
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
- **In this repo, only the two QEMU scripts exercise the agnos paths.**
  - `scripts/agnos-qemu-test.py` types into the interactive shell, at about one keystroke a second,
    so scenarios are slow, and it cannot see a `-c` exit status.
  - A background-job test there needs a sleeper that waits on a signal (the harness creates `/stop`).
    Both alternatives were tried: a busy-count starved the keyboard, and a fixed wall time expired
    before the ninth job was typed.
  - An agnos syscall clobbers `rcx, rdx, rsi, rdi, r8–r11`.
  - `scripts/agnos-qemu-bench.py` boots a driver (`tests/agnos_hostsh.cyr`) that spawns agnsh itself,
    so an agnos `-c` exit-status case is one `hs_c` line there, gated by the bench.
  - The bench's idle gate samples agnsh's state once per tick and wants BLOCKED in most samples. Do
    not go back to one read: agnos wakes a blocked pipe read every 100 ms to re-check it, so one read
    can land on READY. Do not trust ticks alone either: a `#44` poller is charged 0 ticks.
- **Honour ADR-006** at every new Str/cstring boundary: `_in_str` suffix, per-arch syscall wrappers,
  `str_clone` for static-buffer escape, every cstring path NUL-terminated.
- **The lint shield's next free category is I** (A–H are taken; H is the 1.9.10 audit-path seam). Its
  blind spot stands: A/B match only literal arguments.
- **Buffer sizing is by scope**: module-scope `var X[N]` is 8N bytes, function-scope is N
  (`CONTRIBUTING.md`). `sh_env_blob[128]` is 1 KB, not 128 bytes.
- **Coverage**: every wire-up grows the fn denominator; add `test_core` anchors for the new module's
  pure functions. The 80% gate is CI-enforced, and agnos-only functions are reported, not gated.
- **Benchmarks** after 2.1.3 (approval-gated exec) and 2.3.1 (streaming): both add new code paths.
  2.0.0's NL exec is dominated by process creation, which the in-process suite does not measure.
