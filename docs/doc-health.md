---
name: Agnoshi Documentation Health
description: Living state of doc currency in the agnoshi repo — fresh / stale / archived / open-question, refreshed as docs are touched
type: state
---

# Documentation Health — agnoshi

> **Last refresh**: 2026-05-10 (initial audit, paired with the 1.1.0 modernization pass) | **Refresh cadence**: when docs are touched, update the affected row.
> **Scope**: This repo only (`agnoshi`) — root-level files (README, CHANGELOG, CLAUDE.md, etc.) plus the entire `docs/` tree.

This is a **ledger**, not a one-time audit. Rewrite-in-place as docs change. Pattern lifted from the agnosys ledger ([`agnosys/docs/doc-health.md`](https://github.com/MacCracken/agnosys/blob/main/docs/doc-health.md)) — same buckets, agnoshi-shaped tiers.

---

## At a glance — 2026-05-10 inventory

**~26 markdown files** total (7 root + 18 under `docs/`) plus the `agnsh.1` man page. Bucket counts at the start of the 1.1.0 cycle:

| Bucket | Count | What it means |
|---|---|---|
| ✅ **Fresh — refreshed in 1.1.0 modernization** | 6 | `CLAUDE.md` (Rust→Cyrius gate commands swapped, Known Issues purged), `docs/development/roadmap.md` (1.2.x slot rework), `CHANGELOG.md` (1.1.0 entry), `VERSION` (bumped), `doc-health.md` (this file, debut), `cyrius.cyml` (manifest migration). |
| 🟡 **Stale — refresh in 1.1.0 closeout** | 5 | `README.md` (still references Rust toolchain commands + 1.0.0 footprint), `CONTRIBUTING.md` (likely Rust-era), `docs/architecture/overview.md` (post-port Rust→Cyrius — verify module map matches `src/`), `docs/agnsh.1` (regenerate against 1.1.x command surface), `benchmarks-rust-v-cyrius.md` (header line/footnote about "cc3 limits" was misleading — see note below). |
| 🟠 **Read-through outstanding** | 4 | `docs/guides/getting-started.md`, `docs/guides/writing-intents.md`, `docs/guides/security-model.md`, `docs/examples/*` (4 files) — written during the v0.90/v1.0 cycle; need a per-page re-read against current src/ before stamping fresh, especially the security-model guide vs. the approval/sanitize/permissions wiring. |
| 🔵 **Probably evergreen** | 4 | `CODE_OF_CONDUCT.md`, `LICENSE`, `SECURITY.md` (reporting policy), `docs/adr/README.md` (index, not a decision record). Re-read annually. |
| 📦 **Archive / frozen by design** | 7 | The 5 ADRs (each a point-in-time decision record), `docs/audit/2026-04-13.md` (P(-1) report for 1.0.0 cycle), `docs/guides/README.md` / `docs/examples/README.md` (index files — refresh only when contents change). |
| ❓ **Open strategic question** | 1 | Whether `benchmarks-rust-v-cyrius.md` at the repo root should move to `docs/` (matches agnosys's posture) or stay at root as the port-arc headliner. Deferred to 1.2.x. |

**Doc cleanup landed in 1.1.0:**
- ✅ `CLAUDE.md` — Rust toolchain refs (`cargo fmt/clippy/audit/deny/doc`) swapped to Cyrius equivalents (`cyrius check/fmt/lint/vet/capacity`). Known Issues block purged: the "ModeManager undefined" note was self-contradicting (the struct is defined in `src/mode.cyr:8`); the "cc3 token limit" note was about a retired compiler and no longer applies on Cyrius 5.10.34.
- ✅ `docs/development/roadmap.md` — Shipped section dated, 1.1.0 scoped, polish items slotted across 1.2.0 / 1.2.1 / 1.2.2; demand-gated bucket relabeled to v1.3.x+.
- ✅ `docs/doc-health.md` — this file, debut.
- ✅ Manifest discipline note added — version lives in `VERSION`, `cyrius.cyml` pulls via `${file:VERSION}`, no duplicate edits.

---

## Tier 1 — Root files

| File | Last touched | Status | Notes |
|---|---|---|---|
| `README.md` | 2026-04-30 | 🟡 Stale | Likely still references the 1.0.0 binary size / parse-speedup numbers and may reference `cyrius.toml`. Refresh during 1.1.0 closeout: pin to Cyrius 5.10.34, link `cyrius.cyml`, note the `cyrius deps`-managed `./lib/`, and update the CI gate list. |
| `CHANGELOG.md` | 2026-05-10 | ✅ Fresh | 1.1.0 entry written this pass. Source of truth for shipped work. |
| `CLAUDE.md` | 2026-05-10 | ✅ Fresh | Rust→Cyrius gate commands swapped; stale Known Issues purged; version-discipline note added (VERSION is single SoT). |
| `CONTRIBUTING.md` | 2026-04-30 | 🟡 Stale | Likely Rust-era. Refresh: Cyrius prereq, the 5-gate cleanliness step, `cyrius deps` workflow, the `agnosys`-parity CI shape. |
| `SECURITY.md` | 2026-04-30 | 🔵 Evergreen | Reporting policy. No version-tied claims; re-read annually. |
| `CODE_OF_CONDUCT.md` | 2026-04-30 | 🔵 Evergreen | Standard. |
| `VERSION` | 2026-05-10 | ✅ Fresh | `1.1.0` — single source of truth, read into `cyrius.cyml` via `${file:VERSION}`. |
| `LICENSE` | (initial commit) | 🔵 Evergreen | GPL-3.0-only. |

---

## Tier 2 — Project state (`docs/development/`)

| File | Last touched | Status | Notes |
|---|---|---|---|
| `roadmap.md` | 2026-05-10 | ✅ Fresh | 1.1.0 in flight; 1.2.0 / 1.2.1 / 1.2.2 slotted for the polish bucket; v1.3.x+ holds demand-gated systems/UX/consumer-app translators. |

---

## Tier 3 — Architecture (`docs/architecture/`)

| File | Last touched | Status | Notes |
|---|---|---|---|
| `overview.md` | 2026-04-30 | 🟡 Stale | Post-port Rust→Cyrius — confirm the module map matches the 21 files in `src/` (agnsh, aliases, approval, audit, checkpoint, commands, completion, config, history, intent, interpreter, main, mode, output, permissions, prompt, sanitize, security, session, translate, ui). Refresh during 1.1.0 closeout. |

---

## Tier 4 — ADRs (`docs/adr/`)

| File | Last touched | Status | Notes |
|---|---|---|---|
| `README.md` | 2026-04-13 | 🔵 Evergreen | ADR index — refresh only when a new ADR lands. |
| `001-cyrius-port.md` | 2026-04-13 | 📦 Frozen | Accepted (1.0.0). Rust → Cyrius port rationale. Historical record. |
| `002-struct-construction.md` | 2026-04-13 | 📦 Frozen | Accepted (1.0.0). Cyrius struct-construction posture during port. Re-read at the next major if Cyrius gets a struct-literal syntax that obviates it. |
| `003-keyword-parser-over-regex.md` | 2026-04-13 | 📦 Frozen | Accepted (1.0.0). Parser-strategy choice. Verify still holds when the 1.2.0 "deeper intent parsing" work lands — it might surface a successor ADR. |
| `004-split-translate-match.md` | 2026-04-13 | 📦 Frozen | Accepted (1.0.0). Driven by cc3 per-fn match-arm limits — re-read at 1.2.x against Cyrius 5.10.34 limits; the underlying constraint may have moved. |
| `005-string-type-discipline.md` | 2026-04-13 | 📦 Frozen | Accepted (1.0.0). String/Str discipline rules for the Cyrius surface. |

**ADR posture**: low decision-velocity. Only architecturally significant calls earn an ADR. The 1.1.0 cycle is a modernization pass and doesn't earn one (the CHANGELOG entry carries the rationale; no architectural reversal). Re-evaluate at v2.0.0 cut.

---

## Tier 5 — Audit reports (`docs/audit/`)

Date-stamped, frozen by design. Each P(-1) hardening pass per CLAUDE.md cadence lands a new report.

| File | Date | Status | Notes |
|---|---|---|---|
| `2026-04-13.md` | 2026-04-13 | 📦 Frozen | 1.0.0 P(-1) — 21 findings (5 critical, 7 high, 9 medium), all closed in the same cycle. Historical record. |

Next audit slot: 1.2.0 P(-1) pass (paired with intent-parsing depth + translator hardening), or sooner if a CVE pattern surfaces in agnoshi's parser surfaces (shell tokenizer, JSON audit-log writer, alias expander, sanitize.cyr) or in Cyrius itself.

---

## Tier 6 — Guides (`docs/guides/`)

| File | Last touched | Status | Notes |
|---|---|---|---|
| `README.md` | 2026-04-13 | 📦 Frozen | Index — refresh only when the guides set changes. |
| `getting-started.md` | 2026-04-13 | 🟠 Read-through | Should still mostly hold — verify install path matches `scripts/install.sh` and that the first-run UX matches the current interactive shell. |
| `writing-intents.md` | 2026-04-13 | 🟠 Read-through | Re-read against the current `src/intent.cyr` + `src/interpreter.cyr` shape after the 1.2.0 "deeper intent parsing" work. |
| `security-model.md` | 2026-04-13 | 🟠 Read-through | High-priority re-read — must match the current approval/sanitize/permissions wiring; the v1.0 audit closed 21 findings that may have changed the surface. |

---

## Tier 7 — Examples (`docs/examples/`)

| File | Last touched | Status | Notes |
|---|---|---|---|
| `README.md` | 2026-04-13 | 📦 Frozen | Index — refresh only when examples set changes. |
| `common-commands.md` | 2026-04-13 | 🟠 Read-through | Verify each example still parses to the same intent on Cyrius 5.10.34 — a fmt or lint pass on the example snippets would surface drift. |
| `scripting.md` | 2026-04-13 | 🟠 Read-through | Same posture. |
| `server-hardening.md` | 2026-04-13 | 🟠 Read-through | Same posture; cross-check against the firewall/user intents that shipped in 0.90. |

---

## Tier 8 — Headliner / heritage docs

| File | Last touched | Status | Notes |
|---|---|---|---|
| `benchmarks-rust-v-cyrius.md` (repo root) | 2026-04-13 | 🟡 Stale — HEADLINER | Rust → Cyrius port-arc comparison, point-in-time at 1.0.0 / Cyrius 4.5.0. Per agnosys precedent (`docs/benchmarks-rust-vs-cyrius.md`) this kind of doc is **frozen by design** — but the prior CLAUDE.md's "exceeds cc3 limits" footnote is misleading (Cyrius 4.5.0+ has no such limit; the doc reads as if a current constraint, not a port-time observation). Either re-anchor the framing as historical at 1.1.0 closeout OR move it to `docs/` to match the agnosys home and rewrite the footnote. See the Open Question below. |
| `docs/agnsh.1` | 2026-04-13 | 🟡 Stale | Man page — regenerate against the 1.1.x command surface during closeout. |

---

## Open strategic questions

1. **`benchmarks-rust-v-cyrius.md` home + framing.** Stays at the repo root (current state) vs. moves to `docs/` to match agnosys. Either way the framing needs to switch to "historical port-arc, frozen by design" with the misleading cc3-limit footnote rewritten or removed. **Resolution slot**: 1.2.0 doc-sync step.

This section will repopulate when:

- A new doc category appears that doesn't fit an existing tier (e.g. `docs/standards/` or `docs/compliance/` if external spec conformance becomes a thing).
- The audit / review cadence shifts (current pattern: P(-1) at minor cuts).
- An ADR needs to be retired without a successor — would force a posture call.

---

## Forward doc-policy commitments

| # | Commitment | Trigger | Source | Notes |
|---|---|---|---|---|
| 1 | **Audit report retention** — keep all `docs/audit/YYYY-MM-DD.md` reports verbatim through at least v2.0.0; re-evaluate at the major cut whether pre-1.0 reports get folded into a single historical summary. | v2.0.0 cut | This file | Today's surface is 1 report — purge pressure is zero. |
| 2 | **ADR retention** — keep all current ADRs through v2.0.0; revisit ADR-003 / ADR-004 at v1.2.0 (intent parsing depth) and at the next compiler-limit lift (whichever comes first), since their underlying constraints may have moved. | v1.2.0 / v2.0.0 | This file | 5 ADRs at 1.1.0. |
| 3 | **Doc-health refresh** — this file is refreshed *opportunistically* when docs are touched, not on a schedule. Each minor cut's closeout pass updates the affected rows alongside CHANGELOG + roadmap. | Per-cut | CLAUDE.md Work Loop §10 | Pattern proven by the agnosys ledger. |

---

## Refresh procedure

When docs are touched:

1. Find the affected row in the relevant tier table.
2. Update **Last touched** column to the new date.
3. Update **Status** column if the bucket changed.
4. Update **Notes** column if the next step changed.
5. If a doc moved or was archived, update its row to reflect the new home.
6. Re-anchor "Last refresh" date in the header.

When the bucket counts at the top drift by more than ~3 in any cell, refresh the at-a-glance table.

---

## What this file is NOT

- Not a CHANGELOG (which records what shipped, not what's stale).
- Not a roadmap (forward work lives in [`development/roadmap.md`](development/roadmap.md)).
- Not a per-doc review log (we record the result of an audit pass, not the per-doc reasoning).

---

*Last refresh: 2026-05-10 (initial audit, paired with the 1.1.0 modernization pass). Refresh in place when docs are touched.*
