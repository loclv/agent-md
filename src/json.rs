//! Lightweight, zero-dependency JSON data representation, parser, and serializer.

use std::fmt;

/// An owned JSON key-value object that preserves insertion order.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct JsonObject {
	/// Key-value pairs in insertion order.
	pub entries: Vec<(String, JsonValue)>,
}

impl JsonObject {
	/// Creates an empty JSON object.
	pub fn new() -> Self {
		Self {
			entries: Vec::new(),
		}
	}

	/// Creates an empty JSON object with pre-allocated capacity.
	pub fn with_capacity(capacity: usize) -> Self {
		Self {
			entries: Vec::with_capacity(capacity),
		}
	}

	/// Inserts a key-value pair. If the key already exists, replaces its value.
	pub fn insert(&mut self, key: impl Into<String>, value: impl Into<JsonValue>) {
		let key = key.into();
		let value = value.into();
		if let Some((_, v)) = self.entries.iter_mut().find(|(k, _)| k == &key) {
			*v = value;
		} else {
			self.entries.push((key, value));
		}
	}

	/// Gets a reference to the value associated with the specified key.
	pub fn get(&self, key: &str) -> Option<&JsonValue> {
		self.entries.iter().find(|(k, _)| k == key).map(|(_, v)| v)
	}

	/// Gets a mutable reference to the value associated with the specified key.
	pub fn get_mut(&mut self, key: &str) -> Option<&mut JsonValue> {
		self.entries
			.iter_mut()
			.find(|(k, _)| k == key)
			.map(|(_, v)| v)
	}

	/// Removes a key from the object, returning its value if present.
	pub fn remove(&mut self, key: &str) -> Option<JsonValue> {
		if let Some(pos) = self.entries.iter().position(|(k, _)| k == key) {
			Some(self.entries.remove(pos).1)
		} else {
			None
		}
	}

	/// Returns the number of entries in the object.
	pub fn len(&self) -> usize {
		self.entries.len()
	}

	/// Returns true if the object contains no entries.
	pub fn is_empty(&self) -> bool {
		self.entries.is_empty()
	}

	/// Returns an iterator over the key-value pairs.
	pub fn iter(&self) -> std::slice::Iter<'_, (String, JsonValue)> {
		self.entries.iter()
	}

	/// Returns an iterator over the keys.
	pub fn keys(&self) -> impl Iterator<Item = &str> {
		self.entries.iter().map(|(k, _)| k.as_str())
	}

	/// Returns an iterator over the values.
	pub fn values(&self) -> impl Iterator<Item = &JsonValue> {
		self.entries.iter().map(|(_, v)| v)
	}
}

/// Dynamic JSON value enum representing standard JSON data types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonValue {
	/// JSON null.
	Null,
	/// JSON boolean (`true` or `false`).
	Bool(bool),
	/// JSON number represented as a raw numeric string.
	Number(String),
	/// JSON string.
	String(String),
	/// JSON array.
	Array(Vec<JsonValue>),
	/// JSON object mapping strings to JSON values.
	Object(JsonObject),
}

impl JsonValue {
	/// Returns true if this value is null.
	pub fn is_null(&self) -> bool {
		matches!(self, JsonValue::Null)
	}

	/// Returns true if this value is a boolean.
	pub fn is_boolean(&self) -> bool {
		matches!(self, JsonValue::Bool(_))
	}

	/// Returns true if this value is a number.
	pub fn is_number(&self) -> bool {
		matches!(self, JsonValue::Number(_))
	}

	/// Returns true if this value is a string.
	pub fn is_string(&self) -> bool {
		matches!(self, JsonValue::String(_))
	}

	/// Returns true if this value is an array.
	pub fn is_array(&self) -> bool {
		matches!(self, JsonValue::Array(_))
	}

	/// Returns true if this value is an object.
	pub fn is_object(&self) -> bool {
		matches!(self, JsonValue::Object(_))
	}

	/// Extracts a boolean value if this is a `JsonValue::Bool`.
	pub fn as_bool(&self) -> Option<bool> {
		match self {
			JsonValue::Bool(b) => Some(*b),
			_ => None,
		}
	}

	/// Extracts a `u64` integer if this is a `JsonValue::Number`.
	pub fn as_u64(&self) -> Option<u64> {
		match self {
			JsonValue::Number(s) => s.parse::<u64>().ok(),
			_ => None,
		}
	}

