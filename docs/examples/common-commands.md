# Common Commands

Natural language inputs and their translations.

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

`what is the cwd` classifies as QUESTION (tag 42) — handed off to the LLM
if configured, otherwise shows a message.

## Fallthrough

Anything not matched falls to SHELL_COMMAND (tag 15) — runs as a raw shell
command with classification via `analyze_command_permission`.

## Try It

```bash
for cmd in "show me all files" "git status" "install vim" "remove old.log"; do
    echo "=== $cmd ==="
    agnsh -c "$cmd"
done
```
