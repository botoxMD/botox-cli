pub fn parse_callout_header(rest: &str) -> (String, String) {
    let clean = rest.trim_matches(|c| c == '{' || c == '}').trim();
    let mut kind = "note".to_string();
    let mut title = String::new();

    if let Some(t_idx) = clean.find("title=") {
        let after_t = &clean[t_idx + 6..];
        let q = after_t.chars().next().unwrap_or('"');
        if q == '"' || q == '\'' {
            if let Some(end_q) = after_t[1..].find(q) {
                title = after_t[1..=end_q].to_string();
            }
        } else {
            title = after_t.split_whitespace().next().unwrap_or("").to_string();
        }
    }

    for token in clean.split_whitespace() {
        if token.starts_with("title=") {
            continue;
        }
        let token = token.trim_start_matches('.');
        match token.to_lowercase().as_str() {
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
