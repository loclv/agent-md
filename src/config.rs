use crate::json::{JsonObject, JsonValue, ToJson};
use std::fs;
use std::path::{Path, PathBuf};

/// Agent-md native configuration file names in resolution priority order.
pub const AGENT_MD_CONFIG_FILES: &[&str] = &[".agent-md.json", "agent-md.json"];

/// Markdownlint configuration file names in resolution priority order.
///
/// Covers `markdownlintrc.*` variants supported by the markdownlint ecosystem
/// (`.markdownlint.json`, `.markdownlint.jsonc`, `.markdownlint.yaml`,
/// `.markdownlint.yml`, `.markdownlintrc`, `.markdownlintrc.json`).
pub const MARKDOWNLINT_CONFIG_FILES: &[&str] = &[
	".markdownlint.json",
	".markdownlint.jsonc",
	".markdownlint.yaml",
	".markdownlint.yml",
	".markdownlintrc",
	".markdownlintrc.json",
];

/// Candidate configuration file names in resolution priority order.
///
/// Agent-md files take precedence over `markdownlintrc.*` fallbacks.
pub const CONFIG_FILES: &[&str] = &[
	".agent-md.json",
	"agent-md.json",
	".markdownlint.json",
	".markdownlint.jsonc",
	".markdownlint.yaml",
	".markdownlint.yml",
	".markdownlintrc",
	".markdownlintrc.json",
];

/// Maximum line length default (0 means disabled).
const DEFAULT_MAX_LINE_LENGTH: u64 = 0;

/// Configuration status representation for JSON output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigStatus {
	pub exists: bool,
	pub path: Option<String>,
	pub config: Option<JsonValue>,
}

impl ToJson for ConfigStatus {
	fn to_json(&self) -> JsonValue {
		let mut obj = JsonObject::new();
		obj.insert("exists", self.exists);
		match &self.path {
			Some(p) => obj.insert("path", p.as_str()),
			None => obj.insert("path", JsonValue::Null),
		}
		if let Some(ref cfg) = self.config {
			obj.insert("config", cfg.clone());
		}
		JsonValue::Object(obj)
	}
}

/// Fully resolved configuration with all options and their effective values.
#[derive(Debug, Clone, PartialEq, Eq)]
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
	pub ignore_markdownlintrc: bool,
	pub remove_bold: bool,
	pub compact_blank_lines: bool,
	pub collapse_spaces: bool,
	pub remove_horizontal_rules: bool,
	pub remove_emphasis: bool,
	pub minify_html: bool,
}

impl ToJson for ResolvedConfig {
	fn to_json(&self) -> JsonValue {
		let mut obj = JsonObject::new();
		obj.insert("blanks_around_headings", self.blanks_around_headings);
		obj.insert("blanks_around_lists", self.blanks_around_lists);
		obj.insert("blanks_around_fences", self.blanks_around_fences);
		obj.insert("blanks_around_tables", self.blanks_around_tables);
		obj.insert("first_line_heading", self.first_line_heading);
		obj.insert("no_duplicate_heading", self.no_duplicate_heading);
		obj.insert("no_duplicate_headings", self.no_duplicate_headings);
		obj.insert("line_length", self.line_length);
		obj.insert("max_line_length", self.max_line_length);
		obj.insert("ol_prefix", self.ol_prefix);
		obj.insert("table_column_style", self.table_column_style);
		obj.insert("no_hard_tabs", self.no_hard_tabs);
		obj.insert("no_inline_html", self.no_inline_html);
		obj.insert("ignore_markdownlintrc", self.ignore_markdownlintrc);
		obj.insert("remove_bold", self.remove_bold);
		obj.insert("compact_blank_lines", self.compact_blank_lines);
		obj.insert("collapse_spaces", self.collapse_spaces);
		obj.insert("remove_horizontal_rules", self.remove_horizontal_rules);
		obj.insert("remove_emphasis", self.remove_emphasis);
		obj.insert("minify_html", self.minify_html);
		JsonValue::Object(obj)
	}
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
			ignore_markdownlintrc: false,
			remove_bold: true,
			compact_blank_lines: true,
			collapse_spaces: true,
			remove_horizontal_rules: true,
			remove_emphasis: true,
			minify_html: true,
		}
	}
}

