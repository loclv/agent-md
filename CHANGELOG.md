# Changelog

All notable changes to this project will be documented in this file.

## [0.2.11] - 2026-09-12

### Added

- ResolvedConfig Struct: Added `ResolvedConfig` struct to aggregate all configuration options into a single typed struct with defaults.
- Config Value Helpers: Added `get_bool_config`, `get_u64_config`, and `get_string_config` helper functions for extracting typed values from JSON config with fallback defaults.
- resolve_config Function: Added `resolve_config` function that builds a `ResolvedConfig` from a JSON value, falling back to defaults for missing or invalid keys.
- Config-Driven Linter: Updated the linter to use `ResolvedConfig` for all rule toggles: `no-hard-tabs`, `first-line-heading`, `no-duplicate-headings`, `blanks-around-headings`, and `line-length` with `max-line-length` support.
- Linter Config Integration: The `validate_markdown_with_config` function now respects all config options to enable/disable individual lint rules.
- Init Subcommand: Added `init` subcommand and `config --init` flag to generate a default `.agent-md.json` configuration file with support for custom paths and `--force` overwrite.

### Fixed

- Line-Length Code Block Exemption: Moved line-length validation into the primary line-scanning pass, correctly exempting content within code blocks from `max-line-length` warnings.
- Unicode Character Counting: Updated `line-length` checking to count Unicode characters via `chars().count()` rather than raw UTF-8 byte length.
- Duplicate Headings Alias Support: Unified `no-duplicate-heading` (singular) and `no-duplicate-headings` (plural) configuration resolution and rule checking so either alias takes effect.
- CLI Config Passing for Lint: Passed the global `--config` flag to `cmd_lint` and `cmd_lint_file` handlers via `validate_markdown_with_custom_config`.

### Refactor

- Configuration Resolution: Implemented `ResolvedConfig::from_json` with clean closures and combinators, streamlining `resolve_config`, `get_bool_config`, `get_u64_config`, and `get_string_config`.
- Formatter Config Integration: Replaced manual JSON key extraction in `get_format_options` in `src/main.rs` with `resolve_config`.
- Directory Config Search: Extracted `find_config_in_dir` helper in `src/config.rs` to simplify path resolution.

### Tests

- Config Tests: Added 40+ unit tests for `ResolvedConfig`, `resolve_config`, config value helpers, edge cases (invalid JSON, empty files, nonexistent paths, priority ordering, serialization).
- Linter Config Tests: Added 20+ unit tests verifying linter behavior with different config combinations (hard tabs, first-line-heading, duplicate headings, blanks-around-headings, line-length).

## [0.2.10] - 2026-09-12

### Added

- Folder Structure Formatting: Added automatic detection and formatting for folder tree structures in code blocks with language `text`, `txt`, or unlabelled.
- Configuration File Support: Added support for reading and resolving configuration files across `.agent-md.json` (highest priority), `agent-md.json`, and `.markdownlint.json` (fallback).
- Config Subcommand: Added `agent-md config` command with `--check` flag to inspect configuration file existence, path, and parsed content.

### Fixed

- Folder Structure Syntax Compaction: Compacted tree branch markers by removing redundant dashes (`├──` to `├─`, `└──` to `└─`), stripping spacer lines consisting solely of vertical bars (`│`), removing spaces between branch markers and file/folder names, and collapsing multiple spaces before comments.

### Tests

- Folder Structure Tests: Added unit tests and integration tests covering language validation, syntax detection, idempotent formatting, nested folder trees, and end-to-end markdown formatting.
- Configuration Tests: Added tests for configuration file priority resolution, existence checks, parsing, and CLI commands.

## [0.2.9] - 2026-08-17

### Fixed

- Empty List Items: Fixed markdown formatter to remove empty list items (e.g. a trailing `- ` line with no content) instead of preserving a dangling marker line.
- Code Block Language in Lists: Auto add `text` language specifier to unlabelled code fences nested inside list items during formatting, matching the existing behavior for top-level code blocks.

