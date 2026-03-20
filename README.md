# Agent MD - Markdown Editor for AI Agents

A CLI markdown editor written in Rust designed for LLMs to easily read and use.

## Installation

```bash
cargo build --release
# Binary at target/release/agent-md
```

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

The linter enforces AI-friendly markdown:

- No bold text: `**bold**` and `__bold__` are rejected (errors)
- Simple table syntax: Complex table attributes are rejected, very wide tables generate warnings
- No useless links: Links where text equals the URL generate warnings
- No ASCII graphs: Box drawing characters and visual patterns generate warnings
- Single H1 title: Multiple H1 headings are rejected (errors)
- Automatic validation: The `write` command validates content before writing

## Running Tests

```bash
cargo test
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
