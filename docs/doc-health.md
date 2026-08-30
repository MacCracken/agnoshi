---
name: Agnoshi Documentation Health
description: Living state of doc currency in the agnoshi repo — fresh / stale / archived / open-question, refreshed as docs are touched
type: state
---

# Documentation Health — agnoshi

> **Last refresh**: 2026-08-30 — **full sweep against the 1.9.9 tree**. Every doc
> below was audited for factual staleness, adversarially verified, and corrected.
> This replaces the previous ledger, whose rows had drifted so far that several
> claimed "Fresh" for files that were describing a v1.3-era binary.
> **Refresh cadence**: when docs are touched, update the affected row.
> **Scope**: this repo only — 7 root markdown files, 22 under `docs/`, plus the
> `agnsh.1` man page.

---

## What the sweep found, and why it matters

The dominant defect was **not** stale version numbers. It was **documented
security properties the shipped binary does not have.**

`README.md`, `SECURITY.md`, `docs/guides/security-model.md` and
`docs/examples/server-hardening.md` all asserted, as current fact: interactive
approval workflows, checkpoint/undo rollback, sudo re-verification at escalation
time, and a child-process environment whitelist. **None of those exist at
runtime.** They live in `src/approval.cyr`, `src/checkpoint.cyr`,
`src/security.cyr` and `src/session.cyr` — none of which is in the binary's
include graph — and `build_safe_env` has no caller anywhere in the tree.

`server-hardening.md` was the sharpest case: a deployment guide instructing
operators to install agnsh as a **login shell** with `agnsh --strict`, on the
stated basis that every command would require approval. `--strict` is not a
flag; it prints usage and exits **0**. It now opens with a do-not-deploy banner.

⇒ **The rule this sweep established**: a doc may describe a module that exists,
but it must never describe a *behaviour* the binary does not have without
marking it. Every such claim is now marked ⚠ or ⛔ with what is true instead.

**Second finding, from the same sweep**: verifying `docs/examples/`'s command
table against the actual binary found 5 of 46 rows wrong — which turned out to
be a **live code defect**, not a doc defect. The parser stored bare cstring
literals into `intent + 8` (a Str-typed field), leaving `SERVICE_CONTROL` and
`GIT_STASH` entirely dead since v1.0. Fixed, with regression tests. *Checking
docs against the code found a bug in the code.*

---

## Tier 1 — Root files

| File | Last touched | Status | Notes |
|---|---|---|---|
| `README.md` | 2026-08-30 | ✅ Fresh | Stat-line at 1.9.8. **Feature list corrected**: approval workflows and checkpoint/undo moved out of "Features" into an explicit not-shipped note. **Key-protections list corrected** — escape sanitization, env whitelist and sudo re-verification were listed as active and are not. Module tree rewritten to split **COMPILED** from **PRESENT BUT NOT IN THE BINARY**, which is the root cause of the whole class. |
| `CHANGELOG.md` | 2026-08-30 | ✅ Fresh | Source of truth for shipped work. Historical entries are correct as-written for their release and are not retro-edited. |
| `SECURITY.md` | 2026-08-30 | ✅ Fresh | Supported-versions table 1.0.x → 1.9.x. Four sections marked **⚠ NOT IN THE SHIPPED BINARY** (approval, checkpoint/undo, privilege escalation, the env whitelist) with what actually happens instead — including that AGNOS children *inherit* the environment, the opposite of a whitelist. OWASP section rewritten: ASI01 and ASI03 are now marked not-applicable/not-implemented rather than claimed. |
| `CONTRIBUTING.md` | 2026-08-30 | ✅ Fresh | Gate list completed (`lint-cstr-str.sh`, `check-coverage.sh`) **with the linter's literal-only blind spot stated** — a clean run is not proof. Sanitizer rule now distinguishes cstring guards from the `_in_str` twins. CheckpointManager rule marked forward-looking. Carries the **buffer-sizing-by-scope** rule (module 8N vs function N) that 1.9.8 established. |
| `CLAUDE.md` | 2026-08-30 | ✅ Fresh | `cyrius check` corrected to walk the entry. Version-sync step now names `scripts/version-bump.sh` and the `VERSION_STR` literal. "Security first" no longer claims sandbox/approval execution. |
| `benchmarks-rust-v-cyrius.md` | 2026-08-30 | 📦 Frozen | Historical port-arc record, frozen by design; its Rust-vs-Cyrius numbers are **not** to be updated. Only the current-toolchain references were corrected (5.10.x → 6.5.x). |
| `CODE_OF_CONDUCT.md` / `LICENSE` | — | 🔵 Evergreen | Re-read annually. |

