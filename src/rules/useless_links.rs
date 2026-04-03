pub fn find_useless_link(line: &str) -> Vec<usize> {
	let mut results = Vec::new();
	let chars: Vec<char> = line.chars().collect();
	let mut i = 0;

	while i < chars.len() {
		if chars[i] == '[' {
			let bracket_start = i;
			let mut bracket_end = i + 1;
			let mut bracket_content = String::new();
			let mut found_closing_bracket = false;

			while bracket_end < chars.len() {
				let ch = chars[bracket_end];
				if ch == ']' {
					found_closing_bracket = true;
					break;
				}
				bracket_content.push(ch);
				bracket_end += 1;
			}

			if found_closing_bracket
				&& bracket_end + 1 < chars.len()
				&& chars[bracket_end + 1] == '('
			{
				let mut paren_start = bracket_end + 2;
				let mut url = String::new();
				let mut found_closing_paren = false;
				let mut paren_depth = 1;

				while paren_start < chars.len() {
					let ch = chars[paren_start];
					if ch == '(' {
						paren_depth += 1;
						url.push(ch);
					} else if ch == ')' {
						paren_depth -= 1;
						if paren_depth == 0 {
							found_closing_paren = true;
							break;
						}
						url.push(ch);
					} else {
						url.push(ch);
					}
					paren_start += 1;
				}

				if found_closing_paren {
					let link_text = bracket_content.trim();
					let url_trimmed = url.trim();

					let url_without_protocol = url_trimmed
						.trim_start_matches("http://")
						.trim_start_matches("https://");

					let url_without_slash = url_without_protocol.trim_end_matches('/');

					let url_without_www = url_without_slash.trim_start_matches("www.");

					if link_text == url_trimmed
						|| link_text == url_without_protocol
						|| link_text == url_without_slash
						|| link_text == url_without_www
					{
						results.push(bracket_start + 1);
					}
					i = paren_start + 1;
					continue;
				}
			}
		}
		i += 1;
	}

	results
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_find_useless_link_exact_url() {
		let line = "Visit [https://example.com](https://example.com) for more";
		let result = find_useless_link(line);
		assert_eq!(result.len(), 1);
		assert_eq!(result[0], 7); // Position of [
	}

	#[test]
	fn test_find_useless_link_valid_link() {
		let line = "Visit [Example](https://example.com) for more";
		let result = find_useless_link(line);
		assert_eq!(result.len(), 0);
	}

	#[test]
	fn test_find_useless_link_no_links() {
		let line = "This has no links at all";
		let result = find_useless_link(line);
		assert_eq!(result.len(), 0);
	}

	#[test]
	fn test_find_useless_link_with_www() {
		let line = "Visit [www.example.com](https://www.example.com) for more";
		let result = find_useless_link(line);
		assert_eq!(result.len(), 1); // Should detect www.example.com == www.example.com
		assert_eq!(result[0], 7);
	}

	#[test]
	fn test_find_useless_link_without_protocol() {
		let line = "Visit [example.com](https://example.com) for more";
		let result = find_useless_link(line);
		assert_eq!(result.len(), 1); // Should detect example.com == example.com
		assert_eq!(result[0], 7);
	}

	#[test]
	fn test_find_useless_link_malformed_link() {
		let line = "This has [broken(link](https://example.com)";
		let result = find_useless_link(line);
		assert_eq!(result.len(), 0); // Malformed link, no detection
	}

	#[test]
	fn test_find_useless_link_with_trailing_slash() {
		let line = "Visit [example.com](https://example.com/) for more";
		let result = find_useless_link(line);
		assert_eq!(result.len(), 1); // Should detect example.com == example.com/
		assert_eq!(result[0], 7);
	}

	#[test]
	fn test_find_useless_link_with_path() {
		let line = "Visit [example.com/path](https://example.com/path) for more";
		let result = find_useless_link(line);
		assert_eq!(result.len(), 1); // Should detect exact match with path
		assert_eq!(result[0], 7);
	}

	#[test]
	fn test_find_useless_link_http_protocol() {
		let line = "Visit [http://example.com](http://example.com) for more";
		let result = find_useless_link(line);
		assert_eq!(result.len(), 1); // Should detect with http protocol
		assert_eq!(result[0], 7);
	}

	#[test]
	fn test_find_useless_link_mixed_protocol() {
		let line = "Visit [example.com](http://example.com) for more";
		let result = find_useless_link(line);
		assert_eq!(result.len(), 1); // Should detect example.com == example.com
		assert_eq!(result[0], 7);
	}

	#[test]
	fn test_find_useless_link_multiple_links() {
		let line =
			"[https://example.com](https://example.com) and [https://test.com](https://test.com)";
		let result = find_useless_link(line);
		assert_eq!(result.len(), 2); // Should detect both useless links
		assert_eq!(result[0], 1); // First [
		assert_eq!(result[1], 48); // Second [
	}

	#[test]
	fn test_find_useless_link_empty_text() {
		let line = "Visit [](https://example.com) for more";
		let result = find_useless_link(line);
		assert_eq!(result.len(), 0); // Empty text is not considered useless
	}

	#[test]
	fn test_find_useless_link_nested_parentheses() {
		let line = "Visit [example](https://example.com/path(to/file)) for more";
		let result = find_useless_link(line);
		assert_eq!(result.len(), 0); // Should handle nested parentheses correctly
	}

	#[test]
	fn test_find_useless_link_with_query_params() {
		let line = "[example.com?query=1](https://example.com?query=1)";
		let result = find_useless_link(line);
		assert_eq!(result.len(), 1); // Should detect with query params
		assert_eq!(result[0], 1);
	}

	#[test]
	fn test_find_useless_link_partial_match() {
		let line = "Visit [example](https://example.com) for more";
		let result = find_useless_link(line);
		assert_eq!(result.len(), 0); // Partial match should not trigger
	}

	#[test]
	fn test_find_useless_link_whitespace_variations() {
		let line = "Visit [ example.com ]( https://example.com ) for more";
		let result = find_useless_link(line);
		assert_eq!(result.len(), 1); // Should trim whitespace
		assert_eq!(result[0], 7);
	}
}
