use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

/// Candidate configuration file names in resolution priority order.
pub const CONFIG_FILES: &[&str] = &[".agent-md.json", "agent-md.json", ".markdownlint.json"];

/// Maximum line length default (0 means disabled).
const DEFAULT_MAX_LINE_LENGTH: u64 = 0;

/// Configuration status representation for JSON output.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ConfigStatus {
	pub exists: bool,
	pub path: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub config: Option<serde_json::Value>,
}

/// Fully resolved configuration with all options and their effective values.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ResolvedConfig {
	pub blanks_around_headings: bool,
	pub blanks_around_lists: bool,
	pub blanks_around_fences: bool,
	pub blanks_around_tables: bool,
	pub first_line_heading: bool,
	pub no_duplicate_heading: bool,
	pub no_duplicate_headings: bool,
	pub line_length: bool,
	pub max_line_length: u64,
	pub ol_prefix: bool,
	pub table_column_style: bool,
	pub no_hard_tabs: bool,
	pub no_inline_html: bool,
}

impl Default for ResolvedConfig {
	fn default() -> Self {
		Self {
			blanks_around_headings: true,
			blanks_around_lists: true,
			blanks_around_fences: true,
			blanks_around_tables: false,
			first_line_heading: true,
			no_duplicate_heading: true,
			no_duplicate_headings: true,
			line_length: false,
			max_line_length: DEFAULT_MAX_LINE_LENGTH,
			ol_prefix: false,
			table_column_style: false,
			no_hard_tabs: true,
			no_inline_html: false,
		}
	}
}

impl ResolvedConfig {
	/// Build a `ResolvedConfig` from an optional JSON value, falling back to defaults
	/// for any missing or invalid keys.
	pub fn from_json(config: Option<&serde_json::Value>) -> Self {
		let defaults = Self::default();
		let Some(cfg) = config else {
			return defaults;
		};

		let get_bool = |key: &str, def: bool| -> bool {
			cfg.get(key).and_then(|v| v.as_bool()).unwrap_or(def)
		};
		let get_u64 =
			|key: &str, def: u64| -> u64 { cfg.get(key).and_then(|v| v.as_u64()).unwrap_or(def) };

		let no_duplicate = match (
			cfg.get("no-duplicate-heading"),
			cfg.get("no-duplicate-headings"),
		) {
			(Some(v), _) if v.is_boolean() => v.as_bool().unwrap_or(defaults.no_duplicate_heading),
			(_, Some(v)) if v.is_boolean() => v.as_bool().unwrap_or(defaults.no_duplicate_headings),
			_ => defaults.no_duplicate_heading,
		};

		Self {
			blanks_around_headings: get_bool(
				"blanks-around-headings",
				defaults.blanks_around_headings,
			),
			blanks_around_lists: get_bool("blanks-around-lists", defaults.blanks_around_lists),
			blanks_around_fences: get_bool("blanks-around-fences", defaults.blanks_around_fences),
			blanks_around_tables: get_bool("blanks-around-tables", defaults.blanks_around_tables),
			first_line_heading: get_bool("first-line-heading", defaults.first_line_heading),
			no_duplicate_heading: no_duplicate,
			no_duplicate_headings: no_duplicate,
			line_length: get_bool("line-length", defaults.line_length),
			max_line_length: get_u64("max-line-length", defaults.max_line_length),
			ol_prefix: get_bool("ol-prefix", defaults.ol_prefix),
			table_column_style: get_bool("table-column-style", defaults.table_column_style),
			no_hard_tabs: get_bool("no-hard-tabs", defaults.no_hard_tabs),
			no_inline_html: get_bool("no-inline-html", defaults.no_inline_html),
		}
	}
}

/// Extract a boolean value from a JSON config object for the given key.
/// Falls back to `default` when the key is missing or not a boolean.
pub fn get_bool_config(config: Option<&serde_json::Value>, key: &str, default: bool) -> bool {
	config
		.and_then(|c| c.get(key))
		.and_then(|val| val.as_bool())
		.unwrap_or(default)
}

/// Extract a u64 value from a JSON config object for the given key.
/// Falls back to `default` when the key is missing or not a number.
pub fn get_u64_config(config: Option<&serde_json::Value>, key: &str, default: u64) -> u64 {
	config
		.and_then(|c| c.get(key))
		.and_then(|val| val.as_u64())
		.unwrap_or(default)
}

/// Extract a string value from a JSON config object for the given key.
/// Falls back to `default` when the key is missing or not a string.
pub fn get_string_config<'a>(
	config: Option<&'a serde_json::Value>,
	key: &str,
	default: &'a str,
) -> &'a str {
	config
		.and_then(|c| c.get(key))
		.and_then(|val| val.as_str())
		.unwrap_or(default)
}