impl ResolvedConfig {
	/// Build a `ResolvedConfig` from an optional JSON value, falling back to defaults
	/// for any missing or invalid keys.
	pub fn from_json(config: Option<&JsonValue>) -> Self {
		let defaults = Self::default();
		let Some(cfg) = config else {
			return defaults;
		};

		let get_bool = |key: &str, def: bool| -> bool {
			cfg.get(key).and_then(|v| v.as_bool()).unwrap_or(def)
		};
		let get_u64 =
			|key: &str, def: u64| -> u64 { cfg.get(key).and_then(|v| v.as_u64()).unwrap_or(def) };

		// Read a boolean accepting kebab-case or snake_case keys.
		// The kebab-case key wins when both are present; falls back to `def`.
		let get_bool_alias = |kebab: &str, snake: &str, def: bool| -> bool {
			match (cfg.get(kebab), cfg.get(snake)) {
				(Some(v), _) if v.is_boolean() => v.as_bool().unwrap_or(def),
				(_, Some(v)) if v.is_boolean() => v.as_bool().unwrap_or(def),
				_ => def,
			}
		};

		let get_format_bool = |k1: &str, k2: &str, def: bool| -> bool {
			let format_val = cfg
				.get("format")
				.and_then(|v| v.as_object())
				.and_then(|obj| obj.get(k1).or_else(|| obj.get(k2)))
				.and_then(|v| v.as_bool());
			if let Some(val) = format_val {
				return val;
			}
			get_bool_alias(k1, k2, def)
		};

		let no_duplicate = get_bool_alias(
			"no-duplicate-heading",
			"no-duplicate-headings",
			defaults.no_duplicate_heading,
		);

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
			ignore_markdownlintrc: get_bool_alias(
				"ignore-markdownlintrc",
				"ignore_markdownlintrc",
				defaults.ignore_markdownlintrc,
			),
			remove_bold: get_format_bool("remove-bold", "remove_bold", defaults.remove_bold),
			compact_blank_lines: get_format_bool(
				"compact-blank-lines",
				"compact_blank_lines",
				defaults.compact_blank_lines,
			),
			collapse_spaces: get_format_bool(
				"collapse-spaces",
				"collapse_spaces",
				defaults.collapse_spaces,
			),
			remove_horizontal_rules: get_format_bool(
				"remove-horizontal-rules",
				"remove_horizontal_rules",
				defaults.remove_horizontal_rules,
			),
			remove_emphasis: get_format_bool(
				"remove-emphasis",
				"remove_emphasis",
				defaults.remove_emphasis,
			),
			minify_html: get_format_bool("minify-html", "minify_html", defaults.minify_html),
		}
	}
}

/// Extract a boolean value from a JSON config object for the given key.
/// Falls back to `default` when the key is missing or not a boolean.
pub fn get_bool_config(config: Option<&JsonValue>, key: &str, default: bool) -> bool {
	config
		.and_then(|c| c.get(key))
		.and_then(|val| val.as_bool())
		.unwrap_or(default)
}

/// Extract a u64 value from a JSON config object for the given key.
/// Falls back to `default` when the key is missing or not a number.
pub fn get_u64_config(config: Option<&JsonValue>, key: &str, default: u64) -> u64 {
	config
		.and_then(|c| c.get(key))
		.and_then(|val| val.as_u64())
		.unwrap_or(default)
}

/// Extract a string value from a JSON config object for the given key.
/// Falls back to `default` when the key is missing or not a string.
pub fn get_string_config<'a>(
	config: Option<&'a JsonValue>,
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
pub fn resolve_config(config: Option<&JsonValue>) -> ResolvedConfig {
	ResolvedConfig::from_json(config)
}

pub use crate::config_discovery::{
	candidate_config_files, config_value_ignores_markdownlintrc, find_config_file,
	find_config_file_with_options, find_config_for_target, find_config_for_target_with_options,
	find_config_in_ancestors, find_config_in_ancestors_with_options, get_config,
	get_config_for_target, get_config_for_target_with_options, get_config_status,
	get_config_status_with_options, get_config_with_options, get_effective_ignore_markdownlintrc,
	has_config_file, has_config_file_with_options, is_markdownlint_config, parse_config_str,
	read_config, read_config_for_target, read_config_for_target_with_options,
	read_config_with_options,
};

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
	"no-inline-html": false,
	"ignore-markdownlintrc": false,
	"remove-bold": true,
	"compact-blank-lines": true,
	"collapse-spaces": true,
	"remove-horizontal-rules": true,
	"remove-emphasis": true,
	"minify-html": true
}
"#;

