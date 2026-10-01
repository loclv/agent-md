//! Zero-dependency date and time utility functions and structures.
//!
//! Provides lightweight date and time handling without external dependencies,
//! including UTC timestamp generation, ISO 8601 / RFC 3339 parsing and formatting,
//! civil calendar conversions, leap year calculations, and custom strftime-style formatting.

use std::error::Error;
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

/// Day of the week.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Weekday {
	/// Sunday (0)
	Sunday = 0,
	/// Monday (1)
	Monday = 1,
	/// Tuesday (2)
	Tuesday = 2,
	/// Wednesday (3)
	Wednesday = 3,
	/// Thursday (4)
	Thursday = 4,
	/// Friday (5)
	Friday = 5,
	/// Saturday (6)
	Saturday = 6,
}

impl Weekday {
	/// Returns the 3-letter English abbreviation for the weekday.
	#[must_use]
	pub fn short_name(&self) -> &'static str {
		match self {
			Self::Sunday => "Sun",
			Self::Monday => "Mon",
			Self::Tuesday => "Tue",
			Self::Wednesday => "Wed",
			Self::Thursday => "Thu",
			Self::Friday => "Fri",
			Self::Saturday => "Sat",
		}
	}

	/// Returns the full English name for the weekday.
	#[must_use]
	pub fn full_name(&self) -> &'static str {
		match self {
			Self::Sunday => "Sunday",
			Self::Monday => "Monday",
			Self::Tuesday => "Tuesday",
			Self::Wednesday => "Wednesday",
			Self::Thursday => "Thursday",
			Self::Friday => "Friday",
			Self::Saturday => "Saturday",
		}
	}

	/// Returns the weekday number where Sunday is 0 and Saturday is 6.
	#[must_use]
	pub fn num_days_from_sunday(&self) -> u8 {
		*self as u8
	}

	/// Returns the ISO 8601 weekday number where Monday is 1 and Sunday is 7.
	#[must_use]
	pub fn number_from_monday(&self) -> u8 {
		match self {
			Self::Sunday => 7,
			Self::Monday => 1,
			Self::Tuesday => 2,
			Self::Wednesday => 3,
			Self::Thursday => 4,
			Self::Friday => 5,
			Self::Saturday => 6,
		}
	}
}

/// Errors occurring during date and time operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DateTimeError {
	/// The provided year, month, or day values do not form a valid calendar date.
	InvalidDate {
		/// Year value
		year: i32,
		/// Month value (1..=12)
		month: u8,
		/// Day value (1..=31)
		day: u8,
	},
	/// The provided hour, minute, or second values do not form a valid time.
	InvalidTime {
		/// Hour value (0..=23)
		hour: u8,
		/// Minute value (0..=59)
		minute: u8,
		/// Second value (0..=59)
		second: u8,
	},
	/// Nanoseconds value is out of range (must be < 1,000,000,000).
	InvalidNanos(u32),
	/// Failed to parse a date or time string.
	ParseError(String),
}

impl fmt::Display for DateTimeError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::InvalidDate { year, month, day } => {
				write!(f, "Invalid calendar date: {year:04}-{month:02}-{day:02}")
			}
			Self::InvalidTime {
				hour,
				minute,
				second,
			} => {
				write!(f, "Invalid time: {hour:02}:{minute:02}:{second:02}")
			}
			Self::InvalidNanos(nanos) => {
				write!(f, "Invalid nanoseconds: {nanos} (must be < 1_000_000_000)")
			}
			Self::ParseError(msg) => write!(f, "Date/time parse error: {msg}"),
		}
	}
}

impl Error for DateTimeError {}

