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

- [x] M1: Dynamic /proc/self/environ buffer (32KB, bounds-checked) (2026-04-13)
- [x] M2: PID validation via `is_valid_pid()` in translate (2026-04-13)
- [x] M3: Thorough rm flag parsing — combined flags, `--`, per-char scan (2026-04-13)
- [x] M4: `is_safe_path()` on all file translators — find, grep, cp, mv, rm, cat, mkdir (2026-04-13)
- [x] M5: Backslash escape handling in `split_command_line` quote parser (2026-04-13)
- [x] M6: Alias expansion metacharacter validation via `has_shell_metachars()` (2026-04-13)
- [x] M7: Checkpoint failure warning before destructive ops (2026-04-13)
- [x] M8: Username validation via `is_safe_username()` after passwd lookup (2026-04-13)
- [x] M9: Sudo re-verification (existence + root ownership) at escalation time (2026-04-13)
- [x] Security regression test suite: `tests/test_security.tcyr` (2026-04-13)
- [ ] Benchmark suite: `tests/bench_core.bcyr` — blocked on cc3 inline comment + include order fixes

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
