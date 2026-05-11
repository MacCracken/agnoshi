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

## v1.3.x — Polish bucket — *closed*

All v1.3.x items shipped. Next anchor is v1.4.0 below.

*(empty — packaging items moved to v1.4.0 since ark packaging belongs alongside exec wire-up. zugot recipe is updated in-tree at `/home/macro/Repos/zugot/marketplace/agnoshi.cyml` as the upstream changes land — no release-coupled bumps needed for that.)*

## v1.4.0 — Exec wire-up + hoosh modernization

The remaining v1.2.1-scoped items that needed broader infrastructure, plus the deferred MEDIUM findings from v1.3.1 P(-1):

### Core
- [ ] **Exec wire-up for SAFE / READ_ONLY commands** — `print_intent_result` currently *proposes* the command and audits with `result=proposed`; v1.4.0 adds actual `exec_vec(argv)` and flips `result` to `executed` / `denied` / `error` at the call site. Per-session SecurityContext on startup; sudo-escalation path through `execute_with_privileges` already wired in `src/security.cyr` (v1.3.0 slice 5).
- [ ] **`undo` builtin** — wires `src/checkpoint.cyr` (already stdlib-aligned + chmod-return-checked in v1.3.1 slice 4) for destructive-op rollback. Needs exec wire-up first.
- [ ] **LLM response streaming** — requires hoosh modernization (hoosh itself is still on the pre-Cyrius API; that work happens in the hoosh repo first, then agnoshi consumes the modernized surface).
- [ ] **Tab completion** — terminal raw mode (`tcsetattr` + termios), tty escape sequence handling, completion engine wired to `src/completion.cyr` (will need its own Cyrius 4.5 → 5.10 sweep first per the v1.3.0 slice-5 / -7 / -8 pattern).

### Packaging
- [ ] **ark install path reconciliation** — match the `ark install --group shell` install convention (was scoped for v1.3.2; moved here to land alongside exec wire-up since both touch the binary's runtime contract).

### v1.3.1 P(-1) deferred findings
Carry-overs from `docs/audit/2026-05-11-pminus1.md` §6-§8 that need wire-up to land:
- [ ] **session.cyr `SYS_CHDIR(str_data(Str))`** — non-null-terminated buffer to kernel. MEDIUM. Build a cstring path the same way `agnsh.cyr::audit_log_path` does (or add a `str_to_cstring(s)` helper in `lib/io.cyr`). 2 sites.
- [ ] **checkpoint.cyr `sys_chmod(str_data(Str))`** — same shape. MEDIUM. 1 site.
- [ ] **session.cyr `SYS_GETCWD` returns unchecked** — 3 sites. MEDIUM. Needs fallback-path policy (display "?" / use last-known cwd / fail-loudly).
- [ ] **O_NOFOLLOW on audit/history opens** — LOW. Per-arch flag value (`0o400000` x86 vs `0o100000` aarch64) needs a constant + `lib/io.cyr` API extension to thread custom flags through `file_write_all`.

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