/// Check if a given year is a leap year in the Gregorian calendar.
#[must_use]
pub fn is_leap_year(year: i32) -> bool {
	(year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Returns the number of days in the specified month of the given year.
///
/// Returns 0 if `month` is not in the range 1..=12.
#[must_use]
pub fn days_in_month(year: i32, month: u8) -> u8 {
	match month {
		1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
		4 | 6 | 9 | 11 => 30,
		2 => {
			if is_leap_year(year) {
				29
			} else {
				28
			}
		}
		_ => 0,
	}
}

/// Convert civil date (year, month, day) to days since Unix epoch (1970-01-01).
///
/// Uses Howard Hinnant's algorithm.
#[must_use]
pub fn days_from_civil(year: i32, month: u8, day: u8) -> i64 {
	let y = year as i64 - if month <= 2 { 1 } else { 0 };
	let era = (if y >= 0 { y } else { y - 399 }) / 400;
	let yoe = (y - era * 400) as u32;
	let m_adj = if month > 2 {
		month as u32 - 3
	} else {
		month as u32 + 9
	};
	let doy = (153 * m_adj + 2) / 5 + day as u32 - 1;
	let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
	era * 146097 + doe as i64 - 719468
}

/// Convert days since Unix epoch (1970-01-01) to civil date `(year, month, day)`.
///
/// Uses Howard Hinnant's algorithm.
#[must_use]
pub fn civil_from_days(days: i64) -> (i32, u8, u8) {
	let z = days + 719468;
	let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
	let doe = (z - era * 146097) as u32;
	let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
	let y = (yoe as i64) + era * 400;
	let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
	let mp = (5 * doy + 2) / 153;
	let d = (doy - (153 * mp + 2) / 5 + 1) as u8;
	let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u8;
	let y = if m <= 2 { y + 1 } else { y };
	(y as i32, m, d)
}

/// Represents a calendar date (year, month, day) in the Gregorian calendar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
	year: i32,
	month: u8,
	day: u8,
}

impl Date {
	/// Creates a new `Date` after validating month and day ranges.
	pub fn new(year: i32, month: u8, day: u8) -> Result<Self, DateTimeError> {
		if !(1..=12).contains(&month) || !(1..=days_in_month(year, month)).contains(&day) {
			return Err(DateTimeError::InvalidDate { year, month, day });
		}
		Ok(Self { year, month, day })
	}

	/// Returns the year component.
	#[must_use]
	pub fn year(&self) -> i32 {
		self.year
	}

	/// Returns the month component (1..=12).
	#[must_use]
	pub fn month(&self) -> u8 {
		self.month
	}

	/// Returns the day of month component (1..=31).
	#[must_use]
	pub fn day(&self) -> u8 {
		self.day
	}

	/// Returns true if this date is in a leap year.
	#[must_use]
	pub fn is_leap_year(&self) -> bool {
		is_leap_year(self.year)
	}

	/// Returns the day of the week for this date.
	#[must_use]
	pub fn weekday(&self) -> Weekday {
		let days = self.days_since_epoch();
		let idx = (days + 4).rem_euclid(7) as u8;
		match idx {
			0 => Weekday::Sunday,
			1 => Weekday::Monday,
			2 => Weekday::Tuesday,
			3 => Weekday::Wednesday,
			4 => Weekday::Thursday,
			5 => Weekday::Friday,
			_ => Weekday::Saturday,
		}
	}

	/// Returns the number of days since the Unix epoch (1970-01-01).
	#[must_use]
	pub fn days_since_epoch(&self) -> i64 {
		days_from_civil(self.year, self.month, self.day)
	}

	/// Creates a `Date` from the number of days since the Unix epoch (1970-01-01).
	#[must_use]
	pub fn from_days_since_epoch(days: i64) -> Self {
		let (year, month, day) = civil_from_days(days);
		Self { year, month, day }
	}

	/// Returns the current UTC date based on system time.
	#[must_use]
	pub fn today_utc() -> Self {
		DateTime::now_utc().date()
	}

	/// Parses a date string formatted as `YYYY-MM-DD`.
	pub fn parse(s: &str) -> Result<Self, DateTimeError> {
		let trimmed = s.trim();
		let parts: Vec<&str> = trimmed.split('-').collect();
		if parts.len() != 3 {
			return Err(DateTimeError::ParseError(format!(
				"Expected YYYY-MM-DD format, got: '{trimmed}'"
			)));
		}
		let year = parts[0]
			.parse::<i32>()
			.map_err(|_| DateTimeError::ParseError(format!("Invalid year: '{}'", parts[0])))?;
		let month = parts[1]
			.parse::<u8>()
			.map_err(|_| DateTimeError::ParseError(format!("Invalid month: '{}'", parts[1])))?;
		let day = parts[2]
			.parse::<u8>()
			.map_err(|_| DateTimeError::ParseError(format!("Invalid day: '{}'", parts[2])))?;
		Self::new(year, month, day)
	}

	/// Formats the date according to standard format tokens.
	#[must_use]
	pub fn format(&self, fmt_str: &str) -> String {
		let dt = DateTime {
			year: self.year,
			month: self.month,
			day: self.day,
			hour: 0,
			minute: 0,
			second: 0,
			nanos: 0,
		};
		dt.format(fmt_str)
	}
}

impl fmt::Display for Date {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
	}
}

