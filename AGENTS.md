# Agent-MD Project Rules

These rules apply when working with the agent-md project.

## Project Overview

agent-md is a Rust-based Markdown editor designed for AI agents. It provides CLI tools for reading, writing, editing, and validating Markdown files with JSON output.

## Task Execution Process

For any development task:

1. Receive the request - Understand requirements and scope
2. Break down complex requests - Write tasks to `tasks.md` file
3. Execute each task - Mark completed tasks in `tasks.md`
4. Validate completion:
   - Run linting: `make lint`
   - Run formatter: `make format`
   - Run tests: `make test`

After creating or updating any markdown file, always run `agent-md lint path/to/file.md` to validate the content before considering the task complete.

## Markdown Writing Rules

When writing Markdown files in this project, follow these critical rules:

- No bold text: Double asterisks and double underscores are NOT allowed
- Use italics instead of bold for emphasis
- Use headings for structure
- Use code formatting for technical terms with backticks

Additional rules enforced by agent-md linter:

- Simple table syntax (5 columns or fewer, 3-dash separators)
- No useless links (link text must differ from URL)
- No ASCII graphs (use JSON, CSV, or Mermaid)
- Proper heading hierarchy (no level skipping)
- Code blocks must specify language
- No duplicate headings
- Single H1 title per document

## Available Commands

```bash
make help # Show all commands
make build # Build release version
make test # Run tests
make lint # Run all linting checks
make format # Format code
make clippy # Run clippy lints
make ci # Full CI pipeline

cargo build --release
cargo test
cargo fmt
cargo clippy
```

## Code Quality Standards

### Clippy Rules

- Cognitive complexity 30 or less
- Function arguments 7 or fewer
- Type complexity 250 or less
- Documentation required for public items
- No unwrap/expect in production code

### Formatting Rules

- Maximum line width: 100 characters
- 4 spaces for indentation
- Vertical trailing commas
- Reordered imports

## Troubleshooting

- Build fails: Check Rust version in `rust-toolchain.toml` (requires 1.94.0)
- Formatting errors: Run `make format`
- Clippy warnings: Address linting issues in code
- Test failures: Check test output for specific errors

## Detailed Documentation

For comprehensive project information, use the `agent-md` skill when available.
