use crate::LintError;

pub fn validate_heading_structure(content: &str) -> Option<Vec<LintError>> {
	let mut heading_levels = Vec::new();
	let mut h1_count = 0;
	let mut h1_locations = Vec::new();
	let mut in_code_block = false;
	let mut prev_line: Option<&str> = None;

	let mut errors = Vec::new();

	for (line_num, line) in content.lines().enumerate() {
		let line_num = line_num + 1;

		if line.trim().starts_with("```") {
			in_code_block = !in_code_block;
			prev_line = Some(line);
			continue;
		}

		if in_code_block {
			prev_line = Some(line);
			continue;
		}

		// Check for Setext-style headings (underlined headings)
		if let Some(level) = detect_setext_heading(line, prev_line) {
			let heading_line = line_num - 1; // The heading text is on the previous line
			errors.push(LintError {
				line: heading_line,
				column: 1,
				message:
					"Setext-style heading (underlined) detected. Use ATX-style headings with # instead."
						.to_string(),
				rule: "heading-structure".to_string(),
			});
			heading_levels.push((level, heading_line));

			if level == 1 {
				h1_count += 1;
				h1_locations.push(heading_line);
			}
		}

		if let Some(level) = extract_heading_level(line) {
			heading_levels.push((level, line_num));

			if level == 1 {
				h1_count += 1;
				h1_locations.push(line_num);
			}
		}

		prev_line = Some(line);
	}

	if h1_count > 1 {
		for &location in &h1_locations[1..] {
			errors.push(LintError {
				line: location,
				column: 1,
				message:
					"Multiple H1 headings found. Documents should have only one top-level heading"
						.to_string(),
				rule: "heading-structure".to_string(),
			});
		}
	}

	if heading_levels.len() > 1 {
		for i in 1..heading_levels.len() {
			let (current_level, current_line) = heading_levels[i];
			let (prev_level, _) = heading_levels[i - 1];

			if current_level > prev_level + 1 {
				errors.push(LintError {
					line: current_line,
					column: 1,
					message: format!(
						"Heading level skipped: H{} follows H{}. Use sequential heading levels.",
						current_level, prev_level
					),
					rule: "heading-structure".to_string(),
				});
			}
		}
	}

	if errors.is_empty() {
		None
	} else {
		Some(errors)
	}
}

/// Detects Setext-style headings (underlined headings)
/// Returns Some(level) if detected, where level is 1 for H1 (=) or 2 for H2 (-)
pub fn detect_setext_heading(current_line: &str, prev_line: Option<&str>) -> Option<u32> {
	let trimmed = current_line.trim();

	// Check if current line is all = or all -
	if trimmed.is_empty() {
		return None;
	}

	let is_h1_underline = trimmed.chars().all(|c| c == '=');
	let is_h2_underline = trimmed.chars().all(|c| c == '-');

	if !is_h1_underline && !is_h2_underline {
		return None;
	}

	// Check if previous line exists and is not empty (it's the heading text)
	if let Some(prev) = prev_line {
		let prev_trimmed = prev.trim();
		// Previous line should not be a code block fence or another ATX heading
		if !prev_trimmed.is_empty()
			&& !prev_trimmed.starts_with('#')
			&& !prev_trimmed.starts_with("```")
		{
			return Some(if is_h1_underline { 1 } else { 2 });
		}
	}

	None
}

