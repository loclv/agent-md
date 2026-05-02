mod blockquotes;
mod bold_tables;
mod code_blocks;
mod frontmatter;
mod tables;

use crate::*;

#[derive(Clone)]
pub struct FormatOptions {
	pub remove_bold: bool,
	pub compact_blank_lines: bool,
	pub trim_trailing_whitespace: bool,
	pub collapse_spaces: bool,
	pub remove_horizontal_rules: bool,
	pub remove_emphasis: bool,
}

impl FormatOptions {
	#[allow(dead_code)]
	pub fn token_saver() -> Self {
		Self {
			remove_bold: true,
			compact_blank_lines: true,
			trim_trailing_whitespace: true,
			collapse_spaces: true,
			remove_horizontal_rules: true,
			remove_emphasis: true,
		}
	}
}

impl Default for FormatOptions {
	fn default() -> Self {
		Self {
			remove_bold: true,
			compact_blank_lines: true,
			trim_trailing_whitespace: true,
			collapse_spaces: true,
			remove_horizontal_rules: true,
			remove_emphasis: true,
		}
	}
}

#[allow(dead_code)]
pub fn format_markdown(content: &str) -> String {
	format_markdown_with_options(content, FormatOptions::default())
}

pub fn format_markdown_with_options(content: &str, options: FormatOptions) -> String {
	let ends_with_newline = content.ends_with('\n');

	// First pass: collect all lines to determine which blank lines to keep
	let lines: Vec<&str> = content.lines().collect();
	let mut formatted_lines = Vec::new();
	let mut in_code_block = false;
	let mut code_block_lang: Option<String> = None;
	// YAML frontmatter detection: starts with --- at line 0
	let mut in_frontmatter = frontmatter::is_frontmatter_start(&lines);

	for (i, line) in lines.iter().enumerate() {
		let trimmed = line.trim();

		// Handle YAML frontmatter block
		if in_frontmatter {
			// Preserve all lines in frontmatter block as-is
			formatted_lines.push(line.to_string());
			// Check for closing delimiter (--- on its own line, after the opening)
			if frontmatter::is_frontmatter_end(trimmed, i) {
				in_frontmatter = false;
			}
			continue;
		}

		if trimmed.starts_with("```") {
			in_code_block = !in_code_block;
			if in_code_block {
				// Extract language from the opening fence
				code_block_lang = trimmed.strip_prefix("```").map(|s| s.to_lowercase());
			} else {
				code_block_lang = None;
			}
			formatted_lines.push(line.to_string());
			continue;
		}

		if in_code_block {
			if let Some(ref lang) = code_block_lang {
				// For bash/sh code blocks, collapse spaces before # comments
				if code_blocks::is_shell_language(lang) {
					let processed = code_blocks::collapse_spaces_before_comment(line);
					formatted_lines.push(processed);
					continue;
				}
				// For markdown code blocks, apply formatting rules to the content
				if lang == "markdown" || lang == "md" {
					// Process the line as regular markdown
					let processed = process_markdown_line(line, &options, false);
					formatted_lines.push(processed);
					continue;
				}
			}
			formatted_lines.push(line.to_string());
			continue;
		}

		if options.remove_horizontal_rules && is_horizontal_rule(trimmed) {
			continue;
		}

		let is_heading = trimmed.starts_with('#');

		// Use shared function for line processing
		let processed_line = process_markdown_line(line, &options, is_heading);
		let processed_trimmed = processed_line.trim();

		// Handle blank lines with compact_blank_lines option
		if options.compact_blank_lines && processed_trimmed.is_empty() {
			if let Some(prev) = formatted_lines.last() {
				if !prev.is_empty() {
					// Check if next non-empty line is a heading - if so, preserve blank line
					let next_is_heading = lines[i + 1..]
						.iter()
						.find(|l| !l.trim().is_empty())
						.is_some_and(|l| l.trim().starts_with('#'));
					if next_is_heading {
						formatted_lines.push(String::new());
						continue;
					}

					// Check if next non-empty line is a code fence - if so, preserve blank line
					let next_is_code_fence = lines[i + 1..]
						.iter()
						.find(|l| !l.trim().is_empty())
						.is_some_and(|l| l.trim().starts_with("```"));
					if next_is_code_fence {
						formatted_lines.push(String::new());
						continue;
					}

					// Check if previous line was a heading - if so, preserve blank line
					let prev_was_heading = prev.trim().starts_with('#');
					if prev_was_heading {
						formatted_lines.push(String::new());
						continue;
					}

					let needs_line = formatted_lines
						.iter()
						.rev()
						.nth(1)
						.map(|l| !l.trim().is_empty())
						.unwrap_or(true);
					if needs_line || !formatted_lines.iter().rev().take(2).any(|l| l.is_empty()) {
						formatted_lines.push(String::new());
					}
					continue;
				}
			}
		}

		formatted_lines.push(processed_line);
	}

	let mut result = formatted_lines.join("\n");

	if options.compact_blank_lines {
		let lines: Vec<&str> = result.lines().collect();
		let mut compact_lines = Vec::new();
		let mut prev_was_empty = false;

		for line in lines {
			let is_empty = line.trim().is_empty();
			if is_empty && prev_was_empty {
				continue;
			}
			compact_lines.push(line);
			prev_was_empty = is_empty;
		}
		result = compact_lines.join("\n");
	}

	if ends_with_newline && !result.ends_with('\n') {
		result.push('\n');
	}

	result
}

