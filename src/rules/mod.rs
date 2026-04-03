pub mod code_blocks;
pub mod heading_structure;
pub mod list_formatting;
pub mod no_ascii_graph;
pub mod no_bold;
pub mod simple_tables;
pub mod space_indentation;
pub mod useless_links;

pub use code_blocks::validate_code_blocks;
pub use heading_structure::{extract_heading_level, validate_heading_structure};
#[allow(unused_imports)]
pub use list_formatting::{
	detect_list_item, extract_number_from_marker, validate_list_formatting, ListType,
};
pub use no_ascii_graph::find_ascii_graph;
pub use no_bold::find_bold_text;
pub use simple_tables::{validate_table_syntax, validate_table_trailing_spaces, Severity};
pub use space_indentation::validate_space_indentation;
pub use useless_links::find_useless_link;
