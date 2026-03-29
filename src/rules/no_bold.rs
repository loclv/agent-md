pub fn find_bold_text(line: &str) -> Vec<usize> {
    let mut results = Vec::new();

    let mut code_ranges = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let mut in_code = false;
    let mut code_start = 0;

    for (i, &ch) in chars.iter().enumerate() {
        if ch == '`' && (i == 0 || chars[i - 1] != '\\') {
            if !in_code {
                in_code = true;
                code_start = i;
            } else {
                in_code = false;
                code_ranges.push((code_start, i));
            }
        }
    }

    if in_code {
        code_ranges.push((code_start, line.len() - 1));
    }

    let mut search_start = 0;
    while search_start < line.len() {
        if let Some(start) = line[search_start..].find("**") {
            let abs_start = search_start + start;

            let in_code_range = code_ranges
                .iter()
                .any(|&(start, end)| abs_start >= start && abs_start <= end);

            if !in_code_range {
                if let Some(end_offset) = line[abs_start + 2..].find("**") {
                    let abs_end = abs_start + 2 + end_offset;

                    let end_in_code_range = code_ranges
                        .iter()
                        .any(|&(start, end)| abs_end >= start && abs_end <= end);

                    if !end_in_code_range {
                        results.push(abs_start + 1);
                        search_start = abs_end + 2;
                        continue;
                    }
                }
            }
            search_start = abs_start + 2;
        } else {
            break;
        }
    }

    search_start = 0;
    while search_start < line.len() {
        if let Some(start) = line[search_start..].find("__") {
            let abs_start = search_start + start;

            let in_code_range = code_ranges
                .iter()
                .any(|&(start, end)| abs_start >= start && abs_start <= end);

            if !in_code_range {
                if let Some(end_offset) = line[abs_start + 2..].find("__") {
                    let abs_end = abs_start + 2 + end_offset;

                    let end_in_code_range = code_ranges
                        .iter()
                        .any(|&(start, end)| abs_end >= start && abs_end <= end);

                    if !end_in_code_range {
                        results.push(abs_start + 1);
                        search_start = abs_end + 2;
                        continue;
                    }
                }
            }
            search_start = abs_start + 2;
        } else {
            break;
        }
    }

    results
}
