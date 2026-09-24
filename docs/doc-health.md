---
name: Agnoshi Documentation Health
description: Living state of doc currency in the agnoshi repo — fresh / stale / archived / open-question, refreshed as docs are touched
type: state
---

# Documentation Health — agnoshi

> **Last refresh**: 2026-09-23 — **incremental, against the 1.9.14 tree** (cyrius 6.6.6). Rows
> touched by 1.9.13 and 1.9.14 are updated below; the rest carry forward.
>
> **Earlier**: 2026-08-29 — incremental against 1.9.10; 2026-08-30 — **full sweep against the 1.9.9
> tree**. Every doc
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
| `README.md` | 2026-09-23 | ✅ Fresh | 1.9.14: the `>` protection line says a symlinked target is refused on agnos. Stat-line at 1.9.13 (cyrius 6.6.6; 686 unit / 26 security / 92 smoke; 197 KB x86_64 DCE / 661 KB aarch64, with the aarch64 DCE caveat). ⚠ Corrected: `src/approval.cyr` **is** in the include graph (since May) — only its risk classifier is used; the prompt has no caller. Roadmap citations now name arcs. **Feature list corrected**: approval workflows and checkpoint/undo moved out of "Features" into an explicit not-shipped note. **Key-protections list corrected** — escape sanitization, env whitelist and sudo re-verification were listed as active and are not. Module tree rewritten to split **COMPILED** from **PRESENT BUT NOT IN THE BINARY**, which is the root cause of the whole class. |
| `CHANGELOG.md` | 2026-09-23 | ✅ Fresh | Source of truth for shipped work. Historical entries are correct as-written for their release and are not retro-edited — ⚠ except that 1.9.1 / 1.9.4 state the per-arch `O_NOFOLLOW` claim that 1.9.13 found wrong; the 1.9.13 entry carries the correction. The 1.9.12 entry, filed 1,400 lines down under a stray `## [Unreleased]`, was moved to its place and `## [Unreleased]` restored at the top. |
| `SECURITY.md` | 2026-09-23 | ✅ Fresh | ⛔ 1.9.14: § Audit log now says the log only accumulates on agnos from 1.9.14 (before, each record overwrote the last — measured in QEMU). Roadmap citations moved from `Bucket 1 Slice N` to arc slots (1.9.13). Supported-versions table 1.0.x → 1.9.x. Four sections marked **⚠ NOT IN THE SHIPPED BINARY** (approval, checkpoint/undo, privilege escalation, the env whitelist) with what actually happens instead — including that AGNOS children *inherit* the environment, the opposite of a whitelist. OWASP section rewritten: ASI01 and ASI03 are now marked not-applicable/not-implemented rather than claimed. |
| `CONTRIBUTING.md` | 2026-09-23 | ✅ Fresh | 1.9.14: § Development shows how to run the agnos QEMU harness. 1.9.13: the fmt gate is one multi-file `cyrius fmt --check` (`scripts/check-fmt.sh` retired — cyrius 6.6.5 fixed the file-list bug), pin literal at 6.6.6. Gate list completed (`lint-cstr-str.sh`, `check-coverage.sh`) **with the linter's literal-only blind spot stated** — a clean run is not proof. 1.9.10 added lint **Category H** (the audit-path test seam must not be called from `src/`) and fixed the coverage gate's comment-mention blind spot; both noted in § Gates. Sanitizer rule now distinguishes cstring guards from the `_in_str` twins. CheckpointManager rule marked forward-looking. Carries the **buffer-sizing-by-scope** rule (module 8N vs function N) that 1.9.8 established. |
| `CLAUDE.md` | 2026-09-23 | ✅ Fresh | 1.9.14: step 2 names `scripts/agnos-qemu-test.py` for agnos paths. 1.9.13: fmt gate updated; step 1 now says to `rm -rf lib && cyrius deps` at a pin bump; `approval.cyr` claim corrected. `cyrius check` corrected to walk the entry. Version-sync step now names `scripts/version-bump.sh` and the `VERSION_STR` literal. "Security first" no longer claims sandbox/approval execution. |
| `benchmarks-rust-v-cyrius.md` | 2026-08-30 | 📦 Frozen | Historical port-arc record, frozen by design; its Rust-vs-Cyrius numbers are **not** to be updated. Only the current-toolchain references were corrected (5.10.x → 6.5.x). |
| `CODE_OF_CONDUCT.md` / `LICENSE` | — | 🔵 Evergreen | Re-read annually. |

