# Tasks

## Implementation Steps

- [x] Define blanks-around-headings configuration option in `src/format/mod.rs` inside the `FormatOptions` struct.
- [x] Initialize blanks-around-headings to `true` by default in the `Default` and `token_saver` implementations of `FormatOptions`.
- [x] Parse blanks-around-headings from `.markdownlint.json` in `src/main.rs` inside `get_format_options` function.
- [x] Update `format_markdown_structured` inside `src/format/mod.rs` to respect the blanks-around-headings option.
  - [x] If blanks-around-headings is `true`, ensure two newlines before and after a `Heading` block.
  - [x] If blanks-around-headings is `false`, ensure only one newline before and after a `Heading` block, and skip any adjacent `BlankLine` blocks.
- [x] Add tests in `src/format/mod.rs` to verify that blanks-around-headings operates correctly when set to `true` and `false`.
- [x] Support blanks-around-headings configuration in the validator (`validate_heading_structure` in `src/rules/heading_structure.rs` and `validate_markdown` in `src/main.rs`).
- [x] Add tests in `src/rules/heading_structure.rs` to verify that blanks-around-headings is not enforced when set to `false`.
- [x] Update documentation in `README.md` and `docs/markdown-writing-rules.md` to explain the configuration.
- [x] Validate completion by running formatting, clippy, lint, and tests.
- [x] Log the work using l-log CLI.

## Format List Indentation

- [x] Implement `format_list_item_indentation` helper function in `src/format/mod.rs` to convert 4 leading spaces to 2, and 2 leading tabs to 1.
- [x] Update `MarkdownBlock::List` formatting in `src/format/mod.rs` to apply `format_list_item_indentation`.
- [x] Add unit tests in `src/format/mod.rs` to cover various list indentation cases.
- [x] Update documentation in the project.
- [x] Log work using `l-log` CLI.

## Code Block Syntax Errors Validation

- [x] Implement `find_unclosed_code_block` in `src/rules/code_blocks.rs` to detect unclosed code blocks.
- [x] Export `find_unclosed_code_block` in `src/rules/mod.rs`.
- [x] Update `validate_markdown` in `src/main.rs` to stop linting immediately and return a syntax error if an unclosed code block is detected.
- [x] Update `parser::parse` in `src/parser.rs` to panic with a syntax error if an unclosed code block is detected.
- [x] Update `format_single_file` and `cmd_fmt_stdin` in `src/format/io.rs` to validate and return a syntax error/exit before formatting.
- [x] Write unit tests for unclosed code blocks in `src/rules/code_blocks.rs`.
- [x] Update `README.md` and documentation to explain this behavior.
- [x] Validate completion (lint, format, tests).
- [x] Log work using `l-log` CLI.

## Refactor main.rs into Smaller Files

- [x] Identify candidate modules/functions in `src/main.rs` to extract (e.g. types, CLI/argument parsing, command execution handlers).
- [x] Create `src/types.rs` containing `JsonlEntry`, `Document`, `Heading`, `EditResult`, `SearchResult`, `Match`, `LintResult`, `LintError`, `LintWarning`, `json_output`, `unescape_content`.
- [x] Create `src/linter.rs` containing `validate_markdown`, `get_markdownlint_config`.
- [x] Create `src/commands.rs` containing CLI subcommand handlers: `cmd_read`, `cmd_write`, `cmd_write_section`, `cmd_append`, `cmd_insert`, `cmd_delete`, `cmd_list`, `cmd_search`, `cmd_headings`, `cmd_stats`, `cmd_to_jsonl`, `cmd_lint`, `cmd_lint_file`, and helpers like `extract_section_content`, `find_section_range`, `find_section_end`, `parse_markdown`, `parse_markdown_to_jsonl`.
- [x] Update `src/main.rs` to act as the main entrypoint importing these new modules and housing the `Cli` clap parser and the `main` function.
- [x] Ensure all imports and references are resolved across all modules (including `src/format/io.rs`, `src/parser.rs`, and tests).
- [x] Validate completion (lint, format, tests).
- [x] Log work using `l-log` CLI.

## Dynamic Versioning from Cargo.toml

- [x] Update `src/main.rs` to print the version dynamically using `env!("CARGO_PKG_VERSION")`.
- [x] Add or run unit tests to verify the version printing behavior.
- [x] Validate completion using *make format*, *make lint*, and *make test*.
- [x] Log work using `l-log` CLI.
