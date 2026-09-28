//! Configuration file discovery for agent-md.
//!
//! Locates native `agent-md.json` files and `markdownlintrc.*` fallbacks,
//! parses them, and resolves the effective configuration for a target path.
//! Extracted from `config` to keep type resolution separate from I/O.

use std::fs;
use std::path::Path;

use crate::config::{ConfigStatus, AGENT_MD_CONFIG_FILES, CONFIG_FILES, MARKDOWNLINT_CONFIG_FILES};
use crate::jsonc::strip_json_comments;

/// Check whether a path is a `markdownlintrc.*` configuration file.
pub fn is_markdownlint_config(path: &str) -> bool {
	let file_name = Path::new(path)
		.file_name()
		.and_then(|n| n.to_str())
		.unwrap_or(path);
	MARKDOWNLINT_CONFIG_FILES.contains(&file_name)
}

/// Candidate configuration file names for the given `ignore_markdownlintrc` setting.
///
/// When `ignore_markdownlintrc` is true, only native agent-md files are returned.
/// Defaults to false (do not ignore) to preserve existing behavior.
pub fn candidate_config_files(ignore_markdownlintrc: bool) -> &'static [&'static str] {
	if ignore_markdownlintrc {
		AGENT_MD_CONFIG_FILES
	} else {
		CONFIG_FILES
	}
}

/// Parse YAML content into JSON, accepting only mapping objects.
///
/// Requires an object so stray strings or empty files do not count as
/// valid configuration.
fn parse_yaml_object(content: &str) -> Option<serde_json::Value> {
	serde_yaml::from_str::<serde_json::Value>(content)
		.ok()
		.filter(|val| val.is_object())
}

/// Parse configuration file content based on file extension.
///
/// Supports JSON, JSONC (comments stripped), YAML, and extensionless
/// `.markdownlintrc` (tried as JSON, then JSONC, then YAML).
/// `.json` and `.jsonc` files stay strict JSON to preserve existing
/// invalid-file behavior; other extensions fall back to YAML.
pub fn parse_config_str(path_str: &str, content: &str) -> Option<serde_json::Value> {
	if path_str.ends_with(".yaml") || path_str.ends_with(".yml") {
		return parse_yaml_object(content);
	}
	if let Ok(val) = serde_json::from_str::<serde_json::Value>(content) {
		return Some(val);
	}
	let stripped = strip_json_comments(content);
	if let Ok(val) = serde_json::from_str::<serde_json::Value>(&stripped) {
		return Some(val);
	}
	if path_str.ends_with(".json") || path_str.ends_with(".jsonc") {
		return None;
	}
	parse_yaml_object(content)
}

/// Parse a configuration file at `path_str`.
fn parse_config_file(path_str: &str) -> Option<serde_json::Value> {
	let content = fs::read_to_string(path_str).ok()?;
	parse_config_str(path_str, &content)
}

/// Check if a configuration file exists.
pub fn has_config_file(custom_path: Option<&str>) -> bool {
	has_config_file_with_options(custom_path, false)
}

/// Check if a configuration file exists, optionally ignoring `markdownlintrc.*`.
pub fn has_config_file_with_options(
	custom_path: Option<&str>,
	ignore_markdownlintrc: bool,
) -> bool {
	find_config_file_with_options(custom_path, ignore_markdownlintrc).is_some()
}

/// Find configuration file path based on priority or custom path.
pub fn find_config_file(custom_path: Option<&str>) -> Option<String> {
	find_config_file_with_options(custom_path, false)
}

/// Check whether a parsed configuration value sets the `ignore-markdownlintrc` key.
///
/// Accepts both `ignore-markdownlintrc` (kebab-case) and
/// `ignore_markdownlintrc` (snake_case). Only native `agent-md.json`
/// configuration files are consulted for this key; `markdownlintrc.*`
/// files never set it.
pub fn config_value_ignores_markdownlintrc(value: &serde_json::Value) -> bool {
	matches!(
		value.get("ignore-markdownlintrc").and_then(|v| v.as_bool()),
		Some(true)
	) || matches!(
		value.get("ignore_markdownlintrc").and_then(|v| v.as_bool()),
		Some(true)
	)
}

fn find_config_file_raw(custom_path: Option<&str>, ignore_markdownlintrc: bool) -> Option<String> {
	if let Some(custom) = custom_path {
		let path = Path::new(custom);
		if path.is_file() {
			// Explicit user intent always beats discovery heuristics.
			return Some(custom.to_string());
		}
		if path.is_dir() {
			return find_config_in_dir(path, ignore_markdownlintrc);
		}
		return None;
	}

	for &name in candidate_config_files(ignore_markdownlintrc) {
		if Path::new(name).is_file() {
			return Some(name.to_string());
		}
	}
	None
}

