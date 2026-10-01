# LLM Agent Rule: Use agent-md CLI

## Core Rule

When working with markdown files, always use the `agent-md` CLI tool instead of direct file operations.

## Why Use agent-md

- JSON output: All commands return structured JSON for easy parsing
- Built-in validation: Automatic markdown validation ensures AI-friendly content
- LLM-optimized: Designed specifically for AI agent workflows
- Consistent formatting: Enforces markdown standards for readability and token savings

## Core Commands

### Read Files and Extract Fields

```bash
# Read whole document
agent-md read <path>

# Extract specific field (path, content, word_count, line_count, headings)
agent-md read <path> --field content
agent-md read <path> --field headings
agent-md read <path> -f word_count

# Read specific section without reading entire file
agent-md read <path> --content "## Section Name"
# Nested section:
agent-md read <path> --content "## Development > Build"
```

### Write Files and Sections

```bash
# Write file with pre-validation
agent-md write <path> "<content>"

# Write or replace a specific section
agent-md write-section <path> --section "## Section" --content "<content>"

# Append content to file
agent-md append <path> "<content>"

# Insert content at line number
agent-md insert <path> <line> "<content>"

# Delete lines
agent-md delete <path> <line> [count]
```

### Search and Navigation

```bash
# Search within file
agent-md search <path> "<query>"

# Extract document outline
agent-md headings <path>

# Get document statistics
agent-md stats <path>
```

### Validate and Format

```bash
# Validate markdown file
agent-md lint <path>

# Validate content directly without creating a file
agent-md lint --content "<content>"

# Format markdown file in-place
agent-md fmt <path>
```

### Project and Configuration Context

```bash
# Check resolved configuration
agent-md config --check

# Inspect ignore patterns from .markdownlintignore and .gitignore
agent-md ignore

# Run commands with configuration from a specific project directory
agent-md --cwd /path/to/project lint <path>
agent-md --cwd /path/to/project fmt <path>
```

## Integration Workflow

When performing agentic edits on Markdown files, follow this pattern:

```bash
# 1. Inspect document structure
agent-md read README.md --field headings

# 2. Search for relevant sections or keywords
agent-md search README.md "TODO"

# 3. Read only the relevant section
agent-md read README.md --content "## Installation"

# 4. Validate before writing
agent-md lint --content "$NEW_CONTENT"

# 5. Write validated content
agent-md write README.md "$NEW_CONTENT"
```

## Task Execution Requirements

On task completion, AI agents must:

- Update documentation in the `docs/` directory
- Update `README.md`
- Write unit tests for changes
- Run linting: `cargo clippy --all-targets --all-features -- -D warnings`
- Run formatter: `cargo fmt`
- Run tests: `cargo test`
- Run `agent-md lint path/to/file.md` on every created or updated Markdown file
