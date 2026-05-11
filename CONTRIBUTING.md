# Contributing to Agnoshi

Thank you for your interest in contributing to Agnoshi.

## Getting Started

1. Fork the repository
2. Create a feature branch
3. Install Cyrius (the pin lives in `cyrius.cyml`: `cyrius = "5.10.44"`). Toolchain releases: https://github.com/MacCracken/cyrius/releases
4. `cyrius deps` — resolves the version-pinned stdlib snapshot into `./lib/` (gitignored)
5. Make your changes
6. Run the cleanliness gates + `sh tests/test.sh` to verify
7. Submit a pull request

## Development

```bash
# Resolve the version-pinned stdlib snapshot (./lib/ is gitignored)
cyrius deps

# Build the binary
cyrius build src/agnsh.cyr build/agnsh

# Run all tests (unit + security + smoke + bench)
sh tests/test.sh

# Individual suites
cyrius build tests/test_core.tcyr build/test_core && ./build/test_core
cyrius build tests/test_security.tcyr build/test_security && ./build/test_security
cyrius build tests/bench_core.bcyr build/bench_core && ./build/bench_core
sh scripts/smoke-test.sh build/agnsh
```

### Cleanliness gates

These match the CI gate set. Run them before pushing — any drift fails the build.

```bash
cyrius check src/agnsh.cyr             # syntax (entry-walk; modules are stitched through agnsh.cyr)
cyrius capacity --check src/agnsh.cyr  # fn-table / code-size headroom (must be <85%)
cyrius vet src/agnsh.cyr               # include-graph audit
cyrius fmt <file>                      # emits formatted source; diff vs tree must be empty
cyrius lint <file>                     # warn-as-error
```

For the format / lint loop, the CI walks `src/*.cyr tests/*.tcyr tests/*.bcyr` and fails on any drift or `warn` line — auto-discover so new modules pick up the gate.

## Code Standards

### Cyrius idioms

- **Struct construction**: use `alloc + store64` pattern, not struct literals
  ```cyrius
  fn Foo_new(a, b) {
      var p = alloc(16);
      store64(p, a);
      store64(p + 8, b);
      return p;
  }
  ```
- **Field access**: use `load64(p + OFFSET)` / `store64(p + OFFSET, v)`
- **String types**: pick one per function. `lib/string.cyr` (cstring) uses
  `strlen`, `streq`, `memcpy`. `lib/str.cyr` (Str fat pointer) uses
  `str_len`, `str_trim`, `str_sub`. Don't mix.
- **String literals** default to cstring; convert with `str_from()` if you
  need Str semantics.
- **Match statements**: always include a `_ =>` default case.
- **Trailing commas**: Cyrius 5.10.x `cyrius build` rejects a trailing comma
  after the last argument in a call (even though `cyrius fmt` preserves it on
  multi-line calls). Wrap long calls without a trailing comma after the last
  argument.
- **Reserved words**: don't use `match`, `default`, `in`, `shared` as
  variable names.

### Security

- Every user-controlled string that reaches a syscall must pass `is_safe_arg`
  or `is_safe_path` from `sanitize.cyr`.
- Every new Intent needs a translator arm in `translate.cyr` and a handler
  in `Interpreter_translate`.
- Every destructive operation (rm/mv) must go through `CheckpointManager`
  before execution.
- Every new command type must be classified in `permissions.cyr`.

### Documentation

- Every PR that changes behavior gets a CHANGELOG entry.
- Performance claims MUST include benchmark numbers from `bench-history.csv`.
- Breaking changes get a **Breaking** section with migration guide.
- Architectural decisions get an ADR in `docs/adr/`.

## License

By contributing, you agree that your contributions will be licensed under GPL-3.0-only.
