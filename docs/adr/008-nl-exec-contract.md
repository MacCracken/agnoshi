# ADR-008: The NL Execution Contract (2.0.0)

**Status:** Accepted (2026-09-23, for 2.0.0)
**Builds on:** [ADR-007](007-input-classification.md) — which lines are shell and which are natural
language. This ADR settles what happens once a natural-language line is understood.

## Context

Through 1.9.x the natural-language path only *proposes*: `agnsh -c "show files"` prints
`Intent: 0  Command: ls` and runs nothing. 2.0.0 makes it execute. Three questions had to be ruled
first (roadmap § Open decisions, "NL exec contract"), and each ruling has consequences for the
`-c` contract that scripts rely on:

1. Which modes execute, and do they confirm?
2. The report — `Intent:` / `Command:` / `Risk:` — is on stdout, and `docs/examples/scripting.md`
   tells scripts to pipe it. Once the command runs, its output needs stdout too.
3. How does the Linux host reach a program? Its only launcher is `run /abs/path`.

## Decision

### 1. What runs, and who confirms

- **SAFE and READ_ONLY translations execute.** USER_WRITE, SYSTEM_WRITE and ADMIN stay proposals
  (`Approval required -- not executed`) until approval-gated execution (2.0.3). BLOCKED never runs.
- A translation runs only if it names a program. UNKNOWN, a missing translation, QUESTION (no LLM
  yet), an NL pipeline (`… then …`), a translation the safety checks rejected, SHELL_COMMAND (the
  fall-through: its first word is not a program, or shell-first would have launched it), CHANGE_DIR
  (`cd` in a child process changes nothing — agnsh's own `cd` is roadmap 2.1.x) and a translation
  routed to an MCP tool (an `echo` placeholder) are reported and not run.
- An NL command goes through the same launcher as a typed `run` of it — the mode gate, the audit
  records, the exit status — with its program resolved per target (§ 3). Translation arguments are
  cstrings, converted once where the translators push them (ADR-006); they used to mix Strs and
  cstrings, which nothing noticed while nothing read them.
- **The mode policy is `run`'s** (`mode_needs_confirm`): `human` and `strict` confirm first, `auto`
  and `assist` run directly. `-c` defaults to `auto`, as it always has.
- Confirmation prompts go to **stderr**, for `run` as well — they are the shell talking, not the
  program.

### 2. The report goes to a report folder

With `-c`, **stdout carries only the program's output** and stderr its errors. agnsh's report — the
1.9.x lines (`Intent:`, `Command:`, `Risk:`, `Hint:`, `Approval required`, `WARNING: BLOCKED`)
verbatim, plus the input, mode, time and what happened — is written to a file:

| target | report folder |
|---|---|
| Linux host | `$XDG_STATE_HOME/agnoshi/reports/`, default `$HOME/.local/state/agnoshi/reports/` (the XDG base-directory rule: a relative `XDG_STATE_HOME` is ignored); with no `HOME`, `/tmp/agnoshi-reports.<uid>/` |
| agnos | `/.agnsh_reports/`, beside `/.agnsh_history` and `/.agnsh_audit.log` |

- The folder is created 0700. Each NL line run with `-c` writes `<YYYYMMDDTHHMMSSZ>-<pid>.txt`
  (0600, created exclusively, never through a symlink) and replaces `latest.txt` with the same
  content. The newest 100 reports are kept.
- On the host the folder must be a real directory owned by the user and not writable by others;
  otherwise no report is written and stderr says why. (The `/tmp` fallback is exactly where a
  pre-created folder is an attack.)
- When nothing runs, stdout stays empty and stderr gets one line: the reason and the report's path.
- **`--dry-run`** (`-n`) classifies, files the report, runs nothing, and prints the 1.9.x report on
  stdout. *Nothing* means nothing: every line — a bareword, `run`, a pipeline, a power verb — goes to
  the classifier, as it did on the host in 1.9.x; otherwise `agnsh -n -c "rm -rf x"` on agnos would
  still reach kriya's `rm`. A script that used `-c` to classify migrates by adding the flag:
  `agnsh --dry-run -c "$cmd" | grep -q 'Intent:'`.
- Shell lines — `run`, barewords, pipelines — have no NL report; the audit log records them.
- **The interactive shell keeps the on-screen report** (it is what the user reads before the
  command runs) and files nothing; history and the audit log already record the session.

### 3. The host reaches a program by `PATH`, through one launcher

- **One resolver** for shell-first barewords (ADR-007 decision 1) and for NL exec: a `PATH` lookup
  over *absolute* entries only — an empty or relative entry (which would mean the current
  directory) is skipped — returning the first executable regular file.
- **One launcher** for every host program — `run`, barewords, NL exec: fork and `execve` with an
  argv vector, never `/bin/sh -c` (a shell would reopen the metacharacter injection that audit C2
  closed), the child inheriting agnsh's environment (until now the host `run` passed an empty
  one), stdout and stderr untouched. `run PATH ARGS…` takes arguments on the host, as on agnos.
