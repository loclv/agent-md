# Tasks

## Implementation Steps

- [x] Define blanks-around-headings configuration option in `src/format/mod.rs` inside the `FormatOptions` struct.
- [x] Initialize blanks-around-headings to `true` by default in the `Default` and `token_saver` implementations of `FormatOptions`.
- [x] Parse blanks-around-headings from `.markdownlint.json` in `src/main.rs` inside `get_format_options` function.
- [x] Update `format_markdown_structured` inside `src/format/mod.rs` to respect the blanks-around-headings option.
  - [x] If blanks-around-headings is `true`, ensure two newlines before and after a `Heading` block.
  - [x] If blanks-around-headings is `false`, ensure only one newline before and after a `Heading` block, and skip any adjacent `BlankLine` blocks.
- [x] Add tests in `src/format/mod.rs` to verify that blanks-around-headings operates correctly when set to `true` and `false`.
- [x] Validate completion by running formatting, clippy, lint, and tests.
- [x] Log the work using l-log CLI.
