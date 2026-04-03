# Development Roadmap

## Completed

- **v0.1.0** — Initial extraction from agnosticos/userland/ai-shell (2026-04-01)
- **v0.2.0** — Standalone hardening: P(-1) pass, CI workflows, full cleanliness gates (2026-04-02)
- **v0.90.0** — Core shell domains, explain coverage, parser ordering fixes, API stabilization, ark integration, error recovery, revision workflow, richer LLM context, checkpoint/rollback, stiva container intents (2026-04-02)

## v1.0.0 Criteria

- [ ] Git workflow intents — commit, diff, branch, status, log, PR operations
- [ ] User/group management intents — useradd, userdel, usermod, passwd, groups
- [ ] Firewall intents — ufw/nftables allow, deny, list rules
- [ ] All 30+ translators production-tested
- [ ] Approval workflow battle-tested

## Future (post-v1.0.0, demand-gated)

- Docker compatibility layer — translate docker CLI syntax to stiva commands
- AI-powered tab completion — project-type-aware suggestions
- Streaming LLM responses in terminal
- Diff preview before destructive file operations
- SSH key management — generate, add, list, agent
- VPN/proxy configuration intents
- Systemd timers, sockets, and dependency management
- Log rotation intents
