use crate::*;

#[derive(Clone)]
pub struct FormatOptions {
    pub remove_bold: bool,
    pub compact_blank_lines: bool,
    pub trim_trailing_whitespace: bool,
    pub collapse_spaces: bool,
    pub remove_horizontal_rules: bool,
    pub remove_emphasis: bool,
}

impl FormatOptions {
    pub fn token_saver() -> Self {
        Self {
            remove_bold: true,
            compact_blank_lines: true,
            trim_trailing_whitespace: true,
            collapse_spaces: true,
            remove_horizontal_rules: true,
            remove_emphasis: true,
        }
    }
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            remove_bold: true,
            compact_blank_lines: false,
            trim_trailing_whitespace: true,
            collapse_spaces: false,
            remove_horizontal_rules: false,
            remove_emphasis: false,
        }
    }
}

#[allow(dead_code)]
pub fn format_markdown(content: &str) -> String {
    format_markdown_with_options(content, FormatOptions::default())
}

pub fn format_markdown_with_options(content: &str, options: FormatOptions) -> String {
    let ends_with_newline = content.ends_with('\n');

    // First pass: collect all lines to determine which blank lines to keep
    let lines: Vec<&str> = content.lines().collect();
    let mut formatted_lines = Vec::new();
    let mut in_code_block = false;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            formatted_lines.push(line.to_string());
            continue;
        }

        if in_code_block {
            formatted_lines.push(line.to_string());
            continue;
        }

        if options.remove_horizontal_rules && is_horizontal_rule(trimmed) {
            continue;
        }

        let leading_indent = line.len() - line.trim_start().len();
        let indent_str = &line[..leading_indent];

        let (prefix, table_content) = if trimmed.starts_with('|') && trimmed.ends_with('|') {
            (indent_str, trimmed)
        } else if (trimmed.starts_with("> |") || trimmed.starts_with("- |"))
            && trimmed.ends_with('|')
        {
            let table_start = trimmed.find('|').unwrap_or(0);
            let prefix_part = &trimmed[..table_start];
            let table_part = &trimmed[table_start..];
            (&line[..leading_indent + prefix_part.len()], table_part)
        } else {
            ("", "")
        };

        let is_heading = trimmed.starts_with('#');

        if !table_content.is_empty() {
            let is_separator_row = table_content
                .chars()
                .all(|c| c == '|' || c == '-' || c == ' ' || c == ':')
                && table_content.contains('-');

            if !is_separator_row {
                let cells: Vec<&str> = table_content.split('|').collect();
                let mut formatted_cells = Vec::new();

                for (i, cell) in cells.iter().enumerate() {
                    if i == 0 || i == cells.len() - 1 {
                        continue;
                    }
                    let cell_trimmed = cell.trim();
                    formatted_cells.push(cell_trimmed.to_string());
                }

                let formatted_line = format!("{}| {} |", prefix, formatted_cells.join(" | "));
                formatted_lines.push(formatted_line);
            } else {
                formatted_lines.push(line.to_string());
            }
        } else {
            let mut processed_line = if options.remove_bold {
                remove_bold_markers(line)
            } else {
                line.to_string()
            };

            if options.remove_emphasis && !is_heading {
                processed_line = remove_emphasis_markers(&processed_line);
            }

            if options.collapse_spaces && !is_heading {
                processed_line = collapse_multiple_spaces(&processed_line);
            }

            if options.trim_trailing_whitespace {
                processed_line = processed_line.trim_end().to_string();
            }

            if options.compact_blank_lines && trimmed.is_empty() {
                if let Some(prev) = formatted_lines.last() {
                    if !prev.is_empty() {
                        // Check if next non-empty line is a heading - if so, preserve blank line
                        let next_is_heading = lines[i + 1..]
                            .iter()
                            .find(|l| !l.trim().is_empty())
                            .is_some_and(|l| l.trim().starts_with('#'));
                        if next_is_heading {
                            formatted_lines.push(String::new());
                            continue;
                        }

                        // Check if previous line was a heading - if so, preserve blank line
                        let prev_was_heading = prev.trim().starts_with('#');
                        if prev_was_heading {
                            formatted_lines.push(String::new());
                            continue;
                        }

                        let needs_line = formatted_lines
                            .iter()
                            .rev()
                            .nth(1)
                            .map(|l| !l.trim().is_empty())
                            .unwrap_or(true);
                        if needs_line || !formatted_lines.iter().rev().take(2).any(|l| l.is_empty())
                        {
                            formatted_lines.push(String::new());
                        }
                        continue;
                    }
                }
            }

            formatted_lines.push(processed_line);
        }
    }

    let mut result = formatted_lines.join("\n");

    if options.compact_blank_lines {
        let lines: Vec<&str> = result.lines().collect();
        let mut compact_lines = Vec::new();
        let mut prev_was_empty = false;

        for line in lines {
            let is_empty = line.trim().is_empty();
            if is_empty && prev_was_empty {
                continue;
            }
            compact_lines.push(line);
            prev_was_empty = is_empty;
        }
        result = compact_lines.join("\n");
    }

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

