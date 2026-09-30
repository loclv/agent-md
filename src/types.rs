use crate::json::{JsonObject, JsonValue, ToJson};

pub use crate::json::json_output;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonlEntry {
	pub entry_type: String,
	pub content: String,
	pub level: Option<u32>,
	pub language: Option<String>,
}

impl ToJson for JsonlEntry {
	fn to_json(&self) -> JsonValue {
		let mut obj = JsonObject::new();
		obj.insert("type", self.entry_type.as_str());
		obj.insert("content", self.content.as_str());
		if let Some(level) = self.level {
			obj.insert("level", level);
		}
		if let Some(ref lang) = self.language {
			obj.insert("language", lang.as_str());
		}
		JsonValue::Object(obj)
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
	pub path: String,
	pub content: String,
	pub word_count: usize,
	pub line_count: usize,
	pub headings: Vec<Heading>,
}

impl ToJson for Document {
	fn to_json(&self) -> JsonValue {
		let mut obj = JsonObject::new();
		obj.insert("path", self.path.as_str());
		obj.insert("content", self.content.as_str());
		obj.insert("word_count", self.word_count);
		obj.insert("line_count", self.line_count);
		obj.insert("headings", self.headings.to_json());
		JsonValue::Object(obj)
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heading {
	pub level: u32,
	pub text: String,
	pub line: usize,
}

impl ToJson for Heading {
	fn to_json(&self) -> JsonValue {
		let mut obj = JsonObject::new();
		obj.insert("level", self.level);
		obj.insert("text", self.text.as_str());
		obj.insert("line", self.line);
		JsonValue::Object(obj)
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditResult {
	pub success: bool,
	pub message: String,
	pub document: Option<Document>,
}

impl ToJson for EditResult {
	fn to_json(&self) -> JsonValue {
		let mut obj = JsonObject::new();
		obj.insert("success", self.success);
		obj.insert("message", self.message.as_str());
		match &self.document {
			Some(doc) => obj.insert("document", doc.to_json()),
			None => obj.insert("document", JsonValue::Null),
		}
		JsonValue::Object(obj)
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchResult {
	pub query: String,
	pub matches: Vec<Match>,
	pub total: usize,
}

impl ToJson for SearchResult {
	fn to_json(&self) -> JsonValue {
		let mut obj = JsonObject::new();
		obj.insert("query", self.query.as_str());
		obj.insert("matches", self.matches.to_json());
		obj.insert("total", self.total);
		JsonValue::Object(obj)
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match {
	pub line: usize,
	pub content: String,
}

impl ToJson for Match {
	fn to_json(&self) -> JsonValue {
		let mut obj = JsonObject::new();
		obj.insert("line", self.line);
		obj.insert("content", self.content.as_str());
		JsonValue::Object(obj)
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LintResult {
	pub valid: bool,
	pub errors: Vec<LintError>,
	pub warnings: Vec<LintWarning>,
}

impl ToJson for LintResult {
	fn to_json(&self) -> JsonValue {
		let mut obj = JsonObject::new();
		obj.insert("valid", self.valid);
		obj.insert("errors", self.errors.to_json());
		obj.insert("warnings", self.warnings.to_json());
		JsonValue::Object(obj)
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LintError {
	pub line: usize,
	pub column: usize,
	pub message: String,
	pub rule: String,
}

impl ToJson for LintError {
	fn to_json(&self) -> JsonValue {
		let mut obj = JsonObject::new();
		obj.insert("line", self.line);
		obj.insert("column", self.column);
		obj.insert("message", self.message.as_str());
		obj.insert("rule", self.rule.as_str());
		JsonValue::Object(obj)
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LintWarning {
	pub line: usize,
	pub column: usize,
	pub message: String,
	pub rule: String,
}

impl ToJson for LintWarning {
	fn to_json(&self) -> JsonValue {
		let mut obj = JsonObject::new();
		obj.insert("line", self.line);
		obj.insert("column", self.column);
		obj.insert("message", self.message.as_str());
		obj.insert("rule", self.rule.as_str());
		JsonValue::Object(obj)
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
