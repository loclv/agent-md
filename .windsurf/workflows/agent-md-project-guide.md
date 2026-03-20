---
description: Comprehensive guide to the agent-md Rust project
---

# Agent-MD Project Skill

## Overview

The agent-md project is a Rust-based Markdown editor specifically designed for AI agents. It provides command-line tools for reading, writing, editing, and validating Markdown files with JSON output capabilities.

## Project Structure

```text
ralph-md/
├── .github/workflows/
│   └── ci.yml                 # GitHub Actions CI/CD pipeline
├── .vscode/
│   ├── settings.json          # VS Code configuration
│   └── tasks.json             # VS Code tasks
├── src/
│   └── main.rs                # Main application source code
├── Cargo.toml                 # Rust project configuration
├── Cargo.lock                 # Dependency lock file
├── Makefile                   # Development commands
├── README.md                  # Project documentation
├── LINTING.md                 # Linting setup guide
├── rust-toolchain.toml        # Rust version specification
├── rustfmt.toml               # Code formatting configuration
├── clippy.toml                # Clippy linting configuration
└── .pre-commit-config.yaml    # Pre-commit hooks (optional)
```

## Core Components

### 1. Main Application (`src/main.rs`)

- **CLI Interface**: Uses clap for command-line parsing
- **Markdown Parser**: Uses pulldown-cmark for parsing Markdown
- **JSON Serialization**: Uses serde for JSON output
- **Validation**: Built-in linting for Markdown content
- **Commands**: Read, Write, Append, Insert, Delete, List, Search, Headings, Stats, ToJsonl, Lint

### 2. Configuration Files

- **Cargo.toml**: Dependencies, binary configuration, build profiles
- **rust-toolchain.toml**: Specifies Rust 1.94.0 with rustfmt and clippy
- **rustfmt.toml**: Code formatting rules (100-char width, 4-space indent)
- **clippy.toml**: Linting rules (complexity limits, documentation requirements)

### 3. Development Tools

- **Makefile**: Convenient commands for building, testing, linting
- **VS Code Integration**: Tasks and settings for IDE development
- **GitHub Actions**: Automated CI/CD pipeline
- **Pre-commit Hooks**: Optional local validation

## Available Commands

### Core Operations

```bash
# Read a Markdown file (outputs JSON)
agent-md read <file>

# Write content to a Markdown file (validates first)
agent-md write <file> <content>

# Append content to a Markdown file
agent-md append <file> <content>

# Insert content at a specific line
agent-md insert <file> <line> <content>

# Delete lines from a Markdown file
agent-md delete <file> <line> [count]

# List Markdown files in directory
agent-md list [directory]

# Search for text in Markdown file
agent-md search <file> <query>
```

### Analysis Commands

```bash
# Extract headings from Markdown file
agent-md headings <file>

# Get file statistics (word count, line count, headings)
agent-md stats <file>

# Convert Markdown to JSONL format
agent-md to-jsonl <file>

# Validate Markdown content (linting)
agent-md lint <file> [--content]
```

### Development Commands

```bash
# Using Makefile (recommended)
make help          # Show all commands
make build         # Build release version
make test          # Run tests
make lint          # Run all linting checks
make format        # Format code
make clippy        # Run clippy lints
make ci            # Full CI pipeline

# Using Cargo directly
cargo build --release
cargo test
cargo fmt
cargo clippy
```

## Data Structures

### Document Output

```json
{
  "path": "string",
  "content": "string", 
  "word_count": "number",
  "line_count": "number",
  "headings": [
    {
      "level": "number",
      "text": "string",
      "line": "number"
    }
  ]
}
```

### Lint Result

```json
{
  "valid": "boolean",
  "errors": [
    {
      "line": "number",
      "column": "number", 
      "message": "string",
      "rule": "string"
    }
  ],
  "warnings": [
    {
      "line": "number",
      "column": "number",
      "message": "string", 
      "rule": "string"
    }
  ]
}
```

### JSONL Entry

```json
{
  "type": "heading|paragraph|code",
  "content": "string",
  "level": "number|null",
  "language": "string|null"
}
```

## Usage Examples

### Basic File Operations

