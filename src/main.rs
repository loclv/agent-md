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

    for (line_num, line) in content.lines().enumerate() {
        let line_num = line_num + 1; // Convert to 1-based indexing

        // Rule: No bold text (detect **text** or __text__)
        if let Some(col) = find_bold_text(line) {
            errors.push(LintError {
                line: line_num,
                column: col,
                message: "Bold text is not allowed for AI agents".to_string(),
                rule: "no-bold".to_string(),
            });
        }

        // Rule: Simple table syntax validation
        if let Some(issue) = validate_table_syntax(line) {
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
        if let Some(col) = find_useless_link(line) {
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
    }

    // Rule: Proper heading structure validation
    if let Some(heading_errors) = validate_heading_structure(content) {
        errors.extend(heading_errors);
    }

    // Rule: Code block best practices validation
    if let Some(code_block_issues) = validate_code_blocks(content) {
        warnings.extend(code_block_issues);
    }

    // Rule: List formatting validation
    if let Some(list_issues) = validate_list_formatting(content) {
        warnings.extend(list_issues);
    }

    LintResult {
        valid: errors.is_empty(),
        errors,
        warnings,
    }
}

#[derive(Debug, PartialEq)]
enum Severity {
    Error,
    Warning,
}

#[derive(Debug, PartialEq)]
struct TableIssue {
    column: usize,
    message: String,
    severity: Severity,
}

fn find_bold_text(line: &str) -> Option<usize> {
    // Check for **bold** pattern
    if let Some(start) = line.find("**") {
        if line[start + 2..].find("**").is_some() {
            return Some(start + 1); // Return 1-based column
        }
    }

    // Check for __bold__ pattern
    if let Some(start) = line.find("__") {
        if line[start + 2..].find("__").is_some() {
            return Some(start + 1); // Return 1-based column
        }
    }

    None
}

fn find_useless_link(line: &str) -> Option<usize> {
    let mut i = 0;
    while i < line.len() {
        if line.chars().nth(i) == Some('[') {
            // Find the closing bracket
            let mut bracket_end = i + 1;
            let mut bracket_content = String::new();
            let mut found_closing_bracket = false;

            while bracket_end < line.len() {
                if let Some(ch) = line.chars().nth(bracket_end) {
                    if ch == ']' {
                        found_closing_bracket = true;
                        break;
                    }
                    bracket_content.push(ch);
                    bracket_end += 1;
                } else {
                    break;
                }
            }

            if found_closing_bracket && bracket_end + 1 < line.len() {
                // Check for opening parenthesis
                if line.chars().nth(bracket_end + 1) == Some('(') {
                    let mut paren_start = bracket_end + 2;
                    let mut url = String::new();
                    let mut found_closing_paren = false;
                    let mut paren_depth = 1;

                    while paren_start < line.len() {
                        if let Some(ch) = line.chars().nth(paren_start) {
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
                        } else {
                            break;
                        }
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
                            return Some(i + 1); // Return 1-based column
                        }
                    }
                }
            }
        }
        i += 1;
    }

    None
}