/// Represents a combined date and time with UTC semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DateTime {
	year: i32,
	month: u8,
	day: u8,
	hour: u8,
	minute: u8,
	second: u8,
	nanos: u32,
}

impl DateTime {
	/// Creates a new `DateTime` with 0 nanoseconds after validating values.
	pub fn new(
		year: i32,
		month: u8,
		day: u8,
		hour: u8,
		minute: u8,
		second: u8,
	) -> Result<Self, DateTimeError> {
		Self::with_nanos(year, month, day, hour, minute, second, 0)
	}

	/// Creates a new `DateTime` with nanoseconds after validating values.
	pub fn with_nanos(
		year: i32,
		month: u8,
		day: u8,
		hour: u8,
		minute: u8,
		second: u8,
		nanos: u32,
	) -> Result<Self, DateTimeError> {
		if !(1..=12).contains(&month) || !(1..=days_in_month(year, month)).contains(&day) {
			return Err(DateTimeError::InvalidDate { year, month, day });
		}
		if hour > 23 || minute > 59 || second > 59 {
			return Err(DateTimeError::InvalidTime {
				hour,
				minute,
				second,
			});
		}
		if nanos >= 1_000_000_000 {
			return Err(DateTimeError::InvalidNanos(nanos));
		}
		Ok(Self {
			year,
			month,
			day,
			hour,
			minute,
			second,
			nanos,
		})
	}

	/// Returns the current UTC date and time based on system time.
	#[must_use]
	pub fn now_utc() -> Self {
		let now = SystemTime::now();
		match now.duration_since(UNIX_EPOCH) {
			Ok(duration) => {
				let secs = duration.as_secs() as i64;
				let nanos = duration.subsec_nanos();
				let mut dt = Self::from_unix_secs(secs);
				dt.nanos = nanos;
				dt
			}
			Err(err) => {
				let d = err.duration();
				let total_secs = d.as_secs() as i64;
				let sub_nanos = d.subsec_nanos();
				let (secs, nanos) = if sub_nanos == 0 {
					(-total_secs, 0)
				} else {
					(-total_secs - 1, 1_000_000_000 - sub_nanos)
				};
				let mut dt = Self::from_unix_secs(secs);
				dt.nanos = nanos;
				dt
			}
		}
	}

	/// Converts Unix epoch seconds to UTC `DateTime`.
	#[must_use]
	pub fn from_unix_secs(secs: i64) -> Self {
		let days = secs.div_euclid(86400);
		let secs_of_day = secs.rem_euclid(86400);
		let (year, month, day) = civil_from_days(days);
		let hour = (secs_of_day / 3600) as u8;
		let minute = ((secs_of_day % 3600) / 60) as u8;
		let second = (secs_of_day % 60) as u8;
		Self {
			year,
			month,
			day,
			hour,
			minute,
			second,
			nanos: 0,
		}
	}

	/// Converts Unix epoch milliseconds to UTC `DateTime`.
	#[must_use]
	pub fn from_unix_millis(millis: i64) -> Self {
		let secs = millis.div_euclid(1000);
		let rem_millis = millis.rem_euclid(1000) as u32;
		let mut dt = Self::from_unix_secs(secs);
		dt.nanos = rem_millis * 1_000_000;
		dt
	}

	/// Converts this `DateTime` to Unix epoch seconds.
	#[must_use]
	pub fn to_unix_secs(&self) -> i64 {
		let days = days_from_civil(self.year, self.month, self.day);
		days * 86400 + self.hour as i64 * 3600 + self.minute as i64 * 60 + self.second as i64
	}

	/// Converts this `DateTime` to Unix epoch milliseconds.
	#[must_use]
	pub fn to_unix_millis(&self) -> i64 {
		self.to_unix_secs() * 1000 + (self.nanos / 1_000_000) as i64
	}

	/// Returns the year component.
	#[must_use]
	pub fn year(&self) -> i32 {
		self.year
	}

	/// Returns the month component (1..=12).
	#[must_use]
	pub fn month(&self) -> u8 {
		self.month
	}

	/// Returns the day of month component (1..=31).
	#[must_use]
	pub fn day(&self) -> u8 {
		self.day
	}

	/// Returns the hour component (0..=23).
	#[must_use]
	pub fn hour(&self) -> u8 {
		self.hour
	}

	/// Returns the minute component (0..=59).
	#[must_use]
	pub fn minute(&self) -> u8 {
		self.minute
	}

	/// Returns the second component (0..=59).
	#[must_use]
	pub fn second(&self) -> u8 {
		self.second
	}

