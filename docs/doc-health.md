---
name: Agnoshi Documentation Health
description: Living state of doc currency in the agnoshi repo — fresh / stale / archived / open-question, refreshed as docs are touched
type: state
---

# Documentation Health — agnoshi

> **Last refresh**: 2026-08-29 (v1.9.1 — full P(-1) hardening sweep; new audit report, roadmap v1.9.x arc, security-model O_NOFOLLOW correction, CHANGELOG. v1.9.0 earlier the same day: Cyrius pin 6.3.34 → 6.5.36 + lib re-sync) | **Refresh cadence**: when docs are touched, update the affected row.
>
> ⚠ **Known ledger gap**: rows below were last swept at v1.3.4. Releases **v1.4.0 – v1.8.9** (fifteen of them) shipped without a ledger sweep, so any row still stamped `2026-05-28` reflects the v1.3.4 tree, not the current one. v1.9.0 refreshed only the rows it actually touched. `CHANGELOG.md` remains the source of truth for shipped work. **A full re-read sweep is its own slice.**
> **Scope**: This repo only (`agnoshi`) — root-level files (README, CHANGELOG, CLAUDE.md, etc.) plus the entire `docs/` tree.

This is a **ledger**, not a one-time audit. Rewrite-in-place as docs change. Pattern lifted from the agnosys ledger ([`agnosys/docs/doc-health.md`](https://github.com/MacCracken/agnosys/blob/main/docs/doc-health.md)) — same buckets, agnoshi-shaped tiers.

---

## At a glance — 2026-05-10 inventory (post-closeout)

**~26 markdown files** total (7 root + 18 under `docs/`) plus the `agnsh.1` man page. Bucket counts after the 1.1.0 closeout pass:

| Bucket | Count | What it means |
|---|---|---|
| ✅ **Fresh — refreshed in 1.1.0** | 11 | `CLAUDE.md`, `docs/development/roadmap.md`, `CHANGELOG.md` (1.1.0 entry), `VERSION` (bumped), `doc-health.md` (this file), `cyrius.cyml` (manifest migration), **plus the five closeout items below**: `README.md` (Cyrius 5.10.34 + `cyrius deps` install, 272 KB binary stat-line), `CONTRIBUTING.md` (cc3-era warnings stripped, cleanliness gates + `cyrius deps` step added), `docs/architecture/overview.md` (lib/ reframed as `cyrius deps`-managed, build-time req bumped 4.3.0+ → 5.10.34), `docs/agnsh.1` (header bumped 1.0.0 → 1.1.0), `benchmarks-rust-v-cyrius.md` (cc3 framing re-anchored as historical port-arc, in-tree refresh command added). |
| 🟠 **Read-through outstanding** | 4 | `docs/guides/getting-started.md`, `docs/guides/writing-intents.md`, `docs/guides/security-model.md`, `docs/examples/*` (4 files) — written during the v0.90/v1.0 cycle; need a per-page re-read against current src/ before stamping fresh, especially the security-model guide vs. the approval/sanitize/permissions wiring. **Slot**: 1.2.x where intent / approval / interactive-shell work touches the surfaces these guides describe. |
| 🔵 **Probably evergreen** | 4 | `CODE_OF_CONDUCT.md`, `LICENSE`, `SECURITY.md` (reporting policy), `docs/adr/README.md` (index, not a decision record). Re-read annually. |
| 📦 **Archive / frozen by design** | 7 | The 5 ADRs (each a point-in-time decision record), `docs/audit/2026-04-13.md` (P(-1) report for 1.0.0 cycle), `docs/guides/README.md` / `docs/examples/README.md` (index files — refresh only when contents change). |
| ❓ **Open strategic question** | 1 | Whether `benchmarks-rust-v-cyrius.md` at the repo root should move to `docs/` (matches agnosys's posture) or stay at root as the port-arc headliner. **Resolution slot**: 1.2.0 doc-sync. The misleading cc3 framing was fixed in 1.1.0 closeout; the home question is now the only thing outstanding. |

**Doc cleanup landed in 1.1.0:**
- ✅ `CLAUDE.md` — Rust toolchain refs (`cargo fmt/clippy/audit/deny/doc`) swapped to Cyrius equivalents (`cyrius check/fmt/lint/vet/capacity`). Known Issues block purged: the "ModeManager undefined" note was self-contradicting (the struct is defined in `src/mode.cyr:8`); the "cc3 token limit" note was about a retired compiler and no longer applies on Cyrius 5.10.34.
- ✅ `docs/development/roadmap.md` — Shipped section dated, 1.1.0 scoped, polish items slotted across 1.2.0 / 1.2.1 / 1.2.2; demand-gated bucket relabeled to v1.3.x+.
- ✅ `docs/doc-health.md` — this file, debut + closeout refresh.
- ✅ Manifest discipline note added — version lives in `VERSION`, `cyrius.cyml` pulls via `${file:VERSION}`, no duplicate edits.
- ✅ `README.md` — added "1.1.0 · Cyrius 5.10.34 · 21 modules · ~4 K src lines · 272 KB static binary (DCE) · 0 runtime deps" stat-line, install instructions prefixed with `cyrius deps`, 146 KB boast replaced with a 1.0.0-port-arc snapshot framing + pointer to in-tree benchmark refresh, "v1.0 minimal" agnsh.cyr annotation dropped now that the entry is shipped.
- ✅ `CONTRIBUTING.md` — `cyrius deps` step added; cleanliness gate command list (`cyrius check / capacity / vet / fmt / lint`) documented inline; cc3-era warnings purged (`//`-comments-with-colons mis-parse note, 40+ match-arm per-function limit). Cyrius 5.10.x trailing-comma rule from CHANGELOG carried in.
- ✅ `docs/architecture/overview.md` — `lib/` reframed as "Cyrius stdlib (gitignored; populated by `cyrius deps` from the pinned snapshot)"; build-time req bumped `cyrius v4.3.0+` → `Cyrius 5.10.34 pinned in cyrius.cyml`; runtime binary size annotated with the 146 KB → 272 KB 4.5.0→5.10.x toolchain-side growth.
- ✅ `docs/agnsh.1` — `.TH` header bumped `April 2026 / agnoshi 1.0.0` → `May 2026 / agnoshi 1.1.0`. Command surface unchanged in 1.1.0 (modes, builtins, options, files) so the body needed no edits.
- ✅ `benchmarks-rust-v-cyrius.md` — historical-port-arc framing added at top; cc3-limit references called out as point-in-time / no longer applicable on 5.10.34; in-tree refresh command (`cyrius build tests/bench_core.bcyr build/bench_core && ./build/bench_core`) wired in for current-toolchain numbers. Doc itself remains frozen by design.

---

## Tier 1 — Root files

| File | Last touched | Status | Notes |
|---|---|---|---|
| `README.md` | 2026-08-29 | ✅ Fresh | v1.9.1: stat-line bumped to `1.9.1 · Cyrius 6.5.36 · 22 modules · 308 KB / 532 KB · 356+26+59 tests` (tests 301 → 356, x86_64 binary 304 → 308 KB from the added validation + UTF-8 path; aarch64 unchanged at 532 KB). v1.9.0 set it to (`1.9.0 · Cyrius 6.5.36 · 22 modules · 304 KB / 532 KB · 301+26+59 tests`) and binary-size prose rebumped 6.0.56 → 6.5.36 (304 KB x86_64 / 532 KB aarch64, both re-measured in-session). Module count corrected 21 → **22** — it had been stale since a 22nd module landed in `src/`. Note the stat-line had read `1.4.5` while `VERSION` was `1.8.9`. |
| `CHANGELOG.md` | 2026-08-29 | ✅ Fresh | v1.9.1 entry cut — full P(-1) sweep (dead risk classifier, audit-log integrity, symlink races, inverted translator validation, 58.7× history fix, one honest benchmark regression recorded). v1.9.0 entry — pin 6.3.34 → 6.5.36 + lib re-sync; the two silent-gate fixes (CI fmt gate, bench-history parser); explicit **benchmark-methodology break** notice (6.5.x subtracts a measured clock-read floor, so 1.9.0 rows are not comparable to earlier ones and no perf win is claimed). Source of truth for shipped work — and currently the ONLY current record for v1.4.0–v1.8.9. |
| `CLAUDE.md` | 2026-08-29 | ✅ Fresh | v1.9.0: fmt-drift gate corrected to `cyrius fmt --check <file>` — the bare form **rewrites in place** as of 6.5.x, so the documented "diff against tree" step silently reformatted the tree it was meant to audit. Earlier: Rust→Cyrius gate commands swapped; stale Known Issues purged; version-discipline note added (VERSION is single SoT). |
| `CONTRIBUTING.md` | 2026-08-29 | ✅ Fresh | v1.9.0: pin ref bumped 6.0.14 → 6.5.36; cleanliness-gate block corrected to `cyrius fmt --check <file>` (the bare form rewrites in place on 6.5.x — the old line told contributors to diff its now-empty stdout). |
| `SECURITY.md` | 2026-04-30 | 🔵 Evergreen | Reporting policy. No version-tied claims; re-read annually. |
| `CODE_OF_CONDUCT.md` | 2026-04-30 | 🔵 Evergreen | Standard. |
| `VERSION` | 2026-08-29 | ✅ Fresh | `1.9.1` — single source of truth, read into `cyrius.cyml` via `${file:VERSION}`. Bumped via `scripts/version-bump.sh`, which also syncs the `VERSION_STR` banner literal in `src/agnsh.cyr` (the 1.8.9 desync gate). |
| `LICENSE` | (initial commit) | 🔵 Evergreen | GPL-3.0-only. |

---

## Tier 2 — Project state (`docs/development/`)

| File | Last touched | Status | Notes |
|---|---|---|---|
| `roadmap.md` | 2026-08-29 | ✅ Fresh | v1.9.0: **restructured to forward-facing only.** The `Shipped` history list and the closed v1.3.x bucket were deleted outright (`CHANGELOG.md` is the sole record of shipped work), which also retires the v1.4.0→v1.9.0 gap problem rather than papering over it. All 29 remaining items were re-verified against `CHANGELOG.md` and `src/`: **none had fully shipped**, so none was dropped. Nine carried stale premises and were rewritten — notably the `ui.cyr` wire-up (goal met natively in `agnsh.cyr`; now a two-stub cleanup), the exec slice (program exec ships via `run_agnos.cyr`; only the NL path remains), `>` redirection (truncate MVP shipped 1.8.3; `>>`/`<`/`2>`/host-port remain), `.agnshrc` (agnos envp blocker resolved — unblocked, not shipped), and the Stiva "already partially shipped" claim (false — those intents died with `rust-old/` in v1.3.2). Version labels dropped from bucket headers; stale `v1.2.x` re-eval pointer repointed. |

---

## Tier 3 — Architecture (`docs/architecture/`)

| File | Last touched | Status | Notes |
|---|---|---|---|
| `overview.md` | 2026-08-29 | 🟠 Partially fresh | v1.9.0: build-time req pin bumped 6.0.14 → 6.5.36; runtime binary-size line rebumped to 304 KB x86_64 / 532 KB aarch64. Also corrected a **factual error**: the `[deps] stdlib` list still named `json`, which v1.7.1 removed from the manifest (6.2.25 folded `json.cyr` into `bayan`; agnoshi never consumed it). ⚠ Module map NOT re-verified — `src/` now holds 22 files vs the 21 last checked, and the data-flow/module sections have not been read against v1.4.0–v1.8.9 exec/PTY/job-control work. |

---

## Tier 4 — ADRs (`docs/adr/`)

| File | Last touched | Status | Notes |
|---|---|---|---|
| `README.md` | 2026-05-11 | 🔵 Evergreen | ADR index — refresh only when a new ADR lands. Index entry added for ADR-006. |
| `001-cyrius-port.md` | 2026-04-13 | 📦 Frozen | Accepted (1.0.0). Rust → Cyrius port rationale. Historical record. |
| `002-struct-construction.md` | 2026-04-13 | 📦 Frozen | Accepted (1.0.0). Cyrius struct-construction posture during port. Re-read at the next major if Cyrius gets a struct-literal syntax that obviates it. |
| `003-keyword-parser-over-regex.md` | 2026-04-13 | 📦 Frozen | Accepted (1.0.0). Parser-strategy choice. v1.2.0's "deeper intent parsing" + word-prefix matcher landed without forcing a successor ADR — the keyword-parser posture still holds at the strategic level. |
| `004-split-translate-match.md` | 2026-04-13 | 📦 Frozen | Accepted (1.0.0). Driven by cc3 per-fn match-arm limits. Cyrius 5.10.x's capacity gate reports the underlying constraint at ~85% of 4096 fn-table slots — the split is still load-bearing. Re-read at v2.0.0 if the limit moves. |
| `005-string-type-discipline.md` | 2026-04-13 | 📦 Frozen | Accepted (1.0.0). Refined by ADR-006 in v1.3.1 (refinement, not replacement — the per-module convention table still holds). |
| `006-cstr-str-dispatch-discipline.md` | 2026-05-11 | ✅ Fresh | Accepted (v1.3.1 P(-1) slice 5). Refines ADR-005 with three operational rules (explicit `_in_str` suffix, per-arch syscall wrappers, `str_clone` for static-buf escape) + the 14-pattern CI lint shield. Closes the seven-variant bug class that surfaced over v1.2.0/v1.3.0. |

**ADR posture**: low decision-velocity. Only architecturally significant calls earn an ADR. v1.1.0 was a modernization pass (no ADR). v1.2.0 was a feature pass that didn't reverse any architectural call (no ADR). v1.3.0 saw a major refinement of ADR-005's discipline driven by repeated production discovery of the underlying bug class — that earns ADR-006. Re-evaluate at v2.0.0 cut.

---

## Tier 5 — Audit reports (`docs/audit/`)

Date-stamped, frozen by design. Each P(-1) hardening pass per CLAUDE.md cadence lands a new report.

| File | Date | Status | Notes |
|---|---|---|---|
| `2026-04-13.md` | 2026-04-13 | 📦 Frozen | 1.0.0 P(-1) — 21 findings (5 critical, 7 high, 9 medium), all closed in the same cycle. Historical record. |
| `2026-05-11-pminus1.md` | 2026-05-11 | 📦 Frozen | v1.3.1 P(-1) pass — closed. Final tally: 0 CRITICAL / 8 HIGH (all fixed) / 5 MEDIUM (deferred to v1.4.0: getcwd × 3 + str_data(Str)→syscall × 2) / 12 LOW (triaged). 14 lint patterns across 5 categories. All 7 historical bug variants CI-caught. Eight slices documented §1-§8 plus summary. |

| `2026-08-29-pminus1.md` | 2026-08-29 | 📦 Frozen | **v1.9.1 P(-1)** — 8 dimensions, adversarially verified, 93 findings survived. Final tally: 0 CRITICAL / 3 HIGH (all fixed) / ~20 MEDIUM (most fixed, rest bubbled) / rest LOW-INFO. Headline: the SHELL_COMMAND risk classifier was 100% dead (Str/cstring boundary) and the 301-test suite was green through it because every fixture used the wrong type. All five v1.3.1-deferred MEDIUMs closed in place. Carry-over is the roadmap's v1.9.x arc. Includes a method note on why 0/93 refutations is a weak signal. |

Next audit slot: 2.0.0 cut OR sooner if a new CVE pattern surfaces in agnoshi's parser surfaces / sanitize predicates / Cyrius itself. v1.3.1's audit closes out the v1.2.0/v1.3.0 bug-class arc; v1.4.0 exec wire-up may surface a new surface area (real fork+exec error handling, sudo-escalation edge cases) but those are exec-side concerns, not parser-side.

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
| `benchmarks-rust-v-cyrius.md` (repo root) | 2026-05-10 | ✅ Fresh (frozen by design) | Rust → Cyrius port-arc comparison, point-in-time at 1.0.0 / Cyrius 4.5.0. The cc3-limit framing was re-anchored as historical in 1.1.0 closeout (header callout + in-tree refresh command for current-toolchain numbers); substance left frozen. Home decision (root vs `docs/`) still open — see Open Question. |
| `docs/agnsh.1` | 2026-08-29 | 🟠 Partially fresh | v1.9.0: `.TH` header bumped `May 2026 / agnoshi 1.3.3` → `August 2026 / agnoshi 1.9.0` (it had sat at 1.3.3 across fifteen releases). v1.9.0 itself is a toolchain bump with no user-visible options/builtins delta, so its own change needed no body edits. ⚠ **Body NOT re-read against v1.4.0–v1.8.9**, which added job control, PTY and pipe behaviour — the documented command surface may be missing builtins. |

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

*Last refresh: 2026-08-29 (v1.9.0 — Cyrius 6.5.36 pin + lib re-sync; touched rows only: README, CHANGELOG, CONTRIBUTING, VERSION, roadmap, architecture/overview, agnsh.1. Rows still stamped 2026-05-28 or earlier reflect the v1.3.4 tree — see the known-ledger-gap note at the top.) Refresh in place when docs are touched.*
