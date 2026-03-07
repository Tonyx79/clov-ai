# clov

<p align="center">
  <img src="assets/clov_mascot.png" width="400" alt="clov mascot">
</p>

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Version](https://img.shields.io/badge/version-0.27.7-blue.svg)](https://github.com/alexandephilia/clov-ai/releases/tag/v0.27.7)
[![Built with Rust](https://img.shields.io/badge/built_with-Rust-orange?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Claude Code](https://img.shields.io/badge/Claude_Code-integrated-7B2D8B?logo=anthropic&logoColor=white)](https://claude.ai/code)

**The Universal Context Optimizer for MCP Servers and AI Coders.**

![clov preview](clov_1.jpg)

Model Context Protocol (MCP) servers are powerful, but they are incredibly noisy. When your AI agent uses an MCP search tool or database connector, the server often returns massive JSON blobs full of website navigation chrome, tracking parameters, and raw, untruncated arrays.

`clov` is a structure-aware proxy built specifically to intercept and compress MCP tool responses _before_ they flood your LLM's context window. As a secondary feature, it also aggressively filters standard CLI development commands (git, npm, cargo, etc.).

By deploying `clov` between your AI agent and its tools, you can slash token consumption by **60-95%** without sacrificing the critical data your model needs to solve complex problems. Stop burning your API budget on boilerplate.

---

## The Cost of Context Bloat

![clov savings](clov_2.jpg)

When an AI agent performs a deep research task using an MCP server, a single tool response can easily exceed 50,000 tokens.

With `clov`'s Universal MCP Filtering, these blobs are intelligently pruned:

| Operation                           | Raw Tokens | With clov | Cut     |
| ----------------------------------- | ---------- | --------- | ------- |
| **MCP Web Search Results**          | 65,000     | 4,500     | **93%** |
| **MCP Structured Data Retrieval**   | 40,000     | 5,000     | **87%** |
| **CLI: Test Suites (`cargo test`)** | 25,000     | 2,500     | **90%** |
| **CLI: Git Status / Diffs**         | 13,000     | 3,100     | **75%** |
| **CLI: Linters (`eslint`, `tsc`)**  | 15,000     | 3,000     | **80%** |

_Based on real-world AI coding sessions using standard MCP architectures and medium-sized codebases._

---

## 🚀 Quick Install

```bash
# Homebrew (macOS/Linux)
brew tap alexandephilia/clov
brew install clov

# Cargo
cargo install --git https://github.com/alexandephilia/clov-ai

# Direct bash installer
curl -fsSL https://raw.githubusercontent.com/alexandephilia/clov-ai/refs/heads/main/install.sh | sh
```

_(Check releases for pre-built Windows, macOS, and Linux binaries)._

---

## 🔌 Universal MCP Integration

To optimize your MCP servers, simply wrap their execution command with `clov mcp proxy`. `clov` acts as a transparent JSON-RPC layer, analyzing the responses structurally.

Configure your AI agent (e.g., in `~/.claude/settings.json` for Claude Code):

```json
"mcpServers": {
  "generic-search": {
    "command": "clov",
    "args": ["mcp", "proxy", "npx", "-y", "some-mcp-server"]
  },
  "database-connector": {
    "command": "clov",
    "args": ["mcp", "proxy", "python", "-m", "db_mcp"]
  }
}
```

### How the Universal Filter Works:

1. **Content Detection**: Automatically identifies if a response contains Web Search data, Raw Code, Structured JSON arrays, or Plain Text.
2. **Chrome Stripping**: Heuristically removes website navigation headers, footers, ad blocks, and tracking URLs from search results across _all_ providers.
3. **Adaptive Truncation**: Scales truncation limits dynamically based on the information density of the internal text.

---

## 💻 CLI Proxy Integration

In addition to orchestrating MCP servers, `clov` optimizes standard shell commands. If you use Claude Code, `clov` can inject an auto-rewrite hook to automatically filter terminal output without modifying your prompt.

```bash
# Register the auto-rewrite hook globally
clov init --global
```

By installing the hook, your AI agent can issue commands like `git log`, `npm test`, or `cargo clippy`, and `clov` will silently intercept them, stripping ANSI codes, progress bars, and successful test output, returning only the errors and summaries to the model.

### Supported CLI Toolchains

- **Git & GitHub CLI**: Condensed statuses, diffs, PR listings.
- **JavaScript / TypeScript**: `npm`, `pnpm`, `eslint`, `tsc`, `Next.js`, `vitest`, `playwright`, `prisma`.
- **Rust**: `cargo test`, `cargo build`, `cargo clippy`.
- **Python**: `pytest`, `ruff`, `mypy`, `pip`.
- **Go**: `go test`, `go build`, `golangci-lint`.
- **Infra**: `docker`, `kubectl`, `aws sts`.

_Any unrecognized command simply passes through unchanged._

---

## 📊 Analytics & Tracking

Curious how many tokens you've saved? `clov` tracks your session economics locally.

```bash
clov gain             # Summary of total tokens saved
clov gain --graph     # Visual savings chart over 30 days
clov gain --all       # Daily, weekly, and monthly breakdowns
```

**Example output:**

```
╔══════════════════════════════════════════════════════╗
║          CLOV Token Savings (Global Scope)           ║
╠══════════════════════════════════════════════════════╣
║  Total commands  :   133                             ║
║  Input tokens    :  30.5K                            ║
║  Output tokens   :  10.7K                            ║
║  Tokens saved    :  25.3K  (83.0%)                   ║
╚══════════════════════════════════════════════════════╝
```

---

## Configuration & Advanced Usage

`clov` works perfectly out of the box, but you can configure tracking databases and output telemetry via environment variables or `~/.config/clov/config.toml`.

```bash
clov config --create    # Generate default config
clov verify             # Check hook integrity
clov discover           # Scan AI conversation history for missed savings
```

### Unfiltered Recovery (Tee Mode)

If `clov` aggressively filters a test failure and the LLM needs more detail, `clov` saves the pristine output to a temporary log file. The LLM is provided a one-line path to the full log, meaning it can read the raw data if absolutely necessary, entirely circumventing the "silent failure" problem in AI coding.

---

## Documentation

- [ARCHITECTURE.md](ARCHITECTURE.md) — The Universal Filter architecture and JSON-RPC proxy mechanics.
- [CLAUDE.md](CLAUDE.md) — Agentic guidance for Claude Code.
- [docs/AUDIT_GUIDE.md](docs/AUDIT_GUIDE.md) — Generating token economy exports and CSV analytics.

---

## License

MIT — see [LICENSE](LICENSE).

<p align="center">
  <img src="https://skillicons.dev/icons?i=rust,python,go" height="24" />
  <br/><br/>
  <sub>maintained by <a href="https://github.com/alexandephilia">@alexandephilia</a> × claude</sub>
</p>
