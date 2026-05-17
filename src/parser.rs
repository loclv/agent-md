use crate::format::frontmatter;

#[derive(Debug, PartialEq, Clone)]
pub enum MarkdownBlock {
	Frontmatter(String),
	Heading {
		level: u32,
		text: String,
		raw: String,
	},
	CodeBlock {
		language: Option<String>,
		content: String,
		raw: String,
	},
	Table {
		header: String,
		separator: String,
		rows: Vec<String>,
		raw: String,
	},
	List {
		items: Vec<String>,
		raw: String,
	},
	Paragraph(String),
	BlankLine,
	HorizontalRule(String),
}

pub struct ParsedMarkdown {
	pub blocks: Vec<MarkdownBlock>,
}

pub fn parse(content: &str) -> ParsedMarkdown {
	let lines: Vec<&str> = content.lines().collect();
	let mut blocks = Vec::new();
	let mut i = 0;

	// Handle Frontmatter
	if frontmatter::is_frontmatter_start(&lines) {
		let mut fm_content = String::new();
		fm_content.push_str(lines[0]);
		fm_content.push('\n');
		i += 1;
		while i < lines.len() {
			let line = lines[i];
			fm_content.push_str(line);
			fm_content.push('\n');
			if frontmatter::is_frontmatter_end(line.trim(), i) {
				i += 1;
				break;
			}
			i += 1;
		}
		blocks.push(MarkdownBlock::Frontmatter(fm_content));
	}

	let mut in_code_block = false;
	let mut code_block_raw = String::new();
	let mut code_block_lang = None;
	let mut code_block_content = String::new();

	while i < lines.len() {
		let line = lines[i];
		let trimmed = line.trim();

		// Code Block handling
		if trimmed.starts_with("```") {
			if !in_code_block {
				in_code_block = true;
				code_block_lang = trimmed.strip_prefix("```").map(|s| s.trim().to_string());
				code_block_raw = line.to_string();
				code_block_raw.push('\n');
				code_block_content = String::new();
			} else {
				in_code_block = false;
				code_block_raw.push_str(line);
				code_block_raw.push('\n');
				blocks.push(MarkdownBlock::CodeBlock {
					language: if code_block_lang.as_deref() == Some("") {
						None
					} else {
						code_block_lang.clone()
					},
					content: code_block_content.clone(),
					raw: code_block_raw.clone(),
				});
			}
			i += 1;
			continue;
		}

		if in_code_block {
			code_block_content.push_str(line);
			code_block_content.push('\n');
			code_block_raw.push_str(line);
			code_block_raw.push('\n');
			i += 1;
			continue;
		}

		// Blank line
		if trimmed.is_empty() {
			blocks.push(MarkdownBlock::BlankLine);
			i += 1;
			continue;
		}

		// Heading
		if trimmed.starts_with('#') {
			let level = trimmed.chars().take_while(|&c| c == '#').count() as u32;
			let text = trimmed.trim_start_matches('#').trim().to_string();
			blocks.push(MarkdownBlock::Heading {
				level,
				text,
				raw: line.to_string(),
			});
			i += 1;
			continue;
		}

		// Horizontal Rule
		if crate::format::is_horizontal_rule(trimmed) {
			blocks.push(MarkdownBlock::HorizontalRule(line.to_string()));
			i += 1;
			continue;
		}

		// List Item
		if crate::rules::detect_list_item(trimmed).is_some() {
			let mut list_raw = String::new();
			let mut items = Vec::new();
			while i < lines.len() {
				let current_line = lines[i];
				let current_trimmed = current_line.trim();
				if current_trimmed.is_empty() {
					break; // Blank line ends the list (for now, simple)
				}
				// Check if it's a list item or indented content
				if crate::rules::detect_list_item(current_trimmed).is_some()
					|| current_line.starts_with(' ')
					|| current_line.starts_with('\t')
				{
					list_raw.push_str(current_line);
					list_raw.push('\n');
					items.push(current_line.to_string());
					i += 1;
				} else {
					break;
				}
			}
			blocks.push(MarkdownBlock::List {
				items,
				raw: list_raw,
			});
			continue;
		}

		// Table (simple detection: line contains |)
		if trimmed.starts_with('|')
			|| (trimmed.contains('|')
				&& i + 1 < lines.len()
				&& lines[i + 1].trim().contains('|')
				&& lines[i + 1].trim().contains('-'))
		{
			// This is a bit simplified, but let's try to group table lines
			let mut table_raw = String::new();
			while i < lines.len() && lines[i].trim().contains('|') {
				table_raw.push_str(lines[i]);
				table_raw.push('\n');
				i += 1;
			}
			// For now, just store table as raw, but we could structure it more
			blocks.push(MarkdownBlock::Table {
				header: String::new(),    // Placeholder
				separator: String::new(), // Placeholder
				rows: Vec::new(),         // Placeholder
				raw: table_raw,
			});
			continue;
		}

		// Paragraph (default)
		blocks.push(MarkdownBlock::Paragraph(line.to_string()));
		i += 1;
	}

	ParsedMarkdown { blocks }
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_parse_frontmatter() {
		let content = "---\ntitle: Test\n---\n# Heading";
		let parsed = parse(content);
		assert_eq!(parsed.blocks.len(), 2);
		match &parsed.blocks[0] {
			MarkdownBlock::Frontmatter(s) => assert_eq!(s, "---\ntitle: Test\n---\n"),
			_ => panic!("Expected Frontmatter block"),
		}
	}

	#[test]
	fn test_parse_code_block() {
		let content = "```rust\nfn main() {}\n```";
		let parsed = parse(content);
		assert_eq!(parsed.blocks.len(), 1);
		match &parsed.blocks[0] {
			MarkdownBlock::CodeBlock {
				language, content, ..
			} => {
				assert_eq!(language.as_deref(), Some("rust"));
				assert_eq!(content, "fn main() {}\n");
			}
			_ => panic!("Expected CodeBlock"),
		}
	}

	#[test]
	fn test_parse_list() {
		let content = "Text\n- Item 1\n- Item 2";
		let parsed = parse(content);
		assert_eq!(parsed.blocks.len(), 2);
		match &parsed.blocks[0] {
			MarkdownBlock::Paragraph(s) => assert_eq!(s, "Text"),
			_ => panic!("Expected Paragraph"),
		}
		match &parsed.blocks[1] {
			MarkdownBlock::List { items, .. } => assert_eq!(items.len(), 2),
			_ => panic!("Expected List"),
		}
	}

	#[test]
	fn test_parse_multiple_tables() {
		let content = "| T1 |\n|---|\n| V1 |\n\nText\n\n| T2 |\n|---|\n| V2 |\n";
		let parsed = parse(content);
		// Table, BlankLine, Paragraph, BlankLine, Table
		assert_eq!(parsed.blocks.len(), 5);
		assert!(matches!(parsed.blocks[0], MarkdownBlock::Table { .. }));
		assert!(matches!(parsed.blocks[1], MarkdownBlock::BlankLine));
		assert!(matches!(parsed.blocks[2], MarkdownBlock::Paragraph(_)));
		assert!(matches!(parsed.blocks[3], MarkdownBlock::BlankLine));
		assert!(matches!(parsed.blocks[4], MarkdownBlock::Table { .. }));
	}

	#[test]
	fn test_parse_indented_list() {
		let content = "- Item 1\n  - Subitem 1\n  - Subitem 2\n- Item 2";
		let parsed = parse(content);
		assert_eq!(parsed.blocks.len(), 1);
		match &parsed.blocks[0] {
			MarkdownBlock::List { items, .. } => assert_eq!(items.len(), 4),
			_ => panic!("Expected List"),
		}
	}

	#[test]
	fn test_parse_mixed_content() {
		let content =
			"# Title\n\nIntro.\n\n- L1\n- L2\n\n```js\nconst x = 1;\n```\n\n| H |\n|---|\n| V |\n";
		let parsed = parse(content);
		// Heading, BlankLine, Paragraph, BlankLine, List, BlankLine, CodeBlock, BlankLine, Table
		assert_eq!(parsed.blocks.len(), 9);
		assert!(matches!(parsed.blocks[0], MarkdownBlock::Heading { .. }));
		assert!(matches!(parsed.blocks[2], MarkdownBlock::Paragraph(_)));
		assert!(matches!(parsed.blocks[4], MarkdownBlock::List { .. }));
		assert!(matches!(parsed.blocks[6], MarkdownBlock::CodeBlock { .. }));
		assert!(matches!(parsed.blocks[8], MarkdownBlock::Table { .. }));
	}
}