	/// Returns the nanoseconds component (0..999_999_999).
	#[must_use]
	pub fn nanos(&self) -> u32 {
		self.nanos
	}

	/// Extracts the `Date` portion.
	#[must_use]
	pub fn date(&self) -> Date {
		Date {
			year: self.year,
			month: self.month,
			day: self.day,
		}
	}

	/// Returns the day of week.
	#[must_use]
	pub fn weekday(&self) -> Weekday {
		self.date().weekday()
	}

	/// Formats the date and time as an RFC 3339 / ISO 8601 UTC string (`YYYY-MM-DDTHH:MM:SSZ`).
	#[must_use]
	pub fn to_rfc3339(&self) -> String {
		format!(
			"{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
			self.year, self.month, self.day, self.hour, self.minute, self.second
		)
	}

	/// Formats the date and time as RFC 3339 with millisecond precision.
	#[must_use]
	pub fn to_rfc3339_millis(&self) -> String {
		let millis = self.nanos / 1_000_000;
		format!(
			"{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
			self.year, self.month, self.day, self.hour, self.minute, self.second, millis
		)
	}

	/// Parses an RFC 3339 or ISO 8601 string into a UTC `DateTime`.
	///
	/// Supports formats such as:
	/// - `2026-10-01T04:45:49Z`
	/// - `2026-10-01T04:45:49.123Z`
	/// - `2026-10-01T04:45:49+07:00`
	/// - `2026-10-01 04:45:49`
	/// - `2026-10-01`
	pub fn parse_rfc3339(input: &str) -> Result<Self, DateTimeError> {
		let s = input.trim();
		if s.len() < 10 {
			return Err(DateTimeError::ParseError(format!(
				"Timestamp too short: '{s}'"
			)));
		}

		// Split date and time
		let (date_str, time_str) = if let Some(pos) = s.find(['T', 't', ' ']) {
			(&s[..pos], Some(&s[pos + 1..]))
		} else {
			(s, None)
		};

		let date = Date::parse(date_str)?;

		let Some(raw_time) = time_str else {
			return Ok(Self {
				year: date.year,
				month: date.month,
				day: date.day,
				hour: 0,
				minute: 0,
				second: 0,
				nanos: 0,
			});
		};

		// Parse timezone offset from end
		let (time_no_tz, tz_offset_secs) = parse_tz_offset(raw_time)?;
		let (hms_str, nanos) = parse_hms_and_fraction(time_no_tz)?;

		let hms_parts: Vec<&str> = hms_str.split(':').collect();
		if hms_parts.len() < 2 || hms_parts.len() > 3 {
			return Err(DateTimeError::ParseError(format!(
				"Invalid time component in timestamp: '{hms_str}'"
			)));
		}

		let hour = hms_parts[0]
			.parse::<u8>()
			.map_err(|_| DateTimeError::ParseError(format!("Invalid hour: '{}'", hms_parts[0])))?;
		let minute = hms_parts[1].parse::<u8>().map_err(|_| {
			DateTimeError::ParseError(format!("Invalid minute: '{}'", hms_parts[1]))
		})?;
		let second = if hms_parts.len() == 3 {
			hms_parts[2].parse::<u8>().map_err(|_| {
				DateTimeError::ParseError(format!("Invalid second: '{}'", hms_parts[2]))
			})?
		} else {
			0
		};

		let dt = Self::with_nanos(date.year, date.month, date.day, hour, minute, second, nanos)?;

		if tz_offset_secs != 0 {
			let local_secs = dt.to_unix_secs();
			let utc_secs = local_secs - tz_offset_secs;
			let mut utc_dt = Self::from_unix_secs(utc_secs);
			utc_dt.nanos = nanos;
			Ok(utc_dt)
		} else {
			Ok(dt)
		}
	}

