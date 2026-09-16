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

	if options.minify_html {
		processed_line = super::html::minify_html_tags_in_text(&processed_line);
	}

	if options.trim_trailing_whitespace {
		processed_line = processed_line.trim_end().to_string();
	}

	processed_line
}

/// Remove bold markers (`**` and `__`) from a line while preserving the content inside.
///
/// This skips markers inside inline code spans (enclosed in backticks).
/// Delimiters immediately followed by whitespace (opening) or preceded by whitespace (closing)
/// are ignored per Markdown formatting rules.
pub fn remove_bold_markers(line: &str) -> String {
	let mut result = String::new();
	let chars: Vec<char> = line.chars().collect();
	let mut i = 0;

	while i < chars.len() {
		// Skip inline code blocks (e.g., `code with **bold**`)
		if chars[i] == '`' {
			if let Some(code_end) = find_code_span_end(&chars, i) {
				for &c in &chars[i..=code_end] {
					result.push(c);
				}
				i = code_end + 1;
				continue;
			}
		}

		// Check for **bold** pattern
		if i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '*' {
			// Opening delimiter MUST NOT be followed by whitespace (e.g., "** bold")
			if i + 2 < chars.len() && chars[i + 2].is_whitespace() {
				result.push(chars[i]);
				i += 1;
				continue;
			}
			// Search for matching closing **
			let mut j = i + 2;
			while j + 1 < chars.len() {
				if chars[j] == '*' && chars[j + 1] == '*' {
					// Closing delimiter MUST NOT be preceded by whitespace (e.g., "bold **")
					if j > i + 2 && chars[j - 1].is_whitespace() {
						break;
					}
					// Found valid closing marker, copy content between markers
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
			// Opening delimiter MUST NOT be followed by whitespace (e.g., "__ bold")
			if i + 2 < chars.len() && chars[i + 2].is_whitespace() {
				result.push(chars[i]);
				i += 1;
				continue;
			}
			// Search for matching closing __
			let mut j = i + 2;
			while j + 1 < chars.len() {
				if chars[j] == '_' && chars[j + 1] == '_' {
					// Closing delimiter MUST NOT be preceded by whitespace (e.g., "bold __")
					if j > i + 2 && chars[j - 1].is_whitespace() {
						break;
					}
					// Found valid closing marker, copy content between markers
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

/// Check if a line is a horizontal rule (`---`, `***`, or `___`).
pub fn is_horizontal_rule(line: &str) -> bool {
	let trimmed = line.trim();
	trimmed == "---" || trimmed == "***" || trimmed == "___"
}

/// Remove emphasis markers (`*` and `_`) from a line while preserving the content inside.
///
/// Skips inline code spans and link labels. Delimiters immediately followed by whitespace (opening)
/// or preceded by whitespace (closing) are ignored to prevent misinterpreting list item bullets
/// or mathematical expressions.
pub fn remove_emphasis_markers(line: &str) -> String {
	let mut result = String::new();
	let chars: Vec<char> = line.chars().collect();
	let mut i = 0;

	while i < chars.len() {
		// Skip inline code spans
		if chars[i] == '`' {
			if let Some(code_end) = find_code_span_end(&chars, i) {
				for &c in &chars[i..=code_end] {
					result.push(c);
				}
				i = code_end + 1;
				continue;
			}
		}

		// Skip markdown link labels [label]
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

		// Check for single asterisk or underscore emphasis marker (*text* or _text_)
		if i + 1 < chars.len()
			&& ((chars[i] == '*' && chars[i + 1] != '*')
				|| (chars[i] == '_' && chars[i + 1] != '_'))
		{
			// Opening delimiter MUST NOT be followed by whitespace (e.g., "* bullet" or "_ word")
			if chars[i + 1].is_whitespace() {
				result.push(chars[i]);
				i += 1;
				continue;
			}

			let marker = chars[i];

			// For underscore, check if it's part of an identifier (e.g., A_cat_meow)
			// Only skip if BOTH sides are alphanumeric (underscore within a word)
			if marker == '_' {
				let prev_is_word = i > 0 && chars[i - 1].is_alphanumeric();
				let next_is_word = i + 1 < chars.len() && chars[i + 1].is_alphanumeric();
				if prev_is_word && next_is_word {
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
				// Closing delimiter MUST NOT be preceded by whitespace (e.g., "text *")
				if j > i + 1 && chars[j - 1].is_whitespace() {
					result.push(chars[i]);
					i += 1;
					continue;
				}

				// For underscore, also check the closing marker isn't within a word
				if marker == '_' {
					let prev_is_word = j > 0 && chars[j - 1].is_alphanumeric();
					let next_is_word = j + 1 < chars.len() && chars[j + 1].is_alphanumeric();
					if prev_is_word && next_is_word {
						result.push(chars[i]);
						i += 1;
						continue;
					}
				}
				// Copy text inside emphasis markers
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

/// Find the end index (inclusive) of an inline code span starting at `start` where `chars[start] == '`'`.
/// In Markdown, a code span begins with a delimiter run of N backticks and ends with the first
/// subsequent delimiter run of exactly N backticks.
pub fn find_code_span_end(chars: &[char], start: usize) -> Option<usize> {
	if start >= chars.len() || chars[start] != '`' {
		return None;
	}

	let mut tick_count = 0;
	while start + tick_count < chars.len() && chars[start + tick_count] == '`' {
		tick_count += 1;
	}

	let mut idx = start + tick_count;
	while idx < chars.len() {
		if chars[idx] == '`' {
			let mut close_count = 0;
			while idx + close_count < chars.len() && chars[idx + close_count] == '`' {
				close_count += 1;
			}
			if close_count == tick_count {
				return Some(idx + close_count - 1);
			}
			idx += close_count;
		} else {
			idx += 1;
		}
	}

	None
}

/// Collapse multiple consecutive space characters into a single space while preserving leading indentation
/// and preserving spaces inside inline code spans.
pub fn collapse_multiple_spaces(line: &str) -> String {
	// Preserve leading whitespace (indentation)
	let leading_len = line.chars().take_while(|&c| c == ' ').count();
	let leading = &line[..leading_len];
	let rest = &line[leading_len..];

	let chars: Vec<char> = rest.chars().collect();
	let mut result = String::from(leading);
	let mut prev_was_space = false;
	let mut i = 0;

	while i < chars.len() {
		if chars[i] == '`' {
			if let Some(code_end) = find_code_span_end(&chars, i) {
				for &c in &chars[i..=code_end] {
					result.push(c);
				}
				i = code_end + 1;
				prev_was_space = false;
				continue;
			}
		}

		let c = chars[i];
		if c == ' ' {
			if !prev_was_space {
				result.push(c);
			}
			prev_was_space = true;
		} else {
			result.push(c);
			prev_was_space = false;
		}
		i += 1;
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

		// Drop empty list items (marker with no content, e.g. a trailing `- ` line)
		if crate::rules::is_empty_list_item(item) {
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
