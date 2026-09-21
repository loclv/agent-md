# Development

Development setup and guidelines for contributing to agent-md.

## Prerequisites

- Rust 1.94.0 or later (specified in `rust-toolchain.toml`)
- Git
- Make (optional, for convenience commands)

## Project Structure

```text
agent-md/
  src/
    main.rs - CLI entrypoint and clap parsers
    types.rs - Common types, structures and utilities
    config.rs - Configuration file resolution and reading
    ignore.rs - Ignore file (.markdownlintignore, .gitignore) resolution and matching
    linter.rs - Lint rules orchestration
    commands.rs - CLI subcommand handlers and parsing logic
    parser.rs - Structured Markdown parser (block-based)
    format/ - Formatting modules
      mod.rs - Structured formatter orchestration
      tables.rs - Table formatting (separator compaction, row formatting)
      bold_tables.rs - Bold stripping from table cells
      blockquotes.rs
      code_blocks.rs
      frontmatter.rs
      html.rs - HTML minification and autolink preservation
    rules/ - Validation rule modules
    tests.rs - Core unit tests
    html_tests.rs - HTML rendering tests
  docs/ - Documentation
  samples/ - Sample configuration files
  test-md/ - Test markdown files
  Makefile - Convenience commands
```

## Architecture: Structured Parsing and Formatting

agent-md follows a Parse-then-Format architecture:

1. Parsing: The `src/parser.rs` module decomposes the raw Markdown text into a sequence of `MarkdownBlock` elements (e.g., `Heading`, `CodeBlock`, `List`, `Table`, `Html`). This stage also extracts YAML frontmatter.
2. Formatting: The `format_markdown_structured` function in `src/format/mod.rs` iterates over these blocks and applies formatting rules based on the block type and user configuration (from `.agent-md.json`, `agent-md.json`, or `.markdownlint.json`, including `remove_bold`, `compact_blank_lines`, `collapse_spaces`, `remove_horizontal_rules`, `remove_emphasis`, and `minify_html`).
3. Inline Line Processing: The `src/format/lines.rs` module processes individual markdown lines, handling tasks such as collapsing redundant spaces while preserving spaces and content inside inline code spans (`find_code_span_end`), stripping bold markers (`remove_bold`) and emphasis markers (`remove_emphasis`) outside code, and standardizing list item indentations. When `remove_bold` is disabled, double asterisks and underscores are preserved and protected from single emphasis removal. The `src/format/html.rs` module minifies HTML tags and blocks while strictly preserving Markdown autolinks (`<https://...>`, `<user@example.com>`). Inline code blocks throughout paragraphs, lists, and table cells (`src/format/bold_tables.rs`) are preserved unchanged.

This approach is more robust than simple line-based processing, especially for complex structures like nested lists or tables.

## Architecture: Ignore Rules Resolution and Matching

`agent-md` supports ignoring files and directories using `.markdownlintignore` and `.gitignore`:

1. Ignore Pattern Discovery: The `src/ignore.rs` module checks the current directory (or target path) for `.markdownlintignore` and `.gitignore`. If a `.gitignore` is not present in the target directory, it traverses up parent directories to locate the root `.gitignore`.
2. List Merging and Deduplication: `merge_ignore_lists` merges patterns from both ignore files into a single collection, removing duplicate entries while preserving order.
3. Path Matching: `is_ignored` matches paths against ignore rules with support for directory-only patterns ending with `/` (`dist/`), root-anchored patterns starting with `/` (`/target`), glob wildcards with `*` and `?` (`*.log`), and negation rules starting with `!` (`!important.md`).
4. Directory Traversal Integration: Both `agent-md list` and `collect_markdown_files` (`agent-md fmt <dir>`) use `is_ignored` during directory walking to immediately prune ignored directories before reading entries, saving disk I/O and execution time.
5. CLI Command: `agent-md ignore [path]` exposes the merged and deduplicated list as JSON (or human-formatted JSON with `--human`).

## Architecture: Configuration Resolution and Hierarchy

`agent-md` resolves configuration files using an ancestor-walking hierarchy:

1. Candidate Priority: In any directory, configuration files are resolved in order: `.agent-md.json`, `agent-md.json`, `.markdownlint.json`.
2. Target File Ancestor Search: When executing commands against a target Markdown file (such as `agent-md fmt path/to/doc.md` or `agent-md lint path/to/doc.md`), `find_config_for_target` starts in the target file's parent directory and searches upwards through parent directories.
3. Fallback: If no configuration file is located in the target directory tree, the resolver falls back to the current working directory.
4. Explicit Override: When `--config <PATH>` is supplied on the CLI, it bypasses ancestor discovery and uses the specified configuration file or directory directly.

## Building

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release
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
```

## Code Quality

The project enforces strict code quality standards:

```bash
# Run clippy lints
cargo clippy --all-targets --all-features -- -D warnings

# Check code formatting
cargo fmt --check

# Format code
cargo fmt

# Run security audit
cargo audit
```

### Quality Standards

- Clippy rules: Cognitive complexity ≤ 30, function arguments ≤ 7, type complexity ≤ 250
- Documentation: Required for all public items
- Error handling: No `unwrap()`/`expect()` in production code
- Formatting: 100-character line width, 4-space indentation

## Adding New Features

1. Validation Rules: Add logic to `src/rules/` and run them inside `src/linter.rs`'s `validate_markdown` function
2. CLI Commands: Extend `Commands` and `Cli` in `src/main.rs` and add handler functions in `src/commands.rs`
3. Tests: Add comprehensive tests to `src/tests.rs`
4. Documentation: Update relevant sections in `docs/`

### Configuration System

agent-md uses a `ResolvedConfig` struct to aggregate all configuration options:

```rust
// In src/config.rs
pub struct ResolvedConfig {
    pub blanks_around_headings: bool,
    pub blanks_around_lists: bool,
    pub blanks_around_fences: bool,
    pub blanks_around_tables: bool,
    pub first_line_heading: bool,
    pub no_duplicate_heading: bool,
    pub no_duplicate_headings: bool,
    pub line_length: bool,
    pub max_line_length: u64,
    pub ol_prefix: bool,
    pub table_column_style: bool,
    pub no_hard_tabs: bool,
    pub no_inline_html: bool,
}
```

Config values are extracted using typed helpers: `get_bool_config`, `get_u64_config`, `get_string_config`. Invalid or missing values fall back to defaults. Configuration files can be created using `init_config` or the `agent-md init` subcommand.

### Example: Adding a New Validation Rule

```rust
// In src/linter.rs
// Use validate_markdown_with_config for config-aware rules
pub fn validate_markdown_with_config(content: &str, config: &ResolvedConfig) -> LintResult {
    // Access config options:
    if config.no_hard_tabs {
        // Check for hard tabs
    }
    // ...
}

// In src/tests.rs
fn test_validate_new_rule() {
    let mut config = ResolvedConfig::default();
    config.no_hard_tabs = false; // Disable rule
    let result = validate_markdown_with_config("content", &config);
    assert!(result.errors.iter().all(|e| e.rule != "no-hard-tabs"));
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
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test

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
cargo test

# 4. Build release
cargo build --release

# Or install to ~/bin
cargo build --release --target-dir ~/bin

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
