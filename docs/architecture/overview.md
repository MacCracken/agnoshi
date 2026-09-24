# Architecture Overview

## Module Map

```
agnoshi
├── src/
│   ├── agnsh.cyr           -- binary entry point (CLI + interactive loop)
│   ├── sanitize.cyr        -- input validation, JSON escape, env whitelist
│   ├── statepaths.cyr      -- where the state files live ($HOME / uid-qualified /tmp)
│   ├── mode.cyr            -- operating mode (human/assist/auto/strict)
│   ├── permissions.cyr     -- command classification (6-tier permission model)
│   ├── intent.cyr          -- Intent + Translation types, 45 intent tags
│   ├── interpreter.cyr     -- NL parse + translate dispatch
│   ├── translate.cyr       -- 40+ per-intent translators
│   ├── commands.cyr        -- command-line parsing, builtin detection
│   ├── approval.cyr        -- risk assessment, human approval UI
│   ├── security.cyr        -- SecurityContext, privilege escalation
│   ├── session.cyr         -- shell session lifecycle
│   ├── checkpoint.cyr      -- destructive op rollback
│   ├── audit.cyr           -- JSON audit log + audit-record construction
│   ├── history.cyr         -- command history (0600 perms)
│   ├── aliases.cyr         -- user-defined aliases
│   ├── completion.cyr      -- tab completion engine
│   ├── config.cyr          -- shell configuration
│   ├── output.cyr          -- output formatting (auto/json/table)
│   ├── prompt.cyr          -- prompt rendering
│   ├── ui.cyr              -- terminal UI helpers
│   ├── statepaths.cyr      -- state-file paths ($HOME, or uid-qualified /tmp)
│   ├── run_agnos.cyr       -- launch path: sh_run_program + (AGNOS) pipelines, redirect, bg jobs
│   └── main.cyr            -- legacy pre-port entry, never linked
│
│   ⚠ COMPILED (src/agnsh.cyr's include graph): sanitize, statepaths, mode,
│     permissions, intent, commands, translate, interpreter, approval, audit,
│     history, run_agnos — plus agnsh.cyr itself.
│     NOT COMPILED: security, session, checkpoint, completion, prompt, config,
│     aliases, output, ui, main. Features they implement (approval prompts,
│     undo, sudo, tab completion, git-branch prompt) do NOT exist at runtime.
├── lib/                    -- Cyrius stdlib (gitignored; populated by `cyrius deps`
│                              from the pinned snapshot in cyrius.cyml [deps] stdlib)
├── tests/
│   ├── test_core.tcyr      -- 678 unit tests
│   ├── test_security.tcyr  -- 26 security regression tests
│   ├── bench_core.bcyr     -- 11 criterion-style benchmarks
│   └── test.sh             -- run all test suites
├── scripts/
│   ├── install.sh          -- install to /usr/local/bin
│   ├── uninstall.sh        -- clean removal
│   ├── smoke-test.sh       -- 92 end-to-end binary tests
│   └── bench-history.sh    -- benchmark CSV tracker
└── docs/
    ├── agnsh.1             -- man page
    ├── architecture/       -- this directory
    ├── adr/                -- architectural decision records
    ├── audit/              -- security audit reports
    └── development/        -- roadmap, contribution workflow
```

## Data Flow

**Dispatch order** ([ADR-007](../adr/007-input-classification.md)): builtins and `run` first; then,
on AGNOS, the `/bin` launchers — pipeline, redirect, bareword. A shell line runs and never reaches the
parser, and a `|` / `>` line whose stage is not a program is an error, not natural language. Only
what is left reaches `Interpreter_parse`, which tries its specific tier (anchored phrases) before its
broad tier (single keywords). The Linux host has no launchers yet — 2.0.0 adds a `PATH` lookup — so
every host line that is not a builtin or `run` reaches the parser.

