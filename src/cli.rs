//! Command-line interface definitions and arguments for `agent-md`.

use clap::{Parser, Subcommand};

/// Command-line arguments for agent-md.
#[derive(Parser, Debug, Clone)]
#[command(name = "agent-md")]
#[command(about = "Markdown editor for AI agents", long_about = None)]
#[command(disable_version_flag = true)]
pub struct Cli {
	/// Print version information
	#[arg(short = 'v', long = "version", help = "Print version information")]
	pub version: bool,

	/// Pretty print JSON output
	#[arg(long = "human", global = true, help = "Pretty print JSON output")]
	pub human: bool,

	/// Path to custom configuration file or directory
	#[arg(
		long = "config",
		global = true,
		help = "Path to custom configuration file or directory"
	)]
	pub config: Option<String>,

	/// Markdown file path (implies fmt command if no subcommand given)
	#[arg(value_name = "PATH")]
	pub path: Option<String>,

	/// Subcommand to execute
	#[command(subcommand)]
	pub command: Option<Commands>,
}

/// Available subcommands for agent-md.
#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
	/// Initialize a configuration file
	Init {
		/// Optional configuration file path or target directory
		#[arg(help = "Optional configuration file path or target directory")]
		path: Option<String>,
		/// Overwrite existing configuration file
		#[arg(short = 'f', long, help = "Overwrite existing configuration file")]
		force: bool,
	},
	/// Inspect or check configuration status
	Config {
		/// Optional configuration file path or directory
		#[arg(help = "Optional configuration file path or directory")]
		path: Option<String>,
		/// Only check if configuration file exists
		#[arg(long, help = "Only check if configuration file exists")]
		check: bool,
		/// Initialize a default configuration file
		#[arg(long, help = "Initialize a default configuration file")]
		init: bool,
		/// Overwrite existing configuration file
		#[arg(short = 'f', long, help = "Overwrite existing configuration file")]
		force: bool,
	},
	/// Read markdown document or specific fields/sections
	Read {
		/// Markdown file path
		#[arg(help = "Markdown file path")]
		path: String,
		/// Extract specific field (path, content, word_count, line_count, headings)
		#[arg(
			help = "Extract specific field (path, content, word_count, line_count, headings)",
			long,
			short = 'f'
		)]
		field: Option<String>,
		/// Extract specific section content by heading name
		#[arg(
			help = "Extract specific section content by heading name",
			long,
			short = 'c'
		)]
		content: Option<String>,
	},
	/// Write whole content to markdown file with lint validation
	Write {
		/// Markdown file path
		#[arg(help = "Markdown file path")]
		path: String,
		/// Content to write
		#[arg(help = "Content to write")]
		content: String,
	},
	/// Write content to a specific section by heading path
	WriteSection {
		/// Markdown file path
		#[arg(help = "Markdown file path")]
		path: String,
		/// Section heading path (e.g., '## Development' or '## Development > Build')
		#[arg(help = "Section heading path (e.g., '## Development' or '## Development > Build')")]
		section: String,
		/// Content to write to the section
		#[arg(help = "Content to write to the section")]
		content: String,
	},
	/// Append content to the end of a markdown file
	Append {
		/// Markdown file path
		#[arg(help = "Markdown file path")]
		path: String,
		/// Content to append
		#[arg(help = "Content to append")]
		content: String,
	},
	/// Insert content at a specific line number
	Insert {
		/// Markdown file path
		#[arg(help = "Markdown file path")]
		path: String,
		/// Line number to insert at
		#[arg(help = "Line number to insert at")]
		line: usize,
		/// Content to insert
		#[arg(help = "Content to insert")]
		content: String,
	},
	/// Delete lines from a markdown file
	Delete {
		/// Markdown file path
		#[arg(help = "Markdown file path")]
		path: String,
		/// Line number to delete
		#[arg(help = "Line number to delete")]
		line: usize,
		/// Number of lines to delete
		#[arg(help = "Number of lines to delete", default_value = "1")]
		count: usize,
	},
	/// List markdown files in a directory
	List {
		/// Directory to list
		#[arg(help = "Directory to list", default_value = ".")]
		path: String,
	},
	/// Search for text in a markdown file
	Search {
		/// Markdown file path
		#[arg(help = "Markdown file path")]
		path: String,
		/// Search query
		#[arg(help = "Search query")]
		query: String,
	},
	/// Extract all headings from a markdown file
	Headings {
		/// Markdown file path
		#[arg(help = "Markdown file path")]
		path: String,
	},
	/// Get document statistics
	Stats {
		/// Markdown file path
		#[arg(help = "Markdown file path")]
		path: String,
	},
	/// Export document sections to JSON Lines format
	ToJsonl {
		/// Markdown file path
		#[arg(help = "Markdown file path")]
		path: String,
	},
	/// Validate markdown content or file
	Lint {
		/// Markdown file path or content to validate
		#[arg(help = "Markdown file path or content to validate")]
		path: String,
		/// Validate content directly instead of file
		#[arg(
			help = "Validate content directly instead of file",
			long,
			default_value = "false"
		)]
		content: bool,
	},
	/// Validate a markdown file with human-readable output
	LintFile {
		/// Markdown file path to lint
		#[arg(help = "Markdown file path to lint")]
		path: String,
	},
	/// List ignored patterns merged from .markdownlintignore and .gitignore
	Ignore {
		/// Optional directory path (defaults to current directory)
		#[arg(help = "Optional directory path", default_value = ".")]
		path: String,
	},
	/// Format markdown file or standard input
	Fmt {
		/// Markdown file path to format
		#[arg(help = "Markdown file path to format")]
		path: Option<String>,
		/// Read from stdin, write to stdout
		#[arg(long, help = "Read from stdin, write to stdout")]
		stdin: bool,
		/// Remove bold markers (** and __)
		#[arg(
			long,
			help = "Remove bold markers (** and __)",
			num_args = 0..=1,
			default_missing_value = "true",
			require_equals = true
		)]
		remove_bold: Option<bool>,
		/// Compact blank lines (remove multiples)
		#[arg(
			long,
			help = "Compact blank lines (remove multiples)",
			num_args = 0..=1,
			default_missing_value = "true",
			require_equals = true
		)]
		compact_blank_lines: Option<bool>,
		/// Collapse multiple spaces between words
		#[arg(
			long,
			help = "Collapse multiple spaces between words",
			num_args = 0..=1,
			default_missing_value = "true",
			require_equals = true
		)]
		collapse_spaces: Option<bool>,
		/// Remove horizontal rules (---, ***, ___)
		#[arg(
			long,
			help = "Remove horizontal rules (---, ***, ___)",
			num_args = 0..=1,
			default_missing_value = "true",
			require_equals = true
		)]
		remove_horizontal_rules: Option<bool>,
		/// Remove emphasis markers (* and _)
		#[arg(
			long,
			help = "Remove emphasis markers (* and _)",
			num_args = 0..=1,
			default_missing_value = "true",
			require_equals = true
		)]
		remove_emphasis: Option<bool>,
		/// Minify HTML tags and remove useless whitespace
		#[arg(
			long,
			help = "Minify HTML tags and remove useless whitespace",
			num_args = 0..=1,
			default_missing_value = "true",
			require_equals = true
		)]
		minify_html: Option<bool>,
	},
}

