use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::{format_markdown_with_options, FormatOptions};
use crate::{json_output, parse_markdown, Document, EditResult};

pub fn collect_markdown_files(dir: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
	for entry in fs::read_dir(dir)? {
		let entry = entry?;
		let path = entry.path();
		if path.is_dir() {
			collect_markdown_files(&path, files)?;
		} else if let Some(ext) = path.extension() {
			if ext == "md" || ext == "markdown" {
				files.push(path);
			}
		}
	}
	Ok(())
}

pub fn format_single_file(path: &str, options: FormatOptions) -> Result<Document, String> {
	let content = fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;
	if let Some(start_line) = crate::rules::find_unclosed_code_block(&content) {
		return Err(format!(
			"Syntax Error: Code block starting at line {} is missing a closing fence",
			start_line
		));
	}
	let formatted_content = format_markdown_with_options(&content, options);
	fs::write(path, &formatted_content).map_err(|e| format!("Failed to write file: {}", e))?;
	let mut doc = parse_markdown(&formatted_content);
	doc.path = path.to_string();
	Ok(doc)
}

pub fn cmd_fmt(path: &str, human: bool, options: FormatOptions) {
	let path_buf = PathBuf::from(path);

	if path_buf.is_dir() {
		let mut files = Vec::new();
		if let Err(e) = collect_markdown_files(&path_buf, &mut files) {
			println!(
				"{}",
				json_output(
					&EditResult {
						success: false,
						message: format!("Failed to read directory: {}", e),
						document: None,
					},
					human
				)
			);
			return;
		}

		files.sort();

		let mut success_count = 0;
		let mut error_count = 0;

		for file in &files {
			let file_path = file.to_string_lossy().to_string();
			match format_single_file(&file_path, options.clone()) {
				Ok(doc) => {
					success_count += 1;
					println!(
						"{}",
						json_output(
							&EditResult {
								success: true,
								message: format!("File formatted successfully: {}", file_path),
								document: Some(doc),
							},
							human
						)
					);
				}
				Err(e) => {
					error_count += 1;
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
				}
			}
		}

		let summary = format!(
			"Formatted {} files ({} succeeded, {} failed)",
			files.len(),
			success_count,
			error_count
		);
		println!(
			"{}",
			json_output(
				&EditResult {
					success: error_count == 0,
					message: summary,
					document: None,
				},
				human
			)
		);
		if error_count > 0 {
			std::process::exit(1);
		}
	} else {
		match format_single_file(path, options) {
			Ok(doc) => {
				println!(
					"{}",
					json_output(
						&EditResult {
							success: true,
							message: "File formatted successfully".to_string(),
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
}

/// Format markdown from stdin and write to stdout (Prettier-style).
/// This is used by editor integrations to avoid file sync conflicts.
///
/// # Example
///
/// ```bash
/// echo '# Hello\n\n**bold** text' | agent-md fmt --stdin
/// # Output: # Hello\n\nbold text
/// ```
pub fn cmd_fmt_stdin(options: FormatOptions) {
	use std::io::Read;

	let mut input = String::new();
	if let Err(e) = io::stdin().read_to_string(&mut input) {
		eprintln!("Error reading stdin: {}", e);
		std::process::exit(1);
	}

	if let Some(start_line) = crate::rules::find_unclosed_code_block(&input) {
		eprintln!(
			"Syntax Error: Code block starting at line {} is missing a closing fence",
			start_line
		);
		std::process::exit(1);
	}

	let formatted = format_markdown_with_options(&input, options);
	print!("{}", formatted);
}
