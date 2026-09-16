//! HTML minification and formatting for agent-md.
//!
//! Removes useless whitespace, tabs, and newlines inside HTML tags and elements
//! while preserving original tag names, attributes, and attribute values.

/// Void elements in HTML that do not require closing tags.
const VOID_ELEMENTS: &[&str] = &[
	"area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
	"track", "wbr",
];

/// Check if a tag name is a known void element.
pub fn is_void_element(tag_name: &str) -> bool {
	let lower = tag_name.to_lowercase();
	VOID_ELEMENTS.contains(&lower.as_str())
}

/// Find matching `>` for a tag starting at `start` where `chars[start] == '<'`.
/// Respects single and double quotes to avoid terminating on `>` inside attribute values.
pub fn find_tag_end(chars: &[char], start: usize) -> Option<usize> {
	if start >= chars.len() || chars[start] != '<' {
		return None;
	}

	// Check for comment <!-- ... -->
	if start + 3 < chars.len()
		&& chars[start + 1] == '!'
		&& chars[start + 2] == '-'
		&& chars[start + 3] == '-'
	{
		let mut j = start + 4;
		while j + 2 < chars.len() {
			if chars[j] == '-' && chars[j + 1] == '-' && chars[j + 2] == '>' {
				return Some(j + 2);
			}
			j += 1;
		}
		return None;
	}

	let mut i = start + 1;
	let mut quote: Option<char> = None;

	while i < chars.len() {
		let c = chars[i];
		if let Some(q) = quote {
			if c == q {
				quote = None;
			}
		} else if c == '"' || c == '\'' {
			quote = Some(c);
		} else if c == '>' {
			return Some(i);
		}
		i += 1;
	}

	None
}

/// Parse and minify an opening or self-closing HTML tag.
fn minify_open_or_self_closing_tag(tag: &str) -> String {
	let chars: Vec<char> = tag.chars().collect();
	let mut i = 1; // skip '<'

	// Skip leading whitespace inside '<'
	while i < chars.len() && chars[i].is_whitespace() {
		i += 1;
	}

	// Read tag name
	let mut tag_name = String::new();
	while i < chars.len() && !chars[i].is_whitespace() && chars[i] != '>' && chars[i] != '/' {
		tag_name.push(chars[i]);
		i += 1;
	}

	if tag_name.is_empty() {
		return tag.to_string();
	}

	let mut attributes = Vec::new();
	let mut is_self_closing = false;

	while i < chars.len() {
		while i < chars.len() && chars[i].is_whitespace() {
			i += 1;
		}
		if i >= chars.len() {
			break;
		}
		if chars[i] == '>' {
			break;
		}
		if chars[i] == '/' {
			is_self_closing = true;
			break;
		}

		// Read attribute name
		let mut attr_name = String::new();
		while i < chars.len()
			&& !chars[i].is_whitespace()
			&& chars[i] != '='
			&& chars[i] != '>'
			&& chars[i] != '/'
		{
			attr_name.push(chars[i]);
			i += 1;
		}

		if attr_name.is_empty() {
			i += 1;
			continue;
		}

		// Check for '='
		let mut check_eq = i;
		while check_eq < chars.len() && chars[check_eq].is_whitespace() {
			check_eq += 1;
		}

		if check_eq < chars.len() && chars[check_eq] == '=' {
			i = check_eq + 1;
			while i < chars.len() && chars[i].is_whitespace() {
				i += 1;
			}

			let mut attr_val = String::new();
			if i < chars.len() && chars[i] == '"' {
				attr_val.push('"');
				i += 1;
				while i < chars.len() && chars[i] != '"' {
					attr_val.push(chars[i]);
					i += 1;
				}
				if i < chars.len() && chars[i] == '"' {
					attr_val.push('"');
					i += 1;
				}
			} else if i < chars.len() && chars[i] == '\'' {
				attr_val.push('\'');
				i += 1;
				while i < chars.len() && chars[i] != '\'' {
					attr_val.push(chars[i]);
					i += 1;
				}
				if i < chars.len() && chars[i] == '\'' {
					attr_val.push('\'');
					i += 1;
				}
			} else {
				while i < chars.len()
					&& !chars[i].is_whitespace()
					&& chars[i] != '>'
					&& chars[i] != '/'
				{
					attr_val.push(chars[i]);
					i += 1;
				}
			}
			attributes.push(format!("{}={}", attr_name, attr_val));
		} else {
			attributes.push(attr_name);
		}
	}

	let mut result = format!("<{}", tag_name);
	for attr in attributes {
		result.push(' ');
		result.push_str(&attr);
	}

	if is_self_closing {
		result.push_str(" />");
	} else {
		result.push('>');
	}

	result
}

