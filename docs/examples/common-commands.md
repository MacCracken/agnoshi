# Common Commands

Natural language inputs and their translations.

> **These are the NL parser's answers, and CI holds the parser to every row**
> (`tests/test_parse_corpus.tcyr` reads this table). A line reaches the parser only when it is
> not a shell line ([ADR-007](../adr/007-input-classification.md)): on AGNOS, a line whose first
> word is a program in `/bin` runs that program instead — `find files named foo` runs kriya's
> `find` — and a line with `|` or `>` is a pipeline or redirect. On the Linux host every line
> reaches the parser today; 1.10.0 gives the host the same `PATH` lookup.

## Filesystem

| You say | Classified as | Translates to |
|---------|---------------|---------------|
| `show me all files` | LIST_FILES | `ls` |
| `list files in /tmp` | LIST_FILES | `ls /tmp` |
| `show all detail files` | LIST_FILES | `ls -la` |
| `find files named foo` | FIND_FILES | `find . -name foo` |
| `find files named *.log in /var` | FIND_FILES | `find /var -name '*.log'` |
| `search for error in /var/log` | SEARCH_CONTENT | `grep -r error /var/log` |
| `read /etc/hostname` | SHOW_FILE | `cat /etc/hostname` |
| `go to /home` | CHANGE_DIR | `cd /home` |
| `create directory /tmp/foo` | CREATE_DIR | `mkdir -p /tmp/foo` |
| `copy a.txt to b.txt` | COPY | `cp a.txt b.txt` |
| `move a.txt to b.txt` | MOVE | `mv a.txt b.txt` |
| `remove file.txt` | REMOVE | `rm file.txt` (requires approval) |

## System

| You say | Classified as | Translates to |
|---------|---------------|---------------|
| `show running processes` | SHOW_PROCESSES | `ps aux` |
| `kill process 1234` | KILL_PROCESS | `kill 1234` (requires approval) |
| `show system info` | SYSTEM_INFO | `uname -a` |
| `show network info` | NETWORK_INFO | `ip addr show` |
| `show disk usage` | DISK_USAGE | `df -h` |
| `show memory usage` | MEMORY_INFO | `free -h` |
| `show free memory` | MEMORY_INFO | `free -h` |
| `ram usage` | MEMORY_INFO | `free -h` |
| `install vim` | INSTALL_PACKAGE | `apt install -y vim` (requires approval) |
| `start service nginx` | SERVICE_CONTROL | `systemctl start nginx` |
| `stop service apache2` | SERVICE_CONTROL | `systemctl stop apache2` |
| `restart service sshd` | SERVICE_CONTROL | `systemctl restart sshd` |

## Git

| You say | Classified as | Translates to |
|---------|---------------|---------------|
| `git status` | GIT_STATUS | `git status` |
| `git log` | GIT_LOG | `git log --oneline -20` |
| `git commit -m fix bug` | GIT_COMMIT | `git commit -m 'fix bug'` |
| `git commit all` | GIT_COMMIT | `git commit -a` |
| `git diff` | GIT_DIFF | `git diff` |
| `git diff staged` | GIT_DIFF | `git diff --staged` |
| `git branch new-feature` | GIT_BRANCH | `git branch new-feature` |
| `git checkout main` | GIT_CHECKOUT | `git checkout main` |
| `git merge feature` | GIT_MERGE | `git merge feature` |
| `git push` | GIT_PUSH | `git push` |
| `git pull` | GIT_PULL | `git pull` |
| `git stash` | GIT_STASH | `git stash push` |
| `git stash pop` | GIT_STASH | `git stash pop` |

## Firewall (ufw)

| You say | Classified as | Translates to |
|---------|---------------|---------------|
| `firewall allow 8080` | FIREWALL_ALLOW | `ufw allow 8080` |
| `firewall deny 23` | FIREWALL_DENY | `ufw deny 23` |
| `firewall list` | FIREWALL_LIST | `ufw status numbered` |
| `firewall status` | FIREWALL_STATUS | `ufw status` |
| `firewall enable` | FIREWALL_ENABLE | `ufw enable` |
| `firewall disable` | FIREWALL_DISABLE | `ufw disable` |
| `firewall delete 3` | FIREWALL_DELETE | `ufw delete 3` |

## User/Group

| You say | Classified as | Translates to |
|---------|---------------|---------------|
| `add user alice` | USER_ADD | `useradd alice` |
| `delete user bob` | USER_DELETE | `userdel bob` |
| `change password alice` | PASSWD | `passwd alice` |
| `add group devs` | GROUP_ADD | `groupadd devs` |
| `groups` | GROUP_LIST | `groups` |

## Pipelines

`show files | grep error` classifies as PIPELINE (tag 41). The parts are
split and stored in `intent.vec1`.

## Questions

`what is the cwd` classifies as QUESTION (tag 42). ⚠ **There is no LLM
integration in the binary** — no `src/llm.cyr`, and no network stack compiled in
— so this always prints a placeholder and logs `result: needs_llm`. Nothing is
"handed off"; there is nothing to hand it to.

## Fallthrough

Anything not matched falls to SHELL_COMMAND (tag 15) and is classified by
`analyze_command_permission` — which extracts the basename first, so
`/usr/bin/dd` is still BLOCKED.

⚠ **It is classified, not run.** The natural-language path does not execute
anything: it reports the translation and its risk, and writes an audit record.
The only execution paths are `run /abs/path` (any target) and, on AGNOS,
bareword `/bin/<name>`, `cmd1 | cmd2`, `cmd > file` and `prog &`.

## Try It

```bash
for cmd in "show me all files" "git status" "install vim" "remove old.log"; do
    echo "=== $cmd ==="
    agnsh -c "$cmd"
done
```
