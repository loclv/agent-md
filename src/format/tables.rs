// `| **text** | center | right |` becomes `| text | center | right |`

use super::bold_tables::strip_bold_from_cell;

/// Check whether the character at `idx` is escaped by an odd number of preceding backslashes.
///
/// # Arguments
///
/// * `chars` - The character slice
/// * `idx` - The index of the character to check
///
/// # Returns
///
/// * `bool` - `true` if the character is preceded by an odd number of contiguous backslashes
pub fn is_char_escaped(chars: &[char], idx: usize) -> bool {
	let mut count = 0;
	let mut i = idx;
	while i > 0 && chars[i - 1] == '\\' {
		count += 1;
		i -= 1;
	}
	count % 2 != 0
}

/// Check if a string ends with an unescaped pipe character (`|`).
///
/// # Arguments
///
/// * `s` - The string slice to check
///
/// # Returns
///
/// * `bool` - `true` if the string ends with an unescaped `|`
pub fn ends_with_unescaped_pipe(s: &str) -> bool {
	if !s.ends_with('|') {
		return false;
	}
	let chars: Vec<char> = s.chars().collect();
	if chars.is_empty() {
		return false;
	}
	!is_char_escaped(&chars, chars.len() - 1)
}

/// Split table content into cells delimited by unescaped `|` characters outside inline code spans.
///
/// Escaped pipes (`\|`) are treated as literal characters within cell content (for example representing
/// an "or" condition), and pipes inside code spans (e.g., `` `a | b` ``) are preserved within the cell.
///
/// # Arguments
///
/// * `table_content` - The table content line to split
///
/// # Returns
///
/// * `Vec<&str>` - Slices between delimiter pipes
pub fn split_table_cells(table_content: &str) -> Vec<&str> {
	let chars: Vec<char> = table_content.chars().collect();
	let byte_offsets: Vec<usize> = table_content.char_indices().map(|(pos, _)| pos).collect();
	let mut delimiter_byte_indices = Vec::new();
	let mut i = 0;

	while i < chars.len() {
		// Skip inline code spans so pipes within code are not treated as cell delimiters
		if chars[i] == '`' && !is_char_escaped(&chars, i) {
			if let Some(code_end) = crate::format::lines::find_code_span_end(&chars, i) {
				i = code_end + 1;
				continue;
			}
		}

		if chars[i] == '|' && !is_char_escaped(&chars, i) {
			delimiter_byte_indices.push(byte_offsets[i]);
		}
		i += 1;
	}

	let mut cells = Vec::new();
	let mut prev_byte = 0;

	for &delim_byte in &delimiter_byte_indices {
		cells.push(&table_content[prev_byte..delim_byte]);
		prev_byte = delim_byte + 1;
	}
	cells.push(&table_content[prev_byte..]);

	cells
}

/// Compact separator row dashes to exactly 3, preserving alignment colons.
/// |----|-----| becomes |---|---|
/// |:---|:--:|--:| becomes |:---|:--:|--:|
///
/// # Arguments
///
/// * `table_content` - The table content to compact
///
/// # Returns
///
/// * `String` - The compacted table content
pub fn compact_separator_row(table_content: &str) -> String {
	let cells = split_table_cells(table_content);
	let mut formatted_cells = Vec::new();

	for (i, _cell) in cells.iter().enumerate() {
		if i == 0 || i == cells.len() - 1 {
			continue;
		}
		// Always use "---" to save tokens and normalize, removing any alignment colons
		formatted_cells.push("---".to_string());
	}

	format!("|{}|", formatted_cells.join("|"))
}

/// Check if a line is a table separator row (contains only |, -, :, and spaces)
pub fn is_separator_row(table_content: &str) -> bool {
	table_content
		.chars()
		.all(|c| c == '|' || c == '-' || c == ' ' || c == ':')
		&& table_content.contains('-')
}

/// Parse table line and return (prefix, table_content) tuple.
/// Returns ("", "") if not a table line.
pub fn parse_table_line(line: &str) -> (&str, &str) {
	let trimmed = line.trim();
	let leading_indent = line.len() - line.trim_start().len();
	let indent_str = &line[..leading_indent];

	if trimmed.len() >= 2 && trimmed.starts_with('|') && ends_with_unescaped_pipe(trimmed) {
		(indent_str, trimmed)
	} else if (trimmed.starts_with("> |") || trimmed.starts_with("- |"))
		&& ends_with_unescaped_pipe(trimmed)
	{
		let table_start = trimmed.find('|').unwrap_or(0);
		let prefix_part = &trimmed[..table_start];
		let table_part = &trimmed[table_start..];
		(&line[..leading_indent + prefix_part.len()], table_part)
	} else {
		("", "")
	}
}

