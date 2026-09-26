# agnsh's poll + sched_yield loops should use agnos's blocking waits

- **Filed**: 2026-09-26
- **Repo**: agnoshi
- **agnoshi**: 2.0.0
- **Found by**: agnos 1.57.9 (the `sched_yield`#44 ruling)
- **Status**: ✅ **RESOLVED in agnoshi 2.0.1 (2026-09-26)** — loops 1 and 2 converted; loop 3 kept as the issue
  says, and documented as the one deliberate poll. No agnos change was needed.

## Why now

agnos 1.57.8 made `#44` wake another yielding CPU; 1.57.9 took that back (operator ruling): plain `#44` is again a quiet,
local yield — as Linux `sched_yield` and FreeBSD `sched_relinquish` are — and a new `#108 sched_yield_to(pid)` covers the
one case where the caller knows whom it waits for (Mach `thread_switch`, Linux `yield_to`). So a `#44` loop that waits for
another process now parks up to one timer tick (~10 ms) per round. The Linux `sched_yield(2)` man page is explicit that
waiting for another thread with `sched_yield` is a design error; the waits belong in blocking primitives, which agnos now has.

## The loops (agnoshi 2.0.0)

1. `src/run_agnos.cyr` ~:536 (`run`) and ~:650 (`_sh_pipe_reap`): `waitpid` → `-2` → `#44` → retry. Replace with the
   blocking form agnos has had since 1.57.7: `syscall(SYS_WAITPID, 0x100 | pid)` (`WAIT_BLOCK`; `0x1FF` = any child). This is
   Linux/POSIX `waitpid(pid, &st, 0)`.
2. `src/agnsh.cyr` ~:150-169 (stdin is a channel/PTY): the comment says "the kernel NEVER blocks on a channel" — false since
   agnos 1.57.8: `read`#5 with `a4 = 0` on a pipe or channel blocks until data or EOF. The `#44` + 20,000-iteration spin can
   go; one blocking read is enough.
3. `src/agnsh.cyr` ~:120-141 (prompt with background jobs): non-blocking read → `job_reap_poll` → `#44`. This one waits for
   EITHER a key OR a job exit, so a single blocking call does not cover it. Shells elsewhere use `SIGCHLD` + a blocking read
   (bash) or `poll`/`select` on the terminal (zsh). Until agnos offers one of those for this shape, the loop is correct as is;
   it now costs one tick of latency per round instead of spinning.

## Gate

With 1–2 converted: `run` and pipelines reap their children with no `#44` in the loop; a PTY-hosted agnsh idles at 0 %
CPU (agnos `wait-ring3-smoke` P2c shape). agnos's `bench-ring3` `yield_idle` stays one interrupt period.

## Resolution (agnoshi 2.0.1)

1. **`run` and pipelines** (`src/run_agnos.cyr`): one helper, `sh_wait_child(pid)`, is
   `syscall(SYS_WAITPID, 0x100 | pid)` — `WAIT_BLOCK`. `sh_exec_line_sched` and both pipeline reaps call it;
   `_sh_pipe_reap` is gone. A `-2` from the blocking form (the kernel's "this context cannot block", which no ring-3
   process gets on agnos >= 1.57.7) is still read as "not yet" and retried behind a `#44`, so a surprise costs a
   tick per round rather than a hot spin or a misread status.
2. **stdin as a pipe or channel** (`src/agnsh.cyr`): `rl_read_blocking()` is one `read`#5 with `a4 = 0`; the
   20,000-iteration spin and its "the kernel NEVER blocks on a channel" comment are gone. `-2` / `-3` stay "not
   yet", never EOF (the 1.8.7 rule), behind the same guard. The jobs branch's switch to a blocking read, once the
   last job is reaped, now goes through it too — it used to take a bare blocking read straight into the EOF check.
3. **the prompt with background jobs**: unchanged, and now says why in the code — it waits for a key OR a job's
   exit, which no single agnos call covers yet.

Found on the way, same release: agnos 1.57.7 reports a child killed by signal N as `0x100 | N`, and agnsh passed it
through (`run: exit 265`; `agnsh -c` exiting 265 & 0xFF = 9). Every status agnsh reaps on agnos now goes through
`sh_exit_status` — 128 + N, as ADR-008 § 4 and the host launcher already had it.

**Gate.** No `#44` is left in the `run` or pipeline waits except behind the `-2` guard. agnoshi's new
`scripts/agnos-qemu-bench.py` boots a driver (`tests/agnos_hostsh.cyr`) that hosts agnsh on a pipe — the PTY
shape — and measures, a/b in one boot against 2.0.0, on agnos 1.57.9 under KVM, five alternating runs (the
medians of the final build; an earlier boot of the same comparison gave −4.6 % and −1.5 % at `-smp 4`):

| | idle ticks (1 s, state) | `echo hs`, µs per line | `echo hs \| wc`, µs per line |
|---|---|---|---|
| `-smp 1`, 2.0.0 → 2.0.1 | 0 (6) → 0 (6) | 70,018 → 70,168 (+0.2 %) | 134,346 → 134,174 (−0.1 %) |
| `-smp 4`, 2.0.0 → 2.0.1 | 0 (6) → 0 (6) | 78,102 → 73,329 (−6.1 %) | 142,211 → 140,391 (−1.3 %) |

A pipe-hosted agnsh idles at 0 ticks in state 6 (BLOCKED) with either build: since 1.57.8 the kernel blocks the
`a4 = 0` read that agnsh already issued, so the removed spin was no longer reached — this row is the regression
guard, not the win. The win is the reap: at `-smp 1` the old poll's yield handed the CPU straight to the child, so
there is nothing to recover; at `-smp 4` the child runs elsewhere and the old poll parked a tick at a time. Most of
each line is agnos loading kriya's 1.1 MB image. `bench-ring3`'s `yield_idle` is a kernel number that agnsh does
not touch, and was not re-measured here.