fn is_horizontal_rule(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed == "---" || trimmed == "***" || trimmed == "___"
}

fn remove_emphasis_markers(line: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '`' {
            let mut code_end = i + 1;
            while code_end < chars.len() {
                if chars[code_end] == '`' {
                    break;
                }
                code_end += 1;
            }
            for j in i..=code_end {
                if j < chars.len() {
                    result.push(chars[j]);
                }
            }
            i = code_end + 1;
            continue;
        }

        if chars[i] == '[' {
            let mut bracket_end = i;
            while bracket_end < chars.len() && chars[bracket_end] != ']' {
                bracket_end += 1;
            }
            for j in i..=bracket_end {
                if j < chars.len() {
                    result.push(chars[j]);
                }
            }
            i = bracket_end + 1;
            continue;
        }

        if i + 1 < chars.len()
            && ((chars[i] == '*' && chars[i + 1] != '*')
                || (chars[i] == '_' && chars[i + 1] != '_'))
        {
            let marker = chars[i];
            let mut j = i + 1;
            while j < chars.len() && chars[j] != marker {
                j += 1;
            }
            if j < chars.len() && chars[j] == marker {
                chars[i + 1..j].iter().for_each(|&c| result.push(c));
                i = j + 1;
            } else {
                result.push(chars[i]);
                i += 1;
            }
            continue;
        }

        result.push(chars[i]);
        i += 1;
    }

    result
}

fn collapse_multiple_spaces(line: &str) -> String {
    let mut result = String::new();
    let mut prev_was_space = false;

    for c in line.chars() {
        if c == ' ' {
            if !prev_was_space {
                result.push(c);
            }
            prev_was_space = true;
        } else {
            result.push(c);
            prev_was_space = false;
        }
    }

    result
}

