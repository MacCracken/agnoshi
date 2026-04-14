# ADR-001: Port from Rust to Cyrius

**Status:** Accepted (2026-04-13)

## Context

Agnoshi was originally written in Rust (27,251 lines across 62 modules, 24 crate
dependencies, 3.8 MB dynamically linked binary). The AGNOS ecosystem is
migrating to Cyrius — a sovereign, self-hosting systems language — to eliminate
external toolchain dependencies and produce tiny static binaries suitable for
distribution with the hardened server OS image.

## Decision

Port the entire core shell to Cyrius. Keep the Rust implementation in
`rust-old/` for reference during the port.

## Consequences

### Positive
- **Binary size**: 3.8 MB → 146 KB (−96%)
- **Parse throughput**: 32 us → 1 us per command (**32× faster**)
- **Startup time**: ~5 ms (dynamic linker) → microseconds (static ELF)
- **Dependency count**: 24 crates → 0 external runtime deps
- **Build reproducibility**: 29 KB bootstrap binary vs multi-GB Rust toolchain
- **Codebase**: 27K lines → 4K lines (no type annotations, no derive macros,
  no async runtime, no trait impls)

### Negative
- **Per-call translation cost**: 167 ns → 680 ns (4× slower). Acceptable
  because total pipeline (parse + translate) is still 19× faster.
- **Ecosystem**: no regex crate, no tokio, no serde — replaced with
  keyword parsing, synchronous I/O, manual JSON.
- **Language maturity**: Cyrius v4.3 is newer than Rust; some features
  (generics, operator overloading, full async) not yet available.
- **Consumer app translators** (17 domains) deferred — tagged in the Intent
  enum but not implemented. Will be added when those downstream packages
  are also ported.

### Neutral
- **String types**: Cyrius has two string conventions (cstring via
  `lib/string.cyr`, Str via `lib/str.cyr`). Code must be explicit which
  it uses — see `feedback_cyrius_strings` memory and CONTRIBUTING.md.
- **Struct construction**: use `alloc + store64` pattern, not struct
  literals. Field access via `load64(p + OFFSET)`.

## Alternatives Considered

1. **Stay on Rust**: rejected because the whole AGNOS ecosystem is migrating
   — keeping one outlier is churn, not savings.
2. **Port in place** (single PR): rejected in favor of `rust-old/` preservation
   so diffs remain reviewable and nothing is lost.
3. **Partial port** (Rust library + Cyrius frontend): rejected — would
   inherit Rust's binary size and dependency graph.

## References
- `benchmarks-rust-v-cyrius.md` — head-to-head performance
- `rust-old/` — preserved Rust implementation
- `docs/audit/2026-04-13.md` — security audit of the ported code
