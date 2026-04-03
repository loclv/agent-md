pub fn find_bold_text(line: &str) -> Vec<usize> {
	let mut results = Vec::new();

	let mut code_ranges = Vec::new();
	let chars: Vec<char> = line.chars().collect();
	let mut in_code = false;
	let mut code_start = 0;

	for (i, &ch) in chars.iter().enumerate() {
		if ch == '`' && (i == 0 || chars[i - 1] != '\\') {
			if !in_code {
				in_code = true;
				code_start = i;
			} else {
				in_code = false;
				code_ranges.push((code_start, i));
			}
		}
	}

	if in_code {
		code_ranges.push((code_start, line.len() - 1));
	}

	let mut search_start = 0;
	while search_start < line.len() {
		if let Some(start) = line[search_start..].find("**") {
			let abs_start = search_start + start;

			let in_code_range = code_ranges
				.iter()
				.any(|&(start, end)| abs_start >= start && abs_start <= end);

			if !in_code_range {
				if let Some(end_offset) = line[abs_start + 2..].find("**") {
					let abs_end = abs_start + 2 + end_offset;

					let end_in_code_range = code_ranges
						.iter()
						.any(|&(start, end)| abs_end >= start && abs_end <= end);

					if !end_in_code_range {
						results.push(abs_start + 1);
						search_start = abs_end + 2;
						continue;
					}
				}
			}
			search_start = abs_start + 2;
		} else {
			break;
		}
	}

	search_start = 0;
	while search_start < line.len() {
		if let Some(start) = line[search_start..].find("__") {
			let abs_start = search_start + start;

			let in_code_range = code_ranges
				.iter()
				.any(|&(start, end)| abs_start >= start && abs_start <= end);

			if !in_code_range {
				if let Some(end_offset) = line[abs_start + 2..].find("__") {
					let abs_end = abs_start + 2 + end_offset;

					let end_in_code_range = code_ranges
						.iter()
						.any(|&(start, end)| abs_end >= start && abs_end <= end);

					if !end_in_code_range {
						results.push(abs_start + 1);
						search_start = abs_end + 2;
						continue;
					}
				}
			}
			search_start = abs_start + 2;
		} else {
			break;
		}
	}

	results
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_find_bold_text_double_asterisks() {
		let line = "This has **bold** text";
		let result = find_bold_text(line);
		assert_eq!(result.len(), 1);
		assert_eq!(result[0], 10); // Position of first *
	}

	#[test]
	fn test_find_bold_text_double_underscores() {
		let line = "This has __bold__ text";
		let result = find_bold_text(line);
		assert_eq!(result.len(), 1);
		assert_eq!(result[0], 10); // Position of first _
	}

	#[test]
	fn test_find_bold_text_no_bold() {
		let line = "This has no bold text";
		let result = find_bold_text(line);
		assert_eq!(result.len(), 0);
	}

	#[test]
	fn test_find_bold_text_partial_patterns() {
		let line = "This has **bold text";
		let result = find_bold_text(line);
		assert_eq!(result.len(), 0); // Incomplete pattern

		let line = "This has bold** text";
		let result = find_bold_text(line);
		assert_eq!(result.len(), 0); // Incomplete pattern
	}

	#[test]
	fn test_find_bold_text_multiple_instances() {
		let line = "**First** and **second** bold";
		let result = find_bold_text(line);
		assert_eq!(result.len(), 2);
		assert_eq!(result[0], 1); // First **
		assert_eq!(result[1], 15); // Second **
	}

	#[test]
	fn test_find_bold_text_nested_bold_italics() {
		let line = "This has ***bold italic*** text";
		let result = find_bold_text(line);
		assert_eq!(result.len(), 1); // Should detect the outer ** in ***
		assert_eq!(result[0], 10);
	}

	#[test]
	fn test_find_bold_text_mixed_bold_formats() {
		let line = "**Bold** and __bold__ text";
		let result = find_bold_text(line);
		assert_eq!(result.len(), 2);
		assert_eq!(result[0], 1); // First **
		assert_eq!(result[1], 14); // First _
	}

	#[test]
	fn test_find_bold_text_with_escaped_characters() {
		let line = "This has \\**not bold** text";
		let result = find_bold_text(line);
		assert_eq!(result.len(), 1); // Should still find bold after escaped backslash
		assert_eq!(result[0], 11); // Position of first **
	}

	#[test]
	fn test_find_bold_text_with_nested_code() {
		let line = "Text with **bold and `code`** inside";
		let result = find_bold_text(line);
		assert_eq!(result.len(), 1);
		assert_eq!(result[0], 11); // Position of first **
	}

	#[test]
	fn test_find_bold_text_multiple_overlapping() {
		let line = "**bold1****bold2**";
		let result = find_bold_text(line);
		assert_eq!(result.len(), 2);
		assert_eq!(result[0], 1); // First **
		assert_eq!(result[1], 10); // Second **
	}

	#[test]
	fn test_find_bold_text_in_inline_code() {
		let line = "This has `**bold**` in inline code";
		let result = find_bold_text(line);
		assert_eq!(result.len(), 0); // Should not find bold in inline code
	}

	#[test]
	fn test_find_bold_text_with_multiple_inline_codes() {
		let line = "`code1` **bold** `code2` __bold__ `code3`";
		let result = find_bold_text(line);
		assert_eq!(result.len(), 2);
		assert_eq!(result[0], 9); // First **
		assert_eq!(result[1], 26); // First _
	}

	#[test]
	fn test_find_bold_text_with_unclosed_inline_code() {
		let line = "This has **bold** and `unclosed code";
		let result = find_bold_text(line);
		assert_eq!(result.len(), 1);
		assert_eq!(result[0], 10); // Position of first **
	}

	#[test]
	fn test_find_bold_text_empty_line() {
		let line = "";
		let result = find_bold_text(line);
		assert_eq!(result.len(), 0);
	}

	#[test]
	fn test_find_bold_text_only_markers() {
		let line = "****";
		let result = find_bold_text(line);
		assert_eq!(result.len(), 1);
		assert_eq!(result[0], 1); // Position of first **
	}

	#[test]
	fn test_find_bold_text_escaped_backtick() {
		let line = "Text with \\`escaped\\` and **bold**";
		let result = find_bold_text(line);
		assert_eq!(result.len(), 1);
		assert_eq!(result[0], 27); // Position of first **
	}
}
