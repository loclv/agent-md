use clap::{Parser, Subcommand};
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Parser as MarkdownParser, Tag, TagEnd};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

mod rules;
use crate::rules::extract_heading_level;

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

fn json_output<T: ?Sized + Serialize>(value: &T, human: bool) -> String {
    if human {
        serde_json::to_string_pretty(value).unwrap()
    } else {
        serde_json::to_string(value).unwrap()
    }
}

fn validate_markdown(content: &str) -> LintResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    let mut in_code_block = false;

    for (line_num, line) in content.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim().starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }

        if let Some(col) = rules::find_ascii_graph(line) {
            if in_code_block {
                errors.push(LintError {
                    line: line_num,
                    column: col,
                    message: "ASCII graph detected in code block. Use LLM-readable formats instead: Structured CSV, JSON, Mermaid Diagram, Numbered List with Conditions, ZON format, or simple progress indicators".to_string(),
                    rule: "no-ascii-graph".to_string(),
                });
            } else {
                warnings.push(LintWarning {
                    line: line_num,
                    column: col,
                    message: "Human-readable ASCII graph detected. Use LLM-readable formats instead: Structured CSV, JSON, Mermaid Diagram, Numbered List with Conditions, ZON format, or simple progress indicators".to_string(),
                    rule: "no-ascii-graph".to_string(),
                });
            }
        }

        if in_code_block {
            continue;
        }

        for col in rules::find_bold_text(line) {
            errors.push(LintError {
                line: line_num,
                column: col,
                message: "Bold text is not allowed for AI agents".to_string(),
                rule: "no-bold".to_string(),
            });
        }

        for issue in rules::validate_table_syntax(line) {
            match issue.severity {
                rules::Severity::Error => errors.push(LintError {
                    line: line_num,
                    column: issue.column,
                    message: issue.message,
                    rule: "simple-tables".to_string(),
                }),
                rules::Severity::Warning => warnings.push(LintWarning {
                    line: line_num,
                    column: issue.column,
                    message: issue.message,
                    rule: "simple-tables".to_string(),
                }),
            }
        }

        if let Some(issue) = rules::validate_table_trailing_spaces(line) {
            errors.push(LintError {
                line: line_num,
                column: issue.column,
                message: issue.message,
                rule: "table-trailing-spaces".to_string(),
            });
        }

        for col in rules::find_useless_link(line) {
            warnings.push(LintWarning {
                line: line_num,
                column: col,
                message:
                    "Link text should not be the same as the URL - provide meaningful link text"
                        .to_string(),
                rule: "useless-links".to_string(),
            });
        }

        if let Some(col) = rules::validate_space_indentation(line) {
            warnings.push(LintWarning {
                line: line_num,
                column: col,
                message: "Use at most 2 spaces for indentation in regular text. Code blocks are exempt from this rule.".to_string(),
                rule: "space-indentation".to_string(),
            });
        }
    }

    if let Some(heading_errors) = rules::validate_heading_structure(content) {
        errors.extend(heading_errors);
    }

    if let Some(code_block_issues) = rules::validate_code_blocks(content) {
        for issue in code_block_issues {
            warnings.push(LintWarning {
                line: issue.line,
                column: issue.column,
                message: issue.message,
                rule: issue.rule,
            });
        }
    }

    if let Some(list_issues) = rules::validate_list_formatting(content) {
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
#[command(disable_version_flag = true)]
struct Cli {
    #[arg(short = 'v', long = "version", help = "Print version information")]
    version: bool,
    #[arg(long = "human", help = "Pretty print JSON output")]
    human: bool,

    #[command(subcommand)]
    command: Option<Commands>,
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
    Fmt {
        #[arg(help = "Markdown file path to format")]
        path: String,
        #[arg(
            long,
            help = "Remove bold markers (** and __)",
            default_value = "false"
        )]
        remove_bold: bool,
        #[arg(
            long,
            help = "Compact blank lines (remove multiples)",
            default_value = "false"
        )]
        compact_blank_lines: bool,
        #[arg(
            long,
            help = "Use token-saving preset (all compact rules)",
            default_value = "false"
        )]
        token_saver: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    if cli.version {
        println!("0.1.5");
        return;
    }

    match cli.command {
        Some(Commands::Read {
            path,
            field,
            content,
        }) => cmd_read(&path, field.as_deref(), content.as_deref(), cli.human),
        Some(Commands::Write { path, content }) => cmd_write(&path, &content, cli.human),
        Some(Commands::WriteSection {
            path,
            section,
            content,
        }) => cmd_write_section(&path, &section, &content, cli.human),
        Some(Commands::Append { path, content }) => cmd_append(&path, &content, cli.human),
        Some(Commands::Insert {
            path,
            line,
            content,
        }) => cmd_insert(&path, line, &content, cli.human),
        Some(Commands::Delete { path, line, count }) => cmd_delete(&path, line, count, cli.human),
        Some(Commands::List { path }) => cmd_list(&path, cli.human),
        Some(Commands::Search { path, query }) => cmd_search(&path, &query, cli.human),
        Some(Commands::Headings { path }) => cmd_headings(&path, cli.human),
        Some(Commands::Stats { path }) => cmd_stats(&path, cli.human),
        Some(Commands::ToJsonl { path }) => cmd_to_jsonl(&path, cli.human),
        Some(Commands::Lint { path, content }) => cmd_lint(&path, content, cli.human),
        Some(Commands::LintFile { path }) => cmd_lint_file(&path, cli.human),
        Some(Commands::Fmt {
            path,
            remove_bold,
            compact_blank_lines,
            token_saver,
        }) => {
            let options = if token_saver {
                format::FormatOptions::token_saver()
            } else {
                format::FormatOptions {
                    remove_bold,
                    compact_blank_lines,
                    trim_trailing_whitespace: true,
                    collapse_spaces: false,
                    remove_horizontal_rules: false,
                    remove_emphasis: false,
                }
            };
            format::cmd_fmt(&path, cli.human, options)
        }
        None => {
            // If no command and not version, show help
            eprintln!("Usage: agent-md <COMMAND>");
            eprintln!("For more information, try '--help'.");
            std::process::exit(1);
        }
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

fn cmd_read(path: &str, field: Option<&str>, content_filter: Option<&str>, human: bool) {
    match fs::read_to_string(path) {
        Ok(content) => {
            let filtered_content = if let Some(section_name) = content_filter {
                match extract_section_content(&content, section_name) {
                    Some(section) => section,
                    None => {
                        println!(
                            "{}",
                            json_output(
                                &EditResult {
                                    success: false,
                                    message: format!("Section '{}' not found", section_name),
                                    document: None
                                },
                                human
                            )
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
                    "path" => json_output(&doc.path, human),
                    "content" => json_output(&doc.content, human),
                    "word_count" => json_output(&doc.word_count, human),
                    "line_count" => json_output(&doc.line_count, human),
                    "headings" => json_output(&doc.headings, human),
                    _ => {
                        eprintln!("Error: Invalid field '{}'. Valid fields: path, content, word_count, line_count, headings", field_name);
                        std::process::exit(1);
                    }
                }
            } else {
                json_output(&doc, human)
            };

            println!("{}", output);
        }
        Err(e) => {
            println!(
                "{}",
                json_output(
                    &EditResult {
                        success: false,
                        message: format!("Failed to read file: {}", e),
                        document: None
                    },
                    human
                )
            );
        }
    }
}

fn cmd_write(path: &str, content: &str, human: bool) {
    let content = unescape_content(content);
    let validation = validate_markdown(&content);

    if !validation.valid {
        println!(
            "{}",
            json_output(
                &EditResult {
                    success: false,
                    message: format!(
                        "Content validation failed: {} errors found",
                        validation.errors.len()
                    ),
                    document: None,
                },
                human
            )
        );
        println!("{}", json_output(&validation, human));
        return;
    }

    match fs::write(path, &content) {
        Ok(_) => {
            let mut doc = parse_markdown(&content);
            doc.path = path.to_string();
            println!(
                "{}",
                json_output(
                    &EditResult {
                        success: true,
                        message: "File written successfully".to_string(),
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
                        document: None
                    },
                    human
                )
            );
        }
    }
}

fn cmd_write_section(path: &str, section_path: &str, new_content: &str, human: bool) {
    let new_content = unescape_content(new_content);
    let validation = validate_markdown(&new_content);

    if !validation.valid {
        println!(
            "{}",
            json_output(
                &EditResult {
                    success: false,
                    message: format!(
                        "Content validation failed: {} errors found",
                        validation.errors.len()
                    ),
                    document: None,
                },
                human
            )
        );
        println!("{}", json_output(&validation, human));
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
                            json_output(
                                &EditResult {
                                    success: true,
                                    message: format!(
                                        "Section '{}' written successfully",
                                        section_path
                                    ),
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
                },
                Err(e) => {
                    println!(
                        "{}",
                        json_output(
                            &EditResult {
                                success: false,
                                message: e,
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

fn cmd_append(path: &str, content: &str, human: bool) {
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
                        json_output(
                            &EditResult {
                                success: true,
                                message: "Content appended successfully".to_string(),
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
                                document: None
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
                        document: None
                    },
                    human
                )
            );
        }
    }
}

fn cmd_insert(path: &str, line: usize, content: &str, human: bool) {
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
                        json_output(
                            &EditResult {
                                success: true,
                                message: format!("Inserted at line {}", line),
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
                                document: None
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
                        document: None
                    },
                    human
                )
            );
        }
    }
}

fn cmd_delete(path: &str, line: usize, count: usize, human: bool) {
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
                        json_output(
                            &EditResult {
                                success: true,
                                message: format!("Deleted {} lines from line {}", count, line),
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
                                document: None
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
                        document: None
                    },
                    human
                )
            );
        }
    }
}

fn cmd_list(path: &str, human: bool) {
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
            println!("{}", json_output(&files, human));
        }
        Err(e) => {
            println!(
                "{}",
                json_output(
                    &EditResult {
                        success: false,
                        message: format!("Failed to list directory: {}", e),
                        document: None
                    },
                    human
                )
            );
        }
    }
}

