# Agnoshi — Claude Code Instructions

## Project Identity

**Agnoshi** (Sanskrit: not-knowing — discovering through inquiry) — AI-native natural language shell for AGNOS

- **Type**: Single static binary (`agnsh`)
- **Language**: Cyrius (pin lives in `cyrius.cyml`)
- **License**: GPL-3.0-only
- **Version**: SemVer, single source of truth in `VERSION` (manifest pulls via `${file:VERSION}`)
- **Genesis repo**: [agnosticos](https://github.com/MacCracken/agnosticos)
- **Philosophy**: [AGNOS Philosophy & Intention](https://github.com/MacCracken/agnosticos/blob/main/docs/philosophy.md)
- **Standards**: [First-Party Standards](https://github.com/MacCracken/agnosticos/blob/main/docs/development/applications/first-party-standards.md)
- **Recipes**: [zugot](https://github.com/MacCracken/zugot) — takumi build recipes

## Consumers

None yet — standalone shell binary extracted from `agnosticos/userland/ai-shell/`.

## Dependency Stack

```
agnoshi (AI natural language shell, Cyrius)
  ├── stdlib snapshot (resolved by `cyrius deps` into ./lib/, gitignored)
  ├── agnos-common (common types/utilities for AGNOS — when wired)
  ├── agnosys (kernel interface — syscalls, Landlock, seccomp, TPM)
  └── bote (MCP core — JSON-RPC 2.0, tool registry, audit)
```

## Development Process

### P(-1): Scaffold Hardening (before any new features)

0. Read roadmap, CHANGELOG, and `docs/doc-health.md` — know what was intended and what's stale before auditing what was built
1. `cyrius deps` to repopulate `./lib/` from the pinned stdlib snapshot
2. Test + benchmark sweep of existing code
3. Cleanliness gates (match CI):
   - `cyrius check src/*.cyr` (syntax)
   - `cyrius fmt <file>` diff against tree (fmt-drift gate)
   - `cyrius lint <file>` — warn-as-error
   - `cyrius vet src/agnsh.cyr` (include-graph audit)
   - `cyrius capacity --check src/agnsh.cyr` (fn-table / code-size headroom)
4. Get baseline benchmarks (`./scripts/bench-history.sh`)
5. Internal deep review — gaps, optimizations, security, logging/errors, docs
6. External research — domain completeness, missing capabilities, best practices
7. Cleanliness gates — must be clean after review
8. Additional tests/benchmarks from findings
9. Post-review benchmarks — prove the wins
10. Documentation audit — `docs/doc-health.md` refresh, ADRs, source citations, guides, examples
11. Repeat if heavy

### Work Loop (continuous)

1. Work phase — new features, roadmap items, bug fixes
2. Cleanliness gates (the same 5 from P(-1) step 3)
3. Test + benchmark additions for new code
4. Run benchmarks (`./scripts/bench-history.sh`)
5. Internal review — performance, memory, security, correctness
6. Cleanliness gates — must be clean after audit
7. Deeper tests/benchmarks from audit observations
8. Run benchmarks again — prove the wins
9. If audit heavy → return to step 5
10. Documentation — update CHANGELOG, roadmap, doc-health, ADRs for design decisions, guides for new API surface, verify recipe version in zugot
11. Version sync — `VERSION` is the only file that gets edited; `cyrius.cyml` pulls it via `${file:VERSION}`; the zugot recipe is bumped separately
12. Return to step 1

### Task Sizing

- **Low/Medium effort**: Batch freely — multiple items per work loop cycle
- **Large effort**: Small bites only — break into sub-tasks, verify each before moving to the next. Never batch large items together
- **If unsure**: Treat it as large. Smaller bites are always safer than overcommitting

### Refactoring

- Refactor when the code tells you to — duplication, unclear boundaries, performance bottlenecks
- Never refactor speculatively. Wait for the third instance before extracting an abstraction
- Refactoring is part of the work loop, not a separate phase. If a review (step 5) reveals structural issues, refactor before moving to step 6
- Every refactor must pass the same cleanliness + benchmark gates as new code

### Key Principles

- **Never skip benchmarks.** Numbers don't lie. The CSV history is the proof.
- **Tests + benchmarks are the way.** 80%+ coverage target.
- **Own the stack.** Use AGNOS ecosystem types, not raw re-exports.
- **No magic.** Every operation is measurable, auditable, traceable.
- **Single source of truth for version** — `VERSION` file; `cyrius.cyml` pulls via `${file:VERSION}`.
- **Pin the toolchain in `cyrius.cyml`** — CI reads `cyrius = "..."` from the manifest.
- **`./lib/` is gitignored** — `cyrius deps` repopulates from the pinned snapshot; never check stdlib stubs into the tree.
- **Security first** — all commands go through approval workflows and sandbox execution.

## DO NOT

- **Do not commit or push** — the user handles all git operations (commit, push, tag)
- **NEVER use `gh` CLI** — use `curl` to GitHub API only
- Do not add unnecessary dependencies — keep it lean
- Do not skip benchmarks before claiming performance improvements
- Do not bypass the approval/security workflow — every command must be auditable
- Do not check `./lib/` into the tree — `cyrius deps` repopulates it from the pinned snapshot
- Do not edit `version = "..."` in `cyrius.cyml` — bump `VERSION` instead

## Documentation Structure

```
Root files (required):
  README.md, CHANGELOG.md, CLAUDE.md, CONTRIBUTING.md, SECURITY.md, CODE_OF_CONDUCT.md, LICENSE, VERSION

docs/ (required):
  development/roadmap.md  — shipped, in-flight, future
  doc-health.md           — living doc-currency ledger (fresh / stale / archived / open-question)
  audit/                  — dated P(-1) hardening reports
  agnsh.1                 — man page

docs/ (when earned):
  architecture/overview.md — module map, data flow, consumers
  adr/                     — architectural decision records
  guides/                  — usage guides, integration patterns
  examples/                — worked examples
  standards/               — external spec conformance
  compliance/              — regulatory, audit, security compliance
  sources.md               — source citations for algorithms/formulas
```

## CHANGELOG Format

Follow [Keep a Changelog](https://keepachangelog.com/):

```markdown
# Changelog

## [Unreleased]
### Added — new features
### Changed — changes to existing features
### Fixed — bug fixes
### Removed — removed features
### Security — vulnerability fixes
### Performance — benchmark-proven improvements (include numbers)

## [X.Y.Z] - YYYY-MM-DD
### Added
- **module_name** — what was added and why
### Changed
- item: old behavior → new behavior
### Fixed
- issue description (root cause → fix)
### Performance
- benchmark_name: before → after (−XX%)
```

Rules:
- Every PR/commit that changes behavior gets a CHANGELOG entry
- Performance claims MUST include benchmark numbers
- Breaking changes get a **Breaking** section with migration guide
- Group by module when multiple changes in one release
- Link to ADR if a change was driven by an architectural decision
