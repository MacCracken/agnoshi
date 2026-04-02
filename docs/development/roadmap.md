# Development Roadmap

## v0.1.0 — Initial Extraction (current)

- [x] Extract from agnosticos/userland/ai-shell
- [x] Standalone Cargo.toml with path deps
- [x] Verify `cargo check` passes standalone
- [x] Verify `cargo test --all-features` passes
- [x] First benchmark baseline

## v0.2.0 — Standalone Hardening

- [x] P(-1) scaffold hardening pass
- [ ] CI workflows (ci.yml, release.yml)
- [x] Full clippy + fmt + audit + deny clean

## v1.0.0 Criteria

- [ ] Interpreter API stable
- [ ] All 30+ translators production-tested
- [ ] Approval workflow battle-tested
- [ ] 80%+ code coverage
- [ ] Independent ark package: `ark upgrade agnoshi`
