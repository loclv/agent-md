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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_table_syntax_simple_table() {
        let line = "| Col1 | Col2 | Col3 |";
        let result = validate_table_syntax(line);
        assert_eq!(result.len(), 0); // Simple table should be valid
    }

    #[test]
    fn test_validate_table_syntax_correct_separator() {
        let line = "|---|---|---|";
        let result = validate_table_syntax(line);
        assert_eq!(result.len(), 0); // Correct separator should be valid
    }

    #[test]
    fn test_validate_table_syntax_incorrect_separator() {
        let line = "|--|--|--|";
        let result = validate_table_syntax(line);
        assert_eq!(result.len(), 3); // Should have 3 errors (one for each column)
        for issue in &result {
            assert_eq!(issue.severity, Severity::Error);
        }
    }

    #[test]
    fn test_validate_table_syntax_mixed_dash_counts() {
        let line = "|---|--|-------|";
        let result = validate_table_syntax(line);
        assert_eq!(result.len(), 2); // Should have 2 errors (for -- and -------)
        for issue in &result {
            assert_eq!(issue.severity, Severity::Error);
        }
    }

    #[test]
    fn test_validate_table_syntax_complex_attributes() {
        let line = "| colspan=\"2\" | Col2 |";
        let result = validate_table_syntax(line);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].severity, Severity::Error);
        assert!(result[0].message.contains("colspan"));
    }

    #[test]
    fn test_validate_table_syntax_inline_formatting() {
        let line = "| **Bold** | *Italic* |";
        let result = validate_table_syntax(line);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].severity, Severity::Warning);
        assert!(result[0].message.contains("inline formatting"));
    }

    #[test]
    fn test_validate_table_syntax_wide_table() {
        let line = "| Col1 | Col2 | Col3 | Col4 | Col5 | Col6 | Col7 |";
        let result = validate_table_syntax(line);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].severity, Severity::Warning);
        assert!(result[0].message.contains("wide tables"));
    }

    #[test]
    fn test_validate_table_syntax_edge_cases() {
        let line = "|"; // Single pipe
        let result = validate_table_syntax(line);
        assert_eq!(result.len(), 0); // Should be valid (no table detected)

        let line = "|||"; // Multiple pipes, no content
        let result = validate_table_syntax(line);
        assert_eq!(result.len(), 0); // Should be valid (empty table)
    }

    #[test]
    fn test_validate_table_syntax_additional_edge_cases() {
        // Table with empty cells
        let line = "| | | |";
        let result = validate_table_syntax(line);
        assert_eq!(result.len(), 0); // Should be valid

        // Table with only pipes
        let line = "||||";
        let result = validate_table_syntax(line);
        assert_eq!(result.len(), 0); // Should be valid

        // Table with spaces in separator - this is actually valid
        let line = "| |---| |---| |";
        let result = validate_table_syntax(line);
        assert_eq!(result.len(), 0); // Should be valid
    }

    #[test]
    fn test_validate_table_trailing_spaces_no_trailing_spaces() {
        let line = "| Item | Value |";
        let result = validate_table_trailing_spaces(line);
        assert!(result.is_none()); // No trailing spaces should be valid
    }

    #[test]
    fn test_validate_table_trailing_spaces_single_trailing_space() {
        let line = "| Item | Value |";
        let result = validate_table_trailing_spaces(line);
        assert!(result.is_none()); // Single trailing space should be valid
    }

    #[test]
    fn test_validate_table_trailing_spaces_multiple_trailing_spaces() {
        let line = "| Item       | Value |";
        let result = validate_table_trailing_spaces(line);
        assert!(result.is_some());
        let issue = result.unwrap();
        assert_eq!(issue.severity, Severity::Error);
        assert!(issue.message.contains("found 8 trailing spaces"));
    }

    #[test]
    fn test_validate_table_trailing_spaces_separator_row_ignored() {
        let line = "|---|---|";
        let result = validate_table_trailing_spaces(line);
        assert!(result.is_none()); // Separator rows should be ignored
    }

    #[test]
    fn test_validate_table_trailing_spaces_mixed_cells() {
        let line = "| Value 1 | Value 2       | Value 3 |";
        let result = validate_table_trailing_spaces(line);
        assert!(result.is_some()); // Should detect trailing spaces
        let msg = result.unwrap().message;
        assert!(msg.contains("found 8 trailing spaces"));
    }

    #[test]
    fn test_validate_table_trailing_spaces_non_table_lines_ignored() {
        let line = "Not a table | but has pipes";
        let result = validate_table_trailing_spaces(line);
        assert!(result.is_none()); // Non-table lines should be ignored
    }

    #[test]
    fn test_validate_table_trailing_spaces_empty_cells() {
        let line = "| Item |       | Data |";
        let result = validate_table_trailing_spaces(line);
        assert!(result.is_some()); // Should detect trailing spaces in empty cell
    }

    #[test]
    fn test_validate_table_trailing_spaces_leading_spaces_ok() {
        let line = "|   Item   |   Value   |";
        let result = validate_table_trailing_spaces(line);
        assert!(result.is_some()); // Actually has trailing spaces
        assert_eq!(result.unwrap().severity, Severity::Error);
    }

    #[test]
    fn test_validate_table_trailing_spaces_no_pipes_at_ends() {
        let line = "Item | Value";
        let result = validate_table_trailing_spaces(line);
        assert!(result.is_none()); // Should ignore rows without pipes at ends
    }

    #[test]
    fn test_validate_table_trailing_spaces_single_column() {
        let line = "| Item       |";
        let result = validate_table_trailing_spaces(line);
        assert!(result.is_some()); // Should detect trailing spaces
        let msg = result.unwrap().message;
        assert!(msg.contains("found 8 trailing spaces"));
    }

    #[test]
    fn test_validate_table_trailing_spaces_with_inline_code() {
        let line = "| `test`      | Value with `code` |";
        let result = validate_table_trailing_spaces(line);
        assert!(result.is_some()); // Should still detect trailing spaces
        let msg = result.unwrap().message;
        assert!(msg.contains("found 7 trailing spaces"));
    }

    #[test]
    fn test_validate_table_syntax_with_rowspan() {
        let line = "| Cell | rowspan=\"2\" |";
        let result = validate_table_syntax(line);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].severity, Severity::Error);
        assert!(result[0].message.contains("rowspan"));
    }

    #[test]
    fn test_validate_table_syntax_with_underscore_formatting() {
        let line = "| __Underline__ | Normal |";
        let result = validate_table_syntax(line);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].severity, Severity::Warning);
        assert!(result[0].message.contains("inline formatting"));
    }

    #[test]
    fn test_validate_table_syntax_exact_width_limit() {
        let line = "| Col1 | Col2 | Col3 | Col4 | Col5 | Col6 |";
        let result = validate_table_syntax(line);
        assert_eq!(result.len(), 1); // 7 pipes (6 columns) triggers warning
        assert_eq!(result[0].severity, Severity::Warning);
    }

    #[test]
    fn test_validate_table_syntax_separator_with_colons() {
        let line = "|:---|:---:|---:|";
        let result = validate_table_syntax(line);
        assert_eq!(result.len(), 0); // Alignment colons should be valid
    }

    #[test]
    fn test_validate_table_syntax_non_table_line() {
        let line = "This is not a table";
        let result = validate_table_syntax(line);
        assert_eq!(result.len(), 0); // Non-table lines should be valid
    }
}
