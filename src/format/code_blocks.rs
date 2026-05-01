/// Collapse multiple spaces before # comments in bash/sh code.
/// This handles cases like "cd              # goto" -> "cd # goto"
///
/// # Arguments
/// - `line`: A string slice representing a single line of shell code
///
/// # Returns
/// - A new `String` with normalized spacing before comments
///
/// # Behavior
/// - Finds the first `#` that appears to be a comment (preceded by whitespace or at line start)
/// - Checks if the `#` is inside a quoted string to avoid false positives
/// - Collapses multiple spaces/tabs before the comment to a single space
/// - Preserves indentation for comment-only lines
/// - Leaves code without comments unchanged
///
/// # Examples
/// ```
/// let result = collapse_spaces_before_comment("cd              # goto");
/// assert_eq!(result, "cd # goto");
/// ```
pub fn collapse_spaces_before_comment(line: &str) -> String {
	// Find the position of # that looks like a comment (preceded by space or at start)
	let mut comment_byte_pos = None;
	let chars: Vec<char> = line.chars().collect();

	// Build a mapping from char index to byte offset
	let byte_offsets: Vec<usize> = line.char_indices().map(|(byte_pos, _)| byte_pos).collect();

	for i in 0..chars.len() {
		if chars[i] == '#' {
			// Check if it's a comment: at start or preceded by space(s)
			// But not inside a string - simple heuristic: check for unescaped quotes before
			if i == 0 || chars[i - 1] == ' ' || chars[i - 1] == '\t' {
				// Simple check: count unescaped quotes before this position
				let mut single_quotes = 0;
				let mut double_quotes = 0;
				let mut escaped = false;
				for &c in chars.iter().take(i) {
					if escaped {
						escaped = false;
						continue;
					}
					if c == '\\' {
						escaped = true;
						continue;
					}
					if c == '\'' && double_quotes % 2 == 0 {
						single_quotes += 1;
					}
					if c == '"' && single_quotes % 2 == 0 {
						double_quotes += 1;
					}
				}
				// If we're inside quotes, this # is not a comment
				if single_quotes % 2 == 0 && double_quotes % 2 == 0 {
					comment_byte_pos = Some(byte_offsets[i]);
					break;
				}
			}
		}
	}

	if let Some(pos) = comment_byte_pos {
		let before = &line[..pos];
		let comment = &line[pos..];

		// Find the actual content before the comment (collapse trailing whitespace)
		let before_trimmed_end = before.trim_end();
		// Collapse multiple spaces within the comment text on the same line only
		let comment_line_end = comment.find('\n').unwrap_or(comment.len());
		let comment_line = &comment[..comment_line_end];
		let comment_rest = &comment[comment_line_end..];
		let collapsed_comment_line: String = comment_line
			.trim_start()
			.split_whitespace()
			.collect::<Vec<&str>>()
			.join(" ");
		let collapsed_comment = format!("{}{}", collapsed_comment_line, comment_rest);
		// If there's content before the comment, add a single space before the comment
		if before_trimmed_end.is_empty() {
			// Only whitespace/indentation before the comment - preserve indentation
			let indent: String = before
				.chars()
				.take_while(|&c| c == ' ' || c == '\t')
				.collect();
			format!("{}{}", indent, collapsed_comment)
		} else {
			format!("{} {}", before_trimmed_end, collapsed_comment)
		}
	} else {
		line.to_string()
	}
}