---

## Tier 2 — Project state (`docs/development/`)

| File | Last touched | Status | Notes |
|---|---|---|---|
| `roadmap.md` | 2026-09-23 | ✅ Fresh | 1.9.14 shipped and retired from the file (1.9.x now 1.9.15–1.9.16). **Rewritten at 1.9.13 into version-pinned release arcs** — 1.9.x close-out, 1.10.x NL execution, 1.11.x interactive shell, 1.12.x hoosh/LLM — plus Open decisions, Gated (with the agnos issue each gate is filed as), a standing pin-bump checklist replacing the per-version pin sections, and a release checklist. Slots are cited as `roadmap <arc> — <title>`, never by patch number. Every gate re-verified at 6.6.6; three had opened (agnos verification via QEMU, agnos `AO_NOFOLLOW`, the host LLM client). |
| `issue/` | — | 📦 Archive | One archived issue record. Nothing open. |

---

## Tier 3 — Architecture (`docs/architecture/`)

| File | Last touched | Status | Notes |
|---|---|---|---|
| `overview.md` | 2026-09-23 | ✅ Fresh | 1.9.13: toolchain 6.6.6, binary sizes, the clean re-vendor note, and the NL-exec citation. Counts refreshed to 678 / 26 / 92; 45 intent tags (MEMORY_INFO); `run_agnos.cyr` re-described — `sh_run_program` moved there out of the entry file in 1.9.10. Module map gained `statepaths.cyr` / `run_agnos.cyr` / `main.cyr` and now **annotates which modules are compiled**. Data-flow diagram rewritten: it showed approval → checkpoint → `execute_command` as the shipped path, none of which happens. Now shows the NL path stopping at report-and-audit, with execution as a separate branch carrying the exec-audit records. Counts and sizes refreshed (678 unit / 92 smoke / ~316 KB). |

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

**ADR posture**: low decision-velocity. The open call worth an ADR is
**ADR-007 — input classification** (roadmap 1.9.x — ADR-007), which merges the
dispatch-ordering decision, the pre-cascade shell-line discriminator and the
unanchored `>` scan: four shadowing bugs from one ordering property is exactly the
kind of call ADRs exist for, and it must land before the NL path executes (1.10.x).

---

## Tier 5 — Audit reports (`docs/audit/`)

Date-stamped, frozen by design.

| File | Status | Notes |
|---|---|---|
| `2026-04-13.md` | 📦 Frozen | 1.0.0 P(-1) — 21 findings, all closed that cycle. |
| `2026-05-11-pminus1.md` | 📦 Frozen | v1.3.1 P(-1). ⚠ Its deferred items were closed in the 1.9.x arc, not the v1.4.0 it names; `security-model.md` now carries the status. |
| `2026-08-29-pminus1.md` | 📦 Frozen + corrected | **Current.** v1.9.1 P(-1): 8 dimensions, adversarially verified, 93 findings survived. Drove the entire 1.9.x arc. ⛔ Its `O_NOFOLLOW` paragraph and method note are annotated (1.9.13): the "no per-arch split" claim it made was wrong and shipped. |

Next audit slot: 2.0.0 cut, or sooner if a new CVE pattern surfaces.

---

## Tier 6 — Guides (`docs/guides/`)

