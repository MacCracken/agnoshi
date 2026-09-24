# Contributing to Agnoshi

Thank you for your interest in contributing to Agnoshi.

## Getting Started

1. Fork the repository
2. Create a feature branch
3. Install Cyrius (the pin lives in `cyrius.cyml`: `cyrius = "6.6.6"`). Toolchain releases: https://github.com/MacCracken/cyrius/releases
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

# The same suites on aarch64, as CI runs them (needs qemu-user's qemu-aarch64).
# Open flags differ per arch; an x86_64-only run cannot see that class (1.9.13).
cyrius build --aarch64 tests/test_core.tcyr build/test_core_a64 && qemu-aarch64 build/test_core_a64

# agnsh on the real agnos kernel, in QEMU — the only place the agnos-only launch paths
# (pipelines, `>`, `&`) and their audit records actually run. Manual, not CI: needs
# ../agnos (build/agnos + build/rootfs) and ../gnoboot built. Run it for any change that
# touches an agnos path. It only reads the agnos repo.
python3 scripts/agnos-qemu-test.py
```

### Cleanliness gates

These match the CI gate set. Run them before pushing — any drift fails the build.

```bash
cyrius check src/agnsh.cyr             # syntax (entry-walk; modules are stitched through agnsh.cyr)
cyrius capacity --check src/agnsh.cyr  # fn-table / code-size headroom (must be <85%)
cyrius vet src/agnsh.cyr               # include-graph audit
cyrius fmt --check src/*.cyr tests/*.cyr tests/*.tcyr tests/*.bcyr   # fmt-drift gate (drop --check to repair)
cyrius lint <file>                     # warn-as-error
sh scripts/lint-cstr-str.sh src        # Str/cstring antipatterns (A-G) + test-seam containment (H)
sh scripts/check-coverage.sh           # fn-level coverage gate (>=80% host-reachable)
```

⚠ `lint-cstr-str.sh` has a known blind spot: categories A/B match only a
**literal** argument, so a cstring carried in a variable is invisible to it. A
clean run is not proof — it was green through both the dead SHELL_COMMAND
classifier (1.9.1) and the `audit_format_table` defect (1.9.8).

ℹ **`cyrius fmt` takes a file list since cyrius 6.6.5**, in both the `--check`
and the rewrite form, and a missing file fails with `cannot read file` rather
than passing. Before 6.6.5 it silently ignored every file after the first, which
is how 1.9.10 reached CI with drift after a locally "clean" sweep. The per-file
wrapper that worked around it (`scripts/check-fmt.sh`) was retired in 1.9.13,
once the multi-file form was verified to fail on planted drift in each position.

⛔ **Category H — the audit-path test seam stays a test seam.**
`audit_path_override_set` (`src/statepaths.cyr`) redirects where the security log
is written. It exists so tests can assert what an audit record *says*: the audit
writers resolve their own destination, so before 1.9.10 a test had no file to
read back and the six functions writing the security log had no assertion at all.
The safety argument for that mutable global is *"nothing in the shell calls the
setter, so it is unreachable from any input-driven path"* — Category H is what
keeps the argument true. Its definition site is the only permitted mention under
`src/`; anywhere else fails the build.

⚠ **A mention is not a test.** `check-coverage.sh` grepped raw test files until
1.9.10, so naming a function in a *comment* marked it covered — it happened while
writing the 1.9.10 seams and hid five functions that had never been asserted.
Comments are stripped now. The general lesson, which cost a release to learn
twice: **when you add a gate, make it fail on purpose before you trust it.**
Both gates added in 1.9.10 were verified that way.

For the format / lint loop, the CI walks `src/*.cyr tests/*.cyr tests/*.tcyr tests/*.bcyr` and fails on any drift or `warn` line — auto-discover so new modules pick up the gate.

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
- ⛔ **Buffer sizing differs by scope — this has caused real overflows in both
  directions.** Verified by direct probe against cyrius 6.5.36:
  - **module-scope** `var X[N]` allocates **8N bytes** (N u64 slots)
  - **function-scope** `var X[N]` allocates **N bytes**

  Both spellings are load-bearing in this tree. Reading `var buf[256]` as
  "256 slots" hid a 134-byte `uname` overflow in `prompt.cyr` (a `struct utsname`
  is 390 bytes) and a 7-byte uid overflow in `statepaths.cyr`. Reading it the
  other way makes `sh_env_blob[128]`, `rl_buf[512]`, `job_pid[8]` and
  `job_cmd[128]` all *look* like overflows when every one of them is correct.
  **Check the scope, and sweep the whole class rather than trusting a comment
  next to one declaration.**
- **Match statements**: always include a `_ =>` default case.
- **Trailing commas**: `cyrius build` (verified through 6.5.36) rejects a trailing comma
  after the last argument in a call (even though `cyrius fmt` preserves it on
  multi-line calls). Wrap long calls without a trailing comma after the last
  argument.
- **Reserved words**: don't use `match`, `default`, `in`, `shared` as
  variable names.

### Security

- Every user-controlled string that reaches a syscall must pass a `sanitize.cyr`
  validator **matching its type**. cstrings use `is_safe_arg` / `is_safe_path`;
  values that came from the parser are `Str` and must use the `_in_str` twins
  (`safe_arg_in_str`, `safe_path_in_str`, `safe_commit_message_in_str`,
  `safe_branch_name_in_str`). Passing a `Str` to a cstring-typed guard does not
  fail loudly — it reads the 16-byte fat-pointer header and silently passes or
  fails at random. That exact mistake made the whole SHELL_COMMAND risk
  classifier dead until 1.9.1.
- Every new Intent needs a translator arm in `translate.cyr` and a handler
  in `Interpreter_translate`.
- ⚠ Forward-looking, not current: destructive operations will route through
  `CheckpointManager` once `src/checkpoint.cyr` is in the binary's include graph.
  It is not today, so there is no rollback — do not write code or docs that
  assume there is.
- Every new command type must be classified in `permissions.cyr`.

### Documentation

- Every PR that changes behavior gets a CHANGELOG entry.
- Performance claims MUST include benchmark numbers from `bench-history.csv`.
- Breaking changes get a **Breaking** section with migration guide.
- Architectural decisions get an ADR in `docs/adr/`.

## License

By contributing, you agree that your contributions will be licensed under GPL-3.0-only.
