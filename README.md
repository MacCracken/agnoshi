# Agnoshi

**AI-native natural language shell for AGNOS.**

Agnoshi (Sanskrit: not-knowing → discovering through inquiry) is the AI shell for AGNOS. It translates natural language into system commands with human oversight, security approval workflows, and full audit logging.

Written in [Cyrius](https://github.com/MacCracken/cyrius) — a sovereign, self-hosting systems language with zero external dependencies.

**1.4.5 · Cyrius 6.0.56 · 21 modules · ~5 K src lines · 295 KB static binary (DCE, x86_64) · 340 KB aarch64 · 0 runtime deps · 301 unit + 26 security + 59 smoke tests**

## Features

- **Natural language interpretation** — keyword-based intent parser, 44 intent types
- **30+ domain translators** — filesystem, process, network, packages, git, firewall, user/group, services
- **Security-first** — every command classified (SAFE / READ_ONLY / USER_WRITE / SYSTEM_WRITE / ADMIN / BLOCKED)
- **Approval workflows** — risky operations require explicit human approval
- **Checkpoint/undo** — destructive ops (rm, mv) backed up before execution
- **Audit logging** — structured JSON log of every action with timestamp, user, mode, result
- **Four modes** — human, assist, auto, strict
- **Single static binary** — `agnsh`, no dynamic dependencies

## Install

```bash
# Resolve the version-pinned stdlib snapshot into ./lib/ (gitignored).
# Pin lives in cyrius.cyml ([deps] stdlib + cyrius = "6.0.56").
cyrius deps

# Build from source
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
├── agnsh.cyr         — binary entry point (CLI flags, interactive loop)
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

See `docs/guides/security-model.md` for the deep dive. v1.0.0 closed 21 audit findings (5 critical, 7 high, 9 medium). v1.3.1 P(-1) added a 14-pattern CI lint shield (see `scripts/lint-cstr-str.sh` + ADR-006) that retroactively catches all seven Cyrius 4.5 → 5.10 stdlib-drift bug variants that surfaced over v1.2.0/v1.3.0.

**Key protections:**
- Command basename extraction (prevents `/usr/bin/dd` bypass of blocked list)
- Path traversal blocked (`../` rejected)
- Terminal escape sanitization (approval UI, git branch display)
- Environment whitelist for privileged subprocesses (no LD_PRELOAD inheritance)
- JSON-escaped audit log (no injection)
- Sudo re-verified at escalation time (path + root ownership check)

## Benchmarks

Headline numbers from the 1.0.0 port-arc snapshot (Rust 0.90 baseline → Cyrius 4.5.0):

| Metric | Rust 0.90 | Cyrius 1.0 | Δ |
|--------|-----------|-----------|---|
| Parse NL→Intent | 32 us | 1 us | **32× faster** |
| Translate Intent→cmd | 167 ns | 680 ns | 4× slower |
| Full pipeline | 32.2 us | 1.7 us | **19× faster** |
| Binary size | 3.8 MB | 146 KB | **−96%** |
| Startup | ~5 ms | microseconds | near-instant |

Full per-benchmark detail in `benchmarks-rust-v-cyrius.md`. Current binary on Cyrius 6.0.56 is 295 KB (x86_64) / 340 KB (aarch64) — toolchain-side growth from richer stdlib + codegen between 4.5.0 and 6.0.x plus the v1.2.0/v1.3.0 feature additions (approval, audit, history, security wired in), not from agnoshi-side bloat. Run `cyrius build tests/bench_core.bcyr build/bench_core && ./build/bench_core` for an in-tree refresh; `bench-history.csv` carries the bracketed runs.

## Rust Legacy

The original Rust implementation was preserved in `rust-old/` through the v1.0–v1.3.1 port-arc + hardening cycle (27,251 lines, 62 modules, 1,241 unit tests, 30 criterion benchmarks, version 0.90.0). Removed in v1.3.2 per the AGNOS first-party-standards "Delete `rust-old/` only after the Cyrius version has equal or better test coverage and benchmarks" criterion — v1.3.1's CI lint shield, ADR-006 architectural rigor, 301 + 26 + 59 unit/security/smoke tests, and the bracketed `bench-history.csv` numbers all clear that bar.

The historical record lives in:
- `benchmarks-rust-v-cyrius.md` — the v1.0.0 port-arc comparison frozen by design (Rust 0.90 vs Cyrius 4.5.0)
- `docs/adr/001-cyrius-port.md` — the port rationale
- git history of the v0.x → v1.0.0 commits (`git log --oneline` shows the port slices)

## License

GPL-3.0-only
