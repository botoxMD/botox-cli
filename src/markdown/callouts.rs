pub fn parse_callout_header(rest: &str) -> (String, String) {
    let clean = rest.trim_matches(|c| c == '{' || c == '}').trim();
    let mut kind = "note".to_string();
    let mut title = String::new();

    let chars: Vec<char> = clean.chars().collect();
    let mut i = 0;
    let n = chars.len();

    while i < n {
        while i < n && chars[i].is_whitespace() {
            i += 1;
        }
        if i >= n {
            break;
        }

        let remaining: String = chars[i..].iter().collect();
        if remaining.starts_with("title=") {
            i += 6;
            if i < n && (chars[i] == '"' || chars[i] == '\'') {
                let quote = chars[i];
                i += 1;
                let start = i;
                while i < n && chars[i] != quote {
                    i += 1;
                }
                title = chars[start..i].iter().collect();
                if i < n && chars[i] == quote {
                    i += 1;
                }
            } else {
                let start = i;
                while i < n && !chars[i].is_whitespace() && chars[i] != '}' {
                    i += 1;
                }
                title = chars[start..i].iter().collect();
            }
            continue;
        }

        let mut j = i;
        while j < n && !chars[j].is_whitespace() && chars[j] != '=' {
            j += 1;
        }
        if j < n && chars[j] == '=' {
            i = j + 1;
            if i < n && (chars[i] == '"' || chars[i] == '\'') {
                let quote = chars[i];
                i += 1;
                while i < n && chars[i] != quote {
                    i += 1;
                }
                if i < n && chars[i] == quote {
                    i += 1;
                }
            } else {
                while i < n && !chars[i].is_whitespace() {
                    i += 1;
                }
            }
            continue;
        }

        let start = i;
        while i < n && !chars[i].is_whitespace() {
            i += 1;
        }
        let token: String = chars[start..i].iter().collect();
        let clean_token = token.trim_start_matches('.');
        match clean_token.to_lowercase().as_str() {
            "note" | "info" => kind = "note".to_string(),
            "warning" => kind = "warning".to_string(),
            "tip" => kind = "tip".to_string(),
            "important" => kind = "important".to_string(),
            "caution" => kind = "caution".to_string(),
            "danger" => kind = "danger".to_string(),
            _ => {}
        }
    }

    (kind, title)
}

pub fn parse_github_callout_header(s: &str) -> Option<(String, String)> {
    let trimmed = s.trim_start();
    if !trimmed.starts_with("[!") {
        return None;
    }
    let rest = &trimmed[2..];
    let end_bracket = rest.find(']')?;
    let kind_raw = &rest[..end_bracket];
    let kind = match kind_raw.to_ascii_lowercase().as_str() {
        "note" | "info" => "note",
        "tip" | "hint" => "tip",
        "important" => "important",
        "warning" => "warning",
        "caution" | "danger" => "caution",
        _ => return None,
    };
    let title = rest[end_bracket + 1..].trim();
    Some((kind.to_string(), title.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_callout_header_empty_title() {
        let (kind, title) = parse_callout_header(r#"{.note title=""}"#);
        assert_eq!(kind, "note");
        assert_eq!(title, "");

        let (kind, title) = parse_callout_header("{.warning title=''}");
        assert_eq!(kind, "warning");
        assert_eq!(title, "");

        let (kind, title) = parse_callout_header(r#"{.tip title="Custom Tip"}"#);
        assert_eq!(kind, "tip");
        assert_eq!(title, "Custom Tip");
    }

    #[test]
    fn test_parse_callout_header_with_keywords_in_title_and_subtitle() {
        let (kind, title) = parse_callout_header(r#"{.note title="A very important warning message"}"#);
        assert_eq!(kind, "note");
        assert_eq!(title, "A very important warning message");

        let (kind, title) = parse_callout_header(r#"{.warning subtitle="note" title="System Down"}"#);
        assert_eq!(kind, "warning");
        assert_eq!(title, "System Down");
    }
}
