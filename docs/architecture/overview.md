# Architecture Overview

## Module Map

```
agnoshi
├── src/
│   ├── agnsh.cyr           -- binary entry point (CLI + interactive loop)
│   ├── sanitize.cyr        -- input validation, JSON escape, env whitelist
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
│   ├── audit.cyr           -- JSON audit log
│   ├── history.cyr         -- command history (0600 perms)
│   ├── aliases.cyr         -- user-defined aliases
│   ├── completion.cyr      -- tab completion engine
│   ├── config.cyr          -- shell configuration
│   ├── output.cyr          -- output formatting (auto/json/table)
│   ├── prompt.cyr          -- prompt rendering
│   └── ui.cyr              -- terminal UI helpers
├── lib/                    -- Cyrius stdlib (gitignored; populated by `cyrius deps`
│                              from the pinned snapshot in cyrius.cyml [deps] stdlib)
├── tests/
│   ├── test_core.tcyr      -- 57 unit tests
│   ├── test_security.tcyr  -- 26 security regression tests
│   ├── bench_core.bcyr     -- 10 criterion-style benchmarks
│   └── test.sh             -- run all test suites
├── scripts/
│   ├── install.sh          -- install to /usr/local/bin
│   ├── uninstall.sh        -- clean removal
│   ├── smoke-test.sh       -- 20 end-to-end binary tests
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
[Interpreter_parse]  --> Intent struct (tag + 8 fields)
    |                      |
    |                      v
    |                  [Interpreter_translate] --> Translation (cmd, args, perm, explanation)
    |                                                 |
    |                                                 v
    |                                         [analyze_command_permission]
    |                                                 |
    |                  Risky? ---> [ApprovalManager_request] --> user approves/denies
    |                                                 |
    |                                                 v
    |                                         [CheckpointManager_checkpoint]  (for rm/mv)
    |                                                 |
    |                                                 v
    |                                         [execute_command] (fork+exec)
    |                                                 |
    |                                                 v
    |                                         stdout/stderr
    |                                                 |
    |                                                 v
    |                                         [AuditLogger_log] (JSON line)
    |
    +--> History.add() --> ~/.agnsh_history (mode 0600)
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
- Cyrius 6.0.1 (pinned in `cyrius.cyml`: `cyrius = "6.0.1"`)
- Cyrius stdlib snapshot — declared in `cyrius.cyml` under `[deps] stdlib` (string, fmt, alloc, vec, str, syscalls, io, fs, json, chrono, hashmap, args, tagged, process, fnptr, net, sakshi, assert, bench). `./lib/` is gitignored — `cyrius deps` repopulates from the version-pinned snapshot before any build/check/lint step.

**Runtime:**
- None (statically linked ELF, ~295 KB x86_64 / ~340 KB aarch64 on Cyrius 6.0.1; was 146 KB on 4.5.0 at v1.0.0 — toolchain-side codegen growth from richer stdlib + the v1.2.0/v1.3.0 feature additions (approval, audit, history, security modules wired in), not from new agnoshi-side bloat)
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
