mod blockquotes;
mod bold_tables;
mod code_blocks;
pub mod frontmatter;
mod tables;

mod io;
mod lines;
mod options;

#[cfg(test)]
mod tests;

pub use io::{cmd_fmt, cmd_fmt_stdin};
pub use lines::is_horizontal_rule;
pub use options::FormatOptions;

#[allow(dead_code)]
pub fn format_markdown(content: &str) -> String {
	format_markdown_with_options(content, FormatOptions::default())
}

pub fn format_markdown_with_options(content: &str, options: FormatOptions) -> String {
	format_markdown_structured(content, options)
}

fn ensure_newlines(parts: &mut Vec<String>, count: usize, compact: bool) {
	if parts.is_empty() {
		return;
	}

	let mut current_newlines = 0;
	// Check from the end of parts
	for part in parts.iter().rev() {
		if part == "\n" {
			current_newlines += 1;
		} else {
			// Count trailing newlines in the part itself
			let trailing = part.chars().rev().take_while(|&c| c == '\n').count();
			current_newlines += trailing;
			break;
		}
	}

	let target = if compact {
		std::cmp::min(count, 2)
	} else {
		count
	};

	while current_newlines < target {
		parts.push("\n".to_string());
		current_newlines += 1;
	}
}

pub fn format_markdown_structured(content: &str, options: FormatOptions) -> String {
	if content.chars().all(|c| c == '\n') && !content.is_empty() {
		let n = content.len();
		if options.compact_blank_lines {
			return "\n".repeat(std::cmp::min(n, 2));
		} else {
			return content.to_string();
		}
	}

	let parsed = crate::parser::parse(content);
	let mut formatted_parts = Vec::new();
	let mut last_was_frontmatter = false;

	for (idx, block) in parsed.blocks.iter().enumerate() {
		match block {
			crate::parser::MarkdownBlock::Frontmatter(s) => {
				formatted_parts.push(s.clone());
				last_was_frontmatter = true;
			}
			crate::parser::MarkdownBlock::Heading { level, text, .. } => {
				let prev_is_blank_or_fm = idx == 0
					|| last_was_frontmatter
					|| matches!(
						parsed.blocks[idx - 1],
						crate::parser::MarkdownBlock::BlankLine
					);
				if options.blanks_around_headings && !prev_is_blank_or_fm {
					ensure_newlines(&mut formatted_parts, 2, options.compact_blank_lines);
				} else {
					ensure_newlines(&mut formatted_parts, 1, options.compact_blank_lines);
				}
				let mut h = "#".repeat(*level as usize);
				h.push(' ');
				h.push_str(text);
				h.push('\n');
				formatted_parts.push(h);
				let next_is_blank = idx + 1 < parsed.blocks.len()
					&& matches!(
						parsed.blocks[idx + 1],
						crate::parser::MarkdownBlock::BlankLine
					);
				if options.blanks_around_headings && !next_is_blank {
					ensure_newlines(&mut formatted_parts, 2, options.compact_blank_lines);
				} else {
					ensure_newlines(&mut formatted_parts, 1, options.compact_blank_lines);
				}
				last_was_frontmatter = false;
			}
			crate::parser::MarkdownBlock::CodeBlock {
				language,
				content: cb_content,
				..
			} => {
				if options.blanks_around_fences {
					ensure_newlines(&mut formatted_parts, 2, options.compact_blank_lines);
				} else {
					ensure_newlines(&mut formatted_parts, 1, options.compact_blank_lines);
				}

				let mut cb = String::from("```");
				if let Some(lang) = language {
					cb.push_str(lang);
				}
				cb.push('\n');

				if let Some(lang) = language {
					if code_blocks::is_shell_language(lang) {
						for line in cb_content.lines() {
							cb.push_str(&code_blocks::collapse_spaces_before_comment(line));
							cb.push('\n');
						}
					} else if lang == "markdown" || lang == "md" {
						cb.push_str(&format_markdown_structured(cb_content, options.clone()));
						if !cb_content.ends_with('\n') {
							cb.push('\n');
						}
					} else {
						cb.push_str(cb_content);
					}
				} else {
					cb.push_str(cb_content);
				}

				if !cb.ends_with('\n') {
					cb.push('\n');
				}
				cb.push_str("```\n");
				formatted_parts.push(cb);

				if options.blanks_around_fences {
					ensure_newlines(&mut formatted_parts, 2, options.compact_blank_lines);
				}
				last_was_frontmatter = false;
			}
			crate::parser::MarkdownBlock::Table { raw, .. } => {
				ensure_newlines(&mut formatted_parts, 1, options.compact_blank_lines);
				for line in raw.lines() {
					formatted_parts.push(lines::process_markdown_line(line, &options, false));
					formatted_parts.push("\n".to_string());
				}
				last_was_frontmatter = false;
			}
			crate::parser::MarkdownBlock::List { items, .. } => {
				if options.blanks_around_lists {
					ensure_newlines(&mut formatted_parts, 2, options.compact_blank_lines);
				} else {
					ensure_newlines(&mut formatted_parts, 1, options.compact_blank_lines);
				}

				let formatted_items = lines::format_list_items(items);
				for item in formatted_items {
					formatted_parts.push(lines::process_markdown_line(&item, &options, false));
					formatted_parts.push("\n".to_string());
				}

				if options.blanks_around_lists {
					ensure_newlines(&mut formatted_parts, 2, options.compact_blank_lines);
				}
				last_was_frontmatter = false;
			}
			crate::parser::MarkdownBlock::Paragraph(s) => {
				ensure_newlines(&mut formatted_parts, 1, options.compact_blank_lines);
				formatted_parts.push(lines::process_markdown_line(s, &options, false));
				formatted_parts.push("\n".to_string());
				last_was_frontmatter = false;
			}
			crate::parser::MarkdownBlock::BlankLine => {
				let skip_blank_line = if !options.blanks_around_headings {
					let prev_is_heading = idx > 0
						&& matches!(
							parsed.blocks[idx - 1],
							crate::parser::MarkdownBlock::Heading { .. }
						);
					let next_is_heading = idx + 1 < parsed.blocks.len()
						&& matches!(
							parsed.blocks[idx + 1],
							crate::parser::MarkdownBlock::Heading { .. }
						);
					prev_is_heading || next_is_heading
				} else {
					false
				};

				if !skip_blank_line {
					if !options.compact_blank_lines {
						formatted_parts.push("\n".to_string());
					} else {
						ensure_newlines(&mut formatted_parts, 2, options.compact_blank_lines);
					}
				}
			}
			crate::parser::MarkdownBlock::HorizontalRule(s) => {
				if !options.remove_horizontal_rules {
					ensure_newlines(&mut formatted_parts, 1, options.compact_blank_lines);
					formatted_parts.push(s.clone());
					formatted_parts.push("\n".to_string());
				} else if last_was_frontmatter {
					// If HR is removed but it was directly after frontmatter,
					// we might need to ensure something, but the original logic
					// seems to imply it just doesn't add anything.
				}
				last_was_frontmatter = false;
			}
		}
	}

	let mut result = formatted_parts.concat();
	if options.compact_blank_lines {
		let lines: Vec<&str> = result.split('\n').collect();
		let mut compact_lines = Vec::new();
		let mut prev_was_empty = false;

		for (idx, line) in lines.iter().enumerate() {
			let is_empty = line.trim().is_empty();
			if is_empty && prev_was_empty {
				continue;
			}
			if is_empty && idx == 0 {
				continue;
			}
			compact_lines.push(*line);
			prev_was_empty = is_empty;
		}
		result = compact_lines.join("\n");
	}

	while result.ends_with('\n') {
		result.pop();
	}
	if content.ends_with('\n') && !result.is_empty() {
		result.push('\n');
	}
	result
}
