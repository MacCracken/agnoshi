# Agnoshi

**AI-native natural language shell for AGNOS.**

Agnoshi (Sanskrit: not-knowing → discovering through inquiry) is the AI shell for AGNOS. It translates natural language into system commands with human oversight, security approval workflows, and full audit logging.

Written in [Cyrius](https://github.com/MacCracken/cyrius) — a sovereign, self-hosting systems language with zero external dependencies.

## Features

- **Natural language interpretation** — keyword-based intent parser, 44 intent types
- **30+ domain translators** — filesystem, process, network, packages, git, firewall, user/group, services
- **Security-first** — every command classified (SAFE / READ_ONLY / USER_WRITE / SYSTEM_WRITE / ADMIN / BLOCKED)
- **Approval workflows** — risky operations require explicit human approval
- **Checkpoint/undo** — destructive ops (rm, mv) backed up before execution
- **Audit logging** — structured JSON log of every action with timestamp, user, mode, result
- **Four modes** — human, assist, auto, strict
- **Static binary** — 146 KB, no dynamic dependencies

## Install

```bash
# Build from source (requires cyrius compiler)
cyrius build src/agnsh.cyr build/agnsh

# Install to /usr/local/bin
sudo sh scripts/install.sh
```

## Usage

```bash
agnsh                           # interactive shell
agnsh -c "show me all files"    # one-shot command
agnsh --version                 # print version
agnsh --help                    # show usage
```

## Architecture

```
src/
├── agnsh.cyr         — binary entry point (v1.0 minimal)
├── sanitize.cyr      — input validation, JSON escape, env whitelist
├── mode.cyr          — operating mode (human/assist/auto/strict)
├── permissions.cyr   — command classification, permission levels
├── intent.cyr        — Intent + Translation types, 44 intent tags
├── interpreter.cyr   — NL parse + translate pipeline
├── translate.cyr     — 40+ per-intent translators
├── commands.cyr      — command-line parsing, builtin detection
├── approval.cyr      — risk assessment, human approval UI
├── security.cyr      — SecurityContext, privilege escalation
├── session.cyr       — shell session lifecycle
├── checkpoint.cyr    — destructive op rollback
├── audit.cyr         — JSON audit log
├── history.cyr       — command history (persistent, 0600 perms)
├── aliases.cyr       — user-defined aliases
├── completion.cyr    — tab completion engine
├── config.cyr        — shell configuration
├── output.cyr        — output formatting (auto/json/table)
├── prompt.cyr        — prompt rendering
└── ui.cyr            — terminal UI helpers
```

## Documentation

- **Getting started**: `docs/guides/getting-started.md`
- **Writing new intents**: `docs/guides/writing-intents.md`
- **Security model**: `docs/guides/security-model.md`
- **Architecture**: `docs/architecture/overview.md`
- **Examples**:
  - Common commands: `docs/examples/common-commands.md`
  - Scripting: `docs/examples/scripting.md`
  - Server hardening: `docs/examples/server-hardening.md`
- **ADRs** (architectural decisions):
  - 001: Port from Rust to Cyrius
  - 002: Struct construction via alloc + store64
  - 003: Keyword parser instead of regex
  - 004: Split translate match across functions
  - 005: String type discipline (cstring vs Str)
- **Security audit**: `docs/audit/2026-04-13.md` (21 findings, all resolved)
- **Man page**: `docs/agnsh.1` (install via `scripts/install.sh`)

## Security

See `docs/guides/security-model.md` for the deep dive. All 21 audit findings resolved as of v1.0.0.

**Key protections:**
- Command basename extraction (prevents `/usr/bin/dd` bypass of blocked list)
- Path traversal blocked (`../` rejected)
- Terminal escape sanitization (approval UI, git branch display)
- Environment whitelist for privileged subprocesses (no LD_PRELOAD inheritance)
- JSON-escaped audit log (no injection)
- Sudo re-verified at escalation time (path + root ownership check)

## Benchmarks

See `benchmarks-rust-v-cyrius.md` for full Rust-vs-Cyrius comparison.

| Metric | Rust 0.90 | Cyrius 1.0 | Δ |
|--------|-----------|-----------|---|
| Parse NL→Intent | 32 us | 1 us | **32× faster** |
| Translate Intent→cmd | 167 ns | 680 ns | 4× slower |
| Full pipeline | 32.2 us | 1.7 us | **19× faster** |
| Binary size | 3.8 MB | 146 KB | **−96%** |
| Startup | ~5 ms | microseconds | near-instant |

## Rust Legacy

The original Rust implementation is preserved in `rust-old/` for reference during the port.
27,251 lines, 62 modules, 1,241 unit tests, 30 criterion benchmarks, version 0.90.0.

## License

GPL-3.0-only
