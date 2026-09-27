# Agent-MD Formatter for VS Code

<div align="center"><img src="../logo.svg" alt="agent-md logo" width="128" height="128"><br /><div style="display: flex; flex-wrap: wrap; justify-content: center; gap: 12px; margin-top: 22px;"><div style="margin-top: auto; margin-bottom: auto;"><a href="https://dopana.com" target="_blank">Sponsored by dopana.com</a></div><a href="https://unikorn.vn/p/agent-md?ref=embed-agent-md" target="_blank"><img src="https://unikorn.vn/api/widgets/badge/agent-md?theme=dark" alt="agent-md trên Unikorn.vn" style="width: 256px; height: 64px;" width="256" height="64" /></a></div></div>

<div align="center"><a href="https://marketplace.visualstudio.com/items?itemName=loclv.agent-md-formatter&ssr=false#overview" target="_blank">Open in VS Code</a></div>

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

- `agentMd.path`: Path to the agent-md executable (default: `agent-md`). Automatically detects workspace `target/release`, `~/.cargo/bin`, and common local paths when not found in `PATH`.
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
