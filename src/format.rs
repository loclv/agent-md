use crate::*;

pub fn format_markdown(content: &str) -> String {
    // Check if content ends with newline
    let ends_with_newline = content.ends_with('\n');

    let mut formatted_lines = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Calculate leading indentation (spaces/tabs before the |)
        let leading_indent = line.len() - line.trim_start().len();
        let indent_str = &line[..leading_indent];

        // Check if this looks like a table row (must start and end with |)
        if trimmed.starts_with('|') && trimmed.ends_with('|') {
            // Check for separator rows (rows with only pipes and dashes)
            let is_separator_row = trimmed
                .chars()
                .all(|c| c == '|' || c == '-' || c == ' ' || c == ':');

            // Format table cells (except separator rows)
            if !is_separator_row {
                let cells: Vec<&str> = trimmed.split('|').collect();
                let mut formatted_cells = Vec::new();

                // Process middle cells (skip first and last which are empty from leading/trailing pipes)
                for (i, cell) in cells.iter().enumerate() {
                    if i == 0 || i == cells.len() - 1 {
                        // First and last cells are empty due to leading/trailing pipes
                        continue;
                    }
                    // Trim both leading and trailing spaces from cell content
                    let cell_trimmed = cell.trim();
                    formatted_cells.push(cell_trimmed.to_string());
                }

                // Reconstruct with indentation: | cell1 | cell2 | ... |
                let formatted_line = format!("{}| {} |", indent_str, formatted_cells.join(" | "));
                formatted_lines.push(formatted_line);
            } else {
                // Keep separator rows as-is (with indentation)
                formatted_lines.push(line.to_string());
            }
        } else {
            // Non-table lines, keep as-is
            formatted_lines.push(line.to_string());
        }
    }

    let mut result = formatted_lines.join("\n");

    // Preserve trailing newline if original had one
    if ends_with_newline && !result.ends_with('\n') {
        result.push('\n');
    }

    result
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

    #[test]
    fn test_format_markdown_empty_content() {
        let content = "";
        let result = format_markdown(content);
        assert_eq!(result, "");
    }

    #[test]
    fn test_format_markdown_single_column_table() {
        let content = r#"| Header |
|---|
| Value 1  |
| Value 2   |
| Value 3 |
"#;
        let expected = r#"| Header |
|---|
| Value 1 |
| Value 2 |
| Value 3 |
"#;
        let result = format_markdown(content);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_markdown_no_trailing_newline() {
        let content = "| Header |\n|---|\n| Value |";
        let result = format_markdown(content);
        assert!(!result.ends_with('\n'));
        assert_eq!(result, "| Header |\n|---|\n| Value |");
    }

    #[test]
    fn test_format_markdown_mixed_content_with_code_block() {
        let content = r#"# Document

Some text.

```rust
| Not | A | Table |
```

| Real | Table |
|---|---|
| Has | Spaces   |

More text.
"#;
        let expected = r#"# Document

Some text.

```rust
| Not | A | Table |
```

| Real | Table |
|---|---|
| Has | Spaces |

More text.
"#;
        let result = format_markdown(content);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_markdown_table_with_inline_code() {
        let content = r#"| Function | Description |
|---|---|
| `test()`  | Runs tests   |
| `main()`  | Entry point  |
"#;
        let expected = r#"| Function | Description |
|---|---|
| `test()` | Runs tests |
| `main()` | Entry point |
"#;
        let result = format_markdown(content);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_markdown_table_with_special_chars() {
        let content = r#"| Symbol | Meaning |
|---|---|
| ->   | Arrow    |
| =>   | Fat arrow |
| <>   | Not equal |
"#;
        let expected = r#"| Symbol | Meaning |
|---|---|
| -> | Arrow |
| => | Fat arrow |
| <> | Not equal |
"#;
        let result = format_markdown(content);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_markdown_long_cell_content() {
        let content = r#"| Short | Long Description |
|---|---|
| A | This is a very long description with many words   |
"#;
        let expected = r#"| Short | Long Description |
|---|---|
| A | This is a very long description with many words |
"#;
        let result = format_markdown(content);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_markdown_multiple_tables() {
        let content = r#"# Tables

| Table 1 | Col 2 |
|---|---|
| A  | B   |

Some text.

| Table 2 | Col 2 |
|---|---|
| X  | Y   |
"#;
        let expected = r#"# Tables

| Table 1 | Col 2 |
|---|---|
| A | B |

Some text.

| Table 2 | Col 2 |
|---|---|
| X | Y |
"#;
        let result = format_markdown(content);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_markdown_table_with_links() {
        let content = r#"| Name | Link |
|---|---|
| Rust  | [rust-lang.org](https://rust-lang.org)  |
"#;
        let expected = r#"| Name | Link |
|---|---|
| Rust | [rust-lang.org](https://rust-lang.org) |
"#;
        let result = format_markdown(content);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_markdown_non_table_pipes_unchanged() {
        let content = r#"This | is | not | a | table
Just text with | pipes
"#;
        let result = format_markdown(content);
        assert_eq!(result, content);
    }

    #[test]
    fn test_format_markdown_preserve_indentation() {
        let content = r#"  | Indented | Table |
  |---|---|
  | Value 1  | Value 2   |
"#;
        let expected = r#"  | Indented | Table |
  |---|---|
  | Value 1 | Value 2 |
"#;
        let result = format_markdown(content);
        assert_eq!(result, expected);
    }
}
