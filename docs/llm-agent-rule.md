# LLM Agent Rule: Use agent-md CLI

## Core Rule

When working with markdown files, always use the agent-md CLI tool instead of direct file operations.

## Why Use agent-md

- JSON output for easy parsing
- Built-in validation for AI-friendly content
- Designed for AI agent workflows
- Consistent formatting

## Basic Commands

### Read Files

```bash
DATA=$(agent-md read <path>)
CONTENT=$(echo "$DATA" | jq -r '.content')
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

## Integration Pattern

```bash
DOC=$(agent-md read README.md)
agent-md search README.md "TODO"
if agent-md lint --content "$CONTENT" | jq -e '.valid' > /dev/null; then
    agent-md write file.md "$CONTENT"
fi
```

## Key Benefits

1. Structured JSON output
2. Automatic validation
3. Error handling
4. AI-optimized parsing

This rule ensures consistent markdown operations for AI agents.