/// Resolve the directory that ancestor search starts from for a target path.
///
/// Config files live in directories, never in the target file itself, so
/// files resolve via their parent while directories start at themselves.
/// A bare file name (empty parent) starts at the current directory.
fn ancestor_start_dir(target: &str) -> Option<&Path> {
	let path = Path::new(target);
	let dir = if path.is_dir() { path } else { path.parent()? };
	if dir.as_os_str().is_empty() {
		Some(Path::new("."))
	} else {
		Some(dir)
	}
}

fn find_config_for_target_raw(
	target_path: Option<&str>,
	custom_config: Option<&str>,
	ignore_markdownlintrc: bool,
) -> Option<String> {
	if let Some(custom) = custom_config {
		return find_config_file_raw(Some(custom), ignore_markdownlintrc);
	}

	if let Some(dir) = target_path.and_then(ancestor_start_dir) {
		if let Some(cfg) = find_config_in_ancestors_with_options(dir, ignore_markdownlintrc) {
			return Some(cfg);
		}
	}

	find_config_file_raw(None, ignore_markdownlintrc)
}

/// Resolve the effective `ignore-markdownlintrc` setting for a scope.
///
/// The CLI flag always wins. Otherwise the native `agent-md.json` file that
/// would be selected for the same scope (target ancestors or custom path,
/// current directory when neither is given) is inspected for the
/// `ignore-markdownlintrc` / `ignore_markdownlintrc` key. Defaults to false
/// (do not ignore) when no native file sets it.
pub fn get_effective_ignore_markdownlintrc(
	target_path: Option<&str>,
	custom_config: Option<&str>,
	cli_flag: bool,
) -> bool {
	if cli_flag {
		return true;
	}
	let native = find_config_for_target_raw(target_path, custom_config, true);
	match native {
		// Foreign markdownlint files must never control discovery: otherwise a
		// vendored config could silently opt itself out from under the user.
		Some(path) if !is_markdownlint_config(&path) => parse_config_file(&path)
			.is_some_and(|value| config_value_ignores_markdownlintrc(&value)),
		_ => false,
	}
}

/// Find configuration file path, optionally ignoring `markdownlintrc.*` files.
///
/// An explicit file path in `custom_path` is always respected, even when
/// `ignore_markdownlintrc` is true. The flag only affects automatic discovery.
/// When the flag is false, a native `agent-md.json` file in scope can still
/// enable ignoring via its own `ignore-markdownlintrc` key.
pub fn find_config_file_with_options(
	custom_path: Option<&str>,
	ignore_markdownlintrc: bool,
) -> Option<String> {
	let effective = get_effective_ignore_markdownlintrc(None, custom_path, ignore_markdownlintrc);
	find_config_file_raw(custom_path, effective)
}

fn find_config_in_dir(dir: &Path, ignore_markdownlintrc: bool) -> Option<String> {
	for &name in candidate_config_files(ignore_markdownlintrc) {
		let candidate = dir.join(name);
		if candidate.is_file() {
			return candidate.to_str().map(|s| s.to_string());
		}
	}
	None
}

/// Search for candidate configuration files in `dir` and walking up parent directories.
pub fn find_config_in_ancestors(dir: &Path) -> Option<String> {
	find_config_in_ancestors_with_options(dir, false)
}

/// Search ancestors, optionally ignoring `markdownlintrc.*` files.
pub fn find_config_in_ancestors_with_options(
	dir: &Path,
	ignore_markdownlintrc: bool,
) -> Option<String> {
	if let Some(cfg) = find_config_in_dir(dir, ignore_markdownlintrc) {
		return Some(cfg);
	}

	// Walk real ancestors, not path spellings: joining a relative dir onto
	// the cwd first ensures `parent()` climbs the actual filesystem tree.
	let Ok(current) = (if dir == Path::new(".") || dir.as_os_str().is_empty() {
		std::env::current_dir()
	} else if dir.is_relative() {
		std::env::current_dir().map(|cwd| cwd.join(dir))
	} else {
		Ok(dir.to_path_buf())
	}) else {
		return None;
	};

	let mut curr = current;
	while let Some(parent) = curr.parent() {
		if let Some(cfg) = find_config_in_dir(parent, ignore_markdownlintrc) {
			return Some(cfg);
		}
		curr = parent.to_path_buf();
	}

	None
}

/// Find configuration file for a given target path (file or directory).
///
/// Priority:
/// 1. `custom_config` if provided (explicit file or directory lookup).
/// 2. If `target_path` is provided, candidate configuration files (`.agent-md.json`,
///    `agent-md.json`, `.markdownlint.json`, plus other `markdownlintrc.*` variants
///    unless ignored) starting in the target's parent directory
///    and walking up parent directories.
/// 3. Current working directory default configuration.
pub fn find_config_for_target(
	target_path: Option<&str>,
	custom_config: Option<&str>,
) -> Option<String> {
	find_config_for_target_with_options(target_path, custom_config, false)
}