### Tests

- Empty List Items: Added 34 unit tests covering marker edge cases (`-`, `*`, `+`, `1.`, `1)`), indentation variants (spaces, tabs), list boundaries (start, middle, end, consecutive, before headings and paragraphs), code block content preservation, consecutive code blocks (unlabelled, mixed languages, separated by horizontal rules), and direct `format_list_items` behavior.

## [0.2.8] - 2026-08-02

### Fixed

- Formatter Header Trailing Colon: Fixed markdown formatter to remove trailing colons (`:`) from heading text (e.g. `## header:` => `## header`).
- Formatter Code Block Language: Auto add `text` language specifier to unlabelled code blocks during formatting (e.g. ` ``` ` => ` ```text `).

## [0.2.7] - 2026-07-28

### Added

- List Bullet Point Unit Tests: Added unit test `test_format_list_item_asterisk_bullet_with_bold` for asterisk bullet items with bold headers (`* **header**: value`).

### Fixed

- List Bullet Point Emphasis Parsing: Fixed an issue where asterisk bullet list items followed by italicized text (e.g., `* *header*: value`) were incorrectly parsed as emphasis wrappers around spaces, causing `* *` to be stripped to a leading space. Opening and closing emphasis and bold markers are now strictly validated against whitespace boundaries.

## [0.2.6] - 2026-07-28

### Added

- Table Formatter Unit Tests: Added unit test `test_format_markdown_table_empty_cell_trailing_spaces` to verify empty cell formatting with trailing spaces.

### Fixed

- Table Empty Cell Formatting: Fixed table row formatting so empty cells format as `| |` (single space) instead of auto-adding an extra space to format as `|  |`.

## [0.2.4] - 2026-05-24

### Added

- Strict Code Block Syntax Validation: Implemented unclosed code block detection to prevent linter and formatter from continuing on malformed markdown documents.

### Fixed

- Lint cascading failures: Halt linting immediately upon encountering an unclosed code block to avoid generating multiple unrelated false-positive style warnings and errors.
- Formatting safeguard: Abort formatting immediately and return a syntax error instead of writing corrupted contents to files.
- Nested lists with code blocks: Fixed a bug where a nested code block inside a list item that contained blank lines was parsed as premature list end, causing the rest of the document to be parsed inside an unclosed outer code block and silently discarded during formatting.

## [0.2.3] - 2026-05-23

### Fixed

- Prevent code blocks nested inside markdown list items from being corrupted during formatting. This ensures their internal relative indentation is preserved and their content is not formatted as markdown text.

## [0.2.0] - 2026-05-18

### Added

- Structured Parser: A new block-based parser in `src/parser.rs` that decomposes Markdown into logical elements (headings, code blocks, tables, lists, paragraphs).
- YAML Frontmatter Support: Automatically extracts and preserves YAML frontmatter at the beginning of documents.
- Configurable Spacing: Added support for `blanks-around-fences` and `blanks-around-lists` options, configurable via `.markdownlint.json`.

### Changed

- Formatting Architecture: Migrated from a line-based state machine to a more robust Parse-then-Format architecture.
- Improved Code Block Formatting: Enhanced handling of blank lines around fenced code blocks and recursive formatting of nested Markdown content.

### Fixed

- More consistent blank line compaction between paragraphs and document blocks.
- 100% test pass rate for existing and new test cases.

## [0.1.10] - 2026-xx-xx

### Features

- Add recursive directory formatting: `agent-md .` or `agent-md fmt <dir>` formats all `.md`/`.markdown` files recursively
- Add unit tests for `collect_markdown_files`
- Add inline documentation for `collapse_spaces_before_comment` and `compact_separator_row`

## [0.1.9] - 2026-05-01

### Features

- Collapse multiple spaces before # comments in bash/sh/zsh/text code blocks

## [0.1.6] - 2026-03-30

### Features

- Enable token-saving format options by default and remove --token-saver flag
- Add code block with nested markdown content to format-all.md test file

### Refactor

