use crate::rules;
use crate::types::{LintError, LintResult, LintWarning};

use crate::config::{get_config, resolve_config, ResolvedConfig};

/// Resolve the full configuration from the default config file location.
pub fn get_resolved_config() -> ResolvedConfig {
	resolve_config(get_config(None).as_ref())
}

/// Resolve the full configuration from an optional custom config path or directory.
pub fn get_resolved_config_custom(custom_path: Option<&str>) -> ResolvedConfig {
	resolve_config(get_config(custom_path).as_ref())
}

pub fn get_markdownlint_config() -> Option<serde_json::Value> {
	get_config(None)
}

/// Validate markdown content using the given resolved configuration.
pub fn validate_markdown_with_config(content: &str, config: &ResolvedConfig) -> LintResult {
	if let Some(start_line) = rules::find_unclosed_code_block(content) {
		return LintResult {
			valid: false,
			errors: vec![LintError {
				line: start_line,
				column: 1,
				message: format!(
					"Code block starting at line {} is missing a closing fence",
					start_line
				),
				rule: "code-blocks".to_string(),
			}],
			warnings: vec![],
		};
	}

	let mut errors = Vec::new();
	let mut warnings = Vec::new();

	let mut in_code_block = false;

	for (line_num, line) in content.lines().enumerate() {
		let line_num = line_num + 1;

		if line.trim().starts_with("```") {
			in_code_block = !in_code_block;
			continue;
		}

		// no-hard-tabs rule
		if config.no_hard_tabs {
			if let Some(col) = line.find('\t') {
				if !in_code_block {
					errors.push(LintError {
						line: line_num,
						column: col + 1,
						message: "Hard tabs detected. Use spaces for indentation.".to_string(),
						rule: "no-hard-tabs".to_string(),
					});
				}
			}
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

		// line-length check (only applies outside code blocks)
		if config.line_length && config.max_line_length > 0 {
			let char_count = line.chars().count() as u64;
			if char_count > config.max_line_length {
				warnings.push(LintWarning {
					line: line_num,
					column: config.max_line_length as usize + 1,
					message: format!(
						"Line length {} exceeds maximum of {}",
						char_count, config.max_line_length
					),
					rule: "line-length".to_string(),
				});
			}
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

	if let Some(heading_issues) =
		rules::validate_heading_structure(content, config.blanks_around_headings)
	{
		for issue in heading_issues {
			// Respect first-line-heading config
			if !config.first_line_heading && issue.rule == "first-line-h1" {
				continue;
			}
			// Respect no-duplicate-headings config (checks both plural and singular aliases)
			if (!config.no_duplicate_headings || !config.no_duplicate_heading)
				&& issue.rule == "no-duplicate-headings"
			{
				continue;
			}

			if issue.is_error {
				errors.push(LintError {
					line: issue.line,
					column: issue.column,
					message: issue.message,
					rule: issue.rule,
				});
			} else {
				warnings.push(LintWarning {
					line: issue.line,
					column: issue.column,
					message: issue.message,
					rule: issue.rule,
				});
			}
		}
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

	for issue in rules::validate_whitespace(content) {
		// Respect no-hard-tabs config for whitespace rule
		if !config.no_hard_tabs && issue.rule == "no-hard-tabs" {
			continue;
		}
		warnings.push(LintWarning {
			line: issue.line,
			column: issue.column,
			message: issue.message,
			rule: issue.rule,
		});
	}

	LintResult {
		valid: errors.is_empty(),
		errors,
		warnings,
	}
}

/// Validate markdown content using the default configuration.
pub fn validate_markdown(content: &str) -> LintResult {
	let config = get_resolved_config();
	validate_markdown_with_config(content, &config)
}

/// Validate markdown content using an optional custom configuration path or directory.
pub fn validate_markdown_with_custom_config(
	content: &str,
	custom_path: Option<&str>,
) -> LintResult {
	let config = get_resolved_config_custom(custom_path);
	validate_markdown_with_config(content, &config)
}

/// Validate markdown content using configuration resolved for a specific target path.
pub fn validate_markdown_for_target(
	content: &str,
	target_path: Option<&str>,
	custom_path: Option<&str>,
) -> LintResult {
	let config =
		resolve_config(crate::config::get_config_for_target(target_path, custom_path).as_ref());
	validate_markdown_with_config(content, &config)
}

#[cfg(test)]
mod tests {
	use super::*;

	fn default_config() -> ResolvedConfig {
		ResolvedConfig::default()
	}

	// --- no-hard-tabs config ---

	#[test]
	fn test_linter_hard_tabs_enabled() {
		let mut config = default_config();
		config.no_hard_tabs = true;
		let result = validate_markdown_with_config("# Title\n\n\tHard tab content\n", &config);
		assert!(!result.valid);
		assert!(result.errors.iter().any(|e| e.rule == "no-hard-tabs"));
	}

	#[test]
	fn test_linter_hard_tabs_disabled() {
		let mut config = default_config();
		config.no_hard_tabs = false;
		let result = validate_markdown_with_config("# Title\n\n\tHard tab content\n", &config);
		assert!(result.errors.iter().all(|e| e.rule != "no-hard-tabs"));
	}

	#[test]
	fn test_linter_hard_tabs_ignored_in_code_block() {
		let mut config = default_config();
		config.no_hard_tabs = true;
		let content = "# Title\n\n```\n\thard tab in code\n```\n";
		let result = validate_markdown_with_config(content, &config);
		assert!(result.errors.iter().all(|e| e.rule != "no-hard-tabs"));
	}

	// --- first-line-heading config ---

	#[test]
	fn test_linter_first_line_heading_enabled() {
		let mut config = default_config();
		config.first_line_heading = true;
		let content = "## Not H1 first\n\nContent\n";
		let result = validate_markdown_with_config(content, &config);
		assert!(result.warnings.iter().any(|w| w.rule == "first-line-h1"));
	}

	#[test]
	fn test_linter_first_line_heading_disabled() {
		let mut config = default_config();
		config.first_line_heading = false;
		let content = "## Not H1 first\n\nContent\n";
		let result = validate_markdown_with_config(content, &config);
		assert!(!result.warnings.iter().any(|w| w.rule == "first-line-h1"));
	}

	#[test]
	fn test_linter_first_line_heading_disabled_with_valid_doc() {
		let mut config = default_config();
		config.first_line_heading = false;
		let content = "## Section\n\nContent\n";
		let result = validate_markdown_with_config(content, &config);
		// No first-line-h1 warning when disabled
		assert!(!result.warnings.iter().any(|w| w.rule == "first-line-h1"));
	}

	// --- no-duplicate-headings config ---

	#[test]
	fn test_linter_duplicate_headings_enabled() {
		let mut config = default_config();
		config.no_duplicate_headings = true;
		let content = "# Title\n\n## Section\n\n## Section\n";
		let result = validate_markdown_with_config(content, &config);
		assert!(result
			.warnings
			.iter()
			.any(|w| w.rule == "no-duplicate-headings"));
	}

	#[test]
	fn test_linter_duplicate_headings_disabled() {
		let mut config = default_config();
		config.no_duplicate_headings = false;
		let content = "# Title\n\n## Section\n\n## Section\n";
		let result = validate_markdown_with_config(content, &config);
		assert!(!result
			.warnings
			.iter()
			.any(|w| w.rule == "no-duplicate-headings"));
	}

	#[test]
	fn test_linter_duplicate_headings_disabled_via_singular() {
		let mut config = default_config();
		config.no_duplicate_heading = false;
		config.no_duplicate_headings = true; // singular flag should still disable rule
		let content = "# Title\n\n## Section\n\n## Section\n";
		let result = validate_markdown_with_config(content, &config);
		assert!(!result
			.warnings
			.iter()
			.any(|w| w.rule == "no-duplicate-headings"));
	}

	// --- blanks-around-headings config ---

	#[test]
	fn test_linter_blanks_around_headings_enabled() {
		let mut config = default_config();
		config.blanks_around_headings = true;
		let content = "# Title\nContent right after\n";
		let result = validate_markdown_with_config(content, &config);
		assert!(result
			.warnings
			.iter()
			.any(|w| w.rule == "blanks-around-headings"));
	}

	#[test]
	fn test_linter_blanks_around_headings_disabled() {
		let mut config = default_config();
		config.blanks_around_headings = false;
		let content = "# Title\nContent right after\n";
		let result = validate_markdown_with_config(content, &config);
		assert!(!result
			.warnings
			.iter()
			.any(|w| w.rule == "blanks-around-headings"));
	}

	// --- line-length config ---

	#[test]
	fn test_linter_line_length_enabled() {
		let mut config = default_config();
		config.line_length = true;
		config.max_line_length = 10;
		let content = "# Title\n\nShort line\n";
		let result = validate_markdown_with_config(content, &config);
		// "# Title" is 7 chars, "Short line" is 10 chars, both under limit
		assert!(!result.warnings.iter().any(|w| w.rule == "line-length"));
	}

	#[test]
	fn test_linter_line_length_exceeded() {
		let mut config = default_config();
		config.line_length = true;
		config.max_line_length = 5;
		let content = "# Title\n\nShort\n";
		let result = validate_markdown_with_config(content, &config);
		// "# Title" is 7 chars which exceeds 5
		assert!(result.warnings.iter().any(|w| w.rule == "line-length"));
	}

	#[test]
	fn test_linter_line_length_disabled() {
		let mut config = default_config();
		config.line_length = false;
		config.max_line_length = 5;
		let content = "# A very long line that exceeds the limit\n\nContent\n";
		let result = validate_markdown_with_config(content, &config);
		assert!(!result.warnings.iter().any(|w| w.rule == "line-length"));
	}

	#[test]
	fn test_linter_line_length_zero_max() {
		let mut config = default_config();
		config.line_length = true;
		config.max_line_length = 0;
		let content = "# Title\n\nA very long line\n";
		let result = validate_markdown_with_config(content, &config);
		// max_line_length=0 means disabled
		assert!(!result.warnings.iter().any(|w| w.rule == "line-length"));
	}

	#[test]
	fn test_linter_line_length_ignored_in_code_block() {
		let mut config = default_config();
		config.line_length = true;
		config.max_line_length = 10;
		let content = "# Title\n\n```text\nThis is a very long line inside a code block\n```\n";
		let result = validate_markdown_with_config(content, &config);
		// Code blocks are exempt from line length
		assert!(!result.warnings.iter().any(|w| w.rule == "line-length"));
	}

	#[test]
	fn test_linter_line_length_counts_unicode_characters() {
		let mut config = default_config();
		config.line_length = true;
		config.max_line_length = 10;
		// 8 Vietnamese characters with diacritics - each character is 2-3 bytes, total > 15 bytes
		// But in characters, it is 8 characters, which is <= 10.
		let content = "# Tiêu đề\n\nXin chào\n";
		let result = validate_markdown_with_config(content, &config);
		assert!(!result.warnings.iter().any(|w| w.rule == "line-length"));
	}

	// --- no-bold still works with config ---

	#[test]
	fn test_linter_bold_always_detected() {
		let config = default_config();
		let content = "# Title\n\nThis has **bold** text.\n";
		let result = validate_markdown_with_config(content, &config);
		assert!(!result.valid);
		assert!(result.errors.iter().any(|e| e.rule == "no-bold"));
	}

	// --- no-ascii-graph works with config ---

	#[test]
	fn test_linter_ascii_graph_warning() {
		let config = default_config();
		let content = "# Title\n\ngraph: A -> B\n";
		let result = validate_markdown_with_config(content, &config);
		assert!(result.warnings.iter().any(|w| w.rule == "no-ascii-graph"));
	}

	#[test]
	fn test_linter_ascii_graph_error_in_code_block() {
		let config = default_config();
		let content = "# Title\n\n```text\ngraph: A -> B\n```\n";
		let result = validate_markdown_with_config(content, &config);
		assert!(!result.valid);
		assert!(result.errors.iter().any(|e| e.rule == "no-ascii-graph"));
	}

	// --- combined config options ---

	#[test]
	fn test_linter_multiple_config_options() {
		let mut config = default_config();
		config.no_hard_tabs = false;
		config.first_line_heading = false;
		config.blanks_around_headings = false;
		config.line_length = false;

		// Content with tabs, no H1 first, no blanks around headings, long lines
		let content =
			"\tTabbed line\n## Section\nContent\nAnother long line that would exceed a limit\n";
		let result = validate_markdown_with_config(content, &config);

		// Tabs should not be flagged because no_hard_tabs is disabled
		assert!(!result.valid || !result.errors.iter().any(|e| e.rule == "no-hard-tabs"));
		// first-line-h1 should not be flagged
		assert!(!result.warnings.iter().any(|w| w.rule == "first-line-h1"));
		// blanks-around-headings should not be flagged
		assert!(!result
			.warnings
			.iter()
			.any(|w| w.rule == "blanks-around-headings"));
		// line-length should not be flagged
		assert!(!result.warnings.iter().any(|w| w.rule == "line-length"));
	}

	// --- validate_markdown (default config) ---

	#[test]
	fn test_validate_markdown_uses_default_config() {
		let content = "# Title\n\nContent\n";
		let result = validate_markdown(content);
		assert!(result.valid);
		// Should use default ResolvedConfig (no-hard-tabs=true, etc.)
	}

	// --- unclosed code block ---

	#[test]
	fn test_linter_unclosed_code_block() {
		let config = default_config();
		let content = "# Title\n\n```\ncode\n";
		let result = validate_markdown_with_config(content, &config);
		assert!(!result.valid);
		assert!(result.errors.len() == 1);
		assert!(result.errors[0].rule == "code-blocks");
		assert!(result.errors[0].message.contains("missing a closing fence"));
	}

	// --- whitespace rules respect config ---

	#[test]
	fn test_linter_whitespace_tabs_warning_disabled() {
		let mut config = default_config();
		config.no_hard_tabs = false;
		let content = "# Title\n\nText\n";
		let result = validate_markdown_with_config(content, &config);
		// No tabs in content, so no whitespace warning either
		assert!(!result.warnings.iter().any(|w| w.rule == "no-hard-tabs"));
	}

	// --- get_resolved_config ---

	#[test]
	fn test_resolve_config_returns_default_when_none() {
		let config = crate::config::resolve_config(None);
		// Should return defaults (no panic)
		assert!(config.blanks_around_headings);
		assert!(config.no_hard_tabs);
		assert!(!config.line_length);
	}
}