/// Result of initializing a configuration file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitConfigResult {
	pub success: bool,
	pub path: String,
	pub message: String,
}

impl ToJson for InitConfigResult {
	fn to_json(&self) -> JsonValue {
		let mut obj = JsonObject::new();
		obj.insert("success", self.success);
		obj.insert("path", self.path.as_str());
		obj.insert("message", self.message.as_str());
		JsonValue::Object(obj)
	}
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

	// Create missing parents so `init nested/dir/agent-md.json` works without
	// pre-creating the directory tree by hand.
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

	/// Isolated temp directory for tests, removed on drop.
	///
	/// Derefs to `PathBuf`, so `join`, `to_str`, and friends keep working.
	struct TestDir(PathBuf);

	impl TestDir {
		fn new(name: &str) -> Self {
			let dir = std::env::temp_dir().join(name);
			let _ = fs::remove_dir_all(&dir);
			fs::create_dir_all(&dir).unwrap();
			Self(dir)
		}
	}

	impl std::ops::Deref for TestDir {
		type Target = PathBuf;

		fn deref(&self) -> &Self::Target {
			&self.0
		}
	}

	impl Drop for TestDir {
		fn drop(&mut self) {
			let _ = fs::remove_dir_all(&self.0);
		}
	}

	// ---------- existing tests ----------

