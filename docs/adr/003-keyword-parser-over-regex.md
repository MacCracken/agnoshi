# ADR-003: Keyword-Based Parser Instead of Regex

**Status:** Accepted (2026-04-13)

## Context

The Rust implementation used 150+ compiled regex patterns (via `regex` crate
and `LazyLock<HashMap>`) to classify natural language input. This consumed:
- ~200 us of LazyLock initialization on first use
- ~30 us per command parse
- A `regex` crate dependency (~350 KB of machine code)

Cyrius has no regex library. The stdlib provides `lib/regex.cyr` for
**glob matching only** (not full regex).

## Decision

Replace regex matching with keyword-based parsing using case-insensitive
substring search (`str_contains_ci`, `str_find_ci`). Each intent is
classified by the presence of trigger keywords.

## Consequences

### Positive
- **32× faster parsing**: 32 us → 1 us per command
- No library init cost
- No regex dependency
- Patterns are readable Cyrius code, not opaque regex strings
- Easier to extend — add a new intent = add an `if` branch

### Negative
- **Less expressive**: cannot match backreferences, lookaheads, or
  character classes
- **False positives possible**: "show me all shows" may classify as
  SHOW_FILE rather than some media intent. Mitigated by parse ordering
  (most-specific-first).
- **Requires ordering discipline**: later branches must not shadow earlier
  ones that should have matched.

### Neutral
- **Extensibility**: regex patterns were data; keywords are code. Trade-off
  is control-flow readability vs declarativeness.

## Implementation

The main parser is split across multiple functions to stay within cc3's
per-function emission limits (see ADR-004):

```cyrius
fn Interpreter_parse(self, input) {
    var trimmed = str_trim(input);
    # pipeline detection (top priority)
    if (str_contains(trimmed, " | ") == 1) { ... }

    # dispatch to domain parsers
    if (input_has_word(trimmed, "show") == 1) {
        var r = parse_show_commands(trimmed);
        if (r != 0) { return r; }
    }
    var r = parse_file_ops(trimmed);
    if (r != 0) { return r; }
    var r = parse_system_ops(trimmed);
    if (r != 0) { return r; }
    # ... fallthrough to SHELL_COMMAND
}
```

## Alternatives Considered

1. **Write a Cyrius regex engine**: rejected as out-of-scope. Full PCRE
   semantics are a substantial library.
2. **Use `lib/regex.cyr` glob matching**: insufficient — shell intent
   phrases need word boundaries and alternation.
3. **Lex + grammar parser**: overkill for NL inputs that are mostly keyword
   soup. A formal grammar would be harder to extend.

## References
- `benchmarks-rust-v-cyrius.md` — parse performance comparison
- `src/interpreter.cyr` — actual parser implementation
- `lib/regex.cyr` — Cyrius stdlib regex module (glob only)