---

## Tier 2 — Project state (`docs/development/`)

| File | Last touched | Status | Notes |
|---|---|---|---|
| `roadmap.md` | 2026-08-30 | ✅ Fresh | Forward-only. The v1.9.x arc's remaining work is **version-pinned** (1.9.9–1.9.12); 1.9.1–1.9.8 have shipped and are gone from the file. All ✅-completed sub-items stripped from Bucket 1 slices — Slice 2 and Slice 10 retitled, since neither was about what its name said. Two items relocated out of the arc (checkpoint blocker → Slice 4; prompt parent-walk → Bucket 2). |
| `issue/` | — | 📦 Archive | One archived issue record. Nothing open. |

---

## Tier 3 — Architecture (`docs/architecture/`)

| File | Last touched | Status | Notes |
|---|---|---|---|
| `overview.md` | 2026-08-30 | ✅ Fresh | Module map gained `statepaths.cyr` / `run_agnos.cyr` / `main.cyr` and now **annotates which modules are compiled**. Data-flow diagram rewritten: it showed approval → checkpoint → `execute_command` as the shipped path, none of which happens. Now shows the NL path stopping at report-and-audit, with execution as a separate branch carrying the exec-audit records. Counts and sizes refreshed (530 unit / 88 smoke / ~316 KB). |

---

## Tier 4 — ADRs (`docs/adr/`)

Point-in-time decision records: **frozen by design**, corrected only where the
current code contradicts a stated rule.

| File | Status | Notes |
|---|---|---|
| `README.md` | 🔵 Evergreen | Index. |
| `001-cyrius-port.md` | 📦 Frozen + corrected | `rust-old/` annotated as removed at v1.3.2. Consumer-app tags: corrected — they were pruned *out* of the enum at v1.0.0, so nothing is stubbed in-tree. |
| `002-struct-construction.md` | 📦 Frozen | Still holds. |
| `003-keyword-parser-over-regex.md` | 📦 Frozen + corrected | ⚠ Its stated premise ("Cyrius has no regex library") **no longer holds** — the stdlib gained an engine. The decision now rests explicitly on predictability/auditability/no-backtracking, not on absence. |
| `004-split-translate-match.md` | 📦 Frozen | Still load-bearing; re-read at 2.0.0. |
| `005-string-type-discipline.md` | 📦 Frozen + corrected | `has_path_traversal` renamed `path_traversal_in_str` under ADR-006 Rule 1. |
| `006-cstr-str-dispatch-discipline.md` | ✅ Fresh + corrected | The most load-bearing ADR in the tree — 1.9.1, 1.9.5 and 1.9.8 all leaned on it. Corrected: the `_in_str` example said `strlen` where the rule requires `str_len`; `SYS_GETGID`/`SYS_GETEUID` moved to the ✗ list (undefined on agnos); and **"the bug class is shut" softened to "the catalogued variants are shut"** — it was not shut, and three more instances shipped after that sentence was written. |

**ADR posture**: low decision-velocity. The open call worth an ADR is the
**dispatch-ordering decision** (roadmap 1.9.12) — four shadowing bugs from one
ordering property is exactly the kind of call ADRs exist for.

---

## Tier 5 — Audit reports (`docs/audit/`)

Date-stamped, frozen by design.

| File | Status | Notes |
|---|---|---|
| `2026-04-13.md` | 📦 Frozen | 1.0.0 P(-1) — 21 findings, all closed that cycle. |
| `2026-05-11-pminus1.md` | 📦 Frozen | v1.3.1 P(-1). ⚠ Its deferred items were closed in the 1.9.x arc, not the v1.4.0 it names; `security-model.md` now carries the status. |
| `2026-08-29-pminus1.md` | 📦 Frozen | **Current.** v1.9.1 P(-1): 8 dimensions, adversarially verified, 93 findings survived. Drove the entire 1.9.x arc. |

