# agnoshi test suite

Unit tests, benchmarks, and smoke tests for the agnoshi shell. Everything here
is Cyrius and builds with `cyrius build` — no external test runner.

## Layout

| File | Kind | Covers |
|------|------|--------|
| `harness.cyr` | shared include | `check()` / `report()`, the pass/fail tally, and the forward-ref stubs. **Not a test** — has no `main()`. |
| `test_core.tcyr` | unit tests | mode manager, permissions, intent parsing, translators, approval matrix, audit log, security context (738 checks) |
| `test_security.tcyr` | regression | command classification, path/username/branch/commit-message sanitizers, permission gates (26 checks) |
| `test_parse_corpus.tcyr` | spec | the NL classifier against `docs/examples/common-commands.md` (read at run time), shadowing probes in both directions, and the PhraseIndex held to `input_has_phrase` — [ADR-007](../docs/adr/007-input-classification.md) (358 checks) |
| `bench_core.bcyr` | benchmarks | parse / translate / permission / sanitize hot paths |
| `test.sh` | runner | builds the shell, then runs every `test_*.tcyr` suite → smoke → bench |

## Running

```sh
sh tests/test.sh                      # everything (from the repo root)

# or individually — always from the repo root, includes are root-relative:
cyrius build tests/test_core.tcyr build/test_core && ./build/test_core
cyrius build tests/test_security.tcyr build/test_security && ./build/test_security
cyrius build tests/test_parse_corpus.tcyr build/test_parse_corpus && ./build/test_parse_corpus
cyrius build tests/bench_core.bcyr build/bench_core && ./build/bench_core
```

## The shared harness

Each test/bench file follows the same shape:

```cyrius
include "lib/…"              # 1. stdlib (harness's check/report need str_from, println, print_num)
include "tests/harness.cyr"  # 2. shared harness — AFTER lib, BEFORE src
include "src/…"              # 3. the modules under test
# … fn main() { alloc_init(); …checks…; return report(); }
fn _entry(): i64 { var rr = main(); syscall(SYS_EXIT, rr); return 0; }
_entry();                    # bare top-level call (NOT `var r = main()`; see CLAUDE.md § Cyrius)
```

The include order is load-bearing (cyrius is single-pass):

1. **lib before harness** — `check`/`report` call `str_from`/`println`/`print_num`.
2. **harness before src** — the harness stubs (`ui_show_error`, `ui_show_warning`,
   `read_line`, `chrono_now_rfc3339`) satisfy forward references that the src
   modules make into `agnsh.cyr` / `ui.cyr`, which are outside a test's include
   set. cyrius 6.3.x refuses to emit a binary with a reachable-undefined fn, so
   these must resolve.

`getenv` is **not** stubbed — `lib/io.cyr` provides the real reader and the tests
exercise the real env path. A test that needs a deterministic env can define its
own `getenv` after the harness include (last-def-wins).

## Gotcha: function-local `var X[N]` is N **bytes** on the stack

Since cyrius 6.3.13, a function-scope `var buf[N]` reserves **N bytes on the
stack** (module-scope is still N×u64). A buffer sized in the old u64-slot
mental model silently under-allocates and the syscall write smashes the return
address — a crash that only shows up at *run* time, never at compile time.

This is what broke this suite during the 6.3.x migration: `verify_sudo_path`'s
`struct stat` buffer was `var stat_buf[18]` (18 u64 slots = 144 bytes, once) but
became an 18-byte stack buffer that `sys_stat`'s 144-byte write overran. Size
raw-struct buffers in bytes: `var stat_buf[144]`. **Run the tests — a clean
compile proves nothing about buffer sizing.**

## Adding a test

Add `check("name", <condition == expected>)` to the relevant file's `main()`.
For a whole new suite, copy the harness-based skeleton above and name it
`tests/test_<name>.tcyr`: `test.sh` (since 1.9.16) and CI both auto-discover
`tests/test_*.tcyr` — CI also, since 1.9.13, runs the same suites built
`--aarch64` under qemu-user, so a test must not assume x86_64 constants — and CI
lints and fmt-checks `tests/*.cyr` / `*.tcyr` / `*.bcyr`.

A new natural-language phrase gets a row in `docs/examples/common-commands.md`
rather than a hand-written check: `test_parse_corpus` reads that table and holds
the parser to every row (ADR-007).
