use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct HeadingIssue {
	pub line: usize,
	pub column: usize,
	pub message: String,
	pub rule: String,
	pub is_error: bool,
}

pub fn validate_heading_structure(
	content: &str,
	blanks_around_headings: bool,
) -> Option<Vec<HeadingIssue>> {
	let mut heading_levels = Vec::new();
	let mut h1_count = 0;
	let mut h1_locations = Vec::new();
	let mut in_code_block = false;
	let mut prev_line: Option<&str> = None;
	let mut first_content_line = true;

	let mut issues = Vec::new();
	let mut seen_headings = std::collections::HashSet::new();

	for (line_num, line) in content.lines().enumerate() {
		let line_num = line_num + 1;

		if line.trim().starts_with("```") {
			in_code_block = !in_code_block;
			prev_line = Some(line);
			if first_content_line && !line.trim().is_empty() {
				first_content_line = false;
			}
			continue;
		}

		if in_code_block {
			prev_line = Some(line);
			if first_content_line && !line.trim().is_empty() {
				first_content_line = false;
			}
			continue;
		}

		if first_content_line && !line.trim().is_empty() {
			if let Some(level) = extract_heading_level(line) {
				if level != 1 {
					issues.push(HeadingIssue {
						line: line_num,
						column: 1,
						message: "First line in a file should be a top-level heading (H1)"
							.to_string(),
						rule: "first-line-h1".to_string(),
						is_error: false,
					});
				}
			} else {
				issues.push(HeadingIssue {
					line: line_num,
					column: 1,
					message: "First line in a file should be a top-level heading (H1)".to_string(),
					rule: "first-line-h1".to_string(),
					is_error: false,
				});
			}
			first_content_line = false;
		}

		// Check for Setext-style headings (underlined headings)
		if let Some(level) = detect_setext_heading(line, prev_line) {
			let heading_line = line_num - 1; // The heading text is on the previous line
			let heading_text = prev_line.unwrap_or("").trim();
			issues.push(HeadingIssue {
				line: heading_line,
				column: 1,
				message:
					"Setext-style heading (underlined) detected. Use ATX-style headings with # instead."
						.to_string(),
				rule: "heading-structure".to_string(),
				is_error: true,
			});
			heading_levels.push((level, heading_line));

			if level == 1 {
				h1_count += 1;
				h1_locations.push(heading_line);
			}

			// Check for duplicates in Setext headings
			if !heading_text.is_empty() {
				if seen_headings.contains(heading_text) {
					issues.push(HeadingIssue {
						line: heading_line,
						column: 1,
						message: format!("Duplicate heading content found: '{}'", heading_text),
						rule: "no-duplicate-headings".to_string(),
						is_error: false,
					});
				} else {
					seen_headings.insert(heading_text.to_string());
				}
			}
		}

		if let Some(level) = extract_heading_level(line) {
			heading_levels.push((level, line_num));

			let heading_text = line.trim_start_matches('#').trim();
			if !heading_text.is_empty() {
				if seen_headings.contains(heading_text) {
					issues.push(HeadingIssue {
						line: line_num,
						column: 1,
						message: format!("Duplicate heading content found: '{}'", heading_text),
						rule: "no-duplicate-headings".to_string(),
						is_error: false,
					});
				} else {
					seen_headings.insert(heading_text.to_string());
				}
			}

			if level == 1 {
				h1_count += 1;
				h1_locations.push(line_num);
			}

			// MD022: blanks-around-headings
			// Check line before (if not first line and not after another heading/code block fence)
			if blanks_around_headings && line_num > 1 {
				if let Some(prev) = prev_line {
					if !prev.trim().is_empty()
						&& extract_heading_level(prev).is_none()
						&& !prev.trim().starts_with("```")
					{
						issues.push(HeadingIssue {
							line: line_num,
							column: 1,
							message: "Headings should be preceded by a blank line".to_string(),
							rule: "blanks-around-headings".to_string(),
							is_error: false,
						});
					}
				}
			}

			// Check line after
			if blanks_around_headings {
				if let Some(next_line) = content.lines().nth(line_num) {
					if !next_line.trim().is_empty()
						&& extract_heading_level(next_line).is_none()
						&& !next_line.trim().starts_with("```")
					{
						issues.push(HeadingIssue {
							line: line_num,
							column: 1,
							message: "Headings should be followed by a blank line".to_string(),
							rule: "blanks-around-headings".to_string(),
							is_error: false,
						});
					}
				}
			}
		}

		prev_line = Some(line);
	}

	if h1_count > 1 {
		for &location in &h1_locations[1..] {
			issues.push(HeadingIssue {
				line: location,
				column: 1,
				message:
					"Multiple H1 headings found. Documents should have only one top-level heading"
						.to_string(),
				rule: "heading-structure".to_string(),
				is_error: true,
			});
		}
	}

	if heading_levels.len() > 1 {
		for i in 1..heading_levels.len() {
			let (current_level, current_line) = heading_levels[i];
			let (prev_level, _) = heading_levels[i - 1];

			if current_level > prev_level + 1 {
				issues.push(HeadingIssue {
					line: current_line,
					column: 1,
					message: format!(
						"Heading level skipped: H{} follows H{}. Use sequential heading levels.",
						current_level, prev_level
					),
					rule: "heading-structure".to_string(),
					is_error: true,
				});
			}
		}
	}

	if issues.is_empty() {
		None
	} else {
		Some(issues)
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

	fn validate_heading_structure(content: &str) -> Option<Vec<HeadingIssue>> {
		super::validate_heading_structure(content, true)
	}

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
		let issues = result.unwrap();
		assert_eq!(issues.len(), 1); // One issue for second H1
		assert_eq!(issues[0].line, 5); // Second H1 is on line 5
		assert!(issues[0].message.contains("Multiple H1"));
	}

	#[test]
	fn test_validate_heading_structure_skipped_levels() {
		let content = "# H1\n\n### H3 (skipped H2)";
		let result = validate_heading_structure(content);
		assert!(result.is_some());
		let issues = result.unwrap();
		assert_eq!(issues.len(), 1); // One issue for skipped level
		assert_eq!(issues[0].line, 3); // H3 is on line 3
		assert!(issues[0].message.contains("skipped"));
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
		let issues = result.unwrap();
		// Should have Multiple H1 for "Another H1" AND Duplicate for "## H2"
		assert!(issues.iter().any(|i| i.message.contains("Multiple H1")));
		assert!(issues.iter().any(|i| i.rule == "no-duplicate-headings"));
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
		assert!(result.is_some()); // Should have first-line-h1 issue
		let issues = result.unwrap();
		assert_eq!(issues.len(), 1);
		assert_eq!(issues[0].rule, "first-line-h1");
	}

	#[test]
	fn test_validate_heading_structure_no_headings() {
		let content = "Just some text\n\nwith no headings.";
		let result = validate_heading_structure(content);
		assert!(result.is_some()); // Should have first-line-h1 issue
		let issues = result.unwrap();
		assert_eq!(issues.len(), 1);
		assert_eq!(issues[0].rule, "first-line-h1");
	}

	#[test]
	fn test_validate_heading_structure_false_positives() {
		let content = "This is not a heading\n\n# This is a heading\n\nNot a heading: #";
		let result = validate_heading_structure(content);
		assert!(result.is_some()); // Should have first-line-h1 issue
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
		let issues = result.unwrap();
		assert_eq!(issues.len(), 1); // One issue for skipped levels
		assert_eq!(issues[0].line, 3); // H4 is on line 3
		assert!(issues[0].message.contains("skipped"));
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
		let content = "# H1\nHeading level 1\n===============";
		let result = validate_heading_structure(content);
		assert!(result.is_some());
		let issues = result.unwrap();
		assert_eq!(issues.len(), 3); // Setext-style + Multiple H1 + blanks-around-headings
		assert!(issues.iter().any(|i| i.rule == "heading-structure"));
		assert!(issues.iter().any(|i| i.rule == "blanks-around-headings"));
	}

	#[test]
	fn test_validate_heading_structure_setext_h2() {
		let content = "# H1\nHeading level 2\n---------------";
		let result = validate_heading_structure(content);
		assert!(result.is_some());
		let issues = result.unwrap();
		assert_eq!(issues.len(), 2); // Setext-style + blanks-around-headings
		assert!(issues.iter().any(|i| i.rule == "heading-structure"));
		assert!(issues.iter().any(|i| i.rule == "blanks-around-headings"));
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

	#[test]
	fn test_validate_heading_structure_first_line_h1() {
		let content = "# Valid H1\n\nContent";
		let result = validate_heading_structure(content);
		assert!(result.is_none());
	}

	#[test]
	fn test_validate_heading_structure_first_line_not_h1() {
		let content = "## Invalid H2\n\nContent";
		let result = validate_heading_structure(content);
		assert!(result.is_some());
		let errors = result.unwrap();
		assert!(errors.iter().any(|e| e.rule == "first-line-h1"));
	}

	#[test]
	fn test_validate_heading_structure_first_line_not_heading() {
		let content = "Plain text\n\n# H1 later";
		let result = validate_heading_structure(content);
		assert!(result.is_some());
		let errors = result.unwrap();
		assert!(errors.iter().any(|e| e.rule == "first-line-h1"));
	}

	#[test]
	fn test_validate_heading_structure_first_line_empty() {
		let content = "\n\n# H1 after blanks";
		let result = validate_heading_structure(content);
		assert!(result.is_none());
	}

	#[test]
	fn test_validate_heading_structure_blanks_around() {
		let content =
			"# H1\nNo blank line after\n\n## H2\n\nHas blanks\n\n### H3\nNo blank line after";
		let result = validate_heading_structure(content);
		assert!(result.is_some());
		let issues = result.unwrap();
		assert!(issues.iter().any(|i| i.rule == "blanks-around-headings"));
	}

	#[test]
	fn test_validate_heading_structure_blanks_around_code_block() {
		let content = "# H1\n\n```rust\nfn main() {}\n```\n## H2";
		let result = validate_heading_structure(content);
		// H2 doesn't have a blank line before it, but it immediately follows a code block fence.
		// Wait, the rule checks `!prev.trim().starts_with("\`\`\`")`. So it's exempt from having a blank line if it follows a code block immediately? Actually, if there is a blank line, prev is empty. If there isn't, prev is ```. Since we have `!prev.trim().starts_with("\`\`\`")`, it's not flagged.
		assert!(result.is_none());
	}

	#[test]
	fn test_validate_heading_structure_blanks_around_no_issues() {
		let content = "# H1\n\nSome text here\n\n## H2\n\nMore text\n\n### H3\n\nEven more text";
		let result = validate_heading_structure(content);
		assert!(result.is_none());
	}

	#[test]
	fn test_validate_heading_structure_multiple_blanks_around_issues() {
		let content = "# H1\nText\n## H2\nText";
		let result = validate_heading_structure(content);
		assert!(result.is_some());
		let issues = result.unwrap();
		let blanks_issues = issues
			.iter()
			.filter(|i| i.rule == "blanks-around-headings")
			.count();
		// # H1 lacks blank after (1)
		// ## H2 lacks blank before (1)
		// ## H2 lacks blank after (1)
		assert_eq!(blanks_issues, 3);
	}

	#[test]
	fn test_validate_heading_structure_duplicate_headings() {
		let content = "# Title\n\n## Section\n\n## Section\n\n### Subsection\n\n### Subsection";
		let result = validate_heading_structure(content);
		assert!(result.is_some());
		let issues = result.unwrap();
		let duplicate_issues = issues
			.iter()
			.filter(|i| i.rule == "no-duplicate-headings")
			.count();
		assert_eq!(duplicate_issues, 2);
	}

	#[test]
	fn test_validate_heading_structure_duplicate_headings_mixed() {
		let content = "# Title\n\n## Introduction\n\n### Introduction";
		let result = validate_heading_structure(content);
		assert!(result.is_some());
		let issues = result.unwrap();
		assert!(issues.iter().any(|i| i.rule == "no-duplicate-headings"));
	}

	#[test]
	fn test_validate_heading_structure_no_duplicate_headings() {
		let content =
			"# Title\n\n## Section 1\n\n## Section 2\n\n### Subsection A\n\n### Subsection B";
		let result = validate_heading_structure(content);
		// Should be None or contain other issues (like blanks-around-headings) but not duplicate-headings
		if let Some(issues) = result {
			assert!(!issues.iter().any(|i| i.rule == "no-duplicate-headings"));
		}
	}
}
