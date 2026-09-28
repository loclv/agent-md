//! Section helpers for locating and replacing Markdown sections.
//!
//! Extracted from `commands` to keep command handlers focused on I/O.

use crate::rules::extract_heading_level;

/// Extract the content of a section identified by a `>`-separated path.
pub fn extract_section_content(content: &str, section_path: &str) -> Option<String> {
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

/// Find the line range of a section identified by a `>`-separated path.
pub fn find_section_range(content: &str, section_path: &str) -> Option<(usize, usize)> {
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

/// Find the end line of a section starting after `start` with parent level.
pub fn find_section_end(lines: &[&str], start: usize, parent_level: u32) -> usize {
	for (i, line) in lines.iter().enumerate().skip(start) {
		if let Some(level) = extract_heading_level(line) {
			if level <= parent_level {
				return i;
			}
		}
	}
	lines.len()
}

/// Replace a section's content, preserving surrounding lines.
pub fn replace_section_content(
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

/// Insert a new section, creating parents as needed.
pub fn insert_section_content(
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn extract_simple_section() {
		let content = "# Title\n\n## Section\n\n### Subsection\n";
		let result = extract_section_content(content, "Section").unwrap();
		assert!(result.contains("## Section"));
		assert!(result.contains("### Subsection"));
	}

	#[test]
	fn section_range_round_trip() {
		let content = "# Title\n\n## Other\nMore\n\n## Section\nBody\n";
		let (start, end) = find_section_range(content, "Section").unwrap();
		let replaced = replace_section_content(content, start, end, "Section", "New").unwrap();
		assert!(replaced.contains("New"));
		assert!(replaced.contains("Other"));
	}

	#[test]
	fn insert_top_level_section() {
		let content = "# Title\n";
		let result = insert_section_content(content, "New", "Body").unwrap();
		assert!(result.contains("New"));
		assert!(result.contains("Body"));
	}
}
