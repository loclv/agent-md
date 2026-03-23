use crate::*;

pub fn format_markdown(content: &str) -> String {
    let mut formatted_lines = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Check if this looks like a table row (must start and end with |)
        if trimmed.starts_with('|') && trimmed.ends_with('|') {
            // Check for separator rows (rows with only pipes and dashes)
            let is_separator_row = trimmed
                .chars()
                .all(|c| c == '|' || c == '-' || c == ' ' || c == ':');

            // Format table cells (except separator rows)
            if !is_separator_row {
                let mut formatted_cells: Vec<String> = Vec::new();
                let cells: Vec<&str> = trimmed.split('|').collect();

                for (i, cell) in cells.iter().enumerate() {
                    // Skip empty cells at start and end (from leading/trailing pipes)
                    if i == 0 || i == cells.len() - 1 {
                        formatted_cells.push(cell.to_string());
                        continue;
                    }

                    // Trim trailing spaces from cell content
                    let cell_trimmed = cell.trim_end();
                    formatted_cells.push(cell_trimmed.to_string());
                }

                // Reconstruct the line with formatted cells
                let mut all_cells = Vec::new();
                all_cells.push(cells[0].to_string());
                all_cells.extend(formatted_cells);
                all_cells.push(cells[cells.len()-1].to_string());
                let formatted_line = all_cells.join("|");
                formatted_lines.push(formatted_line);
            } else {
                // Keep separator rows as-is
                formatted_lines.push(line.to_string());
            }
        } else {
            // Non-table lines, keep as-is
            formatted_lines.push(line.to_string());
        }
    }

    formatted_lines.join("\n")
}

pub fn cmd_fmt(path: &str, human: bool) {
    match std::fs::read_to_string(path) {
        Ok(content) => {
            let formatted_content = format_markdown(&content);

            match std::fs::write(path, &formatted_content) {
                Ok(_) => {
                    let mut doc = parse_markdown(&formatted_content);
                    doc.path = path.to_string();
                    println!(
                        "{}",
                        json_output(
                            &EditResult {
                                success: true,
                                message: "File formatted successfully".to_string(),
                                document: Some(doc),
                            },
                            human
                        )
                    );
                }
                Err(e) => {
                    println!(
                        "{}",
                        json_output(
                            &EditResult {
                                success: false,
                                message: format!("Failed to write file: {}", e),
                                document: None,
                            },
                            human
                        )
                    );
                }
            }
        }
        Err(e) => {
            println!(
                "{}",
                json_output(
                    &EditResult {
                        success: false,
                        message: format!("Failed to read file: {}", e),
                        document: None,
                    },
                    human
                )
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_markdown_table_trailing_spaces() {
        let content = r#"| Column 1 | Column 2 | Column 3 |
|---|---|---|
| Value 1  | Value 2   | Value 3 |
| Another  | Test   | Here |
"#;
        let expected = r#"| Column 1 | Column 2 | Column 3 |
|---|---|---|
| Value 1 | Value 2 | Value 3 |
| Another | Test | Here |
"#;
        let result = format_markdown(content);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_markdown_table_preserve_leading_spaces() {
        let content = r#"| Column 1 | Column 2 |
|---|---|
| Value 1 |  Value 2 |
|  Value 3 | Value 4 |
"#;
        let expected = r#"| Column 1 | Column 2 |
|---|---|
| Value 1 | Value 2 |
| Value 3 | Value 4 |
"#;
        let result = format_markdown(content);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_markdown_separator_rows_unchanged() {
        let content = r#"| Header 1 | Header 2 |
|---|---|
| Value 1 | Value 2 |
"#;
        let expected = r#"| Header 1 | Header 2 |
|---|---|
| Value 1 | Value 2 |
"#;
        let result = format_markdown(content);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_markdown_non_table_lines_unchanged() {
        let content = r#"# Title

This is a paragraph.

| Column 1 | Column 2 |
|---|---|
| Value 1  | Value 2 |

Another paragraph.
"#;
        let expected = r#"# Title

This is a paragraph.

| Column 1 | Column 2 |
|---|---|
| Value 1 | Value 2 |

Another paragraph.
"#;
        let result = format_markdown(content);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_markdown_empty_cells() {
        let content = r#"| Column 1 | Column 2 | Column 3 |
|---|---|---|
| Value 1  |  | Value 3 |
|  | Value 2  |  |
"#;
        let expected = r#"| Column 1 | Column 2 | Column 3 |
|---|---|---|
| Value 1 |  | Value 3 |
|  | Value 2 |  |
"#;
        let result = format_markdown(content);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_markdown_complex_table() {
        let content = r#"| Name | Age | City      | Notes |
|------|-----|-----------|-------|
| John | 25  | New York  | Test  |
| Jane | 30  | London    |  |
| Bob  | 35  | Paris     | Data  |
"#;
        let expected = r#"| Name | Age | City | Notes |
|------|-----|-----------|-------|
| John | 25 | New York | Test |
| Jane | 30 | London |  |
| Bob | 35 | Paris | Data |
"#;
        let result = format_markdown(content);
        assert_eq!(result, expected);
    }
}
