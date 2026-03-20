# Agent MD - Markdown Editor for AI Agents

A CLI markdown editor written in Rust designed for LLMs to easily read and use.

## Installation

Build from source:

- Install Rust first if not already installed
- Then build the release version:

```bash
cargo build --release
# Binary at target/release/agent-md
```

- Add to PATH (optional):

```bash
# agent-md command
export PATH="/Users/username/w/agent-md/target/release:$PATH"
```

Now you can use the `agent-md` command from anywhere.

## Commands (LLM-friendly JSON output)

All commands return JSON for easy parsing.

### Read a file

```bash
agent-md read <path>
# Returns: {path, content, word_count, line_count, headings}
```

### Write a file

```bash
agent-md write <path> <content>
# Returns: {success, message, document}
```

### Append to a file

```bash
agent-md append <path> <content>
# Returns: {success, message, document}
```

### Insert at line

```bash
agent-md insert <path> <line> <content>
# Returns: {success, message, document}
```

### Delete lines

```bash
agent-md delete <path> <line> [count]
# Returns: {success, message, document}
```

### List markdown files

```bash
agent-md list <directory>
# Returns: [file paths...]
```

### Search in file

```bash
agent-md search <path> <query>
# Returns: {query, matches: [{line, content}], total}
```

### Get headings

```bash
agent-md headings <path>
# Returns: [{level, text, line}...]
```

### Get stats

```bash
agent-md stats <path>
# Returns: {path, word_count, line_count, heading_count}
```

### Convert to JSONL

```bash
agent-md to-jsonl <path>
# Returns: JSONL lines with {type, content, level, language}
```

### Lint/Validate markdown

```bash
agent-md lint <path>
# Returns: {valid, errors: [{line, column, message, rule}], warnings: [{line, column, message, rule}]}

agent-md lint --content "# Markdown content"
# Validate content directly without file

agent-md lint-file <path>
# Returns: Human-readable linting output with errors, warnings, and summary
```

## Validation Rules

The linter enforces AI-friendly markdown standards.

### Error Rules (block content)

- No bold text: `**bold**` and `__bold__` are rejected (errors)

- Heading structure: Multiple H1 headings and skipped heading levels are rejected (errors)

- Table syntax: Complex table attributes and incorrect separator format are rejected (errors)

### Warning Rules (style guidelines)

- Simple table syntax: Very wide tables and inline formatting in cells generate warnings

- No useless links: Links where text equals the URL generate warnings

- No ASCII graphs: Box drawing characters and visual patterns generate warnings

- Code block best practices: Code blocks without language specification generate warnings

- List formatting: Inconsistent list markers and numbering generate warnings

### Automatic validation

The `write` command validates content before writing to ensure AI-friendly markdown.

## Running Tests

```bash
cargo test
```

## Rule for LLMs/Agents: Use agent-md CLI

When working with markdown files, always prefer the `agent-md` CLI tool over direct file editing.

See `docs/llm-agent-rule.md` for the complete integration guideline.

### Why use agent-md?

- JSON output: All commands return structured JSON for easy parsing
- Built-in validation: Automatic markdown validation ensures AI-friendly content
- LLM-optimized: Designed specifically for AI agent workflows
- Consistent formatting: Enforces markdown standards for better readability

### Core Usage Pattern

```bash
# Always read files with agent-md (returns JSON)
DATA=$(agent-md read <path>)
CONTENT=$(echo "$DATA" | jq -r '.content')

# Always write files with agent-md (validates first)
agent-md write <path> "<content>"

# Search within files
agent-md search <path> "<query>"

# Get document structure
agent-md headings <path>
```

### Integration Rule

RULE: When you need to read, write, or modify markdown files, use `agent-md` commands instead of direct file operations. The tool provides structured data and ensures content quality.

### Example Workflow

```bash
# 1. Read existing file
DOC=$(agent-md read README.md)
echo "$DOC" | jq '.headings'  # Get structure

# 2. Search for specific content
agent-md search README.md "TODO"

# 3. Validate new content
agent-md lint --content "# New Title\nContent here"

# 4. Write validated content
agent-md write README.md "# New Title\nValid content"
```

## Example Usage for LLMs

```bash
# Read a file and get structured data
DATA=$(agent-md read /path/to/file.md)
echo "$DATA" | jq '.word_count'

# Search for content
agent-md search /path/to/file.md "TODO"

# Get all headings for navigation
agent-md headings /path/to/file.md

# Lint a file
agent-md lint README.md

# Lint with human-readable output
agent-md lint-file README.md

# Validate markdown before writing
agent-md lint --content "# Title\nContent with **bold** text"
agent-md write document.md "# Title\nValid content without bold"
```
