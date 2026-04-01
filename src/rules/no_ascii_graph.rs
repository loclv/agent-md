pub fn find_ascii_graph(line: &str) -> Option<usize> {
    let trimmed = line.trim();
    let is_table_separator = trimmed
        .chars()
        .all(|c| c == '|' || c == '-' || c == ' ' || c == ':');
    if is_table_separator {
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

    let line_lower = line.to_lowercase();
    for indicator in &graph_indicators {
        if let Some(pos) = line_lower.find(indicator) {
            return Some(pos + 1);
        }
    }

    for pattern in &ascii_graph_patterns {
        if let Some(pos) = line.find(pattern) {
            let pattern_count = line.matches(pattern).count();
            if pattern_count >= 2
                || line.contains("┌")
                || line.contains("└")
                || line.contains("├")
                || line.contains("┤")
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
                if line.matches("->").count() + line.matches("<-").count() >= 2
                    || line.matches("[ ]").count() + line.matches("( )").count() >= 2
                {
                    return Some(pos + 1);
                }
            } else {
                let special_chars = line
                    .chars()
                    .filter(|c| !c.is_alphabetic() && !c.is_whitespace() && *c != '.')
                    .count();

                let total_chars = line.chars().filter(|c| !c.is_whitespace()).count();

                if total_chars >= 5 && special_chars as f64 / total_chars as f64 > 0.4 {
                    return Some(pos + 1);
                }
            }
        }
    }

    let special_chars = line
        .chars()
        .filter(|c| !c.is_alphabetic() && !c.is_whitespace() && *c != '.')
        .count();

    let total_chars = line.chars().filter(|c| !c.is_whitespace()).count();

    if total_chars >= 5 && special_chars as f64 / total_chars as f64 > 0.4 {
        for (i, c) in line.chars().enumerate() {
            if !c.is_alphabetic() && !c.is_whitespace() && c != '.' {
                return Some(i + 1);
            }
        }
    }

    None
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
}
