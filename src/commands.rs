use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use crate::linter::validate_markdown;
pub use crate::sections::{
	extract_section_content, find_section_end, find_section_range, insert_section_content,
	replace_section_content,
};
use crate::types::{
	json_output, unescape_content, Document, EditResult, Heading, JsonlEntry, LintError,
	LintResult, Match, SearchResult,
};

pub fn parse_markdown(content: &str) -> Document {
	let word_count = content.split_whitespace().count();
	let line_count = content.lines().count();
	let parsed = crate::parser::parse(content);
	let mut headings = Vec::new();

	for block in parsed.blocks {
		if let crate::parser::MarkdownBlock::Heading {
			level, text, line, ..
		} = block
		{
			headings.push(Heading { level, text, line });
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

pub fn parse_markdown_to_jsonl(content: &str) -> Vec<JsonlEntry> {
	let parsed = crate::parser::parse(content);
	let mut entries = Vec::new();
	let mut current_paragraph = String::new();

	let flush_paragraph = |current_paragraph: &mut String, entries: &mut Vec<JsonlEntry>| {
		let trimmed = current_paragraph.trim();
		if !trimmed.is_empty() {
			entries.push(JsonlEntry {
				entry_type: "paragraph".to_string(),
				content: trimmed.to_string(),
				level: None,
				language: None,
			});
			current_paragraph.clear();
		}
	};

	for block in parsed.blocks {
		match block {
			crate::parser::MarkdownBlock::Frontmatter(_) => {}
			crate::parser::MarkdownBlock::Heading { level, text, .. } => {
				flush_paragraph(&mut current_paragraph, &mut entries);
				entries.push(JsonlEntry {
					entry_type: "heading".to_string(),
					content: text,
					level: Some(level),
					language: None,
				});
			}
			crate::parser::MarkdownBlock::CodeBlock {
				language, content, ..
			} => {
				flush_paragraph(&mut current_paragraph, &mut entries);
				entries.push(JsonlEntry {
					entry_type: "code_block".to_string(),
					content,
					level: None,
					language,
				});
			}
			crate::parser::MarkdownBlock::List { items, .. } => {
				flush_paragraph(&mut current_paragraph, &mut entries);
				for item in items {
					let trimmed = item.trim();
					let item_text = if let Some(stripped) = trimmed
						.strip_prefix("- ")
						.or_else(|| trimmed.strip_prefix("* "))
						.or_else(|| trimmed.strip_prefix("+ "))
					{
						stripped
					} else if let Some(dot_pos) = trimmed.find(". ") {
						if trimmed[..dot_pos].chars().all(|c| c.is_ascii_digit()) {
							&trimmed[dot_pos + 2..]
						} else {
							trimmed
						}
					} else if let Some(paren_pos) = trimmed.find(") ") {
						if trimmed[..paren_pos].chars().all(|c| c.is_ascii_digit()) {
							&trimmed[paren_pos + 2..]
						} else {
							trimmed
						}
					} else {
						trimmed
					};
					if !item_text.trim().is_empty() {
						entries.push(JsonlEntry {
							entry_type: "paragraph".to_string(),
							content: item_text.trim().to_string(),
							level: None,
							language: None,
						});
					}
				}
			}
			crate::parser::MarkdownBlock::Table { raw, .. } => {
				flush_paragraph(&mut current_paragraph, &mut entries);
				let table_text = raw.trim();
				if !table_text.is_empty() {
					entries.push(JsonlEntry {
						entry_type: "paragraph".to_string(),
						content: table_text.to_string(),
						level: None,
						language: None,
					});
				}
			}
			crate::parser::MarkdownBlock::Html(raw) => {
				flush_paragraph(&mut current_paragraph, &mut entries);
				let html_text = raw.trim();
				if !html_text.is_empty() {
					entries.push(JsonlEntry {
						entry_type: "paragraph".to_string(),
						content: html_text.to_string(),
						level: None,
						language: None,
					});
				}
			}
			crate::parser::MarkdownBlock::Paragraph(s) => {
				let trimmed = s.trim();
				let line_text = if trimmed.starts_with('>') {
					trimmed.trim_start_matches('>').trim()
				} else {
					trimmed
				};
				if !line_text.is_empty() {
					if current_paragraph.is_empty() {
						current_paragraph.push_str(line_text);
					} else {
						current_paragraph.push(' ');
						current_paragraph.push_str(line_text);
					}
				}
			}
			crate::parser::MarkdownBlock::BlankLine => {
				flush_paragraph(&mut current_paragraph, &mut entries);
			}
			crate::parser::MarkdownBlock::HorizontalRule(_) => {
				flush_paragraph(&mut current_paragraph, &mut entries);
			}
		}
	}

	flush_paragraph(&mut current_paragraph, &mut entries);
	entries
}

pub fn cmd_read(path: &str, field: Option<&str>, content_filter: Option<&str>, human: bool) {
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
									document: None,
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
						document: None,
					},
					human
				)
			);
		}
	}
}

pub fn cmd_write(path: &str, content: &str, human: bool) {
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
		std::process::exit(1);
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
						document: None,
					},
					human
				)
			);
			std::process::exit(1);
		}
	}
}

pub fn cmd_write_section(path: &str, section_path: &str, new_content: &str, human: bool) {
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
		std::process::exit(1);
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
						std::process::exit(1);
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
					std::process::exit(1);
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
			std::process::exit(1);
		}
	}
}

