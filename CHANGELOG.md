# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Fixed

- **security** — replaced `format!()` JSON construction with `serde_json::json!()` in knowledge.rs, marketplace.rs, and package.rs to prevent JSON injection
- **security** — added URL path sanitization (`sanitize_url_segment`) in marketplace.rs and package.rs to reject path traversal attacks
- **security** — expanded shell metacharacter filtering in misc.rs pipeline validation to include `|`, `<`, `>`, `\n`, `\r`, `!`
- **security** — added null byte validation to network target validation in network.rs
- **interpreter** — fixed `ai_shell::` crate references to `agnoshi::` in all 3 benchmark files
- **interpreter** — collapsed 10 nested `if` statements into `if let` chains (clippy `collapsible_if`)
- **interpreter** — replaced `unwrap()` in patterns.rs regex compilation with `unwrap_or_else` providing error context
- **interpreter** — replaced `unwrap()` in platforms.rs with proper `if let` pattern matching

### Changed

- **api** — added `#[non_exhaustive]` to all public enums: `Intent`, `Mode`, `PermissionLevel`, `ApprovalRequest`, `ApprovalResponse`, `RiskLevel`
- **api** — added `#[must_use]` to pure functions: `Translation::cmd()`, `Mode` boolean methods, `Interpreter::new()`, `Interpreter::parse()`
- **api** — added `#[inline]` to hot-path utility functions in mcp_helper.rs and mode.rs
- **security** — added doc comments to all `PermissionLevel` enum variants
- **security** — added `.context()` to privilege escalation command execution for better error messages
- **deps** — added `MPL-2.0` and `CDLA-Permissive-2.0` to allowed licenses in deny.toml; removed unused `OpenSSL`, `Unicode-DFS-2016`, `Zlib`; set `wildcards = "allow"` for path dependencies

### Added

- **tests** — 33 new security tests: JSON injection prevention (knowledge, marketplace, package), URL sanitization, shell metacharacter filtering, network null byte validation
- **docs** — CLAUDE.md with development process, principles, and DO NOTs

### Performance

- intent_parsing/batch/100: 2.43ms → 1.09ms (−55%)
- intent_parsing/batch/500: 13.1ms → 5.38ms (−59%)
- intent_parsing/parse_translate/15: 411µs → 169µs (−59%)

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
