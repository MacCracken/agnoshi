# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [1.1.0] - 2026-05-10

Repair-focused modernization. No new shell features — toolchain bump + scaffolding parity with the rest of the AGNOS ecosystem.

### Changed
- **toolchain** — Cyrius pin bumped 4.5.0 → 5.10.34 (latest stable). Pin now lives in `cyrius.cyml` (`cyrius = "5.10.34"`); the standalone `.cyrius-toolchain` file was retired.
- **manifest** — `cyrius.toml` → `cyrius.cyml`. Package version is no longer hand-edited in the manifest — `version = "${file:VERSION}"` reads `VERSION` at toolchain-resolve time, so `VERSION` is the only file the release process touches.
- **lib/** — vendored stdlib stubs removed from the tree; `./lib/` is gitignored. `cyrius deps` repopulates from the version-pinned stdlib snapshot referenced in `[deps] stdlib` (matches the agnosys / yukti / patra convention). Prevents prior-version stubs from sitting in tree across toolchain bumps.
- **ci** — agnosys-parity gate set: syntax check (`cyrius check`), fmt diff-check, lint with warn-as-error, vet (include-graph audit), capacity gate, aarch64 best-effort cross-build, security-pattern scan (raw execve / shadow access / large fn-scope buffers), version-consistency gate (`VERSION` ↔ `CHANGELOG.md` ↔ `cyrius.cyml ${file:VERSION}`), required-docs check now includes `CLAUDE.md`, `docs/development/roadmap.md`, and `docs/doc-health.md`.
- **release** — accepts both `vX.Y.Z` and `X.Y.Z` tag styles; semver shape verified; SHA256SUMS published alongside source archive + per-arch binaries; pre-release flag auto-set for `0.x` tags.
- **scripts/version-bump.sh** — touches only `VERSION` now (was editing both `VERSION` and `cyrius.toml`); the manifest substitutes automatically via `${file:VERSION}`.
- **CLAUDE.md** — cleanliness gates rewritten from Rust toolchain (`cargo fmt/clippy/audit/deny/doc`) to Cyrius equivalents (`cyrius check/fmt/lint/vet/capacity`); P(-1) and Work Loop sections refreshed; version-discipline rules (VERSION is single SoT, `./lib/` never committed) added under Key Principles and DO NOT.
- **docs/development/roadmap.md** — reshaped: shipped items dated, 1.1.0 in flight, post-v1.0 polish items slotted across 1.2.0 (intent parsing + translators), 1.2.1 (approval + interactive shell), 1.2.2 (zugot packaging); demand-gated systems / UX / consumer-app translator items moved to v1.3.x+.

### Added
- **docs/doc-health.md** — living doc-currency ledger (fresh / stale / archived / open-question), agnoshi-shaped tiers, initial audit covering ~26 markdown files plus the `agnsh.1` man page. Refreshed opportunistically when docs are touched (paired with each minor-cut closeout step per CLAUDE.md Work Loop §10).

### Fixed
- **release.yml** — was building `src/main.cyr → agnoshi` (the pre-port Rust entry / pre-rename binary), but `cyrius.cyml [build]` specifies `src/agnsh.cyr → agnsh`. Releases would have shipped the wrong binary name. Release workflow now builds and archives `agnsh`.
- **lint cleanup** — Cyrius 5.10.x added a 120-character line-length lint. Wrapped 49 long lines across `src/interpreter.cyr` (16), `src/translate.cyr` (32, mostly `Translation_new(...)` call sites), and `src/permissions.cyr` (1). Behavior unchanged; CI's lint gate now reports zero warnings.
- **fmt drift** — Cyrius 5.10.x formatter rules differ from 4.5.0. Re-formatted 5 files (`commands.cyr`, `permissions.cyr`, `session.cyr`, `translate.cyr`, `ui.cyr`) so the fmt diff-gate is clean.
- **CLAUDE.md Known Issues** — purged two stale entries: (1) the "ModeManager undefined variable" build-error note (the struct is defined in `src/mode.cyr:8` — the note was a leftover from a mid-port debugging session); (2) the "cc3 function/token limit" comment in `benchmarks-rust-v-cyrius.md` (cc3 is retired, the current Cyrius compiler has no such limit; the doc has been re-classified as historical in `docs/doc-health.md`).

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