/// Process a single markdown line with formatting options.
/// Used for both regular lines and lines inside ```markdown code blocks.
fn process_markdown_line(line: &str, options: &FormatOptions, is_heading: bool) -> String {
	// Handle tables
	let (prefix, table_content) = tables::parse_table_line(line);
	if !table_content.is_empty() {
		if tables::is_separator_row(table_content) {
			let compacted = tables::compact_separator_row(table_content);
			return format!("{}{}", prefix, compacted);
		} else {
			return tables::format_table_row(prefix, table_content, options.remove_bold);
		}
	}

	// Process regular line
	let mut processed_line = if options.remove_bold {
		remove_bold_markers(line)
	} else {
		line.to_string()
	};

	if options.remove_emphasis && !is_heading {
		processed_line = remove_emphasis_markers(&processed_line);
	}

	if options.collapse_spaces && !is_heading {
		processed_line = collapse_multiple_spaces(&processed_line);
	}

	// Normalize blockquote lines (remove extra spaces after > markers)
	processed_line = blockquotes::normalize_blockquote(&processed_line);

	if options.trim_trailing_whitespace {
		processed_line = processed_line.trim_end().to_string();
	}

	processed_line
}

/// Remove bold markers (** and __) from a line while preserving the content inside.
/// This skips markers inside inline code spans.
fn remove_bold_markers(line: &str) -> String {
	let mut result = String::new();
	let chars: Vec<char> = line.chars().collect();
	let mut i = 0;

	while i < chars.len() {
		// Check for inline code start
		if chars[i] == '`' {
			// Find the end of the inline code
			let mut code_end = i + 1;
			while code_end < chars.len() {
				if chars[code_end] == '`' {
					break;
				}
				code_end += 1;
			}
			// Copy the entire code span as-is
			for j in i..=code_end {
				if j < chars.len() {
					result.push(chars[j]);
				}
			}
			i = code_end + 1;
			continue;
		}

		// Check for **bold** pattern
		if i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '*' {
			// Find the closing **
			let mut j = i + 2;
			while j + 1 < chars.len() {
				if chars[j] == '*' && chars[j + 1] == '*' {
					// Found closing marker, copy content between markers
					chars[i + 2..j].iter().for_each(|&c| result.push(c));
					i = j + 2;
					break;
				}
				j += 1;
			}
			// If no closing marker found, copy the ** as-is
			if i <= j {
				result.push(chars[i]);
				i += 1;
			}
			continue;
		}

		// Check for __bold__ pattern
		if i + 1 < chars.len() && chars[i] == '_' && chars[i + 1] == '_' {
			// Find the closing __
			let mut j = i + 2;
			while j + 1 < chars.len() {
				if chars[j] == '_' && chars[j + 1] == '_' {
					// Found closing marker, copy content between markers
					chars[i + 2..j].iter().for_each(|&c| result.push(c));
					i = j + 2;
					break;
				}
				j += 1;
			}
			// If no closing marker found, copy the __ as-is
			if i <= j {
				result.push(chars[i]);
				i += 1;
			}
			continue;
		}

		// Regular character, just copy it
		result.push(chars[i]);
		i += 1;
	}

	result
}

