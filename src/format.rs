use crate::*;

pub fn format_markdown(content: &str) -> String {
    // Check if content ends with newline
    let ends_with_newline = content.ends_with('\n');

    let mut formatted_lines = Vec::new();
    let mut in_code_block = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Track code block boundaries
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            formatted_lines.push(line.to_string());
            continue;
        }

        // Inside code blocks, preserve content as-is
        if in_code_block {
            formatted_lines.push(line.to_string());
            continue;
        }

        // Calculate leading indentation (spaces/tabs before content)
        let leading_indent = line.len() - line.trim_start().len();
        let indent_str = &line[..leading_indent];

        // Check for table rows - either direct |...| or prefixed like > |...| or - |...|
        let (prefix, table_content) = if trimmed.starts_with('|') && trimmed.ends_with('|') {
            // Direct table row
            (indent_str, trimmed)
        } else if (trimmed.starts_with("> |") || trimmed.starts_with("- |"))
            && trimmed.ends_with('|')
        {
            // Table row with blockquote or list prefix
            let table_start = trimmed.find('|').unwrap_or(0);
            let prefix_part = &trimmed[..table_start];
            let table_part = &trimmed[table_start..];
            (&line[..leading_indent + prefix_part.len()], table_part)
        } else {
            // Not a table row
            ("", "")
        };

        if !table_content.is_empty() {
            // Check for separator rows (rows with pipes, dashes, and optional colons - must have at least one dash)
            let is_separator_row = table_content
                .chars()
                .all(|c| c == '|' || c == '-' || c == ' ' || c == ':')
                && table_content.contains('-');

            // Format table cells (except separator rows)
            if !is_separator_row {
                let cells: Vec<&str> = table_content.split('|').collect();
                let mut formatted_cells = Vec::new();

                // Process middle cells (skip first and last which are empty from leading/trailing pipes)
                for (i, cell) in cells.iter().enumerate() {
                    if i == 0 || i == cells.len() - 1 {
                        // Example: "| cell1 | cell2 |" splits to ["", " cell1 ", " cell2 ", ""]
                        continue;
                    }
                    // Trim both leading and trailing spaces from cell content
                    let cell_trimmed = cell.trim();
                    // Push trimmed content (empty string "" becomes "  |" after format)
                    formatted_cells.push(cell_trimmed.to_string());
                }

                // Reconstruct with indentation and prefix: prefix| cell1 | cell2 | ... |
                let formatted_line = format!("{}| {} |", prefix, formatted_cells.join(" | "));
                formatted_lines.push(formatted_line);
            } else {
                // Keep separator rows as-is (with indentation)
                formatted_lines.push(line.to_string());
            }
        } else {
            // Non-table lines: remove bold markers while preserving content
            let line_without_bold = remove_bold_markers(line);
            formatted_lines.push(line_without_bold);
        }
    }

    let mut result = formatted_lines.join("\n");

    // Preserve trailing newline if original had one
    if ends_with_newline && !result.ends_with('\n') {
        result.push('\n');
    }

    result
}