/// Find configuration for a target, optionally ignoring `markdownlintrc.*` files.
///
/// When `ignore_markdownlintrc` is false, a native `agent-md.json` file in
/// scope can still enable ignoring via its own `ignore-markdownlintrc` key.
pub fn find_config_for_target_with_options(
	target_path: Option<&str>,
	custom_config: Option<&str>,
	ignore_markdownlintrc: bool,
) -> Option<String> {
	let effective =
		get_effective_ignore_markdownlintrc(target_path, custom_config, ignore_markdownlintrc);
	find_config_for_target_raw(target_path, custom_config, effective)
}

/// Read and parse configuration file.
/// Returns (resolved_path, parsed_json) if found and valid.
pub fn read_config(custom_path: Option<&str>) -> Option<(String, serde_json::Value)> {
	read_config_with_options(custom_path, false)
}

/// Read and parse configuration file, optionally ignoring `markdownlintrc.*`.
pub fn read_config_with_options(
	custom_path: Option<&str>,
	ignore_markdownlintrc: bool,
) -> Option<(String, serde_json::Value)> {
	let path_str = find_config_file_with_options(custom_path, ignore_markdownlintrc)?;
	let json_val = parse_config_file(&path_str)?;
	Some((path_str, json_val))
}

/// Read and parse configuration file for a specific target path.
pub fn read_config_for_target(
	target_path: Option<&str>,
	custom_config: Option<&str>,
) -> Option<(String, serde_json::Value)> {
	read_config_for_target_with_options(target_path, custom_config, false)
}

/// Read configuration for a target, optionally ignoring `markdownlintrc.*`.
pub fn read_config_for_target_with_options(
	target_path: Option<&str>,
	custom_config: Option<&str>,
	ignore_markdownlintrc: bool,
) -> Option<(String, serde_json::Value)> {
	let path_str =
		find_config_for_target_with_options(target_path, custom_config, ignore_markdownlintrc)?;
	let json_val = parse_config_file(&path_str)?;
	Some((path_str, json_val))
}

/// Get configuration JSON value if configuration file exists and is valid.
pub fn get_config(custom_path: Option<&str>) -> Option<serde_json::Value> {
	get_config_with_options(custom_path, false)
}

/// Get configuration JSON value, optionally ignoring `markdownlintrc.*`.
///
/// A native `agent-md.json` file in scope can also enable ignoring via its
/// own `ignore-markdownlintrc` key, even when the flag argument is false.
pub fn get_config_with_options(
	custom_path: Option<&str>,
	ignore_markdownlintrc: bool,
) -> Option<serde_json::Value> {
	read_config_with_options(custom_path, ignore_markdownlintrc).map(|(_, val)| val)
}

/// Get configuration JSON value for a specific target path.
pub fn get_config_for_target(
	target_path: Option<&str>,
	custom_config: Option<&str>,
) -> Option<serde_json::Value> {
	get_config_for_target_with_options(target_path, custom_config, false)
}

/// Get configuration for a target, optionally ignoring `markdownlintrc.*`.
///
/// A native `agent-md.json` file in scope can also enable ignoring via its
/// own `ignore-markdownlintrc` key, even when the flag argument is false.
pub fn get_config_for_target_with_options(
	target_path: Option<&str>,
	custom_config: Option<&str>,
	ignore_markdownlintrc: bool,
) -> Option<serde_json::Value> {
	read_config_for_target_with_options(target_path, custom_config, ignore_markdownlintrc)
		.map(|(_, val)| val)
}

/// Get configuration status including existence, path, and optional content.
pub fn get_config_status(custom_path: Option<&str>, include_content: bool) -> ConfigStatus {
	get_config_status_with_options(custom_path, include_content, false)
}

/// Get configuration status, optionally ignoring `markdownlintrc.*` files.
pub fn get_config_status_with_options(
	custom_path: Option<&str>,
	include_content: bool,
	ignore_markdownlintrc: bool,
) -> ConfigStatus {
	// A markdown file path asks "which config applies to this file", while a
	// config filename is inspected directly; directories are searched as-is.
	let path_opt = if let Some(custom) = custom_path {
		let path = Path::new(custom);
		if path.is_file() {
			let is_cfg_name = CONFIG_FILES.iter().any(|&name| path.ends_with(name));
			if is_cfg_name {
				find_config_file_with_options(Some(custom), ignore_markdownlintrc)
			} else {
				find_config_for_target_with_options(Some(custom), None, ignore_markdownlintrc)
			}
		} else {
			find_config_file_with_options(Some(custom), ignore_markdownlintrc)
		}
	} else {
		find_config_file_with_options(None, ignore_markdownlintrc)
	};

	if let Some(path_str) = path_opt {
		if let Some(json_val) = parse_config_file(&path_str) {
			return ConfigStatus {
				exists: true,
				path: Some(path_str),
				config: if include_content {
					Some(json_val)
				} else {
					None
				},
			};
		}
		ConfigStatus {
			exists: true,
			path: Some(path_str),
			config: None,
		}
	} else {
		ConfigStatus {
			exists: false,
			path: None,
			config: None,
		}
	}
}
