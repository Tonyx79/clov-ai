#!/usr/bin/env bash
#
# CLOV Smoke Tests — Aristote Project (Vite + React + TS + ESLint)
# Tests CLOV commands in a real JS/TS project context.
# Usage: bash scripts/test-aristote.sh
#
set -euo pipefail

ARISTOTE="/Users/florianbruniaux/Sites/MethodeAristote/aristote-school-boost"

PASS=0
FAIL=0
SKIP=0
FAILURES=()

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

assert_ok() {
    local name="$1"; shift
    local output
    if output=$("$@" 2>&1); then
        PASS=$((PASS + 1))
        printf "  ${GREEN}PASS${NC}  %s\n" "$name"
    else
        FAIL=$((FAIL + 1))
        FAILURES+=("$name")
        printf "  ${RED}FAIL${NC}  %s\n" "$name"
        printf "        cmd: %s\n" "$*"
        printf "        out: %s\n" "$(echo "$output" | head -3)"
    fi
}

assert_contains() {
    local name="$1"; local needle="$2"; shift 2
    local output
    if output=$("$@" 2>&1) && echo "$output" | grep -q "$needle"; then
        PASS=$((PASS + 1))
        printf "  ${GREEN}PASS${NC}  %s\n" "$name"
    else
        FAIL=$((FAIL + 1))
        FAILURES+=("$name")
        printf "  ${RED}FAIL${NC}  %s\n" "$name"
        printf "        expected: '%s'\n" "$needle"
        printf "        got: %s\n" "$(echo "$output" | head -3)"
    fi
}

# Allow non-zero exit but check output
assert_output() {
    local name="$1"; local needle="$2"; shift 2
    local output
    output=$("$@" 2>&1) || true
    if echo "$output" | grep -q "$needle"; then
        PASS=$((PASS + 1))
        printf "  ${GREEN}PASS${NC}  %s\n" "$name"
    else
        FAIL=$((FAIL + 1))
        FAILURES+=("$name")
        printf "  ${RED}FAIL${NC}  %s\n" "$name"
        printf "        expected: '%s'\n" "$needle"
        printf "        got: %s\n" "$(echo "$output" | head -3)"
    fi
}

skip_test() {
    local name="$1"; local reason="$2"
    SKIP=$((SKIP + 1))
    printf "  ${YELLOW}SKIP${NC}  %s (%s)\n" "$name" "$reason"
}

section() {
    printf "\n${BOLD}${CYAN}── %s ──${NC}\n" "$1"
}

# ── Preamble ─────────────────────────────────────────

CLOV=$(command -v clov || echo "")
if [[ -z "$CLOV" ]]; then
    echo "clov not found in PATH. Run: cargo install --path ."
    exit 1
fi

if [[ ! -d "$ARISTOTE" ]]; then
    echo "Aristote project not found at $ARISTOTE"
    exit 1
fi

printf "${BOLD}CLOV Smoke Tests — Aristote Project${NC}\n"
printf "Binary: %s (%s)\n" "$CLOV" "$(clov --version)"
printf "Project: %s\n" "$ARISTOTE"
printf "Date: %s\n\n" "$(date '+%Y-%m-%d %H:%M')"

# ── 1. File exploration ──────────────────────────────

section "Ls & Find"

assert_ok       "clov ls project root"           clov ls "$ARISTOTE"
assert_ok       "clov ls src/"                   clov ls "$ARISTOTE/src"
assert_ok       "clov ls --depth 3"              clov ls --depth 3 "$ARISTOTE/src"
assert_contains "clov ls shows components/"      "components" clov ls "$ARISTOTE/src"
assert_ok       "clov find *.tsx"                clov find "*.tsx" "$ARISTOTE/src"
assert_ok       "clov find *.ts"                 clov find "*.ts" "$ARISTOTE/src"
assert_contains "clov find finds App.tsx"        "App.tsx" clov find "*.tsx" "$ARISTOTE/src"

# ── 2. Read ──────────────────────────────────────────

section "Read"

assert_ok       "clov read tsconfig.json"        clov read "$ARISTOTE/tsconfig.json"
assert_ok       "clov read package.json"         clov read "$ARISTOTE/package.json"
assert_ok       "clov read App.tsx"              clov read "$ARISTOTE/src/App.tsx"
assert_ok       "clov read --level aggressive"   clov read --level aggressive "$ARISTOTE/src/App.tsx"
assert_ok       "clov read --max-lines 10"       clov read --max-lines 10 "$ARISTOTE/src/App.tsx"