fn cmd_search(path: &str, query: &str, human: bool) {
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
                json_output(
                    &SearchResult {
                        query: query.to_string(),
                        total: matches.len(),
                        matches,
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
                        message: format!("Failed to read file: {}", e),
                        document: None
                    },
                    human
                )
            );
        }
    }
}

fn cmd_headings(path: &str, human: bool) {
    match fs::read_to_string(path) {
        Ok(content) => {
            let doc = parse_markdown(&content);
            println!("{}", json_output(&doc.headings, human));
        }
        Err(e) => {
            println!(
                "{}",
                json_output(
                    &EditResult {
                        success: false,
                        message: format!("Failed to read file: {}", e),
                        document: None
                    },
                    human
                )
            );
        }
    }
}

fn cmd_stats(path: &str, human: bool) {
    match fs::read_to_string(path) {
        Ok(content) => {
            let mut doc = parse_markdown(&content);
            doc.path = path.to_string();
            println!(
                "{}",
                json_output(
                    &serde_json::json!({
                        "path": doc.path,
                        "word_count": doc.word_count,
                        "line_count": doc.line_count,
                        "heading_count": doc.headings.len(),
                    }),
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
                        message: format!("Failed to read file: {}", e),
                        document: None
                    },
                    human
                )
            );
        }
    }
}

