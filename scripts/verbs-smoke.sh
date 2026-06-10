#!/bin/sh
# verbs-smoke.sh -- RETIRED at agnoshi 1.5.0.
#
# This suite tested the in-process FS verbs (ls/cat/cp/mv/rm/mkdir/rmdir/touch/
# echo/wc/find/grep) that agnsh carried as builtins from 1.4.2. Those verbs were
# a stopgap "until ring-3 execwait lands" — execwait shipped at agnos 1.43.0, so
# 1.5.0 removed them: agnsh now DELEGATES file verbs to the staged /bin/kriya
# dispatcher and cat to /bin/owl. There is nothing in-process left to assert.
#
# The delegation contract is now validated on the agnos target (not the host):
#   agnos/scripts/agnsh-verb-test.py   -- bareword cp/ls/echo -> /bin/kriya and
#                                         owl -p over the live ext2 root (QEMU
#                                         headless, HMP sendkey).
# Host CLI/intent coverage stays in scripts/smoke-test.sh (59/0).
echo "verbs-smoke.sh is retired at 1.5.0 — file verbs are delegated to kriya/owl."
echo "Delegation is validated on agnos: agnos/scripts/agnsh-verb-test.py"
echo "Host CLI/intent coverage: scripts/smoke-test.sh"
exit 0