/// Resolve a `ResolvedConfig` from a JSON value, falling back to defaults
/// for any missing or invalid keys.
pub fn resolve_config(config: Option<&serde_json::Value>) -> ResolvedConfig {
	ResolvedConfig::from_json(config)
}

/// Check if a configuration file exists.
pub fn has_config_file(custom_path: Option<&str>) -> bool {
	find_config_file(custom_path).is_some()
}

/// Find configuration file path based on priority or custom path.
pub fn find_config_file(custom_path: Option<&str>) -> Option<String> {
	if let Some(custom) = custom_path {
		let path = Path::new(custom);
		if path.is_file() {
			return Some(custom.to_string());
		}
		if path.is_dir() {
			return find_config_in_dir(path);
		}
		return None;
	}

	for &name in CONFIG_FILES {
		if Path::new(name).is_file() {
			return Some(name.to_string());
		}
	}
	None
}

fn find_config_in_dir(dir: &Path) -> Option<String> {
	for &name in CONFIG_FILES {
		let candidate = dir.join(name);
		if candidate.is_file() {
			return candidate.to_str().map(|s| s.to_string());
		}
	}
	None
}

/// Read and parse configuration file.
/// Returns (resolved_path, parsed_json) if found and valid JSON.
pub fn read_config(custom_path: Option<&str>) -> Option<(String, serde_json::Value)> {
	let path_str = find_config_file(custom_path)?;
	let content = fs::read_to_string(&path_str).ok()?;
	let json_val = serde_json::from_str(&content).ok()?;
	Some((path_str, json_val))
}

/// Get configuration JSON value if configuration file exists and is valid.
pub fn get_config(custom_path: Option<&str>) -> Option<serde_json::Value> {
	read_config(custom_path).map(|(_, val)| val)
}

