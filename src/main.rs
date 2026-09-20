use clap::Parser;

use agent_md::cli::{get_format_options, Cli, Commands};
use agent_md::{commands, format};

fn main() {
	let cli = Cli::parse();

	if cli.version {
		println!("{}", env!("CARGO_PKG_VERSION"));
		return;
	}

	match cli.command {
		Some(Commands::Init { path, force }) => {
			let custom = path.as_deref().or(cli.config.as_deref());
			commands::cmd_init(custom, force, cli.human);
		}
		Some(Commands::Config {
			path,
			check,
			init,
			force,
		}) => {
			let custom = path.as_deref().or(cli.config.as_deref());
			commands::cmd_config(custom, check, init, force, cli.human);
		}
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
		Some(Commands::Lint { path, content }) => {
			commands::cmd_lint(&path, content, cli.human, cli.config.as_deref())
		}
		Some(Commands::LintFile { path }) => {
			commands::cmd_lint_file(&path, cli.human, cli.config.as_deref())
		}
		Some(Commands::Ignore { path }) => {
			commands::cmd_ignore(&path, cli.human);
		}
		Some(Commands::Fmt {
			path,
			stdin,
			remove_bold,
			compact_blank_lines,
			collapse_spaces,
			remove_horizontal_rules,
			remove_emphasis,
			minify_html,
		}) => {
			let options = get_format_options(
				remove_bold,
				compact_blank_lines,
				collapse_spaces,
				remove_horizontal_rules,
				remove_emphasis,
				minify_html,
				cli.config.as_deref(),
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
				let options =
					get_format_options(None, None, None, None, None, None, cli.config.as_deref());
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