- Extract validation rules into separate module

## [0.1.4] - 2026-03-23

### Features

- Add table-trailing-spaces validation rule
- Add --human flag for pretty-printed JSON output

### Documentation

- Add example output for lint command with --human flag

### Tests

- Add comprehensive test coverage for content processing and validation

## [0.1.3] - 2026-03-23

### Features

- Add --human flag for pretty-printed JSON output

### Tests

- Add comprehensive test coverage for content processing and validation

## [0.1.2] - 2026-03-22

### Features

- Enforce ASCII graph detection in code blocks as errors
- Add version flag, logging rules, and improve CLI argument handling
- Add agent-md logo and update Vietnamese README introduction

### Documentation

- Add concrete before/after markdown examples to README files
- Add code formatting to command examples in README files
- Add Vietnamese translation of README
- Add MIT license section to README
- Update README title to emphasize LLM-friendly markdown focus
- Restructure README sections to improve clarity and flow

### Refactor

- Change validation rules from errors to warnings and improve code quality

## [0.1.0] - 2026-03-20

### Features

- Initial release of agent-md (formerly ralph-md)
- Add lint-file command with human-readable output
- Add lint command for markdown validation
- Add README, single H1 validation rule, and markdown writing guide
- Add write-section command for targeted section updates
- Add --field option to read command for direct field extraction
- Add --content/-c flag to read command for section extraction
- Add space-indentation validation rule
- Skip validation inside code blocks and detect multiple violations per line
- Reorganize validation rules documentation with heading structure, code block, and list formatting validators

### Documentation

- Add AGENTS.md with project rules
- Add LLM agent integration guidelines
- Improve installation instructions
- Add no-duplicate-headings rule
- Update simple-tables rule examples

### Refactor

- Rename ralph-md to agent-md
- Flatten project structure
- Remove bold formatting from markdown documentation

### Chore

- Add VS Code settings, project guide, and CI/CD configuration
- Add markdownlint config

## Recent Development (Unreleased)

### 2026-05-20

- Normalize table separator rows: Automatically remove alignment colons (e.g., `|:---|:---|` becomes `|---|---|`) during formatting to save tokens
- Update `simple-tables` lint rule to reject alignment colons as errors
- Update documentation and tests for table normalization

### 2026-05-01

- Fix char-index vs byte-index bug in comment collapsing with Unicode characters
- Extend comment collapsing to `text` code blocks
- Preserve leading indentation for comment-only lines in code blocks

### 2026-04-10

- Update Vietnamese README examples and add markdown code block tests
- Apply formatting rules to markdown code blocks
- Remove extra blank lines throughout Vietnamese README
- Add GitHub repository link to intro blog post
- Expand intro blog with lint rules and VS Code extension details

### 2026-04-09

- Add intro blog post and simplify logo design

### 2026-04-05

- Add VS Code extension for agent-md formatter
- Add stdin formatting support and preserve code block indentation
- Add YAML frontmatter preservation and improve code formatting
- Preserve blank lines before code fences
- Configure language-specific formatters in VS Code settings
- Add MIT license and repository URL to VS Code extension

### 2026-04-04

- Handle empty content before comments in code blocks
- Add comprehensive test coverage for HTML tags in Markdown
- Add test coverage for underscore handling in blockquotes
- Add Setext-style heading detection and validation
- Extract blockquote normalization logic into separate module
- Extract code block formatting logic into separate code_blocks module
- Extract table formatting logic into separate tables module
- Add implicit fmt command when markdown file path provided
- Compact table separator rows to exactly 3 dashes
- Preserve leading whitespace when collapsing spaces

### 2026-04-02

- Add test coverage for validation rules and improve code formatting

### 2026-04-01

- Enable token-saving format options by default

### 2026-03-26

- fmt command: Preserves separator rows and code block content

### 2026-03-24

- Improve table formatting to trim leading and trailing spaces from cells
- Add fmt command to auto-format markdown tables
- Add comprehensive test coverage for table formatting edge cases