/// Remove bold markers (** and __) from a line while preserving the content inside.
/// This skips markers inside inline code spans.
fn remove_bold_markers(line: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Check for inline code start
        if chars[i] == '`' {
            // Find the end of the inline code
            let mut code_end = i + 1;
            while code_end < chars.len() {
                if chars[code_end] == '`' {
                    break;
                }
                code_end += 1;
            }
            // Copy the entire code span as-is
            for j in i..=code_end {
                if j < chars.len() {
                    result.push(chars[j]);
                }
            }
            i = code_end + 1;
            continue;
        }

        // Check for **bold** pattern
        if i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '*' {
            // Find the closing **
            let mut j = i + 2;
            while j + 1 < chars.len() {
                if chars[j] == '*' && chars[j + 1] == '*' {
                    // Found closing marker, copy content between markers
                    chars[i + 2..j].iter().for_each(|&c| result.push(c));
                    i = j + 2;
                    break;
                }
                j += 1;
            }
            // If no closing marker found, copy the ** as-is
            if i <= j {
                result.push(chars[i]);
                i += 1;
            }
            continue;
        }

        // Check for __bold__ pattern
        if i + 1 < chars.len() && chars[i] == '_' && chars[i + 1] == '_' {
            // Find the closing __
            let mut j = i + 2;
            while j + 1 < chars.len() {
                if chars[j] == '_' && chars[j + 1] == '_' {
                    // Found closing marker, copy content between markers
                    chars[i + 2..j].iter().for_each(|&c| result.push(c));
                    i = j + 2;
                    break;
                }
                j += 1;
            }
            // If no closing marker found, copy the __ as-is
            if i <= j {
                result.push(chars[i]);
                i += 1;
            }
            continue;
        }

        // Regular character, just copy it
        result.push(chars[i]);
        i += 1;
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

    // Additional comprehensive tests
    #[test]
    fn test_format_markdown_tab_indentation() {
        let content = "\t| Tabbed | Table |\n\t|---|---|\n\t| Value 1  | Value 2 |\n";
        let expected = "\t| Tabbed | Table |\n\t|---|---|\n\t| Value 1 | Value 2 |\n";
        let result = format_markdown(content);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_markdown_only_newlines() {
        let content = "\n\n\n";
        let result = format_markdown(content);
        assert_eq!(result, "\n\n");
    }

    #[test]
    fn test_format_markdown_single_pipe_not_table() {
        let content = "| This is not a table because it doesn't end with pipe\n";
        let result = format_markdown(content);
        assert_eq!(result, content);
    }

    #[test]
    fn test_format_markdown_table_with_unicode() {
        let content = "| Emoji | Text |\n|---|---|\n| 🎉  | Party  |\n| 🦀  | Rust |\n";
        let expected = "| Emoji | Text |\n|---|---|\n| 🎉 | Party |\n| 🦀 | Rust |\n";
        let result = format_markdown(content);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_markdown_table_with_numbers() {
        let content = "| ID | Count |\n|---|---|\n| 1   | 100   |\n| 2   | 200 |\n";
        let expected = "| ID | Count |\n|---|---|\n| 1 | 100 |\n| 2 | 200 |\n";
        let result = format_markdown(content);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_markdown_many_columns() {
        let content = "| A | B | C | D | E | F | G | H |\n|---|---|---|---|---|---|---|---|\n| 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 |\n";
        let expected = "| A | B | C | D | E | F | G | H |\n|---|---|---|---|---|---|---|---|\n| 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 |\n";
        let result = format_markdown(content);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_markdown_table_in_blockquote() {
        let content = "> | Quote | Table |\n> |---|---|\n> | Data 1  | Data 2 |\n";
        let expected = "> | Quote | Table |\n> |---|---|\n> | Data 1 | Data 2 |\n";
        let result = format_markdown(content);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_markdown_list_item_with_table() {
        let content = "- | List | Item |\n- |---|---|\n- | Data  | Value |\n";
        let expected = "- | List | Item |\n- |---|---|\n- | Data | Value |\n";
        let result = format_markdown(content);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_markdown_empty_table_cells() {
        let content = "| A | B | C |\n|---|---|---|\n|   |   |   |\n";
        let expected = "| A | B | C |\n|---|---|---|\n|  |  |  |\n";
        let result = format_markdown(content);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_markdown_windows_line_endings() {
        let content = "| Header |\r\n|---|\r\n| Value  |\r\n";
        let expected = "| Header |\n|---|\n| Value |\n";
        let result = format_markdown(content);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_markdown_nested_pipes_in_code() {
        let content = "| Code | Output |\n|---|---|\n| `a | b`  | Result |\n";
        let expected = "| Code | Output |\n|---|---|\n| `a | b` | Result |\n";
        let result = format_markdown(content);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_markdown_single_row_table() {
        let content = "| Only | Header | Row |\n";
        let expected = "| Only | Header | Row |\n";
        let result = format_markdown(content);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_markdown_bold_in_code_block_preserved() {
        let content = r#"```
This has **bold** text
Also has __bold__ markers
```

Regular text with **bold** and __bold__ should be removed.
"#;
        let expected = r#"```
This has **bold** text
Also has __bold__ markers
```

Regular text with bold and bold should be removed.
"#;
        let result = format_markdown(content);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_markdown_bold_in_inline_code_preserved() {
        let content = r#"Use `**bold**` for emphasis or `__bold__` syntax.

But **remove** these __markers__ outside code.
"#;
        let expected = r#"Use `**bold**` for emphasis or `__bold__` syntax.

But remove these markers outside code.
"#;
        let result = format_markdown(content);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_markdown_bold_in_table_inline_code_preserved() {
        let content = r#"| Syntax | Example |
|---|---|
| Bold | `**text**` |
| Underline | `__text__` |
"#;
        let expected = r#"| Syntax | Example |
|---|---|
| Bold | `**text**` |
| Underline | `__text__` |
"#;
        let result = format_markdown(content);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_markdown_separator_with_colons() {
        let content = "| Left | Center | Right |\n|:---|:---:|---:|\n| A | B | C |\n";
        let expected = "| Left | Center | Right |\n|:---|:---:|---:|\n| A | B | C |\n";
        let result = format_markdown(content);
        assert_eq!(result, expected);
    }
}