/// Minify a single HTML tag by removing unnecessary whitespace between attributes
/// and before closing markers, while preserving tag names, attributes, and attribute values.
pub fn minify_html_tag(tag: &str) -> String {
	let trimmed = tag.trim();
	if trimmed.starts_with("<!--") || trimmed.starts_with("<![CDATA[") || trimmed.starts_with("<?")
	{
		return tag.to_string();
	}

	if trimmed.starts_with("</") {
		let inner = &trimmed[2..trimmed.len().saturating_sub(1)];
		let tag_name = inner.trim();
		return format!("</{}>", tag_name);
	}

	if trimmed.starts_with('<') && trimmed.ends_with('>') {
		return minify_open_or_self_closing_tag(trimmed);
	}

	tag.to_string()
}

/// Check if a string consists solely of closing HTML tags (e.g. `</p>`, `</a></div>`).
pub fn is_only_closing_tags(s: &str) -> bool {
	let trimmed = s.trim();
	if trimmed.is_empty() {
		return false;
	}

	let chars: Vec<char> = trimmed.chars().collect();
	let mut i = 0;

	while i < chars.len() {
		while i < chars.len() && chars[i].is_whitespace() {
			i += 1;
		}
		if i >= chars.len() {
			break;
		}
		if chars[i] != '<' || i + 1 >= chars.len() || chars[i + 1] != '/' {
			return false;
		}
		i += 2;
		let mut has_name = false;
		while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '-' || chars[i] == '_')
		{
			has_name = true;
			i += 1;
		}
		if !has_name {
			return false;
		}
		while i < chars.len() && chars[i].is_whitespace() {
			i += 1;
		}
		if i >= chars.len() || chars[i] != '>' {
			return false;
		}
		i += 1;
	}

	true
}

/// Minify all HTML tags within a text string while preserving content outside tags
/// and leaving inline code blocks untouched.
pub fn minify_html_tags_in_text(text: &str) -> String {
	let chars: Vec<char> = text.chars().collect();
	let mut result = String::new();
	let mut i = 0;

	while i < chars.len() {
		// Skip inline code spans enclosed in backticks
		if chars[i] == '`' {
			if let Some(code_end) = crate::format::lines::find_code_span_end(&chars, i) {
				for &c in &chars[i..=code_end] {
					result.push(c);
				}
				i = code_end + 1;
				continue;
			}
		}

		// Check for HTML comment <!--
		if i + 3 < chars.len()
			&& chars[i] == '<'
			&& chars[i + 1] == '!'
			&& chars[i + 2] == '-'
			&& chars[i + 3] == '-'
		{
			let mut j = i + 4;
			let mut found_end = false;
			while j + 2 < chars.len() {
				if chars[j] == '-' && chars[j + 1] == '-' && chars[j + 2] == '>' {
					for &c in &chars[i..=j + 2] {
						result.push(c);
					}
					i = j + 3;
					found_end = true;
					break;
				}
				j += 1;
			}
			if found_end {
				continue;
			}
		}

		// Check for HTML tag
		if chars[i] == '<' && i + 1 < chars.len() {
			let next_c = chars[i + 1];
			let is_tag_start =
				next_c.is_ascii_alphabetic() || next_c == '/' || next_c == '!' || next_c == '?';
			if is_tag_start {
				if let Some(tag_end) = find_tag_end(&chars, i) {
					let tag_str: String = chars[i..=tag_end].iter().collect();
					result.push_str(&minify_html_tag(&tag_str));
					i = tag_end + 1;
					continue;
				}
			}
		}

		result.push(chars[i]);
		i += 1;
	}

	result
}

/// Check if a line starts an HTML block in markdown.
pub fn is_html_block_start(line: &str) -> bool {
	let trimmed = line.trim();
	if !trimmed.starts_with('<') {
		return false;
	}
	if trimmed.starts_with("<!--") || trimmed.starts_with("<!") || trimmed.starts_with("<?") {
		return true;
	}
	let after_bracket = if let Some(stripped) = trimmed.strip_prefix("</") {
		stripped
	} else if let Some(stripped) = trimmed.strip_prefix('<') {
		stripped
	} else {
		return false;
	};

	// Exclude autolinks like <http://...>, <https://...>, <mailto:...>, or <user@example.com>
	if after_bracket.starts_with("http://")
		|| after_bracket.starts_with("https://")
		|| after_bracket.starts_with("mailto:")
	{
		return false;
	}
	if let Some(c) = after_bracket.chars().next() {
		if !c.is_ascii_alphabetic() {
			return false;
		}
	} else {
		return false;
	}

	// If it contains @ before >, it is likely an email autolink
	if let Some(close_pos) = after_bracket.find('>') {
		let inside = &after_bracket[..close_pos];
		if inside.contains('@') && !inside.contains(' ') && !inside.contains('=') {
			return false;
		}
	}

	true
}

