# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

First slice of v1.2.1. Lead roadmap item is "approval workflow battle-tested interactively"; this cut starts that with unit coverage of `src/approval.cyr` (decision matrix, risk mapping, blocklist, auto-approve toggle — every non-UI surface that was previously untested) AND a follow-on bug-class audit pass that caught 7 more `str_cat(X, "cstring")` sites — same Cyrius 4.5 → 5.10 stdlib drift, but in the *second*-arg position which slice 8's grep missed.

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
