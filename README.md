# CueDeck

An intelligent workspace management system for AI agents. CueDeck helps you organize project context, manage tasks, and integrate seamlessly with AI tools through the Model Context Protocol (MCP).

## ✨ Features

- 📝 **Smart Context Management** - Automatically track and snapshot your project context
- 🎯 **Task Cards** - Simple, file-based task management with unique IDs
- 🔍 **Intelligent Search** - Keyword and semantic search across your workspace
- 🔗 **Dependency Graph** - Visualize relationships between documents
- 🤖 **MCP Integration** - Native support for AI agents (Claude, Antigravity, etc.)
- 🩺 **Health Checks** - Built-in diagnostics for workspace integrity

## 📦 Installation

### Option 1: Pre-built Binary (Recommended)

**Windows:**

1. Download the latest release from [Releases](https://github.com/TCTri205/CueDeck/releases)
2. Extract `cue.exe` to a directory of your choice
3. Add to PATH or copy to `C:\Windows\System32\`

**Verify installation:**

```bash
cue --version
```

Expected output: `cue 0.1.0`

### Option 2: Build from Source

**Requirements:**

- Rust 1.75+ ([install rustup](https://rustup.rs/))
- Git

**Steps:**

```bash
# Clone the repository
git clone https://github.com/TCTri205/CueDeck.git
cd CueDeck

# Build release binary
cargo build --release

# Binary location: target/release/cue.exe
# Add to PATH or run directly
target\release\cue.exe --version
```

## 🚀 Quick Start

```bash
# Initialize workspace
cue init

# Create your first task
cue card new "My first task"

# List all tasks
cue list

# Generate context snapshot
cue scene

# Check workspace health
cue doctor
```

For comprehensive command reference, see [CLI_REFERENCE.md](./docs/04_tools_and_data/CLI_REFERENCE.md).

## 🔌 MCP Integration

CueDeck integrates with AI tools through the **Model Context Protocol** (MCP). This enables AI agents to read your documentation, manage tasks, and search your workspace.

### Antigravity IDE Setup

1. Open Antigravity IDE settings
2. Navigate to **MCP Servers** configuration
3. Add the following configuration:

```json
{
  "cuedeck": {
    "command": "D:\\path\\to\\cue.exe",
    "args": ["mcp"],
    "env": {
      "CUE_WORKSPACE": "D:\\your\\project"
    }
  }
}
```

1. Restart Antigravity IDE
2. Verify connection: Green status in MCP Servers tab

### Claude Desktop Setup

Add to your Claude Desktop configuration file (`claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "cuedeck": {
      "command": "cue",
      "args": ["mcp"]
    }
  }
}
```

**Configuration file locations:**

- Windows: `%APPDATA%\Claude\claude_desktop_config.json`
- macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`

### Available MCP Tools

Once connected, AI agents can use:

- `@cuedeck read_context` - Search workspace with keywords or semantics
- `@cuedeck read_doc` - Read specific markdown files or sections
- `@cuedeck list_tasks` - List cards filtered by status
- `@cuedeck update_task` - Modify task metadata

See [TOOLS_SPEC.md](./docs/04_tools_and_data/TOOLS_SPEC.md) for detailed API documentation.

## 📚 Project Structure

```
CueDeck/
├── crates/
│   ├── cue_common/   # Shared types and errors
│   ├── cue_config/   # Configuration management
│   ├── cue_core/     # Core engine (parser, graph, scene generation)
│   ├── cue_mcp/      # MCP server (JSON-RPC)
│   └── cue_cli/      # Command-line interface
├── docs/             # Comprehensive documentation
└── .cuedeck/         # Workspace data (created by cue init)
    ├── config.toml   # User configuration
    ├── cards/        # Task cards
    └── docs/         # Project documentation
```

## 📖 Documentation

See the [docs/](./docs/00_INDEX.md) directory for comprehensive documentation:

- **General**: [00_INDEX.md](./docs/00_INDEX.md)
- **Architecture**: [docs/02_architecture/](./docs/02_architecture/SYSTEM_ARCHITECTURE.md)
- **Agent Design**: [docs/03_agent_design/](./docs/03_agent_design/AGENT_PERSONA.md)
- **Tools & API**: [docs/04_tools_and_data/](./docs/04_tools_and_data/CLI_REFERENCE.md)
- **Quality & Ops**: [docs/05_quality_and_ops/](./docs/05_quality_and_ops/TESTING_STRATEGY.md)

## 🛠️ Troubleshooting

### "cue is not recognized as a command"

**Solution**: Add `cue.exe` to your system PATH:

```powershell
# PowerShell (run as Administrator)
$env:Path += ";D:\path\to\directory\containing\cue.exe"
setx PATH "$env:Path" /M
```

### MCP Server Not Connecting

1. Verify `cue.exe` path in MCP configuration
2. Check `CUE_WORKSPACE` environment variable
3. Run `cue doctor` to diagnose workspace issues
4. Check AI tool logs for JSON-RPC errors

### Semantic Search Not Working

First-time semantic search downloads a ~22MB embedding model. Ensure:

- Stable internet connection
- Write permissions in `.cuedeck/.cache/`
- Run with `--verbose` flag to see download progress

## 🤝 Contributing

Contributions are welcome! Please see our documentation for architecture details and development guidelines.

## 📄 License

MIT

## 🔗 Links

- [GitHub Repository](https://github.com/TCTri205/CueDeck)
- [Issue Tracker](https://github.com/TCTri205/CueDeck/issues)
- [Full Documentation](./docs/00_INDEX.md)
