pub fn find_ascii_graph(line: &str) -> Option<usize> {
	let trimmed = line.trim();
	let is_table_separator = trimmed
		.chars()
		.all(|c| c == '|' || c == '-' || c == ' ' || c == ':');
	if is_table_separator {
		return None;
	}

	// KaTeX / math blocks and standalone equations are not ASCII graphs
	if (trimmed.starts_with('$') && trimmed.ends_with('$') && trimmed.len() >= 2)
		|| trimmed.starts_with("$$")
	{
		return None;
	}

	let ascii_graph_patterns = [
		"┌─┐",
		"└─┘",
		"├─┤",
		"│ │",
		"├──",
		"└──",
		"│  ",
		"├── ",
		"└── ",
		"│   ",
		"->",
		"<-",
		"<->",
		"==",
		"=>",
		"<=",
		"[",
		"]",
		"flow:",
		"Flow:",
		"FLOW:",
		"diagram:",
		"Diagram:",
		"DIAGRAM:",
		"chart:",
		"Chart:",
		"CHART:",
		"graph:",
		"Graph:",
		"GRAPH:",
		"tree:",
		"Tree:",
		"TREE:",
		"+---+",
		"+---",
		"---+",
		"|   |",
	];

	let graph_indicators = ["graph:", "chart:", "diagram:", "flow:", "tree:"];

	let chars: Vec<char> = line.chars().collect();
	let mut exempt_ranges = Vec::new();
	let mut i = 0;
	while i < chars.len() {
		if chars[i] == '`' {
			if let Some(code_end) = crate::format::lines::find_code_span_end(&chars, i) {
				exempt_ranges.push((i, code_end));
				i = code_end + 1;
				continue;
			}
		}
		if chars[i] == '$' {
			if let Some(math_end) = crate::format::lines::find_math_span_end(&chars, i) {
				exempt_ranges.push((i, math_end));
				i = math_end + 1;
				continue;
			}
		}
		i += 1;
	}

	for indicator in &graph_indicators {
		// SIMD: Accelerate case-insensitive indicator detection across the line.
		// Avoids calling `line.to_lowercase()` (allocating a new String) for every
		// indicator when none are present in the line.
		if crate::simd::contains_ascii_case_insensitive(line, indicator) {
			let line_lower = line.to_lowercase();
			let mut search_pos = 0;
			while let Some(pos) = line_lower[search_pos..].find(indicator) {
				let abs_pos = search_pos + pos;
				if !exempt_ranges
					.iter()
					.any(|&(s, e)| abs_pos >= s && abs_pos <= e)
				{
					return Some(abs_pos + 1);
				}
				search_pos = abs_pos + indicator.len();
			}
		}
	}

	for pattern in &ascii_graph_patterns {
		let mut search_pos = 0;
		let mut matches_outside = Vec::new();
		while let Some(pos) = line[search_pos..].find(pattern) {
			let abs_pos = search_pos + pos;
			if !exempt_ranges
				.iter()
				.any(|&(s, e)| abs_pos >= s && abs_pos <= e)
			{
				matches_outside.push(abs_pos);
			}
			search_pos = abs_pos + pattern.len();
		}

		if matches_outside.is_empty() {
			continue;
		}

		let pos = matches_outside[0];
		let pattern_count = matches_outside.len();

		if pattern_count >= 2
			|| line.contains('┌')
			|| line.contains('└')
			|| line.contains('├')
			|| line.contains('┤')
		{
			return Some(pos + 1);
		}

		if *pattern == "├──"
			|| *pattern == "└──"
			|| *pattern == "│  "
			|| *pattern == "┌─┐"
			|| *pattern == "└─┘"
			|| *pattern == "├─┤"
		{
			return Some(pos + 1);
		} else if *pattern == "[ ]"
			|| *pattern == "( )"
			|| *pattern == "{ }"
			|| *pattern == "->"
			|| *pattern == "<-"
		{
			let arrows_count = count_matches_outside(line, "->", &exempt_ranges)
				+ count_matches_outside(line, "<-", &exempt_ranges);
			let brackets_count = count_matches_outside(line, "[ ]", &exempt_ranges)
				+ count_matches_outside(line, "( )", &exempt_ranges);
			if arrows_count >= 2 || brackets_count >= 2 {
				return Some(pos + 1);
			}
		} else {
			let (special_chars, total_chars) =
				count_special_and_total_chars(&chars, &exempt_ranges);
			if total_chars >= 5 && special_chars as f64 / total_chars as f64 > 0.4 {
				return Some(pos + 1);
			}
		}
	}

	let (special_chars, total_chars) = count_special_and_total_chars(&chars, &exempt_ranges);
	if total_chars >= 5 && special_chars as f64 / total_chars as f64 > 0.4 {
		for (i, &c) in chars.iter().enumerate() {
			if exempt_ranges.iter().any(|&(s, e)| i >= s && i <= e) {
				continue;
			}
			if !c.is_alphabetic() && !c.is_whitespace() && c != '.' {
				return Some(i + 1);
			}
		}
	}

	None
}

