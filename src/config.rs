use serde::Serialize;
use std::fs;
use std::path::Path;

/// Candidate configuration file names in resolution priority order.
pub const CONFIG_FILES: &[&str] = &[".agent-md.json", "agent-md.json", ".markdownlint.json"];

/// Configuration status representation for JSON output.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ConfigStatus {
	pub exists: bool,
	pub path: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub config: Option<serde_json::Value>,
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
			for &name in CONFIG_FILES {
				let candidate = path.join(name);
				if candidate.is_file() {
					return candidate.to_str().map(|s| s.to_string());
				}
			}
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

#[cfg(test)]
mod tests {
	use super::*;
	use std::fs::File;
	use std::io::Write;

	#[test]
	fn test_find_config_file_priority() {
		let temp_dir = std::env::temp_dir().join("agent_md_test_config_priority");
		let _ = fs::remove_dir_all(&temp_dir);
		fs::create_dir_all(&temp_dir).unwrap();

		// Case 1: only .markdownlint.json exists
		let md_lint = temp_dir.join(".markdownlint.json");
		let mut f = File::create(&md_lint).unwrap();
		writeln!(f, r#"{{"default": true}}"#).unwrap();

		let found = find_config_file(temp_dir.to_str());
		assert_eq!(found, Some(md_lint.to_str().unwrap().to_string()));

		// Case 2: agent-md.json is added, should take precedence over .markdownlint.json
		let agent_md = temp_dir.join("agent-md.json");
		let mut f = File::create(&agent_md).unwrap();
		writeln!(f, r#"{{"blanks-around-headings": false}}"#).unwrap();

		let found = find_config_file(temp_dir.to_str());
		assert_eq!(found, Some(agent_md.to_str().unwrap().to_string()));

		// Case 3: .agent-md.json is added, should take highest precedence
		let dot_agent_md = temp_dir.join(".agent-md.json");
		let mut f = File::create(&dot_agent_md).unwrap();
		writeln!(f, r#"{{"blanks-around-headings": true}}"#).unwrap();

		let found = find_config_file(temp_dir.to_str());
		assert_eq!(found, Some(dot_agent_md.to_str().unwrap().to_string()));

		// Read config value check
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

		// Non-existent
		let status = get_config_status(temp_dir.to_str(), true);
		assert!(!status.exists);
		assert_eq!(status.path, None);
		assert_eq!(status.config, None);

		// Existent with content
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
}
