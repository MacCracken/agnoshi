# `/bin` tool launch on AGNOS — agnoshi exonerated (cause was kriya's stdlib pin)

- **Filed**: 2026-06-14
- **Repo**: agnoshi
- **agnoshi**: 1.7.0
- **Status**: RESOLVED — not an agnoshi bug. Kept as a record; close once the kriya fix lands.
- **Companion (the real bug + fix)**: [kriya `2026-06-14-bin-applets-crash-on-agnos-iron.md`](https://github.com/MacCracken/kriya/blob/main/docs/development/issue/2026-06-14-bin-applets-crash-on-agnos-iron.md)

## What looked like an agnoshi bug

On AGNOS, every external `/bin` tool launched from `agnsh` (`echo "Hello"`, `ls`, …) hung, taking the prompt with it. The suspicion was that agnsh's launch path (`run_agnos.cyr` → `execwait #37`) was mishandling dispatch, argv, or env.

## What testing actually showed

The agnoshi launch path is **correct**. Proven in QEMU on the current build:

- `bnrmr hi` launches via `execwait #37`, renders, and returns to the prompt — **agnsh's dispatch + argv + env path works**.
- Once kriya is rebuilt against a current cyrius stdlib (the real fix — see the kriya issue), `echo Hello` prints `Hello`, `owl -p /hello.txt` prints `OWLPROOF`, and `mkdir`/`cp`/`ls` all run from the prompt. Full `agnos/scripts/agnsh-delegation-test.py` goes green.

The hang was entirely inside the **kriya child binary** (a stale-stdlib-pin miscompile of the 934 KB dispatcher — it looped in its own `main()` before producing output). agnsh dispatched it correctly; the kernel loaded and entered it correctly. agnsh "hung" only in the sense that on single-core cooperative agnos a wedged foreground child never returns control — there was nothing for agnsh to do.

## The one genuine agnoshi-side follow-up (minor, optional)

When a foreground `/bin` child **wedges** (infinite loop, not a clean exit), `execwait #37` never returns and agnsh is stuck with it. This is inherent to the single-core cooperative model today and not fixable shell-side until the multithreading arc lands ([[project_multithreading_future_arc]]) — a wedged child can't be preempted. No action now; noting it so it isn't re-discovered as an agnsh bug. (A *crashing* child, vs a *hanging* one, already returns -1 cleanly via `sh_run_program`.)

## Takeaway

No agnoshi change required. The exec wire-up (v1.4.0 intent) is sound on real agnos hardware. Close this when the kriya pin bump is staged.
