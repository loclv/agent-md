use clap::{Parser, Subcommand};
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Parser as MarkdownParser, Tag, TagEnd};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JsonlEntry {
    #[serde(rename = "type")]
    pub entry_type: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Document {
    pub path: String,
    pub content: String,
    pub word_count: usize,
    pub line_count: usize,
    pub headings: Vec<Heading>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Heading {
    pub level: u32,
    pub text: String,
    pub line: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EditResult {
    pub success: bool,
    pub message: String,
    pub document: Option<Document>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub query: String,
    pub matches: Vec<Match>,
    pub total: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Match {
    pub line: usize,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LintResult {
    pub valid: bool,
    pub errors: Vec<LintError>,
    pub warnings: Vec<LintWarning>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct LintError {
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub rule: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LintWarning {
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub rule: String,
}

fn validate_markdown(content: &str) -> LintResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    let mut in_code_block = false;

    for (line_num, line) in content.lines().enumerate() {
        let line_num = line_num + 1; // Convert to 1-based indexing

        // Track code block boundaries
        if line.trim().starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }

        // Skip all checks inside code blocks
        if in_code_block {
            continue;
        }

        // Rule: No bold text (detect **text** or __text__)
        for col in find_bold_text(line) {
            errors.push(LintError {
                line: line_num,
                column: col,
                message: "Bold text is not allowed for AI agents".to_string(),
                rule: "no-bold".to_string(),
            });
        }

        // Rule: Simple table syntax validation
        for issue in validate_table_syntax(line) {
            match issue.severity {
                Severity::Error => errors.push(LintError {
                    line: line_num,
                    column: issue.column,
                    message: issue.message,
                    rule: "simple-tables".to_string(),
                }),
                Severity::Warning => warnings.push(LintWarning {
                    line: line_num,
                    column: issue.column,
                    message: issue.message,
                    rule: "simple-tables".to_string(),
                }),
            }
        }

        // Rule: No useless links where text equals URL
        for col in find_useless_link(line) {
            warnings.push(LintWarning {
                line: line_num,
                column: col,
                message:
                    "Link text should not be the same as the URL - provide meaningful link text"
                        .to_string(),
                rule: "useless-links".to_string(),
            });
        }

        // Rule: No ASCII Graphs - Human-readable ASCII graphs should be replaced with LLM-readable formats
        if let Some(col) = find_ascii_graph(line) {
            warnings.push(LintWarning {
                line: line_num,
                column: col,
                message: "Human-readable ASCII graph detected. Use LLM-readable formats instead: Structured CSV, JSON, Mermaid Diagram, Numbered List with Conditions, ZON format, or simple progress indicators".to_string(),
                rule: "no-ascii-graph".to_string(),
            });
        }

        // Rule: Limited Space Indentation
        if let Some(col) = validate_space_indentation(line) {
            warnings.push(LintWarning {
                line: line_num,
                column: col,
                message: "Use at most 2 spaces for indentation in regular text. Code blocks are exempt from this rule.".to_string(),
                rule: "space-indentation".to_string(),
            });
        }
    }

    // Rule: Proper heading structure validation
    if let Some(heading_errors) = validate_heading_structure(content) {
        errors.extend(heading_errors);
    }

    // Rule: Code block best practices validation
    if let Some(code_block_issues) = validate_code_blocks(content) {
        // Convert code block errors to warnings since they're suggestions
        for issue in code_block_issues {
            warnings.push(LintWarning {
                line: issue.line,
                column: issue.column,
                message: issue.message,
                rule: issue.rule,
            });
        }
    }

    // Rule: List formatting validation
    if let Some(list_issues) = validate_list_formatting(content) {
        // Convert list errors to warnings since they're formatting suggestions
        for issue in list_issues {
            warnings.push(LintWarning {
                line: issue.line,
                column: issue.column,
                message: issue.message,
                rule: issue.rule,
            });
        }
    }

    LintResult {
        valid: errors.is_empty(),
        errors,
        warnings,
    }
}

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

pub fn find_bold_text(line: &str) -> Vec<usize> {
    let mut results = Vec::new();

    // Find all inline code blocks and exclude them
    let mut code_ranges = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let mut in_code = false;
    let mut code_start = 0;

    // Find all inline code blocks
    for (i, &ch) in chars.iter().enumerate() {
        if ch == '`' && (i == 0 || chars[i - 1] != '\\') {
            if !in_code {
                in_code = true;
                code_start = i;
            } else {
                in_code = false;
                code_ranges.push((code_start, i)); // inclusive range
            }
        }
    }

    // If unclosed code block, treat from start to end as code
    if in_code {
        code_ranges.push((code_start, line.len() - 1));
    }

    // Check for **bold** pattern, skipping code ranges
    let mut search_start = 0;
    while search_start < line.len() {
        if let Some(start) = line[search_start..].find("**") {
            let abs_start = search_start + start;

            // Check if this bold is inside any code range
            let in_code_range = code_ranges
                .iter()
                .any(|&(start, end)| abs_start >= start && abs_start <= end);

            if !in_code_range {
                if let Some(end_offset) = line[abs_start + 2..].find("**") {
                    let abs_end = abs_start + 2 + end_offset;

                    // Check if the end is also outside code ranges
                    let end_in_code_range = code_ranges
                        .iter()
                        .any(|&(start, end)| abs_end >= start && abs_end <= end);

                    if !end_in_code_range {
                        results.push(abs_start + 1); // Return 1-based column
                        search_start = abs_end + 2;
                        continue;
                    }
                }
            }
            search_start = abs_start + 2;
        } else {
            break;
        }
    }

    // Check for __bold__ pattern, skipping code ranges
    search_start = 0;
    while search_start < line.len() {
        if let Some(start) = line[search_start..].find("__") {
            let abs_start = search_start + start;

            // Check if this bold is inside any code range
            let in_code_range = code_ranges
                .iter()
                .any(|&(start, end)| abs_start >= start && abs_start <= end);

            if !in_code_range {
                if let Some(end_offset) = line[abs_start + 2..].find("__") {
                    let abs_end = abs_start + 2 + end_offset;

                    // Check if the end is also outside code ranges
                    let end_in_code_range = code_ranges
                        .iter()
                        .any(|&(start, end)| abs_end >= start && abs_end <= end);

                    if !end_in_code_range {
                        results.push(abs_start + 1); // Return 1-based column
                        search_start = abs_end + 2;
                        continue;
                    }
                }
            }
            search_start = abs_start + 2;
        } else {
            break;
        }
    }

    results
}

pub fn find_useless_link(line: &str) -> Vec<usize> {
    let mut results = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '[' {
            let bracket_start = i;
            // Find the closing bracket
            let mut bracket_end = i + 1;
            let mut bracket_content = String::new();
            let mut found_closing_bracket = false;

            while bracket_end < chars.len() {
                let ch = chars[bracket_end];
                if ch == ']' {
                    found_closing_bracket = true;
                    break;
                }
                bracket_content.push(ch);
                bracket_end += 1;
            }

            if found_closing_bracket && bracket_end + 1 < chars.len() {
                // Check for opening parenthesis
                if chars[bracket_end + 1] == '(' {
                    let mut paren_start = bracket_end + 2;
                    let mut url = String::new();
                    let mut found_closing_paren = false;
                    let mut paren_depth = 1;

                    while paren_start < chars.len() {
                        let ch = chars[paren_start];
                        if ch == '(' {
                            paren_depth += 1;
                            url.push(ch);
                        } else if ch == ')' {
                            paren_depth -= 1;
                            if paren_depth == 0 {
                                found_closing_paren = true;
                                break;
                            }
                            url.push(ch);
                        } else {
                            url.push(ch);
                        }
                        paren_start += 1;
                    }

                    if found_closing_paren {
                        // Check if link text is the same as URL
                        let link_text = bracket_content.trim();
                        let url_trimmed = url.trim();

                        // Remove common URL prefixes for comparison
                        let url_without_protocol = url_trimmed
                            .trim_start_matches("http://")
                            .trim_start_matches("https://");

                        // Remove trailing slash before removing www.
                        let url_without_slash = url_without_protocol.trim_end_matches('/');

                        // Remove www. prefix for comparison
                        let url_without_www = url_without_slash.trim_start_matches("www.");

                        // Check if link text matches the URL in any form
                        if link_text == url_trimmed
                            || link_text == url_without_protocol
                            || link_text == url_without_slash
                            || link_text == url_without_www
                        {
                            results.push(bracket_start + 1); // Return 1-based column
                        }
                        i = paren_start + 1;
                        continue;
                    }
                }
            }
        }
        i += 1;
    }

    results
}

pub fn find_ascii_graph(line: &str) -> Option<usize> {
    // Skip table separator lines (lines with only |, -, :, and spaces)
    let trimmed = line.trim();
    let is_table_separator = trimmed
        .chars()
        .all(|c| c == '|' || c == '-' || c == ' ' || c == ':');
    if is_table_separator {
        return None;
    }

    // Common ASCII graph patterns to detect
    let ascii_graph_patterns = [
        // Box drawing characters (more specific)
        "┌─┐",
        "└─┘",
        "├─┤",
        "│ │",
        "├──",
        "└──",
        "│  ",
        // Tree structures (more specific)
        "├── ",
        "└── ",
        "│   ",
        // Simple graph patterns
        "->",
        "<-",
        "<->",
        "==",
        "=>",
        "<=",
        // Progress bars (more specific)
        "[",
        "]",
        // Flow indicators
        "flow:",
        "Flow:",
        "FLOW:",
        "diagram:",
        "Diagram:",
        "DIAGRAM:",
        "chart:",
        "Chart:",
        "CHART:",
        "graph:",
        "Graph:",
        "GRAPH:",
        "tree:",
        "Tree:",
        "TREE:",
        // Common graph elements
        "+---+",
        "+---",
        "---+",
        "|   |",
    ];

    // Check for common ASCII graph indicators
    let graph_indicators = ["graph:", "chart:", "diagram:", "flow:", "tree:"];

    // First check for explicit graph indicators
    let line_lower = line.to_lowercase();
    for indicator in &graph_indicators {
        if let Some(pos) = line_lower.find(indicator) {
            return Some(pos + 1); // Return 1-based column
        }
    }

    // Then check for ASCII graph patterns
    for pattern in &ascii_graph_patterns {
        if let Some(pos) = line.find(pattern) {
            // Additional heuristic: if the line contains multiple such patterns,
            // it's more likely to be an ASCII graph
            let pattern_count = line.matches(pattern).count();
            if pattern_count >= 2
                || line.contains("┌")
                || line.contains("└")
                || line.contains("├")
                || line.contains("┤")
            {
                return Some(pos + 1); // Return 1-based column
            }

            // For single patterns, be more selective
            if *pattern == "├──"
                || *pattern == "└──"
                || *pattern == "│  "
                || *pattern == "┌─┐"
                || *pattern == "└─┘"
                || *pattern == "├─┤"
            {
                // Tree/box patterns are strong indicators
                return Some(pos + 1);
            } else if *pattern == "[ ]"
                || *pattern == "( )"
                || *pattern == "{ }"
                || *pattern == "->"
                || *pattern == "<-"
            {
                // Check if there are multiple such patterns or other graph indicators
                if line.matches("->").count() + line.matches("<-").count() >= 2
                    || line.matches("[ ]").count() + line.matches("( )").count() >= 2
                {
                    return Some(pos + 1);
                }
            } else {
                // For other patterns, check if the line is mostly special characters
                let special_chars = line
                    .chars()
                    .filter(|c| !c.is_alphabetic() && !c.is_whitespace() && *c != '.')
                    .count();

                let total_chars = line.chars().filter(|c| !c.is_whitespace()).count();

                // If more than 40% of non-whitespace characters are special characters,
                // and the line has at least 5 such characters, it's likely an ASCII graph
                if total_chars >= 5 && special_chars as f64 / total_chars as f64 > 0.4 {
                    return Some(pos + 1);
                }
            }
        }
    }

    // Check for lines that look like ASCII art/graphs (high density of special chars)
    let special_chars = line
        .chars()
        .filter(|c| !c.is_alphabetic() && !c.is_whitespace() && *c != '.')
        .count();

    let total_chars = line.chars().filter(|c| !c.is_whitespace()).count();

    // If more than 40% of non-whitespace characters are special characters,
    // and the line has at least 5 such characters, it's likely an ASCII graph
    if total_chars >= 5 && special_chars as f64 / total_chars as f64 > 0.4 {
        // Find the first special character position
        for (i, c) in line.chars().enumerate() {
            if !c.is_alphabetic() && !c.is_whitespace() && c != '.' {
                return Some(i + 1); // Return 1-based column
            }
        }
    }

    None
}

pub fn validate_table_syntax(line: &str) -> Vec<TableIssue> {
    let mut issues = Vec::new();
    let trimmed = line.trim();

    // Check if this looks like a table row
    if trimmed.contains('|') {
        // Check for separator rows (rows with only pipes and dashes)
        let is_separator_row = trimmed
            .chars()
            .all(|c| c == '|' || c == '-' || c == ' ' || c == ':');

        if is_separator_row {
            // Check for exactly 3 dash separators between pipes
            let parts: Vec<&str> = trimmed.split('|').collect();
            for part in parts {
                let part_trimmed = part.trim();
                if !part_trimmed.is_empty() {
                    // Count dashes in this separator part
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

        // Check for complex table syntax that should be avoided
        if trimmed.contains("colspan") || trimmed.contains("rowspan") {
            issues.push(TableIssue {
                column: 1,
                message: "Complex table attributes (colspan/rowspan) are not allowed".to_string(),
                severity: Severity::Error,
            });
            return issues;
        }

        // Check for inline formatting in table cells
        if trimmed.contains("**") || trimmed.contains("__") || trimmed.contains("*") {
            issues.push(TableIssue {
                column: 1,
                message: "inline formatting in table cells should be avoided".to_string(),
                severity: Severity::Warning,
            });
            return issues;
        }

        // Warn about very complex tables
        let pipe_count = trimmed.matches('|').count();
        if pipe_count > 6 {
            // More than 5 columns
            issues.push(TableIssue {
                column: 1,
                message: "Very wide tables should be simplified".to_string(),
                severity: Severity::Warning,
            });
        }
    }

    issues
}

pub fn validate_heading_structure(content: &str) -> Option<Vec<LintError>> {
    let mut heading_levels = Vec::new();
    let mut h1_count = 0;
    let mut h1_locations = Vec::new();
    let mut in_code_block = false;

    for (line_num, line) in content.lines().enumerate() {
        let line_num = line_num + 1; // Convert to 1-based indexing

        // Track code block boundaries
        if line.trim().starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }

        // Skip headings inside code blocks
        if in_code_block {
            continue;
        }

        // Check for heading levels (lines starting with #)
        if let Some(level) = extract_heading_level(line) {
            heading_levels.push((level, line_num));

            if level == 1 {
                h1_count += 1;
                h1_locations.push(line_num);
            }
        }
    }

    let mut errors = Vec::new();

    // Check for multiple H1 headings
    if h1_count > 1 {
        for &location in &h1_locations[1..] {
            // Skip the first H1
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

    // Check for skipped heading levels
    if heading_levels.len() > 1 {
        for i in 1..heading_levels.len() {
            let (current_level, current_line) = heading_levels[i];
            let (prev_level, _) = heading_levels[i - 1];

            // Check if we skipped more than one level
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
        let line_num = line_num + 1; // Convert to 1-based indexing

        // Track code block boundaries
        if line.trim().starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }

        // Skip list items inside code blocks
        if in_code_block {
            continue;
        }

        let trimmed = line.trim();

        // Check for list items (ordered or unordered)
        if let Some(list_info) = detect_list_item(trimmed) {
            list_items.push((list_info, line_num));
        }
    }

    // Check for inconsistent list formatting
    if list_items.len() > 1 {
        let mut current_list_type: Option<ListType> = None;
        let mut _current_list_marker: Option<String> = None;
        let mut expected_next_number: Option<u32> = None;

        for (i, ((list_type, marker), line_num)) in list_items.iter().enumerate() {
            // Skip if this is the first item or if there's a blank line between lists
            if i > 0 {
                let prev_line_num = list_items[i - 1].1;
                if *line_num > prev_line_num + 1 {
                    // New list, reset tracking
                    current_list_type = None;
                    _current_list_marker = None;
                    expected_next_number = None;
                }
            }

            // Update current list tracking first
            if current_list_type.is_none() {
                current_list_type = Some(*list_type);
                _current_list_marker = Some(marker.to_string());

                // For ordered lists, set up expected next number
                if *list_type == ListType::Ordered {
                    if let Some(current_num) = extract_number_from_marker(marker) {
                        expected_next_number = Some(current_num + 1);
                    }
                }
            } else {
                // Check for consistency within the same list
                if current_list_type != Some(*list_type) {
                    errors.push(LintError {
                        line: *line_num,
                        column: 1,
                        message: "inconsistent list formatting detected. Use consistent list markers within the same list".to_string(),
                        rule: "list-formatting".to_string(),
                    });
                    break; // Stop after first inconsistency
                } else if *list_type == ListType::Unordered
                    && _current_list_marker.as_ref() != Some(marker)
                {
                    // For unordered lists, also check for consistent markers
                    errors.push(LintError {
                        line: *line_num,
                        column: 1,
                        message: "inconsistent list formatting detected. Use consistent list markers within the same list".to_string(),
                        rule: "list-formatting".to_string(),
                    });
                    break; // Stop after first inconsistency
                }

                // For ordered lists, check for sequential numbering
                if *list_type == ListType::Ordered {
                    // Check if current marker matches expected sequence
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

                    // Update tracking for next iteration
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
    // Find the separator (. or ))
    let mut separator_pos = None;
    for (i, c) in marker.chars().enumerate() {
        if c == '.' || c == ')' {
            separator_pos = Some(i);
            break;
        }
    }

    if let Some(pos) = separator_pos {
        if pos == 0 {
            return None; // Separator at start, no digits
        }

        let num_part = &marker[..pos];
        // Check if all characters before separator are digits
        if num_part.chars().all(|c| c.is_ascii_digit()) {
            num_part.parse::<u32>().ok()
        } else {
            None
        }
    } else {
        None // No separator found
    }
}

pub fn detect_list_item(line: &str) -> Option<(ListType, String)> {
    // Check for unordered lists: -, *, or + followed by space
    if line.len() >= 2 {
        let first_char = line.chars().next().unwrap();
        let second_char = line.chars().nth(1).unwrap();

        if (first_char == '-' || first_char == '*' || first_char == '+') && second_char == ' ' {
            return Some((ListType::Unordered, first_char.to_string()));
        }
    }

    // Check for ordered lists: number followed by . or ) and space
    if line.len() >= 3 {
        let mut i = 0;
        let mut has_digits = false;

        // Extract digits at the start
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

pub fn validate_code_blocks(content: &str) -> Option<Vec<LintError>> {
    let mut errors = Vec::new();
    let mut in_code_block = false;
    let mut code_block_start_line = 0;

    for (line_num, line) in content.lines().enumerate() {
        let line_num = line_num + 1; // Convert to 1-based indexing

        // Check for code block start
        if line.trim().starts_with("```") {
            if !in_code_block {
                // Starting a new code block
                in_code_block = true;
                code_block_start_line = line_num;

                // Check if language is specified
                let trimmed = line.trim();
                let has_language = if trimmed.len() > 3 {
                    // Has content after the backticks
                    let lang_part = &trimmed[3..];
                    !lang_part.trim().is_empty()
                } else {
                    false
                };

                // Store the language info for this code block
                if !has_language {
                    // We'll check at the end if this was never closed properly
                }
            } else {
                // Ending code block
                in_code_block = false;

                // Check if this code block didn't have a language
                let trimmed = line.trim();
                let _has_language = if trimmed.len() > 3 {
                    let lang_part = &trimmed[3..];
                    !lang_part.trim().is_empty()
                } else {
                    false
                };

                // Actually, we need to check the start line, not the end line
                // Let's get the content from the start line
                if let Some(start_line_content) = content.lines().nth(code_block_start_line - 1) {
                    let start_trimmed = start_line_content.trim();
                    let start_has_language = if start_trimmed.len() > 3 {
                        let lang_part = &start_trimmed[3..];
                        !lang_part.trim().is_empty()
                    } else {
                        false
                    };

                    if !start_has_language {
                        errors.push(LintError {
                            line: code_block_start_line,
                            column: 1,
                            message: "Code block should specify language for better parsing. Use 'text' if no specific language applies".to_string(),
                            rule: "code-blocks".to_string(),
                        });
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

pub fn validate_space_indentation(line: &str) -> Option<usize> {
    // Check for leading spaces in regular text (not code blocks or list items)
    let trimmed = line.trim_end();
    if trimmed.is_empty() {
        return None;
    }

    // Skip if this looks like a list item (including nested ones)
    if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
        return None;
    }

    // Skip if this looks like an ordered list item (including nested ones)
    // Check both at start and after indentation
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

    // Skip if this looks like a heading
    if trimmed.starts_with('#') {
        return None;
    }

    // Skip if this looks like a blockquote
    if trimmed.starts_with('>') {
        return None;
    }

    // Count leading spaces
    let leading_spaces = line.len() - line.trim_start().len();

    // Only check spaces, not tabs
    if leading_spaces > 2 && line.starts_with("   ") {
        // Check if the line starts with spaces (not tabs)
        let has_leading_spaces = line.chars().take(leading_spaces).all(|c| c == ' ');
        if has_leading_spaces {
            return Some(1); // Return column 1 for the indentation issue
        }
    }

    None
}

fn extract_heading_level(line: &str) -> Option<u32> {
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

        // Only count as heading if followed by space
        if matches!(trimmed.chars().nth(level as usize), Some(' ') | Some('\t')) {
            Some(level)
        } else {
            None
        }
    } else {
        None
    }
}

pub fn parse_markdown(content: &str) -> Document {
    let word_count = content.split_whitespace().count();
    let line_count = content.lines().count();
    let mut headings = Vec::new();

    let parser = MarkdownParser::new(content);
    let mut in_heading = false;
    let mut current_level = 0;
    let mut current_heading_offset = 0;

    for (event, range) in parser.into_offset_iter() {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                in_heading = true;
                current_level = level as u32;
                current_heading_offset = range.start;
            }
            Event::End(TagEnd::Heading(_)) => {
                if in_heading {
                    // Extract the raw heading text from the original content
                    let heading_start = current_heading_offset;
                    let heading_end = range.end;
                    let heading_text = content[heading_start..heading_end].trim();

                    // Remove the leading # characters and whitespace
                    let heading_text = heading_text
                        .chars()
                        .skip_while(|c| *c == '#')
                        .skip_while(|c| c.is_whitespace())
                        .collect();

                    let line_num = content[..current_heading_offset]
                        .chars()
                        .filter(|&c| c == '\n')
                        .count()
                        + 1;
                    headings.push(Heading {
                        level: current_level,
                        text: heading_text,
                        line: line_num,
                    });
                }
                in_heading = false;
            }
            _ => {}
        }
    }

    Document {
        path: String::new(),
        content: content.to_string(),
        word_count,
        line_count,
        headings,
    }
}

#[derive(Parser)]
#[command(name = "agent-md")]
#[command(about = "Markdown editor for AI agents", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Read {
        #[arg(help = "Markdown file path")]
        path: String,
        #[arg(
            help = "Extract specific field (path, content, word_count, line_count, headings)",
            long,
            short = 'f'
        )]
        field: Option<String>,
        #[arg(
            help = "Extract specific section content by heading name",
            long,
            short = 'c'
        )]
        content: Option<String>,
    },
    Write {
        #[arg(help = "Markdown file path")]
        path: String,
        #[arg(help = "Content to write")]
        content: String,
    },
    WriteSection {
        #[arg(help = "Markdown file path")]
        path: String,
        #[arg(help = "Section heading path (e.g., '## Development' or '## Development > Build')")]
        section: String,
        #[arg(help = "Content to write to the section")]
        content: String,
    },
    Append {
        #[arg(help = "Markdown file path")]
        path: String,
        #[arg(help = "Content to append")]
        content: String,
    },
    Insert {
        #[arg(help = "Markdown file path")]
        path: String,
        #[arg(help = "Line number to insert at")]
        line: usize,
        #[arg(help = "Content to insert")]
        content: String,
    },
    Delete {
        #[arg(help = "Markdown file path")]
        path: String,
        #[arg(help = "Line number to delete")]
        line: usize,
        #[arg(help = "Number of lines to delete", default_value = "1")]
        count: usize,
    },
    List {
        #[arg(help = "Directory to list", default_value = ".")]
        path: String,
    },
    Search {
        #[arg(help = "Markdown file path")]
        path: String,
        #[arg(help = "Search query")]
        query: String,
    },
    Headings {
        #[arg(help = "Markdown file path")]
        path: String,
    },
    Stats {
        #[arg(help = "Markdown file path")]
        path: String,
    },
    ToJsonl {
        #[arg(help = "Markdown file path")]
        path: String,
    },
    Lint {
        #[arg(help = "Markdown file path or content to validate")]
        path: String,
        #[arg(
            help = "Validate content directly instead of file",
            long,
            default_value = "false"
        )]
        content: bool,
    },
    LintFile {
        #[arg(help = "Markdown file path to lint")]
        path: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Read {
            path,
            field,
            content,
        } => cmd_read(&path, field.as_deref(), content.as_deref()),
        Commands::Write { path, content } => cmd_write(&path, &content),
        Commands::WriteSection {
            path,
            section,
            content,
        } => cmd_write_section(&path, &section, &content),
        Commands::Append { path, content } => cmd_append(&path, &content),
        Commands::Insert {
            path,
            line,
            content,
        } => cmd_insert(&path, line, &content),
        Commands::Delete { path, line, count } => cmd_delete(&path, line, count),
        Commands::List { path } => cmd_list(&path),
        Commands::Search { path, query } => cmd_search(&path, &query),
        Commands::Headings { path } => cmd_headings(&path),
        Commands::Stats { path } => cmd_stats(&path),
        Commands::ToJsonl { path } => cmd_to_jsonl(&path),
        Commands::Lint { path, content } => cmd_lint(&path, content),
        Commands::LintFile { path } => cmd_lint_file(&path),
    }
}

fn extract_section_content(content: &str, section_path: &str) -> Option<String> {
    let path_parts: Vec<&str> = section_path.split('>').map(|s| s.trim()).collect();
    if path_parts.is_empty() {
        return None;
    }

    let lines: Vec<&str> = content.lines().collect();
    let mut section_content = Vec::new();
    let mut in_target_section = false;
    let mut target_level = 0;
    let mut found_section = false;
    let mut current_depth = 0;

    for line in lines.iter() {
        if let Some(level) = extract_heading_level(line) {
            let heading_text = line.trim_start_matches('#').trim();

            if !in_target_section {
                if heading_text == path_parts[0] {
                    if path_parts.len() == 1 {
                        in_target_section = true;
                        target_level = level;
                        found_section = true;
                        section_content.push(*line);
                        continue;
                    } else {
                        current_depth = 1;
                        in_target_section = true;
                        target_level = level;
                        found_section = true;
                        section_content.push(*line);
                        continue;
                    }
                }
            } else if heading_text == path_parts[current_depth] {
                if level <= target_level {
                    break;
                }
                current_depth += 1;
                if current_depth >= path_parts.len() {
                    section_content.push(*line);
                } else {
                    continue;
                }
            } else if level <= target_level {
                break;
            }

            if in_target_section && current_depth < path_parts.len() && level > target_level {
                section_content.push(*line);
            }
        } else if in_target_section && current_depth >= path_parts.len() {
            section_content.push(*line);
        }
    }

    if found_section {
        Some(section_content.join("\n"))
    } else {
        None
    }
}

fn find_section_range(content: &str, section_path: &str) -> Option<(usize, usize)> {
    let path_parts: Vec<&str> = section_path.split('>').map(|s| s.trim()).collect();
    if path_parts.is_empty() {
        return None;
    }

    let lines: Vec<&str> = content.lines().collect();
    let mut in_target_section = false;
    let mut target_level = 0;
    let mut start_line = 0;
    let mut current_depth = 0;

    for (i, line) in lines.iter().enumerate() {
        if let Some(level) = extract_heading_level(line) {
            let heading_text = line.trim_start_matches('#').trim();

            if !in_target_section {
                if heading_text == path_parts[0] {
                    if path_parts.len() == 1 {
                        start_line = i;
                        return Some((start_line, lines.len()));
                    } else {
                        current_depth = 1;
                        in_target_section = true;
                        target_level = level;
                        start_line = i;
                        continue;
                    }
                }
            } else if heading_text == path_parts[current_depth] {
                if level <= target_level {
                    return Some((start_line, i));
                }
                current_depth += 1;
                if current_depth >= path_parts.len() {
                    let end_line = find_section_end(&lines, i + 1, level);
                    return Some((start_line, end_line));
                }
            } else if level <= target_level {
                return Some((start_line, i));
            }
        }
    }

    if in_target_section {
        Some((start_line, lines.len()))
    } else {
        None
    }
}

fn find_section_end(lines: &[&str], start: usize, parent_level: u32) -> usize {
    for (i, line) in lines.iter().enumerate().skip(start) {
        if let Some(level) = extract_heading_level(line) {
            if level <= parent_level {
                return i;
            }
        }
    }
    lines.len()
}

fn unescape_content(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.peek() {
                Some('n') => {
                    result.push('\n');
                    chars.next();
                }
                Some('t') => {
                    result.push('\t');
                    chars.next();
                }
                Some('\\') => {
                    result.push('\\');
                    chars.next();
                }
                _ => result.push(ch),
            }
        } else {
            result.push(ch);
        }
    }
    result
}

fn cmd_read(path: &str, field: Option<&str>, content_filter: Option<&str>) {
    match fs::read_to_string(path) {
        Ok(content) => {
            // Handle content filtering first
            let filtered_content = if let Some(section_name) = content_filter {
                match extract_section_content(&content, section_name) {
                    Some(section) => section,
                    None => {
                        println!(
                            "{}",
                            serde_json::to_string(&EditResult {
                                success: false,
                                message: format!("Section '{}' not found", section_name),
                                document: None
                            })
                            .unwrap()
                        );
                        return;
                    }
                }
            } else {
                content
            };

            let mut doc = parse_markdown(&filtered_content);
            doc.path = path.to_string();

            let output = if let Some(field_name) = field {
                match field_name {
                    "path" => serde_json::to_string(&doc.path).unwrap(),
                    "content" => serde_json::to_string(&doc.content).unwrap(),
                    "word_count" => serde_json::to_string(&doc.word_count).unwrap(),
                    "line_count" => serde_json::to_string(&doc.line_count).unwrap(),
                    "headings" => serde_json::to_string(&doc.headings).unwrap(),
                    _ => {
                        eprintln!("Error: Invalid field '{}'. Valid fields: path, content, word_count, line_count, headings", field_name);
                        std::process::exit(1);
                    }
                }
            } else {
                serde_json::to_string(&doc).unwrap()
            };

            println!("{}", output);
        }
        Err(e) => {
            println!(
                "{}",
                serde_json::to_string(&EditResult {
                    success: false,
                    message: format!("Failed to read file: {}", e),
                    document: None
                })
                .unwrap()
            );
        }
    }
}

fn cmd_write(path: &str, content: &str) {
    let content = unescape_content(content);
    // Validate content before writing
    let validation = validate_markdown(&content);

    if !validation.valid {
        println!(
            "{}",
            serde_json::to_string(&EditResult {
                success: false,
                message: format!(
                    "Content validation failed: {} errors found",
                    validation.errors.len()
                ),
                document: None,
            })
            .unwrap()
        );
        // Also output validation details
        println!("{}", serde_json::to_string(&validation).unwrap());
        return;
    }

    match fs::write(path, &content) {
        Ok(_) => {
            let mut doc = parse_markdown(&content);
            doc.path = path.to_string();
            println!(
                "{}",
                serde_json::to_string(&EditResult {
                    success: true,
                    message: "File written successfully".to_string(),
                    document: Some(doc),
                })
                .unwrap()
            );
        }
        Err(e) => {
            println!(
                "{}",
                serde_json::to_string(&EditResult {
                    success: false,
                    message: format!("Failed to write file: {}", e),
                    document: None
                })
                .unwrap()
            );
        }
    }
}

fn cmd_write_section(path: &str, section_path: &str, new_content: &str) {
    let new_content = unescape_content(new_content);
    let validation = validate_markdown(&new_content);

    if !validation.valid {
        println!(
            "{}",
            serde_json::to_string(&EditResult {
                success: false,
                message: format!(
                    "Content validation failed: {} errors found",
                    validation.errors.len()
                ),
                document: None,
            })
            .unwrap()
        );
        println!("{}", serde_json::to_string(&validation).unwrap());
        return;
    }

    match fs::read_to_string(path) {
        Ok(existing) => {
            let result = if let Some((start, end)) = find_section_range(&existing, section_path) {
                replace_section_content(&existing, start, end, section_path, &new_content)
            } else {
                insert_section_content(&existing, section_path, &new_content)
            };

            match result {
                Ok(updated) => match fs::write(path, &updated) {
                    Ok(_) => {
                        let mut doc = parse_markdown(&updated);
                        doc.path = path.to_string();
                        println!(
                            "{}",
                            serde_json::to_string(&EditResult {
                                success: true,
                                message: format!("Section '{}' written successfully", section_path),
                                document: Some(doc),
                            })
                            .unwrap()
                        );
                    }
                    Err(e) => {
                        println!(
                            "{}",
                            serde_json::to_string(&EditResult {
                                success: false,
                                message: format!("Failed to write file: {}", e),
                                document: None,
                            })
                            .unwrap()
                        );
                    }
                },
                Err(e) => {
                    println!(
                        "{}",
                        serde_json::to_string(&EditResult {
                            success: false,
                            message: e,
                            document: None,
                        })
                        .unwrap()
                    );
                }
            }
        }
        Err(e) => {
            println!(
                "{}",
                serde_json::to_string(&EditResult {
                    success: false,
                    message: format!("Failed to read file: {}", e),
                    document: None,
                })
                .unwrap()
            );
        }
    }
}

fn replace_section_content(
    content: &str,
    start: usize,
    end: usize,
    section_path: &str,
    new_content: &str,
) -> Result<String, String> {
    let lines: Vec<&str> = content.lines().collect();
    let path_parts: Vec<&str> = section_path.split('>').map(|s| s.trim()).collect();
    let target_heading = path_parts.last().unwrap();
    let heading_level =
        extract_heading_level(target_heading.trim_start_matches('#').trim()).unwrap_or(2);
    let hashes = "#".repeat(heading_level as usize);
    let new_section = format!(
        "{} {}\n{}",
        hashes,
        target_heading.trim_start_matches('#').trim(),
        new_content
    );

    let mut result_lines: Vec<String> = lines.iter().take(start).map(|s| s.to_string()).collect();
    result_lines.push(new_section);
    result_lines.extend(lines.iter().skip(end).map(|s| s.to_string()));

    Ok(result_lines.join("\n"))
}

fn insert_section_content(
    content: &str,
    section_path: &str,
    new_content: &str,
) -> Result<String, String> {
    let path_parts: Vec<&str> = section_path.split('>').map(|s| s.trim()).collect();

    if path_parts.len() == 1 {
        let target_heading = path_parts[0];
        let heading_level = extract_heading_level(target_heading).unwrap_or(2);
        let hashes = "#".repeat(heading_level as usize);
        let new_section = format!(
            "{} {}\n{}",
            hashes,
            target_heading.trim_start_matches('#').trim(),
            new_content
        );

        let mut lines: Vec<&str> = content.lines().collect();
        if !content.ends_with('\n') && !lines.is_empty() {
            let last_idx = lines.len() - 1;
            if !lines[last_idx].is_empty() {
                lines[last_idx] = &content[content.len()..];
            }
        }
        if !content.is_empty() && !content.ends_with('\n') {
            return Err("File must end with newline before inserting section".to_string());
        }
        let mut result = content.to_string();
        result.push_str(&new_section);
        result.push('\n');
        return Ok(result);
    }

    let top_level_heading = path_parts[0];
    let mut lines: Vec<&str> = content.lines().collect();
    let mut insert_pos = lines.len();
    let mut in_parent = false;
    let mut parent_level = 0;

    for (i, line) in lines.iter().enumerate() {
        if let Some(level) = extract_heading_level(line) {
            let heading_text = line.trim_start_matches('#').trim();
            if heading_text == top_level_heading {
                in_parent = true;
                parent_level = level;
                insert_pos = i + 1;
                continue;
            }
            if in_parent && level <= parent_level {
                insert_pos = i;
                break;
            }
        }
    }

    let mut current_level = 1;
    let mut section_text = String::new();
    for part in &path_parts {
        let heading_text = part.trim_start_matches('#').trim();
        let level = extract_heading_level(part).unwrap_or(current_level);
        let hashes = "#".repeat(level as usize);
        section_text.push_str(&format!("{} {}\n", hashes, heading_text));
        current_level = level + 1;
    }
    section_text.push_str(new_content);
    section_text.push('\n');

    lines.insert(insert_pos, "");
    let mut result: Vec<String> = lines
        .iter()
        .take(insert_pos)
        .map(|s| s.to_string())
        .collect();
    result.push(section_text);
    result.extend(lines.iter().skip(insert_pos + 1).map(|s| s.to_string()));

    Ok(result.join("\n"))
}

fn cmd_append(path: &str, content: &str) {
    let content = unescape_content(content);
    match fs::read_to_string(path) {
        Ok(mut existing) => {
            if !existing.ends_with('\n') {
                existing.push('\n');
            }
            existing.push_str(&content);
            match fs::write(path, &existing) {
                Ok(_) => {
                    let mut doc = parse_markdown(&existing);
                    doc.path = path.to_string();
                    println!(
                        "{}",
                        serde_json::to_string(&EditResult {
                            success: true,
                            message: "Content appended successfully".to_string(),
                            document: Some(doc),
                        })
                        .unwrap()
                    );
                }
                Err(e) => {
                    println!(
                        "{}",
                        serde_json::to_string(&EditResult {
                            success: false,
                            message: format!("Failed to write file: {}", e),
                            document: None
                        })
                        .unwrap()
                    );
                }
            }
        }
        Err(e) => {
            println!(
                "{}",
                serde_json::to_string(&EditResult {
                    success: false,
                    message: format!("Failed to read file: {}", e),
                    document: None
                })
                .unwrap()
            );
        }
    }
}

fn cmd_insert(path: &str, line: usize, content: &str) {
    let content = unescape_content(content);
    match fs::read_to_string(path) {
        Ok(existing) => {
            let mut lines: Vec<String> = existing.lines().map(|s| s.to_string()).collect();
            let insert_at = line.saturating_sub(1).min(lines.len());
            let new_lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
            lines.splice(insert_at..insert_at, new_lines);
            let result = lines.join("\n");
            match fs::write(path, &result) {
                Ok(_) => {
                    let mut doc = parse_markdown(&result);
                    doc.path = path.to_string();
                    println!(
                        "{}",
                        serde_json::to_string(&EditResult {
                            success: true,
                            message: format!("Inserted at line {}", line),
                            document: Some(doc),
                        })
                        .unwrap()
                    );
                }
                Err(e) => {
                    println!(
                        "{}",
                        serde_json::to_string(&EditResult {
                            success: false,
                            message: format!("Failed to write file: {}", e),
                            document: None
                        })
                        .unwrap()
                    );
                }
            }
        }
        Err(e) => {
            println!(
                "{}",
                serde_json::to_string(&EditResult {
                    success: false,
                    message: format!("Failed to read file: {}", e),
                    document: None
                })
                .unwrap()
            );
        }
    }
}

fn cmd_delete(path: &str, line: usize, count: usize) {
    match fs::read_to_string(path) {
        Ok(existing) => {
            let mut lines: Vec<String> = existing.lines().map(|s| s.to_string()).collect();
            let delete_at = line.saturating_sub(1).min(lines.len());
            let delete_end = (delete_at + count).min(lines.len());
            lines.splice(delete_at..delete_end, std::iter::empty());
            let result = lines.join("\n");
            match fs::write(path, &result) {
                Ok(_) => {
                    let mut doc = parse_markdown(&result);
                    doc.path = path.to_string();
                    println!(
                        "{}",
                        serde_json::to_string(&EditResult {
                            success: true,
                            message: format!("Deleted {} lines from line {}", count, line),
                            document: Some(doc),
                        })
                        .unwrap()
                    );
                }
                Err(e) => {
                    println!(
                        "{}",
                        serde_json::to_string(&EditResult {
                            success: false,
                            message: format!("Failed to write file: {}", e),
                            document: None
                        })
                        .unwrap()
                    );
                }
            }
        }
        Err(e) => {
            println!(
                "{}",
                serde_json::to_string(&EditResult {
                    success: false,
                    message: format!("Failed to read file: {}", e),
                    document: None
                })
                .unwrap()
            );
        }
    }
}

fn cmd_list(path: &str) {
    let path = PathBuf::from(path);
    match fs::read_dir(&path) {
        Ok(entries) => {
            let mut files: Vec<String> = Vec::new();
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "md" || ext == "markdown" {
                        files.push(entry.path().to_string_lossy().to_string());
                    }
                }
            }
            files.sort();
            println!("{}", serde_json::to_string(&files).unwrap());
        }
        Err(e) => {
            println!(
                "{}",
                serde_json::to_string(&EditResult {
                    success: false,
                    message: format!("Failed to list directory: {}", e),
                    document: None
                })
                .unwrap()
            );
        }
    }
}

fn cmd_search(path: &str, query: &str) {
    match fs::read_to_string(path) {
        Ok(content) => {
            let query_lower = query.to_lowercase();
            let mut matches = Vec::new();
            for (i, line) in content.lines().enumerate() {
                if line.to_lowercase().contains(&query_lower) {
                    matches.push(Match {
                        line: i + 1,
                        content: line.to_string(),
                    });
                }
            }
            println!(
                "{}",
                serde_json::to_string(&SearchResult {
                    query: query.to_string(),
                    total: matches.len(),
                    matches,
                })
                .unwrap()
            );
        }
        Err(e) => {
            println!(
                "{}",
                serde_json::to_string(&EditResult {
                    success: false,
                    message: format!("Failed to read file: {}", e),
                    document: None
                })
                .unwrap()
            );
        }
    }
}

fn cmd_headings(path: &str) {
    match fs::read_to_string(path) {
        Ok(content) => {
            let doc = parse_markdown(&content);
            println!("{}", serde_json::to_string(&doc.headings).unwrap());
        }
        Err(e) => {
            println!(
                "{}",
                serde_json::to_string(&EditResult {
                    success: false,
                    message: format!("Failed to read file: {}", e),
                    document: None
                })
                .unwrap()
            );
        }
    }
}

fn cmd_stats(path: &str) {
    match fs::read_to_string(path) {
        Ok(content) => {
            let mut doc = parse_markdown(&content);
            doc.path = path.to_string();
            println!(
                "{}",
                serde_json::to_string(&serde_json::json!({
                    "path": doc.path,
                    "word_count": doc.word_count,
                    "line_count": doc.line_count,
                    "heading_count": doc.headings.len(),
                }))
                .unwrap()
            );
        }
        Err(e) => {
            println!(
                "{}",
                serde_json::to_string(&EditResult {
                    success: false,
                    message: format!("Failed to read file: {}", e),
                    document: None
                })
                .unwrap()
            );
        }
    }
}

fn cmd_to_jsonl(path: &str) {
    match fs::read_to_string(path) {
        Ok(content) => {
            let entries = parse_markdown_to_jsonl(&content);
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            for entry in entries {
                writeln!(handle, "{}", serde_json::to_string(&entry).unwrap()).unwrap();
            }
        }
        Err(e) => {
            println!(
                "{}",
                serde_json::to_string(&EditResult {
                    success: false,
                    message: format!("Failed to read file: {}", e),
                    document: None
                })
                .unwrap()
            );
        }
    }
}

fn cmd_lint(path: &str, is_content: bool) {
    let content = if is_content {
        unescape_content(path)
    } else {
        match fs::read_to_string(path) {
            Ok(content) => content,
            Err(e) => {
                println!(
                    "{}",
                    serde_json::to_string(&LintResult {
                        valid: false,
                        errors: vec![LintError {
                            line: 0,
                            column: 0,
                            message: format!("Failed to read file: {}", e),
                            rule: "file-read".to_string(),
                        }],
                        warnings: vec![],
                    })
                    .unwrap()
                );
                return;
            }
        }
    };

    let result = validate_markdown(&content);
    println!("{}", serde_json::to_string(&result).unwrap());
}

fn cmd_lint_file(path: &str) {
    match fs::read_to_string(path) {
        Ok(content) => {
            let result = validate_markdown(&content);

            // Print file path
            println!("Linting file: {}", path);
            println!();

            // Print errors first
            if !result.errors.is_empty() {
                println!("ERRORS:");
                for error in &result.errors {
                    println!(
                        "ERROR (line {}, column {}): {} [{}]",
                        error.line, error.column, error.message, error.rule
                    );
                }
                println!();
            }

            // Print warnings
            if !result.warnings.is_empty() {
                println!("WARNINGS:");
                for warning in &result.warnings {
                    println!(
                        "WARNING (line {}, column {}): {} [{}]",
                        warning.line, warning.column, warning.message, warning.rule
                    );
                }
                println!();
            }

            // Print summary
            let total_issues = result.errors.len() + result.warnings.len();
            if total_issues == 0 {
                println!("✓ No issues found. File is valid.");
            } else {
                println!(
                    "Summary: {} errors, {} warnings ({} total issues)",
                    result.errors.len(),
                    result.warnings.len(),
                    total_issues
                );
                if !result.valid {
                    println!("✗ File is invalid due to errors.");
                } else {
                    println!("✓ File is valid but has warnings.");
                }
            }
        }
        Err(e) => {
            eprintln!("ERROR: Failed to read file '{}': {}", path, e);
            std::process::exit(1);
        }
    }
}

pub fn parse_markdown_to_jsonl(content: &str) -> Vec<JsonlEntry> {
    let parser = MarkdownParser::new(content);
    let mut entries = Vec::new();
    let mut current_text = String::new();
    let mut current_heading_level: Option<u32> = None;
    let mut current_heading_text = String::new();
    let mut in_heading = false;
    let mut in_code_block = false;
    let mut code_language = String::new();
    let mut code_content = String::new();

    let flush_text = |text: &str, entries: &mut Vec<JsonlEntry>| {
        if !text.trim().is_empty() {
            entries.push(JsonlEntry {
                entry_type: "paragraph".to_string(),
                content: text.trim().to_string(),
                level: None,
                language: None,
            });
        }
    };

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                flush_text(&current_text, &mut entries);
                current_text = String::new();
                in_heading = true;
                current_heading_level = Some(match level {
                    HeadingLevel::H1 => 1,
                    HeadingLevel::H2 => 2,
                    HeadingLevel::H3 => 3,
                    HeadingLevel::H4 => 4,
                    HeadingLevel::H5 => 5,
                    HeadingLevel::H6 => 6,
                });
                current_heading_text = String::new();
            }
            Event::Text(text) if in_heading => {
                current_heading_text.push_str(&text);
            }
            Event::End(TagEnd::Heading(_)) => {
                if !current_heading_text.is_empty() {
                    entries.push(JsonlEntry {
                        entry_type: "heading".to_string(),
                        content: current_heading_text.clone(),
                        level: current_heading_level,
                        language: None,
                    });
                }
                in_heading = false;
                current_heading_level = None;
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                flush_text(&current_text, &mut entries);
                current_text = String::new();
                in_code_block = true;
                code_content = String::new();
                code_language = match kind {
                    CodeBlockKind::Fenced(lang) => lang.to_string(),
                    CodeBlockKind::Indented => String::new(),
                };
            }
            Event::End(TagEnd::CodeBlock) => {
                entries.push(JsonlEntry {
                    entry_type: "code_block".to_string(),
                    content: code_content.clone(),
                    level: None,
                    language: if code_language.is_empty() {
                        None
                    } else {
                        Some(code_language.clone())
                    },
                });
                in_code_block = false;
            }
            Event::Text(text) if in_code_block => {
                code_content.push_str(&text);
            }
            Event::Text(text) if !in_heading && !in_code_block => {
                current_text.push_str(&text);
                current_text.push(' ');
            }
            Event::Code(code) if !in_heading && !in_code_block => {
                current_text.push_str(&code);
                current_text.push(' ');
            }
            Event::End(TagEnd::Paragraph) if !in_heading && !in_code_block => {
                flush_text(&current_text, &mut entries);
                current_text = String::new();
            }
            Event::End(TagEnd::Item) if !in_heading && !in_code_block => {
                flush_text(&current_text, &mut entries);
                current_text = String::new();
            }
            Event::SoftBreak | Event::HardBreak if !in_heading && !in_code_block => {
                current_text.push(' ');
            }
            _ => {}
        }
    }

    flush_text(&current_text, &mut entries);
    entries
}

#[cfg(test)]
mod tests;
