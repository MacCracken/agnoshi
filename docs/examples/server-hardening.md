# Server Hardening with agnsh

> # ⛔ DO NOT DEPLOY THIS AS WRITTEN — IT DESCRIBES A SHELL THAT DOES NOT EXIST YET
>
> This guide was written against an intended design, not the shipped binary.
> Verified against **1.9.9**, the following load-bearing claims below are FALSE:
>
> | The guide says | Reality |
> |---|---|
> | `agnsh --strict` as a login shell | **`--strict` is not a flag.** It prints usage and exits **0** — silently ignored. The flag is `--mode strict`. |
> | strict mode = every command needs approval | **Nothing prompts and nothing is blocked.** A HIGH-risk command prints `Approval required` and continues. `ApprovalManager` is not in the binary. |
> | `/etc/agnoshi/agnsh.conf` configures it | **No config file is read at all** — no path, no parser, nothing in `src/`. |
> | Checkpoint makes destructive ops reversible | **No checkpointing, no `undo`.** `checkpoint.cyr` is not compiled. |
> | Restricted mode blocks privilege escalation | **There is no privilege escalation to block** — nothing invokes `sudo`. |
>
> **What IS true and useful today**: permission classification with basename
> extraction, argument/path sanitization, and a complete audit trail — every
> classification, every launch, and every refusal, JSON-escaped and UTF-8
> validated. That is a real auditing posture. It is **not** an enforcement one.
>
> Treat this document as a design target for the roadmap's Bucket 1 exec +
> approval slices, not as deployment instructions.

This guide demonstrates using agnsh as the default shell on a hardened
Linux server.

## Deployment Model

```
[ops user via SSH]
     |
     v
/usr/local/bin/agnsh --strict   (set as login shell)
     |
     v
Every command -> Intent classified -> risk REPORTED -> Audit logged
                                      (approval is not enforced; see the banner)
```

## Install as Login Shell

### 1. Install agnsh system-wide

```bash
sudo sh scripts/install.sh
```

### 2. Add to /etc/shells

```bash
echo /usr/local/bin/agnsh | sudo tee -a /etc/shells
```

### 3. Change login shell for the ops user

```bash
sudo chsh -s /usr/local/bin/agnsh opsuser
```

### 4. Configure strict mode by default — ⛔ NOT IMPLEMENTED

agnsh reads **no configuration file**. There is no `/etc/agnoshi/`, no parser,
and no path lookup anywhere in `src/`. Mode is selected per-invocation with
`--mode <name>` or interactively with `mode <name>`. The block below is a design
sketch for a future `.agnshrc`-style config (roadmap, Bucket 2):

```toml
default_mode = "strict"
auto_approve_low = false
approval_timeout = 300
audit_log = "/var/log/agnoshi/audit.log"
history_size = 10000
```

### 5. Set up centralized audit

```bash
# Ensure audit dir exists and is writable only by authorized users
sudo mkdir -p /var/log/agnoshi
sudo chown opsuser:adm /var/log/agnoshi
sudo chmod 750 /var/log/agnoshi

# Rotate daily
cat > /etc/logrotate.d/agnoshi <<EOF
/var/log/agnoshi/audit.log {
    daily
    rotate 90
    compress
    delaycompress
    missingok
    notifempty
    create 0640 opsuser adm
}
EOF
```

## Hardened Approval Workflow — ⚠ ASPIRATIONAL, NOT CURRENT

⛔ The transcript below is a design sketch. In 1.9.8 `strict` mode adds a y/n
confirmation **only before a program launch** (`run`, or an AGNOS bareword). A
natural-language translation is reported and audited, never prompted, because
the NL path does not execute. The intended behaviour:

```
$ ssh opsuser@prod-server
> restart nginx

[HIGH] Approval required: systemctl restart nginx
  [a]pprove  [d]eny  [m]odify: a

(command runs)
```

Denied or timed-out commands are logged with `approved: 0` for later
review.

## Audit Review

Daily review with a shell helper:

```bash
#!/bin/sh
# ~/bin/agnsh-review
YESTERDAY=$(date -d yesterday +%Y-%m-%d)
echo "=== Actions on $YESTERDAY ==="
jq -c "select(.timestamp | startswith(\"$YESTERDAY\"))" /var/log/agnoshi/audit.log.1 | \
    jq -r '[.timestamp, .user, (.approved|tostring), .action, .input] | @tsv'
```

Weekly anomaly check:

```bash
# Any BLOCKED attempts?
jq -c 'select(.result | contains("blocked"))' /var/log/agnoshi/audit.log
# Any denials?
jq -c 'select(.approved == 0)' /var/log/agnoshi/audit.log
```

## Restricted Mode for Untrusted Users

For users who should not have privilege escalation at all:

```bash
# Force restricted mode via login wrapper
cat > /usr/local/bin/agnsh-restricted <<'EOF'
#!/bin/sh
exec /usr/local/bin/agnsh --restricted --strict "$@"
EOF
sudo chmod 755 /usr/local/bin/agnsh-restricted
sudo chsh -s /usr/local/bin/agnsh-restricted contractor
```

⚠ Restricted mode is **not implemented** — `SecurityContext` lives in
`src/security.cyr`, which is not in the binary's include graph. Intended:
- Forces `restricted = 1` in `SecurityContext`
- Blocks all sudo/privilege escalation
- Runs same classification, but ADMIN-level ops always deny

## Integration with auditd

For double-audit (agnsh log + kernel audit), configure auditd to watch
agnsh's process calls:

```
# /etc/audit/rules.d/agnoshi.rules
-a always,exit -F arch=b64 -S execve -F path=/usr/local/bin/agnsh -k agnsh_exec
```

Correlate agnsh's JSON log with auditd's records for full forensic view.

## Layered Defense Recap

| Layer | Protection |
|-------|-----------|
| SSH | Key-based auth, fail2ban, port forward restrictions |
| Login shell | ⛔ **Not an enforcement boundary today** — risk is reported, not enforced |
| Command classification | ✅ Accurate and useful: basename extraction, six tiers. BLOCKED is *reported*; the NL path executes nothing either way |
| Input sanitization | No shell injection via crafted NL input |
| ~~Checkpoint~~ | ⛔ **Not shipped** — no checkpointing, no `undo`, no rollback |
| Audit | JSON log of every action, integrity-safe escaping |
| auditd | Kernel-level double-audit |
| Log shipping | Centralize to SIEM for offline review |

## Failure Modes and Recovery

**If agnsh is broken or misclassifies:**

Users with access can break glass by calling a known-safe shell directly:
```bash
/bin/sh
```

Keep `/etc/shells` containing at least one alternative so `chsh` can
recover if agnsh fails to start.

**If audit log rotation fails:**

Agnsh continues to write — it doesn't refuse-to-operate on log errors.
Monitor disk space for `/var/log/agnoshi/`.

**If a user authorizes a bad command:**

Audit log records the approval. Checkpoint lets you `undo` if still in
session. For inter-session recovery, restore from backup.
