#[derive(Clone)]
pub struct FormatOptions {
	pub remove_bold: bool,
	pub compact_blank_lines: bool,
	pub trim_trailing_whitespace: bool,
	pub collapse_spaces: bool,
	pub remove_horizontal_rules: bool,
	pub remove_emphasis: bool,
	pub blanks_around_lists: bool,
	pub blanks_around_fences: bool,
	pub blanks_around_headings: bool,
}

impl FormatOptions {
	#[allow(dead_code)]
	pub fn token_saver() -> Self {
		Self {
			remove_bold: true,
			compact_blank_lines: true,
			trim_trailing_whitespace: true,
			collapse_spaces: true,
			remove_horizontal_rules: true,
			remove_emphasis: true,
			blanks_around_lists: true,
			blanks_around_fences: true,
			blanks_around_headings: true,
		}
	}
}

impl Default for FormatOptions {
	fn default() -> Self {
		Self {
			remove_bold: true,
			compact_blank_lines: true,
			trim_trailing_whitespace: true,
			collapse_spaces: true,
			remove_horizontal_rules: true,
			remove_emphasis: true,
			blanks_around_lists: true,
			blanks_around_fences: true,
			blanks_around_headings: true,
		}
	}
}
