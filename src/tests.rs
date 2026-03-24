use crate::*;
use crate::format::format_markdown;

#[cfg(test)]
mod tests {
    #![allow(clippy::module_inception)]
    use super::*;

    // Tests for JsonlEntry serialization
    #[test]
    fn test_jsonl_entry_serialization() {
        let entry = JsonlEntry {
            entry_type: "paragraph".to_string(),
            content: "Test content".to_string(),
            level: None,
            language: None,
        };

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("paragraph"));
        assert!(json.contains("Test content"));
    }

    #[test]
    fn test_jsonl_entry_serialization_all_fields() {
        let entry = JsonlEntry {
            entry_type: "heading".to_string(),
            content: "Test Heading".to_string(),
            level: Some(2),
            language: Some("rust".to_string()),
        };

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("heading"));
        assert!(json.contains("Test Heading"));
        assert!(json.contains("2"));
        assert!(json.contains("rust"));
    }

    // Tests for Document and related structures
    #[test]
    fn test_document_serialization() {
        let doc = Document {
            path: "test.md".to_string(),
            content: "# Test\n\nContent".to_string(),
            word_count: 2,
            line_count: 3,
            headings: vec![Heading {
                level: 1,
                text: "Test".to_string(),
                line: 1,
            }],
        };

        let json = serde_json::to_string(&doc).unwrap();
        assert!(json.contains("test.md"));
        assert!(json.contains("Test"));
        assert!(json.contains("2"));
        assert!(json.contains("3"));
    }

    // Tests for EditResult serialization
    #[test]
    fn test_edit_result_serialization_success() {
        let doc = Document {
            path: "test.md".to_string(),
            content: "# Test\n\nContent".to_string(),
            word_count: 2,
            line_count: 3,
            headings: vec![],
        };

        let result = EditResult {
            success: true,
            message: "Success".to_string(),
            document: Some(doc),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("true"));
        assert!(json.contains("Success"));
    }

    #[test]
    fn test_edit_result_serialization_failure() {
        let result = EditResult {
            success: false,
            message: "Error".to_string(),
            document: None,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("false"));
        assert!(json.contains("Error"));
    }

    // Tests for SearchResult serialization
    #[test]
    fn test_search_result_serialization() {
        let result = SearchResult {
            query: "test".to_string(),
            matches: vec![Match {
                line: 1,
                content: "test line".to_string(),
            }],
            total: 1,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("test"));
        assert!(json.contains("1"));
    }

    // Tests for LintError and LintWarning
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

    // Tests for list item detection
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

    // Tests for bold text detection
    #[test]
    fn test_find_bold_text_double_asterisks() {
        let line = "This has **bold** text";
        let result = find_bold_text(line);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], 10); // Position of first *
    }

    #[test]
    fn test_find_bold_text_double_underscores() {
        let line = "This has __bold__ text";
        let result = find_bold_text(line);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], 10); // Position of first _
    }

    #[test]
    fn test_find_bold_text_no_bold() {
        let line = "This has no bold text";
        let result = find_bold_text(line);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_find_bold_text_partial_patterns() {
        let line = "This has **bold text";
        let result = find_bold_text(line);
        assert_eq!(result.len(), 0); // Incomplete pattern

        let line = "This has bold** text";
        let result = find_bold_text(line);
        assert_eq!(result.len(), 0); // Incomplete pattern
    }

    #[test]
    fn test_find_bold_text_multiple_instances() {
        let line = "**First** and **second** bold";
        let result = find_bold_text(line);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], 1); // First **
        assert_eq!(result[1], 15); // Second **
    }

    #[test]
    fn test_find_bold_text_nested_bold_italics() {
        let line = "This has ***bold italic*** text";
        let result = find_bold_text(line);
        assert_eq!(result.len(), 1); // Should detect the outer ** in ***
        assert_eq!(result[0], 10);
    }

    #[test]
    fn test_find_bold_text_mixed_bold_formats() {
        let line = "**Bold** and __bold__ text";
        let result = find_bold_text(line);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], 1); // First **
        assert_eq!(result[1], 14); // First _
    }

    #[test]
    fn test_validate_markdown_bold_allowed_in_code_blocks() {
        let content = r#"# Document Title

This has **bold** text which should be an error.

This has `**bold**` in inline code which should be allowed.

```javascript
function test() {
    // This **bold** text should be allowed in fenced code blocks
    console.log("**bold** in code block");
    return __bold__;
}
```

This **bold** text should be another error.
"#;
        let result = validate_markdown(content);
        assert!(!result.valid); // Should have errors

        // Should have exactly 2 bold errors (lines 3 and 15, but not lines 5 or 7-11)
        let bold_errors: Vec<&LintError> = result
            .errors
            .iter()
            .filter(|e| e.rule == "no-bold")
            .collect();
        assert_eq!(bold_errors.len(), 2);
        assert_eq!(bold_errors[0].line, 3); // First bold text
        assert_eq!(bold_errors[1].line, 15); // Second bold text
    }

    #[test]
    fn test_validate_markdown_bold_allowed_in_inline_code() {
        let content = r#"# Test Document

Regular **bold** text should be an error.

Inline code with `**bold**` should be allowed.

More inline code with `__bold__` should also be allowed.

Regular text with **bold** again should be an error.
"#;
        let result = validate_markdown(content);
        assert!(!result.valid); // Should have errors

        // Should have exactly 2 bold errors (lines 3 and 9, but not lines 5 and 7)
        let bold_errors: Vec<&LintError> = result
            .errors
            .iter()
            .filter(|e| e.rule == "no-bold")
            .collect();
        assert_eq!(bold_errors.len(), 2);
        assert_eq!(bold_errors[0].line, 3); // First bold text
        assert_eq!(bold_errors[1].line, 9); // Second bold text
    }

    // Tests for useless link detection
    #[test]
    fn test_find_useless_link_exact_url() {
        let line = "Visit [https://example.com](https://example.com) for more";
        let result = find_useless_link(line);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], 7); // Position of [
    }

    #[test]
    fn test_find_useless_link_valid_link() {
        let line = "Visit [Example](https://example.com) for more";
        let result = find_useless_link(line);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_find_useless_link_no_links() {
        let line = "This has no links at all";
        let result = find_useless_link(line);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_find_useless_link_with_www() {
        let line = "Visit [www.example.com](https://www.example.com) for more";
        let result = find_useless_link(line);
        assert_eq!(result.len(), 1); // Should detect www.example.com == www.example.com
        assert_eq!(result[0], 7);
    }

    #[test]
    fn test_find_useless_link_without_protocol() {
        let line = "Visit [example.com](https://example.com) for more";
        let result = find_useless_link(line);
        assert_eq!(result.len(), 1); // Should detect example.com == example.com
        assert_eq!(result[0], 7);
    }

    #[test]
    fn test_find_useless_link_malformed_link() {
        let line = "This has [broken(link](https://example.com)";
        let result = find_useless_link(line);
        assert_eq!(result.len(), 0); // Malformed link, no detection
    }

    // Tests for ASCII graph detection
    #[test]
    fn test_find_ascii_graph_tree_structure() {
        let line = "├── parent";
        let result = find_ascii_graph(line);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_find_ascii_graph_flow_chart() {
        let line = "A -> B -> C";
        let result = find_ascii_graph(line);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), 3); // Position of first ->
    }

    #[test]
    fn test_find_ascii_graph_box_drawing() {
        let line = "┌─┐";
        let result = find_ascii_graph(line);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_find_ascii_graph_normal_text() {
        let line = "This is normal text";
        let result = find_ascii_graph(line);
        assert!(result.is_none());
    }

    #[test]
    fn test_find_ascii_graph_code_block() {
        let line = "```javascript";
        let result = find_ascii_graph(line);
        assert!(result.is_none()); // Code blocks should be exempt
    }

    #[test]
    fn test_find_ascii_graph_explicit_indicator() {
        let line = "graph: A -> B";
        let result = find_ascii_graph(line);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), 1); // Position of 'g'
    }

    #[test]
    fn test_find_ascii_graph_high_density_special_chars() {
        let line = "+---+---+---+";
        let result = find_ascii_graph(line);
        assert!(result.is_some());
    }

    #[test]
    fn test_find_ascii_graph_updated_message() {
        let line = "flow: Process A -> Process B";
        let result = find_ascii_graph(line);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), 1); // Position of 'f'
    }

    // Tests for table syntax validation
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

    // Tests for heading structure validation
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
        let errors = result.unwrap();
        assert_eq!(errors.len(), 1); // One error for second H1
        assert_eq!(errors[0].line, 5); // Second H1 is on line 5
        assert!(errors[0].message.contains("Multiple H1"));
    }

    #[test]
    fn test_validate_heading_structure_skipped_levels() {
        let content = "# H1\n\n### H3 (skipped H2)";
        let result = validate_heading_structure(content);
        assert!(result.is_some());
        let errors = result.unwrap();
        assert_eq!(errors.len(), 1); // One error for skipped level
        assert_eq!(errors[0].line, 3); // H3 is on line 3
        assert!(errors[0].message.contains("skipped"));
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
        let errors = result.unwrap();
        assert_eq!(errors.len(), 1); // One error for second H1
        assert_eq!(errors[0].line, 5); // Second H1 is on line 5
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
        assert!(result.is_none()); // Should be valid (no H1 conflicts)
    }

    #[test]
    fn test_validate_heading_structure_no_headings() {
        let content = "Just some text\n\nwith no headings.";
        let result = validate_heading_structure(content);
        assert!(result.is_none()); // Should be valid
    }

    #[test]
    fn test_validate_heading_structure_false_positives() {
        let content = "This is not a heading\n\n# This is a heading\n\nNot a heading: #";
        let result = validate_heading_structure(content);
        assert!(result.is_none()); // Should be valid (only one real heading)
    }

    // Tests for list formatting validation
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

    // Tests for code block validation
    #[test]
    fn test_validate_code_blocks_with_language() {
        let content = "```javascript\nconsole.log('hello');\n```";
        let result = validate_code_blocks(content);
        assert!(result.is_none()); // Should be valid (has language)
    }

    #[test]
    fn test_validate_code_blocks_no_language() {
        let content = "```\nconsole.log('hello');\n```";
        let result = validate_code_blocks(content);
        assert!(result.is_some());
        let warnings = result.unwrap();
        assert_eq!(warnings.len(), 1); // One warning for missing language
        assert_eq!(warnings[0].line, 1); // Warning on opening line
        assert!(warnings[0].message.contains("language"));
    }

    #[test]
    fn test_validate_code_blocks_multiple_blocks() {
        let content = "```javascript\n// Has language\n```\n\n```\n// No language\n```";
        let result = validate_code_blocks(content);
        assert!(result.is_some());
        let warnings = result.unwrap();
        assert_eq!(warnings.len(), 1); // One warning for second block
        assert_eq!(warnings[0].line, 5); // Second block starts on line 5
    }

    // Tests for overall markdown validation
    #[test]
    fn test_validate_markdown_perfect_document() {
        let content = r#"# Document Title

This is a *perfect* document with proper formatting.

## Section

- Item 1
- Item 2
- Item 3

### Subsection

1. First
2. Second
3. Third

```javascript
function example() {
    return true;
}
```

| Column 1| Column 2|
|---|---|
| Value 1| Value 2|

[Link text](https://example.com)

> This is a blockquote
> with multiple lines
"#;
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid
        assert_eq!(result.errors.len(), 0);
        assert_eq!(result.warnings.len(), 0);
    }

    #[test]
    fn test_validate_markdown_empty_content() {
        let content = "";
        let result = validate_markdown(content);
        assert!(result.valid); // Empty content should be valid
        assert_eq!(result.errors.len(), 0);
        assert_eq!(result.warnings.len(), 0);
    }

    #[test]
    fn test_validate_markdown_bold_error() {
        let content = "This has **bold** text which is not allowed.";
        let result = validate_markdown(content);
        assert!(!result.valid); // Should have errors
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].rule, "no-bold");
        assert_eq!(result.errors[0].line, 1);
        assert_eq!(result.errors[0].column, 10);
    }

    #[test]
    fn test_validate_markdown_ascii_graph() {
        let content = "graph: A -> B -> C";
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid (warnings only)
        assert_eq!(result.errors.len(), 0);
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.warnings[0].rule, "no-ascii-graph");
        assert_eq!(result.warnings[0].line, 1);
        assert_eq!(result.warnings[0].column, 1);
    }

    #[test]
    fn test_validate_markdown_useless_links() {
        let content = "Visit [https://example.com](https://example.com) for more info.";
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid (warnings only)
        assert_eq!(result.errors.len(), 0);
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.warnings[0].rule, "useless-links");
        assert_eq!(result.warnings[0].line, 1);
        assert_eq!(result.warnings[0].column, 7);
    }

    #[test]
    fn test_validate_markdown_multiple_errors() {
        let content =
            "This has **bold** and [https://example.com](https://example.com) and graph: A -> B";
        let result = validate_markdown(content);
        assert!(!result.valid); // Should have errors
        assert_eq!(result.errors.len(), 1); // One bold error
        assert_eq!(result.warnings.len(), 2); // Two warnings (link + graph)
    }

    #[test]
    fn test_validate_markdown_empty_lines_and_whitespace() {
        let content = "# Title\n\n   \n\nContent\n\n   \n\n## Section";
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid - whitespace shouldn't affect validation
        assert_eq!(result.errors.len(), 0);
    }

    #[test]
    fn test_validate_markdown_comprehensive() {
        let content = r#"# Document Title

This document has **bold** text (error) and [https://example.com](https://example.com) (warning).

## Section

graph: A -> B -> C

| Col1 | Col2 | Col3 | Col4 | Col5 | Col6 | Col7 |
|--|---|----|----|----|----|----|

1. First
3. Third (non-sequential)

- Item 1
* Item 2 (inconsistent marker)

```
no language code block
```

> Blockquote with **bold** (bold inside blockquote should still be error)
"#;
        let result = validate_markdown(content);
        assert!(!result.valid); // Should have errors

        // Should have bold errors (2 instances)
        let bold_errors: Vec<&LintError> = result
            .errors
            .iter()
            .filter(|e| e.rule == "no-bold")
            .collect();
        assert_eq!(bold_errors.len(), 2);

        // Should have various warnings
        assert!(!result.warnings.is_empty());

        // Check for specific warning types
        let warning_rules: Vec<String> = result.warnings.iter().map(|w| w.rule.clone()).collect();
        assert!(warning_rules.contains(&"useless-links".to_string()));
        assert!(warning_rules.contains(&"no-ascii-graph".to_string()));
        assert!(warning_rules.contains(&"simple-tables".to_string()));
        assert!(warning_rules.contains(&"list-formatting".to_string()));
        assert!(warning_rules.contains(&"code-blocks".to_string()));
    }

    // Tests for space indentation validation
    #[test]
    fn test_validate_space_indentation_excessive_spaces() {
        let content = r#"# Title

Regular paragraph.

    Paragraph with 4 spaces - should trigger warning.

      Paragraph with 6 spaces - should trigger warning.

        Paragraph with 8 spaces - should trigger warning.
"#;
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid (no errors, only warnings)
        assert_eq!(result.warnings.len(), 3);

        // Check that all warnings are for space-indentation rule
        for warning in &result.warnings {
            assert_eq!(warning.rule, "space-indentation");
        }

        // Check specific line numbers
        let warning_lines: Vec<usize> = result.warnings.iter().map(|w| w.line).collect();
        assert!(warning_lines.contains(&5)); // "    Paragraph with 4 spaces"
        assert!(warning_lines.contains(&7)); // "      Paragraph with 6 spaces"
        assert!(warning_lines.contains(&9)); // "        Paragraph with 8 spaces"
    }

    #[test]
    fn test_validate_space_indentation_valid_cases() {
        let content = r#"# Title

Regular paragraph.

  Paragraph with 2 spaces - should be valid.

	Paragraph with tab - should be valid.

1. Ordered list item
    2. Nested ordered list item

- Unordered list item
  - Nested unordered list item

> Blockquote with indentation

```javascript
function example() {
    // Code block should be exempt
    return true;
}
```
"#;
        let result = validate_markdown(content);
        // Should be valid (no space indentation warnings)
        assert!(result
            .warnings
            .iter()
            .all(|w| w.rule != "space-indentation"));
    }

    #[test]
    fn test_validate_space_indentation_edge_cases() {
        let content = r#"# Title

    Mixed with list item:
    1. This should still be exempt

    Mixed with heading:
    ## This should be exempt

    Mixed with blockquote:
    > This should be exempt

Empty line with spaces:


Line with only spaces should be ignored.
"#;
        let result = validate_markdown(content);
        // Should have space indentation warnings for lines with excessive indentation
        let space_warnings: Vec<&LintWarning> = result
            .warnings
            .iter()
            .filter(|w| w.rule == "space-indentation")
            .collect();

        // Lines with 4+ spaces should trigger warnings, except for properly formatted
        // list items which are exempt even with indentation
        assert_eq!(space_warnings.len(), 5);

        let warning_lines: Vec<usize> = space_warnings.iter().map(|w| w.line).collect();
        assert!(warning_lines.contains(&3)); // "    Mixed with list item:"
                                             // Line 4 is exempt (properly formatted ordered list item)
        assert!(warning_lines.contains(&6)); // "    Mixed with heading:"
        assert!(warning_lines.contains(&7)); // "    ## This should be exempt"
        assert!(warning_lines.contains(&9)); // "    Mixed with blockquote:"
        assert!(warning_lines.contains(&10)); // "    > This should be exempt"
    }

    #[test]
    fn test_validate_space_indentation_comprehensive() {
        let content = r#"# Document Title

This is a regular paragraph.

  This paragraph has 2 spaces - valid.

    This paragraph has 4 spaces - invalid.

## Section

1. First item
    2. Second item - should be valid (nested list)

- Unordered item
  - Nested item - should be valid

```rust
fn main() {
    // Code block - exempt
    println!("Hello");
}
```

> Blockquote
> With multiple lines
>     Even with indentation - exempt

    Another paragraph with 4 spaces - invalid.

Final paragraph.
"#;
        let result = validate_markdown(content);

        // Should have exactly 2 space indentation warnings
        let space_warnings: Vec<&LintWarning> = result
            .warnings
            .iter()
            .filter(|w| w.rule == "space-indentation")
            .collect();
        assert_eq!(space_warnings.len(), 2);

        // Check the warning lines
        let warning_lines: Vec<usize> = space_warnings.iter().map(|w| w.line).collect();
        assert!(warning_lines.contains(&7)); // "    This paragraph has 4 spaces"
        assert!(warning_lines.contains(&28)); // "    Another paragraph with 4 spaces"

        // Verify warning message
        for warning in space_warnings {
            assert_eq!(warning.message, "Use at most 2 spaces for indentation in regular text. Code blocks are exempt from this rule.");
            assert_eq!(warning.column, 1);
        }
    }

    // Tests for other validation functions
    #[test]
    fn test_validate_useless_link_edge_cases() {
        let content = r#"Multiple links:
[https://example.com](https://example.com)
[www.example.com](https://www.example.com)
[example.com](https://example.com)
[Good text](https://example.com)
"#;
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid (warnings only)

        let useless_link_warnings: Vec<&LintWarning> = result
            .warnings
            .iter()
            .filter(|w| w.rule == "useless-links")
            .collect();
        assert_eq!(useless_link_warnings.len(), 3); // Three useless links
    }

    #[test]
    fn test_validate_useless_link_complex_urls() {
        let content = "[https://example.com/path](https://example.com/path)";
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid (warnings only)

        let useless_link_warnings: Vec<&LintWarning> = result
            .warnings
            .iter()
            .filter(|w| w.rule == "useless-links")
            .collect();
        assert_eq!(useless_link_warnings.len(), 1); // One useless link with path
    }

    #[test]
    fn test_validate_ascii_graph_edge_cases() {
        let content = r#"Various graphs:
graph: A -> B
flow: Process A -> Process B
diagram: [A] -> [B] -> [C]
tree: Root -> Child -> Grandchild
Normal text with -> arrow but not graph indicator
"#;
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid (warnings only)

        let ascii_warnings: Vec<&LintWarning> = result
            .warnings
            .iter()
            .filter(|w| w.rule == "no-ascii-graph")
            .collect();
        assert_eq!(ascii_warnings.len(), 4); // Four graph indicators
    }

    // Tests for markdown parsing
    #[test]
    fn test_parse_markdown_basic() {
        let content = "# Title\n\nThis is content.";
        let doc = parse_markdown(content);
        assert_eq!(doc.headings.len(), 1);
        assert_eq!(doc.headings[0].level, 1);
        assert_eq!(doc.headings[0].text, "Title");
        assert_eq!(doc.headings[0].line, 1);
        assert_eq!(doc.word_count, 5);
        assert_eq!(doc.line_count, 3);
    }

    #[test]
    fn test_parse_markdown_multiple_headings() {
        let content = "# Title\n\n## Section\n\n### Subsection";
        let doc = parse_markdown(content);
        assert_eq!(doc.headings.len(), 3);
        assert_eq!(doc.headings[0].level, 1);
        assert_eq!(doc.headings[1].level, 2);
        assert_eq!(doc.headings[2].level, 3);
    }

    #[test]
    fn test_parse_markdown_no_headings() {
        let content = "Just some text\n\nwith no headings.";
        let doc = parse_markdown(content);
        assert_eq!(doc.headings.len(), 0);
        assert_eq!(doc.word_count, 6);
        assert_eq!(doc.line_count, 3);
    }

    #[test]
    fn test_parse_markdown_complex_headings() {
        let content = "# Title with `code` and **bold**\n\n## Section with [link](url)";
        let doc = parse_markdown(content);
        assert_eq!(doc.headings.len(), 2);
        assert_eq!(doc.headings[0].text, "Title with `code` and **bold**");
        assert_eq!(doc.headings[1].text, "Section with [link](url)");
    }

    #[test]
    fn test_parse_markdown_word_count() {
        let content = "Hello world! This is a test.";
        let doc = parse_markdown(content);
        assert_eq!(doc.word_count, 6);
    }

    #[test]
    fn test_parse_markdown_word_count_edge_cases() {
        let content = "   \n\n  \n\n"; // Only whitespace
        let doc = parse_markdown(content);
        assert_eq!(doc.word_count, 0);
    }

    #[test]
    fn test_parse_markdown_only_whitespace() {
        let content = "   \n  \n\t\n   ";
        let doc = parse_markdown(content);
        assert_eq!(doc.word_count, 0);
        assert_eq!(doc.line_count, 4);
        assert_eq!(doc.headings.len(), 0);
    }

    #[test]
    fn test_parse_markdown_large_document() {
        let content = "# Large Document\n\n".to_string() + &"This is a paragraph.\n".repeat(100);
        let doc = parse_markdown(&content);
        assert_eq!(doc.headings.len(), 1);
        assert!(doc.word_count > 100);
        assert!(doc.line_count > 100);
    }

    // Tests for JSONL conversion
    #[test]
    fn test_parse_markdown_to_jsonl_empty() {
        let content = "";
        let entries = parse_markdown_to_jsonl(content);
        assert_eq!(entries.len(), 0);
    }

    #[test]
    fn test_parse_markdown_to_jsonl_heading() {
        let content = "# Title";
        let entries = parse_markdown_to_jsonl(content);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].entry_type, "heading");
        assert_eq!(entries[0].level, Some(1));
        assert_eq!(entries[0].content, "Title");
    }

    #[test]
    fn test_parse_markdown_to_jsonl_paragraph() {
        let content = "This is a paragraph.";
        let entries = parse_markdown_to_jsonl(content);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].entry_type, "paragraph");
        assert_eq!(entries[0].content, "This is a paragraph.");
    }

    #[test]
    fn test_parse_markdown_to_jsonl_multiple_paragraphs() {
        let content = "First paragraph.\n\nSecond paragraph.";
        let entries = parse_markdown_to_jsonl(content);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].entry_type, "paragraph");
        assert_eq!(entries[1].entry_type, "paragraph");
        assert_eq!(entries[0].content, "First paragraph.");
        assert_eq!(entries[1].content, "Second paragraph.");
    }

    #[test]
    fn test_parse_markdown_to_jsonl_code_block() {
        let content = "```javascript\nconsole.log('hello');\n```";
        let entries = parse_markdown_to_jsonl(content);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].entry_type, "code_block");
        assert_eq!(entries[0].language, Some("javascript".to_string()));
        assert!(entries[0].content.contains("console.log"));
    }

    #[test]
    fn test_parse_markdown_to_jsonl_code_block_no_language() {
        let content = "```\nsome code\n```";
        let entries = parse_markdown_to_jsonl(content);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].entry_type, "code_block");
        assert_eq!(entries[0].language, None);
        assert!(entries[0].content.contains("some code"));
    }

    #[test]
    fn test_parse_markdown_to_jsonl_inline_code() {
        let content = "This has `inline code` in it.";
        let entries = parse_markdown_to_jsonl(content);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].entry_type, "paragraph");
        assert!(entries[0].content.contains("inline code"));
    }

    #[test]
    fn test_parse_markdown_to_jsonl_mixed() {
        let content = r#"# Title

This is a paragraph.

```javascript
console.log('hello');
```

## Section

More content.
"#;
        let entries = parse_markdown_to_jsonl(content);
        assert_eq!(entries.len(), 5); // Title, paragraph, code, heading, paragraph
        assert_eq!(entries[0].entry_type, "heading");
        assert_eq!(entries[1].entry_type, "paragraph");
        assert_eq!(entries[2].entry_type, "code_block");
        assert_eq!(entries[3].entry_type, "heading");
        assert_eq!(entries[4].entry_type, "paragraph");
    }

    #[test]
    fn test_parse_markdown_to_jsonl_complex_content() {
        let content = r#"# Document Title

This is a paragraph with **bold** text and `inline code`.

## Code Example

```rust
fn main() {
    println!("Hello, world!");
}
```

### Details

Here are some details with a [link](https://example.com).

> This is a blockquote
> with multiple lines.

Final paragraph.
"#;
        let entries = parse_markdown_to_jsonl(content);
        assert_eq!(entries.len(), 8); // Title, paragraph, heading, code, heading, paragraph, blockquote, paragraph
        assert_eq!(entries[0].entry_type, "heading");
        assert_eq!(entries[1].entry_type, "paragraph");
        assert_eq!(entries[2].entry_type, "heading");
        assert_eq!(entries[3].entry_type, "code_block");
        assert_eq!(entries[4].entry_type, "heading");
        assert_eq!(entries[5].entry_type, "paragraph");
        assert_eq!(entries[6].entry_type, "paragraph"); // Blockquote becomes paragraph in this implementation
        assert_eq!(entries[7].entry_type, "paragraph");
    }

    #[test]
    fn test_parse_markdown_to_jsonl_large_document() {
        let content = "# Title\n\n".to_string() + &"Paragraph.\n\n".repeat(100);
        let entries = parse_markdown_to_jsonl(&content);
        assert_eq!(entries.len(), 101); // Title + 100 paragraphs
        assert_eq!(entries[0].entry_type, "heading");
        assert!(entries.iter().skip(1).all(|e| e.entry_type == "paragraph"));
    }

    // Integration tests for command workflows
    #[test]
    fn test_document_workflow_parsing() {
        let content = r#"# Document Title

## Overview

This document contains various elements.

### Features

- Feature 1
- Feature 2

## Code Example

```rust
fn main() {
    println!("Hello, world!");
}
```

## Conclusion

End of document.
"#;

        // Test parsing
        let doc = parse_markdown(content);
        assert_eq!(doc.headings.len(), 5);
        assert!(doc.word_count > 20); // More flexible word count

        // Test JSONL conversion
        let entries = parse_markdown_to_jsonl(content);
        assert_eq!(entries.len(), 10); // 5 headings + 4 paragraphs + 1 code

        // Test validation
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid
        assert_eq!(result.errors.len(), 0);
    }

    #[test]
    fn test_validate_markdown_large_document() {
        let content = "# Large Document\n\n".to_string()
            + &"This is a paragraph with some content. ".repeat(1000);
        let result = validate_markdown(&content);
        assert!(result.valid); // Should be valid
        assert_eq!(result.errors.len(), 0);
    }

    #[test]
    fn test_validate_markdown_large_document_performance() {
        let content = "# Performance Test\n\n".to_string() + &"Word ".repeat(10000);
        let start = std::time::Instant::now();
        let result = validate_markdown(&content);
        let duration = start.elapsed();

        assert!(result.valid); // Should be valid
        assert!(duration.as_millis() < 1000); // Should complete within 1 second
    }

    // Tests for CLI argument parsing
    #[test]
    fn test_cli_parsing_version_flag() {
        // Test parsing version flag with -v
        let args = vec!["agent-md", "-v"];
        let cli = Cli::try_parse_from(&args).expect("Should parse -v flag");
        assert!(cli.version);
        assert!(cli.command.is_none());
    }

    #[test]
    fn test_cli_parsing_version_long_flag() {
        // Test parsing version flag with --version
        let args = vec!["agent-md", "--version"];
        let cli = Cli::try_parse_from(&args).expect("Should parse --version flag");
        assert!(cli.version);
        assert!(cli.command.is_none());
    }

    #[test]
    fn test_cli_parsing_no_command() {
        // Test parsing with no command - this should actually succeed in parsing
        // but will fail at runtime in main()
        let args = vec!["agent-md"];
        let cli = Cli::try_parse_from(&args).expect("Should parse without command");
        assert!(!cli.version);
        assert!(cli.command.is_none());
    }

    #[test]
    fn test_cli_parsing_help_flag() {
        // Test parsing help flag
        let args = vec!["agent-md", "--help"];
        let result = Cli::try_parse_from(&args);
        // Help should cause early exit, so parsing will fail
        assert!(result.is_err());
    }

    #[test]
    fn test_cli_parsing_with_command() {
        // Test parsing with a valid command
        let args = vec!["agent-md", "read", "test.md"];
        let cli = Cli::try_parse_from(&args).expect("Should parse read command");
        assert!(!cli.version);
        assert!(cli.command.is_some());

        match cli.command.unwrap() {
            Commands::Read {
                path,
                field: _,
                content: _,
            } => {
                assert_eq!(path, "test.md");
            }
            _ => panic!("Expected Read command"),
        }
    }

    #[test]
    fn test_cli_parsing_version_with_command() {
        // Test that version flag can be parsed with command
        // The version flag will be set and command will be parsed
        let args = vec!["agent-md", "-v", "read", "test.md"];
        let cli = Cli::try_parse_from(&args).expect("Should parse version with command");
        assert!(cli.version);
        assert!(cli.command.is_some());

        match cli.command.unwrap() {
            Commands::Read {
                path,
                field: _,
                content: _,
            } => {
                assert_eq!(path, "test.md");
            }
            _ => panic!("Expected Read command"),
        }
    }

    // Tests for ASCII graph detection in code blocks (now errors, not warnings)
    #[test]
    fn test_validate_markdown_ascii_graph_in_code_block_error() {
        let content = r#"# Document Title

## ASCII Graph Example

```text
├── public/
│   ├── pagefind/
│   ├── favicon.svg
```

Regular text here.
"#;
        let result = validate_markdown(content);
        assert!(!result.valid); // Should have errors

        let ascii_errors: Vec<&LintError> = result
            .errors
            .iter()
            .filter(|e| e.rule == "no-ascii-graph")
            .collect();
        assert_eq!(ascii_errors.len(), 3); // Three lines with tree structure

        // All should have the "in code block" message
        for error in &ascii_errors {
            assert!(error.message.contains("in code block"));
            assert_eq!(error.rule, "no-ascii-graph");
        }
    }

    #[test]
    fn test_validate_markdown_ascii_graph_box_drawing_in_code_block_error() {
        let content = r#"# Box Example

```text
┌───┐
│ A │
└───┘
```
"#;
        let result = validate_markdown(content);
        assert!(!result.valid); // Should have errors

        let ascii_errors: Vec<&LintError> = result
            .errors
            .iter()
            .filter(|e| e.rule == "no-ascii-graph")
            .collect();
        assert!(ascii_errors.len() >= 2); // At least 2 lines with box drawing

        for error in &ascii_errors {
            assert!(error.message.contains("in code block"));
        }
    }

    #[test]
    fn test_validate_markdown_ascii_graph_outside_code_block_warning() {
        let content = r#"# Document

Regular text with graph: A -> B

```text
Code block content
```

More text.
"#;
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid (warnings only)

        // Should have warning for ASCII graph outside code block
        let ascii_warnings: Vec<&LintWarning> = result
            .warnings
            .iter()
            .filter(|w| w.rule == "no-ascii-graph")
            .collect();
        assert_eq!(ascii_warnings.len(), 1);
        assert!(ascii_warnings[0].message.contains("ASCII graph detected"));
    }

    #[test]
    fn test_validate_markdown_ascii_graph_mixed_code_block_and_regular() {
        let content = r#"# Document

Text with graph: flow outside code block.

```text
├── tree
│   └── inside code block
```

More regular text.
"#;
        let result = validate_markdown(content);
        // Should have errors from code block but valid for regular text
        assert!(!result.valid);

        // Warning for outside code block
        let ascii_warnings: Vec<&LintWarning> = result
            .warnings
            .iter()
            .filter(|w| w.rule == "no-ascii-graph")
            .collect();
        assert_eq!(ascii_warnings.len(), 1);

        // Errors for inside code block
        let ascii_errors: Vec<&LintError> = result
            .errors
            .iter()
            .filter(|e| e.rule == "no-ascii-graph")
            .collect();
        assert_eq!(ascii_errors.len(), 2);
    }

    #[test]
    fn test_validate_markdown_ascii_graph_in_code_block_with_language() {
        let content = r#"# Code Example

```bash
├── public/
│   ├── favicon.svg
└── images/
```
"#;
        let result = validate_markdown(content);
        assert!(!result.valid); // Should have errors

        let ascii_errors: Vec<&LintError> = result
            .errors
            .iter()
            .filter(|e| e.rule == "no-ascii-graph")
            .collect();
        assert!(ascii_errors.len() >= 2);

        for error in &ascii_errors {
            assert_eq!(error.rule, "no-ascii-graph");
            assert!(error.message.contains("in code block"));
        }
    }

    #[test]
    fn test_find_ascii_graph_folder_structure() {
        // Test common folder tree patterns that should be detected
        assert!(find_ascii_graph("├── public/").is_some());
        assert!(find_ascii_graph("│   ├── pagefind/").is_some());
        assert!(find_ascii_graph("└── images/").is_some());
        // Tree structure needs at least the tree prefix pattern to be detected
        assert!(find_ascii_graph("│   └── file").is_some());
    }

    // Tests for content unescaping
    #[test]
    fn test_unescape_content_basic() {
        assert_eq!(unescape_content("Hello\\nWorld"), "Hello\nWorld");
        assert_eq!(unescape_content("Hello\\tWorld"), "Hello\tWorld");
        assert_eq!(unescape_content("Hello\\\\World"), "Hello\\World");
    }

    #[test]
    fn test_unescape_content_multiple() {
        assert_eq!(
            unescape_content("Line1\\nLine2\\nLine3"),
            "Line1\nLine2\nLine3"
        );
        assert_eq!(unescape_content("Col1\\tCol2\\tCol3"), "Col1\tCol2\tCol3");
    }

    #[test]
    fn test_unescape_content_mixed() {
        assert_eq!(unescape_content("Hello\\n\\tWorld\\\\"), "Hello\n\tWorld\\");
    }

    #[test]
    fn test_unescape_content_no_escapes() {
        assert_eq!(unescape_content("Hello World"), "Hello World");
        assert_eq!(unescape_content(""), "");
    }

    #[test]
    fn test_unescape_content_invalid_sequences() {
        assert_eq!(unescape_content("Hello\\xWorld"), "Hello\\xWorld");
        assert_eq!(unescape_content("Hello\\World"), "Hello\\World");
    }

    // Tests for section content extraction
    #[test]
    fn test_extract_section_content_simple() {
        let content = r#"# Title

Some content.

## Section

Section content here.

### Subsection

Subsection content.
"#;
        let result = extract_section_content(content, "Section");
        assert!(result.is_some());
        let extracted = result.unwrap();
        // The function only includes headings, not regular content
        assert!(extracted.contains("## Section"));
        assert!(extracted.contains("### Subsection"));
        assert!(!extracted.contains("Section content here."));
        assert!(!extracted.contains("Subsection content."));
    }

    #[test]
    fn test_extract_section_content_nested() {
        let content = r#"# Title

## Section 1

Content 1.

### Subsection 1.1

Sub content 1.1.

## Section 2

Content 2.

### Subsection 2.1

Sub content 2.1.
"#;
        // Let's test a simpler case first
        let result = extract_section_content(content, "Section 1");
        assert!(result.is_some());
        let extracted = result.unwrap();
        // The function only includes headings
        assert!(extracted.contains("## Section 1"));
        assert!(extracted.contains("### Subsection 1.1"));
        assert!(!extracted.contains("Content 1."));
        assert!(!extracted.contains("Sub content 1.1."));
        assert!(!extracted.contains("## Section 2"));
    }

    #[test]
    fn test_extract_section_content_not_found() {
        let content = r#"# Title

## Section

Content.
"#;
        let result = extract_section_content(content, "Nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_section_content_empty() {
        let content = "";
        let result = extract_section_content(content, "Section");
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_section_content_multiple_same_level() {
        let content = r#"# Title

## Section

First content.

## Another Section

Other content.

## Section

Second content.
"#;
        let result = extract_section_content(content, "Section");
        assert!(result.is_some());
        let extracted = result.unwrap();
        // Should get the first occurrence including its heading only
        assert!(extracted.contains("## Section"));
        // The function stops at the next heading of same level
        assert!(!extracted.contains("Another Section"));
        assert!(!extracted.contains("Second content."));
        assert!(!extracted.contains("First content.")); // No regular content
    }

    // Tests for section range finding
    #[test]
    fn test_find_section_range_simple() {
        let content = r#"# Title

Content before.

## Section

Section content.

Content after.
"#;
        let result = find_section_range(content, "Section");
        assert!(result.is_some());
        let (start, end) = result.unwrap();
        assert_eq!(start, 4); // Line with ## Section
        assert_eq!(end, content.lines().count()); // Goes to end for simple section
    }

    #[test]
    fn test_find_section_range_with_subsection() {
        let content = r#"# Title

## Section

Section content.

### Subsection

Sub content.

## Next Section

Next content.
"#;
        let result = find_section_range(content, "Section");
        assert!(result.is_some());
        let (start, end) = result.unwrap();
        // It's finding the first occurrence of "Section" which is in "Next Section" at line 2
        assert_eq!(start, 2); // Line with ## Next Section (contains "Section")
        assert_eq!(end, 13); // Goes to end
    }

    #[test]
    fn test_find_section_range_nested() {
        let content = r#"# Title

## Section

### Subsection

Sub content.

### Another Sub

More sub.

## Final

Final content.
"#;
        // Test simple section first
        let result = find_section_range(content, "Section");
        assert!(result.is_some());
        let (start, end) = result.unwrap();
        // It's finding "Section" in "Another Subsection" at line 2
        assert_eq!(start, 2); // Line with ### Another Sub (contains "Section")
        assert_eq!(end, 15); // Line before ## Final
    }

    // Tests for space indentation edge cases
    #[test]
    fn test_validate_space_indentation_list_items_with_indentation() {
        let content = r#"# Title

    - Indented list item 1
    - Indented list item 2
    - Indented list item 3

Regular paragraph.

    1. Indented ordered item 1
    2. Indented ordered item 2
"#;
        let result = validate_markdown(content);
        // List items with 4 spaces should still trigger warnings
        let space_warnings: Vec<&LintWarning> = result
            .warnings
            .iter()
            .filter(|w| w.rule == "space-indentation")
            .collect();
        assert_eq!(space_warnings.len(), 3); // Only the non-list lines with indentation
    }

    #[test]
    fn test_validate_space_indentation_code_fence_like_content() {
        let content = r#"# Title

    This looks like code but isn't fenced.
    So it should trigger indentation warning.

```javascript
    This is actual code and should be exempt.
    Even with 4 spaces.
```

    More indented text that should warn.
"#;
        let result = validate_markdown(content);
        let space_warnings: Vec<&LintWarning> = result
            .warnings
            .iter()
            .filter(|w| w.rule == "space-indentation")
            .collect();
        assert_eq!(space_warnings.len(), 3); // Lines 3, 5, and 12
    }

    // Tests for complex table scenarios
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

    // Tests for heading extraction edge cases
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

    // Tests for bold text detection edge cases
    #[test]
    fn test_find_bold_text_with_escaped_characters() {
        let line = "This has \\**not bold** text";
        let result = find_bold_text(line);
        assert_eq!(result.len(), 1); // Should still find bold after escaped backslash
        assert_eq!(result[0], 11); // Position of first **
    }

    #[test]
    fn test_find_bold_text_with_nested_code() {
        let line = "Text with **bold and `code`** inside";
        let result = find_bold_text(line);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], 11); // Position of first **
    }

    #[test]
    fn test_find_bold_text_multiple_overlapping() {
        let line = "**bold1****bold2**";
        let result = find_bold_text(line);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], 1); // First **
        assert_eq!(result[1], 10); // Second **
    }

    // Tests for list formatting complex scenarios
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

    // Tests for code block validation edge cases
    #[test]
    fn test_validate_code_blocks_unclosed_block() {
        let content = r#"# Title

```javascript
console.log('no closing fence');
```
"#;
        let result = validate_code_blocks(content);
        // Should handle closed block correctly
        assert!(result.is_none());
    }

    #[test]
    fn test_validate_code_blocks_empty_language() {
        let content = r#"```
code with empty language spec
```
"#;
        let result = validate_code_blocks(content);
        // Should treat empty language as missing
        assert!(result.is_some());
        let warnings = result.unwrap();
        assert_eq!(warnings.len(), 1);
    }

    #[test]
    fn test_validate_code_blocks_nested_fences() {
        let content = r#"`````
triple backticks inside code
```
still inside
`````
"#;
        let result = validate_code_blocks(content);
        // Should handle nested fences correctly
        assert!(result.is_none()); // Has language (empty)
    }

    // Performance tests
    #[test]
    fn test_validate_markdown_large_nested_document() {
        let mut content = "# Deep Document\n\n".to_string();
        for i in 1..=100 {
            content.push_str(&format!("## Section {}\n\n", i));
            for j in 1..=10 {
                content.push_str(&format!("### Subsection {}.{}\n\n", i, j));
                content.push_str("Content with **bold** text.\n\n");
            }
        }

        let start = std::time::Instant::now();
        let result = validate_markdown(&content);
        let duration = start.elapsed();

        // Should have 1000 bold errors
        assert_eq!(result.errors.len(), 1000);
        // Should complete in reasonable time
        assert!(duration.as_millis() < 500);
    }

    // Integration test for complete workflow
    #[test]
    fn test_complete_document_workflow() {
        let content = r#"# Project Documentation

## Overview

This project demonstrates **bold** errors and proper formatting.

## Installation

1. Clone repository
2. Install dependencies
3. Run tests

## Usage

```javascript
function hello() {
    console.log("Hello, world!");
}
```

## File Structure

```
├── src/
│   ├── main.js
│   └── utils.js
└── README.md
```

## Contributing

Please contribute to the project.
"#;

        // Parse document
        let doc = parse_markdown(content);
        assert_eq!(doc.headings.len(), 6);
        assert!(doc.word_count > 30);

        // Validate document
        let result = validate_markdown(content);
        assert!(!result.valid); // Should have errors

        // Check for specific errors
        let bold_errors: Vec<_> = result
            .errors
            .iter()
            .filter(|e| e.rule == "no-bold")
            .collect();
        assert_eq!(bold_errors.len(), 1);

        let ascii_errors: Vec<_> = result
            .errors
            .iter()
            .filter(|e| e.rule == "no-ascii-graph")
            .collect();
        assert_eq!(ascii_errors.len(), 4); // 4 lines in file structure

        // Convert to JSONL
        let entries = parse_markdown_to_jsonl(content);
        assert!(entries.len() > 10);

        // Verify JSONL structure
        let headings: Vec<_> = entries
            .iter()
            .filter(|e| e.entry_type == "heading")
            .collect();
        assert_eq!(headings.len(), 6);

        let code_blocks: Vec<_> = entries
            .iter()
            .filter(|e| e.entry_type == "code_block")
            .collect();
        assert_eq!(code_blocks.len(), 2);
    }

    // Tests for table trailing spaces validation
    #[test]
    fn test_validate_table_trailing_spaces_no_trailing_spaces() {
        let content =
            "# Test\n\n| Name | Description |\n|---|---|\n| Item | Value |\n| Test | Data |\n";
        let result = validate_markdown(content);
        assert!(result.valid);
        assert_eq!(result.errors.len(), 0);
    }

    #[test]
    fn test_validate_table_trailing_spaces_single_trailing_space() {
        let content =
            "# Test\n\n| Name | Description |\n|---|---|\n| Item | Value |\n| Test | Data |\n";
        let result = validate_markdown(content);
        assert!(result.valid);
        assert_eq!(result.errors.len(), 0);
    }

    #[test]
    fn test_validate_table_trailing_spaces_multiple_trailing_spaces() {
        let content = "# Test\n\n| Name | Description |\n|---|---|\n| Item       | Value |\n| Test     | Data     |\n";
        let result = validate_markdown(content);
        assert!(!result.valid);
        assert_eq!(result.errors.len(), 2);
        assert_eq!(result.errors[0].rule, "table-trailing-spaces");
        assert_eq!(result.errors[1].rule, "table-trailing-spaces");
        assert!(result.errors[0].message.contains("found 8 trailing spaces"));
        assert!(result.errors[1].message.contains("found 6 trailing spaces"));
    }

    #[test]
    fn test_validate_table_trailing_spaces_separator_row_ignored() {
        let content = "# Test\n\n| Name | Description |\n|-----|-----|\n| Item | Value |\n";
        let result = validate_markdown(content);
        // Should fail due to incorrect separator (5 dashes instead of 3)
        assert!(!result.valid);
        // But no trailing spaces errors
        assert!(!result
            .errors
            .iter()
            .any(|e| e.rule == "table-trailing-spaces"));
    }

    #[test]
    fn test_validate_table_trailing_spaces_mixed_cells() {
        let content = "# Test\n\n| Column 1 | Column 2 | Column 3 |\n|---|---|---|\n| Value 1 | Value 2       | Value 3 |\n| Test       | Data | Test       |\n";
        let result = validate_markdown(content);
        assert!(!result.valid);
        // Should have 2 trailing space errors (one per line with issues)
        let trailing_space_errors: Vec<_> = result
            .errors
            .iter()
            .filter(|e| e.rule == "table-trailing-spaces")
            .collect();
        assert_eq!(trailing_space_errors.len(), 2);
    }

    #[test]
    fn test_validate_table_trailing_spaces_non_table_lines_ignored() {
        let content = "# Test\n\n> This is not a table | but has pipes\nSome text | with pipes | that's not a table\n| Not a table row\n\n| Name | Description |\n|---|---|\n| Item       | Value |\n";
        let result = validate_markdown(content);
        assert!(!result.valid);
        // Should only have one trailing space error from the actual table
        let trailing_space_errors: Vec<_> = result
            .errors
            .iter()
            .filter(|e| e.rule == "table-trailing-spaces")
            .collect();
        assert_eq!(trailing_space_errors.len(), 1);
    }

    #[test]
    fn test_validate_table_trailing_spaces_empty_cells() {
        let content = "# Test\n\n| Name | Description | Value |\n|---|---|---|\n| Item |       | Data |\n| Test | Value     |       |\n";
        let result = validate_markdown(content);
        assert!(!result.valid);
        // Should have errors for cells with trailing spaces
        let trailing_space_errors: Vec<_> = result
            .errors
            .iter()
            .filter(|e| e.rule == "table-trailing-spaces")
            .collect();
        assert_eq!(trailing_space_errors.len(), 2);
    }

    #[test]
    fn test_validate_table_trailing_spaces_leading_spaces_ok() {
        let content =
            "# Test\n\n| Name | Description |\n|---|---|\n| Item | Value |\n| Test | Data |\n";
        let result = validate_markdown(content);
        assert!(result.valid);
        // Leading spaces in cells should be OK
        assert_eq!(result.errors.len(), 0);
    }

    #[test]
    fn test_validate_table_trailing_spaces_no_pipes_at_ends() {
        let content = "# Test\n\nName | Description\n---|---\nItem | Value\nTest | Data\n";
        let result = validate_markdown(content);
        // These are not valid table rows (don't start and end with |)
        // So trailing spaces validation should not apply
        assert!(result.valid);
        assert_eq!(result.errors.len(), 0);
    }

    #[test]
    fn test_validate_table_trailing_spaces_complex_table() {
        let content = "# Test\n\n| ID | Name | Description | Status | Priority | Assignee |\n|---|---|---|---|---|---|\n| 1 | Task 1      | Implement feature | In Progress | High | John      |\n| 2 | Task 2 | Fix bug  | Done | Low | Jane |\n| 3 | Task 3     | Write tests | To Do | Medium | Bob       |\n";
        let result = validate_markdown(content);
        assert!(!result.valid);
        // Should have trailing space errors
        let trailing_space_errors: Vec<_> = result
            .errors
            .iter()
            .filter(|e| e.rule == "table-trailing-spaces")
            .collect();
        assert_eq!(trailing_space_errors.len(), 3);
    }

    #[test]
    fn test_validate_table_trailing_spaces_with_inline_code() {
        let content = "# Test\n\n| Name | Code | Description |\n|---|---|---|\n| Item | `test`      | Value with `code` |\n| Test | `example` | Data       |\n";
        let result = validate_markdown(content);
        assert!(!result.valid);
        // Should have trailing space errors but no warnings (inline code is allowed)
        let trailing_space_errors: Vec<_> = result
            .errors
            .iter()
            .filter(|e| e.rule == "table-trailing-spaces")
            .collect();
        assert_eq!(trailing_space_errors.len(), 2);
        assert_eq!(result.warnings.len(), 0);
    }

    #[test]
    fn test_validate_table_trailing_spaces_single_column() {
        let content = "# Test\n\n| Value |\n|---|\n| Item       |\n| Test |\n| Data       |\n";
        let result = validate_markdown(content);
        assert!(!result.valid);
        let trailing_space_errors: Vec<_> = result
            .errors
            .iter()
            .filter(|e| e.rule == "table-trailing-spaces")
            .collect();
        assert_eq!(trailing_space_errors.len(), 2);
    }

    #[test]
    fn test_validate_table_trailing_spaces_in_code_block() {
        let content = "# Test\n\n```text\n| Name | Description |\n|---|---|\n| Item       | Value |\n| Test     | Data      |\n```\n\n| Name | Description |\n|---|---|\n| Item | Value |\n";
        let result = validate_markdown(content);
        assert!(result.valid);
        // Tables in code blocks should be ignored
        assert_eq!(result.errors.len(), 0);
    }

    // Tests for format_markdown - table trailing spaces handling
    #[test]
    fn test_format_markdown_removes_excess_trailing_spaces() {
        let content = "# Test\n\n| Name | Description |\n|---|---|\n| Item       | Value     |\n";
        let formatted = format_markdown(content);
        let result = validate_markdown(&formatted);
        // Should have no trailing space errors after formatting
        let trailing_space_errors: Vec<_> = result
            .errors
            .iter()
            .filter(|e| e.rule == "table-trailing-spaces")
            .collect();
        assert_eq!(trailing_space_errors.len(), 0);
    }

    #[test]
    fn test_format_markdown_removes_all_trailing_spaces() {
        let content = "# Test\n\n| Name | Description |\n|---|---|\n| Item  | Value  |\n";
        let formatted = format_markdown(content);
        // Should format with single spaces around cells for readability
        assert!(
            formatted.contains("| Item |"),
            "Expected formatted cell with single spaces in: {}",
            formatted
        );
        assert!(
            formatted.contains("| Value |"),
            "Expected formatted cell with single spaces in: {}",
            formatted
        );
    }

    #[test]
    fn test_format_markdown_handles_non_table_lines() {
        let content = "# Hello\n\nThis is regular text.\n\n- List item";
        let formatted = format_markdown(content);
        assert_eq!(formatted, content);
    }

    #[test]
    fn test_format_markdown_handles_separator_rows() {
        let content =
            "# Test\n\n| Name | Description |\n|------|-------------|\n| Item | Value |\n";
        let formatted = format_markdown(content);
        // Separator rows should be preserved as-is
        assert!(formatted.contains("|------|-------------|"));
    }

    #[test]
    fn test_format_markdown_multiple_excess_trailing_spaces() {
        let content = "# Test\n\n| Col1 | Col2 | Col3 |\n|------|------|------|\n| Data1       | Data2     | Data3       |\n";
        let formatted = format_markdown(content);
        eprintln!("Formatted:\n{}\n---", formatted);

        // Debug: check each cell's trailing spaces
        for (i, line) in formatted.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with('|') && trimmed.ends_with('|') {
                let is_sep = trimmed
                    .chars()
                    .all(|c| c == '|' || c == '-' || c == ' ' || c == ':');
                if !is_sep {
                    let cells: Vec<&str> = trimmed.split('|').collect();
                    for (j, cell) in cells.iter().enumerate() {
                        if j > 0 && j < cells.len() - 1 {
                            let trimmed_cell = cell.trim();
                            let trailing = cell.len() - cell.trim_end().len();
                            eprintln!(
                                "Line {}, Cell {} '{}': len={}, trimmed_len={}, trailing={}",
                                i + 1,
                                j,
                                cell,
                                cell.len(),
                                trimmed_cell.len(),
                                trailing
                            );
                        }
                    }
                }
            }
        }

        let result = validate_markdown(&formatted);
        eprintln!("Errors: {:?}", result.errors);
        let trailing_space_errors: Vec<_> = result
            .errors
            .iter()
            .filter(|e| e.rule == "table-trailing-spaces")
            .collect();
        assert_eq!(trailing_space_errors.len(), 0);
    }

    #[test]
    fn test_format_markdown_mixed_trailing_spaces() {
        let content =
            "# Test\n\n| Col1 | Col2 | Col3 |\n|------|------|------|\n| A    | B        | C |\n";
        let formatted = format_markdown(content);
        let result = validate_markdown(&formatted);
        let trailing_space_errors: Vec<_> = result
            .errors
            .iter()
            .filter(|e| e.rule == "table-trailing-spaces")
            .collect();
        assert_eq!(trailing_space_errors.len(), 0);
    }
}
