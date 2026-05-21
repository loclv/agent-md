use crate::LintError;

pub fn validate_whitespace(content: &str) -> Vec<LintError> {
	let mut errors = Vec::new();
	let mut in_code_block = false;
	let mut blank_line_count = 0;

	for (line_num, line) in content.lines().enumerate() {
		let line_num = line_num + 1;

		if line.trim().starts_with("```") {
			in_code_block = !in_code_block;
			blank_line_count = 0;
			continue;
		}

		if in_code_block {
			continue;
		}

		if line.trim().is_empty() {
			blank_line_count += 1;
			if blank_line_count > 1 {
				errors.push(LintError {
					line: line_num,
					column: 1,
					message: "Multiple consecutive blank lines detected.".to_string(),
					rule: "no-multiple-blanks".to_string(),
				});
			}
		} else {
			blank_line_count = 0;
		}

		// MD010: no-hard-tabs
		if let Some(col) = line.find('\t') {
			errors.push(LintError {
				line: line_num,
				column: col + 1,
				message: "Hard tabs detected. Use spaces for indentation.".to_string(),
				rule: "no-hard-tabs".to_string(),
			});
		}

		// MD009: no-trailing-spaces
		if line.ends_with(' ') || line.ends_with('\t') {
			let trailing_start = line.trim_end().len() + 1;
			errors.push(LintError {
				line: line_num,
				column: trailing_start,
				message: "Trailing spaces detected.".to_string(),
				rule: "no-trailing-spaces".to_string(),
			});
		}
	}

	// MD047: single-trailing-newline
	if !content.is_empty() && !content.ends_with('\n') {
		errors.push(LintError {
			line: content.lines().count(),
			column: content.lines().last().unwrap_or("").len() + 1,
			message: "Files should end with a single newline character.".to_string(),
			rule: "single-trailing-newline".to_string(),
		});
	}

	errors
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_validate_whitespace_tabs() {
		let content = "Text\twith tab\n";
		let errors = validate_whitespace(content);
		assert_eq!(errors.len(), 1);
		assert_eq!(errors[0].rule, "no-hard-tabs");
	}

	#[test]
	fn test_validate_whitespace_trailing_spaces() {
		let content = "Text with trailing spaces  \n";
		let errors = validate_whitespace(content);
		assert_eq!(errors.len(), 1);
		assert_eq!(errors[0].rule, "no-trailing-spaces");
		assert_eq!(errors[0].column, 26);
	}

	#[test]
	fn test_validate_whitespace_clean() {
		let content = "Clean text\nMore clean text\n";
		let errors = validate_whitespace(content);
		assert!(errors.is_empty());
	}

	#[test]
	fn test_validate_whitespace_ignores_code_blocks() {
		let content = "```\n\tCode with tabs\n```\n";
		let errors = validate_whitespace(content);
		assert!(errors.is_empty());
	}

	#[test]
	fn test_validate_whitespace_trailing_newline() {
		let content = "Text without newline at end";
		let errors = validate_whitespace(content);
		assert_eq!(errors.len(), 1);
		assert_eq!(errors[0].rule, "single-trailing-newline");
	}

	#[test]
	fn test_validate_whitespace_empty_file() {
		let content = "";
		let errors = validate_whitespace(content);
		assert!(errors.is_empty()); // Empty file is valid
	}

	#[test]
	fn test_validate_whitespace_multiple_trailing_newlines() {
		let content = "Text\n\n\n";
		let errors = validate_whitespace(content);
		// Now it should have no-multiple-blanks warning for the extra blank line
		assert!(errors.iter().any(|e| e.rule == "no-multiple-blanks"));
	}

	#[test]
	fn test_validate_whitespace_multiple_errors_per_line() {
		let content = "\tText with tab and trailing space  \n";
		let errors = validate_whitespace(content);
		assert_eq!(errors.len(), 2);
		assert!(errors.iter().any(|e| e.rule == "no-hard-tabs"));
		assert!(errors.iter().any(|e| e.rule == "no-trailing-spaces"));
	}

	#[test]
	fn test_validate_whitespace_multiple_blanks() {
		let content = "Line 1\n\n\nLine 4\n";
		let errors = validate_whitespace(content);
		assert!(errors.iter().any(|e| e.rule == "no-multiple-blanks"));
		assert_eq!(
			errors
				.iter()
				.filter(|e| e.rule == "no-multiple-blanks")
				.count(),
			1
		);
		assert_eq!(
			errors
				.iter()
				.find(|e| e.rule == "no-multiple-blanks")
				.unwrap()
				.line,
			3
		);
	}

	#[test]
	fn test_validate_whitespace_single_blanks_ok() {
		let content = "Line 1\n\nLine 3\n\nLine 5\n";
		let errors = validate_whitespace(content);
		assert!(!errors.iter().any(|e| e.rule == "no-multiple-blanks"));
	}

	#[test]
	fn test_validate_whitespace_multiple_blanks_in_code_block_ok() {
		let content = "```\nLine 1\n\n\nLine 4\n```\n";
		let errors = validate_whitespace(content);
		assert!(!errors.iter().any(|e| e.rule == "no-multiple-blanks"));
	}
}
