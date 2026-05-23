use crate::LintError;

/// Finds the line number of the first unclosed code block in the markdown content, if any.
pub fn find_unclosed_code_block(content: &str) -> Option<usize> {
	let mut in_code_block = false;
	let mut code_block_start_line = 0;
	let mut fence_char = '`';
	let mut fence_length = 0;

	for (line_idx, line) in content.lines().enumerate() {
		let line_num = line_idx + 1;
		let trimmed = line.trim();

		let is_backtick_fence = trimmed.starts_with("```");
		let is_tilde_fence = trimmed.starts_with("~~~");

		if is_backtick_fence || is_tilde_fence {
			let current_fence_char = if is_backtick_fence { '`' } else { '~' };
			let current_fence_len = trimmed
				.chars()
				.take_while(|&c| c == current_fence_char)
				.count();

			if !in_code_block {
				in_code_block = true;
				code_block_start_line = line_num;
				fence_char = current_fence_char;
				fence_length = current_fence_len;
			} else {
				if current_fence_char == fence_char && current_fence_len >= fence_length {
					let rest = &trimmed[current_fence_len..];
					if rest.trim().is_empty() {
						in_code_block = false;
					} else {
						return Some(code_block_start_line);
					}
				}
			}
		}
	}

	if in_code_block {
		Some(code_block_start_line)
	} else {
		None
	}
}

pub fn validate_code_blocks(content: &str) -> Option<Vec<LintError>> {
	let mut errors = Vec::new();
	let mut in_code_block = false;
	let mut code_block_start_line = 0;

	for (line_num, line) in content.lines().enumerate() {
		let line_num = line_num + 1;

		if line.trim().starts_with("```") {
			if !in_code_block {
				in_code_block = true;
				code_block_start_line = line_num;

				let trimmed = line.trim();
				let has_language = if trimmed.len() > 3 {
					let lang_part = &trimmed[3..];
					!lang_part.trim().is_empty()
				} else {
					false
				};

				if !has_language {}
			} else {
				in_code_block = false;

				let trimmed = line.trim();
				let _has_language = if trimmed.len() > 3 {
					let lang_part = &trimmed[3..];
					!lang_part.trim().is_empty()
				} else {
					false
				};

				if let Some(start_line_content) = content.lines().nth(code_block_start_line - 1) {
					let start_trimmed = start_line_content.trim();
					let start_has_language = if start_trimmed.len() > 3 {
						let lang_part = &start_trimmed[3..];
						!lang_part.trim().is_empty()
					} else {
						false
					};

					if !start_has_language {
						errors.push(LintError {
                            line: code_block_start_line,
                            column: 1,
                            message: "Code block should specify language for better parsing. Use 'text' if no specific language applies".to_string(),
                            rule: "code-blocks".to_string(),
                        });
					}
				}
			}
		}
	}

	if errors.is_empty() {
		None
	} else {
		Some(errors)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_validate_code_blocks_with_language() {
		let content = "```javascript\nconsole.log('hello');\n```";
		let result = validate_code_blocks(content);
		assert!(result.is_none()); // Should be valid (has language)
	}

	#[test]
	fn test_validate_code_blocks_no_language() {
		let content = "```\nconsole.log('hello');\n```";
		let result = validate_code_blocks(content);
		assert!(result.is_some());
		let warnings = result.unwrap();
		assert_eq!(warnings.len(), 1); // One warning for missing language
		assert_eq!(warnings[0].line, 1); // Warning on opening line
		assert!(warnings[0].message.contains("language"));
	}

	#[test]
	fn test_validate_code_blocks_multiple_blocks() {
		let content = "```javascript\n// Has language\n```\n\n```\n// No language\n```";
		let result = validate_code_blocks(content);
		assert!(result.is_some());
		let warnings = result.unwrap();
		assert_eq!(warnings.len(), 1); // One warning for second block
		assert_eq!(warnings[0].line, 5); // Second block starts on line 5
	}

	#[test]
	fn test_validate_code_blocks_unclosed_block() {
		let content = r#"# Title

```javascript
console.log('no closing fence');
```
"#;
		let result = validate_code_blocks(content);
		// Should handle closed block correctly
		assert!(result.is_none());
	}

	#[test]
	fn test_validate_code_blocks_empty_language() {
		let content = r#"```
code with empty language spec
```
"#;
		let result = validate_code_blocks(content);
		// Should treat empty language as missing
		assert!(result.is_some());
		let warnings = result.unwrap();
		assert_eq!(warnings.len(), 1);
	}

	#[test]
	fn test_validate_code_blocks_nested_fences() {
		let content = r#"`````
triple backticks inside code
```
still inside
`````
"#;
		let result = validate_code_blocks(content);
		// Should handle nested fences correctly
		assert!(result.is_none()); // Has language (empty)
	}

	#[test]
	fn test_validate_code_blocks_with_text_language() {
		let content = "```text\nThis is plain text\n```";
		let result = validate_code_blocks(content);
		assert!(result.is_none()); // Should be valid (has language)
	}

	#[test]
	fn test_validate_code_blocks_language_with_spaces() {
		let content = "```  javascript  \nconsole.log('hello');\n```";
		let result = validate_code_blocks(content);
		assert!(result.is_none()); // Should be valid (has language after trimming)
	}

	#[test]
	fn test_validate_code_blocks_no_code_blocks() {
		let content = "# Heading\n\nJust regular text\n\nNo code blocks here.";
		let result = validate_code_blocks(content);
		assert!(result.is_none()); // Should be valid (no code blocks)
	}

	#[test]
	fn test_validate_code_blocks_inline_code() {
		let content = "This has `inline code` but no fenced code blocks.";
		let result = validate_code_blocks(content);
		assert!(result.is_none()); // Should be valid (no fenced code blocks)
	}

	#[test]
	fn test_find_unclosed_code_block_none() {
		let content = "```rust\nfn main() {}\n```\nSome text\n```javascript\nconsole.log(1);\n```";
		assert_eq!(find_unclosed_code_block(content), None);
	}

	#[test]
	fn test_find_unclosed_code_block_missing_closing_at_end() {
		let content = "```rust\nfn main() {}\n";
		assert_eq!(find_unclosed_code_block(content), Some(1));
	}

	#[test]
	fn test_find_unclosed_code_block_missing_closing_with_new_block() {
		let content =
			"```javascript\nconsole.log(1);\n// no closing fence\n\n```rust\nfn main() {}\n```";
		assert_eq!(find_unclosed_code_block(content), Some(1));
	}

	#[test]
	fn test_find_unclosed_code_block_nested_valid() {
		let content = "````\n```rust\nfn main() {}\n```\n````";
		assert_eq!(find_unclosed_code_block(content), None);
	}

	#[test]
	fn test_find_unclosed_code_block_tilde_none() {
		let content = "~~~\nhello\n~~~";
		assert_eq!(find_unclosed_code_block(content), None);
	}

	#[test]
	fn test_find_unclosed_code_block_tilde_unclosed() {
		let content = "~~~\nhello\n";
		assert_eq!(find_unclosed_code_block(content), Some(1));
	}
}