fn is_horizontal_rule(line: &str) -> bool {
	let trimmed = line.trim();
	trimmed == "---" || trimmed == "***" || trimmed == "___"
}

fn remove_emphasis_markers(line: &str) -> String {
	let mut result = String::new();
	let chars: Vec<char> = line.chars().collect();
	let mut i = 0;

	while i < chars.len() {
		if chars[i] == '`' {
			let mut code_end = i + 1;
			while code_end < chars.len() {
				if chars[code_end] == '`' {
					break;
				}
				code_end += 1;
			}
			for j in i..=code_end {
				if j < chars.len() {
					result.push(chars[j]);
				}
			}
			i = code_end + 1;
			continue;
		}

		if chars[i] == '[' {
			let mut bracket_end = i;
			while bracket_end < chars.len() && chars[bracket_end] != ']' {
				bracket_end += 1;
			}
			for j in i..=bracket_end {
				if j < chars.len() {
					result.push(chars[j]);
				}
			}
			i = bracket_end + 1;
			continue;
		}

		if i + 1 < chars.len()
			&& ((chars[i] == '*' && chars[i + 1] != '*')
				|| (chars[i] == '_' && chars[i + 1] != '_'))
		{
			let marker = chars[i];

			// For underscore, check if it's part of an identifier (e.g., A_cat_meow)
			// Only skip if BOTH sides are alphanumeric (underscore within a word)
			// _word_ at boundaries should still be treated as emphasis
			if marker == '_' {
				let prev_is_word = i > 0 && chars[i - 1].is_alphanumeric();
				let next_is_word = i + 1 < chars.len() && chars[i + 1].is_alphanumeric();
				if prev_is_word && next_is_word {
					// This underscore is within a word (identifier), skip it
					result.push(chars[i]);
					i += 1;
					continue;
				}
			}

			let mut j = i + 1;
			while j < chars.len() && chars[j] != marker {
				j += 1;
			}
			if j < chars.len() && chars[j] == marker {
				// For underscore, also check the closing marker isn't within a word
				if marker == '_' {
					let prev_is_word = j > 0 && chars[j - 1].is_alphanumeric();
					let next_is_word = j + 1 < chars.len() && chars[j + 1].is_alphanumeric();
					if prev_is_word && next_is_word {
						// Closing underscore is within a word (identifier), skip this match
						result.push(chars[i]);
						i += 1;
						continue;
					}
				}
				chars[i + 1..j].iter().for_each(|&c| result.push(c));
				i = j + 1;
			} else {
				result.push(chars[i]);
				i += 1;
			}
			continue;
		}

		result.push(chars[i]);
		i += 1;
	}

	result
}

fn collapse_multiple_spaces(line: &str) -> String {
	// Preserve leading whitespace (indentation)
	let leading_len = line.chars().take_while(|&c| c == ' ').count();
	let leading = &line[..leading_len];
	let rest = &line[leading_len..];

	let mut result = String::from(leading);
	let mut prev_was_space = false;

	for c in rest.chars() {
		if c == ' ' {
			if !prev_was_space {
				result.push(c);
			}
			prev_was_space = true;
		} else {
			result.push(c);
			prev_was_space = false;
		}
	}

	result
}

fn collect_markdown_files(
	dir: &std::path::Path,
	files: &mut Vec<std::path::PathBuf>,
) -> std::io::Result<()> {
	for entry in std::fs::read_dir(dir)? {
		let entry = entry?;
		let path = entry.path();
		if path.is_dir() {
			collect_markdown_files(&path, files)?;
		} else if let Some(ext) = path.extension() {
			if ext == "md" || ext == "markdown" {
				files.push(path);
			}
		}
	}
	Ok(())
}

fn format_single_file(path: &str, options: FormatOptions) -> Result<Document, String> {
	let content =
		std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;
	let formatted_content = format_markdown_with_options(&content, options);
	std::fs::write(path, &formatted_content).map_err(|e| format!("Failed to write file: {}", e))?;
	let mut doc = parse_markdown(&formatted_content);
	doc.path = path.to_string();
	Ok(doc)
}

