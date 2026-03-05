---
model: haiku
description: CLOV environment diagnostics - Checks installation, hooks, version, command routing
---

# /diagnose

Vérifie l'état de l'environnement CLOV et suggère des corrections.

## Quand utiliser

- **Automatiquement suggéré** quand Claude détecte ces patterns d'erreur :
  - `clov: command not found` → CLOV non installé ou pas dans PATH
  - Hook errors in Claude Code → Hooks mal configurés ou non exécutables
  - `Unknown command` dans CLOV → Version incompatible ou commande non supportée
  - Token savings reports missing → `clov gain` not working
  - Command routing errors → Hook integration broken

- **Manuellement** après installation, mise à jour CLOV, ou si comportement suspect

## Exécution

### 1. Vérifications parallèles

Lancer ces commandes en parallèle :

```bash
# CLOV installation check
which clov && clov --version || echo "❌ CLOV not found in PATH"
```

```bash
# Git status (verify working directory)
git status --short && git branch --show-current
```

```bash
# Hook configuration check
if [ -f ".claude/hooks/clov-rewrite.sh" ]; then
    echo "✅ OK: clov-rewrite.sh hook present"
    # Check if hook is executable
    if [ -x ".claude/hooks/clov-rewrite.sh" ]; then
        echo "✅ OK: hook is executable"
    else
        echo "⚠️ WARNING: hook not executable (chmod +x needed)"
    fi
else
    echo "❌ MISSING: clov-rewrite.sh hook"
fi
```

```bash
# Hook clov-suggest.sh check
if [ -f ".claude/hooks/clov-suggest.sh" ]; then
    echo "✅ OK: clov-suggest.sh hook present"
    if [ -x ".claude/hooks/clov-suggest.sh" ]; then
        echo "✅ OK: hook is executable"
    else
        echo "⚠️ WARNING: hook not executable (chmod +x needed)"
    fi
else
    echo "❌ MISSING: clov-suggest.sh hook"
fi
```

```bash
# Claude Code context check
if [ -n "$CLAUDE_CODE_HOOK_BASH_TEMPLATE" ]; then
    echo "✅ OK: Running in Claude Code context"
    echo "   Hook env var set: CLAUDE_CODE_HOOK_BASH_TEMPLATE"
else
    echo "⚠️ WARNING: Not running in Claude Code (hooks won't activate)"
    echo "   CLAUDE_CODE_HOOK_BASH_TEMPLATE not set"
fi
```

```bash
# Test command routing (dry-run)
if command -v clov >/dev/null 2>&1; then
    # Test if clov gain works (validates install)
    if clov --help | grep -q "gain"; then
        echo "✅ OK: clov gain available"
    else
        echo "❌ MISSING: clov gain command (old version or wrong binary)"
    fi
else
    echo "❌ CLOV binary not found"
fi
```

### 2. Validate token analytics

```bash
# Run clov gain to verify analytics work
if command -v clov >/dev/null 2>&1; then
    echo ""
    echo "📊 Token Savings (last 5 commands):"
    clov gain --history 2>&1 | head -8 || echo "⚠️ clov gain failed"
else
    echo "⚠️ Cannot test clov gain (binary not installed)"
fi
```

### 3. Quality checks (if in CLOV repo)

```bash
# Only run if we're in CLOV repository
if [ -f "Cargo.toml" ] && grep -q 'name = "clov"' Cargo.toml 2>/dev/null; then
    echo ""
    echo "🦀 CLOV Repository Quality Checks:"

    # Check if cargo fmt passes
    if cargo fmt --all --check >/dev/null 2>&1; then
        echo "✅ OK: cargo fmt (code formatted)"
    else
        echo "⚠️ WARNING: cargo fmt needed"
    fi

    # Check if cargo clippy would pass (don't run full check, just verify binary)
    if command -v cargo-clippy >/dev/null 2>&1 || cargo clippy --version >/dev/null 2>&1; then
        echo "✅ OK: cargo clippy available"
    else
        echo "⚠️ WARNING: cargo clippy not installed"
    fi
else
    echo "ℹ️ Not in CLOV repository (skipping quality checks)"
fi
```

## Format de sortie

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🔍 CLOV Environment Diagnostic
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📦 CLOV Binary:      ✅ OK (v0.16.0) | ❌ NOT FOUND
🔗 Hooks:           ✅ OK (clov-rewrite.sh + clov-suggest.sh executable)
                    ❌ MISSING or ⚠️ WARNING (not executable)
📊 Token Analytics: ✅ OK (clov gain working)
                    ❌ FAILED (command not available)
🎯 Claude Context:  ✅ OK (hook environment detected)
                    ⚠️ WARNING (not in Claude Code)
🦀 Code Quality:    ✅ OK (fmt + clippy ready) [if in CLOV repo]
                    ⚠️ WARNING (needs formatting/clippy)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

## Actions suggérées

Utiliser `AskUserQuestion` si problèmes détectés :

```
question: "Problèmes détectés. Quelles corrections appliquer ?"
header: "Fixes"
multiSelect: true
options:
  - label: "cargo install --path ."
    description: "Installer CLOV localement depuis le repo"
  - label: "chmod +x .claude/hooks/bash/*.sh"
    description: "Rendre les hooks exécutables"
  - label: "Tout corriger (recommandé)"
    description: "Install CLOV + fix hooks permissions"
```

**Adaptations selon contexte** :

### Si CLOV non installé
```
options:
  - label: "cargo install --path ."
    description: "Installer CLOV localement (si dans le repo)"
  - label: "cargo install clov"
    description: "Installer CLOV depuis crates.io (dernière release)"
  - label: "brew install clov-ai/tap/clov"
    description: "Installer CLOV via Homebrew (macOS/Linux)"
```