	/// Extracts an `i64` integer if this is a `JsonValue::Number`.
	pub fn as_i64(&self) -> Option<i64> {
		match self {
			JsonValue::Number(s) => s.parse::<i64>().ok(),
			_ => None,
		}
	}

	/// Extracts an `f64` float if this is a `JsonValue::Number`.
	pub fn as_f64(&self) -> Option<f64> {
		match self {
			JsonValue::Number(s) => s.parse::<f64>().ok(),
			_ => None,
		}
	}

	/// Extracts a string slice if this is a `JsonValue::String`.
	pub fn as_str(&self) -> Option<&str> {
		match self {
			JsonValue::String(s) => Some(s.as_str()),
			_ => None,
		}
	}

	/// Extracts an array slice if this is a `JsonValue::Array`.
	pub fn as_array(&self) -> Option<&[JsonValue]> {
		match self {
			JsonValue::Array(a) => Some(a.as_slice()),
			_ => None,
		}
	}

	/// Extracts a mutable reference to an array if this is a `JsonValue::Array`.
	pub fn as_array_mut(&mut self) -> Option<&mut Vec<JsonValue>> {
		match self {
			JsonValue::Array(a) => Some(a),
			_ => None,
		}
	}

	/// Extracts an object reference if this is a `JsonValue::Object`.
	pub fn as_object(&self) -> Option<&JsonObject> {
		match self {
			JsonValue::Object(o) => Some(o),
			_ => None,
		}
	}

	/// Extracts a mutable object reference if this is a `JsonValue::Object`.
	pub fn as_object_mut(&mut self) -> Option<&mut JsonObject> {
		match self {
			JsonValue::Object(o) => Some(o),
			_ => None,
		}
	}

	/// Looks up a key in an object. Returns `None` if this value is not an object.
	pub fn get(&self, key: &str) -> Option<&JsonValue> {
		match self {
			JsonValue::Object(o) => o.get(key),
			_ => None,
		}
	}

	/// Looks up a mutable key in an object. Returns `None` if this value is not an object.
	pub fn get_mut(&mut self, key: &str) -> Option<&mut JsonValue> {
		match self {
			JsonValue::Object(o) => o.get_mut(key),
			_ => None,
		}
	}
	/// Serializes this value into an indented, human-readable JSON string (2 spaces).
	pub fn to_string_pretty(&self) -> String {
		let mut out = String::new();
		serialize_pretty(self, &mut out, 0);
		out
	}

	/// Parses a JSON text string into a `JsonValue`.
	pub fn parse(input: &str) -> Result<Self, String> {
		let mut parser = JsonParser::new(input);
		parser.parse_full()
	}
}

impl fmt::Display for JsonValue {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut out = String::new();
		serialize_compact(self, &mut out);
		f.write_str(&out)
	}
}

// From conversions for primitive types
impl From<bool> for JsonValue {
	fn from(b: bool) -> Self {
		JsonValue::Bool(b)
	}
}

impl From<String> for JsonValue {
	fn from(s: String) -> Self {
		JsonValue::String(s)
	}
}

impl From<&str> for JsonValue {
	fn from(s: &str) -> Self {
		JsonValue::String(s.to_string())
	}
}

impl From<JsonObject> for JsonValue {
	fn from(o: JsonObject) -> Self {
		JsonValue::Object(o)
	}
}

impl From<Vec<JsonValue>> for JsonValue {
	fn from(a: Vec<JsonValue>) -> Self {
		JsonValue::Array(a)
	}
}

macro_rules! impl_from_int {
	($($t:ty),*) => {
		$(
			impl From<$t> for JsonValue {
				fn from(v: $t) -> Self {
					JsonValue::Number(v.to_string())
				}
			}
		)*
	};
}

impl_from_int!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);

impl From<f32> for JsonValue {
	fn from(v: f32) -> Self {
		JsonValue::Number(v.to_string())
	}
}

impl From<f64> for JsonValue {
	fn from(v: f64) -> Self {
		JsonValue::Number(v.to_string())
	}
}

/// Trait for types that can be serialized into a `JsonValue`.
pub trait ToJson {
	/// Converts the value to a `JsonValue`.
	fn to_json(&self) -> JsonValue;
}

