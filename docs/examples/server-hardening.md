# Server Hardening with agnsh

This guide demonstrates using agnsh as the default shell on a hardened
Linux server. The combination of AI-assisted classification + forced
approval workflow + full audit logging is well-suited to server ops.

## Deployment Model

```
[ops user via SSH]
     |
     v
/usr/local/bin/agnsh --strict   (set as login shell)
     |
     v
Every command -> Intent classified -> Approval required -> Audit logged
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

### 4. Configure strict mode by default

Create `/etc/agnoshi/agnsh.conf`:

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

## Hardened Approval Workflow

In `strict` mode, **every** command requires explicit approval:

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

Restricted mode:
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
| Login shell | agnsh in strict mode = every command needs approval |
| Command classification | BLOCKED commands never execute, ADMIN requires approval |
| Input sanitization | No shell injection via crafted NL input |
| Checkpoint | Destructive ops reversible for 100 operations |
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
