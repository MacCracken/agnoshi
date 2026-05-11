# ADR-006: Cyrius cstring/Str Dispatch Discipline (refines ADR-005)

**Status:** Accepted (2026-05-11)
**Supersedes (in part):** [ADR-005: Explicit String Type Discipline](005-string-type-discipline.md)

## Context

ADR-005 established the convention that every function picks one of Cyrius's two string conventions (`cstring` null-terminated vs `Str` fat-pointer) and converts at the boundary. It worked at the v1.0 scale (20-ish modules, tightly controlled cross-module calls).

The v1.2.0 toolchain bump to Cyrius 5.10.34 + the v1.3.0 feature work surfaced **seven distinct bug variants** that ADR-005 alone didn't prevent. Each one took a probe / SIGSEGV / first-use crash to discover, and the cumulative cost to track them down was material — by far the most expensive bug-class in agnoshi's life so far.

The seven variants:

1. **`str_len("literal")`** — Str-only primitive called with a cstring; reads `load64(s + 8)` off the literal as a fake length (slice 1 v1.2.0, in `sanitize.cyr` CI helpers).
2. **`str_sub(s, start, end)` semantics drift** — Cyrius 4.5's `str_sub` was `(start, end)`; 5.10.x's is `(start, len)`. 19 sites silently over-read their buffers (slice 1 v1.2.0).
3. **`str_cat("literal", *)`** — Str-typed first arg with cstring literal; no `str_cat_cstr` overload exists in `lib/str.cyr` to rescue it (slice 7 v1.2.0 + slice 8 v1.3.0).
4. **`str_cat(*, "literal")`** — symmetric; second-arg cstring (slice 2 v1.3.1).
5. **`is_safe_path(Str)`** — the original `is_safe_path` / `is_safe_arg` are cstring-typed (`load8(path + i)` dereferences the raw address), but the **parser hands the translators a Str** (via `extract_after` / `extract_between` → `str_substr`). Every NL filesystem operation (copy / move / remove / mkdir / show-file / find / search-content) had been silently routing to `translate_unknown` since v1.0. **Three years of latent regression** before slice 3 v1.3.0 caught it (because slice 8 v1.3.0's hint-line surfaced the silent fallthrough).
6. **aarch64 raw syscalls** — `SYS_OPEN` / `SYS_CHMOD` / `SYS_STAT` exist on x86_64 but not aarch64's generic syscall table (which uses `openat` / `fchmodat` / different stat layout). Direct `syscall(SYS_*, ...)` builds clean on x86 and breaks at link time on aarch64 (v1.3.0 closeout).
7. **`str_from(&static_buf)` aliasing** — `var buf[N]` in Cyrius is *static data* (not stack), so a Str wrapping `&buf` borrows memory the function will overwrite on the next call. Every `read_input_line` / `CommandHistory_add` / similar pattern that stored or returned `str_from(&buf)` produced Strs whose data pointer aliased to whatever was last typed (slice 7 v1.3.0 caught it for history; v1.3.1 P(-1) audit found 5 more dormant copies).

Looking across the seven, three patterns emerge:

- **(a) "No `_cstr` overload" trap** — Cyrius's name-mangling dispatch (`strlen(cstring)` → `strlen_str(Str)` etc.) rescues some Str-typed fns when called with cstring (`str_contains` → `str_contains_cstr`, `str_eq` → `str_eq_cstr`, `str_split` → `str_split_cstr`). But not all. `str_cat`, `str_len`, `str_data`, `str_starts_with`, `str_ends_with` have **no** `_cstr` variant — call them with a cstring and the toolchain happily compiles a runtime fault.
- **(b) "Cross-arch syscall surface" trap** — bare `syscall(SYS_*, ...)` lets you bypass the per-arch wrappers in `lib/io.cyr` (`sys_open`, `sys_chmod`, `sys_stat`). The wrappers exist on both arches; the bare numeric form silently picks x86 numbers that mean something different (or don't exist) on aarch64.
- **(c) "Static-data escape" trap** — `var buf[N]` looks like a stack buffer but isn't. Returning or long-term-storing a pointer into it aliases.

ADR-005 didn't address any of these three. It documented the per-module convention but treated the helper namespace as flat.

## Decision

Extend ADR-005's per-module discipline with three operational rules and a **CI shield** that mechanically enforces them.

### Rule 1 — Explicit `_str` suffix for Str-only operations on cross-type boundaries

When a module is `cstring`-side per ADR-005 but exposes a helper that operates on Str (because a caller in a Str-side module needs it), the helper carries an explicit `_str` (or `_in_str`) suffix. The cstring-typed version keeps the bare name.

Example from v1.3.0 slice 3:

```cyrius
// sanitize.cyr — cstring-typed module
fn is_safe_path(path)        { /* cstring path */ }
fn safe_path_in_str(s)        { /* Str path; uses str_byte_at + strlen */ }
fn is_safe_arg(s)             { /* cstring arg */ }
fn safe_arg_in_str(s)         { /* Str arg */ }
```

Callers that hold a Str (translators, parser-output handlers) call the `_in_str` variant. The renaming makes intent obvious at the call site — no more "is this Str-safe?" question on every safety check.

> *Naming caveat — discovered slice 5 v1.3.0:* an earlier draft used `is_safe_path_str` (the natural `_str` suffix). That suffix turns out to be load-bearing in Cyrius's overload dispatch: the toolchain registered the variant as the *Str overload of `is_safe_path`* and routed `is_safe_path(Str_value)` → `is_safe_path_str` automatically. For Str inputs this was correct, but it also meant that *cstring* callers of `is_safe_path` got rerouted to the Str variant when their cstring happened to look Str-shaped enough at runtime. Renaming to `safe_path_in_str` (preposition-inside, breaking the `_str` suffix convention) sidesteps the overload registration. Future Str-side variants should follow this naming.

### Rule 2 — Per-arch syscall wrappers, no bare `syscall(SYS_*, ...)`

Every syscall site goes through the `lib/syscalls_{x86_64,aarch64}_linux.cyr` wrappers (`sys_open`, `sys_chmod`, `sys_stat`, `sys_read`, `sys_write`, `sys_close`, etc.) which expand to the correct numeric syscall AND, where relevant, the correct calling convention (`openat(AT_FDCWD, ...)` on aarch64 etc.).

Direct `syscall(SYS_*, ...)` is allowed only for syscalls that exist *and have the same calling convention* on both arches:

- ✓ `syscall(SYS_READ, ...)` / `SYS_WRITE` / `SYS_CLOSE` / `SYS_EXIT` — both arches.
- ✓ `syscall(SYS_GETUID|GETGID|GETEUID, ...)` — both arches.
- ✗ `SYS_OPEN` — x86-only, aarch64 has only `openat`. Use `sys_open` wrapper.
- ✗ `SYS_CHMOD` — x86-only, aarch64 has only `fchmodat`. Use `sys_chmod` wrapper.
- ✗ `SYS_STAT` — different struct layout per arch (`st_uid` at offset 28 on x86, 24 on aarch64). Use `sys_stat` wrapper; `#ifdef CYRIUS_ARCH_X86 / AARCH64` for field offsets.

### Rule 3 — Static-buf escape requires `str_clone`

`var buf[N]` is static data. A Str wrapping `&buf` (via `str_from(&buf)` or `str_new(&buf, n)`) only stays valid while the function holding the buf is on its current call. Any of:

- `return str_from(&buf);`
- `store64(target, str_from(&buf));` (storing into a long-lived struct)
- `vec_push(some_vec, str_from(&buf));` (pushing into a collection that outlives the function)

… needs the Str's data to be deep-copied to a heap buffer the Str owns:

```cyrius
return str_clone(str_from(&buf));
store64(target, str_clone(str_from(&buf)));
vec_push(some_vec, str_clone(str_from(&buf)));
```

For *immediate-use* sites (`str_print(str_from(&buf))` or local-only without store/return), the borrow is fine — `# lint:cstr-ok` whitelist available.

### Rule 4 — CI lint shield mechanically enforces the above

`scripts/lint-cstr-str.sh` (added v1.3.1 slice 2, extended in slices 3-4) greps the source for each pattern variant and fails CI on a hit. As of v1.3.1 the linter has **14 patterns across 5 categories**:

- **Category A** (5 patterns) — Str-typed-arg fns receiving cstring literal as **first** arg: `str_len("..."`, `str_data("..."`, `str_cat("..." )`, `str_starts_with("..."`, `str_ends_with("..."`.
- **Category B** (3 patterns) — same with literal as **second** arg.
- **Category C** (3 patterns) — bare `syscall(SYS_OPEN|CHMOD|STAT, ...)`.
- **Category D** (2 patterns) — static-buf escape via `return str_from(&...)` or `store64(*, str_from(&...))`.
- **Category E** (1 pattern) — unchecked `sys_chmod(...)` statement (security-critical syscall return).

Escape hatch: trailing `# lint:cstr-ok` comment on a specific line marks an intentional use.

## Consequences

### Positive

- **The bug class is shut.** Seven variants found over v1.2.0/v1.3.0 → zero new instances pass CI as of v1.3.1. The linter retroactively flags every historical surface.
- **Onboarding cost drops.** A new contributor seeing `safe_path_in_str` knows the function takes Str; seeing `safe_path` knows it takes cstring. No type ambiguity at the call site.
- **aarch64 portability is mechanical, not aspirational.** Wrappers, not bare syscalls. Build-time enforcement.
- **Static-buf semantics are explicit.** `str_clone` discipline is greppable and reviewable.
- **ADR-005 stays valid.** This is a refinement, not a replacement. The per-module convention table in §3 of ADR-005 is still the right top-down framing; this ADR adds the operational rules at the function-name and call-site level.

### Negative

- **More boilerplate at conversion boundaries.** Every Str-side caller of a previously-cstring-only helper needs the explicit `_in_str` variant. Doubles the helper count in `sanitize.cyr`.
- **Naming non-uniformity.** Most of the codebase uses `_str` suffix (per ADR-005 conventions); v1.3.0 had to break that with `_in_str` for the safety helpers to avoid Cyrius's overload-registration trap. Awkward but load-bearing.
- **Static-buf cost.** Every escape path now allocates a clone. On the parse → translate → audit path this is at most a few extra small heap allocations per command; measured zero impact on benchmarks at v1.3.0 closeout.
- **Linter false-positive risk.** The Category B regex (`str_cat\([^"]+,\s*"`) could in principle false-positive on a cstring `vec` element pushed through — none observed yet, but the escape hatch exists.

### Neutral

- The Cyrius toolchain *itself* could in principle add the missing `_cstr` overloads (`str_cat_cstr`, `str_len_cstr` via the strlen pattern). That would obsolete part of this ADR's rule 1. Until then the agnoshi-side discipline is the load-bearing path. Queued as an upstream feature request in the v1.4.0 hoosh-modernization roadmap window.

## Alternatives Considered

1. **Stay with ADR-005 alone** — rejected. v1.2.0/v1.3.0 demonstrated that the per-module convention isn't sufficient at the helper-name granularity. The seven variants all live in modules that *did* follow ADR-005's per-module convention; the bugs were at the seams.

2. **Add a Cyrius-side `--strict-types` flag** — rejected for now. Cyrius's type system isn't strict enough to catch these (the `streq(Str, Str)` case in v1.3.0 slice 5 *was* caught by the new type-warning hint, suggesting the toolchain can get there). Tracking the four other variants would need a more substantial type-check pass. Linter is the fast path; upstream type-check is the right long-term path.

3. **Move all string handling to Str-only** — rejected. `permissions.cyr` compares against 60+ literal command names (`"dd"`, `"chmod"`, `"sudo"`, etc.). Wrapping every literal in `str_from()` at every comparison would allocate uselessly. The cstring convention there is correct; just needs the `_in_str` companion for cross-type seams.

4. **Custom static-analyzer in Cyrius itself (cyrius lint plugin)** — rejected for v1.3.x. The grep-based linter is fast, dependency-free, and lives in the agnoshi repo. A Cyrius-side plugin would need toolchain work and would affect all consumers, not just agnoshi. Reserve for when the bug class is observed elsewhere.

## References

- ADR-005 (the v1.0 precursor this refines)
- `scripts/lint-cstr-str.sh` (the CI shield)
- `docs/audit/2026-05-11-pminus1.md` §2-§4 (the seven-variant catalog with severities)
- CHANGELOG `[1.3.0]` — every variant linked back to its discovery slice
- `feedback_cyrius_strings` memory — original v1.0 discipline notes
