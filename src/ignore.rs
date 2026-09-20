//! Ignore pattern parsing, resolution, and matching for `agent-md`.
//!
//! Supports reading `.markdownlintignore` and `.gitignore` files, merging them,
//! deduplicating entries, and checking whether files or directories should be ignored.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Default file name for Markdownlint ignore rules.
pub const MARKDOWNLINT_IGNORE_FILE: &str = ".markdownlintignore";

/// Default file name for Git ignore rules.
pub const GIT_IGNORE_FILE: &str = ".gitignore";

/// Read ignore patterns from a file.
///
/// Trims whitespace, skips empty lines, and skips comments (lines starting with `#`).
pub fn read_ignore_file(path: &Path) -> Vec<String> {
	let Ok(content) = fs::read_to_string(path) else {
		return Vec::new();
	};

	content
		.lines()
		.map(|line| line.trim())
		.filter(|line| !line.is_empty() && !line.starts_with('#'))
		.map(|line| line.to_string())
		.collect()
}

/// Read `.markdownlintignore` if it exists in the specified directory.
pub fn get_markdownlint_ignore_in_dir(dir: &Path) -> Vec<String> {
	read_ignore_file(&dir.join(MARKDOWNLINT_IGNORE_FILE))
}

/// Read `.markdownlintignore` if it exists in the current directory.
pub fn get_markdownlint_ignore() -> Vec<String> {
	get_markdownlint_ignore_in_dir(Path::new("."))
}

/// Locate `.gitignore` in `dir` or by walking up parent directories.
pub fn find_git_ignore(dir: &Path) -> Option<PathBuf> {
	let candidate = dir.join(GIT_IGNORE_FILE);
	if candidate.is_file() {
		return Some(candidate);
	}

	let mut current = if dir == Path::new(".") || dir.as_os_str().is_empty() {
		let Ok(cwd) = std::env::current_dir() else {
			return None;
		};
		cwd
	} else if dir.is_relative() {
		let Ok(cwd) = std::env::current_dir() else {
			return None;
		};
		cwd.join(dir)
	} else {
		dir.to_path_buf()
	};

	loop {
		let gitignore = current.join(GIT_IGNORE_FILE);
		if gitignore.is_file() {
			return Some(gitignore);
		}
		if current.join(".git").exists() {
			break;
		}
		if !current.pop() {
			break;
		}
	}

	None
}

/// Read Git ignore patterns in `dir` or from parent repositories.
pub fn get_git_ignore_in_dir(dir: &Path) -> Vec<String> {
	match find_git_ignore(dir) {
		Some(path) => read_ignore_file(&path),
		None => Vec::new(),
	}
}

/// Read Git ignore patterns for the current working directory.
pub fn get_git_ignore() -> Vec<String> {
	get_git_ignore_in_dir(Path::new("."))
}

/// Merge two ignore lists and remove duplicate items while preserving order.
pub fn merge_ignore_lists(primary: &[String], secondary: &[String]) -> Vec<String> {
	let mut result = Vec::new();
	let mut seen = HashSet::new();

	for item in primary {
		if seen.insert(item.clone()) {
			result.push(item.clone());
		}
	}

	for item in secondary {
		if seen.insert(item.clone()) {
			result.push(item.clone());
		}
	}

	result
}

/// Get the combined, deduplicated ignore list for a directory.
///
/// Merges `.markdownlintignore` patterns with Git ignore patterns found in `dir`.
pub fn get_ignore_list_in_dir(dir: &Path) -> Vec<String> {
	let markdownlint_list = get_markdownlint_ignore_in_dir(dir);
	let git_list = get_git_ignore_in_dir(dir);
	merge_ignore_lists(&markdownlint_list, &git_list)
}

/// Get the combined, deduplicated ignore list for the current working directory.
pub fn get_ignore_list() -> Vec<String> {
	get_ignore_list_in_dir(Path::new("."))
}

/// Match simple wildcard patterns containing `*` and `?`.
pub fn wildcard_match(pattern: &str, text: &str) -> bool {
	let p: Vec<char> = pattern.chars().collect();
	let t: Vec<char> = text.chars().collect();
	let mut p_idx = 0;
	let mut t_idx = 0;
	let mut star_idx = None;
	let mut match_idx = 0;

	while t_idx < t.len() {
		if p_idx < p.len() && (p[p_idx] == '?' || p[p_idx] == t[t_idx]) {
			p_idx += 1;
			t_idx += 1;
		} else if p_idx < p.len() && p[p_idx] == '*' {
			star_idx = Some(p_idx);
			match_idx = t_idx;
			p_idx += 1;
		} else if let Some(star) = star_idx {
			p_idx = star + 1;
			match_idx += 1;
			t_idx = match_idx;
		} else {
			return false;
		}
	}

	while p_idx < p.len() && p[p_idx] == '*' {
		p_idx += 1;
	}

	p_idx == p.len()
}

/// Match a single pattern against a cleaned relative path.
fn match_single_pattern(pattern: &str, clean_rel: &str, is_dir: bool) -> bool {
	let is_dir_only = pattern.ends_with('/');
	let pat_without_slash = pattern.trim_end_matches('/');
	let is_rooted = pat_without_slash.starts_with('/');
	let clean_pat = pat_without_slash.trim_start_matches('/');

	if clean_pat.is_empty() {
		return false;
	}

	if is_rooted || clean_pat.contains('/') {
		if is_dir_only {
			if is_dir && clean_rel == clean_pat {
				return true;
			}
			return clean_rel.starts_with(&format!("{}/", clean_pat));
		}
		if clean_rel == clean_pat
			|| wildcard_match(clean_pat, clean_rel)
			|| clean_rel.starts_with(&format!("{}/", clean_pat))
		{
			return true;
		}
		return false;
	}

	let components: Vec<&str> = clean_rel.split('/').collect();
	if is_dir_only {
		if is_dir && components.iter().any(|c| wildcard_match(clean_pat, c)) {
			return true;
		}
		if components.len() > 1
			&& components[..components.len() - 1]
				.iter()
				.any(|c| wildcard_match(clean_pat, c))
		{
			return true;
		}
		return false;
	}

	components.iter().any(|c| wildcard_match(clean_pat, c))
}