pub fn extract_heading_level(line: &str) -> Option<u32> {
	let trimmed = line.trim_start();
	if trimmed.starts_with('#') {
		let mut level = 0;
		for ch in trimmed.chars() {
			if ch == '#' {
				level += 1;
			} else {
				break;
			}
		}

		if matches!(trimmed.chars().nth(level as usize), Some(' ') | Some('\t')) {
			Some(level)
		} else {
			None
		}
	} else {
		None
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_extract_heading_level_valid() {
		assert_eq!(extract_heading_level("# Heading"), Some(1));
		assert_eq!(extract_heading_level("## Heading"), Some(2));
		assert_eq!(extract_heading_level("### Heading"), Some(3));
		assert_eq!(extract_heading_level("###### Heading"), Some(6));
	}

	#[test]
	fn test_extract_heading_level_invalid() {
		assert_eq!(extract_heading_level("Heading"), None);
		assert_eq!(extract_heading_level("#No space"), None);
		assert_eq!(extract_heading_level("#\tHeading"), Some(1)); // Tab counts as space
		assert_eq!(extract_heading_level("####### Heading"), Some(7)); // 7 levels
	}

	#[test]
	fn test_extract_heading_level_edge_cases() {
		assert_eq!(extract_heading_level(""), None);
		assert_eq!(extract_heading_level("#"), None);
		assert_eq!(extract_heading_level(" # Heading"), Some(1)); // Leading space
		assert_eq!(extract_heading_level("   # Heading"), Some(1)); // Multiple leading spaces
	}

	#[test]
	fn test_extract_heading_level_with_special_characters() {
		assert_eq!(extract_heading_level("# Heading with # hash"), Some(1));
		assert_eq!(extract_heading_level("## Heading with ## hashes"), Some(2));
		assert_eq!(extract_heading_level("### Heading ###"), Some(3));
	}

	#[test]
	fn test_extract_heading_level_unicode() {
		assert_eq!(extract_heading_level("# Тест"), Some(1));
		assert_eq!(extract_heading_level("## テスト"), Some(2));
		assert_eq!(extract_heading_level("### 测试"), Some(3));
	}

	#[test]
	fn test_validate_heading_structure_single_heading() {
		let content = "# Single Heading\n\nSome content.";
		let result = validate_heading_structure(content);
		assert!(result.is_none()); // Should be valid
	}

	#[test]
	fn test_validate_heading_structure_multiple_headings() {
		let content = "# First Heading\n\n## Second Heading\n\n### Third Heading";
		let result = validate_heading_structure(content);
		assert!(result.is_none()); // Should be valid (proper sequence)
	}

	#[test]
	fn test_validate_heading_structure_multiple_h1() {
		let content = "# First H1\n\nSome content.\n\n# Second H1\n\nMore content.";
		let result = validate_heading_structure(content);
		assert!(result.is_some());
		let errors = result.unwrap();
		assert_eq!(errors.len(), 1); // One error for second H1
		assert_eq!(errors[0].line, 5); // Second H1 is on line 5
		assert!(errors[0].message.contains("Multiple H1"));
	}

	#[test]
	fn test_validate_heading_structure_skipped_levels() {
		let content = "# H1\n\n### H3 (skipped H2)";
		let result = validate_heading_structure(content);
		assert!(result.is_some());
		let errors = result.unwrap();
		assert_eq!(errors.len(), 1); // One error for skipped level
		assert_eq!(errors[0].line, 3); // H3 is on line 3
		assert!(errors[0].message.contains("skipped"));
	}

	#[test]
	fn test_validate_heading_structure_three_headings() {
		let content = "# First\n\n## Second\n\n### Third";
		let result = validate_heading_structure(content);
		assert!(result.is_none()); // Should be valid
	}

	#[test]
	fn test_validate_heading_structure_mixed_headings() {
		let content = "# H1\n\n## H2\n\n# Another H1\n\n## H2";
		let result = validate_heading_structure(content);
		assert!(result.is_some());
		let errors = result.unwrap();
		assert_eq!(errors.len(), 1); // One error for second H1
		assert_eq!(errors[0].line, 5); // Second H1 is on line 5
	}

	#[test]
	fn test_validate_heading_structure_valid_sequence() {
		let content = "# H1\n\n## H2\n\n### H3\n\n#### H4\n\n##### H5\n\n###### H6";
		let result = validate_heading_structure(content);
		assert!(result.is_none()); // Should be valid
	}

	#[test]
	fn test_validate_heading_structure_other_heading_levels() {
		let content = "## H2\n\n### H3\n\n#### H4";
		let result = validate_heading_structure(content);
		assert!(result.is_none()); // Should be valid (no H1 conflicts)
	}

	#[test]
	fn test_validate_heading_structure_no_headings() {
		let content = "Just some text\n\nwith no headings.";
		let result = validate_heading_structure(content);
		assert!(result.is_none()); // Should be valid
	}

	#[test]
	fn test_validate_heading_structure_false_positives() {
		let content = "This is not a heading\n\n# This is a heading\n\nNot a heading: #";
		let result = validate_heading_structure(content);
		assert!(result.is_none()); // Should be valid (only one real heading)
	}

	#[test]
	fn test_validate_heading_structure_ignores_code_blocks() {
		let content = r#"# Main Heading

```javascript
// This is not a heading in a code block
# Not a heading
```

## Real Heading
"#;
		let result = validate_heading_structure(content);
		assert!(result.is_none()); // Should be valid (ignores code block)
	}

	#[test]
	fn test_validate_heading_structure_multiple_skipped_levels() {
		let content = "# H1\n\n#### H4 (skipped H2 and H3)";
		let result = validate_heading_structure(content);
		assert!(result.is_some());
		let errors = result.unwrap();
		assert_eq!(errors.len(), 1); // One error for skipped levels
		assert_eq!(errors[0].line, 3); // H4 is on line 3
		assert!(errors[0].message.contains("skipped"));
	}

	// Setext-style heading tests
	#[test]
	fn test_detect_setext_heading_h1() {
		assert_eq!(detect_setext_heading("=======", Some("Heading")), Some(1));
		assert_eq!(detect_setext_heading("===", Some("Heading")), Some(1));
		assert_eq!(
			detect_setext_heading("===============", Some("Heading")),
			Some(1)
		);
	}

	#[test]
	fn test_detect_setext_heading_h2() {
		assert_eq!(detect_setext_heading("-------", Some("Heading")), Some(2));
		assert_eq!(detect_setext_heading("---", Some("Heading")), Some(2));
		assert_eq!(
			detect_setext_heading("---------------", Some("Heading")),
			Some(2)
		);
	}

	#[test]
	fn test_detect_setext_heading_invalid() {
		// No previous line
		assert_eq!(detect_setext_heading("=======", None), None);
		// Empty previous line
		assert_eq!(detect_setext_heading("=======", Some("")), None);
		// Previous line is ATX heading
		assert_eq!(detect_setext_heading("=======", Some("# Heading")), None);
		// Mixed characters
		assert_eq!(detect_setext_heading("===---", Some("Heading")), None);
		assert_eq!(detect_setext_heading("abc", Some("Heading")), None);
	}

	#[test]
	fn test_validate_heading_structure_setext_h1() {
		let content = "Heading level 1\n===============";
		let result = validate_heading_structure(content);
		assert!(result.is_some());
		let errors = result.unwrap();
		assert_eq!(errors.len(), 1);
		assert_eq!(errors[0].line, 1); // Heading text is on line 1
		assert!(errors[0].message.contains("Setext-style"));
	}

	#[test]
	fn test_validate_heading_structure_setext_h2() {
		let content = "Heading level 2\n---------------";
		let result = validate_heading_structure(content);
		assert!(result.is_some());
		let errors = result.unwrap();
		assert_eq!(errors.len(), 1);
		assert_eq!(errors[0].line, 1); // Heading text is on line 1
		assert!(errors[0].message.contains("Setext-style"));
	}

	#[test]
	fn test_validate_heading_structure_setext_in_code_block() {
		let content = r#"# Main Heading

```
Heading level 1
===============
```

## Real Heading
"#;
		let result = validate_heading_structure(content);
		assert!(result.is_none()); // Should be valid (ignores code block)
	}

	#[test]
	fn test_validate_heading_structure_setext_with_atx() {
		let content = r#"# ATX Heading

Setext Heading
--------------

## Another ATX
"#;
		let result = validate_heading_structure(content);
		assert!(result.is_some());
		let errors = result.unwrap();
		// Should have error for Setext heading
		assert!(errors.iter().any(|e| e.message.contains("Setext-style")));
	}
}
