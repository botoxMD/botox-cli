use serde_yaml::Value;

pub fn escape_typst_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '\\' => out.push_str(r"\\"),
            '"' => out.push_str(r#"\""#),
            '\n' => out.push_str(r"\n"),
            '\r' => {},
            _ => out.push(c),
        }
    }
    out
}

pub fn format_typst_date(d: &str) -> Option<String> {
    let trimmed = d.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed == r"\today" || trimmed.eq_ignore_ascii_case("today") {
        return Some("datetime.today()".to_string());
    }
    if trimmed.eq_ignore_ascii_case("auto") {
        return Some("auto".to_string());
    }
    if trimmed.eq_ignore_ascii_case("none") {
        return Some("none".to_string());
    }

    let date_part = trimmed.split(['T', ' ']).next().unwrap_or(trimmed);
    let parts: Vec<&str> = date_part.split(['-', '/']).collect();
    if parts.len() == 3 {
        if let (Ok(y), Ok(m), Ok(day)) = (parts[0].parse::<i32>(), parts[1].parse::<u8>(), parts[2].parse::<u8>())
            && (1..=12).contains(&m) && (1..=31).contains(&day) {
                return Some(format!("datetime(year: {y}, month: {m}, day: {day})"));
            }
    } else if parts.len() == 2 {
        if let (Ok(y), Ok(m)) = (parts[0].parse::<i32>(), parts[1].parse::<u8>())
            && (1..=12).contains(&m) {
                return Some(format!("datetime(year: {y}, month: {m}, day: 1)"));
            }
    } else if parts.len() == 1
        && let Ok(y) = parts[0].parse::<i32>()
        && (1000..=9999).contains(&y) {
            return Some(format!("datetime(year: {y}, month: 1, day: 1)"));
        }
    Some("auto".to_string())
}

pub fn extract_title(fm: &Value) -> Option<String> {
    fm.get("title").and_then(|v| {
        if let Some(s) = v.as_str() {
            Some(s.to_string())
        } else { v.as_i64().map(|n| n.to_string()) }
    })
}

pub fn extract_authors(fm: &Value, config_author: Option<&str>, cli_author: Option<&str>) -> Vec<String> {
    if let Some(ca) = cli_author {
        vec![ca.to_string()]
    } else if let Some(a) = fm.get("author").or_else(|| fm.get("authors")) {
        if let Some(s) = a.as_str() {
            vec![s.to_string()]
        } else if let Some(arr) = a.as_sequence() {
            arr.iter().filter_map(|item| {
                if let Some(s) = item.as_str() {
                    Some(s.to_string())
                } else { item.get("name").and_then(|v| v.as_str()).map(|name| name.to_string()) }
            }).collect()
        } else {
            Vec::new()
        }
    } else if let Some(ca) = config_author {
        vec![ca.to_string()]
    } else {
        Vec::new()
    }
}

pub fn extract_date(fm: &Value) -> Option<String> {
    fm.get("date").and_then(|v| {
        if let Some(s) = v.as_str() {
            Some(s.to_string())
        } else { v.as_i64().map(|n| n.to_string()) }
    })
}

pub fn build_document_metadata(
    title: Option<&str>,
    authors: &[String],
    date: Option<&str>,
) -> Option<String> {
    let mut fields = Vec::new();

    if let Some(t) = title {
        fields.push(format!("title: \"{}\"", escape_typst_string(t)));
    }

    if !authors.is_empty() {
        if authors.len() == 1 {
            fields.push(format!("author: \"{}\"", escape_typst_string(&authors[0])));
        } else {
            let author_strs = authors
                .iter()
                .map(|a| format!("\"{}\"", escape_typst_string(a)))
                .collect::<Vec<_>>()
                .join(", ");
            fields.push(format!("author: ({author_strs})"));
        }
    }

    if let Some(d) = date
        && let Some(formatted_date) = format_typst_date(d) {
            fields.push(format!("date: {formatted_date}"));
        }

    if fields.is_empty() {
        None
    } else {
        Some(format!("#set document({})\n", fields.join(", ")))
    }
}
