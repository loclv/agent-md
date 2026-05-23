use super::blockquotes;
use super::tables;
use super::FormatOptions;

/// Process a single markdown line with formatting options.
/// Used for both regular lines and lines inside ```markdown code blocks.
pub fn process_markdown_line(line: &str, options: &FormatOptions, is_heading: bool) -> String {
	// Handle tables
	let (prefix, table_content) = tables::parse_table_line(line);
	if !table_content.is_empty() {
		if tables::is_separator_row(table_content) {
			let compacted = tables::compact_separator_row(table_content);
			return format!("{}{}", prefix, compacted);
		} else {
			return tables::format_table_row(prefix, table_content, options.remove_bold);
		}
	}

	// Process regular line
	let mut processed_line = if options.remove_bold {
		remove_bold_markers(line)
	} else {
		line.to_string()
	};

	if options.remove_emphasis && !is_heading {
		processed_line = remove_emphasis_markers(&processed_line);
	}

	if options.collapse_spaces && !is_heading {
		processed_line = collapse_multiple_spaces(&processed_line);
	}

	// Normalize blockquote lines (remove extra spaces after > markers)
	processed_line = blockquotes::normalize_blockquote(&processed_line);

	if options.trim_trailing_whitespace {
		processed_line = processed_line.trim_end().to_string();
	}

	processed_line
}

/// Remove bold markers (** and __) from a line while preserving the content inside.
/// This skips markers inside inline code spans.
pub fn remove_bold_markers(line: &str) -> String {
	let mut result = String::new();
	let chars: Vec<char> = line.chars().collect();
	let mut i = 0;

	while i < chars.len() {
		// Check for inline code start
		if chars[i] == '`' {
			// Find the end of the inline code
			let mut code_end = i + 1;
			while code_end < chars.len() {
				if chars[code_end] == '`' {
					break;
				}
				code_end += 1;
			}
			// Copy the entire code span as-is
			for j in i..=code_end {
				if j < chars.len() {
					result.push(chars[j]);
				}
			}
			i = code_end + 1;
			continue;
		}

		// Check for **bold** pattern
		if i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '*' {
			// Find the closing **
			let mut j = i + 2;
			while j + 1 < chars.len() {
				if chars[j] == '*' && chars[j + 1] == '*' {
					// Found closing marker, copy content between markers
					chars[i + 2..j].iter().for_each(|&c| result.push(c));
					i = j + 2;
					break;
				}
				j += 1;
			}
			// If no closing marker found, copy the ** as-is
			if i <= j {
				result.push(chars[i]);
				i += 1;
			}
			continue;
		}

		// Check for __bold__ pattern
		if i + 1 < chars.len() && chars[i] == '_' && chars[i + 1] == '_' {
			// Find the closing __
			let mut j = i + 2;
			while j + 1 < chars.len() {
				if chars[j] == '_' && chars[j + 1] == '_' {
					// Found closing marker, copy content between markers
					chars[i + 2..j].iter().for_each(|&c| result.push(c));
					i = j + 2;
					break;
				}
				j += 1;
			}
			// If no closing marker found, copy the __ as-is
			if i <= j {
				result.push(chars[i]);
				i += 1;
			}
			continue;
		}

		// Regular character, just copy it
		result.push(chars[i]);
		i += 1;
	}

	result
}

pub fn is_horizontal_rule(line: &str) -> bool {
	let trimmed = line.trim();
	trimmed == "---" || trimmed == "***" || trimmed == "___"
}