Next audit slot: 2.0.0 cut, or sooner if a new CVE pattern surfaces.

---

## Tier 6 — Guides (`docs/guides/`)

| File | Last touched | Status | Notes |
|---|---|---|---|
| `README.md` | — | 🔵 Evergreen | Index. |
| `getting-started.md` | 2026-08-30 | ✅ Fresh | Version/banner refreshed to a real 1.9.8 session. **Modes table rewritten** — it described approval tiers that do not exist; modes gate a *launch confirmation*, not per-tier approval. Audit-log section now documents the two disjoint label sets and `exit_code`. "Undo (v1.4.0)" replaced with an honest not-available note plus a **What actually executes** section. |
| `security-model.md` | 2026-08-30 | ✅ Fresh | Was the highest-risk doc and is now the most heavily corrected: four sections marked NOT-IN-THE-BINARY, the child-environment section replaced with the two real behaviours (host = **empty** env; AGNOS = full inheritance), the lint shield's blind spot documented, and the "known deferred" block re-titled with closure status. ⚠ Also records a **real, narrow gap it previously hid**: the confirmation that *does* ship (`verb_confirm`) does not strip control characters. |
| `writing-intents.md` | 2026-08-30 | 🟠 Partially fresh | Lint-shield description corrected. ⚠ Not yet re-read end-to-end against the 1.9.5 parser-ordering changes and the new `_in_str` guard convention — the contract it teaches for adding an intent should be re-walked when 1.9.12's ordering decision lands, since that may change it. |

---

## Tier 7 — Examples (`docs/examples/`)

| File | Last touched | Status | Notes |
|---|---|---|---|
| `README.md` | — | 🔵 Evergreen | Index. |
| `common-commands.md` | 2026-08-30 | ✅ Fresh | **All 46 documented rows now verified against the binary programmatically** — 5 were wrong, and fixing them uncovered the dead-intent code defect above. Questions section corrected (there is no LLM to hand off to); fall-through section now states that SHELL_COMMAND is classified, not run. |
| `scripting.md` | 2026-08-30 | ✅ Fresh | stdout/stderr split documented (1.9.4). Audit-log schema updated with `exit_code` and the two label sets; added `jq` recipes for "what actually ran" and for detecting a `launched` with no outcome. |
| `server-hardening.md` | 2026-08-30 | 🟠 Aspirational, clearly marked | ⛔ Opens with a **do-not-deploy banner** and a table of its five false claims vs reality. Kept rather than deleted because it is a legitimate design target for the Bucket 1 exec + approval slices — but it is no longer readable as instructions. |

---

## Tier 8 — Man page

| File | Last touched | Status | Notes |
|---|---|---|---|
| `docs/agnsh.1` | 2026-08-30 | ✅ Fresh | `.TH` at 1.9.8. `--mode` documented (with its fail-closed behaviour). `undo` builtin and the checkpoints FILES entry **removed** — neither exists; `run`, `reboot`/`poweroff`/`halt` added. MODES section rewritten to describe launch confirmation rather than per-command approval. Audit-log entry documents `exit_code`, both label sets, and the uid-qualified `/tmp` fallback. Renders warning-free under `groff -man`. |

---

## Standing risks in the documentation itself

1. **The compiled/unwired split is the single biggest source of doc drift.** Ten
   modules sit in `src/` and are not in the binary. Every doc that names a
   feature must be checked against `src/agnsh.cyr`'s include list, not against
   the existence of a file. README and `overview.md` now state the split
   explicitly so the next writer sees it.
2. **Examples decay silently.** `common-commands.md` was wrong in 5 of 46 rows
   and nothing caught it. That table is now machine-checkable — re-run the
   verification when the parser changes.
3. **`writing-intents.md` still owes an end-to-end re-read** (above).

*Refresh in place when docs are touched.*
