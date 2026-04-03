# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Changed
- **deps** — `agnosys` dependency temporarily switched to local path for musl static build (pending agnosys release with ioctl fix)

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

### Changed

- **mode** — `Mode` now derives `Copy` (all unit variants); removed unnecessary `.clone()` calls
- **mode** — `toggle()` now returns `Result<()>` (was `()`)
- **security** — moved `echo` out of `safe` list (was dead entry; already in `read_only` which is checked first)
- **deps** — replaced `once_cell::sync::Lazy` with `std::sync::LazyLock` (stable in Rust 1.89)
- **deps** — removed `once_cell` dependency
- **deps** — added `agnosys` git URL to `deny.toml` `allow-git`
- **api** — added `#[must_use]` to 20+ pure functions across security, permissions, commands, aliases, completion, history, output modules
- **tests** — 1,109 unit tests (up from 1,096); 13 new tests covering regex fixes, security hardening, JSON injection, cache behavior, mode toggle

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