```
User Input (stdin)
    |
    v
[Interpreter_parse]  --> Intent struct (64 bytes: tag + 7 fields)
    |   specific tier, then broad tier (ADR-007)
    |                      |
    |                      v
    |                  [Interpreter_translate] --> Translation (cmd, args, perm, explanation)
    |                                                 |
    |                                                 v
    |                                         [analyze_command_permission]
    |                                                 |
    |                                                 v
    |                                         [print_intent_result]
    |                                           prints Intent / Command / Risk,
    |                                           and "Approval required" or
    |                                           "WARNING: BLOCKED" as a REPORT
    |                                                 |
    |                                                 v
    |                                         [audit_one_shot] -> JSON line
    |                                           result: proposed / needs_approval
    |                                                   / blocked / rejected_safety
    |
    |                                                 |
    |                          SAFE / READ_ONLY only (2.0.0, ADR-008):
    |                                                 v
    |                                         [nl_launch] -> the same launcher
    |                                           as `run` (below); -c files the
    |                                           report in the report folder
    |
    |   The approval loop (ApprovalManager_request) and checkpointing
    |   (CheckpointManager) are NOT in the include graph: USER_WRITE and above
    |   are reported and not run. Approval-gated exec is roadmap 2.0.x.
    |
    +--> SHELL LINES (checked before the parser, ADR-007):
    |      run /abs/path, or a bareword -> [shell-line gate: BLOCKED confirms in
    |      every mode] -> [sh_launch_line] -> host fork/execve (run_host.cyr),
    |      or AGNOS #37/#43
    |      host: bareword found on $PATH; `|`, `>`, `&` refused
    |      AGNOS: bareword /bin/<name>, cmd1 | cmd2, cmd > file, prog &
    |                                                 |
    |                                                 v
    |                       [audit_exec] -> "launched" BEFORE the child starts,
    |                       then executed / failed / error (with exit_code);
    |                       a refusal logs "denied" with approved:0
    |
    +--> History.add() --> ~/.agnsh_history (0600 at open, O_NOFOLLOW)
```

## Type System

Everything in Cyrius is i64. Structs are contiguous i64 fields at 8-byte offsets.

**Intent (64 bytes):**
```
offset 0:  tag (IntentTag enum value)
offset 8:  field1 (primary string: path, name, query)
offset 16: field2 (secondary: pattern, destination, service_name)
offset 24: field3 (tertiary)
offset 32: field4 (quaternary)
offset 40: int1 (flags, count, PID)
offset 48: int2 (secondary integer)
offset 56: vec1 (packages list, args vec)
```

**Translation (56 bytes):**
```
offset 0:  command (Str: executable name)
offset 8:  args (vec of Str)
offset 16: description (Str)
offset 24: permission (PermissionLevel enum)
offset 32: explanation (Str)
offset 40: mcp_tool (Str or 0)
offset 48: mcp_args (Str or 0, JSON payload)
```

## Permission Model

Every command is classified into one of six levels:

| Level | Meaning | Approval Required |
|-------|---------|-------------------|
| SAFE (0) | No state change (cd, echo, help) | No |
| READ_ONLY (1) | System query (ls, cat, ps) | No |
| USER_WRITE (2) | User file mod (cp, mv, touch) | No |
| SYSTEM_WRITE (3) | System path mod | Yes |
| ADMIN (4) | Requires sudo | Yes |
| BLOCKED (5) | Never allowed for AI | Human-only |

**Basename-first classification**: `/usr/bin/dd` → `dd` → BLOCKED. Prevents path-based bypass.

## Dependencies

**Build-time:**
- Cyrius 6.6.6 (pinned in `cyrius.cyml`: `cyrius = "6.6.6"`)
- Cyrius stdlib snapshot — declared in `cyrius.cyml` under `[deps] stdlib` (string, fmt, alloc, vec, str, syscalls, io, fs, chrono, hashmap, args, tagged, process, fnptr, net, sakshi, assert, bench). `./lib/` is gitignored — `cyrius deps` repopulates from the version-pinned snapshot before any build/check/lint step (38 files at 6.6.6 on a clean checkout; it does not delete files an earlier pin vendored, so clear `./lib/` first after a pin bump). (`json` was dropped from this list in v1.7.1: cyrius 6.2.25 folded standalone `json.cyr` into the `bayan` distlib, and agnoshi never consumed it — its `json_escape` is local to `src/sanitize.cyr`.)

**Runtime:**
- None (statically linked ELF, ~197 KB x86_64 (DCE) / ~661 KB aarch64 on Cyrius 6.6.6; was 146 KB on 4.5.0 at v1.0.0 — toolchain-side codegen growth from richer stdlib + the v1.2.0/v1.3.0 feature additions (audit, history, the exec paths), not from new agnoshi-side bloat. aarch64 DCE NOPs unreachable functions in place instead of removing them, so ~347 KB of the aarch64 figure is unreachable code)
- Optional: MCP gateway at `127.0.0.1:8090` for audit/agent/service queries
- Optional: LLM gateway at `127.0.0.1:8088` for question-mode answers

## Language Migration

This project was ported from Rust to Cyrius in April 2026. The original
Rust implementation lived in `rust-old/` through the v1.0–v1.3.1
port-arc + hardening cycle and was removed in v1.3.2 (see the v1.3.2
CHANGELOG entry for the parity audit). The historical record stays in
`benchmarks-rust-v-cyrius.md` (port-arc performance comparison, frozen
by design) and `docs/adr/001-cyrius-port.md` (port rationale).

Key changes:
- 27,251 lines Rust → 4,042 lines Cyrius (85% reduction)
- 3.8 MB dynamic binary → 146 KB static binary (−96%)
- Parse pipeline: 32 us → 1 us (**32× faster**)
- No external runtime deps (was: tokio, reqwest, serde, clap, regex, etc.)
