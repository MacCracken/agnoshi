# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.90.x  | Yes       |
| < 0.90  | No        |

## Reporting a Vulnerability

If you discover a security vulnerability, please report it responsibly:

1. **Do not** open a public issue
2. Email security details to the maintainers
3. Include steps to reproduce if possible
4. Allow reasonable time for a fix before disclosure

## Security Model

Agnoshi enforces:
- Human approval for destructive operations
- Per-command permission checking (6-tier: Safe → ReadOnly → UserWrite → SystemWrite → Admin → Blocked)
- Sandboxed execution via Landlock + seccomp (dotfile protection for .bashrc, .ssh/, .gitconfig)
- Prompt injection defense: all external content sanitized before LLM prompts (OWASP ASI01/ASI02)
- LLM command validation: generated commands syntax-checked before presentation
- Full audit logging of all operations
- Session isolation between users