	/// Formats the date/time using strftime-style format tokens.
	///
	/// Supported tokens:
	/// - `%Y`: 4-digit year (e.g. 2026)
	/// - `%y`: 2-digit year (e.g. 26)
	/// - `%C`: Century (e.g. 20)
	/// - `%m`: 2-digit month (01..12)
	/// - `%B`: Month full name (e.g. October)
	/// - `%b`, `%h`: Month abbreviation (e.g. Oct)
	/// - `%d`: 2-digit day of month (01..31)
	/// - `%e`: Day of month, space padded ( 1..31)
	/// - `%H`: Hour 00..23
	/// - `%I`: Hour 01..12
	/// - `%p`: AM / PM
	/// - `%P`: am / pm
	/// - `%M`: Minute 00..59
	/// - `%S`: Second 00..59
	/// - `%f`: Nanoseconds (9 digits)
	/// - `%3f`: Milliseconds (3 digits)
	/// - `%6f`: Microseconds (6 digits)
	/// - `%A`: Weekday full name (e.g. Thursday)
	/// - `%a`: Weekday abbreviation (e.g. Thu)
	/// - `%u`: Weekday number (1 = Monday .. 7 = Sunday)
	/// - `%w`: Weekday number (0 = Sunday .. 6 = Saturday)
	/// - `%s`: Unix epoch timestamp in seconds
	/// - `%F`: Equivalent to `%Y-%m-%d`
	/// - `%T`: Equivalent to `%H:%M:%S`
	/// - `%R`: Equivalent to `%H:%M`
	/// - `%Z`: Timezone name (`UTC`)
	/// - `%z`: Timezone offset (`+0000`)
	/// - `%%`: Literal `%`
	#[must_use]
	pub fn format(&self, fmt_str: &str) -> String {
		let mut output = String::with_capacity(fmt_str.len() * 2);
		let mut chars = fmt_str.chars().peekable();

		while let Some(ch) = chars.next() {
			if ch != '%' {
				output.push(ch);
				continue;
			}

			// Specifier token
			match chars.next() {
				Some('Y') => output.push_str(&format!("{:04}", self.year)),
				Some('y') => output.push_str(&format!("{:02}", (self.year.abs() % 100))),
				Some('C') => output.push_str(&format!("{:02}", (self.year / 100))),
				Some('m') => output.push_str(&format!("{:02}", self.month)),
				Some('B') => output.push_str(month_name(self.month)),
				Some('b') | Some('h') => output.push_str(month_short_name(self.month)),
				Some('d') => output.push_str(&format!("{:02}", self.day)),
				Some('e') => output.push_str(&format!("{:2}", self.day)),
				Some('H') => output.push_str(&format!("{:02}", self.hour)),
				Some('I') => {
					let h = match self.hour % 12 {
						0 => 12,
						val => val,
					};
					output.push_str(&format!("{h:02}"));
				}
				Some('p') => output.push_str(if self.hour < 12 { "AM" } else { "PM" }),
				Some('P') => output.push_str(if self.hour < 12 { "am" } else { "pm" }),
				Some('M') => output.push_str(&format!("{:02}", self.minute)),
				Some('S') => output.push_str(&format!("{:02}", self.second)),
				Some('f') => output.push_str(&format!("{:09}", self.nanos)),
				Some('3') => {
					if chars.peek() == Some(&'f') {
						chars.next();
						output.push_str(&format!("{:03}", self.nanos / 1_000_000));
					} else {
						output.push('%');
						output.push('3');
					}
				}
				Some('6') => {
					if chars.peek() == Some(&'f') {
						chars.next();
						output.push_str(&format!("{:06}", self.nanos / 1_000));
					} else {
						output.push('%');
						output.push('6');
					}
				}
				Some('A') => output.push_str(self.weekday().full_name()),
				Some('a') => output.push_str(self.weekday().short_name()),
				Some('u') => output.push_str(&self.weekday().number_from_monday().to_string()),
				Some('w') => output.push_str(&self.weekday().num_days_from_sunday().to_string()),
				Some('s') => output.push_str(&self.to_unix_secs().to_string()),
				Some('F') => output.push_str(&format!(
					"{:04}-{:02}-{:02}",
					self.year, self.month, self.day
				)),
				Some('T') => output.push_str(&format!(
					"{:02}:{:02}:{:02}",
					self.hour, self.minute, self.second
				)),
				Some('R') => output.push_str(&format!("{:02}:{:02}", self.hour, self.minute)),
				Some('Z') => output.push_str("UTC"),
				Some('z') => output.push_str("+0000"),
				Some('%') => output.push('%'),
				Some(other) => {
					output.push('%');
					output.push(other);
				}
				None => output.push('%'),
			}
		}

		output
	}
}

impl fmt::Display for DateTime {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.to_rfc3339())
	}
}

/// Helper to parse timezone offset string and return (trimmed_time_str, offset_seconds).
fn parse_tz_offset(s: &str) -> Result<(&str, i64), DateTimeError> {
	let s = s.trim();
	if s.ends_with('Z') || s.ends_with('z') {
		return Ok((&s[..s.len() - 1], 0));
	}

	if let Some(pos) = s.rfind(['+', '-']) {
		// Ensure pos is past the time component (HH:MM)
		if pos > 2 {
			let sign = if &s[pos..pos + 1] == "+" { 1 } else { -1 };
			let offset_part = &s[pos + 1..];
			let offset_secs = parse_offset_string(offset_part, sign)?;
			return Ok((&s[..pos], offset_secs));
		}
	}

	Ok((s, 0))
}

