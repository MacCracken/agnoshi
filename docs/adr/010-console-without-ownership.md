# ADR-010: A Session Started on /dev/null Runs on the Console, Owned by No One (2.1.2)

**Status:** Accepted (2026-11-02, for 2.1.2)

## Context

kybernet starts every service with `/dev/null` on fds 0, 1 and 2 (its standing rule 20). argonaut's
`default_services` starts agnoshi in BOOT_MINIMAL and BOOT_DESKTOP with no arguments, and since
argonaut 1.20.0 / kybernet 1.12.0 runs it as the interactive user, 1000:1000. An interactive agnsh
reads its lines from fd 0, so as a default service it read EOF at its first prompt and exited 0.

The roadmap's P1 item (raised 2026-11-02 by kybernet 1.12.0) left one question open: does agnoshi
open its own terminal, or do argonaut and kybernet give a service one? kybernet's roadmap gated its
side on "a design, in argonaut, for a service that owns a terminal".

## Decision

**Nothing owns the terminal.** That was the user's call (2026-11-02). Every service, agnoshi included,
is a process; no service is designed around owning a terminal. agnoshi is the follow-up:

- When fd 0 **and** fd 1 are both `/dev/null` (`src/console.cyr`'s `fd_is_file`), an interactive
  host agnsh opens `/dev/console` with `O_RDWR | O_NOCTTY`. It `dup2`s the console onto fds 0, 1
  and 2 before its banner, and the programs it runs inherit it as their stdio.
- **No controlling terminal.** agnsh makes no `setsid` or `TIOCSCTTY` call, and the open passes
  `O_NOCTTY`. argonaut makes each service a session leader, and a session leader that opens a tty
  without `O_NOCTTY` takes it as its controlling terminal.
- **If the console cannot be opened**, agnsh writes the errno to stderr and exits 1, instead of
  exiting 0 on EOF. fd 2 may be `/dev/null` too, so the exit status in the supervisor's log is the
  one sure signal.
- **Linux host only.** On agnos, agnsh's stdin is whatever its launcher gives it, and nothing here
  changes.

## Consequences

- **+** kybernet and argonaut stay as they are: rule 20 holds, and no service needs a tty handed to it.
- **+** The trigger is narrow. A pipe, a file or a terminal on fd 0 is someone's input, and so is
  `/dev/null` on stdin with stdout kept, as in `agnsh < /dev/null > out`. All of these behave as before.
- **−** With no controlling terminal, ^C, ^Z and a hangup on the console send agnsh and its children
  no signal. On the host, agnsh has no job control to lose, but a foreground program that does not
  exit cannot be interrupted from the console.
- **−** **The console must be openable by the uid agnsh runs as.** On a kybernet board, devtmpfs
  makes `/dev/console` `root:root 0600` (there is no udev).
  - kybernet's agnos-init grants it to group 1000, mode 0660 (kybernet, unreleased after 1.12.0),
    in the same way it grants `/dev/fb0` and `/dev/input/event*`.
  - argonaut 1.20.0 runs agnos-init in BOOT_DESKTOP only, so on a BOOT_MINIMAL board the default
    agnoshi still exits 1 with `errno 13`.
  - argonaut's unreleased tree adds agnos-init to BOOT_MINIMAL, with the minimal agnoshi depending
    on it. The roadmap's "done when" — a prompt on the console as uid 1000 on a minimal board — waits
    for that argonaut tag and kybernet's pin of it.

## Alternatives considered

- **The supervisor gives the service a tty, owned by it** (kybernet's gate wording). Rejected:
  nothing owns the terminal.
- **agnsh takes the console as its controlling terminal** (`setsid` + `TIOCSCTTY`). Rejected for the
  same reason. It would also make agnsh's session the console's foreground, which it is not.
- **Trigger on "fd 0 is not a tty".** Rejected: every pipe and file on stdin would lose its input to
  the console.
- **Stay alive with no prompt** when there is no input. Rejected: the item asks for a prompt on the
  console, and an idle shell nobody can reach is a process that only looks healthy.

## References

- `src/console.cyr`; `tests/test_core.tcyr` (the console checks); `scripts/smoke-test.sh`
  (exit 1 and its reason, and the `< /dev/null` control).
- kybernet 1.12.0 CHANGELOG ("agnoshi has no terminal as a service"); argonaut 1.20.0 CHANGELOG
  (service identity: agnoshi 1000:1000); `docs/development/roadmap.md` P1.
