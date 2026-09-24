# Writing a New Intent

This guide walks through adding a natural language intent to agnsh.
We'll add an example: "show logged in users" → `who`.

## 1. Add the intent tag

Edit `src/intent.cyr` and append to the `IntentTag` enum — the next free number
(`MEMORY_INFO = 44` is the last one today):

```cyrius
enum IntentTag {
    # ... existing tags ...
    LOGGED_IN_USERS = 45;
}
```

Tag numbers must be sequential. The Cyrius 4.5-era cc3 64-entry limit
on global initializers is gone in 5.10.x; the current `capacity --check`
gate reports headroom for the fn-table, identifiers, var-table, etc.
You'll hit the dispatch-split point (ADR-004) before the enum limit.

## 2. Add a parse rule

⚠ **Parsing runs in two tiers, and the first match wins**
([ADR-007](../adr/007-input-classification.md)). The **specific tier** runs first —
`parse_admin_ops`, `parse_service_action`, `parse_service_query`,
`parse_state_queries`, `parse_file_phrases`, `parse_git_anchored` — then the
**broad tier** in its historical order (show/list/display, file ops, system ops,
the loose git gate, question words), and SHELL_COMMAND last. Until 1.9.16 the
broad parsers ran first, and every specific phrase a broad keyword could reach
needed a guard bolted onto the broad parser.

A rule belongs in the **specific tier** only if its trigger is *anchored*:

- **A bounded word or phrase, asked through the line's PhraseIndex** —
  `index_has_phrase(pix, "logged in")`. It has `input_has_phrase`'s semantics:
  whole words only, case-insensitive, sentence punctuation as a boundary. ⛔ Never
  `input_has_word` for a specific trigger: it matches token *prefixes* and phrase
  *substrings* (`remove user` hits `remove user_data.txt`), and this tier runs
  before the file parser, so a loose trigger here steals file operations.
- **Or a verb position / token count** — `input_starts_with`, `token_count`.
- **And it obeys the file-verb rule**: a sentence that opens with a file verb is
  that file operation, so a trigger later in it is only the object —
  `delete the logged in report` is a deletion. `trigger_claims(trimmed, lead1,
  lead2)` answers it; pass as a lead any phrase of yours that itself starts with
  a file verb (`create user`, `change password`). `parse_state_queries` already
  applies it to every trigger it holds; in another parser, ask it after your
  trigger matches, so lines without your trigger never pay for it.
- Write triggers in lowercase, starting with a letter: the index rejects a phrase
  by its first two letters, and anything else takes the full scan (correct, slower).

Anything else — a single keyword that should claim whatever mentions it — goes in
the **broad tier**, with `input_has_word`.

⚠ **Every trigger is asked of every line that gets that far.** Even through the
index, the specific tier adds 0.35–0.66 µs to each benchmarked line, and
`parse/shell_cmd` falls through everything. Prefer the narrowest check that
covers your phrases: if they all contain one distinctive word, key on the word —
`MEMORY_INFO`'s phrasings collapse to two word checks, and the six-check spelling
measured **+3.9% on `parse/shell_cmd`**. Benchmark before and after.

Edit `src/interpreter.cyr`. Logged-in users are system state, so the rule goes in
`parse_state_queries`, which picks the query and then applies the file-verb rule
once for all of them. (Note the example phrase does not *start* with `who`: under
shell-first dispatch a line that opens with a program's name runs that program and
never reaches the parser — ADR-007.)

```cyrius
fn parse_state_queries(trimmed, pix) {
    var tag = 0 - 1;
    # ... existing groups ...
    } elif (index_has_phrase(pix, "logged in") == 1) {
        tag = IntentTag.LOGGED_IN_USERS;
    }
    if (tag < 0) { return 0; }
    if (trigger_claims(trimmed, 0, 0) == 0) { return 0; }
    # ... build the intent ...
}
```

If none of the existing parse functions fit, add one to the tier your trigger
belongs to in `Interpreter_parse`; a specific-tier parser takes `(trimmed, pix)`.

**Then add a row for your phrase to `docs/examples/common-commands.md`** — the
table is the spec. `tests/test_parse_corpus.tcyr` reads it and fails CI when a row
stops classifying or translating as written (add your tag's name to `_tag_named`
there, or the row fails loudly). If your trigger shares a word with another
intent, add probes in **both** directions to that test: the neighbour must not
claim your phrase, and your trigger must not claim the neighbour's sentences —
including a file verb whose object merely mentions your words.

## 3. Add a translator

Edit `src/translate.cyr`:

```cyrius
fn translate_logged_in_users(intent) {
    var args = vec_new();
    return Translation_new(
        "who",
        args,
        "Show who is logged in",
        PermissionLevel.READ_ONLY,
        "who lists the users logged in to this machine"
    );
}
```

Remember to use the `alloc + store64` pattern (ADR-002) — `Translation_new`
handles this internally.

⛔ **Since 2.0.0 a SAFE or READ_ONLY translation RUNS** ([ADR-008](../adr/008-nl-exec-contract.md)):
the NL path resolves its command (`$PATH` on the host, `/bin` on AGNOS) and executes it with its
arguments as argv. So:

- the command must be a program the target has — `who` is; `cd` is a shell builtin and would change
  nothing in a child process, so `nl_verdict` refuses CHANGE_DIR, and does the same for a
  translation routed to an MCP tool (an `echo` placeholder). Add yours there if it is not a program;
- every argument must be a **cstring**: push a parser field (a Str) with `arg_cstr`, which converts
  it once (ADR-006) and keeps an absent field absent. A Str pushed as-is reaches `execve` as a
  16-byte header, not as text — nothing noticed while nothing ran them;
- if the target lacks the program, give it the one it has: `SHOW_FILE` becomes `owl -p` on AGNOS.

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
        45 => { return translate_logged_in_users(intent); }
        _ => { return 0; }
    }
}
```

If your tag would bring `translate_extended` above ~25 arms, split into a
third dispatch function per ADR-004.

## 5. Test it

Add a unit test in `tests/test_core.tcyr`:

```cyrius
var intent = Intent_new(IntentTag.LOGGED_IN_USERS);
check("intent logged-in users", load64(intent) == IntentTag.LOGGED_IN_USERS);

var t = translate_logged_in_users(intent);
check("translate logged-in users cmd", streq(load64(t), "who") == 1);
check("translate logged-in users perm", load64(t + 24) == PermissionLevel.READ_ONLY);
```

Add a smoke test in `scripts/smoke-test.sh`:

```sh
out=$("$BIN" -c "show logged in users" 2>&1)
check "parse logged in users" "Intent:" "$out"
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