impl ToJson for JsonValue {
	fn to_json(&self) -> JsonValue {
		self.clone()
	}
}

impl ToJson for bool {
	fn to_json(&self) -> JsonValue {
		JsonValue::Bool(*self)
	}
}

impl ToJson for String {
	fn to_json(&self) -> JsonValue {
		JsonValue::String(self.clone())
	}
}

impl ToJson for &str {
	fn to_json(&self) -> JsonValue {
		JsonValue::String((*self).to_string())
	}
}

macro_rules! impl_to_json_num {
	($($t:ty),*) => {
		$(
			impl ToJson for $t {
				fn to_json(&self) -> JsonValue {
					JsonValue::Number(self.to_string())
				}
			}
		)*
	};
}

impl_to_json_num!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize, f32, f64);

impl<T: ToJson> ToJson for Option<T> {
	fn to_json(&self) -> JsonValue {
		match self {
			Some(v) => v.to_json(),
			None => JsonValue::Null,
		}
	}
}

impl<T: ToJson> ToJson for Vec<T> {
	fn to_json(&self) -> JsonValue {
		JsonValue::Array(self.iter().map(|item| item.to_json()).collect())
	}
}

impl<T: ToJson> ToJson for [T] {
	fn to_json(&self) -> JsonValue {
		JsonValue::Array(self.iter().map(|item| item.to_json()).collect())
	}
}

impl<T: ToJson> ToJson for &T {
	fn to_json(&self) -> JsonValue {
		(*self).to_json()
	}
}

/// Serialize a value to a JSON string, optionally pretty-printed with 2 spaces.
pub fn json_output<T: ToJson + ?Sized>(value: &T, human: bool) -> String {
	let val = value.to_json();
	if human {
		val.to_string_pretty()
	} else {
		val.to_string()
	}
}

// ----------------------------------------------------------------------------
// JSON Parser
// ----------------------------------------------------------------------------

struct JsonParser<'a> {
	bytes: &'a [u8],
	pos: usize,
}

impl<'a> JsonParser<'a> {
	fn new(input: &'a str) -> Self {
		Self {
			bytes: input.as_bytes(),
			pos: 0,
		}
	}

	fn parse_full(&mut self) -> Result<JsonValue, String> {
		self.skip_whitespace();
		if self.pos >= self.bytes.len() {
			return Err("Unexpected empty input".to_string());
		}
		let value = self.parse_value()?;
		self.skip_whitespace();
		if self.pos < self.bytes.len() {
			return Err(format!(
				"Unexpected trailing character '{}' at byte position {}",
				self.bytes[self.pos] as char, self.pos
			));
		}
		Ok(value)
	}

	fn skip_whitespace(&mut self) {
		while self.pos < self.bytes.len() {
			match self.bytes[self.pos] {
				b' ' | b'\t' | b'\n' | b'\r' => self.pos += 1,
				_ => break,
			}
		}
	}

	fn peek_byte(&self) -> Option<u8> {
		self.bytes.get(self.pos).copied()
	}

	fn parse_value(&mut self) -> Result<JsonValue, String> {
		self.skip_whitespace();
		let b = self
			.peek_byte()
			.ok_or_else(|| "Unexpected end of input while parsing value".to_string())?;

		match b {
			b'n' => self.parse_null(),
			b't' => self.parse_true(),
			b'f' => self.parse_false(),
			b'"' => self.parse_string().map(JsonValue::String),
			b'[' => self.parse_array(),
			b'{' => self.parse_object(),
			b'-' | b'0'..=b'9' => self.parse_number(),
			_ => Err(format!(
				"Unexpected character '{}' at byte position {}",
				b as char, self.pos
			)),
		}
	}

	fn parse_null(&mut self) -> Result<JsonValue, String> {
		if self.pos + 4 <= self.bytes.len() && &self.bytes[self.pos..self.pos + 4] == b"null" {
			self.pos += 4;
			Ok(JsonValue::Null)
		} else {
			Err(format!("Expected 'null' at byte position {}", self.pos))
		}
	}

	fn parse_true(&mut self) -> Result<JsonValue, String> {
		if self.pos + 4 <= self.bytes.len() && &self.bytes[self.pos..self.pos + 4] == b"true" {
			self.pos += 4;
			Ok(JsonValue::Bool(true))
		} else {
			Err(format!("Expected 'true' at byte position {}", self.pos))
		}
	}

