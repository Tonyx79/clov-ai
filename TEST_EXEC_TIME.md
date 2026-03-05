# Testing Execution Time Tracking

## Quick Test

```bash
# 1. Install latest version
cargo install --path .

# 2. Run a few commands to populate data
clov git status
clov ls .
clov grep "tracking" src/

# 3. Check gain stats (should show execution times)
clov gain

# Expected output:
# Total exec time:   XX.Xs (avg XXms)
# By Command table should show Time column
```

## Detailed Test Scenarios

### 1. Basic Time Tracking
```bash
# Run commands with different execution times
clov git log -10          # Fast (~10ms)
clov cargo test           # Slow (~300ms)
clov vitest run           # Very slow (seconds)

# Verify times are recorded
clov gain
# Should show different avg times per command
```

### 2. Daily Breakdown
```bash
clov gain --daily

# Expected:
# Date column + Time column showing avg time per day
# Today should have non-zero times
# Historical data shows 0ms (no time recorded)
```

### 3. Export Formats

**JSON Export:**
```bash
clov gain --daily --format json | jq '.summary'

# Should include:
# "total_time_ms": 12345,
# "avg_time_ms": 67
```

**CSV Export:**
```bash
clov gain --daily --format csv

# Headers should include:
# date,commands,input_tokens,...,total_time_ms,avg_time_ms
```

### 4. Multiple Commands
```bash
# Run 10 commands and measure total time
for i in {1..10}; do clov git status; done

clov gain
# Total exec time should be ~10-50ms (10 × 1-5ms)
```

## Verification Checklist

- [ ] `clov gain` shows "Total exec time: X (avg Yms)"
- [ ] By Command table has "Time" column
- [ ] `clov gain --daily` shows time per day
- [ ] JSON export includes `total_time_ms` and `avg_time_ms`
- [ ] CSV export has time columns
- [ ] New commands show realistic times (not 0ms)
- [ ] Historical data preserved (old entries show 0ms)

## Database Schema Verification

```bash
# Check SQLite schema includes exec_time_ms
sqlite3 ~/.local/share/clov/history.db "PRAGMA table_info(commands);"

# Should show:
# ...
# 7|exec_time_ms|INTEGER|0|0|0
```

## Performance Impact

The timer adds negligible overhead:
- `Instant::now()` → ~10-50ns
- `elapsed()` → ~10-50ns
- SQLite insert with extra column → ~1-5µs

Total overhead: **< 0.1ms per command**
