use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JsonlEntry {
	#[serde(rename = "type")]
	pub entry_type: String,
	pub content: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub level: Option<u32>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub language: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Document {
	pub path: String,
	pub content: String,
	pub word_count: usize,
	pub line_count: usize,
	pub headings: Vec<Heading>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Heading {
	pub level: u32,
	pub text: String,
	pub line: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EditResult {
	pub success: bool,
	pub message: String,
	pub document: Option<Document>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
	pub query: String,
	pub matches: Vec<Match>,
	pub total: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Match {
	pub line: usize,
	pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LintResult {
	pub valid: bool,
	pub errors: Vec<LintError>,
	pub warnings: Vec<LintWarning>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct LintError {
	pub line: usize,
	pub column: usize,
	pub message: String,
	pub rule: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LintWarning {
	pub line: usize,
	pub column: usize,
	pub message: String,
	pub rule: String,
}

pub fn json_output<T: ?Sized + Serialize>(value: &T, human: bool) -> String {
	if human {
		serde_json::to_string_pretty(value).unwrap()
	} else {
		serde_json::to_string(value).unwrap()
	}
}

pub fn unescape_content(s: &str) -> String {
	let mut result = String::with_capacity(s.len());
	let mut chars = s.chars().peekable();
	while let Some(ch) = chars.next() {
		if ch == '\\' {
			match chars.peek() {
				Some('n') => {
					result.push('\n');
					chars.next();
				}
				Some('t') => {
					result.push('\t');
					chars.next();
				}
				Some('\\') => {
					result.push('\\');
					chars.next();
				}
				_ => result.push(ch),
			}
		} else {
			result.push(ch);
		}
	}
	result
}
