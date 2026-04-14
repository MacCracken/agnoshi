# ADR-005: Explicit String Type Discipline

**Status:** Accepted (2026-04-13)

## Context

Cyrius has two string conventions that look similar but are binary-incompatible:

**cstring** — null-terminated `char *`:
- String literals default to this type: `"hello"`
- Operations in `lib/string.cyr`: `strlen`, `streq`, `memcpy`, `memchr`
- Pointer arithmetic works: `load8(s + i)` reads byte at offset `i`

**Str** — fat pointer struct `{data: ptr, len: i64}` (16 bytes):
- Created by `str_from(cstr)` or `str_new(ptr, len)`
- Operations in `lib/str.cyr`: `str_len`, `str_trim`, `str_sub`, `str_cat`,
  `str_contains`, `str_split`
- Access via `str_data(s)` and `str_len(s)`

Calling the wrong set of functions on the wrong type causes runtime
segfaults — `str_len("hello")` does `load64(cstring_pointer)` which reads
the first 8 bytes of the string content as an integer (usually huge),
then `str_sub` runs off into memory.

During the port, modules casually mixed both conventions, causing the
benchmark and test suite to segfault until the types were untangled.

## Decision

Every function picks one convention and sticks with it. Convert at
boundaries using `str_from(cstring) -> Str`.

| Module | Convention | Rationale |
|--------|-----------|-----------|
| `sanitize.cyr` | cstring | Validates shell args / paths which come in as literals |
| `permissions.cyr` | cstring | Compares against literal command names |
| `interpreter.cyr` | Str | Needs `str_trim`, `str_sub`, `str_contains` for parsing |
| `translate.cyr` | Str | Reads fields that interpreter stored as Str |
| `audit.cyr` | Str | String building via `str_builder` |
| Tests / benches | cstring literals → `str_from()` at call site | Convert at boundary |

Two parallel helper sets live in `sanitize.cyr` when needed:
- `has_path_traversal` (Str) and `has_path_traversal_cstr` (cstring)
- `has_shell_metachars` (Str) and `has_shell_metachars_cstr` (cstring)
- `cstr_starts_with` (cstring version of `str_starts_with`)

## Consequences

### Positive
- No more random segfaults from type confusion
- Each module's internal calls are consistent
- Boundary conversions are explicit and searchable (`grep str_from`)

### Negative
- Some code duplication in sanitize helpers (cstr vs Str variants)
- Developers must track which type each function expects
- Cannot write "generic" string functions that work for both

### Mitigation
- Function names use suffixes where conversion exists (`_cstr` vs Str default)
- `CONTRIBUTING.md` documents the convention per module
- Tests cover each helper in both type contexts where relevant

## Alternatives Considered

1. **Use only cstring**: rejected — no `trim`, `sub`, `contains` for
   parsing; reinventing those is more work than conversion
2. **Use only Str**: rejected — `permissions.cyr` compares against 60+
   literal command names. Converting all of them to Str at every call
   would allocate pointlessly.
3. **Unified string type in a wrapper**: rejected — fights the language.
   Cyrius idiom is primitives-first; hiding type is anti-pattern.

## References
- `feedback_cyrius_strings` memory
- `src/sanitize.cyr` — dual-convention validation helpers
- `lib/string.cyr` vs `lib/str.cyr` — the two stdlib modules
