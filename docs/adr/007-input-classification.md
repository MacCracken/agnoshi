# ADR-007: Input Classification — Shell First, Then Specific Before Broad

**Status:** Accepted (2026-09-23, 1.9.16)
**Refines:** [ADR-003: Keyword-Based Parser Instead of Regex](003-keyword-parser-over-regex.md) — keyword
matching stays; this fixes which matcher gets a line, and how strongly it must match to claim it.

## Context

A line is either a **shell line** (a program with arguments, possibly joined by `|` or redirected
by `>`) or **natural language**, and natural language goes to `Interpreter_parse`: a first-match
cascade of keyword parsers. Roadmap 1.9.16 held three open questions that are one question:

1. **Dispatch order.** The broad parsers ran first (`parse_show_commands` 1st, `parse_file_ops`
   2nd) and the specific ones late (`parse_admin_ops` 5th, `parse_state_queries` 8th). Three of the
   four wrong commands 1.9.5 fixed came from that alone — `delete user bob` → `rm bob`,
   `show contents of FILE` → a bare `ls`, `show memory usage` → `df -h` — each by bolting a guard onto
   the broad parser (one guard measured +31% on `parse/list_files` before it was optimised to +5%).
2. **A pre-cascade discriminator.** A plain shell line paid the whole cascade (~79 keyword probes,
   `parse/shell_cmd` 6.0 µs) before falling through to SHELL_COMMAND.
3. **The `>` scan is unanchored.** On agnos a sentence containing `>` is diverted to the redirect path.

It had to be settled before 2.0.x: a mis-claimed line prints a wrong proposal today, and would run
one once the NL path executes.

## Decision

### 1. Shell first, on both targets

A line whose first word is a program **runs as that program**; only what is not a shell line reaches
the NL parser. The discriminator is the dispatch layer's program lookup — a fact — not a heuristic
inside the parser. agnos does this already (`sh_try_bareword_launch` probes `/bin/<word>` before
`print_intent_result`). The Linux host adopts it with a `PATH` lookup in 2.0.0, alongside NL
execution; until then the host's only launcher is `run /abs/path`.

### 2. Operators are shell syntax

A line with `|` or `>` is a shell line. If a stage is not a program the line is an **error**
(`run: no such command…`) and is never reinterpreted as natural language. agnos behaves this way today
— the pipeline and redirect launchers run before the NL path and fail rather than fall through — and
the host follows in 2.0.0. The unanchored `>` therefore stays: on agnos `show files > 10MB` is an
error, not a proposal.

### 3. Specific before broad

Inside `Interpreter_parse` the **specific tier** runs first — `parse_admin_ops`,
`parse_service_action`, `parse_service_query`, `parse_state_queries`, `parse_file_phrases`
(`contents of`), `parse_git_anchored` (`git <subcommand>`) — then the **broad tier** in its historical
order (show/list/display, file ops, system ops, the loose git gate, question words), then
SHELL_COMMAND.

Reordering alone is not safe. With the 1.9.15 triggers, running the specific parsers first changed
**13 of 161** phrases, every one of them a regression and none a fix, because the "specific" parsers
matched by substring and token prefix: `remove user_data.txt` became `userdel`, `delete firewall.log`
a ufw rule deletion, `find files named ram` a memory query. Bounding the triggers is not enough
either: with a file-verb rule for bare words only, **104 of 189** file-verb sentences whose object
holds a multi-word trigger became that trigger's query or admin action (`copy the add user script` →
`useradd`). So a trigger claims a line in the specific tier only on **anchored** evidence:

- **Whole words and bounded phrases** (`input_has_phrase`): case-insensitive, a word boundary at both
  ends, sentence punctuation as a boundary (`what is my ip?`), and `.` inside a word only when a word
  byte follows it (`firewall.log` is one word; `show memory.` ends one).
- **The file-verb rule** (`trigger_claims`): a sentence that opens with a file verb (`delete`,
  `copy`, `read`, `find`, … — `opens_with_file_verb`) *is* that file operation, and a trigger later in
  it names the operation's object: `delete the disk usage report`, `copy the add user script`. A
  trigger claims such a sentence only when the verb is part of the trigger — when the trigger starts
  the line: `delete user bob`, `create group ops`, `change password alice`, `delete firewall rule 22`.
  Verb position is anchored evidence; a noun is not, so `show the groups file` is a question about
  groups (1.9.15 answered `ls`, because the broad `file` keyword ran first).
- **Anchored git**: `git <subcommand>` is claimed ahead of the file verbs only where git has that
  action — `delete git branch foo` deletes the branch; `remove the git log` stays a file operation.

