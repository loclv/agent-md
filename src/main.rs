use clap::{Parser, Subcommand};

pub mod commands;
mod format;
mod html_tests;
pub mod linter;
mod parser;
mod rules;
#[cfg(test)]
mod tests;
pub mod types;

#[derive(Parser)]
#[command(name = "agent-md")]
#[command(about = "Markdown editor for AI agents", long_about = None)]
#[command(disable_version_flag = true)]
pub struct Cli {
	#[arg(short = 'v', long = "version", help = "Print version information")]
	pub version: bool,
	#[arg(long = "human", help = "Pretty print JSON output")]
	pub human: bool,

	/// Markdown file path (implies fmt command if no subcommand given)
	#[arg(value_name = "PATH")]
	pub path: Option<String>,

	#[command(subcommand)]
	pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
	Read {
		#[arg(help = "Markdown file path")]
		path: String,
		#[arg(
			help = "Extract specific field (path, content, word_count, line_count, headings)",
			long,
			short = 'f'
		)]
		field: Option<String>,
		#[arg(
			help = "Extract specific section content by heading name",
			long,
			short = 'c'
		)]
		content: Option<String>,
	},
	Write {
		#[arg(help = "Markdown file path")]
		path: String,
		#[arg(help = "Content to write")]
		content: String,
	},
	WriteSection {
		#[arg(help = "Markdown file path")]
		path: String,
		#[arg(help = "Section heading path (e.g., '## Development' or '## Development > Build')")]
		section: String,
		#[arg(help = "Content to write to the section")]
		content: String,
	},
	Append {
		#[arg(help = "Markdown file path")]
		path: String,
		#[arg(help = "Content to append")]
		content: String,
	},
	Insert {
		#[arg(help = "Markdown file path")]
		path: String,
		#[arg(help = "Line number to insert at")]
		line: usize,
		#[arg(help = "Content to insert")]
		content: String,
	},
	Delete {
		#[arg(help = "Markdown file path")]
		path: String,
		#[arg(help = "Line number to delete")]
		line: usize,
		#[arg(help = "Number of lines to delete", default_value = "1")]
		count: usize,
	},
	List {
		#[arg(help = "Directory to list", default_value = ".")]
		path: String,
	},
	Search {
		#[arg(help = "Markdown file path")]
		path: String,
		#[arg(help = "Search query")]
		query: String,
	},
	Headings {
		#[arg(help = "Markdown file path")]
		path: String,
	},
	Stats {
		#[arg(help = "Markdown file path")]
		path: String,
	},
	ToJsonl {
		#[arg(help = "Markdown file path")]
		path: String,
	},
	Lint {
		#[arg(help = "Markdown file path or content to validate")]
		path: String,
		#[arg(
			help = "Validate content directly instead of file",
			long,
			default_value = "false"
		)]
		content: bool,
	},
	LintFile {
		#[arg(help = "Markdown file path to lint")]
		path: String,
	},
	Fmt {
		#[arg(help = "Markdown file path to format")]
		path: Option<String>,
		#[arg(long, help = "Read from stdin, write to stdout")]
		stdin: bool,
		#[arg(long, help = "Remove bold markers (** and __)", default_value = "true")]
		remove_bold: bool,
		#[arg(
			long,
			help = "Compact blank lines (remove multiples)",
			default_value = "true"
		)]
		compact_blank_lines: bool,
		#[arg(
			long,
			help = "Collapse multiple spaces between words",
			default_value = "true"
		)]
		collapse_spaces: bool,
		#[arg(
			long,
			help = "Remove horizontal rules (---, ***, ___)",
			default_value = "true"
		)]
		remove_horizontal_rules: bool,
		#[arg(
			long,
			help = "Remove emphasis markers (* and _)",
			default_value = "true"
		)]
		remove_emphasis: bool,
	},
}

fn get_format_options(
	remove_bold: bool,
	compact_blank_lines: bool,
	collapse_spaces: bool,
	remove_horizontal_rules: bool,
	remove_emphasis: bool,
) -> format::FormatOptions {
	let mut blanks_around_lists = true;
	let mut blanks_around_fences = true;
	let mut blanks_around_headings = true;

	if let Some(config) = linter::get_markdownlint_config() {
		if let Some(val) = config.get("blanks-around-lists") {
			if val.is_boolean() {
				blanks_around_lists = val.as_bool().unwrap();
			}
		}
		if let Some(val) = config.get("blanks-around-fences") {
			if val.is_boolean() {
				blanks_around_fences = val.as_bool().unwrap();
			}
		}
		if let Some(val) = config.get("blanks-around-headings") {
			if val.is_boolean() {
				blanks_around_headings = val.as_bool().unwrap();
			}
		}
	}

	format::FormatOptions {
		remove_bold,
		compact_blank_lines,
		trim_trailing_whitespace: true,
		collapse_spaces,
		remove_horizontal_rules,
		remove_emphasis,
		blanks_around_lists,
		blanks_around_fences,
		blanks_around_headings,
	}
}

fn main() {
	let cli = Cli::parse();

	if cli.version {
		// IMPORTANT: Update this version when releasing new versions
		println!("0.2.4");
		return;
	}

	match cli.command {
		Some(Commands::Read {
			path,
			field,
			content,
		}) => commands::cmd_read(&path, field.as_deref(), content.as_deref(), cli.human),
		Some(Commands::Write { path, content }) => commands::cmd_write(&path, &content, cli.human),
		Some(Commands::WriteSection {
			path,
			section,
			content,
		}) => commands::cmd_write_section(&path, &section, &content, cli.human),
		Some(Commands::Append { path, content }) => {
			commands::cmd_append(&path, &content, cli.human)
		}
		Some(Commands::Insert {
			path,
			line,
			content,
		}) => commands::cmd_insert(&path, line, &content, cli.human),
		Some(Commands::Delete { path, line, count }) => {
			commands::cmd_delete(&path, line, count, cli.human)
		}
		Some(Commands::List { path }) => commands::cmd_list(&path, cli.human),
		Some(Commands::Search { path, query }) => commands::cmd_search(&path, &query, cli.human),
		Some(Commands::Headings { path }) => commands::cmd_headings(&path, cli.human),
		Some(Commands::Stats { path }) => commands::cmd_stats(&path, cli.human),
		Some(Commands::ToJsonl { path }) => commands::cmd_to_jsonl(&path, cli.human),
		Some(Commands::Lint { path, content }) => commands::cmd_lint(&path, content, cli.human),
		Some(Commands::LintFile { path }) => commands::cmd_lint_file(&path, cli.human),
		Some(Commands::Fmt {
			path,
			stdin,
			remove_bold,
			compact_blank_lines,
			collapse_spaces,
			remove_horizontal_rules,
			remove_emphasis,
		}) => {
			let options = get_format_options(
				remove_bold,
				compact_blank_lines,
				collapse_spaces,
				remove_horizontal_rules,
				remove_emphasis,
			);
			if stdin {
				format::cmd_fmt_stdin(options)
			} else if let Some(p) = path {
				format::cmd_fmt(&p, cli.human, options)
			} else {
				eprintln!("Error: Either --stdin or a file path is required");
				std::process::exit(1);
			}
		}
		None => {
			// If path provided without command, treat as fmt
			if let Some(path) = cli.path {
				let options = get_format_options(true, true, true, true, true);
				format::cmd_fmt(&path, cli.human, options)
			} else {
				// If no command and not version, show help
				eprintln!("Usage: agent-md <COMMAND>");
				eprintln!("For more information, try '--help'.");
				std::process::exit(1);
			}
		}
	}
}
