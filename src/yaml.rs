//! Lightweight, zero-dependency YAML configuration parser.
//!
//! Parses YAML mapping documents (used by `.markdownlint.yaml` / `.markdownlint.yml`)
//! into `JsonValue` without external C-transpiled or unsafe dependencies.

use crate::json::{JsonObject, JsonValue};

/// Parse YAML content into a `JsonValue`.
pub fn parse_yaml(content: &str) -> Result<JsonValue, String> {
	let trimmed = content.trim();
	if trimmed.is_empty() {
		return Err("Empty YAML content".to_string());
	}

	// First, check if the content is actually inline JSON (valid YAML subset)
	if (trimmed.starts_with('{') && trimmed.ends_with('}'))
		|| (trimmed.starts_with('[') && trimmed.ends_with(']'))
	{
		if let Ok(val) = JsonValue::parse(trimmed) {
			return Ok(val);
		}
	}

	let lines = preprocess_lines(content);
	if lines.is_empty() {
		return Err("Empty YAML content".to_string());
	}

	let mut index = 0;
	let (val, _) = parse_yaml_block(&lines, &mut index, 0)?;
	Ok(val)
}

#[derive(Debug, Clone)]
struct YamlLine<'a> {
	indent: usize,
	text: &'a str,
}

fn preprocess_lines(content: &str) -> Vec<YamlLine<'_>> {
	let mut out = Vec::new();
	for raw_line in content.lines() {
		// Strip comments outside quotes
		let stripped = strip_yaml_comment(raw_line);
		let trimmed = stripped.trim_end();
		if trimmed.is_empty() || trimmed == "---" || trimmed == "..." {
			continue;
		}

		let indent = trimmed.chars().take_while(|c| *c == ' ').count();
		let text = &trimmed[indent..];
		if !text.is_empty() {
			out.push(YamlLine { indent, text });
		}
	}
	out
}

fn strip_yaml_comment(line: &str) -> &str {
	let mut in_single = false;
	let mut in_double = false;
	let mut prev_char = '\0';

	for (i, c) in line.char_indices() {
		match c {
			'\'' if !in_double && prev_char != '\\' => in_single = !in_single,
			'"' if !in_single && prev_char != '\\' => in_double = !in_double,
			'#' if !in_single && !in_double => {
				// Only treat as comment if preceded by start of line or whitespace
				if i == 0 || line[..i].ends_with(|ws: char| ws.is_whitespace()) {
					return &line[..i];
				}
			}
			_ => {}
		}
		prev_char = c;
	}
	line
}

fn parse_yaml_block<'a>(
	lines: &[YamlLine<'a>],
	index: &mut usize,
	expected_indent: usize,
) -> Result<(JsonValue, usize), String> {
	if *index >= lines.len() {
		return Err("Unexpected end of YAML input".to_string());
	}

	let first_line = &lines[*index];
	let current_indent = first_line.indent;
	if current_indent < expected_indent {
		return Err(format!(
			"Unexpected unindent at '{}' (expected >= {}, found {})",
			first_line.text, expected_indent, current_indent
		));
	}

	if first_line.text.starts_with('-') && first_line.text[1..].starts_with([' ', '\0']) {
		// List
		parse_yaml_list(lines, index, current_indent)
	} else {
		// Object mapping
		parse_yaml_object(lines, index, current_indent)
	}
}