/// Construct [`crate::format::FormatOptions`] based on CLI flags and configuration.
pub fn get_format_options(
	remove_bold: Option<bool>,
	compact_blank_lines: Option<bool>,
	collapse_spaces: Option<bool>,
	remove_horizontal_rules: Option<bool>,
	remove_emphasis: Option<bool>,
	minify_html: Option<bool>,
	custom_config: Option<&str>,
) -> crate::format::FormatOptions {
	get_format_options_for_target(
		remove_bold,
		compact_blank_lines,
		collapse_spaces,
		remove_horizontal_rules,
		remove_emphasis,
		minify_html,
		custom_config,
		None,
	)
}

/// Construct [`crate::format::FormatOptions`] based on CLI flags and configuration,
/// searching for configuration in the target path's directory and its ancestors.
pub fn get_format_options_for_target(
	remove_bold: Option<bool>,
	compact_blank_lines: Option<bool>,
	collapse_spaces: Option<bool>,
	remove_horizontal_rules: Option<bool>,
	remove_emphasis: Option<bool>,
	minify_html: Option<bool>,
	custom_config: Option<&str>,
	target_path: Option<&str>,
) -> crate::format::FormatOptions {
	let cfg = crate::config::resolve_config(
		crate::config::get_config_for_target(target_path, custom_config).as_ref(),
	);

	crate::format::FormatOptions {
		remove_bold: remove_bold.unwrap_or(cfg.remove_bold),
		compact_blank_lines: compact_blank_lines.unwrap_or(cfg.compact_blank_lines),
		trim_trailing_whitespace: true,
		collapse_spaces: collapse_spaces.unwrap_or(cfg.collapse_spaces),
		remove_horizontal_rules: remove_horizontal_rules.unwrap_or(cfg.remove_horizontal_rules),
		remove_emphasis: remove_emphasis.unwrap_or(cfg.remove_emphasis),
		blanks_around_lists: cfg.blanks_around_lists,
		blanks_around_fences: cfg.blanks_around_fences,
		blanks_around_headings: cfg.blanks_around_headings,
		minify_html: minify_html.unwrap_or(cfg.minify_html),
	}
}
