# CueDeck

An intelligent workspace management system for AI agents.

## Quick Start

```bash
# Build the project
cargo build --release

# Run CLI
cargo run --bin cue -- --help

# Run tests
cargo test
```

## Project Structure

```
CueDeck/
├── crates/
│   ├── cue_common/   # Shared types and errors
│   ├── cue_config/   # Configuration management
│   ├── cue_core/     # Core engine (parser, graph, scene generation)
│   ├── cue_mcp/      # MCP server (JSON-RPC)
│   └── cue_cli/      # Command-line interface
└── docs/             # Documentation
```

## Documentation

See the [docs/](./docs/) directory for comprehensive documentation:

- **General**: [00_INDEX.md](./docs/00_INDEX.md)
- **Architecture**: [docs/02_architecture/](./docs/02_architecture/)
- **Agent Design**: [docs/03_agent_design/](./docs/03_agent_design/)
- **Tools & API**: [docs/04_tools_and_data/](./docs/04_tools_and_data/)
- **Quality & Ops**: [docs/05_quality_and_ops/](./docs/05_quality_and_ops/)

## License

MIT
