# Architectural Decision Records

ADRs document the why behind significant architectural decisions.

## Index

- [ADR-001: Port from Rust to Cyrius](001-cyrius-port.md) — 2026-04-13
- [ADR-002: Struct Construction via alloc + store64](002-struct-construction.md) — 2026-04-13
- [ADR-003: Keyword-Based Parser Instead of Regex](003-keyword-parser-over-regex.md) — 2026-04-13
- [ADR-004: Split Translate Dispatch Across Multiple Match Functions](004-split-translate-match.md) — 2026-04-13
- [ADR-005: Explicit String Type Discipline](005-string-type-discipline.md) — 2026-04-13
- [ADR-006: Cyrius cstring/Str Dispatch Discipline (refines ADR-005)](006-cstr-str-dispatch-discipline.md) — 2026-05-11
- [ADR-007: Input Classification — Shell First, Then Specific Before Broad (refines ADR-003)](007-input-classification.md) — 2026-09-23
- [ADR-008: The NL Execution Contract (2.0.0)](008-nl-exec-contract.md) — 2026-09-23, amended 2026-09-26 (2.0.2: restricted sessions)

## Format

Each ADR follows the [MADR-inspired](https://adr.github.io/madr/) template:

- **Context**: what problem motivated the decision
- **Decision**: what was decided
- **Consequences**: positive, negative, neutral
- **Alternatives considered**: what was rejected and why
- **References**: linked evidence

## Writing a new ADR

1. Pick the next sequential number (current: 008, next: 009)
2. Copy an existing ADR as template
3. Update the index in this README
4. Keep it short — an ADR is a decision record, not an essay

ADRs are immutable once accepted. To supersede or refine, write a new ADR referencing the old one (cf. ADR-006 → ADR-005).
