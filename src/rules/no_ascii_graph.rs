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
