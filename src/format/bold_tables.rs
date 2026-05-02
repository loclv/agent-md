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
			let mut code_end = i + 1;
			while code_end < chars.len() && chars[code_end] != '`' {
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
}