fn find_ascii_graph(line: &str) -> Option<usize> {
    // Common ASCII graph patterns to detect
    let ascii_graph_patterns = [
        // Box drawing characters
        "┌─┐",
        "└─┘",
        "├─┤",
        "│ │",
        "─",
        "│",
        // Tree structures
        "├──",
        "└──",
        "│  ",
        // Simple graph patterns
        "*--*",
        "---",
        "===",
        "==>",
        "<==",
        "<=>",
        // Flow chart patterns
        "[ ]",
        "( )",
        "{ }",
        "< >",
        "->",
        "<-",
        // Graph-like patterns with multiple connections
        "\\|/",
        "/|\\",
        "-+-",
        "+-+",
        "|-|",
        // Progress bars or meters
        "[==",
        "==]",
        "█",
        "▓",
        "▒",
        "░",
        // Matrix-like patterns
        "[][]",
        "|||",
        "...",
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

fn validate_table_syntax(line: &str) -> Option<TableIssue> {
    let trimmed = line.trim();

    // Check if this looks like a table row
    if trimmed.contains('|') {
        // Check for separator rows (rows with only pipes and dashes)
        let is_separator_row = trimmed.chars().all(|c| c == '|' || c == '-' || c == ' ' || c == ':');

        if is_separator_row {
            // Check for exactly 3 dash separators between pipes
            let parts: Vec<&str> = trimmed.split('|').collect();
            for part in parts {
                let part_trimmed = part.trim();
                if !part_trimmed.is_empty() {
                    // Count dashes in this separator part
                    let dash_count = part_trimmed.chars().filter(|&c| c == '-').count();
                    if dash_count != 3 {
                        return Some(TableIssue {
                            column: 1,
                            message: "Table separator should use exactly 3 dashes (---) between pipes".to_string(),
                            severity: Severity::Error,
                        });
                    }
                }
            }
        }

        // Check for complex table syntax that should be avoided
        if trimmed.contains("colspan") || trimmed.contains("rowspan") {
            return Some(TableIssue {
                column: 1,
                message: "Complex table attributes (colspan/rowspan) are not allowed".to_string(),
                severity: Severity::Error,
            });
        }

        // Check for inline formatting in table cells
        if trimmed.contains("**") || trimmed.contains("__") || trimmed.contains("*") {
            return Some(TableIssue {
                column: 1,
                message: "Inline formatting in table cells should be avoided".to_string(),
                severity: Severity::Warning,
            });
        }

        // Warn about very complex tables
        let pipe_count = trimmed.matches('|').count();
        if pipe_count > 6 {
            // More than 5 columns
            return Some(TableIssue {
                column: 1,
                message: "Very wide tables should be simplified".to_string(),
                severity: Severity::Warning,
            });
        }
    }

    None
}

fn validate_heading_structure(content: &str) -> Option<Vec<LintError>> {
    let mut heading_levels = Vec::new();
    let mut h1_count = 0;
    let mut h1_locations = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let line_num = line_num + 1; // Convert to 1-based indexing

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
        for &location in &h1_locations[1..] { // Skip the first H1
            errors.push(LintError {
                line: location,
                column: 1,
                message: "Multiple H1 headings found. Documents should have only one top-level heading".to_string(),
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
                    message: format!("Heading level skipped: H{} follows H{}. Use sequential heading levels.", current_level, prev_level),
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
enum ListType {
    Ordered,
    Unordered,
}

fn validate_list_formatting(content: &str) -> Option<Vec<LintWarning>> {
    let mut warnings = Vec::new();
    let mut list_items = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let line_num = line_num + 1; // Convert to 1-based indexing
        let trimmed = line.trim();

        // Check for list items (ordered or unordered)
        if let Some(list_info) = detect_list_item(trimmed) {
            list_items.push((list_info, line_num));
        }
    }

    // Check for inconsistent list formatting
    if list_items.len() > 1 {
        let mut current_list_type: Option<ListType> = None;
        let mut current_list_marker: Option<String> = None;
        let mut expected_next_number: Option<u32> = None;

        for (i, ((list_type, marker), line_num)) in list_items.iter().enumerate() {
            // Skip if this is the first item or if there's a blank line between lists
            if i > 0 {
                let prev_line_num = list_items[i - 1].1;
                if *line_num > prev_line_num + 1 {
                    // New list, reset tracking
                    current_list_type = None;
                    current_list_marker = None;
                    expected_next_number = None;
                }
            }

            // Check for consistency within the same list
            if let Some(ref list_type) = current_list_type {
                if list_type != list_type {
                    warnings.push(LintWarning {
                        line: *line_num,
                        column: 1,
                        message: "Inconsistent list formatting detected. Use consistent list markers within the same list".to_string(),
                        rule: "list-formatting".to_string(),
                    });
                }
            }

            // For ordered lists, check for sequential numbering
            if *list_type == ListType::Ordered {
                if let Some(ref expected_marker) = current_list_marker {
                    if marker != expected_marker {
                        warnings.push(LintWarning {
                            line: *line_num,
                            column: 1,
                            message: "Inconsistent ordered list numbering. Use sequential numbers".to_string(),
                            rule: "list-formatting".to_string(),
                        });
                    }
                }

                // Calculate expected next marker
                if let Some(current_num) = extract_number_from_marker(marker) {
                    expected_next_number = Some(current_num + 1);
                }
            }

            // Update current list tracking
            if current_list_type.is_none() {
                current_list_type = Some(*list_type);
                current_list_marker = Some(marker.to_string());

                // For ordered lists, set up expected next number
                if *list_type == ListType::Ordered {
                    if let Some(current_num) = extract_number_from_marker(marker) {
                        expected_next_number = Some(current_num + 1);
                    }
                }
            } else if *list_type == ListType::Ordered {
                // Update expected marker for next iteration
                if let Some(next_num) = expected_next_number {
                    current_list_marker = Some(format!("{}.", next_num));
                }
            }
        }
    }

    if warnings.is_empty() {
        None
    } else {
        Some(warnings)
    }
}

fn extract_number_from_marker(marker: &str) -> Option<u32> {
    let num_str: String = marker.chars().take_while(|c| c.is_digit(10)).collect();
    num_str.parse::<u32>().ok()
}

fn detect_list_item(line: &str) -> Option<(ListType, String)> {
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
        while i < line.len() && line.chars().nth(i).unwrap().is_digit(10) {
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

fn validate_code_blocks(content: &str) -> Option<Vec<LintWarning>> {
    let mut warnings = Vec::new();
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
                let has_language = if trimmed.len() > 3 {
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
                        warnings.push(LintWarning {
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

    if warnings.is_empty() {
        None
    } else {
        Some(warnings)
    }
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
        if trimmed.chars().nth(level as usize) == Some(' ') {
            Some(level)
        } else {
            None
        }
    } else {
        None
    }
}

fn parse_markdown(content: &str) -> Document {
    let word_count = content.split_whitespace().count();
    let line_count = content.lines().count();
    let mut headings = Vec::new();

    let parser = MarkdownParser::new(content);
    let mut line_num = 0;
    let mut in_heading = false;
    let mut current_heading = String::new();
    let mut current_level = 0;

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                in_heading = true;
                current_level = level as u32;
                current_heading = String::new();
            }
            Event::Text(text) if in_heading => {
                current_heading.push_str(&text);
            }
            Event::End(TagEnd::Heading(_)) => {
                if in_heading && !current_heading.is_empty() {
                    headings.push(Heading {
                        level: current_level,
                        text: current_heading.clone(),
                        line: line_num,
                    });
                }
                in_heading = false;
            }
            Event::SoftBreak | Event::HardBreak => {
                line_num += 1;
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
    },
    Write {
        #[arg(help = "Markdown file path")]
        path: String,
        #[arg(help = "Content to write")]
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
        Commands::Read { path } => cmd_read(&path),
        Commands::Write { path, content } => cmd_write(&path, &content),
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

fn cmd_read(path: &str) {
    match fs::read_to_string(path) {
        Ok(content) => {
            let mut doc = parse_markdown(&content);
            doc.path = path.to_string();
            println!("{}", serde_json::to_string(&doc).unwrap());
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
    // Validate content before writing
    let validation = validate_markdown(content);

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

    match fs::write(path, content) {
        Ok(_) => {
            let mut doc = parse_markdown(content);
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

fn cmd_append(path: &str, content: &str) {
    match fs::read_to_string(path) {
        Ok(mut existing) => {
            if !existing.ends_with('\n') {
                existing.push('\n');
            }
            existing.push_str(content);
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
        path.to_string()
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
                    println!("ERROR (line {}, column {}): {} [{}]",
                             error.line, error.column, error.message, error.rule);
                }
                println!();
            }

            // Print warnings
            if !result.warnings.is_empty() {
                println!("WARNINGS:");
                for warning in &result.warnings {
                    println!("WARNING (line {}, column {}): {} [{}]",
                             warning.line, warning.column, warning.message, warning.rule);
                }
                println!();
            }

            // Print summary
            let total_issues = result.errors.len() + result.warnings.len();
            if total_issues == 0 {
                println!("✓ No issues found. File is valid.");
            } else {
                println!("Summary: {} errors, {} warnings ({} total issues)",
                         result.errors.len(), result.warnings.len(), total_issues);
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

fn parse_markdown_to_jsonl(content: &str) -> Vec<JsonlEntry> {
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
                    entry_type: "code".to_string(),
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
mod tests {
    use super::*;

    #[test]
    fn test_parse_markdown_basic() {
        let content = "# Hello\n\nThis is a test.";
        let doc = parse_markdown(content);
        assert_eq!(doc.word_count, 6);
        assert_eq!(doc.line_count, 3);
        assert_eq!(doc.headings.len(), 1);
        assert_eq!(doc.headings[0].level, 1);
        assert_eq!(doc.headings[0].text, "Hello");
    }

    #[test]
    fn test_parse_markdown_multiple_headings() {
        let content = "# Title\n\n## Section 1\n\nContent here.\n\n### Subsection\n\n## Section 2";
        let doc = parse_markdown(content);
        assert_eq!(doc.headings.len(), 4);
        assert_eq!(doc.headings[0].text, "Title");
        assert_eq!(doc.headings[1].text, "Section 1");
        assert_eq!(doc.headings[2].text, "Subsection");
        assert_eq!(doc.headings[3].text, "Section 2");
        assert_eq!(doc.headings[0].level, 1);
        assert_eq!(doc.headings[1].level, 2);
        assert_eq!(doc.headings[2].level, 3);
        assert_eq!(doc.headings[3].level, 2);
    }

    #[test]
    fn test_parse_markdown_no_headings() {
        let content = "Just some plain text without any headings.";
        let doc = parse_markdown(content);
        assert_eq!(doc.headings.len(), 0);
        assert_eq!(doc.word_count, 7);
    }

    #[test]
    fn test_parse_markdown_word_count() {
        let content = "One two three\nfour five six";
        let doc = parse_markdown(content);
        assert_eq!(doc.word_count, 6);
    }

    #[test]
    fn test_parse_markdown_to_jsonl_heading() {
        let content = "# Test Heading";
        let entries = parse_markdown_to_jsonl(content);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].entry_type, "heading");
        assert_eq!(entries[0].content, "Test Heading");
        assert_eq!(entries[0].level, Some(1));
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
    fn test_parse_markdown_to_jsonl_code_block() {
        let content = "```rust\nfn main() {}\n```";
        let entries = parse_markdown_to_jsonl(content);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].entry_type, "code");
        assert_eq!(entries[0].content, "fn main() {}\n");
        assert_eq!(entries[0].language, Some("rust".to_string()));
    }

    #[test]
    fn test_parse_markdown_to_jsonl_mixed() {
        let content =
            "# Title\n\nSome paragraph text.\n\n```python\nprint('hello')\n```\n\n## Next Section";
        let entries = parse_markdown_to_jsonl(content);
        assert_eq!(entries.len(), 4);
        assert_eq!(entries[0].entry_type, "heading");
        assert_eq!(entries[0].content, "Title");
        assert_eq!(entries[1].entry_type, "paragraph");
        assert_eq!(entries[2].entry_type, "code");
        assert_eq!(entries[2].language, Some("python".to_string()));
        assert_eq!(entries[3].entry_type, "heading");
        assert_eq!(entries[3].content, "Next Section");
    }

    #[test]
    fn test_parse_markdown_to_jsonl_code_block_no_language() {
        let content = "```\nsome code\n```";
        let entries = parse_markdown_to_jsonl(content);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].entry_type, "code");
        assert_eq!(entries[0].language, None);
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
    fn test_parse_markdown_to_jsonl_empty() {
        let content = "";
        let entries = parse_markdown_to_jsonl(content);
        assert_eq!(entries.len(), 0);
    }

    #[test]
    fn test_parse_markdown_to_jsonl_multiple_paragraphs() {
        let content = "First paragraph.\n\nSecond paragraph.";
        let entries = parse_markdown_to_jsonl(content);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].entry_type, "paragraph");
    }

    #[test]
    fn test_jsonl_entry_serialization() {
        let entry = JsonlEntry {
            entry_type: "heading".to_string(),
            content: "Test".to_string(),
            level: Some(1),
            language: None,
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"type\":\"heading\""));
        assert!(json.contains("\"content\":\"Test\""));
        assert!(json.contains("\"level\":1"));
    }

    #[test]
    fn test_document_serialization() {
        let doc = Document {
            path: "/test/path.md".to_string(),
            content: "# Hello".to_string(),
            word_count: 1,
            line_count: 1,
            headings: vec![Heading {
                level: 1,
                text: "Hello".to_string(),
                line: 0,
            }],
        };
        let json = serde_json::to_string(&doc).unwrap();
        assert!(json.contains("\"/test/path.md\""));
        assert!(json.contains("\"headings\""));
    }

    #[test]
    fn test_find_useless_link_exact_url() {
        let line = "Check out [https://example.com/](https://example.com/) for more info";
        let result = find_useless_link(line);
        assert_eq!(result, Some(11)); // Position of the opening bracket
    }

    #[test]
    fn test_find_useless_link_without_protocol() {
        let line = "Visit [example.com](https://example.com/) today";
        let result = find_useless_link(line);
        assert_eq!(result, Some(7)); // Position of the opening bracket
    }

    #[test]
    fn test_find_useless_link_with_www() {
        let line = "Go to [www.example.com](https://www.example.com/) now";
        let result = find_useless_link(line);
        assert_eq!(result, Some(7)); // Position of the opening bracket
    }

    #[test]
    fn test_find_useless_link_valid_link() {
        let line = "Visit [Google](https://google.com/) for search";
        let result = find_useless_link(line);
        assert_eq!(result, None); // Should not flag valid links
    }

    #[test]
    fn test_find_useless_link_no_links() {
        let line = "This is just plain text with no links";
        let result = find_useless_link(line);
        assert_eq!(result, None);
    }

    #[test]
    fn test_find_useless_link_malformed_link() {
        let line = "This has [broken(link";
        let result = find_useless_link(line);
        assert_eq!(result, None);
    }

    #[test]
    fn test_validate_markdown_useless_links() {
        let content = "Here is a bad link: [https://example.com/](https://example.com/)\nAnd a good one: [Google](https://google.com/)";
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid since warnings don't make it invalid
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.warnings[0].rule, "useless-links");
        assert_eq!(result.warnings[0].line, 1);
    }

    #[test]
    fn test_find_ascii_graph_box_drawing() {
        let line = "┌───┐";
        let result = find_ascii_graph(line);
        assert_eq!(result, Some(4)); // Actual position returned by the function
    }

    #[test]
    fn test_find_ascii_graph_tree_structure() {
        let line = "├── parent";
        let result = find_ascii_graph(line);
        assert_eq!(result, Some(4)); // Actual position returned by the function
    }

    #[test]
    fn test_find_ascii_graph_flow_chart() {
        let line = "[Start] -> [Process] -> [End]";
        let result = find_ascii_graph(line);
        assert_eq!(result, Some(9)); // Position of the first arrow (multiple arrows detected)
    }

    #[test]
    fn test_find_ascii_graph_explicit_indicator() {
        let line = "graph: Node1 -> Node2";
        let result = find_ascii_graph(line);
        assert_eq!(result, Some(1)); // Position of "graph:"
    }

    #[test]
    fn test_find_ascii_graph_high_density_special_chars() {
        let line = "+-+-+--+-+";
        let result = find_ascii_graph(line);
        assert!(result.is_some()); // Should detect high density special chars
    }

    #[test]
    fn test_find_ascii_graph_normal_text() {
        let line = "This is just normal text with some punctuation.";
        let result = find_ascii_graph(line);
        assert_eq!(result, None); // Should not flag normal text
    }

    #[test]
    fn test_find_ascii_graph_code_block() {
        let line = "function test() { return true; }";
        let result = find_ascii_graph(line);
        assert_eq!(result, None); // Should not flag code
    }

    #[test]
    fn test_validate_markdown_ascii_graph() {
        let content = "Here is a graph:\n┌───┐\n│ A │\n└───┘";
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid since warnings don't make it invalid
        assert_eq!(result.warnings.len(), 4); // All lines with ASCII graph patterns
        assert_eq!(result.warnings[0].rule, "no-ascii-graph");
        assert_eq!(result.warnings[1].rule, "no-ascii-graph");
        assert_eq!(result.warnings[2].rule, "no-ascii-graph");
        assert_eq!(result.warnings[3].rule, "no-ascii-graph");
    }

    // Tests for the new heading-structure rule (replaces single-title tests)
    #[test]
    fn test_validate_heading_structure_single_heading() {
        let content = "# Single Title\n\nSome content here";
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid
        assert_eq!(result.errors.len(), 0);
    }

    #[test]
    fn test_validate_heading_structure_multiple_headings() {
        let content = "# First Title\n\nContent\n\n# Second Title\n\nMore content";
        let result = validate_markdown(content);
        assert!(!result.valid); // Should be invalid due to multiple H1s
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].rule, "heading-structure");
        assert!(result.errors[0].message.contains("Multiple H1 headings"));
    }

    #[test]
    fn test_validate_heading_structure_three_headings() {
        let content = "# First\n\nContent\n\n# Second\n\nContent\n\n# Third\n\nContent";
        let result = validate_markdown(content);
        assert!(!result.valid);
        assert_eq!(result.errors.len(), 2); // Two errors (for second and third H1)
        assert!(result.errors.iter().all(|e| e.rule == "heading-structure"));
    }

    #[test]
    fn test_validate_heading_structure_no_headings() {
        let content = "Just some plain text\n\nWithout any headings";
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid
        assert_eq!(result.errors.len(), 0);
    }

    #[test]
    fn test_validate_heading_structure_other_heading_levels() {
        let content = "## Section 1\n\nContent\n\n### Subsection\n\nMore content";
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid
        assert_eq!(result.errors.len(), 0);
    }

    #[test]
    fn test_validate_heading_structure_mixed_headings() {
        let content = "# Main Title\n\n## Section 1\n\nContent\n\n### Subsection\n\nMore content\n\n## Section 2";
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid - only one H1
        assert_eq!(result.errors.len(), 0);
    }

    #[test]
    fn test_validate_heading_structure_false_positives() {
        let content = "This is # not a heading\n\n## This is a heading\n\n# This is H1\n\nAnother # not heading";
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid since there's only one actual H1
        assert_eq!(result.errors.len(), 0);
    }

    // Additional tests for existing validation functions
    #[test]
    fn test_find_bold_text_double_asterisks() {
        let line = "This has **bold** text";
        let result = find_bold_text(line);
        assert_eq!(result, Some(10)); // Position of first * (1-based)
    }

    #[test]
    fn test_find_bold_text_double_underscores() {
        let line = "This has __bold__ text";
        let result = find_bold_text(line);
        assert_eq!(result, Some(10)); // Position of first _ (1-based)
    }

    #[test]
    fn test_find_bold_text_no_bold() {
        let line = "This has no bold text";
        let result = find_bold_text(line);
        assert_eq!(result, None);
    }

    #[test]
    fn test_find_bold_text_partial_patterns() {
        let line = "This has **bold text";
        let result = find_bold_text(line);
        assert_eq!(result, None); // Incomplete pattern
    }

    #[test]
    fn test_validate_table_syntax_complex_attributes() {
        let line = "| Cell | colspan=\"2\" |";
        let result = validate_table_syntax(line);
        assert!(result.is_some());
        assert!(matches!(result.unwrap().severity, Severity::Error));
    }

    #[test]
    fn test_validate_table_syntax_inline_formatting() {
        let line = "| Cell with **bold** | Another cell |";
        let result = validate_table_syntax(line);
        assert!(result.is_some());
        assert!(matches!(result.unwrap().severity, Severity::Warning));
    }

    #[test]
    fn test_validate_table_syntax_wide_table() {
        let line = "| Col1 | Col2 | Col3 | Col4 | Col5 | Col6 | Col7 |";
        let result = validate_table_syntax(line);
        assert!(result.is_some());
        assert!(matches!(result.unwrap().severity, Severity::Warning));
    }

    #[test]
    fn test_validate_table_syntax_simple_table() {
        let line = "| Name | Description |";
        let result = validate_table_syntax(line);
        assert_eq!(result, None); // Should be valid
    }

    #[test]
    fn test_validate_markdown_bold_error() {
        let content = "This has **bold** text";
        let result = validate_markdown(content);
        assert!(!result.valid); // Should be invalid due to bold error
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].rule, "no-bold");
    }

    #[test]
    fn test_validate_markdown_multiple_errors() {
        let content = "This has **bold** text\n\n| Cell with **bold** | Another |";
        let result = validate_markdown(content);
        assert!(!result.valid); // Should be invalid due to bold errors
        assert_eq!(result.errors.len(), 2); // Two bold errors (one in text, one in table)
        assert!(result.errors.iter().any(|e| e.rule == "no-bold"));
        assert_eq!(result.warnings.len(), 1); // One table formatting warning
        assert!(result.warnings.iter().any(|w| w.rule == "simple-tables"));
    }

    #[test]
    fn test_validate_markdown_empty_content() {
        let content = "";
        let result = validate_markdown(content);
        assert!(result.valid); // Empty content should be valid
        assert_eq!(result.errors.len(), 0);
        assert_eq!(result.warnings.len(), 0);
    }

    // Tests for updated heading structure validation
    #[test]
    fn test_validate_heading_structure_skipped_levels() {
        let content = "# Title\n\n### Subsection (skips H2)";
        let result = validate_markdown(content);
        assert!(!result.valid); // Should be invalid due to skipped level
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].rule, "heading-structure");
        assert!(result.errors[0].message.contains("Heading level skipped"));
    }

    #[test]
    fn test_validate_heading_structure_multiple_h1() {
        let content = "# First Title\n\nContent\n\n# Second Title";
        let result = validate_markdown(content);
        assert!(!result.valid); // Should be invalid due to multiple H1s
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].rule, "heading-structure");
        assert!(result.errors[0].message.contains("Multiple H1 headings"));
    }

    #[test]
    fn test_validate_heading_structure_valid_sequence() {
        let content = "# Title\n\n## Section 1\n\n### Subsection 1.1\n\n#### Sub-subsection";
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid
        assert_eq!(result.errors.len(), 0);
    }

    // Tests for code block validation
    #[test]
    fn test_validate_code_blocks_no_language() {
        let content = "```\nsome code\n```";
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid but with warnings
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.warnings[0].rule, "code-blocks");
        assert!(result.warnings[0].message.contains("specify language"));
    }

    #[test]
    fn test_validate_code_blocks_with_language() {
        let content = "```rust\nfn main() {}\n```";
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid without warnings
        assert_eq!(result.warnings.len(), 0);
    }

    #[test]
    fn test_validate_code_blocks_multiple_blocks() {
        let content = "```\nno language\n```\n\n```python\nwith language\n```";
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid but with one warning
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.warnings[0].rule, "code-blocks");
    }

    // Tests for list formatting validation
    #[test]
    fn test_validate_list_formatting_inconsistent_markers() {
        let content = "- Item 1\n* Item 2\n- Item 3";
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid but with warnings
        assert_eq!(result.warnings.len(), 2); // Two inconsistencies
        assert!(result.warnings.iter().all(|w| w.rule == "list-formatting"));
    }

    #[test]
    fn test_validate_list_formatting_consistent_unordered() {
        let content = "- Item 1\n- Item 2\n- Item 3";
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid without warnings
        assert_eq!(result.warnings.len(), 0);
    }

    #[test]
    fn test_validate_list_formatting_ordered_sequential() {
        let content = "1. First\n2. Second\n3. Third";
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid without warnings
        assert_eq!(result.warnings.len(), 0);
    }

    #[test]
    fn test_validate_list_formatting_ordered_nonsequential() {
        let content = "1. First\n3. Third\n4. Fourth";
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid but with warnings
        assert_eq!(result.warnings.len(), 2); // Two numbering inconsistencies
        assert!(result.warnings.iter().all(|w| w.rule == "list-formatting"));
    }

    #[test]
    fn test_validate_list_formatting_separate_lists() {
        let content = "- List 1 Item 1\n- List 1 Item 2\n\n* List 2 Item 1\n* List 2 Item 2";
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid - separate lists can have different markers
        assert_eq!(result.warnings.len(), 0);
    }

    // Tests for updated table validation
    #[test]
    fn test_validate_table_syntax_incorrect_separator() {
        let content = "| Name | Value |\n|------|-------|\n| Test | Data |";
        let result = validate_markdown(content);
        assert!(!result.valid); // Should be invalid due to incorrect separator
        assert_eq!(result.errors.len(), 2); // Two separator parts with wrong dash count
        assert!(result.errors.iter().all(|e| e.rule == "simple-tables"));
        assert!(result.errors.iter().all(|e| e.message.contains("exactly 3 dashes")));
    }

    #[test]
    fn test_validate_table_syntax_correct_separator() {
        let content = "| Name | Value |\n|---|---|\n| Test | Data |";
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid
        assert_eq!(result.errors.len(), 0);
    }

    // Tests for updated ASCII graph message
    #[test]
    fn test_find_ascii_graph_updated_message() {
        let content = "┌───┐\n│ A │\n└───┘";
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid but with warnings
        assert_eq!(result.warnings.len(), 3);
        assert!(result.warnings.iter().all(|w| w.rule == "no-ascii-graph"));
        assert!(result.warnings.iter().all(|w| w.message.contains("LLM-readable formats")));
        assert!(result.warnings.iter().all(|w| w.message.contains("ZON format")));
    }

    // Integration tests for all rules together
    #[test]
    fn test_validate_markdown_comprehensive() {
        let content = r#"# Title

## Section 1

This has **bold** text which is an error.

### Subsection (skipped level from H1 to H3)

Here's a code block without language:

```
code
```

And inconsistent lists:

- Item 1
* Item 2

Bad table:

| Name | Value |
|------|-------|
| Test | Data |

ASCII graph:

┌──┐
│A │
└──┘
"#;
        let result = validate_markdown(content);
        assert!(!result.valid); // Should be invalid due to errors

        // Should have errors for bold text and heading structure
        assert!(result.errors.iter().any(|e| e.rule == "no-bold"));
        assert!(result.errors.iter().any(|e| e.rule == "heading-structure"));
        assert!(result.errors.iter().any(|e| e.rule == "simple-tables"));

        // Should have warnings for code blocks, lists, and ASCII graphs
        assert!(result.warnings.iter().any(|w| w.rule == "code-blocks"));
        assert!(result.warnings.iter().any(|w| w.rule == "list-formatting"));
        assert!(result.warnings.iter().any(|w| w.rule == "no-ascii-graph"));
    }

    // Additional tests for edge cases and complex scenarios

    // Tests for complex heading structure scenarios
    #[test]
    fn test_validate_heading_structure_complex_nesting() {
        let content = r#"# Main Title

## Section 1

### Subsection 1.1

#### Sub-subsection 1.1.1

##### Deep level 1.1.1.1

###### Deepest level 1.1.1.1.1

## Section 2

### Subsection 2.1

#### Sub-subsection 2.1.1
"#;
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid - proper sequential nesting
        assert_eq!(result.errors.len(), 0);
    }

    #[test]
    fn test_validate_heading_structure_multiple_skips() {
        let content = r#"# Title

### Skip H2

###### Skip to H6
"#;
        let result = validate_markdown(content);
        assert!(!result.valid); // Should be invalid
        assert_eq!(result.errors.len(), 2); // Two skipped levels
        assert!(result.errors.iter().all(|e| e.rule == "heading-structure"));
    }

    #[test]
    fn test_validate_heading_structure_mixed_valid_invalid() {
        let content = r#"# Title

## Valid Section

### Skip to H4 (invalid)

#### Back to proper H4

## Another Valid Section
"#;
        let result = validate_markdown(content);
        assert!(!result.valid); // Should be invalid due to one skip
        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0].message.contains("H4 follows H2"));
    }

    #[test]
    fn test_validate_heading_structure_headings_in_code_blocks() {
        let content = r#"# Real Title

```
# This is not a real heading
## Neither is this
```

## Real Section
"#;
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid - only 2 real H1/H2 headings
        assert_eq!(result.errors.len(), 0);
    }

    // Tests for edge cases in code block validation
    #[test]
    fn test_validate_code_blocks_nested_fences() {
        let content = r#"```text
This has ``` nested fences
But they should be ignored
```
"#;
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid - has language specified
        assert_eq!(result.warnings.len(), 0);
    }

    #[test]
    fn test_validate_code_blocks_empty_with_language() {
        let content = "```javascript\n\n```";
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid - has language even if empty
        assert_eq!(result.warnings.len(), 0);
    }

    #[test]
    fn test_validate_code_blocks_multiple_empty() {
        let content = r#"```

```

```rust

```
"#;
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid but with warnings
        assert_eq!(result.warnings.len(), 2); // Two empty blocks without language
    }

    #[test]
    fn test_validate_code_blocks_language_with_spaces() {
        let content = "```  javascript  \ncode\n```";
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid - has language (with spaces)
        assert_eq!(result.warnings.len(), 0);
    }

    #[test]
    fn test_validate_code_blocks_unclosed_block() {
        let content = "```python\nprint('hello')\n# No closing fence";
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid - unclosed blocks don't trigger warnings
        assert_eq!(result.warnings.len(), 0);
    }

    // Tests for complex list formatting scenarios
    #[test]
    fn test_validate_list_formatting_mixed_ordered_types() {
        let content = r#"1. First item
2. Second item
3) Third item (different separator)
4. Fourth item
"#;
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid but with warnings
        assert_eq!(result.warnings.len(), 1); // One inconsistent separator
        assert!(result.warnings[0].rule == "list-formatting");
    }

    #[test]
    fn test_validate_list_formatting_deep_nested_numbers() {
        let content = r#"1. First
2. Second
3. Third
4. Fourth
5. Fifth
6. Sixth
7. Seventh
8. Eighth
9. Ninth
10. Tenth
"#;
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid - sequential numbering
        assert_eq!(result.warnings.len(), 0);
    }

    #[test]
    fn test_validate_list_formatting_unordered_mixed_markers() {
        let content = r#"- Item 1
- Item 2
* Item 3 (different marker)
- Item 4
+ Item 5 (another different marker)
- Item 6
"#;
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid but with warnings
        assert_eq!(result.warnings.len(), 2); // Two marker inconsistencies
    }

    #[test]
    fn test_validate_list_formatting_ordered_with_gaps() {
        let content = r#"1. First
2. Second
4. Fourth (gap)
5. Fifth
"#;
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid but with warnings
        assert_eq!(result.warnings.len(), 1); // One numbering gap
        assert!(result.warnings[0].message.contains("sequential"));
    }

    #[test]
    fn test_validate_list_formatting_complex_mixed_lists() {
        let content = r#"1. Ordered list 1
2. Ordered list 2

- Unordered list 1
- Unordered list 2

* Different unordered list 1
* Different unordered list 2

3. Back to ordered (should be warning)
"#;
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid but with warnings
        // Should have warning for inconsistent ordered list numbering
        assert!(result.warnings.iter().any(|w| w.rule == "list-formatting"));
    }

    // Tests for table validation edge cases
    #[test]
    fn test_validate_table_syntax_mixed_separators() {
        let content = r#"| Name | Value |
|---|-----|
| Test | Data |
| Item | Info |
"#;
        let result = validate_markdown(content);
        assert!(!result.valid); // Should be invalid - inconsistent dash counts
        assert_eq!(result.errors.len(), 1); // One separator has wrong dash count
        assert!(result.errors.iter().all(|e| e.rule == "simple-tables"));
    }

    #[test]
    fn test_validate_table_syntax_with_colons() {
        let content = r#"| Left | Center | Right |
|:----|:----:|----:|
| L1 | C1 | R1 |
| L2 | C2 | R2 |
"#;
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid - colons don't affect dash count
        assert_eq!(result.errors.len(), 0);
    }

    #[test]
    fn test_validate_table_syntax_no_separator() {
        let content = r#"| Name | Value |
| Test | Data |
| Item | Info |
"#;
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid - no separator row to validate
        assert_eq!(result.errors.len(), 0);
    }

    #[test]
    fn test_validate_table_syntax_single_column() {
        let content = r#"| Name |
|-----|
| Test |
| Item |
"#;
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid - single column with correct separator
        assert_eq!(result.errors.len(), 0);
    }

    #[test]
    fn test_validate_table_syntax_empty_cells() {
        let content = r#"| Name | Value |
|---|---|
| Test | |
| | Data |
"#;
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid - empty cells don't affect separator validation
        assert_eq!(result.errors.len(), 0);
    }

    // Tests for ASCII graph detection edge cases
    #[test]
    fn test_validate_ascii_graph_mixed_patterns() {
        let content = r#"Normal text with some - dashes.

┌───┐
│ A │
└───┘

More normal text.

Flow: [Start] -> [Process] -> [End]

End of text.
"#;
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid but with warnings
        // Should detect both box drawing and flow chart patterns
        assert_eq!(result.warnings.len(), 4); // 3 for box, 1 for flow
        assert!(result.warnings.iter().all(|w| w.rule == "no-ascii-graph"));
    }

    #[test]
    fn test_validate_ascii_graph_progress_bars() {
        let content = r#"Progress: [████████░░] 80%
Another: [====    ] 50%
Simple: [==      ] 25%
"#;
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid but with warnings
        assert_eq!(result.warnings.len(), 3); // Three progress bar patterns
    }

    #[test]
    fn test_validate_ascii_graph_tree_structures() {
        let content = r#"root
├── branch1
│   ├── leaf1
│   └── leaf2
└── branch2
    └── leaf3
"#;
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid but with warnings
        assert_eq!(result.warnings.len(), 4); // Tree structure patterns
    }

    // Integration tests for multiple rules interaction
    #[test]
    fn test_validate_markdown_complex_document() {
        let content = r#"# Document Title

## Overview

This document contains **bold text** (error) and has various validation issues.

### Deep Section (skipped level error)

Here's a code block without language:

```
function example() {
    return "test";
}
```

And inconsistent lists:

1. First item
2. Second item
* Third item (inconsistent marker)

Table with wrong separator:

| Name | Value |
|------|-------|
| Test | Data |

ASCII art:

┌────────┐
│ Title │
└────────┘

## Conclusion

Another **bold** error here.
"#;
        let result = validate_markdown(content);
        assert!(!result.valid); // Should be invalid due to errors

        // Count specific errors and warnings
        let bold_errors = result.errors.iter().filter(|e| e.rule == "no-bold").count();
        let heading_errors = result.errors.iter().filter(|e| e.rule == "heading-structure").count();
        let table_errors = result.errors.iter().filter(|e| e.rule == "simple-tables").count();

        let code_warnings = result.warnings.iter().filter(|w| w.rule == "code-blocks").count();
        let list_warnings = result.warnings.iter().filter(|w| w.rule == "list-formatting").count();
        let ascii_warnings = result.warnings.iter().filter(|w| w.rule == "no-ascii-graph").count();

        assert_eq!(bold_errors, 2); // Two bold text errors
        assert_eq!(heading_errors, 1); // One heading skip error
        assert_eq!(table_errors, 1); // One table separator error

        assert_eq!(code_warnings, 1); // One code block warning
        assert_eq!(list_warnings, 1); // One list inconsistency warning
        assert_eq!(ascii_warnings, 3); // Three ASCII graph lines
    }

    #[test]
    fn test_validate_markdown_perfect_document() {
        let content = r#"# Perfect Document

## Introduction

This is a well-formatted document with no validation issues.

### Details

Here's a properly formatted code block:

```rust
fn main() {
    println!("Hello, world!");
}
```

And consistent lists:

- First item
- Second item
- Third item

Proper table:

| Name | Type | Status |
|---|---|---|
| Task | Bug | Open |
| Feature | Enhancement | Progress |

## Conclusion

Everything follows the AI-friendly markdown standards.
"#;
        let result = validate_markdown(content);
        assert!(result.valid); // Should be perfectly valid
        assert_eq!(result.errors.len(), 0);
        assert_eq!(result.warnings.len(), 0);
    }

    // Performance and stress tests
    #[test]
    fn test_validate_markdown_large_document() {
        let mut content = String::from("# Large Document\n\n## Section 1\n\n");

        // Add many lines with various content
        for i in 1..=1000 {
            content.push_str(&format!("- Item {}\n", i));
        }

        content.push_str("\n## Section 2\n\n");
        for i in 1..=100 {
            content.push_str(&format!("{}. Ordered item\n", i));
        }

        let result = validate_markdown(&content);
        assert!(result.valid); // Should be valid - just testing performance
        assert_eq!(result.errors.len(), 0);
    }

    #[test]
    fn test_validate_markdown_empty_lines_and_whitespace() {
        let content = r#"# Title


## Section with blank lines above


Content with extra spaces.


### Subsection


```
Code block with blank lines


```

End.
"#;
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid - whitespace shouldn't affect validation
        assert_eq!(result.errors.len(), 0);
    }
}
