# Development Roadmap

## Shipped

- **v0.1.0** (2026-04-01) — Initial extraction from agnosticos/userland/ai-shell
- **v0.2.0** (2026-04-02) — Standalone hardening: P(-1) pass, CI workflows, cleanliness gates
- **v0.90.0** (2026-04-02) — Core shell domains, explain coverage, parser ordering, API stabilization, ark integration, error recovery, revision workflow, richer LLM context, checkpoint/rollback, stiva container intents
- **Cyrius port** (2026-04-13) — Rust → Cyrius port (27K → 4K lines, 21 modules), 32× parse speedup, 146 KB static binary
- **Security audit** (2026-04-13) — 21 findings closed (5 critical, 7 high, 9 medium)
- **v1.0.0** (2026-04-13) — Release candidate: tests passing, benchmarks proving Cyrius wins
- **v1.1.0** (2026-05-10) — Cyrius 5.10.34 + ecosystem-parity modernization. Toolchain pin bumped 4.5.0 → 5.10.34; manifest migrated `cyrius.toml` → `cyrius.cyml` with `version = "${file:VERSION}"` (single source of truth); `.cyrius-toolchain` retired; `./lib/` gitignored + repopulated by `cyrius deps` from the pinned stdlib snapshot (matches agnosys/yukti/patra); CI gate set expanded to syntax check + fmt diff + lint warn-as-error + vet + capacity gate + aarch64 best-effort cross-build + agnoshi-shaped security scan; release workflow accepts both `vX.Y.Z` and `X.Y.Z` tag styles with SHA256SUMS + per-arch prebuilt binaries (also fixed: was building wrong entry `src/main.cyr → agnoshi` instead of `src/agnsh.cyr → agnsh`); `CLAUDE.md` cleanliness gates rewritten Rust → Cyrius (`cargo fmt/clippy/audit/deny/doc` → `cyrius check/fmt/lint/vet/capacity`); `docs/doc-health.md` debut as a living doc-currency ledger; full closeout pass landed the five Stale rows it flagged (`README.md`, `CONTRIBUTING.md`, `docs/architecture/overview.md`, `docs/agnsh.1`, `benchmarks-rust-v-cyrius.md`). Binary 271,912 bytes on Cyrius 5.10.x (up from 146 KB on 4.5.0 — toolchain-side growth, not new agnoshi code). Full detail: `CHANGELOG.md` 1.1.0 entry.
- **v1.2.0** (2026-05-11) — Intent parsing depth + translator hardening. All three v1.2.0 roadmap items closed across nine slices: (a) **deeper intent parsing** — fixed two Cyrius 4.5 → 5.10 stdlib regressions (str_len/str_data on cstring needles, str_sub end→length semantics) that had silently left every NL input falling to `SHELL_COMMAND`; added `is_word_prefix` token-aware matcher that retires the substring-trap class while preserving plurals (file→files, process→processes); landed `parse_state_queries` (ip/network/system/disk/process noun-phrase queries), `parse_service_query` (`is X running/active/enabled`, `status of X`), and `parse_service_action` (bare imperative `start X` / `stop X` / `restart X` / `reload X` / `enable X` / `disable X`); (b) **translator production tests** — 200 new assertions across 43 translators, every translator now has command + permission-level locks, safety-check fallbacks have explicit negative tests; (c) **coverage report into CI** — `scripts/check-coverage.sh` gates fn-level coverage of the in-binary modules at ≥80% (current 89%). Also swept three latent-bug-class audits: 12 `str_cat(cstring, Str)` sites fixed across translate / prompt / security / checkpoint / sanitize / session. Test count 57 → 257 (4.5×). Full detail: `CHANGELOG.md` 1.2.0 entry.
- **v1.3.0** (2026-05-11) — Approval workflow battle-tested interactively + interactive shell end-to-end. Both v1.2.1-scoped lead items closed (cycle bumped to v1.3.0 mid-stream to reflect scope). Approval side: every `-c` invocation now prints `Risk: [LOW|MED|HIGH|CRIT]` with `WARNING: BLOCKED` / `Approval required` lines; per-invocation JSON audit log at `$HOME/.agnsh_audit.log` carries timestamp + user + mode + input + action + approved + a six-class `result` field (`proposed`/`needs_approval`/`blocked`/`needs_llm`/`needs_exec`/`rejected_safety`); `verify_sudo_path` re-checks sudo existence + root-ownership at the escalation moment, closing the TOCTOU window between session init and actual escalation. Interactive side: mode-aware prompt (`[ASSIST]>` / `[HUMAN]>` / `[STRICT]>` / `[AUTO]>`), `mode` / `history` / `clear` / `help` builtins, persistent history at `$HOME/.agnsh_history` (last 1000) loaded across sessions, line-oriented `read_line` byte-reader for piped + terminal use, error-recovery `Hint:` lines for parse-succeeded-but-not-runnable cases. Five deferred modules unbusted (audit, security, history, translate, sanitize) — sweep of Cyrius 4.5 → 5.10 stdlib regressions including the latent `is_safe_path(Str)` mismatch that had silently routed *every NL filesystem operation* to `translate_unknown` since v1.0. Test count 257 → **301**; smoke 31 → **58**. Binary 280,344 → 293,824 B. Full detail: `CHANGELOG.md` 1.3.0 entry.
- **v1.3.1** (2026-05-11) — P(-1) audit/review pass per AGNOS first-party standards. Eight slices: Cyrius 5.10.34 → 5.10.44 toolchain bump (zero codegen drift), cstring/Str static analyzer with 14 patterns across 5 categories wired into CI (catches all 7 bug variants discovered across v1.2.0/v1.3.0), buffer-safety sweep with 5 dormant static-buf-escape fixes, syscall-return audit with 2 HIGH-severity unchecked-`sys_chmod` fixes (live multi-user data-leak on history file + checkpoint dir), ADR-006 codifying the four operational rules (refines ADR-005), input-validation sweep clearing 3 stale-stdlib breaks in `prompt.cyr`, path-traversal sweep verifying every file-op site, known-CVE pattern review. Zero CRITICAL findings; 8 HIGH all fixed; 5 MEDIUM deferred to v1.4.0 (`getcwd` × 3 return-handling + 2 `str_data(Str)` → syscall null-termination, all in modules outside agnsh's include graph); 12 LOW triaged. Full report at `docs/audit/2026-05-11-pminus1.md`. Test count stays at 301; smoke 58; coverage 86%. Binary 293,824 → 293,920 B (chmod warning + history changes).
- **v1.3.2** (2026-05-11) — Doc-staleness sweep + `rust-old/` removal. Comprehensive rust-old parity audit (18 cleanly-ported modules; 6 categories of intentionally un-ported Rust code with documented homes in the roadmap — no unintentional gaps). User-facing docs caught up to v1.3.x reality: README stat-line bumped to `1.3.1 · Cyrius 5.10.44 · 294 KB / 337 KB · 301+26+59 tests`; CONTRIBUTING pin ref; architecture overview pin + binary-size + Language Migration section; getting-started examples refreshed to the mode-aware prompt + Risk-line + six-class result vocabulary; writing-intents guide gained the ADR-006 + safety-predicate callout; security-model guide added the cstring/Str split + known LOW deferred items + Forward Shield section; scripting examples got real `jq` recipes for the audit-log `result` field. `rust-old/` (1.2 MB, 65 .rs files) removed per the AGNOS first-party-standards "equal or better coverage" criterion; historical record stays in `benchmarks-rust-v-cyrius.md` + ADR-001 + git history. Repo working tree shrinks from 1.2 MB to 580 KB. Zugot recipe at `~/Repos/zugot/marketplace/agnoshi.cyml` updated locally (no release). Pure docs + deletion + version bump — no source-code changes; binary, tests, smoke, coverage all unchanged from v1.3.1.
- **v1.3.3** (2026-05-20) — Cyrius toolchain bump 5.10.44 → **6.0.1** + a latent path-traversal safety regression caught at the new compiler's codegen layout. Three sites in `sanitize.cyr` (`path_traversal_in_str` / `shell_metachars_in_str` / `safe_arg_in_str`) called bare `strlen(s)` and relied on Cyrius's name-lookup dispatch to route `strlen(Str)` → `strlen_str`. Under 6.0.x's stricter type inference the dispatch falls through to the cstring `strlen`, which walks the Str fat-pointer's bytes looking for a null and hits a zero in the address's high bytes — returning 1-7 instead of the real length. For `path_traversal_in_str("../foo")` the corrupt-length-1 case skipped the loop via the `len < 2` guard, `safe_path_in_str` said "safe", and the translator emitted `mkdir -p ../foo` ~5-10% of the time. Fix: explicit `var len = str_len(s);` (Str-side primitive — no dispatch ambiguity). lint-cstr-str.sh gained Category F (`strlen(...)` inside `_in_str` fn body) as mechanical enforcement. ADR-006's "explicit `_in_str` suffix" rule extends from *naming* to *call-site discipline*. Binary: x86_64 293,920 → **295,312 B** (+1,392 / +0.5%); aarch64 337,168 → **339,512 B** (+2,344 / +0.7%). Bracketed benchmarks pre/post: all 10 averages unchanged. 100/100 traversal-probe repro post-fix. Full detail: `CHANGELOG.md` 1.3.3 entry.
- **v1.3.4** (2026-05-28) — Cyrius toolchain pin 6.0.1 → **6.0.14** (within-6.0.x patch-level bump). Aligns the manifest pin with the already-advanced wrapper (manifest-pin drift closeout). `cyrius deps` repopulated `./lib/` from the 6.0.14 stdlib snapshot; all cleanliness gates clean (check ok, fmt no drift, lint 0 warnings, vet 22 deps / 0 untrusted / 0 missing, capacity all under 85%). **Zero codegen drift** — both binary sizes byte-for-byte identical to v1.3.3 (x86_64 295,312 B / aarch64 339,512 B). Bracketed benchmarks unchanged. Test count stays 301 + 26 + 59; coverage 86%. Full detail: `CHANGELOG.md` 1.3.4 entry.

## v1.3.x — Polish bucket — *closed*

All v1.3.x items shipped. Next anchor is v1.4.0 below.

*(empty — packaging items moved to v1.4.0 since ark packaging belongs alongside exec wire-up. zugot recipe is updated in-tree at `/home/macro/Repos/zugot/marketplace/agnoshi.cyml` as the upstream changes land — no release-coupled bumps needed for that.)*

## v1.4.0 — Exec wire-up + hoosh modernization

The cycle has two headlines (exec wire-up + LLM streaming) and gathers everything that's been blocked on broader infrastructure plus the v1.3.1 P(-1) deferred MEDIUM findings. The order below is the **suggested slice path** for the next agent — it groups by dependency so you can pick the next non-blocked slice without re-thinking the graph each time.

### Dependency map

```
hoosh-side modernization (external, user-owned)
                                                          ─┐
                                                           ▼
[1] security.cyr wire   →   [5] exec wire SAFE/READ_ONLY     [12] hoosh client wire-up
        │                            │                              │
        ▼                            ▼                              ▼
[2] session.cyr wire   →   [6] exec wire higher perms   →   [13] LLM streaming wire-up
        │  (fixes 5 MEDIUM)          │ (approval flow)              (QUESTION + revision)
        ▼                            ▼
[3] ui.cyr wire        →   [7] `undo` builtin
        │                            (needs exec to roll back)
        ▼
[4] checkpoint.cyr wire
   (fixes 1 MEDIUM)

(independent, can land any time)
[8] completion.cyr stdlib sweep (lint pre-flight)
[9] tab completion raw-mode infra (depends on [8])
[10] O_NOFOLLOW hardening (per-arch flag, lib/io.cyr API extension)
[11] ark install-path reconciliation (touches release.yml, no source impact)
```

**Suggested first bite**: Slice 1 — `security.cyr` wire-up. No dependencies; small; gets the SecurityContext scaffolding in place so the exec slices (5/6) have a real per-session context to query at the approval moment. See §Slice 1 below for the exact diff shape.

### Slice 1 — Wire `src/security.cyr` into agnsh binary
**Deps**: none. **Risk**: low. **Bite size**: small.
- Add `include "src/security.cyr"` to `src/agnsh.cyr` (after `src/audit.cyr`).
- In `main()`, after `alloc_init()` / `args_init()`, construct `var sec = SecurityContext_new(0);` (`restricted=0` for non-interactive; SecurityContext_new auto-detects root via euid and sets restricted=1 if so, emitting the existing stderr warning).
- Thread `sec` into a global or pass it to `interactive_loop()` and `print_intent_result()` — these don't *use* it yet (exec wire-up is slice 5), but having the construction in place validates the security.cyr include graph in the live binary.
- Smoke probe: run as non-root → no warning + clean exit. Run as root → "WARNING: Shell running as root" on stderr.
- **CI**: lint-cstr-str + capacity gate may shift slightly (security.cyr brings in ~9 fns and the /etc/passwd 64 KB read buffer). Verify coverage gate still ≥ 80%.

### Slice 2 — Wire `src/session.cyr` into agnsh + fix 5 deferred MEDIUM findings
**Deps**: slice 1 (SecurityContext available). **Risk**: medium (session.cyr brings in ~15 fns + builtin dispatch + chdir paths). **Bite size**: medium-large.
- Drop the `interactive_loop()` body in `src/agnsh.cyr` in favor of `Session_run_interactive(sec, config)` from `src/session.cyr` (or keep the agnsh-side loop and selectively call into Session_* helpers — there's a design choice here; the lighter touch is to keep agnsh's loop and reach in for cd/mode/history dispatch).
- **Fix `SYS_CHDIR(str_data(Str))` × 2** (audit §7 MEDIUM): the two chdir call sites pass `str_data(dir)` where `dir` is a Str. `str_data` returns a non-null-terminated buffer; the kernel reads past the buffer until it finds a zero. Build a cstring path the same way `agnsh.cyr::audit_log_path` does (manual `alloc + memcpy + store8(buf + len, 0)`), or add a `str_to_cstring(s): cstring` helper in `lib/io.cyr` and use it at both call sites.
- **Fix `SYS_GETCWD` returns unchecked × 3** (audit §6 MEDIUM): each `syscall(SYS_GETCWD, &cwd_buf, 4095)` needs `if (rc < 0) { ... fallback ... }`. Policy decision required: display "?" / use last-known cwd / fail-loudly. Recommend: "?" with a one-line stderr warning, matching the chmod-failure pattern from v1.3.1 slice 4.
- Run the lint shield; should be clean (the str_data(Str) → syscall pattern isn't lint-flagged today; consider adding it as Category F in slice 4 since checkpoint has the same shape).
- **Smoke**: extend `scripts/smoke-test.sh` interactive section with `cd` round-trip probes (`cd /tmp; pwd; exit`) once `pwd` exists.

### Slice 3 — Wire `src/ui.cyr` into agnsh binary
**Deps**: slice 2 (session.cyr may use ui_show_*). **Risk**: low. **Bite size**: small.
- Drop the three `ui_show_*` / `chrono_now_rfc3339` stubs at the top of `src/agnsh.cyr` (the chrono one is already real via `lib/chrono.cyr::iso8601_now`).
- Add `include "src/ui.cyr"` after the other src includes.
- ui.cyr brings welcome/goodbye/help banner helpers + colored output paths. Verify the existing interactive banner in agnsh.cyr doesn't double-print.
- Watch for ui_show_* signature mismatches with existing call sites in session/approval — if any, adjust at the call site (don't rewrite ui.cyr).

### Slice 4 — Wire `src/checkpoint.cyr` into agnsh + fix 1 deferred MEDIUM finding
**Deps**: slice 3. **Risk**: low (single fn delta). **Bite size**: small.
- Add `include "src/checkpoint.cyr"` to `src/agnsh.cyr`.
- **Fix `sys_chmod(str_data(dir), 448)`** (audit §7 MEDIUM): same shape as the session.cyr chdir fix in slice 2. Build a cstring path for the checkpoint directory. The CheckpointManager_new comment in `src/checkpoint.cyr` already TODO-flags this — slice 4 is its natural home.
- **Optional**: extend `scripts/lint-cstr-str.sh` with a Category F catching `syscall(SYS_*, str_data(...))` and `sys_X(str_data(...), ...)` patterns. Decided against this in v1.3.1 §7 because of FP risk on `sys_write` (which legitimately takes `data, len`); now that we have a concrete bug pattern to anchor on, the regex can target `sys_chmod` / `SYS_CHDIR` / `SYS_STAT` / `sys_open` specifically (these all expect a cstring path, not a buffer).

### Slice 5 — Exec wire-up for SAFE / READ_ONLY commands
**Deps**: slice 1 (SecurityContext). **Risk**: medium-high (first real exec; new fail modes). **Bite size**: medium.
- In `print_intent_result()` and `interactive_loop()`'s command dispatch, after `Risk:` / `Hint:` print, **if** `perm == SAFE || perm == READ_ONLY`, call `execute_command(cmd, args, argc)` from `src/security.cyr`. `exec_vec` already wraps fork + execve + waitpid; security.cyr's `execute_command` is the agnoshi-side wrapper that builds argv with `cmd` as `argv[0]`.
- Update `audit_one_shot` to take an `exec_result` parameter (positive = exit code, negative = error). Update `classify_audit_result` to flip `"proposed"` → `"executed"` (rc == 0) / `"failed"` (rc > 0) / `"error"` (rc < 0). The other five labels (`needs_approval`, `blocked`, `needs_llm`, `needs_exec`, `rejected_safety`) stay as-is — they're parse-time decisions, not runtime outcomes.
- Smoke probe additions: `agnsh -c "show files"` should actually run `ls` and print directory listing; audit log shows `"result":"executed"`.
- **Watch out**: stdout/stderr from the child process now appears in agnoshi's output. The existing `Intent: / Command: / Risk:` lines may want to move to stderr to keep stdout clean for downstream piping. Design decision; recommend stderr for the metadata, stdout passes through.

### Slice 6 — Exec wire-up for higher perms (approval flow)
**Deps**: slice 5. **Risk**: high (interactive UI, sudo escalation). **Bite size**: medium.
- For `perm == USER_WRITE / SYSTEM_WRITE / ADMIN`, invoke `ApprovalManager_request(am, cmd, args, argc, risk)` from `src/approval.cyr` before executing.
- ApprovalManager_request today does `syscall(SYS_READ, 0, &buf, 63)` for the approve/deny/modify single-char input. This works in interactive but not in `-c` mode (no stdin). For `-c`, decline by default (current behavior — print "Approval required" and don't execute).
- For ADMIN commands, after approval, route through `execute_with_privileges(sec, cmd, args, argc)` which prepends `sudo -n` and re-verifies sudo path via `verify_sudo_path` (already implemented v1.3.0 slice 5; the TOCTOU window is the gap, documented in ADR-006).
- BLOCKED commands stay blocked — the `WARNING: BLOCKED` line is the final word. No approval flow for BLOCKED.
- Audit: result becomes `"approved_executed"` (or just `"executed"` with `approved=1`), `"denied"` (user pressed d), `"timed_out"` (no input within `ApprovalManager.timeout_seconds`).

### Slice 7 — `undo` builtin
**Deps**: slice 6 (exec must be wired so checkpoint can run *before* exec). **Risk**: low (CheckpointManager logic exists). **Bite size**: small-medium.
- Before each REMOVE/MOVE exec, call `CheckpointManager_checkpoint(cm, intent)` to back up the source file(s) into `$HOME/.agnoshi/checkpoints/`.
- Add `undo` to the interactive_loop builtin dispatch: calls `CheckpointManager_undo(cm)` which pops the last checkpoint and restores.
- Watch the auto-prune: CheckpointManager keeps the most-recent 100 entries; older ones are deleted to keep disk usage bounded.
- Tests: round-trip in a tempdir — `mkdir foo; touch foo/a; agnsh -c "remove foo/a"; agnsh -c "undo"; ls foo/a` should succeed.

### Slice 8 — `src/completion.cyr` stdlib sweep
**Deps**: none (lint pre-flight). **Risk**: low. **Bite size**: small-medium.
- `src/completion.cyr` is the v1.0-era completion engine, not yet wired into any binary. Per the v1.3.0 slice-5/-7/-8 pattern (security/history/audit), modules wired into the binary's include graph after v1.1.0 typically need a Cyrius 4.5 → 5.10 stdlib alignment sweep.
- Run `cyrius check src/completion.cyr` standalone to surface any undefined references.
- Run the lint shield (`sh scripts/lint-cstr-str.sh src`) — it'll flag the obvious cstring/Str mismatches.
- Manually grep for: `fs_exists` (→ `file_exists`), single-arg `file_read_all` (→ buffer-based), `str_cat(cstring, *)` / `str_cat(*, cstring)`, `str_starts_with(*, cstring)`, `str_data(cstring)`, raw `syscall(SYS_OPEN|CHMOD|STAT, ...)`.
- This slice produces NO live-binary change — it's a pre-flight to clear completion.cyr so slice 9 can wire it without doubling as a bug-discovery slice.

### Slice 9 — Tab completion: raw-mode terminal infra
**Deps**: slice 8 (completion.cyr clean). **Risk**: medium-high (termios is its own footgun universe). **Bite size**: large.
- Terminal raw mode via `tcgetattr` / `tcsetattr` — disable `ICANON` (line buffering) + `ECHO` (auto-print), set `VMIN=1` / `VTIME=0` for byte-at-a-time reads.
- Needs syscall wrappers in `lib/io.cyr` for `tcgetattr` / `tcsetattr` if not already there (they're typically `ioctl(fd, TCGETS, ...)` / `TCSETS, ...` on Linux). Both arches need the same constants — check `lib/syscalls_{x86,aarch64}_linux.cyr` first.
- Tab completion engine in `src/completion.cyr` already has a trie / prefix-match design (v1.0-era); slice 9 wires it to the raw-mode read loop.
- ANSI escape handling: arrow keys (`\x1b[A` / `B` / `C` / `D`) for history navigation come along with raw mode. Worth doing in the same slice since the read-loop shape changes anyway.
- **Watch out**: raw mode breaks the existing `read_line` byte-by-byte path; need a clean restore-on-exit (trap signals to put terminal back in cooked mode). Otherwise a crashed agnsh leaves the terminal unusable.

### Slice 10 — O_NOFOLLOW hardening
**Deps**: none. **Risk**: low. **Bite size**: small.
- Per-arch O_NOFOLLOW constant: `0o400000` on x86_64 (`asm-generic/fcntl.h` defines this via the x86-specific override), `0o100000` on aarch64-generic. Add to `lib/syscalls_{x86,aarch64}_linux.cyr` if not present (likely needs an upstream cyrius bump, OR define inline with `#ifdef CYRIUS_ARCH_X86 / AARCH64`).
- Extend `lib/io.cyr::file_write_all` to take an optional flags parameter, OR add a `file_write_all_with_flags(path, buf, len, flags)` variant. agnoshi's `audit_one_shot` uses an explicit open + write + close already (in `audit.cyr::AuditLogger_log`); just OR `O_NOFOLLOW` into the flags arg there. For history.cyr's `file_write_all` call site, switch to the new variant.
- Smoke: pre-place a symlink at `$HOME/.agnsh_audit.log → /tmp/decoy`, run agnsh, verify the symlink is *not* followed (audit log isn't created → check stderr for the EOPENERROR; OR the open fails and audit_one_shot returns -1 silently).

### Slice 11 — ark install-path reconciliation
**Deps**: none. **Risk**: low (release-side only). **Bite size**: small.
- The `ark install --group shell` convention expects specific install paths. Today `scripts/install.sh` installs to `/usr/local/bin/agnsh` + `/usr/local/share/man/man1/agnsh.1` + `/usr/local/share/agnoshi/` (README/CHANGELOG/LICENSE per `docs/guides/getting-started.md`).
- Reconcile against the ark `--group shell` install layout. May need to add `/etc/agnoshi/` for system config (currently agnoshi has no system config; the ShellConfig is built in code).
- The zugot recipe at `~/Repos/zugot/marketplace/agnoshi.cyml` already uses `$PKG/usr/bin/agnsh` — verify that matches the ark install convention. If not, update both.

### Slice 12 — hoosh client wire-up (post-hoosh modernization)
**Deps**: external hoosh modernization complete. **Risk**: medium. **Bite size**: medium.
- New file `src/llm.cyr` (per the v1.0-era `rust-old/src/llm.rs` which had the reference shape — see also v1.3.2 removal notes and git history).
- Connect to hoosh's modernized API surface (likely at `http://127.0.0.1:8088` per the v1.0 default; confirm with the hoosh-side work).
- Includes prompt-injection sanitization (the v1.0 `sanitize_llm_input` helper) — port from `rust-old/src/llm.rs:sanitize_llm_input` via git history (`git log --all --diff-filter=D -- rust-old/src/llm.rs`).
- Watch out: HTTP client needs JSON serialization. `lib/json.cyr` is present and was used by audit_view in v1.3.0 (though the read-side AuditViewer_query was stubbed; this is the time to revisit). Bone up on `json_parse` / `json_get` / `json_get_int`.

### Slice 13 — LLM streaming wire-up (QUESTION + revision)
**Deps**: slice 12 (LLM client). **Risk**: medium. **Bite size**: medium.
- Replace `translate_question`'s "needs LLM" echo with an actual streaming call into the LLM client from slice 12.
- Revision workflow: when intent is UNKNOWN (parser couldn't classify) AND the input looks like NL (not a bare shell command), query the LLM with the original input + recent history + cwd + last exit code for a suggested command. The v0.90.0 CHANGELOG describes this — `suggest_command_with_context` was the v1.0-era name.
- Audit: when LLM is the action, `result` becomes `"llm_suggested"` (new label, document in ADR if it becomes architectural — likely a 7th class for the result vocabulary).
- Tests: with hoosh mocked, verify the streaming output appears progressively (not buffered to completion before display).

### Closeout
- VERSION 1.3.2 → 1.4.0
- CHANGELOG `[Unreleased]` → `[1.4.0]` with consolidated release summary
- Roadmap: move v1.4.0 to Shipped, advance to v1.5.x bucket
- doc-health row refresh
- ADR slot if any: candidates are exec-side approval-vs-execute split, or the LLM result-class addition

### Notes for the next agent
- **Suggested first bite**: slice 1 (security.cyr wire-up). Smallest, no dependencies, gets the SecurityContext scaffolding in place. Reading time vs work-to-do ratio is best here.
- **Parallelism opportunity**: slices 8 (completion lint pre-flight), 10 (O_NOFOLLOW), 11 (ark) are all independent and can interleave with the 1→7 sequence if you hit a hoosh-blocked moment.
- **Don't skip slice 8** before slice 9. The v1.3.0 slice-5/-7/-8 pattern showed every newly-wired deferred module needed a stdlib sweep first; wiring completion.cyr without it would surface 3-5 build breaks at the worst moment (mid-raw-mode-debug).
- **Watch the lint shield** (`scripts/lint-cstr-str.sh`) as new modules come online — the 14 patterns across 5 categories will catch the known bug class; new patterns surfaced during wire-up should land as Category F/G/H additions, not as one-off fixes.
- **Honor ADR-006** for any new Str/cstring boundary: explicit `_in_str` suffix, per-arch syscall wrappers, `str_clone` for static-buf escape, every cstring path null-terminated.
- **bench-history.csv**: rerun benchmarks after slice 5 (exec wire-up) and slice 13 (LLM); both add genuinely new code paths and may shift parse/translate timings.
- **Coverage**: every new wire-up brings more fns into the in-binary scope (denominator grows); add test_core anchors for new modules' pure-logic fns (per v1.2.0 slice 9 + v1.3.0 slice 5 patterns). 80% gate is enforced by CI.

## v1.5.x and beyond — Demand-gated

### Systems features
- Docker compatibility layer — translate docker CLI syntax to stiva commands
- SSH key management — generate, add, list, agent
- VPN/proxy configuration intents
- Systemd timers, sockets, and dependency management
- Log rotation intents
- Diff preview before destructive file operations

### UX features
- AI-powered completion — project-type-aware suggestions (basic raw-mode tab completion lands in v1.4.0)
- History fuzzy search (fzf-style) — extends v1.3.0's persistent history
- Rich prompt themes — extends the v1.3.0 mode-aware prompt
- Man page integration (`explain <cmd>` pulls from man)

### Consumer app translators (from IntentTag stubs)
Stubbed for later — wire up only when the consumer app lands a public surface for agnoshi to translate into.

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
- Stiva (containers) — already partially shipped (12 intents in v0.90)
- Aequi (finance)
- Photis (tasks/habits)

## v2.0.0 — Future major

No scoped work yet. Candidates that would justify a major cut:

- Breaking change to the public API surface (intent enum, translator trait shape, session contract)
- Audit-log format break (would require migration tooling)
- Switch to a different LLM transport (away from hoosh)

Re-evaluate when the v1.2.x bucket is fully shipped.
