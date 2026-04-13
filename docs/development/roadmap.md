# Development Roadmap

## Completed

- **v0.1.0** — Initial extraction from agnosticos/userland/ai-shell (2026-04-01)
- **v0.2.0** — Standalone hardening: P(-1) pass, CI workflows, full cleanliness gates (2026-04-02)
- **v0.90.0** — Core shell domains, explain coverage, parser ordering fixes, API stabilization, ark integration, error recovery, revision workflow, richer LLM context, checkpoint/rollback, stiva container intents (2026-04-02)
- **Cyrius port** — Full port from Rust (27K lines) to Cyrius (3,643 lines), 20 modules (2026-04-13)
- **Security audit** — 21 findings (5 critical, 7 high, 9 medium), critical+high fixed (2026-04-13)

## v1.0.0 Criteria

- [x] Git workflow intents — commit, diff, branch, status, log, push, pull, checkout, merge, stash (2026-04-05)
- [x] User/group management intents — useradd, userdel, usermod, passwd, groupadd, groupdel, groups (2026-04-05)
- [x] Firewall intents — ufw allow, deny, list, status, enable, disable, delete rule (2026-04-05)
- [x] Cyrius port — all core modules ported (2026-04-13)
- [x] Security audit — critical and high findings fixed (2026-04-13)
- [ ] Medium security findings (M1-M9) addressed
- [ ] All core translators production-tested
- [ ] Approval workflow battle-tested

## Security Backlog (from 2026-04-13 audit)

- [ ] M1: Increase /proc/self/environ buffer or use dynamic allocation
- [ ] M2: Validate PID > 0 after str_to_int in kill parser *(fixed in translate)*
- [ ] M3: Parse rm flags more thoroughly — handle `--`, combined flags, flags after args
- [ ] M4: Block `../` in all file path arguments *(partially fixed via is_safe_path)*
- [ ] M5: Handle backslash escapes in split_command_line quote parser
- [ ] M6: Validate alias expansion values against metacharacter set
- [ ] M7: Block execution if checkpoint fails for destructive operations
- [ ] M8: Validate /etc/passwd username characters *(fixed)*
- [ ] M9: Re-verify sudo path/ownership/perms at escalation time, not just init
- [ ] Add integration test suite for each audit finding
- [ ] Add `sanitize.cyr` test coverage

## Future (post-v1.0.0, demand-gated)

- Docker compatibility layer — translate docker CLI syntax to stiva commands
- AI-powered tab completion — project-type-aware suggestions
- Streaming LLM responses in terminal
- Diff preview before destructive file operations
- SSH key management — generate, add, list, agent
- VPN/proxy configuration intents
- Systemd timers, sockets, and dependency management
- Log rotation intents
- Consumer app translators (Agnostic, Delta, Edge, Shruti, Tazama, etc.)
