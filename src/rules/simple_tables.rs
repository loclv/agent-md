#[derive(Debug, PartialEq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, PartialEq)]
pub struct TableIssue {
    pub column: usize,
    pub message: String,
    pub severity: Severity,
}

pub fn validate_table_syntax(line: &str) -> Vec<TableIssue> {
    let mut issues = Vec::new();
    let trimmed = line.trim();

    if trimmed.contains('|') {
        let is_separator_row = trimmed
            .chars()
            .all(|c| c == '|' || c == '-' || c == ' ' || c == ':');

        if is_separator_row {
            let parts: Vec<&str> = trimmed.split('|').collect();
            for part in parts {
                let part_trimmed = part.trim();
                if !part_trimmed.is_empty() {
                    let dash_count = part_trimmed.chars().filter(|&c| c == '-').count();
                    if dash_count != 3 {
                        issues.push(TableIssue {
                            column: 1,
                            message:
                                "Table separator should use exactly 3 dashes (---) between pipes"
                                    .to_string(),
                            severity: Severity::Error,
                        });
                    }
                }
            }
            if !issues.is_empty() {
                return issues;
            }
        }

        if trimmed.contains("colspan") || trimmed.contains("rowspan") {
            issues.push(TableIssue {
                column: 1,
                message: "Complex table attributes (colspan/rowspan) are not allowed".to_string(),
                severity: Severity::Error,
            });
            return issues;
        }

        if trimmed.contains("**") || trimmed.contains("__") || trimmed.contains("*") {
            issues.push(TableIssue {
                column: 1,
                message: "inline formatting in table cells should be avoided".to_string(),
                severity: Severity::Warning,
            });
            return issues;
        }

        let pipe_count = trimmed.matches('|').count();
        if pipe_count > 6 {
            issues.push(TableIssue {
                column: 1,
                message: "Very wide tables should be simplified".to_string(),
                severity: Severity::Warning,
            });
        }
    }

    issues
}

pub fn validate_table_trailing_spaces(line: &str) -> Option<TableIssue> {
    let trimmed = line.trim();

    if trimmed.starts_with('|') && trimmed.ends_with('|') {
        let is_separator_row = trimmed
            .chars()
            .all(|c| c == '|' || c == '-' || c == ' ' || c == ':');

        if !is_separator_row {
            let cells: Vec<&str> = trimmed.split('|').collect();
            for (i, cell) in cells.iter().enumerate() {
                if i == 0 || i == cells.len() - 1 {
                    continue;
                }

                let cell_trimmed = cell.trim();
                let trailing_spaces = cell.len() - cell_trimmed.len();

                if trailing_spaces > 2 {
                    return Some(TableIssue {
                        column: 1,
                        message: format!("Table cell should not have trailing spaces, found {} trailing spaces (should be 0, 1, or 2)", trailing_spaces),
                        severity: Severity::Error,
                    });
                }
            }
        }
    }

    None
}
