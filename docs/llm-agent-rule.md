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

# Or extract specific fields without jq
agent-md read <path> --field path # Get file path
agent-md read <path> --field content # Get content
agent-md read <path> --field headings # Get headings
agent-md read <path> -f word_count # Short form for word count
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

## Troubleshooting jq Errors

When parsing JSON output with jq, you may encounter control character errors:

```bash
# This may fail with parsing error
DOC=$(agent-md read README.md)
echo "$DOC" | jq '.headings'
```

**Use these solutions:**

1. **Use --field option (recommended):**

```bash
agent-md read README.md --field path # Get file path
agent-md read README.md --field content # Get content
agent-md read README.md --field headings # Get headings
agent-md read README.md -f word_count # Short form for word count
```

- **Raw output for specific fields:**

```bash
agent-md read README.md | jq --raw-output '.headings'
```

- **Dedicated commands:**

```bash
agent-md headings README.md
```

- **Store and access:**

```bash
DOC=$(agent-md read README.md)
echo "$DOC" | jq '.headings'
```

## Key Benefits

1. Structured JSON output
2. Automatic validation
3. Error handling
4. AI-optimized parsing

This rule ensures consistent markdown operations for AI agents.