```bash
# Read file with JSON output
agent-md read README.md

# Write new content (validates first)
agent-md write new.md "# Title\n\nContent here"

# Append to existing file
agent-md append README.md "\n## New Section\n\nMore content"

# Insert at line 5
agent-md insert README.md 5 "## Inserted Section\n\nContent"

# Delete line 10
agent-md delete README.md 10

# Delete 3 lines starting at line 5
agent-md delete README.md 5 3
```

### Analysis and Search

```bash
# Get all headings
agent-md headings README.md

# Get file statistics
agent-md stats README.md

# Search for text
agent-md search README.md "function"

# List all Markdown files
agent-md list .

# Convert to JSONL
agent-md to-jsonl README.md > output.jsonl
```

### Validation

```bash
# Validate file
agent-md lint README.md

# Validate content directly
agent-md lint "# Title\n\nContent" --content
```

## Development Workflow

### 1. Setup Development Environment

```bash
# Install Rust components
make setup

# Or manually:
rustup component add rustfmt clippy
cargo install cargo-watch cargo-audit
```

### 2. Make Changes

```bash
# Edit source files
vim src/main.rs

# Format code
make format

# Run linting
make lint

# Run tests
make test

# Build and test
make build && ./target/release/agent-md --help
```

### 3. Validation Pipeline

```bash
# Full CI pipeline locally
make ci

# Individual checks
make test          # Run tests
make clippy        # Linting checks
make check-format  # Format validation
```

## Linting and Code Quality

### Clippy Rules

- Cognitive complexity ≤ 30
- Function arguments ≤ 7
- Type complexity ≤ 250
- Documentation required for public items
- No unwrap/expect in production code

### Formatting Rules

- Maximum line width: 100 characters
- 4 spaces for indentation
- Vertical trailing commas
- Reordered imports

### Pre-commit Hooks (Optional)

```bash
# Install
pip install pre-commit
pre-commit install

# Run manually
pre-commit run --all-files
```

## Dependencies

### Core Dependencies

- **clap 4.5**: Command-line argument parsing
- **pulldown-cmark 0.12**: Markdown parsing
- **serde 1.0**: JSON serialization
- **dirs 5.0**: Directory handling
- **chrono 0.4**: Date/time utilities

### Development Dependencies

- **cargo-watch**: File watching for development
- **cargo-audit**: Security vulnerability checking

## Build Configuration

### Release Profile

```toml
[profile.release]
strip = true        # Strip debug symbols
lto = true         # Link-time optimization
opt-level = "z"    # Optimize for size
```

### Binary Configuration

```toml
[[bin]]
name = "agent-md"
path = "src/main.rs"
```

## CI/CD Pipeline

### GitHub Actions Workflow

- **Test**: Runs unit tests on Ubuntu
- **Lint**: Checks formatting and runs clippy
- **Security**: Runs cargo audit
- **Build**: Creates release binary
- **Artifact**: Uploads binary as GitHub artifact

### Pipeline Steps

1. Checkout code
2. Install Rust toolchain
3. Cache dependencies
4. Check formatting
5. Run clippy
6. Run tests
7. Security audit
8. Build release
9. Upload artifact

## Troubleshooting

### Common Issues

1. **Build fails**: Check Rust version in `rust-toolchain.toml`
2. **Formatting errors**: Run `make format` to fix
3. **Clippy warnings**: Address linting issues in code
4. **Test failures**: Check test output for specific errors

### Getting Help

```bash
# Command help
agent-md --help
agent-md <command> --help

# Makefile help
make help

# Rust tool help
cargo --help
cargo fmt --help
cargo clippy --help
```

### Debug Mode

```bash
# Build with debug info
cargo build

# Run tests with output
cargo test -- --nocapture

# Verbose clippy output
cargo clippy --verbose
```

## Contributing

### Code Style

- Follow rustfmt formatting
- Address all clippy warnings
- Add tests for new features
- Update documentation

### Testing

```bash
# Run all tests
make test

# Run specific test
cargo test test_name

# Run with coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html
```

### Documentation

- Update README.md for user-facing changes
- Update LINTING.md for development changes
- Add inline documentation for public APIs
