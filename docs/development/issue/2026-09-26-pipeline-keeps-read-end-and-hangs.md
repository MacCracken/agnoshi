# agnsh pipeline hangs when the consumer exits early — the shell keeps the pipe's read end

- **Filed**: 2026-09-26
- **Repo**: agnoshi
- **agnoshi**: 2.0.0
- **Found by**: agnos 1.57.9 (end review of its blocking pipe writes; `scripts/smoke/pipeline-smoke.sh` in agnos)
- **Status**: OPEN — fix is below (tested against agnos 1.57.9 from a scratch copy of agnoshi; not applied here)

## What happens

On agnos ≥ 1.57.9, a pipeline whose consumer exits before reading everything never returns to the prompt:

- `grep . /etc/ssl/cert.pem | echo x`
- any pipeline whose stage 2 fails to spawn

agnos 1.57.9 made pipe writes block when the pipe is full and a reader is still open (as Linux `fs/pipe.c`
`pipe_write` and POSIX `write()` do). agnsh keeps its own copy of the pipe's read end open while it reaps both stages,
and stage 1 is spawned without `SPAWN_F_CLEANFD`, so it inherits that read end too. The producer therefore always sees
a live reader, never gets EPIPE, and blocks forever on the full pipe while agnsh waits to reap it.

This is not new behaviour caused by agnos: with 1.57.9's blocking write reverted, the same smoke is still red, because
kriya's `k_write` stall loop retries a stuck write for about 200 s. Blocking just makes it permanent.

## Prior art

bash `execute_pipeline` (execute_cmd.c) closes each pipe end in the parent as soon as the stage that uses it exists
(`fds_to_close`), so a producer whose consumer has exited sees EPIPE/SIGPIPE instead of blocking. Every POSIX shell does
the same; a shell must never hold a pipe end it does not read or write.

## Fix (tested)

In `src/run_agnos.cyr`:

1. Spawn pipeline stages with `SPAWN_F_CLEANFD` (0x20000) so a stage gets fds 0/1/2 after the armed redirect and nothing
   else — it must not inherit the shell's copy of the other pipe end.
2. Close the shell's read end (`rfd`) as soon as stage 2 exists, and on both failure paths BEFORE reaping stage 1.

```diff
--- a/src/run_agnos.cyr
+++ b/src/run_agnos.cyr
@@ -639,8 +639,11 @@
     while (load8(cmd + len) != 0) { len = len + 1; }
     if (len > 127) { return 0 - 1; }               # #43 path cap
     var blen = sh_build_env_blob();
-    if (blen > 0) { return syscall(43, cmd, len, &sh_env_blob, blen); }
-    return syscall(43, cmd, len, 0, 0);
+    # SPAWN_F_CLEANFD (0x20000): the stage gets 0/1/2 after the armed redirect and nothing else -- it must NOT
+    # inherit the shell's copy of the other pipe end (bash execute_pipeline's fds_to_close: a producer holding the
+    # read end never sees EPIPE and blocks forever on a full pipe once the consumer is gone).
+    if (blen > 0) { return syscall(43, cmd, len | 0x20000, &sh_env_blob, blen); }
+    return syscall(43, cmd, len | 0x20000, 0, 0);
 }
@@ -770,22 +773,25 @@
     if (sh_exec_redirect(0, rfd) < 0) {
         eprintln_cstr("run: failed to arm pipeline stage 2 redirect");
+        sys_close(rfd);                             # before the reap: stage 1 must see EPIPE, not a live reader
         _sh_pipe_reap(pid1);
-        sys_close(rfd);
         audit_exec_ctx(line, "error", AUDIT_NO_EXIT);
         return 1;
     }
     var pid2 = _sh_pipe_spawn(cmd2);
     if (pid2 < 0) {
         eprintln_cstr("run: failed to launch pipeline stage 2");
+        sys_close(rfd);                             # before the reap: stage 1 must see EPIPE, not a live reader
         _sh_pipe_reap(pid1);
-        sys_close(rfd);
         audit_exec_ctx(line, "error", pid2);
         return 1;
     }
+    # Drop the SHELL's read end now that stage 2 holds its own (bash closes each pipe end in the parent once the
+    # stage that uses it exists).
+    sys_close(rfd);
     var rc2 = _sh_pipe_reap(pid2);
     _sh_pipe_reap(pid1);                            # producer normally exits first; reap it regardless
-    sys_close(rfd);
```

## Gate

agnos `scripts/smoke/pipeline-smoke.sh` (sweep row "1.57.9 agnsh pipeline with an early-exiting consumer returns to the
prompt"): red with agnsh 2.0.0, green on all four boots (-smp 1 and 4) with the patched agnsh. It stays red in agnos's
sweep until this lands.

## Related (not agnoshi)

kriya `k_write` (src/lib/sys.cyr) bounds a stalled write by 20,000 × `sched_yield`#44; since #44 parks up to one timer
tick when nothing else is ready, that is ~200 s. It should be time-based.