	fn parse_false(&mut self) -> Result<JsonValue, String> {
		if self.pos + 5 <= self.bytes.len() && &self.bytes[self.pos..self.pos + 5] == b"false" {
			self.pos += 5;
			Ok(JsonValue::Bool(false))
		} else {
			Err(format!("Expected 'false' at byte position {}", self.pos))
		}
	}

	fn parse_string(&mut self) -> Result<String, String> {
		if self.peek_byte() != Some(b'"') {
			return Err(format!("Expected '\"' at byte position {}", self.pos));
		}
		self.pos += 1;

		let mut s = String::new();
		while self.pos < self.bytes.len() {
			let b = self.bytes[self.pos];
			match b {
				b'"' => {
					self.pos += 1;
					return Ok(s);
				}
				b'\\' => {
					self.pos += 1;
					let esc = self
						.peek_byte()
						.ok_or_else(|| "Unexpected end of input in escape sequence".to_string())?;
					self.pos += 1;
					match esc {
						b'"' => s.push('"'),
						b'\\' => s.push('\\'),
						b'/' => s.push('/'),
						b'b' => s.push('\x08'),
						b'f' => s.push('\x0C'),
						b'n' => s.push('\n'),
						b'r' => s.push('\r'),
						b't' => s.push('\t'),
						b'u' => {
							let code = self.parse_hex4()?;
							if (0xD800..=0xDBFF).contains(&code) {
								// Surrogate pair
								if self.pos + 2 <= self.bytes.len()
									&& &self.bytes[self.pos..self.pos + 2] == b"\\u"
								{
									self.pos += 2;
									let low = self.parse_hex4()?;
									if (0xDC00..=0xDFFF).contains(&low) {
										let high_bits = (code - 0xD800) as u32;
										let low_bits = (low - 0xDC00) as u32;
										let scalar = 0x10000 + ((high_bits << 10) | low_bits);
										if let Some(ch) = char::from_u32(scalar) {
											s.push(ch);
											continue;
										}
									}
								}
								return Err(format!(
									"Invalid UTF-16 surrogate pair at byte position {}",
									self.pos
								));
							}
							let ch = char::from_u32(code as u32).ok_or_else(|| {
								format!("Invalid Unicode escape code at byte position {}", self.pos)
							})?;
							s.push(ch);
						}
						_ => {
							return Err(format!(
								"Invalid escape '\\{}' at byte position {}",
								esc as char, self.pos
							))
						}
					}
				}
				0..=0x1F => {
					return Err(format!(
						"Unescaped control character in string at byte position {}",
						self.pos
					));
				}
				_ => {
					// Read valid UTF-8 character
					let rest = &self.bytes[self.pos..];
					let ch = std::str::from_utf8(rest)
						.map_err(|e| e.to_string())?
						.chars()
						.next()
						.ok_or_else(|| "Unexpected end of string".to_string())?;
					s.push(ch);
					self.pos += ch.len_utf8();
				}
			}
		}

		Err("Unterminated string literal".to_string())
	}

	fn parse_hex4(&mut self) -> Result<u16, String> {
		if self.pos + 4 > self.bytes.len() {
			return Err("Incomplete \\u hex escape".to_string());
		}
		let hex_str =
			std::str::from_utf8(&self.bytes[self.pos..self.pos + 4]).map_err(|e| e.to_string())?;
		self.pos += 4;
		u16::from_str_radix(hex_str, 16)
			.map_err(|_| format!("Invalid hex in \\u escape: '{}'", hex_str))
	}

