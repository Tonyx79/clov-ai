---
model: haiku
description: Test CLOV command routing without execution (dry-run) - verifies which commands have filters
---

# /test-routing

Vérifie le routing de commandes CLOV sans exécution (dry-run). Utile pour tester si une commande a un filtre disponible avant de l'exécuter.

## Usage

```
/test-routing <command> [args...]
```

## Exemples

```bash
/test-routing git status
# Output: ✅ CLOV filter available: git status → clov git status

/test-routing npm install
# Output: ⚠️  No CLOV filter, would execute raw: npm install

/test-routing cargo test
# Output: ✅ CLOV filter available: cargo test → clov cargo test
```

## Quand utiliser

- **Avant d'exécuter une commande**: Vérifier si CLOV a un filtre
- **Debugging hook integration**: Tester le command routing sans side-effects
- **Documentation**: Identifier quelles commandes CLOV supporte
- **Testing**: Valider routing logic sans exécuter de vraies commandes

## Implémentation

### Option 1: Check CLOV Help Output

```bash
COMMAND="$1"
shift
ARGS="$@"

# Check if CLOV has subcommand for this command
if clov --help | grep -E "^  $COMMAND" >/dev/null 2>&1; then
    echo "✅ CLOV filter available: $COMMAND $ARGS → clov $COMMAND $ARGS"
    echo ""
    echo "Expected behavior:"
    echo "  - Command will be filtered through CLOV"
    echo "  - Output condensed for token efficiency"
    echo "  - Exit code preserved from original command"
else
    echo "⚠️  No CLOV filter available, would execute raw: $COMMAND $ARGS"
    echo ""
    echo "Expected behavior:"
    echo "  - Command executed without CLOV filtering"
    echo "  - Full command output (no token savings)"
    echo "  - Original command behavior unchanged"
fi
```

### Option 2: Check CLOV Source Code

```bash
COMMAND="$1"
shift
ARGS="$@"

# List of supported CLOV commands (from src/main.rs)
CLOV_COMMANDS=(
    "git"
    "grep"
    "ls"
    "read"
    "err"
    "test"
    "log"
    "json"
    "lint"
    "tsc"
    "next"
    "prettier"
    "playwright"
    "prisma"
    "gh"
    "vitest"
    "pnpm"
    "ruff"
    "pytest"
    "pip"
    "go"
    "golangci-lint"
    "docker"
    "cargo"
    "smart"
    "summary"
    "diff"
    "env"
    "discover"
    "gain"
    "proxy"
)

# Check if command in supported list
if [[ " ${CLOV_COMMANDS[@]} " =~ " ${COMMAND} " ]]; then
    echo "✅ CLOV filter available: $COMMAND $ARGS → clov $COMMAND $ARGS"
    echo ""

    # Show filter details if available
    case "$COMMAND" in
        git)
            echo "Filter: git operations (status, log, diff, etc.)"
            echo "Token savings: 60-80% depending on subcommand"
            ;;
        cargo)
            echo "Filter: cargo build/test/clippy output"
            echo "Token savings: 80-90% (failures only for tests)"
            ;;
        gh)
            echo "Filter: GitHub CLI (pr, issue, run)"
            echo "Token savings: 26-87% depending on subcommand"
            ;;
        pnpm)
            echo "Filter: pnpm package manager"
            echo "Token savings: 70-90% (dependency trees)"
            ;;
        *)
            echo "Filter: Available for $COMMAND"
            echo "Token savings: 60-90% (typical)"
            ;;
    esac
else
    echo "⚠️  No CLOV filter available, would execute raw: $COMMAND $ARGS"
    echo ""
    echo "Note: You can still use 'clov proxy $COMMAND $ARGS' to:"
    echo "  - Execute command without filtering"
    echo "  - Track usage in 'clov gain --history'"
    echo "  - Measure potential for new filter development"
fi
```

### Option 3: Interactive Mode