/// Check if the language is a shell language that needs comment formatting
pub fn is_shell_language(lang: &str) -> bool {
	matches!(lang, "bash" | "sh" | "shell" | "zsh")
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_collapse_spaces_before_comment_simple() {
		assert_eq!(
			collapse_spaces_before_comment("cd              # goto"),
			"cd # goto"
		);
	}

	#[test]
	fn test_collapse_spaces_before_comment_no_spaces() {
		assert_eq!(
			collapse_spaces_before_comment("agent-md # format"),
			"agent-md # format"
		);
	}

	#[test]
	fn test_collapse_spaces_before_comment_in_string() {
		// # inside string should not be treated as comment
		assert_eq!(
			collapse_spaces_before_comment("echo \"hello # world\"   # real comment"),
			"echo \"hello # world\" # real comment"
		);
	}

	#[test]
	fn test_collapse_spaces_before_comment_single_quotes() {
		// # inside single quotes should not be treated as comment
		assert_eq!(
			collapse_spaces_before_comment("echo 'hello # world'   # real comment"),
			"echo 'hello # world' # real comment"
		);
	}

	#[test]
	fn test_collapse_spaces_before_comment_no_comment() {
		assert_eq!(collapse_spaces_before_comment("echo hello"), "echo hello");
	}

	#[test]
	fn test_is_shell_language() {
		assert!(is_shell_language("bash"));
		assert!(is_shell_language("sh"));
		assert!(is_shell_language("shell"));
		assert!(is_shell_language("zsh"));
		assert!(!is_shell_language("python"));
	}

	#[test]
	fn test_collapse_spaces_before_comment_install_command() {
		// Test case from test-md/format/code-block.md
		// Input: "cargo install cargo-generate"
		// This line has no comment, so it should remain unchanged
		assert_eq!(
			collapse_spaces_before_comment("cargo install cargo-generate"),
			"cargo install cargo-generate"
		);
	}

	#[test]
	fn test_collapse_spaces_before_comment_with_added_comment() {
		// Test case from test-md/format/code-block.md
		// Expected behavior: format can add comments to bash code blocks
		// This tests the formatted output with a comment added
		assert_eq!(
			collapse_spaces_before_comment("# Install cargo-generate if you haven't already"),
			"# Install cargo-generate if you haven't already"
		);
		assert_eq!(
			collapse_spaces_before_comment("cargo install cargo-generate"),
			"cargo install cargo-generate"
		);
	}

	#[test]
	fn test_bash_code_block_unchanged() {
		// Test case from test-md/format/code-block.md
		// Input and expected output are identical - nothing should change
		let comment_line = "# Install cargo-generate if you haven't already";
		let command_line = "cargo install cargo-generate";

		// Comment line should remain unchanged
		assert_eq!(collapse_spaces_before_comment(comment_line), comment_line);
		// Command line without comment should remain unchanged
		assert_eq!(collapse_spaces_before_comment(command_line), command_line);
	}

	#[test]
	fn test_bash_code_block_remove_leading_spaces_before_comment() {
		// Test case from test-md/format/code-block-comment.md
		// Input has leading space before comment: " # Install..."
		// This is a comment at the START of a line (no command before it)
		// Leading spaces should be preserved for indentation purposes
		let input = " # Install cargo-generate if you haven't already";
		let expected = " # Install cargo-generate if you haven't already";
		assert_eq!(collapse_spaces_before_comment(input), expected);
	}

	#[test]
	fn test_indented_comment_preserved() {
		// Indented comment inside a code block should preserve indentation
		// e.g., inside a for loop
		let input = "    # Search for TODOs";
		let expected = "    # Search for TODOs";
		assert_eq!(collapse_spaces_before_comment(input), expected);
	}

	#[test]
	fn test_indented_command_with_comment() {
		// Indented command with comment should preserve indentation
		let input = "    agent-md search \"$file\" \"TODO\"";
		let expected = "    agent-md search \"$file\" \"TODO\"";
		assert_eq!(collapse_spaces_before_comment(input), expected);
	}

	#[test]
	fn test_collapse_spaces_before_comment_with_box_drawing_single_space() {
		// Test with Unicode box-drawing characters
		let input = "│   ├── pagefind/         # auto-generated when build";
		let expected = "│   ├── pagefind/ # auto-generated when build";
		assert_eq!(collapse_spaces_before_comment(input), expected);
	}

	#[test]
	fn test_collapse_spaces_before_comment_with_text_block_single_space() {
		// Test case from test-md/test-graph.md
		let input = "```text\n│   ├── pagefind/ # auto-generated when build\n```";
		let expected = "```text\n│   ├── pagefind/ # auto-generated when build\n```";
		assert_eq!(collapse_spaces_before_comment(input), expected);
	}

	#[test]
	fn test_collapse_spaces_before_comment_with_text_block_multiple_spaces() {
		// Test case from test-md/test-graph.md
		let input = "```text\n│   ├── pagefind/ #  auto-generated when build\n```";
		let expected = "```text\n│   ├── pagefind/ # auto-generated when build\n```";
		assert_eq!(collapse_spaces_before_comment(input), expected);
	}

	#[test]
	fn test_collapse_spaces_before_comment_with_txt_block_single_space() {
		// Test case from test-md/test-graph.md
		let input = "```txt\n│   ├── pagefind/ # auto-generated when build\n```";
		let expected = "```txt\n│   ├── pagefind/ # auto-generated when build\n```";
		assert_eq!(collapse_spaces_before_comment(input), expected);
	}

	#[test]
	fn test_collapse_spaces_before_comment_with_txt_block_multiple_spaces() {
		// Test case from test-md/test-graph.md
		let input = "```txt\n│   ├── pagefind/   # auto-generated when build\n```";
		let expected = "```txt\n│   ├── pagefind/ # auto-generated when build\n```";
		assert_eq!(collapse_spaces_before_comment(input), expected);
	}

	#[test]
	fn test_collapse_spaces_before_comment_with_txt_block_two_spaces() {
		// Test case from test-md/test-graph.md
		let input = "```txt\n│   ├── pagefind/  # auto-generated when build\n```";
		let expected = "```txt\n│   ├── pagefind/ # auto-generated when build\n```";
		assert_eq!(collapse_spaces_before_comment(input), expected);
	}

	#[test]
	fn test_indented_command_with_inline_comment() {
		// Indented command with inline comment - collapse extra spaces but preserve indent
		let input = "    echo test      # comment";
		let expected = "    echo test # comment";
		assert_eq!(collapse_spaces_before_comment(input), expected);
	}

	#[test]
	fn test_hash_not_preceded_by_space() {
		// # directly after text is not a comment
		let input = "echo#notacomment";
		assert_eq!(collapse_spaces_before_comment(input), "echo#notacomment");
	}

	#[test]
	fn test_hash_at_start_of_line() {
		let input = "# this is a comment";
		assert_eq!(collapse_spaces_before_comment(input), "# this is a comment");
	}

	#[test]
	fn test_empty_line() {
		assert_eq!(collapse_spaces_before_comment(""), "");
	}

	#[test]
	fn test_line_with_only_spaces() {
		assert_eq!(collapse_spaces_before_comment("    "), "    ");
	}

	#[test]
	fn test_tab_before_comment() {
		let input = "echo hello\t# comment";
		let expected = "echo hello # comment";
		assert_eq!(collapse_spaces_before_comment(input), expected);
	}

	#[test]
	fn test_comment_with_only_hash() {
		let input = "echo hello   #";
		let expected = "echo hello #";
		assert_eq!(collapse_spaces_before_comment(input), expected);
	}

	#[test]
	fn test_multiple_hash_signs() {
		// Second # is part of the comment text, not a new comment
		let input = "echo hello   # comment # more";
		let expected = "echo hello # comment # more";
		assert_eq!(collapse_spaces_before_comment(input), expected);
	}

	#[test]
	fn test_escaped_hash() {
		// Escaped # should not be treated as comment start
		let input = r"echo \#notcomment";
		assert_eq!(collapse_spaces_before_comment(input), input);
	}

	#[test]
	fn test_is_shell_language_txt() {
		assert!(!is_shell_language("txt"));
	}

	#[test]
	fn test_comment_with_trailing_spaces() {
		let input = "echo hello   # comment   ";
		let expected = "echo hello # comment";
		assert_eq!(collapse_spaces_before_comment(input), expected);
	}

	#[test]
	fn test_mixed_tabs_and_spaces() {
		let input = "echo hello\t\t   # comment";
		// Tabs and spaces are both trimmed before comment
		let expected = "echo hello # comment";
		assert_eq!(collapse_spaces_before_comment(input), expected);
	}

	#[test]
	fn test_comment_after_pipe() {
		let input = "cat file | grep foo   # filter results";
		let expected = "cat file | grep foo # filter results";
		assert_eq!(collapse_spaces_before_comment(input), expected);
	}

	#[test]
	fn test_comment_after_command_no_space_before_hash() {
		// # directly attached to text is not a comment
		let input = "echo hello#notacomment";
		assert_eq!(
			collapse_spaces_before_comment(input),
			"echo hello#notacomment"
		);
	}

	#[test]
	fn test_only_hash_sign() {
		let input = "#";
		assert_eq!(collapse_spaces_before_comment(input), "#");
	}

	#[test]
	fn test_hash_with_only_spaces_after() {
		let input = "echo hello   #   ";
		let expected = "echo hello #";
		assert_eq!(collapse_spaces_before_comment(input), expected);
	}

	#[test]
	fn test_long_spaces_before_comment() {
		let input = "echo hello                    # far comment";
		let expected = "echo hello # far comment";
		assert_eq!(collapse_spaces_before_comment(input), expected);
	}

	#[test]
	fn test_comment_with_unicode() {
		let input = "echo hello   # comment with unicode: 你好 🎉";
		let expected = "echo hello # comment with unicode: 你好 🎉";
		assert_eq!(collapse_spaces_before_comment(input), expected);
	}

	#[test]
	fn test_hash_in_url_not_comment() {
		// URL with # is not a comment since no space before #
		let input = "curl https://example.com/page#section";
		assert_eq!(collapse_spaces_before_comment(input), input);
	}

	#[test]
	fn test_backtick_in_single_quotes_with_comment() {
		let input = "echo '`not code`'   # real comment";
		let expected = "echo '`not code`' # real comment";
		assert_eq!(collapse_spaces_before_comment(input), expected);
	}

	#[test]
	fn test_hash_in_backticks_not_comment() {
		// Backticks don't prevent # from being treated as comment
		// since backtick is not a quote character in this function
		let input = "echo `hello # world`   # real comment";
		let expected = "echo `hello # world` # real comment";
		assert_eq!(collapse_spaces_before_comment(input), expected);
	}

	#[test]
	fn test_nested_quotes_comment() {
		let input = "echo \"it's a 'test'\"   # comment";
		let expected = "echo \"it's a 'test'\" # comment";
		assert_eq!(collapse_spaces_before_comment(input), expected);
	}
}
