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
│   ├── intent.cyr          -- Intent + Translation types, 44 intent tags
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
│   ├── run_agnos.cyr       -- AGNOS launch path: exec, pipelines, redirect, bg jobs
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
│   ├── test_core.tcyr      -- 530 unit tests
│   ├── test_security.tcyr  -- 26 security regression tests
│   ├── bench_core.bcyr     -- 11 criterion-style benchmarks
│   └── test.sh             -- run all test suites
├── scripts/
│   ├── install.sh          -- install to /usr/local/bin
│   ├── uninstall.sh        -- clean removal
│   ├── smoke-test.sh       -- 88 end-to-end binary tests
│   └── bench-history.sh    -- benchmark CSV tracker
└── docs/
    ├── agnsh.1             -- man page
    ├── architecture/       -- this directory
    ├── adr/                -- architectural decision records
    ├── audit/              -- security audit reports
    └── development/        -- roadmap, contribution workflow
```

## Data Flow

```
User Input (stdin)
    |
    v
[Interpreter_parse]  --> Intent struct (64 bytes: tag + 7 fields)
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
    |   ⛔ THE NL PATH STOPS HERE. It does not execute. The approval loop
    |      (ApprovalManager_request) and checkpointing (CheckpointManager) live
    |      in src/session.cyr + src/checkpoint.cyr, NEITHER of which is in the
    |      binary's include graph. Wiring exec in is roadmap Bucket 1 Slice 5.
    |
    +--> EXECUTION, a separate path entirely:
    |      run /abs/path  -> [sh_run_program] -> host fork/exec, or AGNOS #37/#43
    |      AGNOS only:    bareword /bin/<name>, cmd1 | cmd2, cmd > file, prog &
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
- Cyrius 6.5.36 (pinned in `cyrius.cyml`: `cyrius = "6.5.36"`)
- Cyrius stdlib snapshot — declared in `cyrius.cyml` under `[deps] stdlib` (string, fmt, alloc, vec, str, syscalls, io, fs, chrono, hashmap, args, tagged, process, fnptr, net, sakshi, assert, bench). `./lib/` is gitignored — `cyrius deps` repopulates from the version-pinned snapshot before any build/check/lint step. (`json` was dropped from this list in v1.7.1: cyrius 6.2.25 folded standalone `json.cyr` into the `bayan` distlib, and agnoshi never consumed it — its `json_escape` is local to `src/sanitize.cyr`.)

**Runtime:**
- None (statically linked ELF, ~316 KB x86_64 / ~532 KB aarch64 on Cyrius 6.5.36; was 146 KB on 4.5.0 at v1.0.0 — toolchain-side codegen growth from richer stdlib + the v1.2.0/v1.3.0 feature additions (approval, audit, history, security modules wired in), not from new agnoshi-side bloat)
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