The guards this retires: `parse_show_commands`' `contents of` hand-off and its memory/ram hand-off in
the disk branch. Kept as a backstop: `parse_file_ops`' REMOVE guard (never `rm` a word naming an
account, a group or a firewall rule) for admin phrasings the admin parser does not claim, such as
`remove the user bob`.

### 4. Enforcement

`tests/test_parse_corpus.tcyr`, in CI on x86_64 and aarch64 (qemu): every row of
`docs/examples/common-commands.md`, read at run time, must parse to its class and translate to its
command; shadowing probes are asserted in both directions; and the PhraseIndex (below) must answer
exactly what `input_has_phrase` answers on every line the corpus parses. A trigger or reordering that
re-routes a documented phrase fails CI.

## Consequences

**Positive**
- Measured against 1.9.15 over 176 phrases (the table, the probes, the literal parses in `test_core`),
  **11 classify differently and all eleven are intended**: `delete git branch foo`,
  `remove git branch foo` and `git branch delete foo` (were `rm`), `git branch remove-old` and
  `git branch delete-old` (were `rm`; now create the branch), `is there enough ram?`
  and `show memory.` (were SHELL_COMMAND), `delete the firewall rules file` (was a ufw rule listing;
  now not understood — never an `rm` of an admin word), `show the groups file` (above), and — from
  decision 1 — two `cat …` lines on the host, which now reach the NL path instead of an AGNOS-only
  `owl` hint.
- The file-verb rule also corrects 1.9.15: of 189 file-verb sentences whose object holds a query or
  admin trigger, 31 that 1.9.15 answered with the object's action (`delete the add user script` →
  `useradd`, `find the disk usage report` → `df -h`) are no longer claimed by it; none moved the other
  way.
- A guard lives in the parser it protects, as a rule about that parser's own trigger.

**Negative**
- **Every NL line now pays the specific tier first.** Asked by scanning, its ~50 phrase checks cost
  4.3 µs a line (`parse/list_files` +326%). A PhraseIndex — one pass that records where each
  letter-led word starts, plus masks of first and second letters, so most phrases are rejected by an
  AND — brings the tier to 0.35–0.66 µs on the benchmarked lines. Net against 1.9.15, median of
  five alternating runs on a quiet host: `parse/list_files` 1.281 → 1.945 µs (+52%), `parse/cd`
  1.237 → 1.591 µs (+29%), `parse/find_files` 1.658 → 2.149 µs (+30%); `parse/git_status`
  2.354 → 1.560 µs (−34%), `parse/shell_cmd` 6.178 → 3.419 µs (−45%). The five together: −16%.
  The x86_64 binary grows 8.3 KB.
- Triggers are spelled exactly: `ip address` and `ip addresses` are two phrases.

**Neutral**
- A sentence that opens with a program's name is that program's command line, not natural language:
  on agnos `find files named foo` runs kriya's `find` today, and from 2.0.0 `git status` on the host
  runs git. About a third of the rows in `docs/examples/common-commands.md` open with such a word; the
  table documents the parser, which those lines reach only when no program by that name is found, and
  its scope note says so.
- `docs/guides/writing-intents.md` changes: a trigger belongs in the specific tier only if it is
  anchored; anything else goes to the broad tier, and the corpus test decides.

## Alternatives considered

- **Guard-by-guard, broad first** (the 1.9.5 posture). Each specific phrase needs a guard in every
  broad parser that shares one of its keywords; the guards are non-local and each costs time on a
  hot path. Rejected.
- **The naive reorder** — the 1.9.15 triggers, specific first: 13 regressions, 0 fixes. Rejected.
- **A "this is a shell line" heuristic in front of the cascade.** It would decide which parser gets
  a line on a guess; the dispatch layer's program lookup (decision 1) answers the same question with
  a fact. The `parse/shell_cmd` cost it was meant to cut fell 45% with the index anyway. Rejected.
- **Anchoring `>`** so a sentence containing it stays NL. Rejected by decision 2.
- **A table-driven specific tier** (phrases as data, matched word by word) would cut the remaining
  ~50 calls per line, at the price of moving every trigger away from the code it drives. Deferred
  until the tier's cost matters.

## References

- `src/interpreter.cyr` — `Interpreter_parse`, `input_has_phrase`, `phrase_index` /
  `index_has_phrase`, `trigger_claims`, `opens_with_file_verb`
- `src/agnsh.cyr` — dispatch order; `src/run_agnos.cyr` — `sh_try_pipeline_launch`,
  `sh_try_redirect_launch`, `sh_try_bareword_launch`
- `tests/test_parse_corpus.tcyr`, `docs/examples/common-commands.md`
- [CHANGELOG](../../CHANGELOG.md) 1.9.5 (the shadowing fixes) and 1.9.16
- [Roadmap](../development/roadmap.md) — 2.0.x NL exec (the host's `PATH` lookup)
