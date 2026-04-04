# Agent-MD Formatter for VS Code

Format Markdown files using the `agent-md` CLI tool.

## Features

- Format Markdown files on demand or on save
- Configurable formatting options
- Integrates with VS Code's built-in formatting system

## Requirements

The `agent-md` CLI must be installed and available in your PATH.

### Installing agent-md

```bash
# From source
git clone https://github.com/loclv/agent-md
cd agent-md
cargo build --release
```

## Extension Settings

This extension contributes the following settings:
- `agentMd.path`: Path to the agent-md executable (default: `agent-md`)
- `agentMd.format.removeBold`: Remove bold markers (`**` and `__`) (default: `true`)
- `agentMd.format.compactBlankLines`: Compact blank lines (default: `true`)
- `agentMd.format.collapseSpaces`: Collapse multiple spaces between words (default: `true`)
- `agentMd.format.removeHorizontalRules`: Remove horizontal rules (`---`, `***`, `___`) (default: `true`)
- `agentMd.format.removeEmphasis`: Remove emphasis markers (`*` and `_`) (default: `true`)

## Usage

1. Open a Markdown file (`.md` or `.markdown`)
2. Use `Shift+Alt+F` (Windows/Linux) or `Shift+Option+F` (macOS) to format
3. Or enable "Format on Save" in VS Code settings:
   ```json
   {
     "[markdown]": {
       "editor.formatOnSave": true,
       "editor.defaultFormatter": "agent-md.agent-md-formatter"
     }
   }
   ```

## Keyboard Shortcuts

| Command | Key |
|---|---|
| Format Document | `Shift+Alt+F` (Windows/Linux) or `Shift+Option+F` (macOS) |

## Known Issues

None at this time.

## Release Notes

### 0.1.0

Initial release:
- Document formatting support for Markdown files
- Configurable formatting options matching agent-md CLI flags
- Error handling for missing agent-md executable
