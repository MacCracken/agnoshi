# Writing a New Intent

This guide walks through adding a natural language intent to agnsh.
We'll add an example: "show uptime" → `uptime`.

## 1. Add the intent tag

Edit `src/intent.cyr` and append to the `IntentTag` enum:

```cyrius
enum IntentTag {
    # ... existing tags ...
    UPTIME = 44;
}
```

Tag numbers must be sequential. The Cyrius 4.5-era cc3 64-entry limit
on global initializers is gone in 5.10.x; the current `capacity --check`
gate reports headroom for the fn-table, identifiers, var-table, etc.
You'll hit the dispatch-split point (ADR-004) before the enum limit.

## 2. Add a parse rule

⚠ **Parse arms are ordered, and the first match wins.** Before adding a keyword,
check what already claims it — a broad earlier arm silently shadows a specific
later one. Four such shadowing bugs shipped before 1.9.5 found them, and the
memory/disk cross (a memory question answered with `df -h`) survived two
releases because each fix only moved it to the next wrong arm. Adding an
end-to-end anchor for the *neighbouring* intents, not just yours, is what stops
a re-cross: 1.9.10 pins memory, disk and system together for exactly this reason.

⚠ **Prefer the narrowest check that covers your phrases.** Every arm in a
fall-through parser is paid for by every unrecognised line. If your phrases all
contain the same distinctive word, key on the word — `MEMORY_INFO`'s four
phrases collapse to two `input_has_exact_word` checks, and the six-check
spelling measured **+3.9% on `parse/shell_cmd`**. Benchmark before and after;
`parse/shell_cmd` is the one that falls through everything.

Edit `src/interpreter.cyr`. Find the appropriate parse function —
for uptime, `parse_system_ops`:

```cyrius
fn parse_system_ops(trimmed) {
    # ... existing rules ...
    if (input_has_word(trimmed, "uptime") == 1) {
        return Intent_new(IntentTag.UPTIME);
    }
    # ... rest ...
}
```

If none of the existing parse functions fit, call your new handler
from `Interpreter_parse`.

## 3. Add a translator

Edit `src/translate.cyr`:

```cyrius
fn translate_uptime(intent) {
    var args = vec_new();
    return Translation_new(
        "uptime",
        args,
        "Show system uptime",
        PermissionLevel.READ_ONLY,
        "uptime shows how long the system has been running"
    );
}
```

Remember to use the `alloc + store64` pattern (ADR-002) — `Translation_new`
handles this internally.

If your translator reads parser-extracted string fields from the intent
(path, name, etc.), **validate them with the Str-aware safety predicates**
per [ADR-006](../adr/006-cstr-str-dispatch-discipline.md):

```cyrius
fn translate_remove(intent) {
    if (load64(intent + 8) == 0) { return translate_unknown(intent); }
    # Parser hands a Str — use safe_path_in_str (NOT is_safe_path,
    # which is cstring-typed and silently fails on Str input).
    if (safe_path_in_str(load64(intent + 8)) == 0) { return translate_unknown(intent); }
    # ... build args, return Translation ...
}
```

The CI lint shield (`scripts/lint-cstr-str.sh`) catches seven catalogued
variants (categories A–G) of the Str/cstring bug class — `str_cat(cstring, *)`
errors, cross-arch-broken raw syscalls, static-buffer escape, unchecked
`sys_chmod`, `strlen` inside an `_in_str` body, and `str_data()` handed to a
path-taking syscall. Run it before pushing: `sh scripts/lint-cstr-str.sh src`.

⛔ **Do not treat a clean run as proof.** Categories A/B match only a *literal*
argument, so a cstring carried in a **variable** is invisible to it. That blind
spot let three separate defects ship green: the entire SHELL_COMMAND risk
classifier (dead until 1.9.1), `audit_format_table` (1.9.8), and the parser
storing cstring literals into Str-typed intent fields — which left
`SERVICE_CONTROL` and `GIT_STASH` producing nothing but `echo` from v1.0 until
it was caught by checking the documented examples against the binary.

