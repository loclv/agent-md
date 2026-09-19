pub fn find_bold_text(line: &str) -> Vec<usize> {
	let mut results = Vec::new();

	let mut exempt_ranges = Vec::new();
	let chars: Vec<char> = line.chars().collect();
	let mut i = 0;

	while i < chars.len() {
		// Exempt inline code spans
		if chars[i] == '`' {
			if let Some(code_end) = crate::format::lines::find_code_span_end(&chars, i) {
				exempt_ranges.push((i, code_end));
				i = code_end + 1;
				continue;
			}
		}

		// Exempt HTML tags, comments, and autolinks
		if chars[i] == '<' && i + 1 < chars.len() {
			let next_c = chars[i + 1];
			if next_c.is_ascii_alphabetic() || next_c == '/' || next_c == '!' || next_c == '?' {
				if let Some(tag_end) = crate::format::html::find_tag_end(&chars, i) {
					exempt_ranges.push((i, tag_end));
					i = tag_end + 1;
					continue;
				}
			}
		}

		// Exempt link destinations in [text](url) or ![alt](url)
		if chars[i] == ']' && i + 1 < chars.len() && chars[i + 1] == '(' {
			if let Some(dest_end) = crate::format::lines::find_link_destination_end(&chars, i + 1) {
				exempt_ranges.push((i + 1, dest_end));
				i = dest_end + 1;
				continue;
			}
		}

		// Exempt reference link destinations in [id]: url
		if chars[i] == ']' && i + 1 < chars.len() && chars[i + 1] == ':' {
			exempt_ranges.push((i + 1, chars.len().saturating_sub(1)));
			break;
		}

		i += 1;
	}

	let mut search_start = 0;
	while search_start < line.len() {
		if let Some(start) = line[search_start..].find("**") {
			let abs_start = search_start + start;

			let in_exempt = exempt_ranges
				.iter()
				.any(|&(start, end)| abs_start >= start && abs_start <= end);

			if !in_exempt {
				if let Some(end_offset) = line[abs_start + 2..].find("**") {
					let abs_end = abs_start + 2 + end_offset;

					let end_in_exempt = exempt_ranges
						.iter()
						.any(|&(start, end)| abs_end >= start && abs_end <= end);

					if !end_in_exempt {
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

			let in_exempt = exempt_ranges
				.iter()
				.any(|&(start, end)| abs_start >= start && abs_start <= end);

			if !in_exempt {
				if let Some(end_offset) = line[abs_start + 2..].find("__") {
					let abs_end = abs_start + 2 + end_offset;

					let end_in_exempt = exempt_ranges
						.iter()
						.any(|&(start, end)| abs_end >= start && abs_end <= end);

					if !end_in_exempt {
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

	#[test]
	fn test_find_bold_text_in_link_destination_exempt() {
		let line = "[Package](https://github.com/org/repo/blob/main/__init__.py)";
		let result = find_bold_text(line);
		assert_eq!(result.len(), 0);
	}

	#[test]
	fn test_find_bold_text_in_autolink_exempt() {
		let line = "See <https://github.com/org/repo/blob/main/__init__.py>";
		let result = find_bold_text(line);
		assert_eq!(result.len(), 0);
	}

	#[test]
	fn test_find_bold_text_in_html_tag_exempt() {
		let line = "<a href=\"https://example.com/__init__.py\">Link</a>";
		let result = find_bold_text(line);
		assert_eq!(result.len(), 0);
	}

	#[test]
	fn test_find_bold_text_in_reference_link_exempt() {
		let line = "[1]: https://example.com/__init__.py";
		let result = find_bold_text(line);
		assert_eq!(result.len(), 0);
	}

	#[test]
	fn test_find_bold_text_in_link_label_still_detected() {
		let line = "[**Bold Link**](https://example.com/__init__.py)";
		let result = find_bold_text(line);
		assert_eq!(result.len(), 1);
		assert_eq!(result[0], 2);
	}
}
