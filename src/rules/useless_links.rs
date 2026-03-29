pub fn find_useless_link(line: &str) -> Vec<usize> {
    let mut results = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '[' {
            let bracket_start = i;
            let mut bracket_end = i + 1;
            let mut bracket_content = String::new();
            let mut found_closing_bracket = false;

            while bracket_end < chars.len() {
                let ch = chars[bracket_end];
                if ch == ']' {
                    found_closing_bracket = true;
                    break;
                }
                bracket_content.push(ch);
                bracket_end += 1;
            }

            if found_closing_bracket
                && bracket_end + 1 < chars.len()
                && chars[bracket_end + 1] == '('
            {
                let mut paren_start = bracket_end + 2;
                let mut url = String::new();
                let mut found_closing_paren = false;
                let mut paren_depth = 1;

                while paren_start < chars.len() {
                    let ch = chars[paren_start];
                    if ch == '(' {
                        paren_depth += 1;
                        url.push(ch);
                    } else if ch == ')' {
                        paren_depth -= 1;
                        if paren_depth == 0 {
                            found_closing_paren = true;
                            break;
                        }
                        url.push(ch);
                    } else {
                        url.push(ch);
                    }
                    paren_start += 1;
                }

                if found_closing_paren {
                    let link_text = bracket_content.trim();
                    let url_trimmed = url.trim();

                    let url_without_protocol = url_trimmed
                        .trim_start_matches("http://")
                        .trim_start_matches("https://");

                    let url_without_slash = url_without_protocol.trim_end_matches('/');

                    let url_without_www = url_without_slash.trim_start_matches("www.");

                    if link_text == url_trimmed
                        || link_text == url_without_protocol
                        || link_text == url_without_slash
                        || link_text == url_without_www
                    {
                        results.push(bracket_start + 1);
                    }
                    i = paren_start + 1;
                    continue;
                }
            }
        }
        i += 1;
    }

    results
}
