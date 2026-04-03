/// Collapse multiple spaces before # comments in bash/sh code.
/// This handles cases like "cd              # goto" -> "cd # goto"
pub fn collapse_spaces_before_comment(line: &str) -> String {
	// Find the position of # that looks like a comment (preceded by space or at start)
	let mut comment_pos = None;
	let chars: Vec<char> = line.chars().collect();

	for i in 0..chars.len() {
		if chars[i] == '#' {
			// Check if it's a comment: at start or preceded by space(s)
			// But not inside a string - simple heuristic: check for unescaped quotes before
			if i == 0 || chars[i - 1] == ' ' || chars[i - 1] == '\t' {
				// Simple check: count unescaped quotes before this position
				let mut single_quotes = 0;
				let mut double_quotes = 0;
				let mut escaped = false;
				for j in 0..i {
					if escaped {
						escaped = false;
						continue;
					}
					if chars[j] == '\\' {
						escaped = true;
						continue;
					}
					if chars[j] == '\'' && double_quotes % 2 == 0 {
						single_quotes += 1;
					}
					if chars[j] == '"' && single_quotes % 2 == 0 {
						double_quotes += 1;
					}
				}
				// If we're inside quotes, this # is not a comment
				if single_quotes % 2 == 0 && double_quotes % 2 == 0 {
					comment_pos = Some(i);
					break;
				}
			}
		}
	}

	if let Some(pos) = comment_pos {
		let before = &line[..pos];
		let comment = &line[pos..];

		// Collapse trailing spaces before the comment
		let before_trimmed = before.trim_end();
		format!("{} {}", before_trimmed, comment)
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
		assert!(!is_shell_language("rust"));
	}
}
