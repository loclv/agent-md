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
agent-md read <path> --field content   # Get content
# example:
agent-md read README.md -f content

agent-md read <path> --field headings  # Get headings
# example:
agent-md read README.md -f headings

agent-md read <path> -f word_count     # Short form for word count
# example:
agent-md read README.md -f word_count

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
# example:
agent-md lint --content "# Hello\n\nThis is a test"
```

## Integration Pattern

```bash
agent-md read README.md -f headings
agent-md search README.md "TODO"
agent-md lint --content "$CONTENT"
agent-md write file.md "$CONTENT"
```

## Key Benefits

1. Structured JSON output
2. Automatic validation
3. Error handling
4. AI-optimized parsing

This rule ensures consistent markdown operations for AI agents.
