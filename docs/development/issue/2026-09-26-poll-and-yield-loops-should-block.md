# agnsh's poll + sched_yield loops should use agnos's blocking waits

- **Filed**: 2026-09-26
- **Repo**: agnoshi
- **agnoshi**: 2.0.0
- **Found by**: agnos 1.57.9 (the `sched_yield`#44 ruling)
- **Status**: OPEN — no agnos change required; this is agnsh catching up with kernel features that now exist

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
