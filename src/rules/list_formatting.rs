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
