/// Remove ** and __ bold markers from a single table cell, preserving inline code.
///
/// # Arguments
///
/// * `cell` - The table cell content as a string slice
///
/// # Returns
///
/// * `String` - The cell content with bold markers removed
///
/// # Examples
///
/// ```
/// use agent_md::format::bold_tables::strip_bold_from_cell;
/// assert_eq!(strip_bold_from_cell("**hello**"), "hello");
/// assert_eq!(strip_bold_from_cell("__hello__"), "hello");
/// assert_eq!(strip_bold_from_cell("`__x__`"), "`__x__`");
/// assert_eq!(strip_bold_from_cell("| **hello** | __world__ | `code` |"), "| hello | world | `code` |");
/// ```
pub fn strip_bold_from_cell(cell: &str) -> String {
	let mut result = String::new();
	let chars: Vec<char> = cell.chars().collect();
	let mut i = 0;

	while i < chars.len() {
		// Preserve inline code spans unchanged
		if chars[i] == '`' {
			if let Some(code_end) = super::lines::find_code_span_end(&chars, i) {
				for &c in &chars[i..=code_end] {
					result.push(c);
				}
				i = code_end + 1;
				continue;
			}
		}

		// Skip HTML tags and autolinks
		if chars[i] == '<' && i + 1 < chars.len() {
			let next_c = chars[i + 1];
			if next_c.is_ascii_alphabetic() || next_c == '/' || next_c == '!' || next_c == '?' {
				if let Some(tag_end) = crate::format::html::find_tag_end(&chars, i) {
					for &c in &chars[i..=tag_end] {
						result.push(c);
					}
					i = tag_end + 1;
					continue;
				}
			}
		}

		// Skip link destination in [text](url) or ![alt](url)
		if chars[i] == ']' && i + 1 < chars.len() && chars[i + 1] == '(' {
			result.push(']');
			if let Some(dest_end) = super::lines::find_link_destination_end(&chars, i + 1) {
				for &c in &chars[i + 1..=dest_end] {
					result.push(c);
				}
				i = dest_end + 1;
				continue;
			} else {
				i += 1;
				continue;
			}
		}

		// Check for **bold** pattern
		if i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '*' {
			let mut j = i + 2;
			while j + 1 < chars.len() {
				if chars[j] == '*' && chars[j + 1] == '*' {
					chars[i + 2..j].iter().for_each(|&c| result.push(c));
					i = j + 2;
					break;
				}
				j += 1;
			}
			if i <= j {
				result.push(chars[i]);
				i += 1;
			}
			continue;
		}

		// Check for __bold__ pattern
		if i + 1 < chars.len() && chars[i] == '_' && chars[i + 1] == '_' {
			let mut j = i + 2;
			while j + 1 < chars.len() {
				if chars[j] == '_' && chars[j + 1] == '_' {
					chars[i + 2..j].iter().for_each(|&c| result.push(c));
					i = j + 2;
					break;
				}
				j += 1;
			}
			if i <= j {
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_strip_bold_from_cell_asterisks() {
		assert_eq!(strip_bold_from_cell("**hello**"), "hello");
		assert_eq!(strip_bold_from_cell("a **b** c"), "a b c");
	}

	#[test]
	fn test_strip_bold_from_cell_underscores() {
		assert_eq!(strip_bold_from_cell("__hello__"), "hello");
		assert_eq!(strip_bold_from_cell("a __b__ c"), "a b c");
	}

	#[test]
	fn test_strip_bold_from_cell_preserves_code() {
		assert_eq!(strip_bold_from_cell("`__x__`"), "`__x__`");
		assert_eq!(strip_bold_from_cell("`a | b`**c**"), "`a | b`c");
	}

	#[test]
	fn test_strip_bold_from_cell_mixed() {
		assert_eq!(strip_bold_from_cell("**hello** __world__"), "hello world");
		assert_eq!(strip_bold_from_cell("`__x__` **y**"), "`__x__` y");
	}

	#[test]
	fn test_strip_bold_from_cell_complex() {
		assert_eq!(
			strip_bold_from_cell("**hello** __world__ `code`"),
			"hello world `code`"
		);
		assert_eq!(strip_bold_from_cell("`__x__` **y** `z`"), "`__x__` y `z`");
	}

	#[test]
	fn test_strip_bold_from_cell_edge_cases() {
		assert_eq!(strip_bold_from_cell(""), "");
		assert_eq!(strip_bold_from_cell("no bold here"), "no bold here");
	}

	#[test]
	fn test_strip_bold_from_cell_complex_table() {
		assert_eq!(
			strip_bold_from_cell("| **hello** | __world__ | `code` |"),
			"| hello | world | `code` |"
		);
		assert_eq!(
			strip_bold_from_cell("| `__x__` | **y** | `z` |"),
			"| `__x__` | y | `z` |"
		);
	}

	#[test]
	fn test_strip_bold_from_cell_multi_backticks() {
		assert_eq!(
			strip_bold_from_cell("`` `let a = 1;` `` **bold**"),
			"`` `let a = 1;` `` bold"
		);
		assert_eq!(strip_bold_from_cell("`let a = 1;`"), "`let a = 1;`");
	}
}