fn cmd_to_jsonl(path: &str, human: bool) {
    match fs::read_to_string(path) {
        Ok(content) => {
            let entries = parse_markdown_to_jsonl(&content);
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            for entry in entries {
                writeln!(handle, "{}", json_output(&entry, human)).unwrap();
            }
        }
        Err(e) => {
            println!(
                "{}",
                json_output(
                    &EditResult {
                        success: false,
                        message: format!("Failed to read file: {}", e),
                        document: None
                    },
                    human
                )
            );
        }
    }
}

fn cmd_lint(path: &str, is_content: bool, human: bool) {
    let content = if is_content {
        unescape_content(path)
    } else {
        match fs::read_to_string(path) {
            Ok(content) => content,
            Err(e) => {
                println!(
                    "{}",
                    json_output(
                        &LintResult {
                            valid: false,
                            errors: vec![LintError {
                                line: 0,
                                column: 0,
                                message: format!("Failed to read file: {}", e),
                                rule: "file-read".to_string(),
                            }],
                            warnings: vec![],
                        },
                        human
                    )
                );
                return;
            }
        }
    };

    let result = validate_markdown(&content);
    println!("{}", json_output(&result, human));
}

fn cmd_lint_file(path: &str, human: bool) {
    match fs::read_to_string(path) {
        Ok(content) => {
            let result = validate_markdown(&content);
            println!("{}", json_output(&result, human));

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

mod format;
#[cfg(test)]
mod tests;
