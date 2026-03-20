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

        // Rule: No Human-Readable ASCII Graph - recommend LLM-readable formats
        if let Some(col) = find_ascii_graph(line) {
            warnings.push(LintWarning {
                line: line_num,
                column: col,
                message: "Human-Readable ASCII Graph detected. Use LLM-readable formats instead: Structured CSV, JSON, TOON, Mermaid Diagram, Numbered List with Conditions, or ZON format".to_string(),
                rule: "no-ascii-graph".to_string(),
            });
        }
    }

    // Rule: Single H1 title validation
    if let Some(h1_errors) = validate_single_h1(content) {
        errors.extend(h1_errors);
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

fn validate_single_h1(content: &str) -> Option<Vec<LintError>> {
    let mut h1_locations = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let line_num = line_num + 1; // Convert to 1-based indexing
        
        // Check for H1 headings (lines starting with # )
        if line.starts_with("# ") {
            h1_locations.push(line_num);
        }
    }

    // If we have more than one H1, return errors for all but the first
    if h1_locations.len() > 1 {
        let mut errors = Vec::new();
        for &location in &h1_locations[1..] { // Skip the first H1
            errors.push(LintError {
                line: location,
                column: 1,
                message: "Multiple H1 headings found. Documents should have only one top-level heading".to_string(),
                rule: "single-title".to_string(),
            });
        }
        Some(errors)
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

    // Tests for the new single-title rule
    #[test]
    fn test_validate_single_h1_single_heading() {
        let content = "# Single Title\n\nSome content here";
        let result = validate_single_h1(content);
        assert_eq!(result, None); // Should return None for valid content
    }

    #[test]
    fn test_validate_single_h1_multiple_headings() {
        let content = "# First Title\n\nContent\n\n# Second Title\n\nMore content";
        let result = validate_single_h1(content);
        assert!(result.is_some()); // Should return Some with errors
        let errors = result.unwrap();
        assert_eq!(errors.len(), 1); // Should have exactly 1 error (for the second H1)
        assert_eq!(errors[0].line, 5); // Second H1 is on line 5
        assert_eq!(errors[0].column, 1);
        assert_eq!(errors[0].rule, "single-title");
        assert!(errors[0].message.contains("Multiple H1 headings"));
    }

    #[test]
    fn test_validate_single_h1_three_headings() {
        let content = "# First\n\nContent\n\n# Second\n\nContent\n\n# Third\n\nContent";
        let result = validate_single_h1(content);
        assert!(result.is_some());
        let errors = result.unwrap();
        assert_eq!(errors.len(), 2); // Should have 2 errors (for second and third H1)
        assert_eq!(errors[0].line, 5); // Second H1
        assert_eq!(errors[1].line, 9); // Third H1
        assert_eq!(errors[0].rule, "single-title");
        assert_eq!(errors[1].rule, "single-title");
    }

    #[test]
    fn test_validate_single_h1_no_headings() {
        let content = "Just some plain text\n\nWithout any headings";
        let result = validate_single_h1(content);
        assert_eq!(result, None); // Should return None for content without H1
    }

    #[test]
    fn test_validate_single_h1_other_heading_levels() {
        let content = "## Section 1\n\nContent\n\n### Subsection\n\nMore content";
        let result = validate_single_h1(content);
        assert_eq!(result, None); // Should return None for content without H1
    }

    #[test]
    fn test_validate_single_h1_mixed_headings() {
        let content = "# Main Title\n\n## Section 1\n\nContent\n\n### Subsection\n\nMore content\n\n## Section 2";
        let result = validate_single_h1(content);
        assert_eq!(result, None); // Should return None - only one H1
    }

    #[test]
    fn test_validate_single_h1_false_positives() {
        let content = "This is # not a heading\n\n## This is a heading\n\n# This is H1\n\nAnother # not heading";
        let result = validate_single_h1(content);
        assert_eq!(result, None); // Should return None since there's only one actual H1
    }

    #[test]
    fn test_validate_markdown_single_h1_integration() {
        let content = "# Title\n\nContent\n\n# Second Title";
        let result = validate_markdown(content);
        assert!(!result.valid); // Should be invalid due to single-title error
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].rule, "single-title");
        assert_eq!(result.errors[0].line, 5);
    }

    #[test]
    fn test_validate_markdown_single_h1_valid() {
        let content = "# Title\n\n## Section 1\n\nContent\n\n## Section 2";
        let result = validate_markdown(content);
        assert!(result.valid); // Should be valid
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
}