fn count_matches_outside(line: &str, pattern: &str, exempt_ranges: &[(usize, usize)]) -> usize {
	let mut count = 0;
	let mut search_pos = 0;
	while let Some(pos) = line[search_pos..].find(pattern) {
		let abs_pos = search_pos + pos;
		if !exempt_ranges
			.iter()
			.any(|&(s, e)| abs_pos >= s && abs_pos <= e)
		{
			count += 1;
		}
		search_pos = abs_pos + pattern.len();
	}
	count
}

fn count_special_and_total_chars(
	chars: &[char],
	exempt_ranges: &[(usize, usize)],
) -> (usize, usize) {
	let mut special_chars = 0;
	let mut total_chars = 0;
	for (idx, &c) in chars.iter().enumerate() {
		if exempt_ranges.iter().any(|&(s, e)| idx >= s && idx <= e) {
			continue;
		}
		if !c.is_whitespace() {
			total_chars += 1;
			if !c.is_alphabetic() && c != '.' {
				special_chars += 1;
			}
		}
	}
	(special_chars, total_chars)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_find_ascii_graph_tree_structure() {
		let line = "├── parent";
		let result = find_ascii_graph(line);
		assert!(result.is_some());
		assert_eq!(result.unwrap(), 1);
	}

	#[test]
	fn test_find_ascii_graph_flow_chart() {
		let line = "A -> B -> C";
		let result = find_ascii_graph(line);
		assert!(result.is_some());
		assert_eq!(result.unwrap(), 3); // Position of first ->
	}

	#[test]
	fn test_find_ascii_graph_box_drawing() {
		let line = "┌─┐";
		let result = find_ascii_graph(line);
		assert!(result.is_some());
		assert_eq!(result.unwrap(), 1);
	}

	#[test]
	fn test_find_ascii_graph_normal_text() {
		let line = "This is normal text";
		let result = find_ascii_graph(line);
		assert!(result.is_none());
	}

	#[test]
	fn test_find_ascii_graph_code_block() {
		let line = "```javascript";
		let result = find_ascii_graph(line);
		assert!(result.is_none()); // Code blocks should be exempt
	}

	#[test]
	fn test_find_ascii_graph_explicit_indicator() {
		let line = "graph: A -> B";
		let result = find_ascii_graph(line);
		assert!(result.is_some());
		assert_eq!(result.unwrap(), 1); // Position of 'g'
	}

	#[test]
	fn test_find_ascii_graph_high_density_special_chars() {
		let line = "+---+---+---+";
		let result = find_ascii_graph(line);
		assert!(result.is_some());
	}

	#[test]
	fn test_find_ascii_graph_updated_message() {
		let line = "flow: Process A -> Process B";
		let result = find_ascii_graph(line);
		assert!(result.is_some());
		assert_eq!(result.unwrap(), 1); // Position of 'f'
	}

	#[test]
	fn test_find_ascii_graph_folder_structure() {
		// Test common folder tree patterns that should be detected
		assert!(find_ascii_graph("├── public/").is_some());
		assert!(find_ascii_graph("│   ├── pagefind/").is_some());
		assert!(find_ascii_graph("└── images/").is_some());
		// Tree structure needs at least the tree prefix pattern to be detected
		assert!(find_ascii_graph("│   └── file").is_some());
	}

	#[test]
	fn test_find_ascii_graph_table_separator_ignored() {
		let line = "|---|---|---|";
		let result = find_ascii_graph(line);
		assert!(result.is_none()); // Table separators should be ignored
	}

	#[test]
	fn test_find_ascii_graph_with_colons() {
		let line = "|:---|:---:|---:|";
		let result = find_ascii_graph(line);
		assert!(result.is_none()); // Table alignment should be ignored
	}

	#[test]
	fn test_find_ascii_graph_case_insensitive() {
		assert!(find_ascii_graph("GRAPH: A -> B").is_some());
		assert!(find_ascii_graph("Flow: Process A").is_some());
		assert!(find_ascii_graph("DIAGRAM: [A]").is_some());
		assert!(find_ascii_graph("chart: Simple").is_some());
		assert!(find_ascii_graph("TREE: Root").is_some());
	}

	#[test]
	fn test_find_ascii_graph_multiple_arrows() {
		let line = "A -> B -> C -> D";
		let result = find_ascii_graph(line);
		assert!(result.is_some());
		assert_eq!(result.unwrap(), 3); // Position of first ->
	}

	#[test]
	fn test_find_ascii_graph_single_arrow() {
		let line = "A -> B";
		let result = find_ascii_graph(line);
		assert!(result.is_none()); // Single arrow without indicator shouldn't trigger
	}

	#[test]
	fn test_find_ascii_graph_bidirectional_arrow() {
		let line = "A <-> B";
		let result = find_ascii_graph(line);
		assert!(result.is_some());
		assert_eq!(result.unwrap(), 4); // Position of <->
	}

	#[test]
	fn test_find_ascii_graph_empty_line() {
		let line = "";
		let result = find_ascii_graph(line);
		assert!(result.is_none());
	}

	#[test]
	fn test_find_ascii_graph_only_whitespace() {
		let line = "   \t  ";
		let result = find_ascii_graph(line);
		assert!(result.is_none());
	}

	#[test]
	fn test_find_ascii_graph_mixed_content() {
		let line = "Normal text with -> arrow";
		let result = find_ascii_graph(line);
		assert!(result.is_none()); // Single arrow without indicator shouldn't trigger
	}

	#[test]
	fn test_find_ascii_graph_brackets_and_arrows() {
		let line = "[A] -> [B] -> [C]";
		let result = find_ascii_graph(line);
		assert!(result.is_some()); // Multiple arrows with brackets
		assert_eq!(result.unwrap(), 5); // Position of first ->
	}

	#[test]
	fn test_find_ascii_graph_high_special_char_density() {
		let line = "+-+=-+-+=-+";
		let result = find_ascii_graph(line);
		assert!(result.is_some()); // High density of special chars
	}

	#[test]
	fn test_find_ascii_graph_normal_special_chars() {
		let line = "Price: $5.99 (20% off)";
		let result = find_ascii_graph(line);
		// This line actually has high special char density, so it will trigger
		assert!(result.is_some());
	}

	#[test]
	fn test_find_ascii_graph_ignores_katex_inline_and_display() {
		assert!(find_ascii_graph("$y = ax + b$").is_none());
		assert!(find_ascii_graph("$$y = ax + b$$").is_none());
		assert!(find_ascii_graph("Formula: $f(x) = x^2 + 2x + 1$").is_none());
		assert!(find_ascii_graph(
			"Check: $f(x) = \\int_0^\\infty e^{-x^2} dx = \\frac{\\sqrt{\\pi}}{2}$"
		)
		.is_none());
	}
}
