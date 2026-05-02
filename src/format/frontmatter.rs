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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_is_frontmatter_start_with_valid_start() {
		assert!(is_frontmatter_start(&["---"]));
		assert!(is_frontmatter_start(&["---", "title: Test"]));
	}

	#[test]
	fn test_is_frontmatter_start_with_whitespace() {
		assert!(is_frontmatter_start(&["  ---  "]));
	}

	#[test]
	fn test_is_frontmatter_start_empty_lines() {
		assert!(!is_frontmatter_start(&[]));
	}

	#[test]
	fn test_is_frontmatter_start_no_frontmatter() {
		assert!(!is_frontmatter_start(&["# Heading"]));
		assert!(!is_frontmatter_start(&["not frontmatter"]));
	}

	#[test]
	fn test_is_frontmatter_end_valid() {
		assert!(is_frontmatter_end("---", 1));
		assert!(is_frontmatter_end("  ---  ", 2));
	}

	#[test]
	fn test_is_frontmatter_end_at_index_zero() {
		assert!(!is_frontmatter_end("---", 0));
	}

	#[test]
	fn test_is_frontmatter_end_not_delimiter() {
		assert!(!is_frontmatter_end("--- ", 0));
		assert!(!is_frontmatter_end("text", 1));
		assert!(!is_frontmatter_end("----", 1));
	}
}
