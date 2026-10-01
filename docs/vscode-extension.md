# VS Code Extension: Agent-MD Formatter

Format Markdown files using the `agent-md` CLI tool directly inside VS Code, Cursor, Devin, and Antigravity.

## Overview

The `agent-md` formatter extension integrates the `agent-md` CLI into your editor, providing instant, token-efficient Markdown formatting on demand or on save.

## Installation

### Install from Marketplace

Search for "Agent-MD Formatter" in the VS Code Extensions tab or install directly from the [VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=loclv.agent-md-formatter).

### Install from Source

1. Build the `agent-md` CLI first:

```bash
cargo build --release
```

2. Build and package the extension:

```bash
cd vscode-extension
bun i
bun compile
```

3. Install the resulting `.vsix` file in VS Code:

- Open VS Code
- Press `Cmd+Shift+X` (macOS) or `Ctrl+Shift+X` (Windows/Linux) to open Extensions
- Click the "..." menu in the top right of the Extensions panel
- Select "Install from VSIX..."
- Choose `vscode-extension/agent-md-formatter-0.1.0.vsix`

Or install via terminal:

```bash
code --install-extension vscode-extension/agent-md-formatter-0.1.0.vsix
```

## Features

- Format on demand via editor command or shortcut
- Format on save support
- Respects project `.markdownlintignore` and `.gitignore` rules
- Configurable token optimization options
- Seamless integration with standard VS Code formatter provider

## Extension Settings

All settings can be configured in your VS Code user or workspace `settings.json`:

- `agentMd.path`: Path to `agent-md` executable (default: `agent-md`, automatically detects workspace `target/release`, `~/.cargo/bin`, and common local paths)
- `agentMd.format.removeBold`: Remove bold markers (default: `true`)
- `agentMd.format.compactBlankLines`: Compact consecutive blank lines (default: `true`)
- `agentMd.format.collapseSpaces`: Collapse redundant spaces between words (default: `true`)
- `agentMd.format.removeHorizontalRules`: Remove horizontal rule lines (default: `true`)
- `agentMd.format.removeEmphasis`: Remove italic markers (default: `true`)

## Usage and Keybindings

| Command | Keybinding |
|---|---|
| Format Document | `Shift+Option+F` (macOS) / `Shift+Alt+F` (Windows/Linux) |

### Enable Format on Save

Add the following to your VS Code `settings.json`:

```json
{
  "[markdown]": {
    "editor.formatOnSave": true,
    "editor.defaultFormatter": "agent-md.agent-md-formatter"
  }
}
```
