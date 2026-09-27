//! JSONC comment stripping for configuration files.
//!
//! `agent-md` accepts `.jsonc` configuration files and comment-annotated
//! `.markdownlintrc` files, but `serde_json` only parses strict JSON.
//! This module bridges the gap by removing `//` and `/* */` comments
//! before parsing, so the config layer never deals with comments directly.

/// Strip `//` line comments and `/* */` block comments outside of strings.
///
/// Scans with SIMD-accelerated `memchr` jumps between significant bytes and
/// copies the text in between in bulk. This is sound because `"`, `/`, `\`,
/// `*`, and `\n` are ASCII and can never occur inside multi-byte UTF-8
/// sequences, so byte offsets always land on character boundaries.
pub(crate) fn strip_json_comments(content: &str) -> String {
	let bytes = content.as_bytes();
	let mut out = String::with_capacity(content.len());
	let mut i = 0;
	let mut in_string = false;
	while i < bytes.len() {
		if in_string {
			// Inside strings only `"` and `\` are significant; `/` is ordinary.
			match memchr::memchr2(b'"', b'\\', &bytes[i..]) {
				None => {
					out.push_str(&content[i..]);
					break;
				}
				Some(rel) => {
					let pos = i + rel;
					out.push_str(&content[i..=pos]);
					if bytes[pos] == b'\\' {
						// Copy the escaped character whole (it may be multi-byte).
						if let Some(ch) = content[pos + 1..].chars().next() {
							out.push(ch);
							i = pos + 1 + ch.len_utf8();
						} else {
							i = pos + 1;
						}
					} else {
						in_string = false;
						i = pos + 1;
					}
				}
			}
			continue;
		}
		match memchr::memchr2(b'"', b'/', &bytes[i..]) {
			None => {
				out.push_str(&content[i..]);
				break;
			}
			Some(rel) => {
				let pos = i + rel;
				out.push_str(&content[i..pos]);
				if bytes[pos] == b'"' {
					out.push('"');
					in_string = true;
					i = pos + 1;
				} else if bytes.get(pos + 1) == Some(&b'/') {
					// Line comment: skip to (and keep) the newline.
					match memchr::memchr(b'\n', &bytes[pos + 2..]) {
						Some(nl) => {
							out.push('\n');
							i = pos + 2 + nl + 1;
						}
						None => break,
					}
				} else if bytes.get(pos + 1) == Some(&b'*') {
					// Block comment: skip to `*/`, preserving newlines so the
					// stripped output keeps a 1:1 line mapping with the original.
					let mut j = pos + 2;
					let mut prev_star = false;
					while j < bytes.len() {
						let b = bytes[j];
						if prev_star && b == b'/' {
							j += 1;
							break;
						}
						prev_star = b == b'*';
						if b == b'\n' {
							out.push('\n');
						}
						j += 1;
					}
					i = j;
				} else {
					out.push('/');
					i = pos + 1;
				}
			}
		}
	}
	out
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_strip_json_comments_preserves_urls_in_strings() {
		let content = "{\"url\": \"https://example.com//path\", \"other\": 1} // done";
		let stripped = strip_json_comments(content);
		assert!(stripped.contains("https://example.com//path"));
		assert!(!stripped.contains("// done"));
		let val: serde_json::Value = serde_json::from_str(&stripped).unwrap();
		assert_eq!(val.get("other").unwrap(), 1);
	}

	#[test]
	fn test_strip_json_comments_escaped_quotes_and_backslashes() {
		// `\"` must not end the string; `\\` must not escape the quote.
		let content = "{\"a\": \"x\\\"//kept\", \"b\": \"y\\\\\", \"c\": 1} /* gone */";
		let stripped = strip_json_comments(content);
		assert!(stripped.contains("//kept"));
		assert!(!stripped.contains("gone"));
		let val: serde_json::Value = serde_json::from_str(&stripped).unwrap();
		assert_eq!(val.get("c").unwrap(), 1);
	}

	#[test]
	fn test_strip_json_comments_block_preserves_line_numbers() {
		let content = "{\n/* one\ntwo */\n\"a\": 1\n}";
		let stripped = strip_json_comments(content);
		assert_eq!(stripped.lines().count(), content.lines().count());
		let val: serde_json::Value = serde_json::from_str(&stripped).unwrap();
		assert_eq!(val.get("a").unwrap(), 1);
	}

	#[test]
	fn test_strip_json_comments_unterminated_comment() {
		assert_eq!(strip_json_comments("{\"a\": 1} // trailing"), "{\"a\": 1} ");
		assert_eq!(
			strip_json_comments("{\"a\": 1} /* never ends"),
			"{\"a\": 1} "
		);
		assert_eq!(strip_json_comments("/"), "/");
	}

	#[test]
	fn test_strip_json_comments_multibyte_content() {
		let content = "{\"greeting\": \"Xin chào // thế giới\", \"a\": 1} // chú thích";
		let stripped = strip_json_comments(content);
		assert!(stripped.contains("Xin chào // thế giới"));
		let val: serde_json::Value = serde_json::from_str(&stripped).unwrap();
		assert_eq!(val.get("a").unwrap(), 1);
	}
}