# ── 3. Grep ──────────────────────────────────────────

section "Grep"

assert_ok       "clov grep import"               clov grep "import" "$ARISTOTE/src"
assert_ok       "clov grep with type filter"     clov grep "useState" "$ARISTOTE/src" -t tsx
assert_contains "clov grep finds components"     "import" clov grep "import" "$ARISTOTE/src"

# ── 4. Git ───────────────────────────────────────────

section "Git (in Aristote repo)"

# clov git doesn't support -C, use git -C via subshell
assert_ok       "clov git status"                bash -c "cd $ARISTOTE && clov git status"
assert_ok       "clov git log"                   bash -c "cd $ARISTOTE && clov git log"
assert_ok       "clov git branch"                bash -c "cd $ARISTOTE && clov git branch"

# ── 5. Deps ──────────────────────────────────────────

section "Deps"

assert_ok       "clov deps"                      clov deps "$ARISTOTE"
assert_contains "clov deps shows package.json"   "package.json" clov deps "$ARISTOTE"

# ── 6. Json ──────────────────────────────────────────

section "Json"

assert_ok       "clov json tsconfig"             clov json "$ARISTOTE/tsconfig.json"
assert_ok       "clov json package.json"         clov json "$ARISTOTE/package.json"

# ── 7. Env ───────────────────────────────────────────

section "Env"

assert_ok       "clov env"                       clov env
assert_ok       "clov env --filter NODE"         clov env --filter NODE

# ── 8. Tsc ───────────────────────────────────────────

section "TypeScript (tsc)"

if command -v npx >/dev/null 2>&1 && [[ -d "$ARISTOTE/node_modules" ]]; then
    assert_output "clov tsc (in aristote)" "error\|✅\|TS" clov tsc --project "$ARISTOTE"
else
    skip_test "clov tsc" "node_modules not installed"
fi

# ── 9. ESLint ────────────────────────────────────────

section "ESLint (lint)"

if command -v npx >/dev/null 2>&1 && [[ -d "$ARISTOTE/node_modules" ]]; then
    assert_output "clov lint (in aristote)" "error\|warning\|✅\|violations\|clean" clov lint --project "$ARISTOTE"
else
    skip_test "clov lint" "node_modules not installed"
fi

# ── 10. Build (Vite) ─────────────────────────────────

section "Build (Vite via clov next)"

if [[ -d "$ARISTOTE/node_modules" ]]; then
    # Aristote uses Vite, not Next — but clov next wraps the build script
    # Test with a timeout since builds can be slow
    skip_test "clov next build" "Vite project, not Next.js — use npm run build directly"
else
    skip_test "clov next build" "node_modules not installed"
fi

# ── 11. Diff ─────────────────────────────────────────

section "Diff"

# Diff two config files that exist in the project
assert_ok       "clov diff tsconfigs"            clov diff "$ARISTOTE/tsconfig.json" "$ARISTOTE/tsconfig.app.json"

# ── 12. Summary & Err ────────────────────────────────

section "Summary & Err"

assert_ok       "clov summary ls"                clov summary ls "$ARISTOTE/src"
assert_ok       "clov err ls"                    clov err ls "$ARISTOTE/src"

# ── 13. Gain ─────────────────────────────────────────

section "Gain (after above commands)"

assert_ok       "clov gain"                      clov gain
assert_ok       "clov gain --history"            clov gain --history

# ══════════════════════════════════════════════════════
# Report
# ══════════════════════════════════════════════════════

printf "\n${BOLD}══════════════════════════════════════${NC}\n"
printf "${BOLD}Results: ${GREEN}%d passed${NC}, ${RED}%d failed${NC}, ${YELLOW}%d skipped${NC}\n" "$PASS" "$FAIL" "$SKIP"

if [[ ${#FAILURES[@]} -gt 0 ]]; then
    printf "\n${RED}Failures:${NC}\n"
    for f in "${FAILURES[@]}"; do
        printf "  - %s\n" "$f"
    done
fi

printf "${BOLD}══════════════════════════════════════${NC}\n"

exit "$FAIL"