fn parse_yaml_object<'a>(
	lines: &[YamlLine<'a>],
	index: &mut usize,
	block_indent: usize,
) -> Result<(JsonValue, usize), String> {
	let mut obj = JsonObject::new();

	while *index < lines.len() {
		let line = &lines[*index];
		if line.indent < block_indent {
			break;
		}
		if line.indent > block_indent {
			return Err(format!(
				"Inconsistent indentation in mapping: '{}' (expected {}, found {})",
				line.text, block_indent, line.indent
			));
		}

		// Find key: value separator
		let text = line.text;
		let colon_pos = find_key_colon(text).ok_or_else(|| {
			format!(
				"Expected key-value pair with ':' in YAML mapping: '{}'",
				text
			)
		})?;

		let key = text[..colon_pos].trim();
		let raw_val = text[colon_pos + 1..].trim();
		let clean_key = parse_yaml_key(key);

		*index += 1;

		if raw_val.is_empty() {
			// Sub-block follows if next line is indented more
			if *index < lines.len() && lines[*index].indent > block_indent {
				let child_indent = lines[*index].indent;
				let (child_val, _) = parse_yaml_block(lines, index, child_indent)?;
				obj.insert(clean_key, child_val);
			} else {
				obj.insert(clean_key, JsonValue::Null);
			}
		} else {
			let val = parse_scalar(raw_val)?;
			obj.insert(clean_key, val);
		}
	}

	Ok((JsonValue::Object(obj), block_indent))
}

fn parse_yaml_list<'a>(
	lines: &[YamlLine<'a>],
	index: &mut usize,
	block_indent: usize,
) -> Result<(JsonValue, usize), String> {
	let mut list = Vec::new();

	while *index < lines.len() {
		let line = &lines[*index];
		if line.indent < block_indent {
			break;
		}
		if !line.text.starts_with('-') {
			break;
		}

		let item_text = line.text[1..].trim();
		*index += 1;

		if item_text.is_empty() {
			// Nested block
			if *index < lines.len() && lines[*index].indent > block_indent {
				let child_indent = lines[*index].indent;
				let (child_val, _) = parse_yaml_block(lines, index, child_indent)?;
				list.push(child_val);
			} else {
				list.push(JsonValue::Null);
			}
		} else if let Some(colon_pos) = find_key_colon(item_text) {
			// Sub-object started on the same line as list item `- key: value`
			let key = parse_yaml_key(item_text[..colon_pos].trim());
			let raw_val = item_text[colon_pos + 1..].trim();
			let mut sub_obj = JsonObject::new();

			if raw_val.is_empty() {
				if *index < lines.len() && lines[*index].indent > block_indent {
					let child_indent = lines[*index].indent;
					let (child_val, _) = parse_yaml_block(lines, index, child_indent)?;
					sub_obj.insert(key, child_val);
				} else {
					sub_obj.insert(key, JsonValue::Null);
				}
			} else {
				sub_obj.insert(key, parse_scalar(raw_val)?);
			}

			// Read remaining indented mapping keys if any
			while *index < lines.len() && lines[*index].indent > block_indent {
				let sub_line = &lines[*index];
				if let Some(c_pos) = find_key_colon(sub_line.text) {
					let k = parse_yaml_key(sub_line.text[..c_pos].trim());
					let v = parse_scalar(sub_line.text[c_pos + 1..].trim())?;
					sub_obj.insert(k, v);
					*index += 1;
				} else {
					break;
				}
			}
			list.push(JsonValue::Object(sub_obj));
		} else {
			list.push(parse_scalar(item_text)?);
		}
	}

	Ok((JsonValue::Array(list), block_indent))
}

fn find_key_colon(s: &str) -> Option<usize> {
	let mut in_single = false;
	let mut in_double = false;
	let mut prev_char = '\0';

	for (i, c) in s.char_indices() {
		match c {
			'\'' if !in_double && prev_char != '\\' => in_single = !in_single,
			'"' if !in_single && prev_char != '\\' => in_double = !in_double,
			':' if !in_single && !in_double => {
				let rest = &s[i + 1..];
				if rest.is_empty() || rest.starts_with(' ') || rest.starts_with('\t') {
					return Some(i);
				}
			}
			_ => {}
		}
		prev_char = c;
	}
	None
}

fn parse_yaml_key(key: &str) -> String {
	if (key.starts_with('"') && key.ends_with('"'))
		|| (key.starts_with('\'') && key.ends_with('\''))
	{
		key[1..key.len() - 1].to_string()
	} else {
		key.to_string()
	}
}