pub fn cmd_fmt(path: &str, human: bool, options: FormatOptions) {
    match std::fs::read_to_string(path) {
        Ok(content) => {
            let formatted_content = format_markdown_with_options(&content, options);

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

    #[test]
    fn test_format_options_token_saver() {
        let content = "# Title\n\n\n\n## Section\n\n\nText with **bold**   \n\n- Item 1\n- Item 2\n\n\n## Another\n\n```\n**bold in code**\n```\n";
        let options = FormatOptions::token_saver();
        let result = format_markdown_with_options(content, options);

        assert!(!result.contains("**bold**"));
        assert!(result.contains("bold"));
        assert!(result.contains("```\n**bold in code**\n```"));
    }

    #[test]
    fn test_format_options_compact_blank_lines() {
        let content = "Line 1\n\n\n\nLine 2\n\n\n\nLine 3";
        let options = FormatOptions {
            remove_bold: false,
            compact_blank_lines: true,
            trim_trailing_whitespace: false,
            collapse_spaces: false,
            remove_horizontal_rules: false,
            remove_emphasis: false,
        };
        let result = format_markdown_with_options(content, options);
        assert_eq!(result, "Line 1\n\nLine 2\nLine 3");
    }

    #[test]
    fn test_format_options_remove_bold_disabled() {
        let content = "This is **bold** text";
        let options = FormatOptions {
            remove_bold: false,
            compact_blank_lines: false,
            trim_trailing_whitespace: false,
            collapse_spaces: false,
            remove_horizontal_rules: false,
            remove_emphasis: false,
        };
        let result = format_markdown_with_options(content, options);
        assert_eq!(result, "This is **bold** text");
    }

    #[test]
    fn test_format_options_trim_trailing_whitespace() {
        let content = "Text with trailing   \nMore text   ";
        let options = FormatOptions {
            remove_bold: false,
            compact_blank_lines: false,
            trim_trailing_whitespace: true,
            collapse_spaces: false,
            remove_horizontal_rules: false,
            remove_emphasis: false,
        };
        let result = format_markdown_with_options(content, options);
        assert_eq!(result, "Text with trailing\nMore text");
    }

    #[test]
    fn test_format_options_collapse_spaces() {
        let content = "This    has    multiple   spaces";
        let options = FormatOptions {
            remove_bold: false,
            compact_blank_lines: false,
            trim_trailing_whitespace: false,
            collapse_spaces: true,
            remove_horizontal_rules: false,
            remove_emphasis: false,
        };
        let result = format_markdown_with_options(content, options);
        assert_eq!(result, "This has multiple spaces");
    }

    #[test]
    fn test_format_options_collapse_spaces_in_heading_preserved() {
        let content = "# Heading   with    spaces";
        let options = FormatOptions {
            remove_bold: false,
            compact_blank_lines: false,
            trim_trailing_whitespace: false,
            collapse_spaces: true,
            remove_horizontal_rules: false,
            remove_emphasis: false,
        };
        let result = format_markdown_with_options(content, options);
        assert_eq!(result, "# Heading   with    spaces");
    }

    #[test]
    fn test_format_options_remove_horizontal_rules() {
        let content = "Before\n\n---\n\nAfter";
        let options = FormatOptions {
            remove_bold: false,
            compact_blank_lines: false,
            trim_trailing_whitespace: false,
            collapse_spaces: false,
            remove_horizontal_rules: true,
            remove_emphasis: false,
        };
        let result = format_markdown_with_options(content, options);
        assert!(!result.contains("---"));
        assert!(result.contains("Before"));
        assert!(result.contains("After"));
    }

    #[test]
    fn test_format_options_remove_horizontal_rules_variations() {
        let content = "Text\n\n***\n\nMore\n\n___\n\nEnd";
        let options = FormatOptions {
            remove_bold: false,
            compact_blank_lines: false,
            trim_trailing_whitespace: false,
            collapse_spaces: false,
            remove_horizontal_rules: true,
            remove_emphasis: false,
        };
        let result = format_markdown_with_options(content, options);
        assert!(!result.contains("***"));
        assert!(!result.contains("___"));
        assert!(result.contains("Text"));
        assert!(result.contains("More"));
        assert!(result.contains("End"));
    }

    #[test]
    fn test_format_options_remove_emphasis() {
        let content = "This is *italic* text";
        let options = FormatOptions {
            remove_bold: false,
            compact_blank_lines: false,
            trim_trailing_whitespace: false,
            collapse_spaces: false,
            remove_horizontal_rules: false,
            remove_emphasis: true,
        };
        let result = format_markdown_with_options(content, options);
        assert_eq!(result, "This is italic text");
    }

    #[test]
    fn test_format_options_remove_emphasis_underscore() {
        let content = "This is _italic_ text";
        let options = FormatOptions {
            remove_bold: false,
            compact_blank_lines: false,
            trim_trailing_whitespace: false,
            collapse_spaces: false,
            remove_horizontal_rules: false,
            remove_emphasis: true,
        };
        let result = format_markdown_with_options(content, options);
        assert_eq!(result, "This is italic text");
    }

    #[test]
    fn test_format_options_remove_emphasis_in_heading_preserved() {
        let content = "# *Heading* with emphasis";
        let options = FormatOptions {
            remove_bold: false,
            compact_blank_lines: false,
            trim_trailing_whitespace: false,
            collapse_spaces: false,
            remove_horizontal_rules: false,
            remove_emphasis: true,
        };
        let result = format_markdown_with_options(content, options);
        assert_eq!(result, "# *Heading* with emphasis");
    }

    #[test]
    fn test_format_options_remove_emphasis_in_code_preserved() {
        let content = "Use `*italic*` in code";
        let options = FormatOptions {
            remove_bold: false,
            compact_blank_lines: false,
            trim_trailing_whitespace: false,
            collapse_spaces: false,
            remove_horizontal_rules: false,
            remove_emphasis: true,
        };
        let result = format_markdown_with_options(content, options);
        assert_eq!(result, "Use `*italic*` in code");
    }

    #[test]
    fn test_format_options_token_saver_includes_new_rules() {
        let content = "# Title\n\nText  with   *emphasis*\n\n---\n\nMore  text";
        let options = FormatOptions::token_saver();
        let result = format_markdown_with_options(content, options);

        assert!(result.contains("# Title"));
        assert!(result.contains("Text with emphasis"));
        assert!(!result.contains("---"));
        assert!(result.contains("More text"));
    }
}