/// Parses offset part like "07:00", "0700", or "07" into total seconds.
fn parse_offset_string(part: &str, sign: i64) -> Result<i64, DateTimeError> {
	let cleaned = part.replace(':', "");
	match cleaned.len() {
		2 => {
			let h = cleaned.parse::<i64>().map_err(|_| {
				DateTimeError::ParseError(format!("Invalid timezone offset: '{part}'"))
			})?;
			Ok(sign * h * 3600)
		}
		4 => {
			let h = cleaned[..2].parse::<i64>().map_err(|_| {
				DateTimeError::ParseError(format!("Invalid timezone offset: '{part}'"))
			})?;
			let m = cleaned[2..].parse::<i64>().map_err(|_| {
				DateTimeError::ParseError(format!("Invalid timezone offset: '{part}'"))
			})?;
			Ok(sign * (h * 3600 + m * 60))
		}
		_ => Err(DateTimeError::ParseError(format!(
			"Invalid timezone offset: '{part}'"
		))),
	}
}

/// Separates HMS string from fractional seconds, returning `(hms_str, nanos)`.
fn parse_hms_and_fraction(s: &str) -> Result<(&str, u32), DateTimeError> {
	if let Some(pos) = s.find(['.', ',']) {
		let hms = &s[..pos];
		let frac = &s[pos + 1..];
		let mut frac_padded = String::new();
		for ch in frac.chars() {
			if ch.is_ascii_digit() {
				frac_padded.push(ch);
			} else {
				break;
			}
		}
		if frac_padded.is_empty() {
			return Err(DateTimeError::ParseError(format!(
				"Empty fractional second in: '{s}'"
			)));
		}
		while frac_padded.len() < 9 {
			frac_padded.push('0');
		}
		frac_padded.truncate(9);
		let nanos = frac_padded
			.parse::<u32>()
			.map_err(|_| DateTimeError::ParseError(format!("Invalid fraction: '{frac}'")))?;
		Ok((hms, nanos))
	} else {
		Ok((s, 0))
	}
}

/// Returns the full English name of a month (1..=12).
fn month_name(month: u8) -> &'static str {
	match month {
		1 => "January",
		2 => "February",
		3 => "March",
		4 => "April",
		5 => "May",
		6 => "June",
		7 => "July",
		8 => "August",
		9 => "September",
		10 => "October",
		11 => "November",
		12 => "December",
		_ => "",
	}
}

/// Returns the 3-letter English abbreviation of a month (1..=12).
fn month_short_name(month: u8) -> &'static str {
	match month {
		1 => "Jan",
		2 => "Feb",
		3 => "Mar",
		4 => "Apr",
		5 => "May",
		6 => "Jun",
		7 => "Jul",
		8 => "Aug",
		9 => "Sep",
		10 => "Oct",
		11 => "Nov",
		12 => "Dec",
		_ => "",
	}
}

/// Returns the current UTC timestamp formatted as ISO 8601 (`YYYY-MM-DDTHH:MM:SSZ`).
#[must_use]
pub fn now_iso8601() -> String {
	DateTime::now_utc().to_rfc3339()
}

/// Returns the current UTC date formatted as ISO 8601 (`YYYY-MM-DD`).
#[must_use]
pub fn today_iso8601() -> String {
	Date::today_utc().to_string()
}

/// Formats a Unix timestamp in seconds using strftime-style format tokens.
#[must_use]
pub fn format_unix_timestamp(secs: i64, fmt_str: &str) -> String {
	DateTime::from_unix_secs(secs).format(fmt_str)
}

