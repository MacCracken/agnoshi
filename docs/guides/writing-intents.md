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

Tag numbers must be sequential and < 64 (cc3 global initializer limit).

## 2. Add a parse rule

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

## 4. Wire up dispatch

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

Be conservative — if a command *could* cause state change, err toward
higher permission. Users can always explicitly mode-switch to `human` for
raw shell execution.