fn parse_scalar(val: &str) -> Result<JsonValue, String> {
	let val = val.trim();
	if val.is_empty() || val == "~" || val.eq_ignore_ascii_case("null") {
		return Ok(JsonValue::Null);
	}

	// Booleans
	if val.eq_ignore_ascii_case("true")
		|| val.eq_ignore_ascii_case("yes")
		|| val.eq_ignore_ascii_case("on")
	{
		return Ok(JsonValue::Bool(true));
	}
	if val.eq_ignore_ascii_case("false")
		|| val.eq_ignore_ascii_case("no")
		|| val.eq_ignore_ascii_case("off")
	{
		return Ok(JsonValue::Bool(false));
	}

	// Double-quoted string
	if val.starts_with('"') && val.ends_with('"') && val.len() >= 2 {
		return JsonValue::parse(val);
	}

	// Single-quoted string
	if val.starts_with('\'') && val.ends_with('\'') && val.len() >= 2 {
		let unescaped = val[1..val.len() - 1].replace("''", "'");
		return Ok(JsonValue::String(unescaped));
	}

	// Inline JSON array or object
	if (val.starts_with('[') && val.ends_with(']')) || (val.starts_with('{') && val.ends_with('}'))
	{
		if let Ok(json_val) = JsonValue::parse(val) {
			return Ok(json_val);
		}
	}

	// Number check
	if is_yaml_number(val) {
		return Ok(JsonValue::Number(val.to_string()));
	}

	// Plain unquoted string
	Ok(JsonValue::String(val.to_string()))
}

fn is_yaml_number(s: &str) -> bool {
	let bytes = s.as_bytes();
	if bytes.is_empty() {
		return false;
	}

	let mut i = 0;
	if bytes[0] == b'-' || bytes[0] == b'+' {
		i += 1;
	}
	if i >= bytes.len() {
		return false;
	}

	let mut has_digits = false;
	let mut has_dot = false;
	let mut has_exp = false;

	while i < bytes.len() {
		let b = bytes[i];
		if b.is_ascii_digit() {
			has_digits = true;
			i += 1;
		} else if b == b'.' && !has_dot && !has_exp {
			has_dot = true;
			i += 1;
		} else if (b == b'e' || b == b'E') && !has_exp && has_digits {
			has_exp = true;
			i += 1;
			if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
				i += 1;
			}
		} else {
			return false;
		}
	}

	has_digits
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_parse_simple_yaml_mapping() {
		let yaml_str = "default: true\nline-length: false\nmax-length: 80\n";
		let val = parse_yaml(yaml_str).unwrap();
		assert!(val.is_object());
		assert_eq!(val.get("default").unwrap().as_bool(), Some(true));
		assert_eq!(val.get("line-length").unwrap().as_bool(), Some(false));
		assert_eq!(val.get("max-length").unwrap().as_u64(), Some(80));
	}

	#[test]
	fn test_parse_nested_yaml_mapping() {
		let yaml_str = r#"
# Comment
line-length:
  code_blocks: false
  line_length: 120
MD007:
  indent: 4
"#;
		let val = parse_yaml(yaml_str).unwrap();
		assert!(val.is_object());
		let ll = val.get("line-length").unwrap();
		assert_eq!(ll.get("code_blocks").unwrap().as_bool(), Some(false));
		assert_eq!(ll.get("line_length").unwrap().as_u64(), Some(120));
		assert_eq!(
			val.get("MD007").unwrap().get("indent").unwrap().as_u64(),
			Some(4)
		);
	}

	#[test]
	fn test_parse_yaml_list() {
		let yaml_str = r#"
tags:
  - formatting
  - whitespace
"#;
		let val = parse_yaml(yaml_str).unwrap();
		let tags = val.get("tags").unwrap().as_array().unwrap();
		assert_eq!(tags.len(), 2);
		assert_eq!(tags[0].as_str(), Some("formatting"));
		assert_eq!(tags[1].as_str(), Some("whitespace"));
	}
}