- Arguments split on whitespace, as agnos's launchers do; there is no quoting yet. A line with a
  shell metacharacter is refused before anything runs.
- `|`, `>` and a trailing `&` are shell syntax (ADR-007 decision 2). The host has no pipeline,
  redirect or job launcher yet, so on the host such a line is an error — never natural language.
- agnos keeps its launchers. NL exec there runs `/bin/<command>`; a translation whose program is not
  in `/bin` (`ps`, `df`, …) fails as `no such command`. `SHOW_FILE` translates to `owl -p`, agnos's
  reader, instead of `cat`, which agnos does not have.
- **A typed line the classifier calls BLOCKED always confirms** (ruled 2026-09-23). Shell lines —
  barewords, `run`, agnos pipelines, redirects and `&` jobs — are the user's own commands and do not
  pass the permission tiers (a documented gap until now), so shell-first on the host would have let
  `agnsh -c "rm -rf ~"` run in `auto`, where 1.9.x classified it BLOCKED and ran nothing. Every shell
  line is classified first; a BLOCKED one (`rm -rf`, `dd`, `mkfs`, `chmod`, `chown`, `shred`) asks
  `[y/N]` in **every** mode, and `-c` with no answer declines (126). Both targets — agnos's `auto`
  now asks before a typed `rm -rf dir` too. The AI never runs BLOCKED (§ 1); the human may, after an
  explicit yes. Other tiers keep the mode policy: gating them before approval exists (2.0.3) would
  stop agnos's kriya file verbs working.

### 4. Exit status of `-c`

| outcome | status |
|---|---|
| the program ran | its exit status (128 + N when signal N killed it) |
| understood but not run: approval required, BLOCKED, declined at the prompt (a BLOCKED shell line included), arguments refused by the safety checks | **126** |
| nothing to run: UNKNOWN, no translation, QUESTION, an NL pipeline, SHELL_COMMAND, or the program could not be found or launched | **127** |
| usage error | 1, as before |

126 and 127 are the shell's own codes for "found but not executable" and "not found". Shell lines
follow the same table (they returned 1 for every refusal and launch failure until 2.0.0), so a script
sees one convention whichever path handled its line. Under `--dry-run` the status is 0 for a line
that would run.

### 5. Audit

The parse-time record (`audit_one_shot`: `proposed` / `needs_approval` / `blocked` /
`rejected_safety` / …) keeps its vocabulary, with one correction: an understood USER_WRITE line is
now `needs_approval` with `approved: 0`, since that is what stops it running. Until 2.0.0 it was
`proposed` / `approved: 1` — true while nothing ran, false once SAFE and READ_ONLY lines did.
`approved` now follows the label. A command that runs adds the exec records `run` already writes:
`launched` before the child starts, then `executed` / `failed` with its exit code, or `error`.

### 6. Versioning

This is **2.0.0**. The `-c` stdout contract changes, and `-c` starts executing what it used to
only classify — both break scripts written against 1.x. The roadmap's 1.10.x arc becomes 2.0.x.

## Consequences

- `agnsh -c "show files"` prints a directory listing and exits with `ls`'s status.
- A script that parsed `-c` stdout must add `--dry-run` or read `latest.txt`.
- Host programs now inherit the environment (`PATH`, `HOME`, `LANG`, …), as on agnos.
- On the host, `ls | sort` is an error until the host gains a pipeline launcher — in 1.9.x it was
  an NL proposal.
- The report folder is new state on disk: bounded to 100 reports, 0700/0600.

## Alternatives considered

- **The report on stderr.** stderr carries the program's own errors; mixing agnsh's report into
  them makes both unparseable. Rejected by ruling.
- **Keep the report on stdout** (1.x). Program output and report interleave. Rejected by ruling.
- **Ship in 1.10.0 marked Breaking.** Keeps the numbering but breaks the SemVer the project declares.
  Rejected by ruling.
- **`/bin/sh -c` on the host** (lib `exec_cmd`). Gets quoting and pipelines for free and reopens
  injection. Rejected.
- **An empty environment for host programs** (the 1.x `run`). Tools that need `HOME`, `PATH` or
  `LANG` misbehave, and agnos already passes its environment. Rejected.
- **Filing interactive reports too.** Doubles what history and the audit log already hold.
  Rejected; reconsider if a use appears.

## References

- `src/agnsh.cyr` — `print_intent_result`, the `-c` path; `src/statepaths.cyr` — state paths
- `src/run_agnos.cyr` — `sh_run_program` and the agnos launchers; `src/sanitize.cyr` —
  `mode_needs_confirm`, `verb_confirm`
- [ADR-007](007-input-classification.md); `docs/examples/scripting.md`
- [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/latest/)
  — `$XDG_STATE_HOME`
