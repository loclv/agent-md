# CLI Usage Guide

`agent-md` provides a powerful CLI for both human and AI workflows. It supports three main modes: linting, formatting, and AI-assisted writing.

## Specific Usage Example

### Input: Regular Markdown

This is a standard markdown file that many LLMs generate:

```markdown
# My Project

## Overview

This is a really awesome project with many outstanding features:

| Feature | Description | Status |
|---|---|---|
| API | Complete RESTful API | ✅ Complete |
| UI | Modern user interface | 🚧 In Progress |
| Tests | Unit tests and integration tests | ✅ Complete |

### Steps to Follow

1. Clone repository
2. Install dependencies: `bun i`
3. Run server: `bun dev`

>Note: Make sure you have Node.js 22+ installed!
```

### Output: AI-friendly Markdown via agent-md

After processing with agent-md, the content becomes cleaner:

```markdown
# My Project

## Overview

This is a really awesome project with many outstanding features:

- API: Complete RESTful API (Complete)
- UI: Modern user interface (In Progress)
- Tests: Unit tests and integration tests (Complete)

## Steps to Follow

1. Clone repository
2. Install dependencies: `bun i`
3. Run server: `bun dev`

Note: Make sure you have Node.js 22+ installed.
```

### Efficiency Comparison

```bash
# Lint regular markdown file
agent-md lint regular-markdown.md
# {"valid":false,"errors":[
# {"line":3,"message":"No bold text allowed","rule":"no-bold"},
# {"line":6,"message":"Complex table detected","rule":"simple-table"},
# {"line":15,"message":"No bold text allowed","rule":"no-bold"}
# ]}

# Lint agent-md file
agent-md lint agent-md-markdown.md
# {"valid":true,"errors":[],"warnings":[]}
```

Benefits:

- In some cases, reduces ~20% unnecessary tokens
- Faster LLM reading and processing
- Maintains full information content
- Easy section extraction

## Add a rule or command for LLM/Agents to follow

For example, when adding a new rule, add it to the `AGENTS.md` file.
>After creating or updating any markdown file, always run `agent-md lint path/to/file.md` to validate the content before considering the task complete.
>
>On task completion, you must update documentation (`docs/`), update `README.md`, and write unit tests for your changes.

Or tell LLM/Agents:
>Use `agent-md` CLI to run lint

## Commands (LLM-friendly JSON output)

All commands return JSON for easy parsing. Use `--human` flag before the subcommand for pretty-printed output:

```bash
agent-md --human lint README.md
agent-md --human read README.md --field headings
```

### Read a file

```bash
agent-md read <path>
# Returns: {path, content, word_count, line_count, headings}

# Extract specific field
agent-md read <path> --field <field_name>
# Available fields: path, content, word_count, line_count, headings

# Read specific section by heading path (no need to read entire file)
agent-md read <path> --content <section_path>
# Example: agent-md read README.md --content "## Development"
# Nested sections: agent-md read README.md --content "## Development > Build"
```

### Write a file

```bash
agent-md write <path> <content>
# Returns: {success, message, document}
```

### Write to a specific section

```bash
agent-md write-section <path> --section <heading_path> --content <content>
# Replaces existing section content or creates new section
# Example: agent-md write-section README.md --section "## Development" --content "New content"
# Nested sections: agent-md write-section README.md --section "## Development > Build" --content "New content"
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

Lists Markdown files in the specified directory (defaults to `.`). Automatically excludes files and directories matching `.markdownlintignore` and `.gitignore`.

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

### Working directory for configuration

Use the global `--cwd` option to specify a working directory for configuration discovery. This allows you to read configuration files from a different directory than the current working directory.

```bash
agent-md --cwd <path> <command> <args>
# Reads configuration files from the specified directory

agent-md --cwd /path/to/project lint document.md
# Lints document.md using configuration from /path/to/project

agent-md --cwd /path/to/project fmt document.md
# Formats document.md using configuration from /path/to/project
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

### Initialize configuration

Initializes a default `.agent-md.json` configuration file in the current directory or a specified path.

```bash
agent-md init
# Initializes .agent-md.json in the current directory

agent-md init custom.json
# Initializes configuration at a specific file path

agent-md init path/to/dir
# Initializes .agent-md.json inside the specified directory

agent-md init --force
# Overwrites existing configuration file (-f also supported)

agent-md config --init
# Alternative via config subcommand with optional --force flag
```

### Check configuration

Checks configuration file resolution and status. Configuration files are resolved in the following priority order:

1. `.agent-md.json`
2. `agent-md.json`
3. `markdownlintrc.*` fallback (`.markdownlint.json`, `.markdownlint.jsonc`, `.markdownlint.yaml`, `.markdownlint.yml`, `.markdownlintrc`, `.markdownlintrc.json`)