/// Format a table row, trimming cell content and standardizing spacing.
/// When `remove_bold` is true, strips `**` and `__` markers from cell text.
/// Escaped pipes (`\|`) within cells are preserved without inserting spaces.
pub fn format_table_row(prefix: &str, table_content: &str, remove_bold: bool) -> String {
	let cells = split_table_cells(table_content);
	let mut formatted_cells = Vec::new();

	for (i, cell) in cells.iter().enumerate() {
		if i == 0 || i == cells.len() - 1 {
			continue;
		}
		let mut cell_trimmed = cell.trim().to_string();
		if remove_bold {
			cell_trimmed = strip_bold_from_cell(&cell_trimmed);
		}
		if cell_trimmed.is_empty() {
			formatted_cells.push(" ".to_string());
		} else {
			formatted_cells.push(format!(" {} ", cell_trimmed));
		}
	}

	format!("{}|{}|", prefix, formatted_cells.join("|"))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_compact_separator_row_basic() {
		let content = "|----|-----|";
		let expected = "|---|---|";
		assert_eq!(compact_separator_row(content), expected);
	}

	#[test]
	fn test_compact_separator_row_with_alignment() {
		let content = "|:---|:--:|---:|";
		let expected = "|---|---|---|";
		assert_eq!(compact_separator_row(content), expected);
	}

	#[test]
	fn test_compact_separator_row_center_only() {
		let content = "|:--:|";
		let expected = "|---|";
		assert_eq!(compact_separator_row(content), expected);
	}

	#[test]
	fn test_is_separator_row_basic() {
		assert!(is_separator_row("|---|---|"));
		assert!(is_separator_row("|----|-----|"));
		assert!(is_separator_row("|:---|:--:|---:|"));
	}

	#[test]
	fn test_is_separator_row_not_separator() {
		assert!(!is_separator_row("| Header | Value |"));
		assert!(!is_separator_row(""));
		assert!(!is_separator_row("| text |"));
	}

	#[test]
	fn test_parse_table_line_basic() {
		let line = "| Header | Value |";
		let (prefix, content) = parse_table_line(line);
		assert_eq!(prefix, "");
		assert_eq!(content, "| Header | Value |");
	}

	#[test]
	fn test_parse_table_line_indented() {
		let line = "  | Header | Value |";
		let (prefix, content) = parse_table_line(line);
		assert_eq!(prefix, "  ");
		assert_eq!(content, "| Header | Value |");
	}

	#[test]
	fn test_parse_table_line_blockquote() {
		let line = "> | Quote | Table |";
		let (prefix, content) = parse_table_line(line);
		assert_eq!(prefix, "> ");
		assert_eq!(content, "| Quote | Table |");
	}

	#[test]
	fn test_parse_table_line_not_table() {
		let line = "This is not a table";
		let (prefix, content) = parse_table_line(line);
		assert_eq!(prefix, "");
		assert_eq!(content, "");
	}

	#[test]
	fn test_format_table_row_basic() {
		let result = format_table_row("", "| Header | Value |", false);
		assert_eq!(result, "| Header | Value |");
	}

	#[test]
	fn test_format_table_row_trailing_spaces() {
		let result = format_table_row("", "| Header  | Value   |", false);
		assert_eq!(result, "| Header | Value |");
	}

	#[test]
	fn test_format_table_row_with_prefix() {
		let result = format_table_row("  ", "| Header | Value |", false);
		assert_eq!(result, "  | Header | Value |");
	}

	#[test]
	fn test_format_table_row_removes_bold_asterisks() {
		let result = format_table_row("", "| **text** | center | right |", true);
		assert_eq!(result, "| text | center | right |");
	}

	#[test]
	fn test_format_table_row_removes_bold_underscores() {
		let result = format_table_row("", "| __text__ | value |", true);
		assert_eq!(result, "| text | value |");
	}

	#[test]
	fn test_format_table_row_preserves_bold_in_inline_code() {
		let result = format_table_row("", "| `**text**` | normal |", true);
		assert_eq!(result, "| `**text**` | normal |");
	}

	#[test]
	fn test_format_table_row_bold_disabled() {
		let result = format_table_row("", "| **text** | value |", false);
		assert_eq!(result, "| **text** | value |");
	}

	#[test]
	fn test_format_table_row_empty_cells() {
		let result = format_table_row("", "| c1 | |", false);
		assert_eq!(result, "| c1 | |");
	}

	#[test]
	fn test_format_table_row_escaped_pipe() {
		let result = format_table_row("", "| cell \\| cell | other |", false);
		assert_eq!(result, "| cell \\| cell | other |");
	}

	#[test]
	fn test_format_table_row_escaped_pipe_multiple() {
		let result = format_table_row("", "| a \\| b \\| c | d |", false);
		assert_eq!(result, "| a \\| b \\| c | d |");
	}

	#[test]
	fn test_format_table_row_escaped_backslash_before_pipe() {
		let result = format_table_row("", "| a\\\\|b |", false);
		assert_eq!(result, "| a\\\\ | b |");
	}

	#[test]
	fn test_split_table_cells_inline_code_pipe() {
		let cells = split_table_cells("| `a | b` | c |");
		assert_eq!(cells, vec!["", " `a | b` ", " c ", ""]);
	}

	#[test]
	fn test_parse_table_line_escaped_pipe_not_end() {
		let (prefix, content) = parse_table_line("| text \\|");
		assert_eq!(prefix, "");
		assert_eq!(content, "");
	}
}
