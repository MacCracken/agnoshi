# ADR-002: Struct Construction via alloc + store64

**Status:** Accepted (2026-04-13)

## Context

Cyrius documents two struct construction syntaxes:
1. **Literal**: `var p = Point { 10, 20 };`
2. **Manual**: `var p = alloc(16); store64(p, 10); store64(p + 8, 20);`

During the initial port, all 21 source files used the literal syntax. The
resulting binary failed to compile with `undefined variable 'ModeManager'`
errors at the point of `return ModeManager { ... }`.

Investigation revealed the literal syntax is **not fully implemented** in
cc3 (Cyrius compiler) for multi-field structs. The Cyrius project roadmap
lists "Struct initializer syntax" as a Medium-priority pending feature.
The FAQ explicitly warns that `var a = Point { 1, 2 }` "passes the first
field value, not the address".

The entire Cyrius stdlib uses the manual pattern exclusively.

## Decision

Use `alloc + store64` for all struct construction. Use `load64 / store64`
with explicit offsets for all field access. Keep `struct` declarations as
layout documentation.

## Consequences

### Positive
- Works reliably on current cc3 (v4.3.0 at time of this ADR)
- Matches the stdlib convention — familiar to Cyrius developers
- Explicit memory layout makes the binary interface obvious
- No compiler bugs to work around

### Negative
- More verbose than the literal form
- Field offsets must be maintained when struct layout changes
- Mistakes can corrupt memory silently (offset math is manual)

### Mitigation
- `struct` declarations still document field order and count
- Constructor functions (`Foo_new`) encapsulate the alloc + store sequence
- Offsets documented in comments for complex structs (see `intent.cyr`,
  `session.cyr`)

## Example

```cyrius
struct Intent { tag; field1; field2; field3; field4; int1; int2; vec1; }

fn Intent_new(tag) {
    var p = alloc(64);            # 8 fields * 8 bytes
    store64(p, tag);              # offset 0: tag
    store64(p + 8, 0);            # offset 8: field1
    store64(p + 16, 0);           # offset 16: field2
    # ...
    return p;
}

fn Intent_tag(self) { return load64(self); }
fn Intent_set_field1(self, v) { store64(self + 8, v); }
```

## When This Can Change

This ADR should be revisited when Cyrius ships reliable struct literal
syntax (tracked in Cyrius roadmap). Revert would be a large mechanical
change across all source files but preserves semantics.

## References
- Cyrius FAQ entry on struct literals
- Cyrius roadmap: "Struct initializer syntax" (Medium priority)
- `feedback_struct_syntax` memory