**The rule that actually protects you**: a value that came from the parser is a
`Str`. Guard it with the `_in_str` twin (`safe_arg_in_str`, `safe_path_in_str`,
`safe_commit_message_in_str`, `safe_branch_name_in_str`), and store it with
`str_from(...)` if it is a literal. Passing a `Str` to a cstring-typed guard
does not fail loudly — it reads the fat-pointer header.

## 4. Wire up dispatch

⚠ **`Interpreter_translate` splits on tag value**: `tag <= 18` goes to
`translate_core`, higher tags to `translate_extended`. Add your arm to the right
one — a tag added to the wrong half is simply never reached, and the symptom is
an intent that parses correctly and then translates to `echo`.

⛔ **Both halves are BOUNDED RANGES, and the upper bound is the one that bites.**
The dispatch reads:

```cyrius
if (tag <= 18) { ... translate_core(tag, intent) ... }
if (tag >= 19 && tag <= 44) { ... translate_extended(tag, intent) ... }
```

A new tag is by definition the **highest** number in the enum, so it lands
*above* the extended range and `translate_extended` is never called for it. Your
arm is present, correct, and unreachable — same `echo` symptom as putting it in
the wrong half, with nothing in the code looking wrong. **Widen the upper bound
in `Interpreter_translate` in the same edit that adds the arm.** This is not
hypothetical: `MEMORY_INFO` (1.9.10) hit it, and the bound had to go 42 → 44.

Guard it with a test that goes **through `Interpreter_translate`**, not one that
calls your translator directly — a direct call passes whether or not the
dispatch can reach it:

```cyrius
check("MY_TAG is reachable through the dispatch",
  streq(load64(Interpreter_translate(Interpreter_new(), Intent_new(IntentTag.MY_TAG))), "mycmd") == 1);
```

Edit `src/interpreter.cyr` — add the tag to `translate_extended` (or
`translate_core` if your tag is <= 18):

```cyrius
fn translate_extended(tag, intent) {
    match tag {
        # ... existing arms ...
        44 => { return translate_uptime(intent); }
        _ => { return 0; }
    }
}
```

If your tag would bring `translate_extended` above ~25 arms, split into a
third dispatch function per ADR-004.

## 5. Test it

Add a unit test in `tests/test_core.tcyr`:

```cyrius
var intent = Intent_new(IntentTag.UPTIME);
check("intent uptime", load64(intent) == IntentTag.UPTIME);

var t = translate_uptime(intent);
check("translate uptime cmd", streq(load64(t), "uptime") == 1);
check("translate uptime perm", load64(t + 24) == PermissionLevel.READ_ONLY);
```

Add a smoke test in `scripts/smoke-test.sh`:

```sh
out=$("$BIN" -c "show uptime" 2>&1)
check "parse uptime" "Intent:" "$out"
```

## 6. Verify

```bash
sh tests/test.sh
```

Should show:
```
=== All tests passed ===
```

## 7. Document

- Add a CHANGELOG entry under `### Added`
- If you invented a new translation pattern or data model, write an ADR

## Permission Level Cheat Sheet

| Command Type | Permission | Approval? |
|--------------|-----------|-----------|
| Pure query (ls, cat, ps, uptime, free, uname) | `READ_ONLY` | No |
| Navigation (cd, pwd, clear) | `SAFE` | No |
| User file mod (cp, mv, touch, mkdir) | `USER_WRITE` | No |
| System file mod (/etc, /usr writes) | `SYSTEM_WRITE` | Yes |
| Admin ops (apt, systemctl, kill, iptables) | `ADMIN` | Yes |
| Destructive (rm -rf, dd, mkfs, chmod) | `BLOCKED` | Human only |

Be conservative — if a command *could* cause state change, err toward higher
permission.

⚠ **There is no raw-shell escape hatch to fall back on.** `mode human` does not
hand the user a shell; it adds a confirmation prompt before a program launch. A
user who wants to run something directly uses `run /abs/path` (confirmed under
`human`/`strict`), or, on AGNOS, a bareword `/bin/<name>`. So a too-strict
classification is not softened by a mode — it just refuses.