/// Check if a path should be ignored according to an ignore list.
pub fn is_ignored(path: &Path, base_dir: &Path, ignore_list: &[String]) -> bool {
	if ignore_list.is_empty() {
		return false;
	}

	let rel_path = if base_dir == Path::new(".") || base_dir.as_os_str().is_empty() {
		path
	} else {
		match path.strip_prefix(base_dir) {
			Ok(p) => p,
			Err(_) => path,
		}
	};

	let rel_str = rel_path.to_string_lossy().replace('\\', "/");
	let clean_rel = rel_str.trim_start_matches("./");
	if clean_rel.is_empty() || clean_rel == "." {
		return false;
	}

	let is_dir = path.is_dir();
	let mut ignored = false;

	for pattern in ignore_list {
		let trimmed = pattern.trim();
		if trimmed.is_empty() || trimmed.starts_with('#') {
			continue;
		}

		let (negated, pat) = if let Some(stripped) = trimmed.strip_prefix('!') {
			(true, stripped)
		} else {
			(false, trimmed)
		};

		if match_single_pattern(pat, clean_rel, is_dir) {
			ignored = !negated;
		}
	}

	ignored
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::fs::File;
	use std::io::Write;

	#[test]
	fn test_wildcard_match() {
		assert!(wildcard_match("*.log", "error.log"));
		assert!(wildcard_match("*.log", "test.log"));
		assert!(!wildcard_match("*.log", "error.md"));
		assert!(wildcard_match("test-?.md", "test-1.md"));
		assert!(!wildcard_match("test-?.md", "test-12.md"));
		assert!(wildcard_match("*", "anything"));
		assert!(wildcard_match("target", "target"));
	}

	#[test]
	fn test_read_ignore_file() {
		let temp_dir = std::env::temp_dir().join("agent_md_test_read_ignore");
		let _ = fs::remove_dir_all(&temp_dir);
		fs::create_dir_all(&temp_dir).unwrap();

		let file_path = temp_dir.join(".markdownlintignore");
		let mut f = File::create(&file_path).unwrap();
		writeln!(f, "# A comment line").unwrap();
		writeln!(f).unwrap();
		writeln!(f, "  dist/  ").unwrap();
		writeln!(f, "logs/").unwrap();
		writeln!(f, "target/").unwrap();

		let items = read_ignore_file(&file_path);
		assert_eq!(items, vec!["dist/", "logs/", "target/"]);

		let _ = fs::remove_dir_all(&temp_dir);
	}

	#[test]
	fn test_merge_ignore_lists_deduplicated() {
		let list_a = vec![
			"dist/".to_string(),
			"logs/".to_string(),
			"target/".to_string(),
		];
		let list_b = vec![
			"/target".to_string(),
			"temp/".to_string(),
			"*.log".to_string(),
			"logs/".to_string(),
			".antigravitycli".to_string(),
		];

		let merged = merge_ignore_lists(&list_a, &list_b);
		assert_eq!(
			merged,
			vec![
				"dist/".to_string(),
				"logs/".to_string(),
				"target/".to_string(),
				"/target".to_string(),
				"temp/".to_string(),
				"*.log".to_string(),
				".antigravitycli".to_string(),
			]
		);
	}

	#[test]
	fn test_get_ignore_list_in_dir() {
		let temp_dir = std::env::temp_dir().join("agent_md_test_get_ignore_in_dir");
		let _ = fs::remove_dir_all(&temp_dir);
		fs::create_dir_all(&temp_dir).unwrap();

		let md_ignore = temp_dir.join(".markdownlintignore");
		let mut f1 = File::create(&md_ignore).unwrap();
		writeln!(f1, "build/").unwrap();
		writeln!(f1, "logs/").unwrap();

		let git_ignore = temp_dir.join(".gitignore");
		let mut f2 = File::create(&git_ignore).unwrap();
		writeln!(f2, "logs/").unwrap();
		writeln!(f2, "*.tmp").unwrap();

		let merged = get_ignore_list_in_dir(&temp_dir);
		assert_eq!(
			merged,
			vec![
				"build/".to_string(),
				"logs/".to_string(),
				"*.tmp".to_string()
			]
		);

		let _ = fs::remove_dir_all(&temp_dir);
	}

	#[test]
	fn test_is_ignored_directories_and_wildcards() {
		let ignore_list = vec![
			"dist/".to_string(),
			"logs/".to_string(),
			"/target".to_string(),
			"*.log".to_string(),
			"!important.log".to_string(),
		];

		let base = Path::new(".");
		assert!(is_ignored(Path::new("dist/bundle.js"), base, &ignore_list));
		assert!(is_ignored(Path::new("logs/app.log"), base, &ignore_list));
		assert!(is_ignored(
			Path::new("target/debug/agent-md"),
			base,
			&ignore_list
		));
		assert!(is_ignored(
			Path::new("sub/logs/deep.md"),
			base,
			&ignore_list
		));
		assert!(is_ignored(Path::new("server.log"), base, &ignore_list));
		assert!(!is_ignored(Path::new("important.log"), base, &ignore_list));
		assert!(!is_ignored(Path::new("src/main.rs"), base, &ignore_list));
		assert!(!is_ignored(Path::new("README.md"), base, &ignore_list));
	}
}
