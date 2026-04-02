# Architecture Overview

## Module Map

```
agnoshi
├── interpreter/         — NL→command translation engine
│   ├── intent.rs        — Intent classification (action, target, context)
│   ├── patterns.rs      — Pattern matching rules for all domains
│   ├── explain.rs       — Command explanation generator
│   ├── parse/           — Parsing subsystem
│   │   ├── creative.rs  — Creative/media command parsing
│   │   ├── platforms.rs — Platform-specific parsing
│   │   ├── system.rs    — System command parsing
│   │   └── tools.rs     — Tool command parsing
│   ├── translate/       — Per-domain command translators
│   │   ├── filesystem, process, network, system — OS domains
│   │   ├── agnos, package, marketplace — AGNOS domains
│   │   ├── photis, delta, bullshift, ... — Consumer app domains
│   │   ├── mcp_helper.rs — MCP tool call helper
│   │   └── misc.rs      — Catch-all translator
│   └── tests.rs         — Interpreter test suite
├── approval.rs          — Human-in-the-loop approval
├── security.rs          — Permission checks, sandbox
├── session.rs           — Session lifecycle
├── completion.rs        — Tab completion, fuzzy matching
├── commands.rs          — Built-in commands
├── dashboard.rs         — System status display
├── aliases.rs           — User aliases
├── config.rs            — Configuration
├── history.rs           — Command history
├── llm.rs               — LLM via hoosh
├── audit.rs             — Audit logging
├── prompt.rs            — Prompt rendering
├── output.rs            — Output formatting
├── ui.rs                — Terminal UI
├── sandbox.rs           — Sandbox integration
├── schema_filter.rs     — Schema filtering
├── mode.rs              — Shell modes
├── permissions.rs       — Permission model
├── lib.rs               — Library root
└── main.rs              — Binary entrypoint (agnsh)
```

## Data Flow

```
User Input → Interpreter → Intent Classification → Domain Translator → Command
                                                          ↕
                                              Security Check → Approval
                                                          ↕
                                              Sandbox Exec → Output → Audit
```

## Dependencies

- **agnos-common** — AGNOS shared types
- **agnosys** — Kernel interface
- **hoosh** — LLM gateway (via HTTP, port 8088)
- **daimon** — Agent runtime (via HTTP, port 8090)
