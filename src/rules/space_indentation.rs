pub fn validate_space_indentation(line: &str) -> Option<usize> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
        return None;
    }

    let line_without_leading_spaces = line.trim_start();
    if line_without_leading_spaces.len() >= 3 {
        let mut i = 0;
        let mut has_digits = false;
        while i < line_without_leading_spaces.len()
            && line_without_leading_spaces
                .chars()
                .nth(i)
                .unwrap()
                .is_ascii_digit()
        {
            has_digits = true;
            i += 1;
        }
        if has_digits && i < line_without_leading_spaces.len() {
            let separator = line_without_leading_spaces.chars().nth(i).unwrap();
            if (separator == '.' || separator == ')') && i + 1 < line_without_leading_spaces.len() {
                let next_char = line_without_leading_spaces.chars().nth(i + 1).unwrap();
                if next_char == ' ' {
                    return None;
                }
            }
        }
    }

    if trimmed.starts_with('#') {
        return None;
    }

    if trimmed.starts_with('>') {
        return None;
    }

    let leading_spaces = line.len() - line.trim_start().len();

    if leading_spaces > 2 && line.starts_with("   ") {
        let has_leading_spaces = line.chars().take(leading_spaces).all(|c| c == ' ');
        if has_leading_spaces {
            return Some(1);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_space_indentation_excessive_spaces() {
        let line = "    Paragraph with 4 spaces";
        let result = validate_space_indentation(line);
        assert!(result.is_some()); // Should trigger warning
        assert_eq!(result.unwrap(), 1);

        let line = "      Paragraph with 6 spaces";
        let result = validate_space_indentation(line);
        assert!(result.is_some()); // Should trigger warning

        let line = "        Paragraph with 8 spaces";
        let result = validate_space_indentation(line);
        assert!(result.is_some()); // Should trigger warning
    }

    #[test]
    fn test_validate_space_indentation_valid_cases() {
        let line = "Regular paragraph.";
        let result = validate_space_indentation(line);
        assert!(result.is_none()); // Should be valid

        let line = "  Paragraph with 2 spaces";
        let result = validate_space_indentation(line);
        assert!(result.is_none()); // Should be valid

        let line = "\tParagraph with tab";
        let result = validate_space_indentation(line);
        assert!(result.is_none()); // Should be valid

        let line = "1. Ordered list item";
        let result = validate_space_indentation(line);
        assert!(result.is_none()); // Should be valid

        let line = "- Unordered list item";
        let result = validate_space_indentation(line);
        assert!(result.is_none()); // Should be valid

        let line = "> Blockquote with indentation";
        let result = validate_space_indentation(line);
        assert!(result.is_none()); // Should be valid

        let line = "## Heading";
        let result = validate_space_indentation(line);
        assert!(result.is_none()); // Should be valid
    }

    #[test]
    fn test_validate_space_indentation_edge_cases() {
        let line = "    1. This should still be exempt";
        let result = validate_space_indentation(line);
        assert!(result.is_none()); // Numbered lists are exempt

        let line = "    ## This should be exempt";
        let result = validate_space_indentation(line);
        assert!(result.is_none()); // Headings are exempt

        let line = "    > This should be exempt";
        let result = validate_space_indentation(line);
        assert!(result.is_none()); // Blockquotes are exempt

        let line = "    ";
        let result = validate_space_indentation(line);
        assert!(result.is_none()); // Empty line with spaces should be ignored

        let line = "";
        let result = validate_space_indentation(line);
        assert!(result.is_none()); // Empty line should be ignored
    }

    #[test]
    fn test_validate_space_indentation_comprehensive() {
        let line = "This is a regular paragraph.";
        let result = validate_space_indentation(line);
        assert!(result.is_none()); // Should be valid

        let line = "  This paragraph has 2 spaces - valid.";
        let result = validate_space_indentation(line);
        assert!(result.is_none()); // Should be valid

        let line = "    This paragraph has 4 spaces - invalid.";
        let result = validate_space_indentation(line);
        assert!(result.is_some()); // Should be invalid

        let line = "## Section";
        let result = validate_space_indentation(line);
        assert!(result.is_none()); // Should be valid

        let line = "    2. Second item - should be valid (nested list)";
        let result = validate_space_indentation(line);
        assert!(result.is_none()); // Ordered lists are exempt

        let line = "  - Nested item - should be valid";
        let result = validate_space_indentation(line);
        assert!(result.is_none()); // Should be valid
    }

    #[test]
    fn test_validate_space_indentation_list_items_with_indentation() {
        let line = "    - Indented list item 1";
        let result = validate_space_indentation(line);
        assert!(result.is_none()); // List items are exempt

        let line = "    1. Indented ordered item 1";
        let result = validate_space_indentation(line);
        assert!(result.is_none()); // Ordered lists are exempt
    }

    #[test]
    fn test_validate_space_indentation_code_fence_like_content() {
        let line = "    This looks like code but isn't fenced.";
        let result = validate_space_indentation(line);
        assert!(result.is_some()); // Should trigger indentation warning

        let line = "```javascript";
        let result = validate_space_indentation(line);
        assert!(result.is_none()); // Code fences should be exempt
    }

    #[test]
    fn test_validate_space_indentation_mixed_whitespace() {
        let line = "   \tParagraph with mixed spaces and tab";
        let result = validate_space_indentation(line);
        assert!(result.is_none()); // Mixed whitespace should be valid

        let line = "    \tParagraph starting with spaces then tab";
        let result = validate_space_indentation(line);
        assert!(result.is_none()); // Mixed should be valid
    }

    #[test]
    fn test_validate_space_indentation_ordered_list_edge_cases() {
        let line = "    123. Ordered list with three digits";
        let result = validate_space_indentation(line);
        assert!(result.is_none()); // Ordered lists are exempt

        let line = "    1) Ordered list with parenthesis";
        let result = validate_space_indentation(line);
        assert!(result.is_none()); // Ordered lists are exempt

        let line = "    1.No space after number";
        let result = validate_space_indentation(line);
        assert!(result.is_some()); // Should warn (not a valid list item)
    }

    #[test]
    fn test_validate_space_indentation_unordered_list_variations() {
        let line = "    + Plus list item";
        let result = validate_space_indentation(line);
        assert!(result.is_none()); // List items are exempt

        let line = "    * Asterisk list item";
        let result = validate_space_indentation(line);
        assert!(result.is_none()); // List items are exempt

        let line = "    -No space after dash";
        let result = validate_space_indentation(line);
        assert!(result.is_some()); // Should warn (not a valid list item)
    }

    #[test]
    fn test_validate_space_indentation_single_space() {
        let line = " Single space";
        let result = validate_space_indentation(line);
        assert!(result.is_none()); // Should be valid

        let line = "  Two spaces";
        let result = validate_space_indentation(line);
        assert!(result.is_none()); // Should be valid
    }

    #[test]
    fn test_validate_space_indentation_three_spaces() {
        let line = "   Three spaces";
        let result = validate_space_indentation(line);
        assert!(result.is_some()); // Should warn
    }

    #[test]
    fn test_validate_space_indentation_heading_variations() {
        let line = "    # Heading with spaces";
        let result = validate_space_indentation(line);
        assert!(result.is_none()); // Headings are exempt

        let line = "    ###### H6 with spaces";
        let result = validate_space_indentation(line);
        assert!(result.is_none()); // Headings are exempt

        let line = "    #No space after hash";
        let result = validate_space_indentation(line);
        assert!(result.is_none()); // Still starts with #, so exempt
    }
}