When processing a target Markdown file, `agent-md` automatically searches the file's parent directory and walks up parent directories to discover configuration, falling back to the current working directory unless overridden with `--config` or `--cwd`.

Use global `--cwd <path>` to specify a working directory for configuration discovery. This is useful when you want to read configuration files from a different directory than the current working directory.

Use global `--ignore-markdownlintrc` to skip `markdownlintrc.*` files during discovery (default is `false`, so they are used as fallback). The same option can be set in `agent-md.json` via the `ignore-markdownlintrc` (or `ignore_markdownlintrc`) key; the CLI flag always wins, and only native `agent-md.json` files are consulted for this key.

A sample configuration file is provided in `samples/.agent-md.json`.

```bash
agent-md config
# Returns: {exists, path, config}

agent-md config --check
# Returns: {exists, path}

agent-md config samples
# Inspect configuration in directory

agent-md config path/to/document.md
# Inspect configuration resolved for specific file

agent-md config custom.json
# Inspect specific configuration path

agent-md --config samples/.agent-md.json fmt document.md
# Use specific configuration file via global --config flag

agent-md --ignore-markdownlintrc lint document.md
# Skip markdownlintrc.* files and use only agent-md configuration

# Or set it in agent-md.json:
# { "ignore-markdownlintrc": true }

agent-md --cwd /path/to/project lint document.md
# Read configuration files from /path/to/project directory

agent-md --cwd /path/to/project fmt document.md
# Format document.md using configuration from /path/to/project
```

### Inspect ignore rules

Reads `.markdownlintignore` if present in the target directory, merges it with the current Git ignore list (`.gitignore`), and outputs deduplicated ignore patterns:

```bash
agent-md ignore
# Returns: ["dist/", "logs/", "target/", "/target", "temp/", "*.log", "*.tgz", ".antigravitycli"]

agent-md ignore path/to/dir
# Returns ignore patterns for specified directory

agent-md --human ignore
# Pretty-printed JSON array
```

Commands like `list` and directory-level `fmt` automatically respect these ignore rules to skip ignored paths and avoid processing build artifacts or temporary files.

### Format markdown

- Formats the markdown file in-place, trimming leading and trailing spaces from table cells.
- Preserves escaped pipes (`\|`) and pipes inside inline code spans in table cells without splitting cells or inserting extra spaces.
- Formats empty table cells with a single space (`| |`) to maintain clean table structure.
- Normalizes table separator rows (e.g., `|:---|:---|` becomes `|---|---|`), removing alignment colons to save tokens.
- Removes trailing colons from headings (e.g., `## header:` becomes `## header`).
- Automatically appends `text` as default language tag for unlabelled code blocks (e.g., ` ``` ` becomes ` ```text `), including code fences nested inside list items.
- Removes empty list items (e.g., a trailing `- ` line with no content).
- Preserves code block content, including relative indentation and syntax inside nested code blocks within list items.
- Collapses multiple spaces before `#` comments in shell code blocks (`bash`, `sh`, `shell`, `zsh`).
- Formats folder structures in `text`, `txt`, or unlabelled code blocks by removing redundant `─` dashes (e.g., `├──` to `├─`, `└──` to `└─`), stripping spacer lines (`│`), removing spaces before file names, normalizing 4-space indentation to 2 spaces for nested branches, and collapsing spaces before comments.
- Automatically converts 4 leading spaces of list item indentation to 2 spaces, and 2 leading tabs to 1 tab for sub-items, reducing token usage in nested lists.
- Respects `.markdownlintignore` and `.gitignore` when formatting directories, skipping ignored paths (like `target/`, `dist/`, `logs/`) and temporary files.

```bash
agent-md fmt <path>
# Returns JSON data: {success, message, document}
```

#### Code Block Comment Formatting

For code blocks, the formatter automatically collapses excessive spaces before `#` comments while preserving indentation:
~~~text
After:

```bash
echo hello # this is a comment
    # indented comment
```

~~~

#### Folder Structure Formatting

For code blocks with language `text`, `txt`, or empty language tag containing folder structures, the formatter compacts branch markers to single-dash prefixes (`├─` and `└─`), removes vertical spacer lines (`│`), and aligns comments:

~~~text

```text
data/
├─input/ # input
├─output/ # output
└─logs/ # logs
```

~~~

#### Inline Code Block Preservation

Inline code blocks enclosed in backticks are preserved as-is without any formatting modifications:

~~~text
code block: `let a = 1;`
~~~

The output remains unchanged.

#### Format Options