	fn parse_number(&mut self) -> Result<JsonValue, String> {
		let start = self.pos;
		if self.peek_byte() == Some(b'-') {
			self.pos += 1;
		}

		let digit_start = self.pos;
		while let Some(b'0'..=b'9') = self.peek_byte() {
			self.pos += 1;
		}
		if self.pos == digit_start {
			return Err(format!(
				"Expected digits in number at byte position {}",
				self.pos
			));
		}

		// Fractional part
		if self.peek_byte() == Some(b'.') {
			self.pos += 1;
			let frac_start = self.pos;
			while let Some(b'0'..=b'9') = self.peek_byte() {
				self.pos += 1;
			}
			if self.pos == frac_start {
				return Err(format!(
					"Expected digits after decimal point at byte position {}",
					self.pos
				));
			}
		}

		// Exponent part
		if let Some(b'e' | b'E') = self.peek_byte() {
			self.pos += 1;
			if let Some(b'+' | b'-') = self.peek_byte() {
				self.pos += 1;
			}
			let exp_start = self.pos;
			while let Some(b'0'..=b'9') = self.peek_byte() {
				self.pos += 1;
			}
			if self.pos == exp_start {
				return Err(format!(
					"Expected digits in exponent at byte position {}",
					self.pos
				));
			}
		}

		let num_str = std::str::from_utf8(&self.bytes[start..self.pos])
			.map_err(|e| e.to_string())?
			.to_string();
		Ok(JsonValue::Number(num_str))
	}

	fn parse_array(&mut self) -> Result<JsonValue, String> {
		self.pos += 1; // skip '['
		self.skip_whitespace();

		let mut items = Vec::new();
		if self.peek_byte() == Some(b']') {
			self.pos += 1;
			return Ok(JsonValue::Array(items));
		}

		loop {
			let val = self.parse_value()?;
			items.push(val);
			self.skip_whitespace();

			match self.peek_byte() {
				Some(b']') => {
					self.pos += 1;
					break;
				}
				Some(b',') => {
					self.pos += 1;
					self.skip_whitespace();
					if self.peek_byte() == Some(b']') {
						return Err(format!(
							"Trailing comma in array at byte position {}",
							self.pos
						));
					}
				}
				Some(b) => {
					return Err(format!(
						"Expected ',' or ']' in array, found '{}' at byte position {}",
						b as char, self.pos
					))
				}
				None => return Err("Unterminated array literal".to_string()),
			}
		}

		Ok(JsonValue::Array(items))
	}

	fn parse_object(&mut self) -> Result<JsonValue, String> {
		self.pos += 1; // skip '{'
		self.skip_whitespace();

		let mut obj = JsonObject::new();
		if self.peek_byte() == Some(b'}') {
			self.pos += 1;
			return Ok(JsonValue::Object(obj));
		}

		loop {
			self.skip_whitespace();
			let key = self.parse_string()?;
			self.skip_whitespace();

			if self.peek_byte() != Some(b':') {
				return Err(format!(
					"Expected ':' after key at byte position {}",
					self.pos
				));
			}
			self.pos += 1; // skip ':'

			let value = self.parse_value()?;
			obj.insert(key, value);
			self.skip_whitespace();

			match self.peek_byte() {
				Some(b'}') => {
					self.pos += 1;
					break;
				}
				Some(b',') => {
					self.pos += 1;
					self.skip_whitespace();
					if self.peek_byte() == Some(b'}') {
						return Err(format!(
							"Trailing comma in object at byte position {}",
							self.pos
						));
					}
				}
				Some(b) => {
					return Err(format!(
						"Expected ',' or '}}' in object, found '{}' at byte position {}",
						b as char, self.pos
					))
				}
				None => return Err("Unterminated object literal".to_string()),
			}
		}

		Ok(JsonValue::Object(obj))
	}
}

// ----------------------------------------------------------------------------
// JSON Serialization
// ----------------------------------------------------------------------------

fn escape_json_str(s: &str, out: &mut String) {
	out.push('"');
	for c in s.chars() {
		match c {
			'"' => out.push_str("\\\""),
			'\\' => out.push_str("\\\\"),
			'\n' => out.push_str("\\n"),
			'\r' => out.push_str("\\r"),
			'\t' => out.push_str("\\t"),
			'\x08' => out.push_str("\\b"),
			'\x0C' => out.push_str("\\f"),
			c if (c as u32) < 0x20 => {
				use std::fmt::Write;
				let _ = write!(out, "\\u{:04x}", c as u32);
			}
			c => out.push(c),
		}
	}
	out.push('"');
}

