# Architectural Decision Records

ADRs document the why behind significant architectural decisions.

## Index

- [ADR-001: Port from Rust to Cyrius](001-cyrius-port.md) — 2026-04-13
- [ADR-002: Struct Construction via alloc + store64](002-struct-construction.md) — 2026-04-13
- [ADR-003: Keyword-Based Parser Instead of Regex](003-keyword-parser-over-regex.md) — 2026-04-13
- [ADR-004: Split Translate Dispatch Across Multiple Match Functions](004-split-translate-match.md) — 2026-04-13
- [ADR-005: Explicit String Type Discipline](005-string-type-discipline.md) — 2026-04-13

## Format

Each ADR follows the [MADR-inspired](https://adr.github.io/madr/) template:

- **Context**: what problem motivated the decision
- **Decision**: what was decided
- **Consequences**: positive, negative, neutral
- **Alternatives considered**: what was rejected and why
- **References**: linked evidence

## Writing a new ADR

1. Pick the next sequential number (current: 005, next: 006)
2. Copy an existing ADR as template
3. Update the index in this README
4. Keep it short — an ADR is a decision record, not an essay

ADRs are immutable once accepted. To supersede, write a new ADR referencing the old one.
