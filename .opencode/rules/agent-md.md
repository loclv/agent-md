# Use agent-md CLI for Markdown Operations

When working with markdown files in this project, always use the `agent-md` CLI tool instead of direct file operations.

## Commands

### Read Files

```bash
agent-md read <path> --field content   # Get content
agent-md read <path> --field headings  # Get headings
agent-md read <path> -f word_count     # Get word count
```

### Write Files

```bash
agent-md write <path> "<content>"
```

### Search Content

```bash
agent-md search <path> "<query>"
```

### Validate Content

```bash
agent-md lint --content "<content>"
```

## Workflow

1. Read markdown with `agent-md read` instead of `cat` or file read tools
2. Search markdown with `agent-md search` instead of `grep`
3. Validate content with `agent-md lint` before writing
4. Write markdown with `agent-md write` instead of direct file writes

## Why

- JSON output for easy parsing
- Built-in validation for AI-friendly content
- Consistent formatting
- Automatic error handling
