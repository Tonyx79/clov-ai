# CLOV - Clov Token Omitter

**Usage**: Token-optimized CLI proxy (60-90% savings on dev operations)

## Meta Commands (always use clov directly)

```bash
clov gain              # Show token savings analytics
clov gain --history    # Show command usage history with savings
clov discover          # Analyze Claude Code history for missed opportunities
clov proxy <cmd>       # Execute raw command without filtering (for debugging)
```

## Installation Verification

```bash
clov --version         # Should show: clov X.Y.Z
clov gain              # Should work (not "command not found")
which clov             # Verify correct binary
```

⚠️ **Name collision**: If `clov gain` fails, you may have reachingforthejack/clov (Rust Type Kit) installed instead.

## Hook-Based Usage

All other commands are automatically rewritten by the Claude Code hook.
Example: `git status` → `clov git status` (transparent, 0 tokens overhead)

Refer to CLAUDE.md for full command reference.