pub fn cmd_append(path: &str, content: &str, human: bool) {
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

pub fn cmd_insert(path: &str, line: usize, content: &str, human: bool) {
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

pub fn cmd_delete(path: &str, line: usize, count: usize, human: bool) {
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

pub fn cmd_list(path: &str, human: bool) {
	let path = PathBuf::from(path);
	match fs::read_dir(&path) {
		Ok(entries) => {
			let ignore_list = crate::ignore::get_ignore_list_in_dir(&path);
			let mut files: Vec<String> = Vec::new();
			for entry in entries.flatten() {
				let entry_path = entry.path();
				if crate::ignore::is_ignored(&entry_path, &path, &ignore_list) {
					continue;
				}
				if let Some(ext) = entry_path.extension() {
					if ext == "md" || ext == "markdown" {
						files.push(entry_path.to_string_lossy().to_string());
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
						document: None,
					},
					human
				)
			);
		}
	}
}

pub fn cmd_ignore(path: &str, human: bool) {
	let path_buf = PathBuf::from(path);
	let list = crate::ignore::get_ignore_list_in_dir(&path_buf);
	println!("{}", json_output(&list, human));
}

pub fn cmd_search(path: &str, query: &str, human: bool) {
	match fs::read_to_string(path) {
		Ok(content) => {
			// SIMD: Accelerate case-insensitive matching for ASCII queries across all lines.
			// This avoids heap-allocating `line.to_lowercase()` for every line in the file
			// by using vectorized 16/32-byte chunks and first-byte matching to quickly skip
			// non-matching lines.
			let query_is_ascii = query.is_ascii();
			let query_lower = if query_is_ascii {
				String::new()
			} else {
				query.to_lowercase()
			};
			let mut matches = Vec::new();
			for (i, line) in content.lines().enumerate() {
				let matched = if query_is_ascii {
					crate::simd::contains_ascii_case_insensitive(line, query)
				} else {
					line.to_lowercase().contains(&query_lower)
				};
				if matched {
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
						document: None,
					},
					human
				)
			);
		}
	}
}

pub fn cmd_headings(path: &str, human: bool) {
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
						document: None,
					},
					human
				)
			);
		}
	}
}

pub fn cmd_stats(path: &str, human: bool) {
	match fs::read_to_string(path) {
		Ok(content) => {
			let mut doc = parse_markdown(&content);
			doc.path = path.to_string();
			let mut stats_obj = crate::json::JsonObject::new();
			stats_obj.insert("path", doc.path);
			stats_obj.insert("word_count", doc.word_count);
			stats_obj.insert("line_count", doc.line_count);
			stats_obj.insert("heading_count", doc.headings.len());
			println!(
				"{}",
				json_output(&crate::json::JsonValue::Object(stats_obj), human)
			);
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

pub fn cmd_to_jsonl(path: &str, human: bool) {
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
						document: None,
					},
					human
				)
			);
		}
	}
}

pub fn cmd_lint(path: &str, is_content: bool, human: bool, custom_config: Option<&str>) {
	cmd_lint_with_options(path, is_content, human, custom_config, false, None)
}

pub fn cmd_lint_with_options(
	path: &str,
	is_content: bool,
	human: bool,
	custom_config: Option<&str>,
	ignore_markdownlintrc: bool,
	cwd: Option<&str>,
) {
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

	let target = cwd.or(if is_content { None } else { Some(path) });
	let result = crate::linter::validate_markdown_for_target_with_options(
		&content,
		target,
		custom_config,
		ignore_markdownlintrc,
	);
	println!("{}", json_output(&result, human));
	if !result.valid {
		std::process::exit(1);
	}
}

pub fn cmd_lint_file(path: &str, human: bool, custom_config: Option<&str>) {
	cmd_lint_file_with_options(path, human, custom_config, false, None)
}

pub fn cmd_lint_file_with_options(
	path: &str,
	human: bool,
	custom_config: Option<&str>,
	ignore_markdownlintrc: bool,
	cwd: Option<&str>,
) {
	match fs::read_to_string(path) {
		Ok(content) => {
			let target = cwd.or(Some(path));
			let result = crate::linter::validate_markdown_for_target_with_options(
				&content,
				target,
				custom_config,
				ignore_markdownlintrc,
			);
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
					std::process::exit(1);
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

pub fn cmd_init(custom_path: Option<&str>, force: bool, human: bool) {
	match crate::config::init_config(custom_path, force) {
		Ok(path) => {
			let result = crate::config::InitConfigResult {
				success: true,
				path,
				message: "Configuration file initialized successfully".to_string(),
			};
			println!("{}", json_output(&result, human));
		}
		Err(msg) => {
			let path = custom_path.unwrap_or(".agent-md.json").to_string();
			let result = crate::config::InitConfigResult {
				success: false,
				path,
				message: msg,
			};
			println!("{}", json_output(&result, human));
			std::process::exit(1);
		}
	}
}

pub fn cmd_config(custom_path: Option<&str>, check: bool, init: bool, force: bool, human: bool) {
	cmd_config_with_options(custom_path, check, init, force, human, false, None)
}

pub fn cmd_config_with_options(
	custom_path: Option<&str>,
	check: bool,
	init: bool,
	force: bool,
	human: bool,
	ignore_markdownlintrc: bool,
	cwd: Option<&str>,
) {
	if init {
		cmd_init(custom_path.or(cwd), force, human);
		return;
	}

	// Use custom_path if provided, otherwise fall back to cwd
	let base_dir = custom_path.or(cwd);
	let status =
		crate::config::get_config_status_with_options(base_dir, !check, ignore_markdownlintrc);
	println!("{}", json_output(&status, human));
}
