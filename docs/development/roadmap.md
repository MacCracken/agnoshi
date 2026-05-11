# Development Roadmap

## Shipped

- **v0.1.0** (2026-04-01) — Initial extraction from agnosticos/userland/ai-shell
- **v0.2.0** (2026-04-02) — Standalone hardening: P(-1) pass, CI workflows, cleanliness gates
- **v0.90.0** (2026-04-02) — Core shell domains, explain coverage, parser ordering, API stabilization, ark integration, error recovery, revision workflow, richer LLM context, checkpoint/rollback, stiva container intents
- **Cyrius port** (2026-04-13) — Rust → Cyrius port (27K → 4K lines, 21 modules), 32× parse speedup, 146 KB static binary
- **Security audit** (2026-04-13) — 21 findings closed (5 critical, 7 high, 9 medium)
- **v1.0.0** (2026-04-13) — Release candidate: tests passing, benchmarks proving Cyrius wins
- **v1.1.0** (2026-05-10) — Cyrius 5.10.34 + ecosystem-parity modernization. Toolchain pin bumped 4.5.0 → 5.10.34; manifest migrated `cyrius.toml` → `cyrius.cyml` with `version = "${file:VERSION}"` (single source of truth); `.cyrius-toolchain` retired; `./lib/` gitignored + repopulated by `cyrius deps` from the pinned stdlib snapshot (matches agnosys/yukti/patra); CI gate set expanded to syntax check + fmt diff + lint warn-as-error + vet + capacity gate + aarch64 best-effort cross-build + agnoshi-shaped security scan; release workflow accepts both `vX.Y.Z` and `X.Y.Z` tag styles with SHA256SUMS + per-arch prebuilt binaries (also fixed: was building wrong entry `src/main.cyr → agnoshi` instead of `src/agnsh.cyr → agnsh`); `CLAUDE.md` cleanliness gates rewritten Rust → Cyrius (`cargo fmt/clippy/audit/deny/doc` → `cyrius check/fmt/lint/vet/capacity`); `docs/doc-health.md` debut as a living doc-currency ledger; full closeout pass landed the five Stale rows it flagged (`README.md`, `CONTRIBUTING.md`, `docs/architecture/overview.md`, `docs/agnsh.1`, `benchmarks-rust-v-cyrius.md`). Binary 271,912 bytes on Cyrius 5.10.x (up from 146 KB on 4.5.0 — toolchain-side growth, not new agnoshi code). Full detail: `CHANGELOG.md` 1.1.0 entry.

## v1.2.x — Polish bucket (slotted)

The post-v1.0 quality items are split across three minor cuts so each is small enough to land cleanly without backfilling.

### v1.2.0 — Intent parsing depth + translator hardening
- [ ] Deeper intent parsing — currently classifies most NL inputs as `SHELL_COMMAND`; expand pattern coverage so the natural-language path is actually leveraged
- [ ] All core translators production-tested on real commands (filesystem, process, network, git, user/group, firewall, stiva)
- [ ] Coverage report wired into CI (target: 80%+)

### v1.2.1 — Approval workflow + interactive shell
- [ ] Approval workflow battle-tested interactively (decision UI, audit-log shape, sudo re-verification timing)
- [ ] Interactive shell end-to-end: history, prompt, mode switching, completion, error recovery loop
- [ ] Streaming LLM responses in terminal (already drafted in v0.90 spec — finish wiring)

### v1.2.2 — Packaging + zugot recipe
- [ ] zugot recipe bump for AGNOS packaging (`agnoshi` 1.2.2)
- [ ] Install path conventions reconciled with `ark install --group shell`
- [ ] Man page (`docs/agnsh.1`) regenerated for the 1.2.x command surface

## v1.3.x and beyond — Demand-gated

### Systems features
- Docker compatibility layer — translate docker CLI syntax to stiva commands
- SSH key management — generate, add, list, agent
- VPN/proxy configuration intents
- Systemd timers, sockets, and dependency management
- Log rotation intents
- Diff preview before destructive file operations

### UX features
- AI-powered tab completion — project-type-aware suggestions
- History fuzzy search (fzf-style)
- Rich prompt themes
- Man page integration (`explain <cmd>` pulls from man)

### Consumer app translators (from IntentTag stubs)
Stubbed for later — wire up only when the consumer app lands a public surface for agnoshi to translate into.

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
- Stiva (containers) — already partially shipped (12 intents in v0.90)
- Aequi (finance)
- Photis (tasks/habits)

## v2.0.0 — Future major

No scoped work yet. Candidates that would justify a major cut:

- Breaking change to the public API surface (intent enum, translator trait shape, session contract)
- Audit-log format break (would require migration tooling)
- Switch to a different LLM transport (away from hoosh)

Re-evaluate when the v1.2.x bucket is fully shipped.
