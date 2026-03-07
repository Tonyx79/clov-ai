# CLOV - Clov Token Omitter

**Usage**: Token-optimized CLI proxy (60-90% savings on dev operations)

## Meta Commands (always use clov directly)

```bash
clov gain              # Show token savings analytics
clov gain --history    # Show command usage history with savings
clov cc-savings        # Claude Code spend vs CLOV savings breakdown
clov discover          # Analyze Claude Code history for missed opportunities
clov proxy <cmd>       # Execute raw command without filtering (for debugging)
```

## Installation Verification

```bash
clov --version         # Should show: clov 0.26.2+
clov gain              # Should work (not "command not found")
which clov             # Verify correct binary
```

## Hook-Based Usage

All other commands are automatically rewritten by the Claude Code hook.
Example: `git status` → `clov git status` (transparent, 0 tokens overhead)
Example: `gt log` → `clov gt log` (Graphite stacked PRs supported)

Refer to CLAUDE.md for full command reference.
