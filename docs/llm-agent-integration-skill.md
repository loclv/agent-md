# LLM/Agent Integration Skill: agent-md CLI

## Overview

This skill provides guidelines for LLMs and AI agents to effectively use the agent-md CLI tool for markdown file operations. The tool is specifically designed for AI workflows with JSON output and built-in validation.

## Core Principle

ALWAYS use agent-md commands instead of direct file operations when working with markdown files.

## Command Reference

### Reading Files

```bash
DATA=$(agent-md read <path>)
CONTENT=$(echo "$DATA" | jq -r '.content')
WORD_COUNT=$(echo "$DATA" | jq '.word_count')
HEADINGS=$(echo "$DATA" | jq '.headings')
LINE_COUNT=$(echo "$DATA" | jq '.line_count')
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
DOC=$(agent-md read README.md)
echo "Document has $(echo "$DOC" | jq '.word_count') words"
echo "Document structure:"
echo "$DOC" | jq '.headings'
```

### Content Search

```bash
# Find specific sections or content
SEARCH_RESULT=$(agent-md search README.md "installation")
echo "Found $(echo "$SEARCH_RESULT" | jq '.total') matches"
```

### Content Validation

```bash
# Validate new content before writing
NEW_CONTENT="# New Section\nContent here"
VALIDATION=$(agent-md lint --content "$NEW_CONTENT")

if echo "$VALIDATION" | jq -e '.valid' > /dev/null; then
    echo "Content is valid, proceeding with write"
    agent-md write document.md "$NEW_CONTENT"
else
    echo "Validation failed:"
    echo "$VALIDATION" | jq '.errors'
fi
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

- Parse JSON responses with jq or equivalent
- Never rely on plain text output for programmatic use
- Handle validation errors from JSON responses

### Validate Before Writing

```bash
# Good: Validate first
CONTENT="# Title\nSome content"
if agent-md lint --content "$CONTENT" | jq -e '.valid' > /dev/null; then
    agent-md write file.md "$CONTENT"
fi

# Bad: Write without validation
echo "$CONTENT" > file.md
```

### Use Structured Data

```bash
# Good: Use structured JSON data
DOC=$(agent-md read file.md)
HEADINGS=$(echo "$DOC" | jq '.headings[] | .text')

# Bad: Parse plain text
grep "^#" file.md
```

### Handle Errors Gracefully

```bash
# Always check command success
RESULT=$(agent-md read file.md 2>/dev/null)
if [ $? -eq 0 ]; then
    # Process successful result
    echo "$RESULT" | jq '.content'
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

for file in $(agent-md list . | jq -r '.[]'); do
    # Search for TODOs
    todos=$(agent-md search "$file" "TODO")

    if echo "$todos" | jq -e '.total > 0' > /dev/null; then
        echo "Found TODOs in $file:"
        echo "$todos" | jq '.matches[] | .content'

        # Process each TODO...
    fi
done
```

### Document Processing Pipeline

```bash
# 1. Extract structure
structure=$(agent-md headings document.md)

# 2. Validate content
validation=$(agent-md lint document.md)

# 3. Convert to structured format
jsonl=$(agent-md to-jsonl document.md)

# 4. Process based on validation
if echo "$validation" | jq -e '.valid' > /dev/null; then
    echo "Document is valid, processing..."
    # Continue with processing
else
    echo "Document has issues:"
    echo "$validation" | jq '.errors'
fi
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
# Build document index
declare -A doc_cache
for file in $(agent-md list . | jq -r '.[]'); do
    doc_cache["$file"]=$(agent-md read "$file")
done

# Search across all documents
for file in "${!doc_cache[@]}"; do
    content=$(echo "${doc_cache[$file]}" | jq -r '.content')
    if [[ "$content" == *"$search_term"* ]]; then
        echo "Found in $file"
        agent-md search "$file" "$search_term"
    fi
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