```bash
COMMAND="$1"
shift
ARGS="$@"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🧪 CLOV Command Routing Test"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Command: $COMMAND $ARGS"
echo ""

# Check if CLOV installed
if ! command -v clov >/dev/null 2>&1; then
    echo "❌ ERROR: CLOV not installed"
    echo "   Install with: cargo install --path ."
    exit 1
fi

# Check CLOV version
CLOV_VERSION=$(clov --version 2>/dev/null | awk '{print $2}')
echo "CLOV Version: $CLOV_VERSION"
echo ""

# Check if command has filter
if clov --help | grep -E "^  $COMMAND" >/dev/null 2>&1; then
    echo "✅ Filter: Available"
    echo ""
    echo "Routing:"
    echo "  Input:  $COMMAND $ARGS"
    echo "  Route:  clov $COMMAND $ARGS"
    echo "  Filter: Applied"
    echo ""

    # Estimate token savings (based on historical data)
    case "$COMMAND" in
        git)
            echo "Expected Token Savings: 60-80%"
            echo "Startup Time: <10ms"
            ;;
        cargo)
            echo "Expected Token Savings: 80-90%"
            echo "Startup Time: <10ms"
            ;;
        gh)
            echo "Expected Token Savings: 26-87%"
            echo "Startup Time: <10ms"
            ;;
        *)
            echo "Expected Token Savings: 60-90%"
            echo "Startup Time: <10ms"
            ;;
    esac
else
    echo "⚠️  Filter: Not available"
    echo ""
    echo "Routing:"
    echo "  Input:  $COMMAND $ARGS"
    echo "  Route:  $COMMAND $ARGS (raw, no CLOV)"
    echo "  Filter: None"
    echo ""
    echo "Alternatives:"
    echo "  - Use 'clov proxy $COMMAND $ARGS' to track usage"
    echo "  - Consider contributing a filter for this command"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
```

## Expected Output

### Cas 1: Commande avec filtre

```bash
/test-routing git status

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🧪 CLOV Command Routing Test
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Command: git status

CLOV Version: 0.16.0

✅ Filter: Available

Routing:
  Input:  git status
  Route:  clov git status
  Filter: Applied

Expected Token Savings: 60-80%
Startup Time: <10ms

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### Cas 2: Commande sans filtre

```bash
/test-routing npm install express

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🧪 CLOV Command Routing Test
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Command: npm install express

CLOV Version: 0.16.0

⚠️  Filter: Not available

Routing:
  Input:  npm install express
  Route:  npm install express (raw, no CLOV)
  Filter: None

Alternatives:
  - Use 'clov proxy npm install express' to track usage
  - Consider contributing a filter for this command

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### Cas 3: CLOV non installé

```bash
/test-routing cargo test

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🧪 CLOV Command Routing Test
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Command: cargo test

❌ ERROR: CLOV not installed
   Install with: cargo install --path .
```

## Use Cases

### Use Case 1: Pre-Flight Check

Avant d'exécuter une commande coûteuse, vérifier si CLOV a un filtre :

```bash
/test-routing cargo build --all-targets
# ✅ Filter available → use clov cargo build
# ⚠️  No filter → use raw cargo build
```

### Use Case 2: Hook Debugging

Tester le hook integration sans side-effects :

```bash
# Test several commands
/test-routing git log -10
/test-routing gh pr view 123
/test-routing docker ps

# Verify routing logic works for all
```

### Use Case 3: Documentation

Générer liste de commandes supportées :

```bash
# Test all common commands
for cmd in git cargo gh pnpm docker npm yarn; do
    /test-routing $cmd
done

# Output shows which have filters
```

### Use Case 4: Contributing New Filter

Identifier commandes sans filtre qui pourraient bénéficier :

```bash
/test-routing pytest
# ⚠️  No filter

# Consider contributing pytest filter
# Expected savings: 90% (failures only)
# Complexity: Medium (JSON output parsing)
```

## Integration avec Claude Code

Dans Claude Code, cette command permet de :

1. **Vérifier hook integration** : Test si hooks rewrites commands correctement
2. **Debugging** : Identifier pourquoi certaines commandes ne sont pas filtrées
3. **Documentation** : Montrer à l'utilisateur quelles commandes CLOV supporte

**Exemple workflow** :

```
User: "Is git status supported by CLOV?"
Assistant: "Let me check with /test-routing git status"
[Runs command]
Assistant: "Yes! CLOV has a filter for git status with 60-80% token savings."
```

## Limitations

- **Dry-run only** : Ne teste pas l'exécution réelle (pas de validation output)
- **No side-effects** : Aucune commande n'est exécutée
- **Routing check only** : Vérifie seulement la disponibilité du filtre, pas la qualité

Pour tester le filtre complet, utiliser :
```bash
clov <cmd>  # Exécution réelle avec filtre
```
