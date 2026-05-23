use crate::rules;
use crate::types::{LintError, LintResult, LintWarning};
use std::fs;

pub fn get_markdownlint_config() -> Option<serde_json::Value> {
	if let Ok(content) = fs::read_to_string(".markdownlint.json") {
		serde_json::from_str(&content).ok()
	} else {
		None
	}
}

pub fn validate_markdown(content: &str) -> LintResult {
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

	let mut blanks_around_headings = true;
	if let Some(config) = get_markdownlint_config() {
		if let Some(val) = config.get("blanks-around-headings") {
			if val.is_boolean() {
				blanks_around_headings = val.as_bool().unwrap();
			}
		}
	}

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

	if let Some(heading_issues) = rules::validate_heading_structure(content, blanks_around_headings)
	{
		for issue in heading_issues {
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
