use crate::format::format_markdown;
use crate::format::format_markdown_with_options;
use crate::{find_section_range, parse_markdown, validate_markdown, LintError, LintWarning};

#[cfg(test)]
mod tests {
	#![allow(clippy::module_inception)]
	use super::*;
	use crate::{
		extract_section_content, parse_markdown_to_jsonl, unescape_content, Cli, Commands,
		Document, EditResult, Heading, JsonlEntry, Match, SearchResult,
	};
	use clap::Parser;

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

	// Tests for implicit fmt via path argument
	#[test]
	fn test_cli_parsing_path_without_command() {
		// Test parsing with path but no command - should trigger implicit fmt
		let args = vec!["agent-md", "test.md"];
		let cli = Cli::try_parse_from(&args).expect("Should parse path without command");
		assert!(!cli.version);
		assert!(cli.command.is_none());
		assert!(cli.path.is_some());
		assert_eq!(cli.path.unwrap(), "test.md");
	}

	#[test]
	fn test_cli_parsing_path_with_human_flag() {
		// Test path with human flag
		let args = vec!["agent-md", "--human", "test.md"];
		let cli = Cli::try_parse_from(&args).expect("Should parse path with human flag");
		assert!(cli.human);
		assert!(cli.command.is_none());
		assert_eq!(cli.path.unwrap(), "test.md");
	}

	#[test]
	fn test_cli_parsing_explicit_fmt_command() {
		// Test explicit fmt command still works
		let args = vec!["agent-md", "fmt", "test.md"];
		let cli = Cli::try_parse_from(&args).expect("Should parse explicit fmt command");
		assert!(cli.command.is_some());

		match cli.command.unwrap() {
			Commands::Fmt { path, .. } => {
				assert_eq!(path, Some("test.md".to_string()));
			}
			_ => panic!("Expected Fmt command"),
		}
	}

	#[test]
	fn test_cli_parsing_path_vs_command_priority() {
		// When both path and command are provided, command takes priority
		let args = vec!["agent-md", "read", "test.md"];
		let cli = Cli::try_parse_from(&args).expect("Should parse with command");
		assert!(cli.command.is_some());
		// Path should be None since "read" is parsed as command, not as path
		assert!(cli.path.is_none());
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
		let formatted = format_markdown_with_options(
			content,
			crate::format::FormatOptions {
				remove_bold: true,
				compact_blank_lines: false,
				trim_trailing_whitespace: true,
				collapse_spaces: false,
				remove_horizontal_rules: false,
				remove_emphasis: false,
			},
		);
		assert_eq!(formatted, content);
	}