/// Get configuration status including existence, path, and optional content.
pub fn get_config_status(custom_path: Option<&str>, include_content: bool) -> ConfigStatus {
	if let Some((path, val)) = read_config(custom_path) {
		ConfigStatus {
			exists: true,
			path: Some(path),
			config: if include_content { Some(val) } else { None },
		}
	} else if let Some(path) = find_config_file(custom_path) {
		// File exists on disk but failed to parse as valid JSON
		ConfigStatus {
			exists: true,
			path: Some(path),
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

/// Default template content for newly initialized configuration files.
pub const DEFAULT_CONFIG_TEMPLATE: &str = r#"{
	"default": true,
	"blanks-around-headings": true,
	"blanks-around-lists": true,
	"blanks-around-fences": true,
	"blanks-around-tables": false,
	"first-line-heading": true,
	"no-duplicate-heading": true,
	"no-duplicate-headings": true,
	"line-length": false,
	"max-line-length": 80,
	"ol-prefix": false,
	"table-column-style": false,
	"no-hard-tabs": true,
	"no-inline-html": false
}
"#;

/// Result of initializing a configuration file.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct InitConfigResult {
	pub success: bool,
	pub path: String,
	pub message: String,
}

/// Initialize a configuration file at the specified path or directory.
/// If `custom_path` is None, creates `.agent-md.json` in the current working directory.
/// If `custom_path` is a directory, creates `.agent-md.json` inside that directory.
/// If `force` is false and the target file already exists, returns an error without overwriting.
pub fn init_config(custom_path: Option<&str>, force: bool) -> Result<String, String> {
	let target_path = match custom_path {
		Some(p) => {
			let path = Path::new(p);
			if path.is_dir() {
				path.join(".agent-md.json")
			} else {
				path.to_path_buf()
			}
		}
		None => PathBuf::from(".agent-md.json"),
	};

	let path_str = target_path.display().to_string();

	if target_path.exists() && !force {
		return Err(format!(
			"Configuration file already exists at '{}'. Use --force to overwrite.",
			path_str
		));
	}

	if let Some(parent) = target_path.parent() {
		if !parent.as_os_str().is_empty() && !parent.exists() {
			if let Err(e) = fs::create_dir_all(parent) {
				return Err(format!(
					"Failed to create parent directory '{}': {}",
					parent.display(),
					e
				));
			}
		}
	}

	if let Err(e) = fs::write(&target_path, DEFAULT_CONFIG_TEMPLATE) {
		return Err(format!(
			"Failed to write configuration file '{}': {}",
			path_str, e
		));
	}

	Ok(path_str)
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::fs::File;
	use std::io::Write;

	// ---------- existing tests ----------

	#[test]
	fn test_find_config_file_priority() {
		let temp_dir = std::env::temp_dir().join("agent_md_test_config_priority");
		let _ = fs::remove_dir_all(&temp_dir);
		fs::create_dir_all(&temp_dir).unwrap();

		let md_lint = temp_dir.join(".markdownlint.json");
		let mut f = File::create(&md_lint).unwrap();
		writeln!(f, r#"{{"default": true}}"#).unwrap();

		let found = find_config_file(temp_dir.to_str());
		assert_eq!(found, Some(md_lint.to_str().unwrap().to_string()));

		let agent_md = temp_dir.join("agent-md.json");
		let mut f = File::create(&agent_md).unwrap();
		writeln!(f, r#"{{"blanks-around-headings": false}}"#).unwrap();

		let found = find_config_file(temp_dir.to_str());
		assert_eq!(found, Some(agent_md.to_str().unwrap().to_string()));

		let dot_agent_md = temp_dir.join(".agent-md.json");
		let mut f = File::create(&dot_agent_md).unwrap();
		writeln!(f, r#"{{"blanks-around-headings": true}}"#).unwrap();

		let found = find_config_file(temp_dir.to_str());
		assert_eq!(found, Some(dot_agent_md.to_str().unwrap().to_string()));

		let config_val = get_config(temp_dir.to_str());
		assert!(config_val.is_some());
		assert_eq!(
			config_val.unwrap().get("blanks-around-headings").unwrap(),
			&serde_json::Value::Bool(true)
		);

		let _ = fs::remove_dir_all(&temp_dir);
	}

	#[test]
	fn test_has_config_file() {
		let temp_dir = std::env::temp_dir().join("agent_md_test_has_config");
		let _ = fs::remove_dir_all(&temp_dir);
		fs::create_dir_all(&temp_dir).unwrap();

		assert!(!has_config_file(temp_dir.to_str()));

		let cfg_path = temp_dir.join(".agent-md.json");
		let mut f = File::create(&cfg_path).unwrap();
		writeln!(f, "{{}}").unwrap();

		assert!(has_config_file(temp_dir.to_str()));
		assert!(has_config_file(cfg_path.to_str()));

		let _ = fs::remove_dir_all(&temp_dir);
	}

	#[test]
	fn test_get_config_status() {
		let temp_dir = std::env::temp_dir().join("agent_md_test_status");
		let _ = fs::remove_dir_all(&temp_dir);
		fs::create_dir_all(&temp_dir).unwrap();

		let status = get_config_status(temp_dir.to_str(), true);
		assert!(!status.exists);
		assert_eq!(status.path, None);
		assert_eq!(status.config, None);

		let cfg_path = temp_dir.join("agent-md.json");
		let mut f = File::create(&cfg_path).unwrap();
		writeln!(f, r#"{{"test-key": "test-val"}}"#).unwrap();

		let status_with_content = get_config_status(temp_dir.to_str(), true);
		assert!(status_with_content.exists);
		assert!(status_with_content.path.is_some());
		assert!(status_with_content.config.is_some());

		let status_check_only = get_config_status(temp_dir.to_str(), false);
		assert!(status_check_only.exists);
		assert!(status_check_only.path.is_some());
		assert_eq!(status_check_only.config, None);

		let _ = fs::remove_dir_all(&temp_dir);
	}

	// ---------- ResolvedConfig tests ----------

	#[test]
	fn test_resolved_config_defaults() {
		let cfg = ResolvedConfig::default();
		assert!(cfg.blanks_around_headings);
		assert!(cfg.blanks_around_lists);
		assert!(cfg.blanks_around_fences);
		assert!(!cfg.blanks_around_tables);
		assert!(cfg.first_line_heading);
		assert!(cfg.no_duplicate_heading);
		assert!(cfg.no_duplicate_headings);
		assert!(!cfg.line_length);
		assert_eq!(cfg.max_line_length, 0);
		assert!(!cfg.ol_prefix);
		assert!(!cfg.table_column_style);
		assert!(cfg.no_hard_tabs);
		assert!(!cfg.no_inline_html);
	}

	#[test]
	fn test_resolve_config_none_returns_defaults() {
		let cfg = resolve_config(None);
		assert_eq!(cfg, ResolvedConfig::default());
	}

	#[test]
	fn test_resolve_config_empty_json_returns_defaults() {
		let json = serde_json::json!({});
		let cfg = resolve_config(Some(&json));
		assert_eq!(cfg, ResolvedConfig::default());
	}

	#[test]
	fn test_resolve_config_override_all_booleans() {
		let json = serde_json::json!({
			"blanks-around-headings": false,
			"blanks-around-lists": false,
			"blanks-around-fences": false,
			"blanks-around-tables": true,
			"first-line-heading": false,
			"no-duplicate-heading": false,
			"no-duplicate-headings": false,
			"line-length": true,
			"max-line-length": 120,
			"ol-prefix": true,
			"table-column-style": true,
			"no-hard-tabs": false,
			"no-inline-html": true
		});
		let cfg = resolve_config(Some(&json));
		assert!(!cfg.blanks_around_headings);
		assert!(!cfg.blanks_around_lists);
		assert!(!cfg.blanks_around_fences);
		assert!(cfg.blanks_around_tables);
		assert!(!cfg.first_line_heading);
		assert!(!cfg.no_duplicate_heading);
		assert!(!cfg.no_duplicate_headings);
		assert!(cfg.line_length);
		assert_eq!(cfg.max_line_length, 120);
		assert!(cfg.ol_prefix);
		assert!(cfg.table_column_style);
		assert!(!cfg.no_hard_tabs);
		assert!(cfg.no_inline_html);
	}

	#[test]
	fn test_resolve_config_partial_override() {
		let json = serde_json::json!({
			"blanks-around-headings": false,
			"no-hard-tabs": false
		});
		let cfg = resolve_config(Some(&json));
		assert!(!cfg.blanks_around_headings);
		assert!(!cfg.no_hard_tabs);
		// All other values should remain defaults
		assert!(cfg.blanks_around_lists);
		assert!(cfg.blanks_around_fences);
		assert!(!cfg.blanks_around_tables);
		assert!(cfg.first_line_heading);
		assert!(cfg.no_duplicate_heading);
		assert!(cfg.no_duplicate_headings);
		assert!(!cfg.line_length);
		assert_eq!(cfg.max_line_length, 0);
		assert!(!cfg.ol_prefix);
		assert!(!cfg.table_column_style);
		assert!(!cfg.no_inline_html);
	}

	#[test]
	fn test_resolve_config_singular_duplicate_heading() {
		let json = serde_json::json!({
			"no-duplicate-heading": false
		});
		let cfg = resolve_config(Some(&json));
		assert!(!cfg.no_duplicate_heading);
		assert!(!cfg.no_duplicate_headings);
	}

	#[test]
	fn test_resolve_config_plural_duplicate_heading() {
		let json = serde_json::json!({
			"no-duplicate-headings": false
		});
		let cfg = resolve_config(Some(&json));
		assert!(!cfg.no_duplicate_heading);
		assert!(!cfg.no_duplicate_headings);
	}

	#[test]
	fn test_resolve_config_invalid_types_fallback_to_defaults() {
		let json = serde_json::json!({
			"blanks-around-headings": "not-a-bool",
			"max-line-length": "not-a-number",
			"line-length": 42,
			"first-line-heading": null
		});
		let cfg = resolve_config(Some(&json));
		// Invalid types should fall back to defaults
		assert!(cfg.blanks_around_headings);
		assert_eq!(cfg.max_line_length, 0);
		// Non-boolean number should also fallback
		assert!(cfg.first_line_heading);
		// line-length: 42 is a number, not bool, so falls back
		assert!(!cfg.line_length);
	}

	#[test]
	fn test_resolve_config_unknown_keys_ignored() {
		let json = serde_json::json!({
			"unknown-key": true,
			"another-unknown": 123,
			"blanks-around-headings": false
		});
		let cfg = resolve_config(Some(&json));
		assert!(!cfg.blanks_around_headings);
		// All other values should remain defaults
		assert!(cfg.blanks_around_lists);
	}

	// ---------- get_bool_config tests ----------

	#[test]
	fn test_get_bool_config_none_config() {
		assert!(get_bool_config(None, "blanks-around-headings", true));
		assert!(!get_bool_config(None, "some-key", false));
	}

	#[test]
	fn test_get_bool_config_missing_key() {
		let json = serde_json::json!({ "other": true });
		assert!(get_bool_config(Some(&json), "blanks-around-headings", true));
		assert!(!get_bool_config(Some(&json), "missing", false));
	}

	#[test]
	fn test_get_bool_config_valid_value() {
		let json = serde_json::json!({ "flag": false });
		assert!(!get_bool_config(Some(&json), "flag", true));

		let json = serde_json::json!({ "flag": true });
		assert!(get_bool_config(Some(&json), "flag", false));
	}

	#[test]
	fn test_get_bool_config_invalid_type_fallback() {
		let json = serde_json::json!({ "flag": "string" });
		assert!(get_bool_config(Some(&json), "flag", true));

		let json = serde_json::json!({ "flag": 123 });
		assert!(!get_bool_config(Some(&json), "flag", false));
	}

	// ---------- get_u64_config tests ----------

	#[test]
	fn test_get_u64_config_none_config() {
		assert_eq!(get_u64_config(None, "max-line-length", 80), 80);
	}

	#[test]
	fn test_get_u64_config_missing_key() {
		let json = serde_json::json!({ "other": 100 });
		assert_eq!(get_u64_config(Some(&json), "max-line-length", 80), 80);
	}

	#[test]
	fn test_get_u64_config_valid_value() {
		let json = serde_json::json!({ "max-line-length": 120 });
		assert_eq!(get_u64_config(Some(&json), "max-line-length", 80), 120);
	}

	#[test]
	fn test_get_u64_config_invalid_type_fallback() {
		let json = serde_json::json!({ "max-line-length": "not-a-number" });
		assert_eq!(get_u64_config(Some(&json), "max-line-length", 80), 80);

		let json = serde_json::json!({ "max-line-length": -5 });
		// Negative numbers are not valid u64, so fallback
		assert_eq!(get_u64_config(Some(&json), "max-line-length", 80), 80);
	}

	#[test]
	fn test_get_u64_config_zero_value() {
		let json = serde_json::json!({ "val": 0 });
		assert_eq!(get_u64_config(Some(&json), "val", 99), 0);
	}

	// ---------- get_string_config tests ----------

	#[test]
	fn test_get_string_config_none_config() {
		assert_eq!(get_string_config(None, "format", "markdown"), "markdown");
	}

	#[test]
	fn test_get_string_config_missing_key() {
		let json = serde_json::json!({ "other": "value" });
		assert_eq!(
			get_string_config(Some(&json), "format", "markdown"),
			"markdown"
		);
	}

	#[test]
	fn test_get_string_config_valid_value() {
		let json = serde_json::json!({ "format": "html" });
		assert_eq!(get_string_config(Some(&json), "format", "markdown"), "html");
	}

	#[test]
	fn test_get_string_config_invalid_type_fallback() {
		let json = serde_json::json!({ "format": 123 });
		assert_eq!(
			get_string_config(Some(&json), "format", "markdown"),
			"markdown"
		);
	}

	#[test]
	fn test_get_string_config_empty_string() {
		let json = serde_json::json!({ "format": "" });
		assert_eq!(get_string_config(Some(&json), "format", "markdown"), "");
	}

	// ---------- find_config_file edge cases ----------

	#[test]
	fn test_find_config_file_none_path() {
		// Without config files in cwd this returns None (or a cwd file)
		let result = find_config_file(None);
		// We just verify it doesn't panic
		let _ = result;
	}

	#[test]
	fn test_find_config_file_custom_path_to_file() {
		let temp_dir = std::env::temp_dir().join("agent_md_test_custom_file");
		let _ = fs::remove_dir_all(&temp_dir);
		fs::create_dir_all(&temp_dir).unwrap();

		let custom = temp_dir.join("my-config.json");
		let mut f = File::create(&custom).unwrap();
		writeln!(f, "{{}}").unwrap();

		let found = find_config_file(Some(custom.to_str().unwrap()));
		assert_eq!(found, Some(custom.to_str().unwrap().to_string()));

		let _ = fs::remove_dir_all(&temp_dir);
	}

	#[test]
	fn test_find_config_file_custom_path_to_nonexistent_file() {
		let result = find_config_file(Some("/nonexistent/path/config.json"));
		assert_eq!(result, None);
	}

	#[test]
	fn test_find_config_file_custom_path_to_empty_dir() {
		let temp_dir = std::env::temp_dir().join("agent_md_test_empty_dir");
		let _ = fs::remove_dir_all(&temp_dir);
		fs::create_dir_all(&temp_dir).unwrap();

		let found = find_config_file(Some(temp_dir.to_str().unwrap()));
		assert_eq!(found, None);

		let _ = fs::remove_dir_all(&temp_dir);
	}

	#[test]
	fn test_find_config_file_custom_dir_with_only_agent_md() {
		let temp_dir = std::env::temp_dir().join("agent_md_test_custom_dir_agent");
		let _ = fs::remove_dir_all(&temp_dir);
		fs::create_dir_all(&temp_dir).unwrap();

		let agent_md = temp_dir.join("agent-md.json");
		let mut f = File::create(&agent_md).unwrap();
		writeln!(f, "{{}}").unwrap();

		let found = find_config_file(Some(temp_dir.to_str().unwrap()));
		assert_eq!(found, Some(agent_md.to_str().unwrap().to_string()));

		let _ = fs::remove_dir_all(&temp_dir);
	}

	#[test]
	fn test_find_config_file_custom_dir_priority_order() {
		let temp_dir = std::env::temp_dir().join("agent_md_test_custom_dir_priority");
		let _ = fs::remove_dir_all(&temp_dir);
		fs::create_dir_all(&temp_dir).unwrap();

		// Only .markdownlint.json
		let md_lint = temp_dir.join(".markdownlint.json");
		let mut f = File::create(&md_lint).unwrap();
		writeln!(f, "{{}}").unwrap();
		assert_eq!(
			find_config_file(temp_dir.to_str()),
			Some(md_lint.to_str().unwrap().to_string())
		);

		// Add .agent-md.json (higher priority)
		let dot = temp_dir.join(".agent-md.json");
		let mut f = File::create(&dot).unwrap();
		writeln!(f, "{{}}").unwrap();
		assert_eq!(
			find_config_file(temp_dir.to_str()),
			Some(dot.to_str().unwrap().to_string())
		);

		let _ = fs::remove_dir_all(&temp_dir);
	}

	// ---------- read_config edge cases ----------

	#[test]
	fn test_read_config_invalid_json() {
		let temp_dir = std::env::temp_dir().join("agent_md_test_invalid_json");
		let _ = fs::remove_dir_all(&temp_dir);
		fs::create_dir_all(&temp_dir).unwrap();

		let cfg_path = temp_dir.join(".agent-md.json");
		let mut f = File::create(&cfg_path).unwrap();
		writeln!(f, "not valid json {{{{").unwrap();

		let result = read_config(temp_dir.to_str());
		assert!(result.is_none()); // Invalid JSON should return None

		let _ = fs::remove_dir_all(&temp_dir);
	}

	#[test]
	fn test_read_config_empty_file() {
		let temp_dir = std::env::temp_dir().join("agent_md_test_empty_file");
		let _ = fs::remove_dir_all(&temp_dir);
		fs::create_dir_all(&temp_dir).unwrap();

		let cfg_path = temp_dir.join(".agent-md.json");
		let mut f = File::create(&cfg_path).unwrap();
		// Empty file is valid JSON (empty string)
		writeln!(f).unwrap();

		let result = read_config(temp_dir.to_str());
		assert!(result.is_none()); // Empty file is not valid JSON

		let _ = fs::remove_dir_all(&temp_dir);
	}

	#[test]
	fn test_read_config_valid_complex_json() {
		let temp_dir = std::env::temp_dir().join("agent_md_test_complex_json");
		let _ = fs::remove_dir_all(&temp_dir);
		fs::create_dir_all(&temp_dir).unwrap();

		let cfg_path = temp_dir.join("agent-md.json");
		let mut f = File::create(&cfg_path).unwrap();
		writeln!(
			f,
			r#"{{"blanks-around-headings": true, "max-line-length": 100, "nested": {{"key": "val"}}}}"#
		)
		.unwrap();

		let result = read_config(temp_dir.to_str());
		assert!(result.is_some());
		let (path, val) = result.unwrap();
		assert_eq!(path, cfg_path.to_str().unwrap());
		assert_eq!(val.get("blanks-around-headings").unwrap(), true);
		assert_eq!(val.get("max-line-length").unwrap(), 100);
		assert!(val.get("nested").is_some());

		let _ = fs::remove_dir_all(&temp_dir);
	}

	// ---------- get_config_status edge cases ----------

	#[test]
	fn test_get_config_status_invalid_json_file() {
		let temp_dir = std::env::temp_dir().join("agent_md_test_status_invalid");
		let _ = fs::remove_dir_all(&temp_dir);
		fs::create_dir_all(&temp_dir).unwrap();

		let cfg_path = temp_dir.join(".agent-md.json");
		let mut f = File::create(&cfg_path).unwrap();
		writeln!(f, "not json").unwrap();

		let status = get_config_status(temp_dir.to_str(), true);
		assert!(status.exists);
		assert!(status.path.is_some());
		// Config should be None because JSON is invalid
		assert_eq!(status.config, None);

		let _ = fs::remove_dir_all(&temp_dir);
	}

	#[test]
	fn test_get_config_status_custom_path_nonexistent() {
		let status = get_config_status(Some("/nonexistent/path"), true);
		assert!(!status.exists);
		assert_eq!(status.path, None);
		assert_eq!(status.config, None);
	}

	// ---------- ResolvedConfig serialization ----------

	#[test]
	fn test_resolved_config_serialization() {
		let cfg = ResolvedConfig::default();
		let json = serde_json::to_string(&cfg).unwrap();
		assert!(json.contains("blanks_around_headings"));
		assert!(json.contains("max_line_length"));
		assert!(json.contains("no_hard_tabs"));
	}

	#[test]
	fn test_config_status_serialization() {
		let status = ConfigStatus {
			exists: true,
			path: Some("test.json".to_string()),
			config: Some(serde_json::json!({ "key": "val" })),
		};
		let json = serde_json::to_string(&status).unwrap();
		assert!(json.contains("exists"));
		assert!(json.contains("path"));
		assert!(json.contains("config"));
	}

	#[test]
	fn test_config_status_serialization_skip_none_config() {
		let status = ConfigStatus {
			exists: true,
			path: Some("test.json".to_string()),
			config: None,
		};
		let json = serde_json::to_string(&status).unwrap();
		assert!(!json.contains("config"));
	}

	// ---------- resolve_config with all sample config options ----------

	#[test]
	fn test_resolve_config_matches_sample_config() {
		// Match the sample .agent-md.json but with values flipped to false
		let json = serde_json::json!({
			"default": true,
			"blanks-around-headings": false,
			"blanks-around-lists": false,
			"blanks-around-fences": false,
			"blanks-around-tables": false,
			"first-line-heading": false,
			"no-duplicate-heading": false,
			"no-duplicate-headings": false,
			"line-length": false,
			"ol-prefix": false,
			"table-column-style": false,
			"no-hard-tabs": false,
			"no-inline-html": false
		});
		let cfg = resolve_config(Some(&json));
		assert!(!cfg.blanks_around_headings);
		assert!(!cfg.blanks_around_lists);
		assert!(!cfg.blanks_around_fences);
		assert!(!cfg.blanks_around_tables);
		assert!(!cfg.first_line_heading);
		assert!(!cfg.no_duplicate_heading);
		assert!(!cfg.no_duplicate_headings);
		assert!(!cfg.line_length);
		assert!(!cfg.ol_prefix);
		assert!(!cfg.table_column_style);
		assert!(!cfg.no_hard_tabs);
		assert!(!cfg.no_inline_html);
	}

	#[test]
	fn test_resolve_config_all_enabled() {
		let json = serde_json::json!({
			"blanks-around-headings": true,
			"blanks-around-lists": true,
			"blanks-around-fences": true,
			"blanks-around-tables": true,
			"first-line-heading": true,
			"no-duplicate-heading": true,
			"no-duplicate-headings": true,
			"line-length": true,
			"max-line-length": 200,
			"ol-prefix": true,
			"table-column-style": true,
			"no-hard-tabs": true,
			"no-inline-html": true
		});
		let cfg = resolve_config(Some(&json));
		assert!(cfg.blanks_around_headings);
		assert!(cfg.blanks_around_lists);
		assert!(cfg.blanks_around_fences);
		assert!(cfg.blanks_around_tables);
		assert!(cfg.first_line_heading);
		assert!(cfg.no_duplicate_heading);
		assert!(cfg.no_duplicate_headings);
		assert!(cfg.line_length);
		assert_eq!(cfg.max_line_length, 200);
		assert!(cfg.ol_prefix);
		assert!(cfg.table_column_style);
		assert!(cfg.no_hard_tabs);
		assert!(cfg.no_inline_html);
	}

	// ---------- has_config_file edge cases ----------

	#[test]
	fn test_has_config_file_none() {
		// Should not panic; result depends on cwd
		let _ = has_config_file(None);
	}

	#[test]
	fn test_has_config_file_nonexistent_custom_path() {
		assert!(!has_config_file(Some("/nonexistent/path")));
	}

	#[test]
	fn test_has_config_file_custom_path_to_existing_config() {
		let temp_dir = std::env::temp_dir().join("agent_md_test_has_existing");
		let _ = fs::remove_dir_all(&temp_dir);
		fs::create_dir_all(&temp_dir).unwrap();

		let cfg = temp_dir.join("agent-md.json");
		let mut f = File::create(&cfg).unwrap();
		writeln!(f, "{{}}").unwrap();

		assert!(has_config_file(temp_dir.to_str()));

		let _ = fs::remove_dir_all(&temp_dir);
	}

	// ---------- get_config edge cases ----------

	#[test]
	fn test_get_config_returns_none_when_no_file() {
		let temp_dir = std::env::temp_dir().join("agent_md_test_get_config_none");
		let _ = fs::remove_dir_all(&temp_dir);
		fs::create_dir_all(&temp_dir).unwrap();

		let result = get_config(temp_dir.to_str());
		assert!(result.is_none());

		let _ = fs::remove_dir_all(&temp_dir);
	}

	#[test]
	fn test_get_config_returns_value_when_file_exists() {
		let temp_dir = std::env::temp_dir().join("agent_md_test_get_config_some");
		let _ = fs::remove_dir_all(&temp_dir);
		fs::create_dir_all(&temp_dir).unwrap();

		let cfg = temp_dir.join(".agent-md.json");
		let mut f = File::create(&cfg).unwrap();
		writeln!(f, r#"{{"key": "value"}}"#).unwrap();

		let result = get_config(temp_dir.to_str());
		assert!(result.is_some());
		assert_eq!(
			result.unwrap().get("key").unwrap(),
			&serde_json::Value::String("value".to_string())
		);

		let _ = fs::remove_dir_all(&temp_dir);
	}

	// ---------- init_config tests ----------

	#[test]
	fn test_init_config_template_is_valid_json() {
		let parsed: serde_json::Value =
			serde_json::from_str(DEFAULT_CONFIG_TEMPLATE).expect("Template should be valid JSON");
		assert!(parsed.is_object());
		assert_eq!(parsed.get("default"), Some(&serde_json::json!(true)));

		let resolved = resolve_config(Some(&parsed));
		assert!(resolved.blanks_around_headings);
		assert!(resolved.blanks_around_lists);
		assert!(resolved.blanks_around_fences);
		assert!(!resolved.blanks_around_tables);
		assert!(resolved.first_line_heading);
		assert!(resolved.no_duplicate_heading);
		assert!(resolved.no_duplicate_headings);
		assert!(!resolved.line_length);
		assert_eq!(resolved.max_line_length, 80);
		assert!(resolved.no_hard_tabs);
	}

	#[test]
	fn test_init_config_creates_in_directory() {
		let temp_dir = std::env::temp_dir().join("agent_md_test_init_dir");
		let _ = fs::remove_dir_all(&temp_dir);
		fs::create_dir_all(&temp_dir).unwrap();

		let res = init_config(temp_dir.to_str(), false);
		assert!(res.is_ok());
		let created_path = res.unwrap();
		assert!(created_path.ends_with(".agent-md.json"));
		assert!(Path::new(&created_path).is_file());

		// Verify content
		let content = fs::read_to_string(&created_path).unwrap();
		assert_eq!(content, DEFAULT_CONFIG_TEMPLATE);

		let _ = fs::remove_dir_all(&temp_dir);
	}

	#[test]
	fn test_init_config_creates_custom_file_path() {
		let temp_dir = std::env::temp_dir().join("agent_md_test_init_custom");
		let _ = fs::remove_dir_all(&temp_dir);
		fs::create_dir_all(&temp_dir).unwrap();

		let custom_file = temp_dir.join("my-config.json");
		let res = init_config(custom_file.to_str(), false);
		assert!(res.is_ok());
		assert!(custom_file.is_file());

		let _ = fs::remove_dir_all(&temp_dir);
	}

	#[test]
	fn test_init_config_creates_nested_directories() {
		let temp_dir = std::env::temp_dir().join("agent_md_test_init_nested");
		let _ = fs::remove_dir_all(&temp_dir);

		let nested_file = temp_dir.join("sub").join("nested").join("agent-md.json");
		let res = init_config(nested_file.to_str(), false);
		assert!(res.is_ok());
		assert!(nested_file.is_file());

		let _ = fs::remove_dir_all(&temp_dir);
	}

	#[test]
	fn test_init_config_already_exists_fails_without_force() {
		let temp_dir = std::env::temp_dir().join("agent_md_test_init_exists");
		let _ = fs::remove_dir_all(&temp_dir);
		fs::create_dir_all(&temp_dir).unwrap();

		let file_path = temp_dir.join(".agent-md.json");
		fs::write(&file_path, "existing content").unwrap();

		let res = init_config(file_path.to_str(), false);
		assert!(res.is_err());
		let err_msg = res.unwrap_err();
		assert!(err_msg.contains("already exists"));

		// Verify existing content was not modified
		assert_eq!(fs::read_to_string(&file_path).unwrap(), "existing content");

		let _ = fs::remove_dir_all(&temp_dir);
	}

	#[test]
	fn test_init_config_already_exists_overwrites_with_force() {
		let temp_dir = std::env::temp_dir().join("agent_md_test_init_force");
		let _ = fs::remove_dir_all(&temp_dir);
		fs::create_dir_all(&temp_dir).unwrap();

		let file_path = temp_dir.join(".agent-md.json");
		fs::write(&file_path, "existing content").unwrap();

		let res = init_config(file_path.to_str(), true);
		assert!(res.is_ok());

		// Verify content was overwritten with default template
		assert_eq!(
			fs::read_to_string(&file_path).unwrap(),
			DEFAULT_CONFIG_TEMPLATE
		);

		let _ = fs::remove_dir_all(&temp_dir);
	}
}
