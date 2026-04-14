# ADR-004: Split Translate Dispatch Across Multiple Match Functions

**Status:** Accepted (2026-04-13)

## Context

`Interpreter_translate` needs to dispatch 42 intent tags to their translator
functions. The natural implementation is a single `match` statement:

```cyrius
fn Interpreter_translate(self, intent) {
    var tag = load64(intent);
    match tag {
        0 => { return translate_list_files(intent); }
        1 => { return translate_show_file(intent); }
        # ... 40 more arms ...
        _ => { return translate_unknown(intent); }
    }
}
```

When compiled, this caused cc3 (v4.3.0) to segfault during codegen — likely
exceeding a per-function local variable or emitted code limit.

## Decision

Split the dispatch across multiple functions, each handling a contiguous
range of tags. Wrap with a thin dispatcher.

```cyrius
fn translate_core(tag, intent) {
    match tag {
        0 => { return translate_list_files(intent); }
        # ... tags 0-18 (19 arms) ...
        _ => { return 0; }
    }
}

fn translate_extended(tag, intent) {
    match tag {
        19 => { return translate_git_commit(intent); }
        # ... tags 19-42 (~23 arms) ...
        _ => { return 0; }
    }
}

fn Interpreter_translate(self, intent) {
    var tag = load64(intent);
    if (tag <= 18) {
        var r = translate_core(tag, intent);
        if (r != 0) { return r; }
    }
    if (tag >= 19 && tag <= 42) {
        var r = translate_extended(tag, intent);
        if (r != 0) { return r; }
    }
    return translate_unknown(intent);
}
```

## Consequences

### Positive
- Compiles cleanly on cc3 v4.3.0
- Each sub-function stays well under any per-function limit
- Easy to add more intents: append to `translate_extended` or add a
  third dispatch function if that also fills up

### Negative
- Extra indirection: one function call becomes two
- Small performance cost (single call overhead, ~1 ns)
- Sentinel return value (0) for "no match in this range" — must be kept
  in sync with what `translate_unknown` returns

### Neutral
- Tag numbering must be dense and range-partitioned for the `tag <= 18`
  style dispatch. Not a burden, but it is a convention.

## When This Can Change

When cc3 lifts per-function limits (or we discover the real limit wasn't
match arms specifically), we can consolidate back into a single function.

## References
- `src/interpreter.cyr` — actual implementation
- Memory: `feedback_struct_syntax` (related struct+match issues)