/// Parses an ISO 8601 / RFC 3339 timestamp string into a UTC `DateTime`.
pub fn parse_iso8601(s: &str) -> Result<DateTime, DateTimeError> {
	DateTime::parse_rfc3339(s)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_leap_year_calculation() {
		assert!(is_leap_year(2000));
		assert!(is_leap_year(2024));
		assert!(is_leap_year(2028));
		assert!(!is_leap_year(1900));
		assert!(!is_leap_year(2021));
		assert!(!is_leap_year(2022));
		assert!(!is_leap_year(2023));
	}

	#[test]
	fn test_days_in_month() {
		assert_eq!(days_in_month(2024, 2), 29);
		assert_eq!(days_in_month(2023, 2), 28);
		assert_eq!(days_in_month(2024, 1), 31);
		assert_eq!(days_in_month(2024, 4), 30);
		assert_eq!(days_in_month(2024, 13), 0);
	}

	#[test]
	fn test_civil_date_roundtrip() {
		let test_cases = [
			(1970, 1, 1),
			(1969, 12, 31),
			(2000, 2, 29),
			(2024, 2, 29),
			(2026, 10, 1),
			(1800, 1, 1),
			(2100, 12, 31),
		];
		for (y, m, d) in test_cases {
			let days = days_from_civil(y, m, d);
			let (ry, rm, rd) = civil_from_days(days);
			assert_eq!((y, m, d), (ry, rm, rd));
		}
	}

	#[test]
	fn test_date_weekday() {
		// 1970-01-01 was Thursday
		let d = Date::new(1970, 1, 1).unwrap();
		assert_eq!(d.weekday(), Weekday::Thursday);
		assert_eq!(d.weekday().short_name(), "Thu");
		assert_eq!(d.weekday().full_name(), "Thursday");
		assert_eq!(d.weekday().num_days_from_sunday(), 4);
		assert_eq!(d.weekday().number_from_monday(), 4);

		// 2026-10-01 is Thursday
		let d = Date::new(2026, 10, 1).unwrap();
		assert_eq!(d.weekday(), Weekday::Thursday);

		// 2024-02-29 was Thursday
		let d = Date::new(2024, 2, 29).unwrap();
		assert_eq!(d.weekday(), Weekday::Thursday);

		// 2026-09-27 was Sunday
		let d = Date::new(2026, 9, 27).unwrap();
		assert_eq!(d.weekday(), Weekday::Sunday);
		assert_eq!(d.weekday().number_from_monday(), 7);
	}

	#[test]
	fn test_date_parse() {
		let d = Date::parse("2026-10-01").unwrap();
		assert_eq!(d.year(), 2026);
		assert_eq!(d.month(), 10);
		assert_eq!(d.day(), 1);
		assert_eq!(d.to_string(), "2026-10-01");

		assert!(Date::parse("2024-02-29").is_ok());
		assert!(Date::parse("2023-02-29").is_err());
		assert!(Date::parse("invalid").is_err());
	}

	#[test]
	fn test_datetime_unix_secs() {
		let dt = DateTime::new(1970, 1, 1, 0, 0, 0).unwrap();
		assert_eq!(dt.to_unix_secs(), 0);

		let dt = DateTime::from_unix_secs(0);
		assert_eq!(dt.year(), 1970);
		assert_eq!(dt.month(), 1);
		assert_eq!(dt.day(), 1);
		assert_eq!(dt.hour(), 0);
		assert_eq!(dt.minute(), 0);
		assert_eq!(dt.second(), 0);

		let dt = DateTime::from_unix_secs(1_700_000_000);
		assert_eq!(dt.to_unix_secs(), 1_700_000_000);
	}

	#[test]
	fn test_datetime_rfc3339() {
		let dt = DateTime::new(2026, 10, 1, 4, 45, 49).unwrap();
		assert_eq!(dt.to_rfc3339(), "2026-10-01T04:45:49Z");

		let parsed = DateTime::parse_rfc3339("2026-10-01T04:45:49Z").unwrap();
		assert_eq!(parsed, dt);

		let parsed_tz = DateTime::parse_rfc3339("2026-10-01T11:45:49+07:00").unwrap();
		assert_eq!(parsed_tz.to_unix_secs(), dt.to_unix_secs());
		assert_eq!(parsed_tz.to_rfc3339(), "2026-10-01T04:45:49Z");
	}

	#[test]
	fn test_datetime_fractional() {
		let dt = DateTime::with_nanos(2026, 10, 1, 4, 45, 49, 123_000_000).unwrap();
		assert_eq!(dt.to_rfc3339_millis(), "2026-10-01T04:45:49.123Z");

		let parsed = DateTime::parse_rfc3339("2026-10-01T04:45:49.123Z").unwrap();
		assert_eq!(parsed.nanos(), 123_000_000);
	}

	#[test]
	fn test_datetime_format() {
		let dt = DateTime::new(2026, 10, 1, 4, 45, 49).unwrap();
		assert_eq!(dt.format("%Y-%m-%d"), "2026-10-01");
		assert_eq!(dt.format("%F %T"), "2026-10-01 04:45:49");
		assert_eq!(dt.format("%B %d, %Y"), "October 01, 2026");
		assert_eq!(dt.format("%A (%a)"), "Thursday (Thu)");
		assert_eq!(dt.format("Literal %% percent"), "Literal % percent");
	}

	#[test]
	fn test_now_functions() {
		let now = now_iso8601();
		assert!(now.ends_with('Z'));
		assert!(now.contains('T'));

		let today = today_iso8601();
		assert_eq!(today.len(), 10);
		assert_eq!(&today[4..5], "-");
		assert_eq!(&today[7..8], "-");
	}

	#[test]
	fn test_invalid_dates_and_times() {
		assert!(Date::new(2026, 0, 1).is_err());
		assert!(Date::new(2026, 13, 1).is_err());
		assert!(Date::new(2026, 4, 31).is_err());
		assert!(Date::new(2026, 2, 29).is_err());
		assert!(Date::new(2024, 2, 29).is_ok());

		assert!(DateTime::new(2026, 1, 1, 24, 0, 0).is_err());
		assert!(DateTime::new(2026, 1, 1, 12, 60, 0).is_err());
		assert!(DateTime::new(2026, 1, 1, 12, 0, 60).is_err());
		assert!(DateTime::with_nanos(2026, 1, 1, 12, 0, 0, 1_000_000_000).is_err());
	}

	#[test]
	fn test_negative_timezone_offset() {
		let parsed = DateTime::parse_rfc3339("2026-10-01T00:00:00-04:00").unwrap();
		assert_eq!(parsed.to_rfc3339(), "2026-10-01T04:00:00Z");

		let parsed2 = DateTime::parse_rfc3339("2026-10-01 00:00:00-0500").unwrap();
		assert_eq!(parsed2.to_rfc3339(), "2026-10-01T05:00:00Z");
	}

	#[test]
	fn test_unix_millis() {
		let dt = DateTime::with_nanos(1970, 1, 1, 0, 0, 1, 500_000_000).unwrap();
		assert_eq!(dt.to_unix_millis(), 1500);

		let from_millis = DateTime::from_unix_millis(1500);
		assert_eq!(from_millis.to_unix_secs(), 1);
		assert_eq!(from_millis.nanos(), 500_000_000);
	}

	#[test]
	fn test_format_specifiers() {
		let dt = DateTime::with_nanos(2026, 10, 1, 15, 7, 9, 123_456_789).unwrap();
		assert_eq!(dt.format("%Y"), "2026");
		assert_eq!(dt.format("%y"), "26");
		assert_eq!(dt.format("%C"), "20");
		assert_eq!(dt.format("%m"), "10");
		assert_eq!(dt.format("%b"), "Oct");
		assert_eq!(dt.format("%h"), "Oct");
		assert_eq!(dt.format("%d"), "01");
		assert_eq!(dt.format("%e"), " 1");
		assert_eq!(dt.format("%H"), "15");
		assert_eq!(dt.format("%I"), "03");
		assert_eq!(dt.format("%p"), "PM");
		assert_eq!(dt.format("%P"), "pm");
		assert_eq!(dt.format("%M"), "07");
		assert_eq!(dt.format("%S"), "09");
		assert_eq!(dt.format("%f"), "123456789");
		assert_eq!(dt.format("%3f"), "123");
		assert_eq!(dt.format("%6f"), "123456");
		assert_eq!(dt.format("%R"), "15:07");
		assert_eq!(dt.format("%Z"), "UTC");
		assert_eq!(dt.format("%z"), "+0000");
		assert_eq!(dt.format("%u"), "4");
		assert_eq!(dt.format("%w"), "4");
	}

	#[test]
	fn test_error_display() {
		let err1 = DateTimeError::InvalidDate {
			year: 2026,
			month: 2,
			day: 30,
		};
		assert_eq!(err1.to_string(), "Invalid calendar date: 2026-02-30");

		let err2 = DateTimeError::InvalidTime {
			hour: 25,
			minute: 0,
			second: 0,
		};
		assert_eq!(err2.to_string(), "Invalid time: 25:00:00");

		let err3 = DateTimeError::InvalidNanos(1_000_000_000);
		assert!(err3.to_string().contains("1000000000"));

		let err4 = DateTimeError::ParseError("bad".to_string());
		assert!(err4.to_string().contains("bad"));
	}

	#[test]
	fn test_standalone_helpers() {
		let formatted = format_unix_timestamp(0, "%Y-%m-%d %H:%M:%S");
		assert_eq!(formatted, "1970-01-01 00:00:00");

		let parsed = parse_iso8601("2026-10-01T04:45:49Z").unwrap();
		assert_eq!(parsed.year(), 2026);
	}
}
