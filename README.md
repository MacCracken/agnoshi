# Agnoshi

**AI-native natural language shell for AGNOS.**

Agnoshi (Sanskrit: not-knowing → discovering through inquiry) is the AI shell for AGNOS. It translates natural language into system commands with human oversight, security approval workflows, and full audit logging.

Written in [Cyrius](https://github.com/MacCracken/cyrius) — a sovereign, self-hosting systems language with zero external dependencies.

**1.9.16 · Cyrius 6.6.6 · 23 modules · ~5 K src lines · 205 KB static binary (DCE, x86_64) · 661 KB aarch64 · 0 runtime deps · 738 unit + 26 security + 358 parse-corpus + 94 smoke tests · 100% host-reachable fn coverage**

## Features

- **Natural language interpretation** — keyword-based intent parser, 44 intent types
- **30+ domain translators** — filesystem, process, network, packages, git, firewall, user/group, services
- **Security-first** — every command classified (SAFE / READ_ONLY / USER_WRITE / SYSTEM_WRITE / ADMIN / BLOCKED), with basename extraction so `/usr/bin/dd` cannot bypass the blocklist
- **Audit logging** — structured JSON log of every action *and every refusal*, with timestamp, user, mode, result and exit code
- **Four modes** — human, assist, auto, strict
- **Single static binary** — `agnsh`, no dynamic dependencies

⚠ **Not shipped yet, though the modules exist in `src/`**: interactive approval
prompts, checkpoint/`undo`, and privilege escalation. `src/checkpoint.cyr` and
`src/security.cyr` are **not in the binary's include graph**, and
`src/approval.cyr` is compiled in only for its risk classifier —
`ApprovalManager_request`, the prompt itself, has no caller. A HIGH-risk command
reports `Approval required` but is not prompted or blocked, there is no `undo`
builtin, and nothing invokes `sudo`. See `docs/development/roadmap.md` (the
1.10.x arc) for the wire-up slices, and `SECURITY.md` for exactly what does and
does not hold today.

## Install

```bash
# Resolve the version-pinned stdlib snapshot into ./lib/ (gitignored).
# Pin lives in cyrius.cyml ([deps] stdlib + cyrius = "6.6.6").
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
src/                                COMPILED INTO THE BINARY (src/agnsh.cyr's include graph)
├── agnsh.cyr         — binary entry point (CLI flags, interactive loop)
├── sanitize.cyr      — input validation, safety predicates, JSON escape
├── statepaths.cyr    — where the state files live ($HOME, or uid-qualified /tmp)
├── mode.cyr          — operating mode (human/assist/auto/strict)
├── permissions.cyr   — command classification, permission levels
├── intent.cyr        — Intent + Translation types, 44 intent tags
├── commands.cyr      — command-line parsing, builtin detection
├── translate.cyr     — 40+ per-intent translators
├── interpreter.cyr   — NL parse + translate pipeline
├── approval.cyr      — risk assessment (the approval UI itself is NOT wired)
├── audit.cyr         — JSON audit log + audit-record construction
├── history.cyr       — command history (persistent, 0600 at open)
└── run_agnos.cyr     — AGNOS launch path: exec, pipelines, redirect, bg jobs
                        (agnos-only; its pure parsers are host-testable)

src/                                PRESENT BUT NOT IN THE BINARY
├── security.cyr      — SecurityContext, privilege escalation
├── session.cyr       — shell session lifecycle, cd/undo builtins
├── checkpoint.cyr    — destructive-op rollback (blocked: 7 missing stdlib symbols)
├── completion.cyr    — tab completion engine
├── prompt.cyr        — prompt rendering with git branch
├── config.cyr        — shell configuration
├── aliases.cyr       — user-defined aliases
├── output.cyr        — output formatting (auto/json/table)
├── ui.cyr            — terminal UI helpers
└── main.cyr          — legacy pre-port entry, never linked
```

⚠ **The split matters.** Anything in the second group is not in the shipped
binary, so features it implements — approval prompts, `undo`, sudo escalation,
tab completion, the git-branch prompt — do not exist at runtime today. Wire-up
slices are in `docs/development/roadmap.md` (the 1.10.x and 1.11.x arcs).

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
  - 006: cstring/Str dispatch discipline (refines 005)
  - 007: Input classification — shell first, then specific before broad (refines 003)
- **Security audit**: `docs/audit/2026-04-13.md` (21 findings, all resolved)
- **Man page**: `docs/agnsh.1` (install via `scripts/install.sh`)

## Security

See `docs/guides/security-model.md` for the deep dive. v1.0.0 closed 21 audit findings (5 critical, 7 high, 9 medium). v1.3.1 P(-1) added a 14-pattern CI lint shield (see `scripts/lint-cstr-str.sh` + ADR-006) that retroactively catches all seven Cyrius 4.5 → 5.10 stdlib-drift bug variants that surfaced over v1.2.0/v1.3.0.

**Key protections that ship today:**
- Command basename extraction (prevents `/usr/bin/dd` bypass of the blocked list)
- Path traversal and shell metacharacters rejected before any launch
- Argument validation on every ADMIN-level translator (`useradd`, `passwd`, `firewall_*`, git)
- Audit log JSON-escaped **and UTF-8 validated**, so one crafted byte cannot make the log unparseable
- State files opened `O_NOFOLLOW`, created 0600, audit-log mode re-asserted every open
- `>` refuses to truncate the shell's own audit log or history, and (AGNOS, 1.9.14) refuses a symlink as its target

⚠ **Documented elsewhere but NOT active in the binary** — the modules exist and
are unwired: terminal-escape stripping in the approval UI and git-branch prompt,
the child-process environment whitelist, and sudo re-verification at escalation
time. `SECURITY.md` marks each one.

## Benchmarks

Headline numbers from the 1.0.0 port-arc snapshot (Rust 0.90 baseline → Cyrius 4.5.0):

| Metric | Rust 0.90 | Cyrius 1.0 | Δ |
|--------|-----------|-----------|---|
| Parse NL→Intent | 32 us | 1 us | **32× faster** |
| Translate Intent→cmd | 167 ns | 680 ns | 4× slower |
| Full pipeline | 32.2 us | 1.7 us | **19× faster** |
| Binary size | 3.8 MB | 146 KB | **−96%** |
| Startup | ~5 ms | microseconds | near-instant |

Full per-benchmark detail in `benchmarks-rust-v-cyrius.md`. Current binary on Cyrius 6.6.6 is 205 KB (x86_64, DCE) / 661 KB (aarch64) — growth over 4.5.0 is toolchain-side (richer stdlib + codegen) plus the v1.2.0/v1.3.0 feature additions (audit, history, the exec paths), not agnoshi-side bloat. ⚠ The aarch64 figure is not like-for-like with x86_64: aarch64 DCE NOPs unreachable functions in place rather than removing them, so about 347 KB of it is unreachable code. Run `cyrius build tests/bench_core.bcyr build/bench_core && ./build/bench_core` for an in-tree refresh; `bench-history.csv` carries the bracketed runs.

## Rust Legacy

The original Rust implementation was preserved in `rust-old/` through the v1.0–v1.3.1 port-arc + hardening cycle (27,251 lines, 62 modules, 1,241 unit tests, 30 criterion benchmarks, version 0.90.0). Removed in v1.3.2 per the AGNOS first-party-standards "Delete `rust-old/` only after the Cyrius version has equal or better test coverage and benchmarks" criterion — v1.3.1's CI lint shield, ADR-006 architectural rigor, 301 + 26 + 59 unit/security/smoke tests, and the bracketed `bench-history.csv` numbers all clear that bar.

The historical record lives in:
- `benchmarks-rust-v-cyrius.md` — the v1.0.0 port-arc comparison frozen by design (Rust 0.90 vs Cyrius 4.5.0)
- `docs/adr/001-cyrius-port.md` — the port rationale
- git history of the v0.x → v1.0.0 commits (`git log --oneline` shows the port slices)

## License

GPL-3.0-only