pub fn cmd_fmt(path: &str, human: bool, options: FormatOptions) {
	let path_buf = std::path::PathBuf::from(path);

	if path_buf.is_dir() {
		let mut files = Vec::new();
		if let Err(e) = collect_markdown_files(&path_buf, &mut files) {
			println!(
				"{}",
				json_output(
					&EditResult {
						success: false,
						message: format!("Failed to read directory: {}", e),
						document: None,
					},
					human
				)
			);
			return;
		}

		files.sort();

		let mut success_count = 0;
		let mut error_count = 0;

		for file in &files {
			let file_path = file.to_string_lossy().to_string();
			match format_single_file(&file_path, options.clone()) {
				Ok(doc) => {
					success_count += 1;
					println!(
						"{}",
						json_output(
							&EditResult {
								success: true,
								message: format!("File formatted successfully: {}", file_path),
								document: Some(doc),
							},
							human
						)
					);
				}
				Err(e) => {
					error_count += 1;
					println!(
						"{}",
						json_output(
							&EditResult {
								success: false,
								message: e,
								document: None,
							},
							human
						)
					);
				}
			}
		}

		let summary = format!(
			"Formatted {} files ({} succeeded, {} failed)",
			files.len(),
			success_count,
			error_count
		);
		println!(
			"{}",
			json_output(
				&EditResult {
					success: error_count == 0,
					message: summary,
					document: None,
				},
				human
			)
		);
	} else {
		match format_single_file(path, options) {
			Ok(doc) => {
				println!(
					"{}",
					json_output(
						&EditResult {
							success: true,
							message: "File formatted successfully".to_string(),
							document: Some(doc),
						},
						human
					)
				);
			}
			Err(e) => {
				println!(
					"{}",
					json_output(
						&EditResult {
							success: false,
							message: e,
							document: None,
						},
						human
					)
				);
			}
		}
	}
}

/// Format markdown from stdin and write to stdout (Prettier-style).
/// This is used by editor integrations to avoid file sync conflicts.
///
/// # Example
///
/// ```bash
/// echo '# Hello\n\n**bold** text' | agent-md fmt --stdin
/// # Output: # Hello\n\nbold text
/// ```
pub fn cmd_fmt_stdin(options: FormatOptions) {
	use std::io::Read;

	let mut input = String::new();
	if let Err(e) = std::io::stdin().read_to_string(&mut input) {
		eprintln!("Error reading stdin: {}", e);
		std::process::exit(1);
	}

	let formatted = format_markdown_with_options(&input, options);
	print!("{}", formatted);
}

#[cfg(test)]
mod tests {
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
| Value 1 |  | Value 3 |
|  | Value 2 |  |
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
| Jane | 30 | London |  |
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

	// Additional comprehensive tests
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
			},
		);
		assert_eq!(result, "\n\n");
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
		let expected = "| A | B | C |\n|---|---|---|\n|  |  |  |\n";
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
		let expected = r#"```
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
		let expected = "| Left | Center | Right |\n|:---|:---:|---:|\n| A | B | C |\n";
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
		assert!(result.contains("```\n**bold in code**\n```"));
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
		};
		let result = format_markdown_with_options(content, options);
		assert_eq!(result, "Line 1\n\nLine 2\nLine 3");
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
		};
		let result = format_markdown_with_options(content, options);
		assert_eq!(result, "# Heading   with    spaces");
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
	fn test_format_markdown_compact_separator_dashes() {
		let content = "| Real | Table |\n|----|----|\n| Has | dashes |\n";
		let expected = "| Real | Table |\n|---|---|\n| Has | dashes |\n";
		let result = format_markdown(content);
		assert_eq!(result, expected);
	}

	#[test]
	fn test_format_markdown_compact_separator_preserve_alignment() {
		let content = "| Left | Center | Right |\n|:-----|:------:|------:|\n| A | B | C |\n";
		let expected = "| Left | Center | Right |\n|:---|:---:|---:|\n| A | B | C |\n";
		let result = format_markdown(content);
		assert_eq!(result, expected);
	}

	#[test]
	fn test_format_markdown_preserve_list_indentation() {
		let content = "lists:\n  - 2 spaces\n  - 2 spaces\n";
		let expected = "lists:\n  - 2 spaces\n  - 2 spaces\n";
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
		let result = format_markdown(content);
		assert_eq!(result, content);
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
}
