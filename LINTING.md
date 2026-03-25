# Linting and Development Tools

This project includes a comprehensive linting setup to maintain code quality and consistency.

## Available Linting Tools

### 1. Rustfmt

- Purpose: Code formatting and style consistency
- Configuration: `rustfmt.toml`
- Usage:

  ```bash
  cargo fmt # Format code
  cargo fmt --check # Check if code is formatted
  ```

### 2. Clippy

- Purpose: Rust linter for catching common mistakes and improving code
- Configuration: `clippy.toml`
- Usage:

  ```bash
  cargo clippy # Run clippy
  cargo clippy --all-targets --all-features -- -D warnings # Strict mode
  ```

### 3. Make Commands

The project includes a Makefile for convenient commands:

```bash
make help # Show all available commands
make lint # Run all linting checks (clippy + format check)
make format # Format code with rustfmt
make check-format # Check if code is formatted
make clippy # Run clippy lints
make test # Run tests
make ci # Full CI pipeline (test + lint + audit)
```

### 4. VS Code Integration

- Tasks: `.vscode/tasks.json` provides tasks for VS Code
- Settings: `.vscode/settings.json` configures rust-analyzer and formatting
- Usage: Run tasks via Command Palette (Ctrl+Shift+P) → "Tasks: Run Task"

### 5. Pre-commit Hooks (Optional)

Install pre-commit hooks to automatically run linting before commits:

```bash
pip install pre-commit
pre-commit install
```

## Configuration Files

- `rust-toolchain.toml`: Specifies Rust version and components
- `rustfmt.toml`: Rustfmt configuration for code formatting
- `clippy.toml`: Clippy configuration for linting rules
- `Makefile`: Convenient commands for development
- `.vscode/settings.json`: VS Code configuration
- `.vscode/tasks.json`: VS Code tasks
- `.github/workflows/ci.yml`: GitHub Actions CI/CD pipeline

## Development Workflow

1. Write code: Make your changes
2. Format: `make format` or `cargo fmt`
3. Lint: `make clippy` or `cargo clippy`
4. Test: `make test` or `cargo test`
5. Full check: `make lint` (runs both formatting check and clippy)

## CI/CD Pipeline

The GitHub Actions workflow (`.github/workflows/ci.yml`) runs:

- Code formatting check
- Clippy linting
- Unit tests
- Security audit (cargo audit)

## Linting Rules

### Clippy Configuration

- Cognitive complexity threshold: 30
- Too many arguments threshold: 7
- Type complexity threshold: 250
- Trivial copy size limit: 64 bytes
- Pass-by-value size limit: 256 bytes
- Documentation required for crate items

### Rustfmt Configuration

- Maximum line width: 100 characters
- 4 spaces for indentation
- Vertical trailing commas
- Reorder imports enabled

### Markdown Linting Rules

The agent-md tool includes custom linting rules for Markdown content:

#### no-bold

- Type: Error
- Description: Bold text is not allowed for AI agents
- Detection: Detects `**text**` or `__text__` patterns

#### simple-tables

- Type: Error/Warning
- Description: Enforces simple table syntax
- Detection:
  - Error: Complex table attributes (colspan/rowspan)
  - Warning: Inline formatting in table cells, very wide tables (>5 columns)

#### useless-links

- Type: Error
- Description: Link text should not be the same as the URL
- Detection: Finds links where the display text equals the URL (with common prefixes stripped)

#### no-ascii-graph

- Type: Error
- Description: Human-readable ASCII Graph detected
- Detection: Identifies ASCII art, box drawing characters, tree structures, flow charts, and high-density special character patterns. Applies to ALL content including code blocks
- Recommendation: Use LLM-readable formats instead:
  - Structured CSV
  - JSON
  - Mermaid Diagram
  - Numbered List with Conditions
  - ZON format

#### table-trailing-spaces

- Type: Error
- Description: Table cells must not have more than 1 trailing space
- Detection: Checks table data rows (ignores separator rows and code blocks). Cells with 2 or more trailing spaces are rejected
- Auto-fix: Use `agent-md fmt <path>` to automatically trim leading and trailing spaces from all table cells

#### single-title

- Type: Error
- Description: Multiple H1 headings detected
- Detection: Finds documents with more than one top-level heading (# Title)
- Recommendation: Use only one H1 heading per document for clear document structure

## Adding New Dependencies

When adding new dependencies:

1. Add them to `Cargo.toml`
2. Run `cargo check` to verify compilation
3. Run `cargo clippy` to check for new linting issues
4. Run `cargo test` to ensure tests still pass

## Troubleshooting

### Common Issues

1. "Unstable features" warnings: Some rustfmt options require nightly Rust
2. Clippy configuration errors: Check `clippy.toml` for valid field names
3. Formatting conflicts: Use `cargo fmt` to auto-fix most issues

### Getting Help

- `cargo fmt --help`: Rustfmt help
- `cargo clippy --help`: Clippy help
- `make help`: Available make commands
