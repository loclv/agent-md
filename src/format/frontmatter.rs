/// Check if the content starts with a YAML frontmatter block.
/// Returns true if the first line is "---".
pub fn is_frontmatter_start(lines: &[&str]) -> bool {
	!lines.is_empty() && lines[0].trim() == "---"
}

/// Check if a line is the closing delimiter of a frontmatter block.
/// Returns true if the line is "---" and it's not the first line (i > 0).
pub fn is_frontmatter_end(line: &str, line_index: usize) -> bool {
	line_index > 0 && line.trim() == "---"
}
