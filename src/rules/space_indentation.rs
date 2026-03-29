pub fn validate_space_indentation(line: &str) -> Option<usize> {
    let trimmed = line.trim_end();
    if trimmed.is_empty() {
        return None;
    }

    if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
        return None;
    }

    let line_without_leading_spaces = line.trim_start();
    if line_without_leading_spaces.len() >= 3 {
        let mut i = 0;
        let mut has_digits = false;
        while i < line_without_leading_spaces.len()
            && line_without_leading_spaces
                .chars()
                .nth(i)
                .unwrap()
                .is_ascii_digit()
        {
            has_digits = true;
            i += 1;
        }
        if has_digits && i < line_without_leading_spaces.len() {
            let separator = line_without_leading_spaces.chars().nth(i).unwrap();
            if (separator == '.' || separator == ')') && i + 1 < line_without_leading_spaces.len() {
                let next_char = line_without_leading_spaces.chars().nth(i + 1).unwrap();
                if next_char == ' ' {
                    return None;
                }
            }
        }
    }

    if trimmed.starts_with('#') {
        return None;
    }

    if trimmed.starts_with('>') {
        return None;
    }

    let leading_spaces = line.len() - line.trim_start().len();

    if leading_spaces > 2 && line.starts_with("   ") {
        let has_leading_spaces = line.chars().take(leading_spaces).all(|c| c == ' ');
        if has_leading_spaces {
            return Some(1);
        }
    }

    None
}
