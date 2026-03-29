use crate::LintError;

pub fn validate_heading_structure(content: &str) -> Option<Vec<LintError>> {
    let mut heading_levels = Vec::new();
    let mut h1_count = 0;
    let mut h1_locations = Vec::new();
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

        if let Some(level) = extract_heading_level(line) {
            heading_levels.push((level, line_num));

            if level == 1 {
                h1_count += 1;
                h1_locations.push(line_num);
            }
        }
    }

    let mut errors = Vec::new();

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