/// Update the tag stack and comment state based on tags present in a line.
pub fn update_tag_state(line: &str, tag_stack: &mut Vec<String>, in_comment: &mut bool) {
	let chars: Vec<char> = line.chars().collect();
	let mut i = 0;

	while i < chars.len() {
		// If in comment, look for -->
		if *in_comment {
			if i + 2 < chars.len() && chars[i] == '-' && chars[i + 1] == '-' && chars[i + 2] == '>'
			{
				*in_comment = false;
				i += 3;
			} else {
				i += 1;
			}
			continue;
		}

		// Look for comment start <!--
		if i + 3 < chars.len()
			&& chars[i] == '<'
			&& chars[i + 1] == '!'
			&& chars[i + 2] == '-'
			&& chars[i + 3] == '-'
		{
			*in_comment = true;
			i += 4;
			continue;
		}

		// Look for HTML tag
		if chars[i] == '<' && i + 1 < chars.len() {
			if let Some(tag_end) = find_tag_end(&chars, i) {
				let tag_str: String = chars[i..=tag_end].iter().collect();
				let trimmed = tag_str.trim();
				if trimmed.starts_with("</") {
					let inner = &trimmed[2..trimmed.len().saturating_sub(1)];
					let name = inner.trim().to_lowercase();
					if let Some(pos) = tag_stack.iter().rposition(|t| t == &name) {
						tag_stack.truncate(pos);
					}
				} else if trimmed.starts_with('<') && !trimmed.ends_with("/>") {
					let inner = &trimmed[1..trimmed.len().saturating_sub(1)];
					let name: String = inner
						.chars()
						.take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
						.collect::<String>()
						.to_lowercase();
					if !name.is_empty() && !is_void_element(&name) {
						tag_stack.push(name);
					}
				}
				i = tag_end + 1;
				continue;
			}
		}

		i += 1;
	}
}

/// Collapse tags that span multiple lines into single lines.
fn collapse_multiline_tags(raw: &str) -> Vec<String> {
	let mut result = Vec::new();
	let mut current_line = String::new();
	let mut in_tag = false;

	for line in raw.lines() {
		let trimmed = line.trim();
		if trimmed.is_empty() && !in_tag {
			continue;
		}

		if in_tag {
			if !current_line.ends_with(' ')
				&& !trimmed.starts_with('>')
				&& !trimmed.starts_with("/>")
			{
				current_line.push(' ');
			}
			current_line.push_str(trimmed);
		} else {
			if !current_line.is_empty() {
				result.push(current_line);
				current_line = String::new();
			}
			current_line.push_str(line.trim_start());
		}

		// Recompute in_tag state for current_line
		let chars: Vec<char> = current_line.chars().collect();
		in_tag = false;
		let mut quote_char = None;
		let mut idx = 0;
		while idx < chars.len() {
			let c = chars[idx];
			if let Some(q) = quote_char {
				if c == q {
					quote_char = None;
				}
			} else if c == '"' || c == '\'' {
				if in_tag {
					quote_char = Some(c);
				}
			} else if c == '<' {
				if idx + 1 < chars.len() {
					let next = chars[idx + 1];
					if next.is_ascii_alphabetic() || next == '/' || next == '!' || next == '?' {
						in_tag = true;
					}
				}
			} else if c == '>' && in_tag {
				in_tag = false;
			}
			idx += 1;
		}

		if !in_tag {
			result.push(current_line);
			current_line = String::new();
		}
	}

	if !current_line.is_empty() {
		result.push(current_line);
	}

	result
}

/// Format an HTML block by minifying tags, removing redundant indentation,
/// removing useless blank lines, and merging closing tags onto previous lines.
pub fn format_html_block(raw: &str) -> String {
	let collapsed_lines = collapse_multiline_tags(raw);
	let mut cleaned_lines = Vec::new();

	for line in collapsed_lines {
		let trimmed = line.trim();
		if trimmed.is_empty() {
			continue;
		}
		let minified = minify_html_tags_in_text(trimmed);
		cleaned_lines.push(minified);
	}

	let mut merged_lines: Vec<String> = Vec::new();
	for line in cleaned_lines {
		if is_only_closing_tags(&line) && !merged_lines.is_empty() {
			if let Some(last) = merged_lines.last_mut() {
				last.push_str(&line);
			}
		} else {
			merged_lines.push(line);
		}
	}

	merged_lines.join("\n")
}
