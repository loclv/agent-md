//! `agent-md`: A fast Markdown editor, linter, and formatter designed for AI agents.
//!
//! This crate provides programmatic APIs for parsing, linting, formatting,
//! and editing Markdown documents with AI-friendly and token-efficient standards.
//!
//! # Usage
//!
//! ```rust
//! use agent_md::{format_markdown, validate_markdown};
//!
//! let content = "# Title\n\n**bold** text";
//! let formatted = format_markdown(content);
//! let result = validate_markdown(&formatted);
//! assert!(result.valid);
//! ```

pub mod cli;
pub mod commands;
pub mod config;
pub mod format;
#[cfg(test)]
mod html_tests;
pub mod ignore;
pub mod linter;
pub mod parser;
pub mod rules;
#[cfg(test)]
mod tests;
pub mod types;

pub use cli::{Cli, Commands};
pub use commands::parse_markdown;
pub use format::{format_markdown, format_markdown_with_options, FormatOptions};
pub use ignore::{
	get_git_ignore, get_ignore_list, get_ignore_list_in_dir, get_markdownlint_ignore, is_ignored,
	merge_ignore_lists, read_ignore_file,
};
pub use linter::{
	validate_markdown, validate_markdown_with_config, validate_markdown_with_custom_config,
};
pub use types::{Document, EditResult, LintError, LintResult};