### Si hooks manquants/non exécutables
```
options:
  - label: "chmod +x .claude/hooks/*.sh"
    description: "Rendre tous les hooks exécutables"
  - label: "Copier hooks depuis template"
    description: "Si hooks manquants, copier depuis repository principal"
```

### Si clov gain échoue
```
options:
  - label: "Réinstaller CLOV"
    description: "cargo install --path . --force (version outdated?)"
  - label: "Vérifier version"
    description: "clov --version (besoin v0.16.0+ pour clov gain)"
```

## Exécution des fixes

### Fix 1 : Installer CLOV localement
```bash
cd /Users/florianbruniaux/Sites/clov-ai/clov
cargo install --path .
# Vérifier installation
which clov && clov --version
```

### Fix 2 : Rendre hooks exécutables
```bash
chmod +x .claude/hooks/*.sh
# Vérifier permissions
ls -la .claude/hooks/*.sh
```

### Fix 3 : Tout corriger (recommandé)
```bash
# Install CLOV
cargo install --path .

# Fix hooks permissions
chmod +x .claude/hooks/*.sh

# Verify
which clov && clov --version && clov gain --history | head -3
```

## Détection automatique

**IMPORTANT** : Claude doit suggérer `/diagnose` automatiquement quand il voit :

| Erreur | Pattern | Cause probable |
|--------|---------|----------------|
| CLOV not found | `clov: command not found` | Pas installé ou pas dans PATH |
| Hook error | Hook execution failed, permission denied | Hooks non exécutables (`chmod +x` needed) |
| Version mismatch | `Unknown command` in CLOV output | Version CLOV incompatible (upgrade needed) |
| No analytics | `clov gain` fails or command not found | CLOV install incomplete or old version |
| Command not rewritten | Commands not proxied via CLOV | Hook integration broken (check `CLAUDE_CODE_HOOK_BASH_TEMPLATE`) |

### Exemples de suggestion automatique

**Cas 1 : CLOV command not found**
```
Cette erreur "clov: command not found" indique que CLOV n'est pas installé
ou pas dans le PATH. Je suggère de lancer `/diagnose` pour vérifier
l'installation et obtenir les commandes de fix.
```

**Cas 2 : Hook permission denied**
```
L'erreur "Permission denied" sur le hook clov-rewrite.sh indique que
les hooks ne sont pas exécutables. Lance `/diagnose` pour identifier
le problème et corriger les permissions avec `chmod +x`.
```

**Cas 3 : clov gain unavailable**
```
La commande `clov gain` échoue, ce qui suggère une version CLOV obsolète
ou une installation incomplète. `/diagnose` va vérifier la version et
suggérer une réinstallation si nécessaire.
```

## Troubleshooting Common Issues

### Issue : CLOV installed but not in PATH

**Symptom**: `cargo install --path .` succeeds but `which clov` fails

**Diagnosis**:
```bash
# Check if binary installed in Cargo bin
ls -la ~/.cargo/bin/clov

# Check if ~/.cargo/bin in PATH
echo $PATH | grep -q .cargo/bin && echo "✅ In PATH" || echo "❌ Not in PATH"
```

**Fix**:
```bash
# Add to ~/.zshrc or ~/.bashrc
export PATH="$HOME/.cargo/bin:$PATH"

# Reload shell
source ~/.zshrc  # or source ~/.bashrc
```

### Issue : Multiple CLOV binaries (name collision)

**Symptom**: `clov gain` fails with "command not found" even though `clov --version` works

**Diagnosis**:
```bash
# Check if wrong CLOV installed (alexandephilia/clov-ai)
clov --version
# Should show "clov X.Y.Z", NOT "Clov Token Omitter"

clov --help | grep gain
# Should show "gain" command - if missing, wrong binary
```

**Fix**:
```bash
# Uninstall wrong CLOV
cargo uninstall clov

# Install correct CLOV (this repo)
cargo install --path .

# Verify
clov gain --help  # Should work
```

### Issue : Hooks not triggering in Claude Code

**Symptom**: Commands not rewritten to `clov <cmd>` automatically

**Diagnosis**:
```bash
# Check if in Claude Code context
echo $CLAUDE_CODE_HOOK_BASH_TEMPLATE
# Should print hook template path - if empty, not in Claude Code

# Check hooks exist and executable
ls -la .claude/hooks/*.sh
# Should show -rwxr-xr-x (executable)
```

**Fix**:
```bash
# Make hooks executable
chmod +x .claude/hooks/*.sh

# Verify hooks load in new Claude Code session
# (restart Claude Code session after chmod)
```

## Version Compatibility Matrix

| CLOV Version | clov gain | clov discover | Python/Go support | Notes |
|-------------|----------|--------------|-------------------|-------|
| v0.14.x     | ❌ No    | ❌ No        | ❌ No             | Outdated, upgrade |
| v0.15.x     | ✅ Yes   | ❌ No        | ❌ No             | Missing discover |
| v0.16.x     | ✅ Yes   | ✅ Yes       | ✅ Yes            | **Recommended** |
| main branch | ✅ Yes   | ✅ Yes       | ✅ Yes            | Latest features |

**Upgrade recommendation**: If running v0.15.x or older, upgrade to v0.16.x:

```bash
cd /Users/florianbruniaux/Sites/clov-ai/clov
git pull origin main
cargo install --path . --force
clov --version  # Should show 0.16.x or newer
```