fn serialize_compact(val: &JsonValue, out: &mut String) {
	match val {
		JsonValue::Null => out.push_str("null"),
		JsonValue::Bool(true) => out.push_str("true"),
		JsonValue::Bool(false) => out.push_str("false"),
		JsonValue::Number(s) => out.push_str(s),
		JsonValue::String(s) => escape_json_str(s, out),
		JsonValue::Array(items) => {
			out.push('[');
			for (i, item) in items.iter().enumerate() {
				if i > 0 {
					out.push(',');
				}
				serialize_compact(item, out);
			}
			out.push(']');
		}
		JsonValue::Object(obj) => {
			out.push('{');
			for (i, (k, v)) in obj.iter().enumerate() {
				if i > 0 {
					out.push(',');
				}
				escape_json_str(k, out);
				out.push(':');
				serialize_compact(v, out);
			}
			out.push('}');
		}
	}
}

fn serialize_pretty(val: &JsonValue, out: &mut String, indent: usize) {
	match val {
		JsonValue::Null => out.push_str("null"),
		JsonValue::Bool(true) => out.push_str("true"),
		JsonValue::Bool(false) => out.push_str("false"),
		JsonValue::Number(s) => out.push_str(s),
		JsonValue::String(s) => escape_json_str(s, out),
		JsonValue::Array(items) => {
			if items.is_empty() {
				out.push_str("[]");
				return;
			}
			out.push_str("[\n");
			for (i, item) in items.iter().enumerate() {
				if i > 0 {
					out.push_str(",\n");
				}
				write_indent(out, indent + 1);
				serialize_pretty(item, out, indent + 1);
			}
			out.push('\n');
			write_indent(out, indent);
			out.push(']');
		}
		JsonValue::Object(obj) => {
			if obj.is_empty() {
				out.push_str("{}");
				return;
			}
			out.push_str("{\n");
			for (i, (k, v)) in obj.iter().enumerate() {
				if i > 0 {
					out.push_str(",\n");
				}
				write_indent(out, indent + 1);
				escape_json_str(k, out);
				out.push_str(": ");
				serialize_pretty(v, out, indent + 1);
			}
			out.push('\n');
			write_indent(out, indent);
			out.push('}');
		}
	}
}

fn write_indent(out: &mut String, count: usize) {
	for _ in 0..count {
		out.push_str("  ");
	}
}

// PartialEq implementations for primitive types and comparisons
impl PartialEq<bool> for JsonValue {
	fn eq(&self, other: &bool) -> bool {
		self.as_bool() == Some(*other)
	}
}

impl PartialEq<JsonValue> for bool {
	fn eq(&self, other: &JsonValue) -> bool {
		Some(*self) == other.as_bool()
	}
}

impl PartialEq<bool> for &JsonValue {
	fn eq(&self, other: &bool) -> bool {
		(*self).as_bool() == Some(*other)
	}
}

impl PartialEq<&JsonValue> for bool {
	fn eq(&self, other: &&JsonValue) -> bool {
		Some(*self) == (*other).as_bool()
	}
}

macro_rules! impl_partial_eq_num {
	($($t:ty),*) => {
		$(
			impl PartialEq<$t> for JsonValue {
				fn eq(&self, other: &$t) -> bool {
					match self {
						JsonValue::Number(s) => s.parse::<$t>().map(|v| v == *other).unwrap_or(false),
						_ => false,
					}
				}
			}

			impl PartialEq<JsonValue> for $t {
				fn eq(&self, other: &JsonValue) -> bool {
					other == self
				}
			}

			impl PartialEq<$t> for &JsonValue {
				fn eq(&self, other: &$t) -> bool {
					*self == other
				}
			}

			impl PartialEq<&JsonValue> for $t {
				fn eq(&self, other: &&JsonValue) -> bool {
					*other == self
				}
			}
		)*
	};
}

impl_partial_eq_num!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);

impl PartialEq<str> for JsonValue {
	fn eq(&self, other: &str) -> bool {
		self.as_str() == Some(other)
	}
}

impl PartialEq<&str> for JsonValue {
	fn eq(&self, other: &&str) -> bool {
		self.as_str() == Some(*other)
	}
}

impl PartialEq<String> for JsonValue {
	fn eq(&self, other: &String) -> bool {
		self.as_str() == Some(other.as_str())
	}
}

