# CueDeck VSCode Extension

AI-native knowledge graph extension for CueDeck workspaces.

## Features

- **Smart Search**: Search across your markdown knowledge graph with `Cmd+Shift+P` → "CueDeck: Search Documents"
- **Task Management**: View and manage tasks from your workspace
- **Graph Visualization**: (Coming in Phase 8.2)
- **Real-time Sync**: (Coming in Phase 8.2)

## Requirements

- CueDeck CLI must be installed and available in PATH
- A workspace with `.cuedeck/config.toml` file

## Extension Settings

This extension contributes the following settings:

- `cuedeck.cliPath`: Path to CueDeck CLI binary (default: `cue`)
- `cuedeck.enableSync`: Enable real-time sync with team members (default: `false`)

## Usage

1. Open a CueDeck workspace (folder containing `.cuedeck/config.toml`)
2. The extension will activate automatically
3. Use `Cmd/Ctrl+Shift+P` and search for "CueDeck" commands

## Development

```bash
cd extensions/vscode
npm install
npm run compile
# Press F5 in VSCode to launch Extension Development Host
```

## Release Notes

### 0.1.0 (Phase 8.1)

- Initial release
- Search panel with CLI integration
- Task sidebar view
- JSON output support

## License

MIT
