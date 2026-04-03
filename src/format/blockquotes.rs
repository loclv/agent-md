/// Normalize blockquote lines by removing extra spaces after > markers.
/// Example: ">  text" becomes ">text", ">>  text" becomes ">>text"
/// Leading whitespace before > markers is preserved.
pub fn normalize_blockquote(line: &str) -> String {
	let chars: Vec<char> = line.chars().collect();
	let mut i = 0;

	// Skip leading whitespace (preserve indentation)
	let leading_len = i;
	while i < chars.len() && (chars[i] == ' ' || chars[i] == '\t') {
		i += 1;
	}
	let leading = &line[leading_len..i];

	// Count consecutive > markers
	let mut markers = String::new();
	while i < chars.len() && chars[i] == '>' {
		markers.push('>');
		i += 1;
	}

	// If no > markers, return original line
	if markers.is_empty() {
		return line.to_string();
	}

	// Skip all spaces after > markers
	while i < chars.len() && chars[i] == ' ' {
		i += 1;
	}

	// Build result: leading whitespace + markers + rest of line
	let mut result = String::from(leading);
	result.push_str(&markers);
	for &c in chars.iter().skip(i) {
		result.push(c);
	}

	result
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_normalize_blockquote_single_level() {
		assert_eq!(normalize_blockquote(">  text"), ">text");
		assert_eq!(normalize_blockquote(">text"), ">text");
		assert_eq!(
			normalize_blockquote(">   multiple spaces"),
			">multiple spaces"
		);
	}

	#[test]
	fn test_normalize_blockquote_nested() {
		assert_eq!(normalize_blockquote(">>  text"), ">>text");
		assert_eq!(normalize_blockquote(">>>  6"), ">>>6");
		assert_eq!(normalize_blockquote(">> 3"), ">>3");
	}

	#[test]
	fn test_normalize_blockquote_no_marker() {
		assert_eq!(normalize_blockquote("regular text"), "regular text");
		assert_eq!(normalize_blockquote(""), "");
	}

	#[test]
	fn test_normalize_blockquote_empty_content() {
		assert_eq!(normalize_blockquote(">"), ">");
		assert_eq!(normalize_blockquote(">>"), ">>");
		assert_eq!(normalize_blockquote(">   "), ">");
	}

	#[test]
	fn test_normalize_blockquote_with_leading_spaces() {
		// Leading spaces before > should be preserved (indentation)
		assert_eq!(normalize_blockquote("  > text"), "  >text");
	}
}
