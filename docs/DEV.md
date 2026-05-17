# Development

Development setup and guidelines for contributing to agent-md.

## Prerequisites

- Rust 1.94.0 or later (specified in `rust-toolchain.toml`)
- Git
- Make (optional, for convenience commands)

## Project Structure

```text
agent-md/
├── src/
│   ├── main.rs # Main application logic and CLI
│   ├── parser.rs # Structured Markdown parser (block-based)
│   ├── format/ # Formatting modules
│   │   ├── mod.rs # Structured formatter orchestration
│   │   ├── tables.rs # Table formatting (separator compaction, row formatting)
│   │   ├── bold_tables.rs # Bold stripping from table cells
│   │   ├── blockquotes.rs
│   │   ├── code_blocks.rs
│   │   └── frontmatter.rs
│   ├── rules/ # Validation rule modules
│   ├── tests.rs # Core unit tests
│   └── html_tests.rs # HTML rendering tests
├── docs/ # Documentation
├── test-md/ # Test markdown files
└── Makefile # Convenience commands
```

## Architecture: Structured Parsing and Formatting

`agent-md` follows a **Parse-then-Format** architecture:

1.  **Parsing**: The `src/parser.rs` module decomposes the raw Markdown text into a sequence of `MarkdownBlock` elements (e.g., `Heading`, `CodeBlock`, `List`, `Table`). This stage also extracts YAML frontmatter.
2.  **Formatting**: The `format_markdown_structured` function in `src/format/mod.rs` iterates over these blocks and applies formatting rules based on the block type and user configuration.

This approach is more robust than simple line-based processing, especially for complex structures like nested lists or tables.

## Building

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Using Makefile
make build
```

## Testing

The project uses inline tests within each module:

```bash
# Run all tests
cargo test

# Run specific module tests
cargo test format::tables::tests
cargo test format::bold_tables::tests
cargo test rules::code_blocks::tests

# Run specific test
cargo test test_validate_space_indentation

# Run tests with output
cargo test -- --nocapture

# Using Makefile
make test
```

### Test Coverage

- 400+ unit tests across all validation and formatting modules
- Integration tests for complete workflows
- Performance tests for large documents
- Edge case testing for all parsing functions

## Code Quality

The project enforces strict code quality standards:

```bash
# Run all linting checks
make lint
# Or: cargo clippy && cargo fmt --check

# Format code
make format
# Or: cargo fmt

# Run full CI pipeline
make ci
```

### Quality Standards

- Clippy rules: Cognitive complexity ≤ 30, function arguments ≤ 7, type complexity ≤ 250
- Documentation: Required for all public items
- Error handling: No `unwrap()`/`expect()` in production code
- Formatting: 100-character line width, 4-space indentation

## Adding New Features

1. Validation Rules: Add to `src/main.rs` in the validation section
2. CLI Commands: Extend the `Commands` enum and add handler functions
3. Tests: Add comprehensive tests to `src/tests.rs`
4. Documentation: Update relevant sections in `docs/`

### Example: Adding a New Validation Rule

```rust
// In src/main.rs
fn validate_new_rule(line: &str) -> Option<usize> {
    // Validation logic here
    None
}

// In validate_markdown function
if let Some(col) = validate_new_rule(line) {
    warnings.push(LintWarning {
        line: line_num,
        column: col,
        message: "Rule violation".to_string(),
        rule: "new-rule".to_string(),
    });
}

// In src/tests.rs
#[test]
fn test_validate_new_rule() {
    // Test cases here
}
```

## Development Workflow

```bash
# 1. Create feature branch
git checkout -b feature/new-validation-rule

# 2. Implement changes
# - Add validation logic
# - Write comprehensive tests
# - Update documentation

# 3. Verify quality
make ci # Runs lint, format, and test

# 4. Test manually
./target/release/agent-md lint test-file.md

# 5. Commit and push
git add .
git commit -m "feat: add new validation rule"
git push origin feature/new-validation-rule
```

## Debugging

For debugging validation issues:

```bash
# Create test file
echo "Test content" > debug.md

# Run with detailed output
./target/debug/agent-md lint debug.md | jq .

# Run specific test
cargo test test_validate_space_indentation -- --nocapture
```

## Performance Considerations

- Large documents (>10,000 words) should complete validation within 1 second
- Memory usage scales linearly with document size
- Test performance with: `cargo test test_validate_markdown_large_document_performance`

## Release Process

```bash
# 1. Update version in Cargo.toml
# 2. Update CHANGELOG.md
# 3. Run full test suite
make ci

# 4. Build release
cargo build --release

# 5. Tag release
git tag -a v0.1.0 -m "Release version 0.1.0"
git push origin v0.1.0
```

## Contributing Guidelines

- Follow existing code style and patterns
- Add tests for all new functionality
- Update documentation for user-facing changes
- Ensure all tests pass before submitting PR
- Use descriptive commit messages
- Keep PRs focused on single features/fixes
