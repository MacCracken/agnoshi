# Agnoshi

**AI-native natural language shell for AGNOS.**

Agnoshi (Sanskrit: not-knowing → discovering through inquiry) is the AI shell interface for AGNOS. It translates natural language into system commands with human oversight, security approval workflows, and full audit logging.

## Features

- **Natural language interpretation** — translate intent to shell commands
- **19-file interpreter** — intent parsing, pattern matching, per-domain translation
- **30+ domain translators** — filesystem, process, network, packages, marketplace, and all consumer apps
- **Security model** — approval workflows, permission checking, sandbox execution
- **Session management** — history, context tracking, mode switching
- **Completion engine** — fuzzy matching, command suggestions
- **Dashboard** — system status, agent activity
- **Aliases** — user-defined command shortcuts

## Architecture

```
agnoshi
├── interpreter/       — NL→command translation engine
│   ├── intent.rs      — Intent classification
│   ├── patterns.rs    — Pattern matching rules
│   ├── parse/         — Creative, platform, system, tools parsing
│   ├── translate/     — 30+ per-domain translators
│   ├── explain.rs     — Command explanation
│   └── tests.rs       — Interpreter test suite
├── approval.rs        — Human-in-the-loop approval workflows
├── security.rs        — Permission checking, sandbox integration
├── session.rs         — Session lifecycle and context
├── completion.rs      — Tab completion and fuzzy matching
├── commands.rs        — Built-in shell commands
├── dashboard.rs       — System status display
├── aliases.rs         — User-defined aliases
├── prompt.rs          — Prompt rendering
├── output.rs          — Output formatting
├── ui.rs              — Terminal UI
├── config.rs          — Shell configuration
├── history.rs         — Command history
├── llm.rs             — LLM integration via hoosh
├── audit.rs           — Audit logging
└── main.rs            — Binary entrypoint (agnsh)
```

## Building

```bash
cargo build --release
```

## License

GPL-3.0-only
