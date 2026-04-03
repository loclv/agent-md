use crate::LintError;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ListType {
	Ordered,
	Unordered,
}

pub fn validate_list_formatting(content: &str) -> Option<Vec<LintError>> {
	let mut errors = Vec::new();
	let mut list_items = Vec::new();
	let mut in_code_block = false;

	for (line_num, line) in content.lines().enumerate() {
		let line_num = line_num + 1;

		if line.trim().starts_with("```") {
			in_code_block = !in_code_block;
			continue;
		}

		if in_code_block {
			continue;
		}

		let trimmed = line.trim();

		if let Some(list_info) = detect_list_item(trimmed) {
			list_items.push((list_info, line_num));
		}
	}

	if list_items.len() > 1 {
		let mut current_list_type: Option<ListType> = None;
		let mut _current_list_marker: Option<String> = None;
		let mut expected_next_number: Option<u32> = None;

		for (i, ((list_type, marker), line_num)) in list_items.iter().enumerate() {
			if i > 0 {
				let prev_line_num = list_items[i - 1].1;
				if *line_num > prev_line_num + 1 {
					current_list_type = None;
					_current_list_marker = None;
					expected_next_number = None;
				}
			}

			if current_list_type.is_none() {
				current_list_type = Some(*list_type);
				_current_list_marker = Some(marker.to_string());

				if *list_type == ListType::Ordered {
					if let Some(current_num) = extract_number_from_marker(marker) {
						expected_next_number = Some(current_num + 1);
					}
				}
			} else {
				if current_list_type != Some(*list_type)
					|| (*list_type == ListType::Unordered
						&& _current_list_marker.as_ref() != Some(marker))
				{
					errors.push(LintError {
                        line: *line_num,
                        column: 1,
                        message: "inconsistent list formatting detected. Use consistent list markers within the same list".to_string(),
                        rule: "list-formatting".to_string(),
                    });
					break;
				}

				if *list_type == ListType::Ordered {
					if let Some(expected_num) = expected_next_number {
						let expected_marker = format!("{}.", expected_num);
						if *marker != expected_marker {
							errors.push(LintError {
								line: *line_num,
								column: 1,
								message:
									"Inconsistent ordered list numbering. Use sequential numbers"
										.to_string(),
								rule: "list-formatting".to_string(),
							});
						}
					}

					if let Some(current_num) = extract_number_from_marker(marker) {
						_current_list_marker = Some(marker.to_string());
						expected_next_number = Some(current_num + 1);
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

pub fn extract_number_from_marker(marker: &str) -> Option<u32> {
	let mut separator_pos = None;
	for (i, c) in marker.chars().enumerate() {
		if c == '.' || c == ')' {
			separator_pos = Some(i);
			break;
		}
	}

	if let Some(pos) = separator_pos {
		if pos == 0 {
			return None;
		}

		let num_part = &marker[..pos];
		if num_part.chars().all(|c| c.is_ascii_digit()) {
			num_part.parse::<u32>().ok()
		} else {
			None
		}
	} else {
		None
	}
}

pub fn detect_list_item(line: &str) -> Option<(ListType, String)> {
	if line.len() >= 2 {
		let first_char = line.chars().next().unwrap();
		let second_char = line.chars().nth(1).unwrap();

		if (first_char == '-' || first_char == '*' || first_char == '+') && second_char == ' ' {
			return Some((ListType::Unordered, first_char.to_string()));
		}
	}

	if line.len() >= 3 {
		let mut i = 0;
		let mut has_digits = false;

		while i < line.len() && line.chars().nth(i).unwrap().is_ascii_digit() {
			has_digits = true;
			i += 1;
		}

		if has_digits && i < line.len() {
			let separator = line.chars().nth(i).unwrap();
			if (separator == '.' || separator == ')') && i + 1 < line.len() {
				let next_char = line.chars().nth(i + 1).unwrap();
				if next_char == ' ' {
					let marker = line[..i + 1].to_string();
					return Some((ListType::Ordered, marker));
				}
			}
		}
	}

	None
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_detect_list_item_unordered() {
		assert_eq!(
			detect_list_item("- Item"),
			Some((ListType::Unordered, "-".to_string()))
		);
		assert_eq!(
			detect_list_item("* Item"),
			Some((ListType::Unordered, "*".to_string()))
		);
		assert_eq!(
			detect_list_item("+ Item"),
			Some((ListType::Unordered, "+".to_string()))
		);
	}

	#[test]
	fn test_detect_list_item_ordered() {
		assert_eq!(
			detect_list_item("1. Item"),
			Some((ListType::Ordered, "1.".to_string()))
		);
		assert_eq!(
			detect_list_item("2) Item"),
			Some((ListType::Ordered, "2)".to_string()))
		);
		assert_eq!(
			detect_list_item("123. Item"),
			Some((ListType::Ordered, "123.".to_string()))
		);
	}

	#[test]
	fn test_detect_list_item_invalid() {
		assert_eq!(detect_list_item("Item"), None);
		assert_eq!(detect_list_item("-Item"), None); // No space after marker
		assert_eq!(detect_list_item("1.Item"), None); // No space after number
		assert_eq!(detect_list_item("1 .Item"), None); // Space before dot
		assert_eq!(detect_list_item(""), None);
	}

	#[test]
	fn test_detect_list_item_edge_cases() {
		assert_eq!(
			detect_list_item("- "),
			Some((ListType::Unordered, "-".to_string()))
		); // Empty item
		assert_eq!(
			detect_list_item("1. "),
			Some((ListType::Ordered, "1.".to_string()))
		); // Empty item
		assert_eq!(detect_list_item("-Item with no space"), None);
		assert_eq!(detect_list_item("1.No space"), None);
	}

	#[test]
	fn test_extract_number_from_marker_valid() {
		assert_eq!(extract_number_from_marker("1."), Some(1));
		assert_eq!(extract_number_from_marker("2)"), Some(2));
		assert_eq!(extract_number_from_marker("123."), Some(123));
	}

	#[test]
	fn test_extract_number_from_marker_invalid() {
		assert_eq!(extract_number_from_marker("a."), None);
		assert_eq!(extract_number_from_marker("1a."), None);
		assert_eq!(extract_number_from_marker(""), None);
		assert_eq!(extract_number_from_marker("."), None);
	}

	#[test]
	fn test_validate_list_formatting_consistent_unordered() {
		let content = "- First item\n- Second item\n- Third item";
		let result = validate_list_formatting(content);
		assert!(result.is_none()); // Should be valid
	}

	#[test]
	fn test_validate_list_formatting_ordered_sequential() {
		let content = "1. First item\n2. Second item\n3. Third item";
		let result = validate_list_formatting(content);
		assert!(result.is_none()); // Should be valid
	}

	#[test]
	fn test_validate_list_formatting_ordered_nonsequential() {
		let content = "1. First item\n3. Third item\n4. Fourth item";
		let result = validate_list_formatting(content);
		assert!(result.is_some());
		let warnings = result.unwrap();
		assert_eq!(warnings.len(), 1); // One warning for non-sequential numbering
		assert_eq!(warnings[0].line, 2); // Third item is on line 2
		assert!(warnings[0].message.contains("sequential"));
	}

	#[test]
	fn test_validate_list_formatting_inconsistent_markers() {
		let content = "- First item\n* Second item\n- Third item";
		let result = validate_list_formatting(content);
		assert!(result.is_some());
		let warnings = result.unwrap();
		assert_eq!(warnings.len(), 1); // One warning for inconsistent marker
		assert_eq!(warnings[0].line, 2); // Second item is on line 2
		assert!(warnings[0].message.contains("inconsistent"));
	}

	#[test]
	fn test_validate_list_formatting_separate_lists() {
		let content = "- First list\n- Second list\n\n1. New list\n2. Second item";
		let result = validate_list_formatting(content);
		assert!(result.is_none()); // Should be valid (separate lists)
	}

	#[test]
	fn test_validate_list_formatting_mixed_ordered_unordered() {
		let content = r#"1. First ordered

- First unordered

2. Second ordered

- Second unordered

3. Third ordered
"#;
		let result = validate_list_formatting(content);
		// Should be valid - blank lines reset list context
		assert!(result.is_none());
	}

	#[test]
	fn test_validate_list_formatting_deeply_nested() {
		let content = r#"1. First
2. Second
3. Third
4. Fourth
5. Fifth
"#;
		let result = validate_list_formatting(content);
		// Should be valid - sequential numbering
		assert!(result.is_none());
	}

	#[test]
	fn test_validate_list_formatting_single_item() {
		let content = "- Single item";
		let result = validate_list_formatting(content);
		assert!(result.is_none()); // Should be valid (single item)
	}

	#[test]
	fn test_validate_list_formatting_with_code_block() {
		let content = r#"- Item 1
- Item 2

```
- Not a list item in code block
1. Also not a list item
```

- Item 3
"#;
		let result = validate_list_formatting(content);
		assert!(result.is_none()); // Should be valid (ignores code blocks)
	}

	#[test]
	fn test_validate_list_formatting_ordered_with_parentheses() {
		let content = "1) First item\n2) Second item\n3) Third item";
		let result = validate_list_formatting(content);
		assert!(result.is_some()); // Should error (parentheses not supported in numbering check)
		let errors = result.unwrap();
		assert_eq!(errors.len(), 2); // Lines 2 and 3
	}

	#[test]
	fn test_validate_list_formatting_mixed_separators() {
		let content = "1. First item\n2) Second item\n3. Third item";
		let result = validate_list_formatting(content);
		assert!(result.is_some()); // Should error (mixed separators)
		let errors = result.unwrap();
		assert_eq!(errors.len(), 1);
		assert_eq!(errors[0].line, 2);
	}
}
