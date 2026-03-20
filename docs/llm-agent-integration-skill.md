# LLM/Agent Integration Skill: agent-md CLI

## Overview

This skill provides guidelines for LLMs and AI agents to effectively use the agent-md CLI tool for markdown file operations. The tool is specifically designed for AI workflows with JSON output and built-in validation.

## Core Principle

ALWAYS use agent-md commands instead of direct file operations when working with markdown files.

## Command Reference

### Reading Files

```bash
CONTENT=$(agent-md read <path> --field content)
WORD_COUNT=$(agent-md read <path> -f word_count)
HEADINGS=$(agent-md read <path> -f headings)
LINE_COUNT=$(agent-md read <path> -f line_count)
```

### Writing Files

```bash
agent-md write <path> "<content>"
agent-md lint --content "<content>" && agent-md write <path> "<content>"
```

### Searching Content

```bash
agent-md search <path> "<query>"
```

### Navigation

```bash
agent-md headings <path>
agent-md stats <path>
agent-md list <directory>
```

### Content Modification

```bash
agent-md append <path> "<content>"
agent-md insert <path> <line> "<content>"
agent-md delete <path> <line> [count]
```

### Validation

```bash
agent-md lint <path>
agent-md lint --content "<content>"
agent-md lint-file <path>
```

## Standard Workflow Pattern

### Document Analysis

```bash
# Read and analyze existing document
echo "Document has $(agent-md read README.md -f word_count) words"
echo "Document structure:"
agent-md read README.md -f headings
```

### Content Search

```bash
# Find specific sections or content
agent-md search README.md "installation"
```

### Content Validation

```bash
# Validate new content before writing
NEW_CONTENT="# New Section\nContent here"
agent-md lint --content "$NEW_CONTENT"

# If valid, write the content
agent-md write document.md "$NEW_CONTENT"
```

### Document Updates

```bash
# Insert content at specific location
agent-md insert README.md 10 "## New Section\nAdded content"

# Append to document
agent-md append README.md "## Additional Notes\nMore content"
```

## Best Practices

### Always Use JSON Output

- Use --field (-f) to extract specific fields from JSON responses
- Never rely on plain text output for programmatic use
- Handle validation errors from JSON responses

### Validate Before Writing

```bash
# Good: Validate first
CONTENT="# Title\nSome content"
agent-md lint --content "$CONTENT"
agent-md write file.md "$CONTENT"

# Bad: Write without validation
echo "$CONTENT" > file.md
```

### Use Structured Data

```bash
# Good: Use structured JSON data
HEADINGS=$(agent-md read file.md -f headings)

# Bad: Parse plain text
grep "^#" file.md
```

### Handle Errors Gracefully

```bash
# Always check command success
CONTENT=$(agent-md read file.md -f content 2>/dev/null)
if [ $? -eq 0 ]; then
    # Process successful result
    echo "$CONTENT"
else
    # Handle error case
    echo "Failed to read file"
fi
```

## Integration Examples

### ChatGPT/Claude Integration

When I ask you to read a markdown file, use:

```bash
agent-md read <path>
```

When I ask you to write markdown content, use:

```bash
agent-md write <path> "<content>"
```

When I ask you to search in markdown, use:

```bash
agent-md search <path> "<query>"
```

### Automated Scripting

```bash
#!/bin/bash
# Example: Update all TODO items in markdown files

for file in $(agent-md list .); do
    # Search for TODOs
    agent-md search "$file" "TODO"
done
```

### Document Processing Pipeline

```bash
# 1. Extract structure
agent-md headings document.md

# 2. Validate content
agent-md lint document.md

# 3. Convert to structured format
agent-md to-jsonl document.md
```

## Error Handling

### Common Error Scenarios

```bash
# File not found
agent-md read nonexistent.md
# Returns: {"success": false, "message": "Failed to read file: ...", "document": null}

# Validation errors
agent-md write file.md "**Bold text**"
# Returns: {"success": false, "message": "Content validation failed: 1 errors found", "document": null}

# Invalid command parameters
agent-md delete file.md 0
# Returns error via stderr
```

### Error Handling Pattern

```bash
execute_agent_md() {
    local cmd="$1"
    shift
    local result

    result=$("$cmd" "$@" 2>/dev/null)
    local exit_code=$?

    if [ $exit_code -eq 0 ]; then
        echo "$result"
    else
        echo "{\"success\": false, \"message\": \"Command failed: $cmd $*\"}"
    fi
}
```

## Memory Context

When working with multiple markdown files, maintain context using the structured data:

```bash
# Search across all markdown files
for file in $(agent-md list .); do
    agent-md search "$file" "$search_term"
done
```

## Performance Considerations

- Use agent-md list to discover files before processing
- Cache document data when working with multiple operations on the same file
- Use agent-md stats for quick document overviews
- Validate content in batches when possible

## Tool Limitations

- Only handles markdown files (.md, .markdown)
- Large files (>10MB) may have performance impacts
- Complex nested structures may require additional processing
- Some advanced markdown features may not be fully supported

## Troubleshooting

### Common Issues

1. Command not found: Ensure agent-md is in PATH
2. Permission denied: Check file permissions
3. Validation failures: Review markdown writing rules
4. JSON parsing errors: Validate JSON structure before processing

### Debug Mode

```bash
# Enable verbose output for debugging
export AGENT_MD_DEBUG=1
agent-md read file.md
```

This skill ensures consistent and reliable markdown file operations for AI agents and LLMs.