| File | Last touched | Status | Notes |
|---|---|---|---|
| `README.md` | — | 🔵 Evergreen | Index. |
| `getting-started.md` | 2026-09-23 | ✅ Fresh | 1.9.13: NL-exec citation moved to the arc slot. Version/banner refreshed to a real 1.9.8 session. **Modes table rewritten** — it described approval tiers that do not exist; modes gate a *launch confirmation*, not per-tier approval. Audit-log section now documents the two disjoint label sets and `exit_code`. "Undo (v1.4.0)" replaced with an honest not-available note plus a **What actually executes** section. |
| `security-model.md` | 2026-09-23 | ✅ Fresh | ⛔ 1.9.14: § Audit Log Integrity's launched-record guarantee is now marked as never having held on agnos before 1.9.14; the `>` redirect's missing `AO_NOFOLLOW` is recorded as closed, with the QEMU evidence. ⛔ 1.9.13: the symlink-race item's 1.9.1 "correction" (no per-arch `O_NOFOLLOW`) **was wrong** — replaced with the real history (x86_64 closed 1.9.1; aarch64 and agnos 1.9.13) and the `>` redirect gap. Citations moved to arc slots. Was the highest-risk doc and is now the most heavily corrected: four sections marked NOT-IN-THE-BINARY, the child-environment section replaced with the two real behaviours (host = **empty** env; AGNOS = full inheritance), the lint shield's blind spot documented, and the "known deferred" block re-titled with closure status. ⚠ Also records a **real, narrow gap it previously hid**: the confirmation that *does* ship (`verb_confirm`) does not strip control characters. |
| `writing-intents.md` | 2026-08-29 | ✅ Fresh | **Re-walked end-to-end by actually adding an intent** (MEMORY_INFO, 1.9.10) — the debt this row recorded is paid, and walking it found two gaps the guide had. (1) It documented the dispatch *split* but not that both halves are **bounded ranges**: a new tag is by definition the highest number, so it lands above the extended range and is never dispatched — arm present, correct, unreachable, `echo` symptom. Now documented with the widen-the-bound step and a through-the-dispatch test. (2) Nothing warned that parse arms are ordered-first-match or that every arm is paid for by every unrecognised line; both now carry the measured evidence (the +3.9% six-check arm). ⚠ Still to re-read when ADR-007 lands (roadmap 1.9.x — ADR-007), since that may change the contract again. |

---

## Tier 7 — Examples (`docs/examples/`)

| File | Last touched | Status | Notes |
|---|---|---|---|
| `README.md` | — | 🔵 Evergreen | Index. |
| `common-commands.md` | 2026-08-29 | ✅ Fresh | Three MEMORY_INFO rows added (`show memory usage` / `show free memory` / `ram usage` → `free -h`), verified against the 1.9.10 binary. **All 46 previous rows were verified against the binary programmatically** — 5 were wrong, and fixing them uncovered the dead-intent code defect above. Questions section corrected (there is no LLM to hand off to); fall-through section now states that SHELL_COMMAND is classified, not run. |
| `scripting.md` | 2026-08-30 | ✅ Fresh | stdout/stderr split documented (1.9.4). Audit-log schema updated with `exit_code` and the two label sets; added `jq` recipes for "what actually ran" and for detecting a `launched` with no outcome. |
| `server-hardening.md` | 2026-09-23 | 🟠 Aspirational, clearly marked | 1.9.13: citations moved to arc slots. ⛔ Opens with a **do-not-deploy banner** and a table of its five false claims vs reality. Kept rather than deleted because it is a legitimate design target for the Bucket 1 exec + approval slices — but it is no longer readable as instructions. |

---

## Tier 8 — Man page

| File | Last touched | Status | Notes |
|---|---|---|---|
| `docs/agnsh.1` | 2026-08-29 | ✅ Fresh | `.TH` at 1.9.10. `--mode` documented (with its fail-closed behaviour). `undo` builtin and the checkpoints FILES entry **removed** — neither exists; `run`, `reboot`/`poweroff`/`halt` added. MODES section rewritten to describe launch confirmation rather than per-command approval. Audit-log entry documents `exit_code`, both label sets, and the uid-qualified `/tmp` fallback. Renders warning-free under `groff -man`. |

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
3. **A gate that credits a mention is worse than no gate.** `check-coverage.sh`
   counted a function as covered if its name appeared anywhere in a test file —
   including in a comment. It happened for real while writing the 1.9.10 seams,
   and hid five functions that had never been asserted. Fixed, but the shape of
   the mistake generalises: when adding a gate, check what it accepts, not only
   what it rejects. Both new gates in 1.9.10 were verified by making them fail
   on purpose before being trusted.
4. **A test can lock in the wrong answer and pass forever.** `check("config
   history", load64(config + 8) == 10000)` asserted the value nothing used, and
   the smoke test named `show memory usage is not df` asserted `uname`. Both
   passed for releases. Prefer asserting against the constant or the actual
   expected output over a literal that merely matches today's behaviour.
5. **A guarantee documented for a target nothing runs is a hypothesis.**
   `security-model.md` promised that a `launched` record with no outcome is the
   signal of a hung or fatal launch — and on agnos, where every one of those
   launches happens, the first QEMU run (1.9.14) found the log kept one record at
   a time. The aarch64 `O_NOFOLLOW` defect (1.9.13) was the same shape. Before a
   doc states a property, check which targets it has actually been exercised on.

*Refresh in place when docs are touched.*