	#[test]
	fn test_find_config_file_priority() {
		let temp_dir = TestDir::new("agent_md_test_config_priority");

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
			&JsonValue::Bool(true)
		);
	}

	#[test]
	fn test_has_config_file() {
		let temp_dir = TestDir::new("agent_md_test_has_config");

		assert!(!has_config_file(temp_dir.to_str()));

		let cfg_path = temp_dir.join(".agent-md.json");
		let mut f = File::create(&cfg_path).unwrap();
		writeln!(f, "{{}}").unwrap();

		assert!(has_config_file(temp_dir.to_str()));
		assert!(has_config_file(cfg_path.to_str()));
	}

	#[test]
	fn test_get_config_status() {
		let temp_dir = TestDir::new("agent_md_test_status");

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
		assert!(cfg.remove_bold);
		assert!(cfg.compact_blank_lines);
		assert!(cfg.collapse_spaces);
		assert!(cfg.remove_horizontal_rules);
		assert!(cfg.remove_emphasis);
		assert!(cfg.minify_html);
	}

	#[test]
	fn test_resolve_config_none_returns_defaults() {
		let cfg = resolve_config(None);
		assert_eq!(cfg, ResolvedConfig::default());
	}

	#[test]
	fn test_resolve_config_empty_json_returns_defaults() {
		let json = crate::json!({});
		let cfg = resolve_config(Some(&json));
		assert_eq!(cfg, ResolvedConfig::default());
	}

	#[test]
	fn test_resolve_config_override_all_booleans() {
		let json = crate::json!({
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
	fn test_resolve_config_format_options_kebab_case() {
		let json = crate::json!({
			"remove-bold": false,
			"compact-blank-lines": false,
			"collapse-spaces": false,
			"remove-horizontal-rules": false,
			"remove-emphasis": false,
			"minify-html": false
		});
		let cfg = resolve_config(Some(&json));
		assert!(!cfg.remove_bold);
		assert!(!cfg.compact_blank_lines);
		assert!(!cfg.collapse_spaces);
		assert!(!cfg.remove_horizontal_rules);
		assert!(!cfg.remove_emphasis);
		assert!(!cfg.minify_html);
	}

	#[test]
	fn test_resolve_config_format_options_snake_case() {
		let json = crate::json!({
			"remove_bold": false,
			"compact_blank_lines": false,
			"collapse_spaces": false,
			"remove_horizontal_rules": false,
			"remove_emphasis": false,
			"minify_html": false
		});
		let cfg = resolve_config(Some(&json));
		assert!(!cfg.remove_bold);
		assert!(!cfg.compact_blank_lines);
		assert!(!cfg.collapse_spaces);
		assert!(!cfg.remove_horizontal_rules);
		assert!(!cfg.remove_emphasis);
		assert!(!cfg.minify_html);
	}

	#[test]
	fn test_resolve_config_format_options_nested() {
		let json = crate::json!({
			"format": {
				"remove_bold": false,
				"minify-html": false
			}
		});
		let cfg = resolve_config(Some(&json));
		assert!(!cfg.remove_bold);
		assert!(!cfg.minify_html);
		assert!(cfg.compact_blank_lines);
	}

	#[test]
	fn test_resolve_config_partial_override() {
		let json = crate::json!({
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
		let json = crate::json!({
			"no-duplicate-heading": false
		});
		let cfg = resolve_config(Some(&json));
		assert!(!cfg.no_duplicate_heading);
		assert!(!cfg.no_duplicate_headings);
	}

	#[test]
	fn test_resolve_config_plural_duplicate_heading() {
		let json = crate::json!({
			"no-duplicate-headings": false
		});
		let cfg = resolve_config(Some(&json));
		assert!(!cfg.no_duplicate_heading);
		assert!(!cfg.no_duplicate_headings);
	}

	#[test]
	fn test_resolve_config_invalid_types_fallback_to_defaults() {
		let json = crate::json!({
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
		let json = crate::json!({
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
		let json = crate::json!({ "other": true });
		assert!(get_bool_config(Some(&json), "blanks-around-headings", true));
		assert!(!get_bool_config(Some(&json), "missing", false));
	}

	#[test]
	fn test_get_bool_config_valid_value() {
		let json = crate::json!({ "flag": false });
		assert!(!get_bool_config(Some(&json), "flag", true));

		let json = crate::json!({ "flag": true });
		assert!(get_bool_config(Some(&json), "flag", false));
	}

	#[test]
	fn test_get_bool_config_invalid_type_fallback() {
		let json = crate::json!({ "flag": "string" });
		assert!(get_bool_config(Some(&json), "flag", true));

		let json = crate::json!({ "flag": 123 });
		assert!(!get_bool_config(Some(&json), "flag", false));
	}

	// ---------- get_u64_config tests ----------

	#[test]
	fn test_get_u64_config_none_config() {
		assert_eq!(get_u64_config(None, "max-line-length", 80), 80);
	}

	#[test]
	fn test_get_u64_config_missing_key() {
		let json = crate::json!({ "other": 100 });
		assert_eq!(get_u64_config(Some(&json), "max-line-length", 80), 80);
	}

	#[test]
	fn test_get_u64_config_valid_value() {
		let json = crate::json!({ "max-line-length": 120 });
		assert_eq!(get_u64_config(Some(&json), "max-line-length", 80), 120);
	}

	#[test]
	fn test_get_u64_config_invalid_type_fallback() {
		let json = crate::json!({ "max-line-length": "not-a-number" });
		assert_eq!(get_u64_config(Some(&json), "max-line-length", 80), 80);

		let json = crate::json!({ "max-line-length": -5 });
		// Negative numbers are not valid u64, so fallback
		assert_eq!(get_u64_config(Some(&json), "max-line-length", 80), 80);
	}

	#[test]
	fn test_get_u64_config_zero_value() {
		let json = crate::json!({ "val": 0 });
		assert_eq!(get_u64_config(Some(&json), "val", 99), 0);
	}

	// ---------- get_string_config tests ----------

	#[test]
	fn test_get_string_config_none_config() {
		assert_eq!(get_string_config(None, "format", "markdown"), "markdown");
	}

	#[test]
	fn test_get_string_config_missing_key() {
		let json = crate::json!({ "other": "value" });
		assert_eq!(
			get_string_config(Some(&json), "format", "markdown"),
			"markdown"
		);
	}

	#[test]
	fn test_get_string_config_valid_value() {
		let json = crate::json!({ "format": "html" });
		assert_eq!(get_string_config(Some(&json), "format", "markdown"), "html");
	}

	#[test]
	fn test_get_string_config_invalid_type_fallback() {
		let json = crate::json!({ "format": 123 });
		assert_eq!(
			get_string_config(Some(&json), "format", "markdown"),
			"markdown"
		);
	}

	#[test]
	fn test_get_string_config_empty_string() {
		let json = crate::json!({ "format": "" });
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
		let temp_dir = TestDir::new("agent_md_test_custom_file");

		let custom = temp_dir.join("my-config.json");
		let mut f = File::create(&custom).unwrap();
		writeln!(f, "{{}}").unwrap();

		let found = find_config_file(Some(custom.to_str().unwrap()));
		assert_eq!(found, Some(custom.to_str().unwrap().to_string()));
	}

	#[test]
	fn test_find_config_file_custom_path_to_nonexistent_file() {
		let result = find_config_file(Some("/nonexistent/path/config.json"));
		assert_eq!(result, None);
	}

	#[test]
	fn test_find_config_file_custom_path_to_empty_dir() {
		let temp_dir = TestDir::new("agent_md_test_empty_dir");

		let found = find_config_file(Some(temp_dir.to_str().unwrap()));
		assert_eq!(found, None);
	}

	#[test]
	fn test_find_config_file_custom_dir_with_only_agent_md() {
		let temp_dir = TestDir::new("agent_md_test_custom_dir_agent");

		let agent_md = temp_dir.join("agent-md.json");
		let mut f = File::create(&agent_md).unwrap();
		writeln!(f, "{{}}").unwrap();

		let found = find_config_file(Some(temp_dir.to_str().unwrap()));
		assert_eq!(found, Some(agent_md.to_str().unwrap().to_string()));
	}

	#[test]
	fn test_find_config_file_custom_dir_priority_order() {
		let temp_dir = TestDir::new("agent_md_test_custom_dir_priority");

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
	}

	#[test]
	fn test_find_config_in_ancestors_walks_up() {
		let temp_dir = TestDir::new("agent_md_test_ancestors_walk");
		let nested = temp_dir.join("sub").join("nested");
		fs::create_dir_all(&nested).unwrap();

		let parent_cfg = temp_dir.join("agent-md.json");
		fs::write(&parent_cfg, "{}").unwrap();

		let found = find_config_in_ancestors(&nested);
		assert_eq!(found, Some(parent_cfg.to_str().unwrap().to_string()));
	}

	#[test]
	fn test_find_config_for_target_closest_ancestor() {
		let temp_dir = TestDir::new("agent_md_test_target_closest");
		let sub_dir = temp_dir.join("sub");
		fs::create_dir_all(&sub_dir).unwrap();

		let root_cfg = temp_dir.join("agent-md.json");
		fs::write(&root_cfg, r#"{"remove_bold": true}"#).unwrap();

		let sub_cfg = sub_dir.join("agent-md.json");
		fs::write(&sub_cfg, r#"{"remove_bold": false}"#).unwrap();

		let target_file = sub_dir.join("test.md");
		fs::write(&target_file, "# Test").unwrap();

		let found = find_config_for_target(Some(target_file.to_str().unwrap()), None);
		assert_eq!(found, Some(sub_cfg.to_str().unwrap().to_string()));
	}

	#[test]
	fn test_find_config_for_target_explicit_override() {
		let temp_dir = TestDir::new("agent_md_test_target_explicit");

		let sub_cfg = temp_dir.join("agent-md.json");
		fs::write(&sub_cfg, "{}").unwrap();

		let explicit_cfg = temp_dir.join("custom-config.json");
		fs::write(&explicit_cfg, "{}").unwrap();

		let target_file = temp_dir.join("test.md");
		fs::write(&target_file, "# Test").unwrap();

		let found = find_config_for_target(
			Some(target_file.to_str().unwrap()),
			Some(explicit_cfg.to_str().unwrap()),
		);
		assert_eq!(found, Some(explicit_cfg.to_str().unwrap().to_string()));
	}

	// ---------- read_config edge cases ----------

	#[test]
	fn test_read_config_invalid_json() {
		let temp_dir = TestDir::new("agent_md_test_invalid_json");

		let cfg_path = temp_dir.join(".agent-md.json");
		let mut f = File::create(&cfg_path).unwrap();
		writeln!(f, "not valid json {{{{").unwrap();

		let result = read_config(temp_dir.to_str());
		assert!(result.is_none()); // Invalid JSON should return None
	}

	#[test]
	fn test_read_config_empty_file() {
		let temp_dir = TestDir::new("agent_md_test_empty_file");

		let cfg_path = temp_dir.join(".agent-md.json");
		let mut f = File::create(&cfg_path).unwrap();
		// Empty file is valid JSON (empty string)
		writeln!(f).unwrap();

		let result = read_config(temp_dir.to_str());
		assert!(result.is_none()); // Empty file is not valid JSON
	}

	#[test]
	fn test_read_config_valid_complex_json() {
		let temp_dir = TestDir::new("agent_md_test_complex_json");

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
	}

	// ---------- get_config_status edge cases ----------

	#[test]
	fn test_get_config_status_invalid_json_file() {
		let temp_dir = TestDir::new("agent_md_test_status_invalid");

		let cfg_path = temp_dir.join(".agent-md.json");
		let mut f = File::create(&cfg_path).unwrap();
		writeln!(f, "not json").unwrap();

		let status = get_config_status(temp_dir.to_str(), true);
		assert!(status.exists);
		assert!(status.path.is_some());
		// Config should be None because JSON is invalid
		assert_eq!(status.config, None);
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
		let json = crate::json::json_output(&cfg, false);
		assert!(json.contains("blanks_around_headings"));
		assert!(json.contains("max_line_length"));
		assert!(json.contains("no_hard_tabs"));
	}

	#[test]
	fn test_config_status_serialization() {
		let status = ConfigStatus {
			exists: true,
			path: Some("test.json".to_string()),
			config: Some(crate::json!({ "key": "val" })),
		};
		let json = crate::json::json_output(&status, false);
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
		let json = crate::json::json_output(&status, false);
		assert!(!json.contains("config"));
	}

	// ---------- resolve_config with all sample config options ----------

	#[test]
	fn test_resolve_config_matches_sample_config() {
		// Match the sample .agent-md.json but with values flipped to false
		let json = crate::json!({
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
		let json = crate::json!({
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
		let temp_dir = TestDir::new("agent_md_test_has_existing");

		let cfg = temp_dir.join("agent-md.json");
		let mut f = File::create(&cfg).unwrap();
		writeln!(f, "{{}}").unwrap();

		assert!(has_config_file(temp_dir.to_str()));
	}

	// ---------- get_config edge cases ----------

	#[test]
	fn test_get_config_returns_none_when_no_file() {
		let temp_dir = TestDir::new("agent_md_test_get_config_none");

		let result = get_config(temp_dir.to_str());
		assert!(result.is_none());
	}

	#[test]
	fn test_get_config_returns_value_when_file_exists() {
		let temp_dir = TestDir::new("agent_md_test_get_config_some");

		let cfg = temp_dir.join(".agent-md.json");
		let mut f = File::create(&cfg).unwrap();
		writeln!(f, r#"{{"key": "value"}}"#).unwrap();

		let result = get_config(temp_dir.to_str());
		assert!(result.is_some());
		assert_eq!(
			result.unwrap().get("key").unwrap(),
			&JsonValue::String("value".to_string())
		);
	}

	// ---------- init_config tests ----------

	#[test]
	fn test_init_config_template_is_valid_json() {
		let parsed: JsonValue =
			JsonValue::parse(DEFAULT_CONFIG_TEMPLATE).expect("Template should be valid JSON");
		assert!(parsed.is_object());
		assert_eq!(parsed.get("default"), Some(&crate::json!(true)));

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
		assert!(!resolved.ignore_markdownlintrc);
		assert!(resolved.remove_bold);
		assert!(resolved.compact_blank_lines);
		assert!(resolved.collapse_spaces);
		assert!(resolved.remove_horizontal_rules);
		assert!(resolved.remove_emphasis);
		assert!(resolved.minify_html);
	}

	#[test]
	fn test_init_config_creates_in_directory() {
		let temp_dir = TestDir::new("agent_md_test_init_dir");

		let res = init_config(temp_dir.to_str(), false);
		assert!(res.is_ok());
		let created_path = res.unwrap();
		assert!(created_path.ends_with(".agent-md.json"));
		assert!(Path::new(&created_path).is_file());

		// Verify content
		let content = fs::read_to_string(&created_path).unwrap();
		assert_eq!(content, DEFAULT_CONFIG_TEMPLATE);
	}

	#[test]
	fn test_init_config_creates_custom_file_path() {
		let temp_dir = TestDir::new("agent_md_test_init_custom");

		let custom_file = temp_dir.join("my-config.json");
		let res = init_config(custom_file.to_str(), false);
		assert!(res.is_ok());
		assert!(custom_file.is_file());
	}

	#[test]
	fn test_init_config_creates_nested_directories() {
		let temp_dir = TestDir::new("agent_md_test_init_nested");

		let nested_file = temp_dir.join("sub").join("nested").join("agent-md.json");
		let res = init_config(nested_file.to_str(), false);
		assert!(res.is_ok());
		assert!(nested_file.is_file());
	}

	#[test]
	fn test_init_config_already_exists_fails_without_force() {
		let temp_dir = TestDir::new("agent_md_test_init_exists");

		let file_path = temp_dir.join(".agent-md.json");
		fs::write(&file_path, "existing content").unwrap();

		let res = init_config(file_path.to_str(), false);
		assert!(res.is_err());
		let err_msg = res.unwrap_err();
		assert!(err_msg.contains("already exists"));

		// Verify existing content was not modified
		assert_eq!(fs::read_to_string(&file_path).unwrap(), "existing content");
	}

	#[test]
	fn test_init_config_already_exists_overwrites_with_force() {
		let temp_dir = TestDir::new("agent_md_test_init_force");

		let file_path = temp_dir.join(".agent-md.json");
		fs::write(&file_path, "existing content").unwrap();

		let res = init_config(file_path.to_str(), true);
		assert!(res.is_ok());

		// Verify content was overwritten with default template
		assert_eq!(
			fs::read_to_string(&file_path).unwrap(),
			DEFAULT_CONFIG_TEMPLATE
		);
	}

	#[test]
	fn test_is_markdownlint_config() {
		assert!(is_markdownlint_config(".markdownlint.json"));
		assert!(is_markdownlint_config(".markdownlint.yaml"));
		assert!(is_markdownlint_config(".markdownlintrc"));
		assert!(is_markdownlint_config("/some/dir/.markdownlintrc.json"));
		assert!(!is_markdownlint_config(".agent-md.json"));
		assert!(!is_markdownlint_config("agent-md.json"));
	}

	#[test]
	fn test_candidate_config_files_ignore_flag() {
		let all = candidate_config_files(false);
		assert!(all.contains(&".agent-md.json"));
		assert!(all.contains(&".markdownlint.json"));
		assert!(all.contains(&".markdownlintrc"));

		let native = candidate_config_files(true);
		assert_eq!(native, AGENT_MD_CONFIG_FILES);
		assert!(!native.contains(&".markdownlint.json"));
	}

	#[test]
	fn test_parse_config_str_yaml() {
		let val = parse_config_str("cfg/.markdownlint.yaml", "line-length: true\n");
		assert!(val.is_some());
		assert_eq!(
			val.unwrap().get("line-length").unwrap(),
			&JsonValue::Bool(true)
		);
	}

	#[test]
	fn test_parse_config_str_jsonc_strips_comments() {
		let content = "{\n// comment\n\"line-length\": true\n/* block */\n}";
		let val = parse_config_str("cfg/.markdownlint.jsonc", content);
		assert!(val.is_some());
		assert_eq!(
			val.unwrap().get("line-length").unwrap(),
			&JsonValue::Bool(true)
		);
	}

	#[test]
	fn test_parse_config_str_extensionless_markdownlintrc() {
		let val = parse_config_str(".markdownlintrc", "{\"line-length\": true}");
		assert!(val.is_some());
	}

	#[test]
	fn test_parse_config_str_json_stays_strict() {
		assert!(parse_config_str(".agent-md.json", "not json").is_none());
		assert!(parse_config_str(".agent-md.json", "").is_none());
	}

	#[test]
	fn test_find_config_ignores_markdownlintrc_when_flag_set() {
		let temp_dir = TestDir::new("agent_md_test_ignore_mdrc");

		let mdrc = temp_dir.join(".markdownlint.json");
		fs::write(&mdrc, "{\"line-length\": true}").unwrap();

		let found = find_config_file_with_options(temp_dir.to_str(), false);
		assert_eq!(found, Some(mdrc.to_str().unwrap().to_string()));

		let ignored = find_config_file_with_options(temp_dir.to_str(), true);
		assert_eq!(ignored, None);
	}

	#[test]
	fn test_find_config_prefers_agent_md_over_markdownlintrc() {
		let temp_dir = TestDir::new("agent_md_test_prefer_native");

		let mdrc = temp_dir.join(".markdownlint.json");
		fs::write(&mdrc, "{\"line-length\": true}").unwrap();
		let native = temp_dir.join(".agent-md.json");
		fs::write(&native, "{\"line-length\": false}").unwrap();

		let found = find_config_file_with_options(temp_dir.to_str(), false);
		assert_eq!(found, Some(native.to_str().unwrap().to_string()));

		let found_ignored = find_config_file_with_options(temp_dir.to_str(), true);
		assert_eq!(found_ignored, Some(native.to_str().unwrap().to_string()));
	}

	#[test]
	fn test_explicit_markdownlintrc_file_still_respected_when_ignored() {
		let temp_dir = TestDir::new("agent_md_test_explicit_mdrc");

		let mdrc = temp_dir.join(".markdownlint.json");
		fs::write(&mdrc, "{\"line-length\": true}").unwrap();

		let found = find_config_file_with_options(Some(mdrc.to_str().unwrap()), true);
		assert_eq!(found, Some(mdrc.to_str().unwrap().to_string()));
	}

	#[test]
	fn test_resolve_config_ignore_markdownlintrc_keys() {
		let kebab = crate::json!({ "ignore-markdownlintrc": true });
		assert!(resolve_config(Some(&kebab)).ignore_markdownlintrc);

		let snake = crate::json!({ "ignore_markdownlintrc": true });
		assert!(resolve_config(Some(&snake)).ignore_markdownlintrc);

		assert!(!resolve_config(None).ignore_markdownlintrc);
		assert!(!ResolvedConfig::default().ignore_markdownlintrc);

		let invalid = crate::json!({ "ignore-markdownlintrc": "yes" });
		assert!(!resolve_config(Some(&invalid)).ignore_markdownlintrc);
	}

	#[test]
	fn test_config_value_ignores_markdownlintrc() {
		assert!(config_value_ignores_markdownlintrc(
			&crate::json!({ "ignore-markdownlintrc": true })
		));
		assert!(config_value_ignores_markdownlintrc(
			&crate::json!({ "ignore_markdownlintrc": true })
		));
		assert!(!config_value_ignores_markdownlintrc(
			&crate::json!({ "ignore-markdownlintrc": false })
		));
		assert!(!config_value_ignores_markdownlintrc(&crate::json!({})));
	}

	#[test]
	fn test_native_config_key_ignores_markdownlintrc() {
		let temp_dir = TestDir::new("agent_md_test_native_key_ignore");

		let native = temp_dir.join(".agent-md.json");
		fs::write(&native, "{\"ignore-markdownlintrc\": true}").unwrap();
		let mdrc = temp_dir.join(".markdownlint.json");
		fs::write(&mdrc, "{\"line-length\": true}").unwrap();

		// No CLI flag, but the native key enables ignoring.
		assert!(get_effective_ignore_markdownlintrc(
			None,
			temp_dir.to_str(),
			false
		));
		let found = find_config_file_with_options(temp_dir.to_str(), false);
		assert_eq!(found, Some(native.to_str().unwrap().to_string()));
		assert!(get_config_with_options(temp_dir.to_str(), false).is_some());
	}

	#[test]
	fn test_markdownlintrc_key_never_ignores() {
		let temp_dir = TestDir::new("agent_md_test_mdrc_key_noop");

		let mdrc = temp_dir.join(".markdownlint.json");
		fs::write(&mdrc, "{\"ignore-markdownlintrc\": true}").unwrap();

		assert!(!get_effective_ignore_markdownlintrc(
			None,
			temp_dir.to_str(),
			false
		));
		let found = find_config_file_with_options(temp_dir.to_str(), false);
		assert_eq!(found, Some(mdrc.to_str().unwrap().to_string()));
	}

	#[test]
	fn test_target_ancestor_native_key_ignores_subdir_markdownlintrc() {
		let temp_dir = TestDir::new("agent_md_test_ancestor_key_ignore");
		let sub_dir = temp_dir.join("sub");
		fs::create_dir_all(&sub_dir).unwrap();

		let native = temp_dir.join("agent-md.json");
		fs::write(&native, "{\"ignore_markdownlintrc\": true}").unwrap();
		let mdrc = sub_dir.join(".markdownlint.json");
		fs::write(&mdrc, "{\"line-length\": true}").unwrap();
		let target = sub_dir.join("doc.md");
		fs::write(&target, "# Doc").unwrap();

		assert!(get_effective_ignore_markdownlintrc(
			Some(target.to_str().unwrap()),
			None,
			false
		));
		let found =
			find_config_for_target_with_options(Some(target.to_str().unwrap()), None, false);
		assert_eq!(found, Some(native.to_str().unwrap().to_string()));
	}

	#[test]
	fn test_get_config_with_options_ignore_flag() {
		let temp_dir = TestDir::new("agent_md_test_get_ignore");

		let mdrc = temp_dir.join(".markdownlintrc");
		fs::write(&mdrc, "{\"line-length\": true}").unwrap();

		let cfg = get_config_with_options(temp_dir.to_str(), false);
		assert!(cfg.is_some());

		let ignored = get_config_with_options(temp_dir.to_str(), true);
		assert!(ignored.is_none());
	}
}
