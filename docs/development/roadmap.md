# Development Roadmap

## Completed

- **v0.1.0** — Initial extraction from agnosticos/userland/ai-shell (2026-04-01)
- **v0.2.0** — Standalone hardening: P(-1) pass, CI workflows, full cleanliness gates (2026-04-02)
- **v0.90.0** — Core shell domains, explain coverage, parser ordering fixes, API stabilization, ark integration, error recovery, revision workflow, richer LLM context, checkpoint/rollback, stiva container intents (2026-04-02)
- **Cyrius port** — Full port from Rust (27K lines) to Cyrius (4K lines), 21 modules (2026-04-13)
- **Security audit** — 21 findings (5 critical, 7 high, 9 medium), all fixed (2026-04-13)
- **v1.0.0** — Release candidate: tests passing, benchmarks showing 32× parse speedup vs Rust (2026-04-13)

## v1.0.0 — Shipped

- [x] Git workflow intents (2026-04-05)
- [x] User/group management intents (2026-04-05)
- [x] Firewall intents (2026-04-05)
- [x] Cyrius port (2026-04-13)
- [x] Security audit — all findings fixed (2026-04-13)
- [x] Benchmark suite running (2026-04-13)
- [x] Unit test suite: test_core.tcyr (57 tests) + test_security.tcyr (26 tests)
- [x] Smoke test: 20 end-to-end checks on the binary
- [x] CI pipeline: build + smoke + bench on every push
- [x] Install script: `sudo sh scripts/install.sh`
- [x] Man page: `docs/agnsh.1`
- [x] Static 146KB binary (no dynamic deps)
- [x] Checkpoint auto-prune (keeps 100 most recent)

## Post-v1.0 — Quality / Polish

- [ ] Deeper intent parsing (currently classifies most as SHELL_COMMAND, works but doesn't leverage NL)
- [ ] All core translators production-tested on real commands
- [ ] Approval workflow battle-tested interactively
- [ ] Interactive shell end-to-end (history, prompt, mode switching)
- [ ] zugot recipe bump for packaging in AGNOS

## Future (post-v1.0.0, demand-gated)

### Systems features
- Docker compatibility layer — translate docker CLI syntax to stiva commands
- SSH key management — generate, add, list, agent
- VPN/proxy configuration intents
- Systemd timers, sockets, and dependency management
- Log rotation intents
- Diff preview before destructive file operations

### UX features
- AI-powered tab completion — project-type-aware suggestions
- Streaming LLM responses in terminal
- History fuzzy search (fzf-style)
- Rich prompt themes
- Man page integration (`explain <cmd>` pulls from man)

### Consumer app translators (from IntentTag stubs, stubbed for later)
- Agnostic (QA orchestration)
- Delta (git hosting)
- Edge (fleet management)
- Shruti (DAW)
- Tazama (video editor)
- Rasa (image canvas)
- Mneme (knowledge base)
- Synapse (LLM management)
- BullShift (trading)
- Yeoman (agent orchestration)
- Phylax (threat detection)
- T-Ron (security monitor)
- Tarang (media analysis)
- Jalwa (media player)
- Stiva (containers)
- Aequi (finance)
- Photis (tasks/habits)
