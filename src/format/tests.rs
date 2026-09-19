use super::io::*;
use super::*;

#[test]
fn test_format_markdown_table_trailing_spaces() {
	let content = r#"| Column 1 | Column 2 | Column 3 |
|---|---|---|
| Value 1  | Value 2   | Value 3 |
| Another  | Test   | Here |
"#;
	let expected = r#"| Column 1 | Column 2 | Column 3 |
|---|---|---|
| Value 1 | Value 2 | Value 3 |
| Another | Test | Here |
"#;
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_heading_trailing_colon() {
	let content = "## header:\n### subheader:\n# main header\n";
	let expected = "## header\n\n### subheader\n\n# main header\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_table_preserve_leading_spaces() {
	let content = r#"| Column 1 | Column 2 |
|---|---|
| Value 1 |  Value 2 |
|  Value 3 | Value 4 |
"#;
	let expected = r#"| Column 1 | Column 2 |
|---|---|
| Value 1 | Value 2 |
| Value 3 | Value 4 |
"#;
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_separator_rows_unchanged() {
	let content = r#"| Header 1 | Header 2 |
|---|---|
| Value 1 | Value 2 |
"#;
	let expected = r#"| Header 1 | Header 2 |
|---|---|
| Value 1 | Value 2 |
"#;
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_non_table_lines_unchanged() {
	let content = r#"# Title

This is a paragraph.

| Column 1 | Column 2 |
|---|---|
| Value 1  | Value 2 |

Another paragraph.
"#;
	let expected = r#"# Title

This is a paragraph.

| Column 1 | Column 2 |
|---|---|
| Value 1 | Value 2 |

Another paragraph.
"#;
	let result = format_markdown_with_options(
		content,
		FormatOptions {
			remove_bold: true,
			compact_blank_lines: false,
			trim_trailing_whitespace: true,
			collapse_spaces: false,
			remove_horizontal_rules: false,
			remove_emphasis: false,
			blanks_around_lists: false,
			blanks_around_fences: false,
			blanks_around_headings: true,
			minify_html: true,
		},
	);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_empty_cells() {
	let content = r#"| Column 1 | Column 2 | Column 3 |
|---|---|---|
| Value 1  |  | Value 3 |
|  | Value 2  |  |
"#;
	let expected = r#"| Column 1 | Column 2 | Column 3 |
|---|---|---|
| Value 1 | | Value 3 |
| | Value 2 | |
"#;
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_table_empty_cell_trailing_spaces() {
	let content = r#"| Header1 | Header2 |
|---|---|
| c1 |  |
"#;
	let expected = r#"| Header1 | Header2 |
|---|---|
| c1 | |
"#;
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_complex_table() {
	let content = r#"| Name | Age | City      | Notes |
|------|-----|-----------|-------|
| John | 25  | New York  | Test  |
| Jane | 30  | London    |  |
| Bob  | 35  | Paris     | Data  |
"#;
	let expected = r#"| Name | Age | City | Notes |
|---|---|---|---|
| John | 25 | New York | Test |
| Jane | 30 | London | |
| Bob | 35 | Paris | Data |
"#;
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_empty_content() {
	let content = "";
	let result = format_markdown(content);
	assert_eq!(result, "");
}

#[test]
fn test_format_markdown_single_column_table() {
	let content = r#"| Header |
|---|
| Value 1  |
| Value 2   |
| Value 3 |
"#;
	let expected = r#"| Header |
|---|
| Value 1 |
| Value 2 |
| Value 3 |
"#;
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_no_trailing_newline() {
	let content = "| Header |\n|---|\n| Value |";
	let result = format_markdown(content);
	assert!(!result.ends_with('\n'));
	assert_eq!(result, "| Header |\n|---|\n| Value |");
}

#[test]
fn test_format_markdown_mixed_content_with_code_block() {
	let content = r#"# Document

Some text.

```rust
| Not | A | Table |
```

| Real | Table |
|---|---|
| Has | Spaces   |

More text.
"#;
	let expected = r#"# Document

Some text.

```rust
| Not | A | Table |
```

| Real | Table |
|---|---|
| Has | Spaces |

More text.
"#;
	let result = format_markdown_with_options(
		content,
		FormatOptions {
			remove_bold: true,
			compact_blank_lines: false,
			trim_trailing_whitespace: true,
			collapse_spaces: false,
			remove_horizontal_rules: false,
			remove_emphasis: false,
			blanks_around_lists: false,
			blanks_around_fences: false,
			blanks_around_headings: true,
			minify_html: true,
		},
	);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_table_with_inline_code() {
	let content = r#"| Function | Description |
|---|---|
| `test()`  | Runs tests   |
| `main()`  | Entry point  |
"#;
	let expected = r#"| Function | Description |
|---|---|
| `test()` | Runs tests |
| `main()` | Entry point |
"#;
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_table_with_special_chars() {
	let content = r#"| Symbol | Meaning |
|---|---|
| ->   | Arrow    |
| =>   | Fat arrow |
| <>   | Not equal |
"#;
	let expected = r#"| Symbol | Meaning |
|---|---|
| -> | Arrow |
| => | Fat arrow |
| <> | Not equal |
"#;
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_long_cell_content() {
	let content = r#"| Short | Long Description |
|---|---|
| A | This is a very long description with many words   |
"#;
	let expected = r#"| Short | Long Description |
|---|---|
| A | This is a very long description with many words |
"#;
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_multiple_tables() {
	let content = r#"# Tables

| Table 1 | Col 2 |
|---|---|
| A  | B   |

Some text.

| Table 2 | Col 2 |
|---|---|
| X  | Y   |
"#;
	let expected = r#"# Tables

| Table 1 | Col 2 |
|---|---|
| A | B |

Some text.

| Table 2 | Col 2 |
|---|---|
| X | Y |
"#;
	let result = format_markdown_with_options(
		content,
		FormatOptions {
			remove_bold: true,
			compact_blank_lines: false,
			trim_trailing_whitespace: true,
			collapse_spaces: false,
			remove_horizontal_rules: false,
			remove_emphasis: false,
			blanks_around_lists: false,
			blanks_around_fences: false,
			blanks_around_headings: true,
			minify_html: true,
		},
	);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_table_with_links() {
	let content = r#"| Name | Link |
|---|---|
| Rust  | [rust-lang.org](https://rust-lang.org)  |
"#;
	let expected = r#"| Name | Link |
|---|---|
| Rust | [rust-lang.org](https://rust-lang.org) |
"#;
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_non_table_pipes_unchanged() {
	let content = r#"This | is | not | a | table
Just text with | pipes
"#;
	let result = format_markdown(content);
	assert_eq!(result, content);
}

#[test]
fn test_format_markdown_preserve_indentation() {
	let content = r#"  | Indented | Table |
  |---|---|
  | Value 1  | Value 2   |
"#;
	let expected = r#"  | Indented | Table |
  |---|---|
  | Value 1 | Value 2 |
"#;
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_tab_indentation() {
	let content = "\t| Tabbed | Table |\n\t|---|---|\n\t| Value 1  | Value 2 |\n";
	let expected = "\t| Tabbed | Table |\n\t|---|---|\n\t| Value 1 | Value 2 |\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_only_newlines() {
	let content = "\n\n\n";
	let result = format_markdown_with_options(
		content,
		FormatOptions {
			remove_bold: true,
			compact_blank_lines: false,
			trim_trailing_whitespace: true,
			collapse_spaces: false,
			remove_horizontal_rules: false,
			remove_emphasis: false,
			blanks_around_lists: false,
			blanks_around_fences: false,
			blanks_around_headings: true,
			minify_html: true,
		},
	);
	assert_eq!(result, "\n\n\n");
}

#[test]
fn test_format_markdown_single_pipe_not_table() {
	let content = "| This is not a table because it doesn't end with pipe\n";
	let result = format_markdown(content);
	assert_eq!(result, content);
}

#[test]
fn test_format_markdown_table_with_unicode() {
	let content = "| Emoji | Text |\n|---|---|\n| 🎉  | Party  |\n| 🦀  | Rust |\n";
	let expected = "| Emoji | Text |\n|---|---|\n| 🎉 | Party |\n| 🦀 | Rust |\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_table_with_numbers() {
	let content = "| ID | Count |\n|---|---|\n| 1   | 100   |\n| 2   | 200 |\n";
	let expected = "| ID | Count |\n|---|---|\n| 1 | 100 |\n| 2 | 200 |\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_many_columns() {
	let content = "| A | B | C | D | E | F | G | H |\n|---|---|---|---|---|---|---|---|\n| 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 |\n";
	let expected = "| A | B | C | D | E | F | G | H |\n|---|---|---|---|---|---|---|---|\n| 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 |\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_table_in_blockquote() {
	let content = "> | Quote | Table |\n> |---|---|\n> | Data 1  | Data 2 |\n";
	let expected = "> | Quote | Table |\n> |---|---|\n> | Data 1 | Data 2 |\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_list_item_with_table() {
	let content = "- | List | Item |\n- |---|---|\n- | Data  | Value |\n";
	let expected = "- | List | Item |\n- |---|---|\n- | Data | Value |\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_empty_table_cells() {
	let content = "| A | B | C |\n|---|---|---|\n|   |   |   |\n";
	let expected = "| A | B | C |\n|---|---|---|\n| | | |\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_windows_line_endings() {
	let content = "| Header |\r\n|---|\r\n| Value  |\r\n";
	let expected = "| Header |\n|---|\n| Value |\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_nested_pipes_in_code() {
	let content = "| Code | Output |\n|---|---|\n| `a | b`  | Result |\n";
	let expected = "| Code | Output |\n|---|---|\n| `a | b` | Result |\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_single_row_table() {
	let content = "| Only | Header | Row |\n";
	let expected = "| Only | Header | Row |\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_bold_in_code_block_preserved() {
	let content = r#"```
This has **bold** text
Also has __bold__ markers
```

Regular text with **bold** and __bold__ should be removed.
"#;
	let expected = r#"```text
This has **bold** text
Also has __bold__ markers
```

Regular text with bold and bold should be removed.
"#;
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_bold_in_inline_code_preserved() {
	let content = r#"Use `**bold**` for emphasis or `__bold__` syntax.

But **remove** these __markers__ outside code.
"#;
	let expected = r#"Use `**bold**` for emphasis or `__bold__` syntax.

But remove these markers outside code.
"#;
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_bold_in_table_inline_code_preserved() {
	let content = r#"| Syntax | Example |
|---|---|
| Bold | `**text**` |
| Underline | `__text__` |
"#;
	let expected = r#"| Syntax | Example |
|---|---|
| Bold | `**text**` |
| Underline | `__text__` |
"#;
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_separator_with_colons() {
	let content = "| Left | Center | Right |\n|:---|:---:|---:|\n| A | B | C |\n";
	let expected = "| Left | Center | Right |\n|---|---|---|\n| A | B | C |\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_options_token_saver() {
	let content = "# Title\n\n\n\n## Section\n\n\nText with **bold**   \n\n- Item 1\n- Item 2\n\n\n## Another\n\n```\n**bold in code**\n```\n";
	let options = FormatOptions::token_saver();
	let result = format_markdown_with_options(content, options);

	assert!(!result.contains("**bold**"));
	assert!(result.contains("bold"));
	assert!(result.contains("```text\n**bold in code**\n```"));
}

#[test]
fn test_format_options_compact_blank_lines() {
	let content = "Line 1\n\n\n\nLine 2\n\n\n\nLine 3";
	let options = FormatOptions {
		remove_bold: false,
		compact_blank_lines: true,
		trim_trailing_whitespace: false,
		collapse_spaces: false,
		remove_horizontal_rules: false,
		remove_emphasis: false,
		blanks_around_lists: false,
		blanks_around_fences: false,
		blanks_around_headings: true,
		minify_html: true,
	};
	let result = format_markdown_with_options(content, options);
	assert_eq!(result, "Line 1\n\nLine 2\n\nLine 3");
}

#[test]
fn test_format_options_remove_bold_disabled() {
	let content = "This is **bold** text";
	let options = FormatOptions {
		remove_bold: false,
		compact_blank_lines: false,
		trim_trailing_whitespace: false,
		collapse_spaces: false,
		remove_horizontal_rules: false,
		remove_emphasis: false,
		blanks_around_lists: false,
		blanks_around_fences: false,
		blanks_around_headings: true,
		minify_html: true,
	};
	let result = format_markdown_with_options(content, options);
	assert_eq!(result, "This is **bold** text");
}

#[test]
fn test_format_options_trim_trailing_whitespace() {
	let content = "Text with trailing   \nMore text   ";
	let options = FormatOptions {
		remove_bold: false,
		compact_blank_lines: false,
		trim_trailing_whitespace: true,
		collapse_spaces: false,
		remove_horizontal_rules: false,
		remove_emphasis: false,
		blanks_around_lists: false,
		blanks_around_fences: false,
		blanks_around_headings: true,
		minify_html: true,
	};
	let result = format_markdown_with_options(content, options);
	assert_eq!(result, "Text with trailing\nMore text");
}

#[test]
fn test_format_options_collapse_spaces() {
	let content = "This    has    multiple   spaces";
	let options = FormatOptions {
		remove_bold: false,
		compact_blank_lines: false,
		trim_trailing_whitespace: false,
		collapse_spaces: true,
		remove_horizontal_rules: false,
		remove_emphasis: false,
		blanks_around_lists: false,
		blanks_around_fences: false,
		blanks_around_headings: true,
		minify_html: true,
	};
	let result = format_markdown_with_options(content, options);
	assert_eq!(result, "This has multiple spaces");
}

#[test]
fn test_format_options_collapse_spaces_in_heading_preserved() {
	let content = "# Heading   with    spaces";
	let options = FormatOptions {
		remove_bold: false,
		compact_blank_lines: false,
		trim_trailing_whitespace: false,
		collapse_spaces: true,
		remove_horizontal_rules: false,
		remove_emphasis: false,
		blanks_around_lists: false,
		blanks_around_fences: false,
		blanks_around_headings: true,
		minify_html: true,
	};
	let result = format_markdown_with_options(content, options);
	assert_eq!(result, "# Heading   with    spaces");
}

#[test]
fn test_format_options_collapse_spaces_preserves_inline_code() {
	let content = "- Table Empty Cell Formatting: Fixed table row formatting so empty cells format as `| |` (single space) instead of auto-adding an extra space to format as `|  |`.\n";
	let result = format_markdown(content);
	assert_eq!(result, content);
}

#[test]
fn test_format_options_collapse_spaces_preserves_inline_code_variations() {
	let content = "Some    text   with   `foo   bar`   and  `` `|  |` ``   code.\n";
	let expected = "Some text with `foo   bar` and `` `|  |` `` code.\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_find_code_span_end() {
	use super::lines::find_code_span_end;

	let s: Vec<char> = "`hello`".chars().collect();
	assert_eq!(find_code_span_end(&s, 0), Some(6));

	let s2: Vec<char> = "`` `hello` ``".chars().collect();
	assert_eq!(find_code_span_end(&s2, 0), Some(12));

	let s3: Vec<char> = "`unclosed".chars().collect();
	assert_eq!(find_code_span_end(&s3, 0), None);
}

#[test]
fn test_format_options_remove_horizontal_rules() {
	let content = "Before\n\n---\n\nAfter";
	let options = FormatOptions {
		remove_bold: false,
		compact_blank_lines: false,
		trim_trailing_whitespace: false,
		collapse_spaces: false,
		remove_horizontal_rules: true,
		remove_emphasis: false,
		blanks_around_lists: false,
		blanks_around_fences: false,
		blanks_around_headings: true,
		minify_html: true,
	};
	let result = format_markdown_with_options(content, options);
	assert!(!result.contains("---"));
	assert!(result.contains("Before"));
	assert!(result.contains("After"));
}

#[test]
fn test_format_options_remove_horizontal_rules_variations() {
	let content = "Text\n\n***\n\nMore\n\n___\n\nEnd";
	let options = FormatOptions {
		remove_bold: false,
		compact_blank_lines: false,
		trim_trailing_whitespace: false,
		collapse_spaces: false,
		remove_horizontal_rules: true,
		remove_emphasis: false,
		blanks_around_lists: false,
		blanks_around_fences: false,
		blanks_around_headings: true,
		minify_html: true,
	};
	let result = format_markdown_with_options(content, options);
	assert!(!result.contains("***"));
	assert!(!result.contains("___"));
	assert!(result.contains("Text"));
	assert!(result.contains("More"));
	assert!(result.contains("End"));
}

#[test]
fn test_format_options_remove_emphasis() {
	let content = "This is *italic* text";
	let options = FormatOptions {
		remove_bold: false,
		compact_blank_lines: false,
		trim_trailing_whitespace: false,
		collapse_spaces: false,
		remove_horizontal_rules: false,
		remove_emphasis: true,
		blanks_around_lists: false,
		blanks_around_fences: false,
		blanks_around_headings: true,
		minify_html: true,
	};
	let result = format_markdown_with_options(content, options);
	assert_eq!(result, "This is italic text");
}

#[test]
fn test_format_options_remove_emphasis_underscore() {
	let content = "This is _italic_ text";
	let options = FormatOptions {
		remove_bold: false,
		compact_blank_lines: false,
		trim_trailing_whitespace: false,
		collapse_spaces: false,
		remove_horizontal_rules: false,
		remove_emphasis: true,
		blanks_around_lists: false,
		blanks_around_fences: false,
		blanks_around_headings: true,
		minify_html: true,
	};
	let result = format_markdown_with_options(content, options);
	assert_eq!(result, "This is italic text");
}

#[test]
fn test_format_options_remove_emphasis_in_heading_preserved() {
	let content = "# *Heading* with emphasis";
	let options = FormatOptions {
		remove_bold: false,
		compact_blank_lines: false,
		trim_trailing_whitespace: false,
		collapse_spaces: false,
		remove_horizontal_rules: false,
		remove_emphasis: true,
		blanks_around_lists: false,
		blanks_around_fences: false,
		blanks_around_headings: true,
		minify_html: true,
	};
	let result = format_markdown_with_options(content, options);
	assert_eq!(result, "# *Heading* with emphasis");
}

#[test]
fn test_format_options_remove_emphasis_in_code_preserved() {
	let content = "Use `*italic*` in code";
	let options = FormatOptions {
		remove_bold: false,
		compact_blank_lines: false,
		trim_trailing_whitespace: false,
		collapse_spaces: false,
		remove_horizontal_rules: false,
		remove_emphasis: true,
		blanks_around_lists: false,
		blanks_around_fences: false,
		blanks_around_headings: true,
		minify_html: true,
	};
	let result = format_markdown_with_options(content, options);
	assert_eq!(result, "Use `*italic*` in code");
}

#[test]
fn test_format_options_token_saver_includes_new_rules() {
	let content = "# Title\n\nText  with   *emphasis*\n\n---\n\nMore  text";
	let options = FormatOptions::token_saver();
	let result = format_markdown_with_options(content, options);

	assert!(result.contains("# Title"));
	assert!(result.contains("Text with emphasis"));
	assert!(!result.contains("---"));
	assert!(result.contains("More text"));
}

#[test]
fn test_format_options_blanks_around_lists_add() {
	let content = "Text\n- Item 1\n- Item 2\nEnd";
	let options = FormatOptions {
		blanks_around_lists: true,
		..FormatOptions::default()
	};
	let result = format_markdown_with_options(content, options);
	assert_eq!(result, "Text\n\n- Item 1\n- Item 2\n\nEnd");
}

#[test]
fn test_format_options_blanks_around_lists_keep() {
	let content = "Text\n\n- Item 1\n- Item 2\n\nEnd";
	let options = FormatOptions {
		blanks_around_lists: true,
		..FormatOptions::default()
	};
	let result = format_markdown_with_options(content, options);
	assert_eq!(result, "Text\n\n- Item 1\n- Item 2\n\nEnd");
}

#[test]
fn test_format_options_blanks_around_lists_disabled() {
	let content = "Text\n- Item 1\n- Item 2\nEnd";
	let options = FormatOptions {
		blanks_around_lists: false,
		blanks_around_fences: false,
		blanks_around_headings: true,
		..FormatOptions::default()
	};
	let result = format_markdown_with_options(content, options);
	assert_eq!(result, "Text\n- Item 1\n- Item 2\nEnd");
}

#[test]
fn test_format_options_blanks_around_lists_nested() {
	let content = "Text\n- Item 1\n  - Nested\nEnd";
	let options = FormatOptions {
		blanks_around_lists: true,
		..FormatOptions::default()
	};
	let result = format_markdown_with_options(content, options);
	assert_eq!(result, "Text\n\n- Item 1\n  - Nested\n\nEnd");
}

#[test]
fn test_format_options_blanks_around_lists_consecutive_lists() {
	let content = "- List 1\n\n1. List 2";
	let options = FormatOptions {
		blanks_around_lists: true,
		..FormatOptions::default()
	};
	let result = format_markdown_with_options(content, options);
	// Should preserve/ensure exactly one blank line between different list types
	assert_eq!(result, "- List 1\n\n1. List 2");
}

#[test]
fn test_format_options_blanks_around_lists_start_of_file() {
	let content = "- Item 1\nText";
	let options = FormatOptions {
		blanks_around_lists: true,
		..FormatOptions::default()
	};
	let result = format_markdown_with_options(content, options);
	assert_eq!(result, "- Item 1\n\nText");
}

#[test]
fn test_format_options_blanks_around_lists_end_of_file() {
	let content = "Text\n- Item 1";
	let options = FormatOptions {
		blanks_around_lists: true,
		..FormatOptions::default()
	};
	let result = format_markdown_with_options(content, options);
	assert_eq!(result, "Text\n\n- Item 1");
}

#[test]
fn test_format_options_blanks_around_lists_with_heading() {
	let content = "# Title\n- Item 1\n## Section";
	let options = FormatOptions {
		blanks_around_lists: true,
		..FormatOptions::default()
	};
	let result = format_markdown_with_options(content, options);
	// Heading logic already adds blank lines, should not double up
	assert_eq!(result, "# Title\n\n- Item 1\n\n## Section");
}

#[test]
fn test_format_options_blanks_around_lists_in_blockquote() {
	let content = "> - Item 1\n> - Item 2";
	let options = FormatOptions {
		blanks_around_lists: true,
		..FormatOptions::default()
	};
	let result = format_markdown_with_options(content, options);
	// Inside blockquotes, we don't currently add blank lines because we process line by line
	// and the prefix "> " makes it not look like a list item to the simple detector.
	// This test documents current behavior.
	assert_eq!(result, ">- Item 1\n>- Item 2");
}

#[test]
fn test_format_options_blanks_around_lists_with_code_block() {
	let content = "Text\n- Item 1\n```rust\nfn main() {}\n```";
	let options = FormatOptions {
		blanks_around_lists: true,
		..FormatOptions::default()
	};
	let result = format_markdown_with_options(content, options);
	assert_eq!(result, "Text\n\n- Item 1\n\n```rust\nfn main() {}\n```");
}

#[test]
fn test_format_options_blanks_around_lists_compact_multiple() {
	let content = "Text\n\n\n\n- Item 1\n\n\n\nEnd";
	let options = FormatOptions {
		blanks_around_lists: true,
		compact_blank_lines: true,
		..FormatOptions::default()
	};
	let result = format_markdown_with_options(content, options);
	assert_eq!(result, "Text\n\n- Item 1\n\nEnd");
}

#[test]
fn test_format_options_blanks_around_lists_disabled_keep_existing() {
	let content = "Text\n\n- Item 1\n\nEnd";
	let options = FormatOptions {
		blanks_around_lists: false,
		compact_blank_lines: false,
		..FormatOptions::default()
	};
	let result = format_markdown_with_options(content, options);
	assert_eq!(result, "Text\n\n- Item 1\n\nEnd");
}

#[test]
fn test_format_options_blanks_around_lists_with_multiple_paragraphs() {
	let content = "- Item 1\n\n  Still item 1\n\nNext";
	let options = FormatOptions {
		blanks_around_lists: true,
		..FormatOptions::default()
	};
	let result = format_markdown_with_options(content, options);
	// Note: The current line-by-line processor might not perfectly handle
	// multiline list items if they have blank lines inside, as it resets context.
	// This test helps understand how it handles them.
	assert_eq!(result, "- Item 1\n\n  Still item 1\n\nNext");
}

#[test]
fn test_format_options_blanks_around_lists_loose_list() {
	let content = "- Item 1\n\n- Item 2";
	let options = FormatOptions {
		blanks_around_lists: true,
		compact_blank_lines: true,
		..FormatOptions::default()
	};
	let result = format_markdown_with_options(content, options);
	// Loose lists should have their internal blank lines preserved
	assert_eq!(result, "- Item 1\n\n- Item 2");
}

#[test]
fn test_format_markdown_compact_separator_dashes() {
	let content = "| Real | Table |\n|----|----|\n| Has | dashes |\n";
	let expected = "| Real | Table |\n|---|---|\n| Has | dashes |\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_compact_separator_preserve_alignment() {
	let content = "| Left | Center | Right |\n|:-----|:------:|------:|\n| A | B | C |\n";
	let expected = "| Left | Center | Right |\n|---|---|---|\n| A | B | C |\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_preserve_list_indentation() {
	let content = "lists:\n  - 2 spaces\n  - 2 spaces\n";
	let expected = "lists:\n\n  - 2 spaces\n  - 2 spaces\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_preserve_nested_list_indentation() {
	let content = "- Level 1\n  - Level 2\n    - Level 3\n";
	let expected = "- Level 1\n  - Level 2\n    - Level 3\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_bash_comment_space_collapse() {
	let content = r#"```bash
cd              # goto
agent-md # format
```
"#;
	let expected = r#"```bash
cd # goto
agent-md # format
```
"#;
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_inside_markdown_code_block() {
	let content = r#"```markdown
| Column 1 | Column 2 | Column 3 |
|-----------|---------|----------|
| Value 1   | Value 2 | Value 3  |
```
"#;
	let expected = r#"```markdown
| Column 1 | Column 2 | Column 3 |
|---|---|---|
| Value 1 | Value 2 | Value 3 |
```
"#;
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_inside_md_code_block() {
	let content = r#"```md
**Bold** text and *italic* text.
```
"#;
	let expected = r#"```md
Bold text and italic text.
```
"#;
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_inside_markdown_block_with_list() {
	let content = r#"```markdown
1. **First item**
2. **Second item**: `code`
3. **Third item**
```
"#;
	let expected = r#"```markdown
1. First item
2. Second item: `code`
3. Third item
```
"#;
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_cmd_fmt_stdin_basic() {
	// Test that cmd_fmt_stdin produces correct output via format_markdown_with_options
	let input = "# Test\n\nThis has **bold** text.\n";
	let options = FormatOptions::default();
	let result = format_markdown_with_options(input, options);
	assert!(result.contains("# Test"));
	assert!(!result.contains("**bold**"));
	assert!(result.contains("bold"));
}

#[test]
fn test_cmd_fmt_stdin_preserves_code_blocks() {
	// Non-markdown code block content should be preserved
	let input = r#"```text
**bold** should stay
```

**bold** should go
"#;
	let options = FormatOptions::default();
	let result = format_markdown_with_options(input, options);
	assert!(result.contains("**bold** should stay"));
	assert!(result.contains("bold should go"));
}

#[test]
fn test_cmd_fmt_stdin_options_respected() {
	// Test that options are properly applied
	let input = "Text with *emphasis* and **bold**.\n";
	let options = FormatOptions {
		remove_bold: false,
		remove_emphasis: false,
		..FormatOptions::default()
	};
	let result = format_markdown_with_options(input, options);
	assert!(result.contains("**bold**"));
	assert!(result.contains("*emphasis*"));
}

#[test]
fn test_format_markdown_sh_comment_space_collapse() {
	let content = r#"```sh
echo "hello"   # comment
```
"#;
	let expected = r#"```sh
echo "hello" # comment
```
"#;
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_highlight_syntax_preserved() {
	let content = "I need to highlight these ==very important words==.";
	let result = format_markdown(content);
	assert_eq!(result, content);
}

#[test]
fn test_format_markdown_highlight_multiple_instances() {
	let content = "This ==highlighted text== and ==this too== should stay.";
	let result = format_markdown(content);
	assert_eq!(result, content);
}

#[test]
fn test_format_markdown_highlight_in_code_block_preserved() {
	let content = r#"```
==highlight in code==
```
"#;
	let expected = r#"```text
==highlight in code==
```
"#;
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_blockquote_normalize_spaces() {
	let content = r#"> 1
>     2
>> 3
>>4
>>> 5
>>>  6
"#;
	let expected = r#">1
>2
>>3
>>4
>>>5
>>>6
"#;
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_blockquote_with_text() {
	let content = r#">  This is a quote
>>  Nested quote
"#;
	let expected = r#">This is a quote
>>Nested quote
"#;
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_blockquote_with_emphasis_and_underscores() {
	// Only spaces after > marker removed
	// Emphasis markers (*cat*) preserved
	// Underscores in identifiers (A_cat_meow) preserved
	let content = r#"> A*cat*meow
> A_cat_meow
"#;
	let expected = r#">A*cat*meow
>A_cat_meow
"#;
	let options = FormatOptions {
		remove_bold: true,
		compact_blank_lines: true,
		trim_trailing_whitespace: true,
		collapse_spaces: true,
		remove_horizontal_rules: true,
		remove_emphasis: false, // Preserve emphasis markers
		blanks_around_lists: false,
		blanks_around_fences: false,
		blanks_around_headings: true,
		minify_html: true,
	};
	let result = format_markdown_with_options(content, options);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_frontmatter_preserved() {
	let content = r#"---
trigger: always_on
---

Should not change, --- should not be remove by format
"#;
	let expected = r#"---
trigger: always_on
---

Should not change, --- should not be remove by format
"#;
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_frontmatter_with_content() {
	let content = r#"---
title: Test Document
author: Author Name
---

# Heading

Content here.
"#;
	let expected = r#"---
title: Test Document
author: Author Name
---

# Heading

Content here.
"#;
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_horizontal_rule_after_frontmatter_removed() {
	let content = r#"---
title: Test
---

Text above

---

Text below
"#;
	// When horizontal rule is removed, blank lines get compacted
	let expected = r#"---
title: Test
---

Text above

Text below
"#;
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_blank_line_before_code_fence_preserved() {
	let content = r#"run:

```bash
ls
```
"#;
	// Blank line before code fence should be preserved
	let expected = r#"run:

```bash
ls
```
"#;
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_blank_line_before_code_fence_with_text() {
	let content = r#"Some text here.

```rust
fn main() {}
```
"#;
	let expected = r#"Some text here.

```rust
fn main() {}
```
"#;
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_collect_markdown_files_finds_md_and_markdown() {
	use std::fs;
	let temp_dir = std::env::temp_dir().join("agent_md_test_collect");
	let _ = fs::remove_dir_all(&temp_dir);
	fs::create_dir_all(&temp_dir).unwrap();
	fs::create_dir_all(temp_dir.join("sub")).unwrap();

	fs::write(temp_dir.join("a.md"), "# A").unwrap();
	fs::write(temp_dir.join("b.markdown"), "# B").unwrap();
	fs::write(temp_dir.join("sub/c.md"), "# C").unwrap();
	fs::write(temp_dir.join("readme.txt"), "not md").unwrap();

	let mut files = Vec::new();
	collect_markdown_files(&temp_dir, &mut files).unwrap();
	files.sort();

	let names: Vec<String> = files
		.iter()
		.map(|p| p.file_name().unwrap().to_string_lossy().to_string())
		.collect();

	assert_eq!(names, vec!["a.md", "b.markdown", "c.md"]);
	let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_format_options_blanks_around_fences_add() {
	let content = "Text\n```rust\nfn main() {}\n```\nEnd";
	let options = FormatOptions {
		remove_bold: false,
		compact_blank_lines: true,
		trim_trailing_whitespace: true,
		collapse_spaces: false,
		remove_horizontal_rules: false,
		remove_emphasis: false,
		blanks_around_lists: false,
		blanks_around_fences: true,
		blanks_around_headings: true,
		minify_html: true,
	};
	let result = format_markdown_with_options(content, options);
	let expected = "Text\n\n```rust\nfn main() {}\n```\n\nEnd";
	assert_eq!(result, expected);
}

#[test]
fn test_format_options_blanks_around_fences_disabled() {
	let content = "Text\n\n```rust\nfn main() {}\n```\n\nEnd";
	let options = FormatOptions {
		remove_bold: false,
		compact_blank_lines: false,
		trim_trailing_whitespace: true,
		collapse_spaces: false,
		remove_horizontal_rules: false,
		remove_emphasis: false,
		blanks_around_lists: false,
		blanks_around_fences: false,
		blanks_around_headings: true,
		minify_html: true,
	};
	let result = format_markdown_with_options(content, options);
	let expected = "Text\n\n```rust\nfn main() {}\n```\n\nEnd";
	assert_eq!(result, expected);
}

#[test]
fn test_format_options_blanks_around_fences_compact() {
	let content = "Text\n\n\n\n```rust\nfn main() {}\n```\n\n\n\nEnd";
	let options = FormatOptions {
		remove_bold: false,
		compact_blank_lines: true,
		trim_trailing_whitespace: true,
		collapse_spaces: false,
		remove_horizontal_rules: false,
		remove_emphasis: false,
		blanks_around_lists: false,
		blanks_around_fences: true,
		blanks_around_headings: true,
		minify_html: true,
	};
	let result = format_markdown_with_options(content, options);
	let expected = "Text\n\n```rust\nfn main() {}\n```\n\nEnd";
	assert_eq!(result, expected);
}

#[test]
fn test_collect_markdown_files_empty_dir() {
	use std::fs;
	let temp_dir = std::env::temp_dir().join("agent_md_test_empty");
	let _ = fs::remove_dir_all(&temp_dir);
	fs::create_dir_all(&temp_dir).unwrap();

	let mut files = Vec::new();
	collect_markdown_files(&temp_dir, &mut files).unwrap();
	assert!(files.is_empty());
	let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_collect_markdown_files_nested_dirs() {
	use std::fs;
	let temp_dir = std::env::temp_dir().join("agent_md_test_nested");
	let _ = fs::remove_dir_all(&temp_dir);
	fs::create_dir_all(temp_dir.join("a/b/c")).unwrap();

	fs::write(temp_dir.join("root.md"), "# root").unwrap();
	fs::write(temp_dir.join("a/level1.md"), "# l1").unwrap();
	fs::write(temp_dir.join("a/b/level2.md"), "# l2").unwrap();
	fs::write(temp_dir.join("a/b/c/level3.markdown"), "# l3").unwrap();

	let mut files = Vec::new();
	collect_markdown_files(&temp_dir, &mut files).unwrap();
	assert_eq!(files.len(), 4);
	let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_format_options_blanks_around_headings_enabled() {
	let content = "Text\n# Heading\nParagraph";
	let options = FormatOptions {
		remove_bold: false,
		compact_blank_lines: true,
		trim_trailing_whitespace: true,
		collapse_spaces: false,
		remove_horizontal_rules: false,
		remove_emphasis: false,
		blanks_around_lists: false,
		blanks_around_fences: false,
		blanks_around_headings: true,
		minify_html: true,
	};
	let result = format_markdown_with_options(content, options);
	let expected = "Text\n\n# Heading\n\nParagraph";
	assert_eq!(result, expected);
}

#[test]
fn test_format_options_blanks_around_headings_disabled() {
	let content = "Text\n\n# Heading\n\nParagraph";
	let options = FormatOptions {
		remove_bold: false,
		compact_blank_lines: true,
		trim_trailing_whitespace: true,
		collapse_spaces: false,
		remove_horizontal_rules: false,
		remove_emphasis: false,
		blanks_around_lists: false,
		blanks_around_fences: false,
		blanks_around_headings: false,
		minify_html: true,
	};
	let result = format_markdown_with_options(content, options);
	let expected = "Text\n# Heading\nParagraph";
	assert_eq!(result, expected);
}

#[test]
fn test_format_list_indentation() {
	let content = "- list:\n    - A\n    - B\n";
	let expected = "- list:\n  - A\n  - B\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_list_indentation_nested() {
	let content = "- list:\n    - A\n        - B\n";
	let expected = "- list:\n  - A\n    - B\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_list_indentation_tabs() {
	let content = "- list:\n\t\t- A\n\t\t- B\n";
	let expected = "- list:\n\t- A\n\t- B\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_nested_code_block_in_list() {
	let content = "- Here is the code:\n  ```ts\n      users: new Users(db.collection<IUser>('User') as any)\n  ```\n";
	let expected = "- Here is the code:\n  ```ts\n      users: new Users(db.collection<IUser>('User') as any)\n  ```\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_nested_code_block_in_list_preserves_content() {
	let content = "- List item:\n  ```ts\n     //   Comment with spaces\n     const x = \"some  text\";\n     let y = **bold**;\n  ```\n";
	let expected = "- List item:\n  ```ts\n     //   Comment with spaces\n     const x = \"some  text\";\n     let y = **bold**;\n  ```\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_bug_reproduction() {
	let content = r#"# 🥦 Linked-Mind

- Usage:
  ```bash
  # Generates map.toon (default TOON format)
  map-builder

  # Generates map.json (optional JSON format)
  map-builder --json
  ```
## 🧠 Why Graph-based KB for LLMs?
Standard RAG (Retrieval-Augmented Generation) often treats files as isolated chunks. However, human knowledge is a web. By using Linked-Mind, you provide the LLM with:
1. Contextual Proximity: If Node A links to Node B, the LLM knows they are related even if they don't share keywords.
2. Structural Understanding: The AI sees the hierarchy and tags, allowing it to "browse" your brain more effectively.
## 📂 Project Structure
- `src/parser.zig`: Unified parser for multiple formats (.md, .org, .txt, .pdf) extracting `[[links]]` and `#tags`.
- `src/graph.zig`: Adjacency-list based graph representation and link resolver.
- `src/li.zig`: Workspace-aware Workspace-aware CLI with `init`, `scan`, `export`, `path`, `clusters`, `gc`, `similar`, `suggest`, `visualize`.
- `src/cache.zig`: Incremental scanning engine with `mtime` + SHA-256 cache.
- `src/main.zig`: Legacy CLI handler (direct path mode).

Built with speed and precision in Zig.
"#;
	let parsed = crate::parser::parse(content);
	for (idx, block) in parsed.blocks.iter().enumerate() {
		println!("Block {}: {:?}", idx, block);
	}
	let result = format_markdown(content);
	println!("Formatted result:\n{}", result);
}

#[test]
fn test_format_list_item_asterisk_bullet_with_italics() {
	let content = r#"* *header 1*: value 1
* *header 2*: value 2
* *header 3*: value 3
"#;
	let expected = r#"* header 1: value 1
* header 2: value 2
* header 3: value 3
"#;
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_list_item_asterisk_bullet_with_bold() {
	let content = r#"* **header 2**: value 2
* **header 3**: value 3
"#;
	let expected = r#"* header 2: value 2
* header 3: value 3
"#;
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_heading_trailing_colon_edge_cases() {
	let content =
		"# Title:\n## Section 1: Overview:\n### Sub-section: Details:\n#### Note: Important:\n";
	let expected =
		"# Title\n\n## Section 1: Overview\n\n### Sub-section: Details\n\n#### Note: Important\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_heading_trailing_colon_with_whitespace() {
	let content = "## header:   \n";
	let expected = "## header\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_code_block_default_text_language() {
	let content = "```\nabc\n```\n";
	let expected = "```text\nabc\n```\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_consecutive_code_blocks_default_text() {
	let content = "```\nabc\n```\n```\nxyz\n```\n";
	let expected = "```text\nabc\n```\n\n```text\nxyz\n```\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_consecutive_code_blocks_first_unlabelled_second_labelled() {
	let content = "```\nabc\n```\n```ts\nxyz\n```\n";
	let expected = "```text\nabc\n```\n\n```ts\nxyz\n```\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_consecutive_code_blocks_first_labelled_second_unlabelled() {
	let content = "```ts\nxyz\n```\n```\nabc\n```\n";
	let expected = "```ts\nxyz\n```\n\n```text\nabc\n```\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_consecutive_code_blocks_with_blank_line_between() {
	let content = "```\nabc\n```\n\n```\nxyz\n```\n";
	let expected = "```text\nabc\n```\n\n```text\nxyz\n```\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_consecutive_code_blocks_separated_by_horizontal_rule() {
	let content = "```\nabc\n```\n---\n```ts\n```\nxxx\n```\nxyz\n```\n";
	let expected = "```text\nabc\n```\n\n```ts\n```\n\nxxx\n\n```text\nxyz\n```\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_code_block_default_text_language_nested_in_list() {
	let content = "- item:\n  ```\n  code\n  ```\n";
	let expected = "- item:\n  ```text\n  code\n  ```\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_empty_list_item_removed() {
	let content = "- abc\n- \n";
	let expected = "- abc\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_empty_list_item_middle_removed() {
	let content = "- a\n- \n- b\n";
	let expected = "- a\n- b\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_empty_list_item_ordered_removed() {
	let content = "1. a\n2. \n3. b\n";
	let expected = "1. a\n3. b\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_empty_list_item_at_start() {
	let content = "- \n- abc\n";
	let expected = "- abc\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_empty_list_item_multiple_consecutive() {
	let content = "- \n- \n- abc\n";
	let expected = "- abc\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_empty_list_item_only() {
	let content = "- \n";
	let result = format_markdown(content);
	assert_eq!(result, "");
}

#[test]
fn test_format_markdown_empty_list_item_asterisk_marker() {
	let content = "* a\n* \n";
	let expected = "* a\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_empty_list_item_plus_marker() {
	let content = "+ a\n+ \n";
	let expected = "+ a\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_empty_list_item_mixed_markers() {
	let content = "- a\n* \n- b\n";
	let expected = "- a\n- b\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_empty_list_item_ordered_parenthesis() {
	let content = "1) a\n2) \n";
	let expected = "1) a\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_empty_list_item_no_trailing_newline() {
	let content = "- abc\n- ";
	let expected = "- abc";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_empty_list_item_before_heading() {
	let content = "- a\n- \n# Heading\n";
	let expected = "- a\n\n# Heading\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_empty_list_item_before_paragraph() {
	let content = "- a\n- \nText\n";
	let expected = "- a\n\nText\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_empty_list_item_indented_nested() {
	let content = "- a\n  - \n  - sub\n- b\n";
	let expected = "- a\n  - sub\n- b\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_empty_list_item_with_blank_line() {
	let content = "- abc\n- \n\n";
	let expected = "- abc\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_list_code_block_content_marker_lines_preserved() {
	let content = "- item:\n  ```\n  - \n  - code\n  ```\n";
	let expected = "- item:\n  ```text\n  - \n  - code\n  ```\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_code_block_content_marker_lines_preserved() {
	let content = "```\n- \n- code\n```\n";
	let expected = "```text\n- \n- code\n```\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_nested_list_code_fence_language_preserved() {
	let content = "- item:\n  ```ts\n  code\n  ```\n";
	let expected = "- item:\n  ```ts\n  code\n  ```\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_nested_list_code_fence_deeper_indent() {
	let content = "- a\n  - b:\n    ```\n    c\n    ```\n";
	let expected = "- a\n  - b:\n    ```text\n    c\n    ```\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_list_multiple_code_blocks() {
	let content = "- a:\n  ```\n  x\n  ```\n- b:\n  ```\n  y\n  ```\n";
	let expected = "- a:\n  ```text\n  x\n  ```\n- b:\n  ```text\n  y\n  ```\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_list_fence_with_text_already_present() {
	let content = "- item:\n  ```text\n  code\n  ```\n";
	let expected = "- item:\n  ```text\n  code\n  ```\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_list_items_drops_empty_items() {
	let items = vec!["- a".to_string(), "- ".to_string(), "- b".to_string()];
	let result = super::lines::format_list_items(&items);
	assert_eq!(result, vec!["- a".to_string(), "- b".to_string()]);
}

#[test]
fn test_format_list_items_drops_empty_sub_items() {
	let items = vec!["- a".to_string(), "  - ".to_string(), "  - sub".to_string()];
	let result = super::lines::format_list_items(&items);
	assert_eq!(result, vec!["- a".to_string(), "  - sub".to_string()]);
}

#[test]
fn test_format_list_items_keeps_marker_lines_inside_code_block() {
	let items = vec![
		"- item:".to_string(),
		"  ```".to_string(),
		"  - ".to_string(),
		"  - code".to_string(),
		"  ```".to_string(),
	];
	let result = super::lines::format_list_items(&items);
	assert_eq!(result[2], "  - ");
	assert_eq!(result[3], "  - code");
	assert_eq!(result.len(), 5);
}

#[test]
fn test_format_markdown_folder_structure_text() {
	let content = r#"# Project Structure

```text
data/
│
├── input/     # input
│
├── output/    # output
│
└── logs/      # logs
```
"#;
	let expected = r#"# Project Structure

```text
data/
├─input/ # input
├─output/ # output
└─logs/ # logs
```
"#;
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_folder_structure_txt() {
	let content = r#"```txt
data/
│
├── input/     # input
│
└── logs/      # logs
```
"#;
	let expected = r#"```txt
data/
├─input/ # input
└─logs/ # logs
```
"#;
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_folder_structure_unlabelled() {
	let content = r#"```
data/
│
├── input/     # input
│
└── logs/      # logs
```
"#;
	let expected = r#"```text
data/
├─input/ # input
└─logs/ # logs
```
"#;
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_non_folder_code_block_unchanged() {
	let content = r#"```text
Just some arbitrary text
with │ vertical bar and stuff
```
"#;
	let expected = r#"```text
Just some arbitrary text
with │ vertical bar and stuff
```
"#;
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_inline_code_block_unchanged() {
	let content = "code block: `let a = 1;`\n";
	let expected = "code block: `let a = 1;`\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_inline_code_block_in_fenced_md() {
	let content = "```md\ncode block: `let a = 1;`\n```\n";
	let expected = "```md\ncode block: `let a = 1;`\n```\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_inline_code_block_in_table() {
	let content = "| Code | Description |\n|---|---|\n| `let a = 1;` | Variable assignment |\n";
	let expected = "| Code | Description |\n|---|---|\n| `let a = 1;` | Variable assignment |\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_inline_code_block_in_list() {
	let content = "- code block: `let a = 1;`\n";
	let expected = "- code block: `let a = 1;`\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_html_minification_user_case() {
	let content = r#"<p align="center">
  <img src="https://img.shields.io/badge/Markdown-000000?style=for-the-badge&logo=markdown&logoColor=white" alt="Markdown" />
</p>
"#;
	let expected = r#"<p align="center">
<img src="https://img.shields.io/badge/Markdown-000000?style=for-the-badge&logo=markdown&logoColor=white" alt="Markdown" /></p>
"#;
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_html_multiline_tag() {
	let content = r#"<p
  align="center"
>
  <img
    src="https://img.shields.io/badge/Markdown-000000"
    alt="Markdown"
  />
</p>
"#;
	let expected = r#"<p align="center">
<img src="https://img.shields.io/badge/Markdown-000000" alt="Markdown" /></p>
"#;
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_html_preserves_spaces_in_attribute_values() {
	let content = r#"<p align="center">
  <img src="badge.png" alt="Badge with   multiple   spaces" />
</p>
"#;
	let expected = r#"<p align="center">
<img src="badge.png" alt="Badge with   multiple   spaces" /></p>
"#;
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_html_nested_elements() {
	let content = r#"<div align="center">
  <a href="https://example.com">
    <img src="logo.svg" alt="logo" width="128" height="128" />
  </a>
</div>
"#;
	let expected = r#"<div align="center">
<a href="https://example.com">
<img src="logo.svg" alt="logo" width="128" height="128" /></a></div>
"#;
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_html_multiple_children() {
	let content = r#"<p align="center">
  <img src="badge1.png" alt="1" />
  <img src="badge2.png" alt="2" />
</p>
"#;
	let expected = r#"<p align="center">
<img src="badge1.png" alt="1" />
<img src="badge2.png" alt="2" /></p>
"#;
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_html_inline_tag_in_text() {
	let content = "Click <a   href=\"https://example.com\"   target=\"_blank\"  >here</a> to view.";
	let expected = "Click <a href=\"https://example.com\" target=\"_blank\">here</a> to view.";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_html_disabled_option() {
	let content = r#"<p align="center">
  <img src="badge.png" alt="Badge" />
</p>
"#;
	let options = FormatOptions {
		minify_html: false,
		..FormatOptions::default()
	};
	let result = format_markdown_with_options(content, options);
	assert_eq!(result, content);
}

#[test]
fn test_format_markdown_html_in_code_block_preserved() {
	let content = r#"```html
<p align="center">
  <img src="test.png" />
</p>
```
"#;
	let result = format_markdown(content);
	assert_eq!(result, content);
}

#[test]
fn test_format_markdown_html_comments_preserved() {
	let content = "<!-- Input: -->\n\n<p align=\"center\">\n  <img src=\"badge.png\" alt=\"Badge\" />\n</p>\n";
	let expected =
		"<!-- Input: -->\n\n<p align=\"center\">\n<img src=\"badge.png\" alt=\"Badge\" /></p>\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_preserves_autolink_in_sentence() {
	let content = "Root development tools are managed by `mise.toml` - <https://mise.en.dev/getting-started.html>.\n";
	let result = format_markdown(content);
	assert_eq!(result, content);
}

#[test]
fn test_format_markdown_preserves_various_autolinks() {
	let content =
		"# Links\n\nSee <http://example.com/path?query=1#hash> and <mailto:contact@domain.org>.\n";
	let result = format_markdown(content);
	assert_eq!(result, content);
}

#[test]
fn test_format_markdown_preserves_email_autolink() {
	let content = "Contact us at <support@example.com>.\n";
	let result = format_markdown(content);
	assert_eq!(result, content);
}

#[test]
fn test_format_markdown_preserves_url_underscores_in_link_destination() {
	let content = "[Package](https://github.com/org/repo/blob/main/__init__.py)\n";
	let result = format_markdown(content);
	assert_eq!(result, content);
}

#[test]
fn test_format_markdown_strips_bold_in_link_label_but_preserves_url() {
	let content = "[**Package**](https://github.com/org/repo/blob/main/__init__.py)\n";
	let expected = "[Package](https://github.com/org/repo/blob/main/__init__.py)\n";
	let result = format_markdown(content);
	assert_eq!(result, expected);
}

#[test]
fn test_format_markdown_preserves_url_in_reference_link() {
	let content = "[1]: https://example.com/__init__.py\n";
	let result = format_markdown(content);
	assert_eq!(result, content);
}

#[test]
fn test_format_markdown_preserves_url_in_html_attribute() {
	let content = "Click <a href=\"https://example.com/__init__.py\">here</a>.\n";
	let result = format_markdown(content);
	assert_eq!(result, content);
}

#[test]
fn test_format_markdown_preserves_url_underscores_in_table() {
	let content =
		"| Name | Link |\n|---|---|\n| File | [Init](https://example.com/__init__.py) |\n";
	let result = format_markdown(content);
	assert_eq!(result, content);
}