/// Macro for creating JSON values with syntax similar to `serde_json::json!`.
#[macro_export]
macro_rules! json {
	(true) => {
		$crate::json::JsonValue::Bool(true)
	};
	(false) => {
		$crate::json::JsonValue::Bool(false)
	};
	(null) => {
		$crate::json::JsonValue::Null
	};
	({ $($tt:tt)* }) => {
		$crate::json::JsonValue::parse(stringify!({ $($tt)* })).expect("valid json syntax")
	};
	([ $($tt:tt)* ]) => {
		$crate::json::JsonValue::parse(stringify!([ $($tt)* ])).expect("valid json syntax")
	};
	($lit:literal) => {
		$crate::json::JsonValue::from($lit)
	};
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_parse_primitives() {
		assert_eq!(JsonValue::parse("null").unwrap(), JsonValue::Null);
		assert_eq!(JsonValue::parse("true").unwrap(), JsonValue::Bool(true));
		assert_eq!(JsonValue::parse("false").unwrap(), JsonValue::Bool(false));
		assert_eq!(
			JsonValue::parse("123").unwrap(),
			JsonValue::Number("123".to_string())
		);
		assert_eq!(
			JsonValue::parse("-45.67").unwrap(),
			JsonValue::Number("-45.67".to_string())
		);
		assert_eq!(
			JsonValue::parse("\"hello\\nworld\"").unwrap(),
			JsonValue::String("hello\nworld".to_string())
		);
	}

	#[test]
	fn test_parse_array_and_object() {
		let json_str = r#"{"name": "agent-md", "version": 3, "tags": ["rust", "markdown"]}"#;
		let val = JsonValue::parse(json_str).unwrap();
		assert!(val.is_object());
		assert_eq!(val.get("name").unwrap().as_str(), Some("agent-md"));
		assert_eq!(val.get("version").unwrap().as_u64(), Some(3));
		let tags = val.get("tags").unwrap().as_array().unwrap();
		assert_eq!(tags.len(), 2);
		assert_eq!(tags[0].as_str(), Some("rust"));
		assert_eq!(tags[1].as_str(), Some("markdown"));
	}

	#[test]
	fn test_serialization_compact_and_pretty() {
		let mut obj = JsonObject::new();
		obj.insert("valid", true);
		obj.insert("count", 42);
		let val = JsonValue::Object(obj);

		assert_eq!(val.to_string(), "{\"valid\":true,\"count\":42}");
		assert_eq!(
			val.to_string_pretty(),
			"{\n  \"valid\": true,\n  \"count\": 42\n}"
		);
	}

	#[test]
	fn test_json_macro() {
		let v1 = json!({ "flag": false });
		assert_eq!(v1.get("flag").unwrap().as_bool(), Some(false));

		let v2 = json!({ "max-line-length": 120 });
		assert_eq!(v2.get("max-line-length").unwrap().as_u64(), Some(120));

		let v3 = json!({});
		assert!(v3.is_object());
	}

	#[test]
	fn test_unicode_and_escapes() {
		let json_str = r#""\u0041\u0042\u0043 \uD83D\uDE00 \t\r\n\\\"""#;
		let val = JsonValue::parse(json_str).unwrap();
		assert_eq!(val.as_str().unwrap(), "ABC 😀 \t\r\n\\\"");
	}

	#[test]
	fn test_number_parsing_exponents() {
		let val = JsonValue::parse("1.25e+3").unwrap();
		assert_eq!(val.as_f64(), Some(1250.0));

		let val = JsonValue::parse("-2e-2").unwrap();
		assert_eq!(val.as_f64(), Some(-0.02));
	}

	#[test]
	fn test_invalid_json_rejected() {
		assert!(JsonValue::parse("").is_err());
		assert!(JsonValue::parse("{, }").is_err());
		assert!(JsonValue::parse("[1, 2, ]").is_err());
		assert!(JsonValue::parse("{\"a\": 1,}").is_err());
		assert!(JsonValue::parse("{\"a\"}").is_err());
		assert!(JsonValue::parse("true trailing").is_err());
	}

	#[test]
	fn test_json_object_methods() {
		let mut obj = JsonObject::with_capacity(4);
		assert!(obj.is_empty());
		obj.insert("a", 1);
		obj.insert("b", 2);
		assert_eq!(obj.len(), 2);
		assert_eq!(obj.get("a").unwrap().as_u64(), Some(1));
		assert_eq!(obj.remove("a").unwrap().as_u64(), Some(1));
		assert_eq!(obj.len(), 1);
		assert!(obj.get("a").is_none());
	}
}