pub fn remove_emphasis_markers(line: &str) -> String {
	let mut result = String::new();
	let chars: Vec<char> = line.chars().collect();
	let mut i = 0;

	while i < chars.len() {
		if chars[i] == '`' {
			let mut code_end = i + 1;
			while code_end < chars.len() {
				if chars[code_end] == '`' {
					break;
				}
				code_end += 1;
			}
			for j in i..=code_end {
				if j < chars.len() {
					result.push(chars[j]);
				}
			}
			i = code_end + 1;
			continue;
		}

		if chars[i] == '[' {
			let mut bracket_end = i;
			while bracket_end < chars.len() && chars[bracket_end] != ']' {
				bracket_end += 1;
			}
			for j in i..=bracket_end {
				if j < chars.len() {
					result.push(chars[j]);
				}
			}
			i = bracket_end + 1;
			continue;
		}

		if i + 1 < chars.len()
			&& ((chars[i] == '*' && chars[i + 1] != '*')
				|| (chars[i] == '_' && chars[i + 1] != '_'))
		{
			let marker = chars[i];

			// For underscore, check if it's part of an identifier (e.g., A_cat_meow)
			// Only skip if BOTH sides are alphanumeric (underscore within a word)
			// _word_ at boundaries should still be treated as emphasis
			if marker == '_' {
				let prev_is_word = i > 0 && chars[i - 1].is_alphanumeric();
				let next_is_word = i + 1 < chars.len() && chars[i + 1].is_alphanumeric();
				if prev_is_word && next_is_word {
					// This underscore is within a word (identifier), skip it
					result.push(chars[i]);
					i += 1;
					continue;
				}
			}

			let mut j = i + 1;
			while j < chars.len() && chars[j] != marker {
				j += 1;
			}
			if j < chars.len() && chars[j] == marker {
				// For underscore, also check the closing marker isn't within a word
				if marker == '_' {
					let prev_is_word = j > 0 && chars[j - 1].is_alphanumeric();
					let next_is_word = j + 1 < chars.len() && chars[j + 1].is_alphanumeric();
					if prev_is_word && next_is_word {
						// Closing underscore is within a word (identifier), skip this match
						result.push(chars[i]);
						i += 1;
						continue;
					}
				}
				chars[i + 1..j].iter().for_each(|&c| result.push(c));
				i = j + 1;
			} else {
				result.push(chars[i]);
				i += 1;
			}
			continue;
		}

		result.push(chars[i]);
		i += 1;
	}

	result
}

pub fn collapse_multiple_spaces(line: &str) -> String {
	// Preserve leading whitespace (indentation)
	let leading_len = line.chars().take_while(|&c| c == ' ').count();
	let leading = &line[..leading_len];
	let rest = &line[leading_len..];

	let mut result = String::from(leading);
	let mut prev_was_space = false;

	for c in rest.chars() {
		if c == ' ' {
			if !prev_was_space {
				result.push(c);
			}
			prev_was_space = true;
		} else {
			result.push(c);
			prev_was_space = false;
		}
	}

	result
}

