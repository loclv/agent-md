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
│   └── tests.rs # All unit tests (separate module)
├── docs/ # Documentation
├── test-md/ # Test markdown files
└── Makefile # Convenience commands
```

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

The project uses a modular test structure with all tests in `src/tests.rs`:

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_validate_space_indentation

# Run tests with output
cargo test -- --nocapture

# Using Makefile
make test
```

### Test Coverage

- *99 unit tests* covering all validation rules
- *Integration tests* for complete workflows
- *Performance tests* for large documents
- *Edge case testing* for all parsing functions

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

- *Clippy rules*: Cognitive complexity ≤ 30, function arguments ≤ 7, type complexity ≤ 250
- *Documentation*: Required for all public items
- *Error handling*: No `unwrap()`/`expect()` in production code
- *Formatting*: 100-character line width, 4-space indentation

## Adding New Features

1. *Validation Rules*: Add to `src/main.rs` in the validation section
2. *CLI Commands*: Extend the `Commands` enum and add handler functions
3. *Tests*: Add comprehensive tests to `src/tests.rs`
4. *Documentation*: Update relevant sections in `docs/`

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
make ci  # Runs lint, format, and test

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