The formatter applies compact rules by default to reduce token count. These can be configured in `.agent-md.json` or `agent-md.json` (using kebab-case or snake_case, top-level or under a `format` object) and overridden via CLI flags:
| Option | Description | Default |
|---|---|---|
| `remove_bold` | Removes `**bold**` and `__bold__` markers (preserves code spans, URLs, and link destinations; configurable via `remove_bold` or `remove-bold`) | `true` |
| `compact_blank_lines` | Collapses multiple consecutive blank lines (preserves single blank lines around headings) | `true` |
| `collapse_spaces` | Collapses multiple spaces between words (preserves spaces inside inline code) | `true` |
| `remove_horizontal_rules` | Removes `---`, `***`, `___` lines | `true` |
| `remove_emphasis` | Removes `*italic*` and `_italic_` markers (preserves code spans, URLs, and link destinations) | `true` |
| `blanks_around_lists` | Ensures lists are surrounded by blank lines (configured in `.agent-md.json` or `.markdownlint.json`) | `true` |
| `blanks_around_fences` | Ensures fenced code blocks are surrounded by blank lines (configured in `.agent-md.json` or `.markdownlint.json`) | `true` |
| `blanks_around_headings` | Ensures headings are surrounded by blank lines (configured in `.agent-md.json` or `.markdownlint.json`) | `true` |
| `minify_html` | Minifies HTML tags and blocks by removing useless whitespace and newlines | `true` |

Example:

```bash
agent-md fmt document.md
agent-md fmt --remove-bold=false document.md
```

##### HTML Minification

HTML blocks and tags are minified to eliminate token waste:

- Removes useless spaces, tabs, and newlines inside HTML tags
- Strips redundant indentation inside HTML blocks
- Collapses each HTML block onto a single line, joining tag boundaries directly and text lines with a single space
- Keeps line breaks inside `pre`, `code`, `textarea`, `script`, and `style` blocks where whitespace is significant
- Preserves original HTML tag names, attributes, and attribute values intact
- Preserves Markdown autolinks and pure URLs (e.g., `<https://...>`, `<user@example.com>`, `<example.dev/path/>`) untouched
- Set `minify-html` (or `minify_html`) to `false` in `agent-md.json`, or pass `--minify-html=false`, to leave HTML blocks unchanged

Input:

```html
<p align="center">
  <img src="badge.png" alt="Markdown" />
</p>
```

Output:

```html
<p align="center"><img src="badge.png" alt="Markdown" /></p>
```

## How it Works: Structured Parsing

Unlike simple line-based formatters, `agent-md` uses a structured parser that:

1. Extracts YAML Frontmatter: Preserves metadata at the beginning of the document exactly as-is.
2. Identifies Document Blocks: Recognizes headings, code blocks, tables, lists, HTML blocks, and paragraphs.
3. Applies Context-Aware Formatting: Formats each block according to its type and your configuration options.
4. Optimizes for LLMs: Ensures the output is clean, consistent, and token-efficient while remaining human-readable.
5. Hardware SIMD Acceleration: Employs cross-platform vectorization (ARM NEON and x86_64 AVX2/SSE2) for high-throughput newline counting, search scanning, and early-exit rule checks.

## Validation Rules

The linter enforces AI-friendly markdown standards.

### Error Rules (block content)

- "no-bold": No bold text: `**bold**` and `__bold__` are rejected, except in code blocks
- "heading-structure": Heading structure: Multiple H1 headings and skipped heading levels are rejected
- "table-syntax": Table syntax: Complex table attributes and incorrect separator format are rejected
- "simple-table-syntax": Simple table syntax: Very wide tables and inline formatting in cells are rejected
- "table-trailing-spaces": Table trailing spaces: Table cells with more than 1 trailing space are rejected
- "no-ascii-graphs": No ASCII graphs: Box drawing characters and visual patterns are rejected, even in code blocks
- "code-blocks": Code block validation: Code blocks without language specification or with missing closing fences (unclosed code blocks) are rejected, with unclosed blocks immediately aborting formatting and linting
- "list-formatting": List formatting: Inconsistent list markers and numbering are rejected
- "space-indentation": Space indentation: Excessive indentation (more than 2 spaces) in regular text is rejected (code blocks exempt)
- "no-useless-links": No useless links: Links where text equals the URL are rejected

### Warning Rules (style guidelines)

- "no-duplicate-headings": No duplicate headings: Headings with same content are warned
- "no-multiple-blanks": No multiple blank lines: Multiple consecutive blank lines are warned

Detail at [docs/markdown-writing-rules.md](./markdown-writing-rules.md)

### Automatic validation

The `write` command validates content before writing to ensure AI-friendly markdown.
