/// Compact separator row dashes to exactly 3, preserving alignment colons.
/// |----|-----| becomes |---|---|
/// |:---|:--:|--:| becomes |:---|:--:|--:|
pub fn compact_separator_row(table_content: &str) -> String {
	let cells: Vec<&str> = table_content.split('|').collect();
	let mut formatted_cells = Vec::new();

	for (i, cell) in cells.iter().enumerate() {
		if i == 0 || i == cells.len() - 1 {
			continue;
		}
		let cell_trimmed = cell.trim();
		let has_left_colon = cell_trimmed.starts_with(':');
		let has_right_colon = cell_trimmed.ends_with(':');

		let compacted = if has_left_colon && has_right_colon {
			// Center alignment: :---:
			":---:"
		} else if has_left_colon {
			// Left alignment: :---
			":---"
		} else if has_right_colon {
			// Right alignment: ---:
			"---:"
		} else {
			// No alignment: ---
			"---"
		};
		formatted_cells.push(compacted.to_string());
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

	if trimmed.starts_with('|') && trimmed.ends_with('|') {
		(indent_str, trimmed)
	} else if (trimmed.starts_with("> |") || trimmed.starts_with("- |")) && trimmed.ends_with('|') {
		let table_start = trimmed.find('|').unwrap_or(0);
		let prefix_part = &trimmed[..table_start];
		let table_part = &trimmed[table_start..];
		(&line[..leading_indent + prefix_part.len()], table_part)
	} else {
		("", "")
	}
}

/// Format a table row, trimming cell content and standardizing spacing.
pub fn format_table_row(prefix: &str, table_content: &str) -> String {
	let cells: Vec<&str> = table_content.split('|').collect();
	let mut formatted_cells = Vec::new();

	for (i, cell) in cells.iter().enumerate() {
		if i == 0 || i == cells.len() - 1 {
			continue;
		}
		let cell_trimmed = cell.trim();
		formatted_cells.push(cell_trimmed.to_string());
	}

	format!("{}| {} |", prefix, formatted_cells.join(" | "))
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
		let expected = "|:---|:---:|---:|";
		assert_eq!(compact_separator_row(content), expected);
	}

	#[test]
	fn test_compact_separator_row_center_only() {
		let content = "|:--:|";
		let expected = "|:---:|";
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
		let result = format_table_row("", "| Header | Value |");
		assert_eq!(result, "| Header | Value |");
	}

	#[test]
	fn test_format_table_row_trailing_spaces() {
		let result = format_table_row("", "| Header  | Value   |");
		assert_eq!(result, "| Header | Value |");
	}

	#[test]
	fn test_format_table_row_with_prefix() {
		let result = format_table_row("  ", "| Header | Value |");
		assert_eq!(result, "  | Header | Value |");
	}
}