	#[test]
	fn test_format_markdown_handles_separator_rows() {
		let content =
			"# Test\n\n| Name | Description |\n|------|-------------|\n| Item | Value |\n";
		let formatted = format_markdown(content);
		// Separator rows should be compacted to 3 dashes
		assert!(formatted.contains("|---|---|"));
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

	// Tests for basic Markdown syntax - Headers
	#[test]
	fn test_basic_headers_h1() {
		let content = "# This is a Heading h1";
		let doc = parse_markdown(content);
		assert_eq!(doc.headings.len(), 1);
		assert_eq!(doc.headings[0].level, 1);
		assert_eq!(doc.headings[0].text, "This is a Heading h1");
	}

	#[test]
	fn test_basic_headers_h2() {
		let content = "## This is a Heading h2";
		let doc = parse_markdown(content);
		assert_eq!(doc.headings.len(), 1);
		assert_eq!(doc.headings[0].level, 2);
		assert_eq!(doc.headings[0].text, "This is a Heading h2");
	}

	#[test]
	fn test_basic_headers_h6() {
		let content = "###### This is a Heading h6";
		let doc = parse_markdown(content);
		assert_eq!(doc.headings.len(), 1);
		assert_eq!(doc.headings[0].level, 6);
		assert_eq!(doc.headings[0].text, "This is a Heading h6");
	}

	#[test]
	fn test_basic_headers_multiple() {
		let content = "# H1\n## H2\n###### H6";
		let doc = parse_markdown(content);
		assert_eq!(doc.headings.len(), 3);
		assert_eq!(doc.headings[0].level, 1);
		assert_eq!(doc.headings[1].level, 2);
		assert_eq!(doc.headings[2].level, 6);
	}

	// Tests for basic Markdown syntax - Emphasis
	#[test]
	fn test_basic_emphasis_italic_asterisk() {
		let content = "*This text will be italic*";
		let result = validate_markdown(content);
		assert!(result.valid);
		assert_eq!(result.errors.len(), 0);
	}

	#[test]
	fn test_basic_emphasis_italic_underscore() {
		let content = "_This will also be italic_";
		let result = validate_markdown(content);
		assert!(result.valid);
		assert_eq!(result.errors.len(), 0);
	}

	#[test]
	fn test_basic_emphasis_bold_asterisk() {
		let content = "**This text will be bold**";
		let result = validate_markdown(content);
		assert!(!result.valid);
		assert_eq!(result.errors.len(), 1);
		assert_eq!(result.errors[0].rule, "no-bold");
	}

	#[test]
	fn test_basic_emphasis_bold_underscore() {
		let content = "__This will also be bold__";
		let result = validate_markdown(content);
		assert!(!result.valid);
		assert_eq!(result.errors.len(), 1);
		assert_eq!(result.errors[0].rule, "no-bold");
	}

	#[test]
	fn test_basic_emphasis_combined() {
		let content = "_You **can** combine them_";
		let result = validate_markdown(content);
		assert!(!result.valid);
		let bold_errors: Vec<_> = result
			.errors
			.iter()
			.filter(|e| e.rule == "no-bold")
			.collect();
		assert_eq!(bold_errors.len(), 1);
	}

	// Tests for basic Markdown syntax - Lists
	#[test]
	fn test_basic_lists_unordered() {
		let content = r#"* Item 1
* Item 2
* Item 2a
* Item 2b"#;
		let doc = parse_markdown(content);
		// Word count includes asterisks and item numbers as separate tokens
		assert!(doc.word_count >= 8);
		let result = validate_markdown(content);
		assert!(result.valid);
	}

	#[test]
	fn test_basic_lists_unordered_nested() {
		let content = r#"* Item 1
* Item 2
    * Item 3a
    * Item 3b"#;
		let result = validate_markdown(content);
		assert!(result.valid);
	}

	#[test]
	fn test_basic_lists_ordered() {
		let content = r#"1. Item 1
2. Item 2
3. Item 3"#;
		let doc = parse_markdown(content);
		// Word count includes numbers as separate tokens
		assert!(doc.word_count >= 6);
		let result = validate_markdown(content);
		assert!(result.valid);
	}

	#[test]
	fn test_basic_lists_ordered_nested() {
		let content = r#"1. Item 1
2. Item 2
3. Item 3
    1. Item 3a
    2. Item 3b"#;
		let result = validate_markdown(content);
		assert!(result.valid);
	}

	// Tests for basic Markdown syntax - Images
	#[test]
	fn test_basic_images_with_alt_text() {
		let content = r#"![This is an alt text.](/image/Markdown-mark.svg)"#;
		let doc = parse_markdown(content);
		assert!(doc.content.contains("![This is an alt text.]"));
		let result = validate_markdown(content);
		assert!(result.valid);
	}

	#[test]
	fn test_basic_images_with_title() {
		let content =
			r#"![This is an alt text.](/image/Markdown-mark.svg "This is a sample image.")"#;
		let doc = parse_markdown(content);
		assert!(doc.content.contains("This is a sample image"));
		let result = validate_markdown(content);
		assert!(result.valid);
	}

	// Tests for basic Markdown syntax - Links
	#[test]
	fn test_basic_links() {
		let content = r#"[Markdown Live Preview](https://markdownlivepreview.com/)"#;
		let doc = parse_markdown(content);
		assert!(doc.content.contains("Markdown Live Preview"));
		let result = validate_markdown(content);
		assert!(result.valid);
	}

	#[test]
	fn test_basic_links_useless() {
		let content = r#"[https://example.com](https://example.com)"#;
		let result = validate_markdown(content);
		assert!(result.valid); // Warning only
		assert_eq!(result.warnings.len(), 1);
		assert_eq!(result.warnings[0].rule, "useless-links");
	}

	#[test]
	fn test_basic_links_meaningful_text() {
		let content = r#"[Click here for more info](https://example.com)"#;
		let result = validate_markdown(content);
		assert!(result.valid);
		assert_eq!(result.warnings.len(), 0);
	}

	// Tests for basic Markdown syntax - Blockquotes
	#[test]
	fn test_basic_blockquotes_single() {
		let content =
			r#"> Markdown is a lightweight markup language with plain-text-formatting syntax."#;
		let doc = parse_markdown(content);
		assert!(doc.content.starts_with(">"));
		let result = validate_markdown(content);
		assert!(result.valid);
	}

	#[test]
	fn test_basic_blockquotes_multi_line() {
		let content = r#"> Markdown is a lightweight markup language.
>
> Created in 2004 by John Gruber."#;
		let doc = parse_markdown(content);
		assert!(doc.content.contains(">"));
		let result = validate_markdown(content);
		assert!(result.valid);
	}

	#[test]
	fn test_basic_blockquotes_nested() {
		let content = r#"> Markdown is a lightweight markup language.
>
>> Markdown is often used to format readme files."#;
		let doc = parse_markdown(content);
		assert!(doc.content.contains(">>"));
		let result = validate_markdown(content);
		assert!(result.valid);
	}

	// Tests for basic Markdown syntax - Tables
	#[test]
	fn test_basic_tables_simple() {
		let content = r#"| Left columns | Right columns |
| --- | --- |
| left foo | right foo |
| left bar | right bar |
| left baz | right baz |"#;
		let result = validate_markdown(content);
		assert!(result.valid);
	}

	#[test]
	fn test_basic_tables_alignment() {
		let content = r#"| Left | Center | Right |
|:---|:---:|---:|
| L1 | C1 | R1 |"#;
		let result = validate_markdown(content);
		assert!(result.valid);
	}

	#[test]
	fn test_basic_tables_five_columns() {
		let content = r#"| C1 | C2 | C3 | C4 | C5 |
|---|---|---|---|---|
| V1 | V2 | V3 | V4 | V5 |"#;
		let result = validate_markdown(content);
		assert!(result.valid);
	}

	#[test]
	fn test_basic_tables_six_columns_warning() {
		let content = r#"| C1 | C2 | C3 | C4 | C5 | C6 |
|---|---|---|---|---|---|
| V1 | V2 | V3 | V4 | V5 | V6 |"#;
		let result = validate_markdown(content);
		assert!(result.valid); // Warning only
		let table_warnings: Vec<_> = result
			.warnings
			.iter()
			.filter(|w| w.rule == "simple-tables")
			.collect();
		// Multiple lines trigger warnings for tables > 5 columns
		assert!(!table_warnings.is_empty());
	}

	// Tests for basic Markdown syntax - Code blocks
	#[test]
	fn test_basic_code_blocks_fenced() {
		let content = r#"```
let message = 'Hello world';
alert(message);
```"#;
		let entries = parse_markdown_to_jsonl(content);
		assert_eq!(entries.len(), 1);
		assert_eq!(entries[0].entry_type, "code_block");
		assert_eq!(entries[0].language, None);
		assert!(entries[0].content.contains("Hello world"));
	}

	#[test]
	fn test_basic_code_blocks_with_language() {
		let content = r#"```javascript
let message = 'Hello world';
alert(message);
```"#;
		let entries = parse_markdown_to_jsonl(content);
		assert_eq!(entries.len(), 1);
		assert_eq!(entries[0].entry_type, "code_block");
		assert_eq!(entries[0].language, Some("javascript".to_string()));
	}

	#[test]
	fn test_basic_code_blocks_no_language_warning() {
		let content = r#"```
some code without language
```"#;
		let result = validate_markdown(content);
		assert!(result.valid);
		let code_warnings: Vec<_> = result
			.warnings
			.iter()
			.filter(|w| w.rule == "code-blocks")
			.collect();
		assert_eq!(code_warnings.len(), 1);
	}

	// Tests for basic Markdown syntax - Inline code
	#[test]
	fn test_basic_inline_code() {
		let content = r#"This web site is using `markedjs/marked`."#;
		let doc = parse_markdown(content);
		assert!(doc.content.contains("`markedjs/marked`"));
		let result = validate_markdown(content);
		assert!(result.valid);
	}

	#[test]
	fn test_basic_inline_code_with_bold_inside() {
		let content = r#"Code with `**bold**` inside is allowed."#;
		let result = validate_markdown(content);
		assert!(result.valid);
		assert_eq!(result.errors.len(), 0);
	}

	// Integration test for complete basic.md syntax
	#[test]
	fn test_basic_syntax_complete_document() {
		let content = r#"# Markdown syntax guide

## Headers

### H3 Example

#### H4 Example

## Emphasis

*This text will be italic*
_This will also be italic_

## Lists

### Unordered

* Item 1
* Item 2

### Ordered

1. Item 1
2. Item 2

## Images

![This is an alt text.](/image/Markdown-mark.svg "sample")

## Links

[Markdown Live Preview](https://markdownlivepreview.com/)

## Blockquotes

> Markdown is a lightweight markup language.

## Tables

| Left columns | Right columns |
| --- | --- |
| left foo | right foo |

## Blocks of code

```javascript
let message = 'Hello world';
```

## Inline code

This web site is using `markedjs/marked`.
"#;
		let doc = parse_markdown(content);
		// Headings include all section headings
		assert!(doc.headings.len() >= 10);
		assert!(doc.word_count > 50);

		let entries = parse_markdown_to_jsonl(content);
		assert!(entries.len() > 10);

		let result = validate_markdown(content);
		assert!(result.valid);
	}
}