/// Dynamic, context-aware list item indentation formatting.
/// Compresses 4 spaces to 2 and 2 tabs to 1 relative to parent nesting.
pub fn format_list_items(items: &[String]) -> Vec<String> {
	let mut formatted_items = Vec::new();
	let mut stack: Vec<(String, String)> = Vec::new(); // (original_leading, formatted_leading)
	let mut in_code_block = false;
	let mut code_block_indent = String::new();
	let mut code_block_formatted_indent = String::new();

	for item in items {
		let trimmed = item.trim();
		if trimmed.starts_with("```") {
			if !in_code_block {
				in_code_block = true;
				// Remember the indentation of the opening fence
				let leading_len = item.chars().take_while(|&c| c == ' ' || c == '\t').count();
				code_block_indent = item[..leading_len].to_string();

				// Determine formatted leading for the fence
				let formatted_leading = if code_block_indent.is_empty() {
					String::new()
				} else if stack.is_empty() {
					let mut f = code_block_indent.clone();
					f = f.replace("    ", "  ").replace("\t\t", "\t");
					stack.push((code_block_indent.clone(), f.clone()));
					f
				} else {
					let mut resolved = code_block_indent.clone();
					if let Some((top_orig, top_formatted)) = stack.last() {
						if code_block_indent == *top_orig {
							resolved = top_formatted.clone();
						} else if code_block_indent.starts_with(top_orig)
							&& code_block_indent.len() > top_orig.len()
						{
							let suffix = &code_block_indent[top_orig.len()..];
							let mut formatted_suffix = suffix.to_string();
							if formatted_suffix == "    " {
								formatted_suffix = "  ".to_string();
							} else if formatted_suffix == "\t\t" {
								formatted_suffix = "\t".to_string();
							} else {
								formatted_suffix =
									formatted_suffix.replace("    ", "  ").replace("\t\t", "\t");
							}
							resolved = format!("{}{}", top_formatted, formatted_suffix);
							stack.push((code_block_indent.clone(), resolved.clone()));
						}
					}
					resolved
				};
				code_block_formatted_indent = formatted_leading.clone();
				formatted_items.push(format!("{}{}", formatted_leading, trimmed));
			} else {
				in_code_block = false;
				formatted_items.push(format!("{}{}", code_block_formatted_indent, trimmed));
			}
			continue;
		}

		if in_code_block {
			// Inside code block, adjust only the base indentation and preserve everything else!
			if item.starts_with(&code_block_indent) {
				let remainder = &item[code_block_indent.len()..];
				formatted_items.push(format!("{}{}", code_block_formatted_indent, remainder));
			} else {
				formatted_items.push(item.clone());
			}
			continue;
		}

		let leading_len = item.chars().take_while(|&c| c == ' ' || c == '\t').count();
		let orig_leading = &item[..leading_len];
		let rest = &item[leading_len..];

		// Determine formatted leading
		let formatted_leading = if orig_leading.is_empty() {
			String::new()
		} else if stack.is_empty() {
			// First item with leading whitespace
			let mut f = orig_leading.to_string();
			if f == "    " {
				f = "  ".to_string();
			} else if f == "\t\t" {
				f = "\t".to_string();
			} else {
				f = f.replace("    ", "  ").replace("\t\t", "\t");
			}
			stack.push((orig_leading.to_string(), f.clone()));
			f
		} else {
			let (top_orig, top_formatted) = match stack.last() {
				Some(t) => t.clone(),
				None => (orig_leading.to_string(), orig_leading.to_string()),
			};
			if orig_leading == top_orig {
				top_formatted
			} else if orig_leading.starts_with(&top_orig) && orig_leading.len() > top_orig.len() {
				// Deeper nesting
				let suffix = &orig_leading[top_orig.len()..];
				let mut formatted_suffix = suffix.to_string();
				if formatted_suffix == "    " {
					formatted_suffix = "  ".to_string();
				} else if formatted_suffix == "\t\t" {
					formatted_suffix = "\t".to_string();
				} else {
					formatted_suffix = formatted_suffix.replace("    ", "  ").replace("\t\t", "\t");
				}
				let f = format!("{}{}", top_formatted, formatted_suffix);
				stack.push((orig_leading.to_string(), f.clone()));
				f
			} else {
				// Pop until we find a match or prefix
				while let Some((pop_orig, _)) = stack.last() {
					if orig_leading.starts_with(pop_orig) {
						break;
					}
					stack.pop();
				}
				if let Some((pop_orig, pop_formatted)) = stack.last().cloned() {
					if orig_leading == pop_orig {
						pop_formatted
					} else {
						let suffix = &orig_leading[pop_orig.len()..];
						let mut formatted_suffix = suffix.to_string();
						if formatted_suffix == "    " {
							formatted_suffix = "  ".to_string();
						} else if formatted_suffix == "\t\t" {
							formatted_suffix = "\t".to_string();
						} else {
							formatted_suffix =
								formatted_suffix.replace("    ", "  ").replace("\t\t", "\t");
						}
						let f = format!("{}{}", pop_formatted, formatted_suffix);
						stack.push((orig_leading.to_string(), f.clone()));
						f
					}
				} else {
					// Fallback if popped everything
					let mut f = orig_leading.to_string();
					if f == "    " {
						f = "  ".to_string();
					} else if f == "\t\t" {
						f = "\t".to_string();
					} else {
						f = f.replace("    ", "  ").replace("\t\t", "\t");
					}
					stack.push((orig_leading.to_string(), f.clone()));
					f
				}
			}
		};

		formatted_items.push(format!("{}{}", formatted_leading, rest));
	}

	formatted_items
}
