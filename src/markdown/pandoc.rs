
pub fn clean_command_title(mut rest: &str) -> Option<String> {
    rest = rest.trim();
    if rest.starts_with(':') {
        rest = rest[1..].trim();
    }
    if (rest.starts_with('"') && rest.ends_with('"'))
        || (rest.starts_with('\'') && rest.ends_with('\''))
        || (rest.starts_with('{') && rest.ends_with('}'))
    {
        rest = rest[1..rest.len() - 1].trim();
    }
    if rest.is_empty() {
        None
    } else {
        Some(rest.to_string())
    }
}

pub fn parse_toc_line(line: &str) -> Option<Option<String>> {
    let trimmed = line.trim();

    if trimmed.starts_with("<!--") && trimmed.ends_with("-->") && trimmed.len() >= 7 {
        let inner = trimmed[4..trimmed.len() - 3].trim();
        if inner == "toc" || inner == "tableofcontents" {
            return Some(None);
        }
        for prefix in &["toc", "tableofcontents"] {
            if let Some(rest) = inner.strip_prefix(prefix)
                && (rest.starts_with(' ') || rest.starts_with(':')) {
                    return Some(clean_command_title(rest));
                }
        }
    }

    if trimmed == r"\toc" {
        return Some(None);
    }
    if trimmed.starts_with(r"\toc ") || trimmed.starts_with(r"\toc:") || trimmed.starts_with(r"\toc{") {
        return Some(clean_command_title(&trimmed[4..]));
    }

    if trimmed == r"\tableofcontents" {
        return Some(None);
    }
    if trimmed.starts_with(r"\tableofcontents ") || trimmed.starts_with(r"\tableofcontents:") || trimmed.starts_with(r"\tableofcontents{") {
        return Some(clean_command_title(&trimmed[16..]));
    }

    None
}

pub fn parse_ref_line(line: &str) -> Option<Option<String>> {
    let trimmed = line.trim();

    if trimmed.starts_with("<!--") && trimmed.ends_with("-->") && trimmed.len() >= 7 {
        let inner = trimmed[4..trimmed.len() - 3].trim();
        if inner == "ref" || inner == "references" || inner == "bibliography" {
            return Some(None);
        }
        for prefix in &["ref", "references", "bibliography"] {
            if let Some(rest) = inner.strip_prefix(prefix)
                && (rest.starts_with(' ') || rest.starts_with(':')) {
                    return Some(clean_command_title(rest));
                }
        }
    }

    if trimmed == r"\ref" {
        return Some(None);
    }
    if trimmed.starts_with(r"\ref ") || trimmed.starts_with(r"\ref:") {
        return Some(clean_command_title(&trimmed[4..]));
    }
    if trimmed.starts_with(r"\ref{") {
        if !trimmed.contains(':') {
            return Some(clean_command_title(&trimmed[4..]));
        } else {
            return None;
        }
    }

    if trimmed == r"\references" {
        return Some(None);
    }
    if trimmed.starts_with(r"\references ") || trimmed.starts_with(r"\references:") || trimmed.starts_with(r"\references{") {
        return Some(clean_command_title(&trimmed[11..]));
    }

    if trimmed == r"\bibliography" {
        return Some(None);
    }
    if trimmed.starts_with(r"\bibliography ") || trimmed.starts_with(r"\bibliography:") || trimmed.starts_with(r"\bibliography{") {
        return Some(clean_command_title(&trimmed[13..]));
    }

    None
}

pub fn normalize_dimension(val: &str) -> Option<String> {
    let s = val.trim().trim_matches(|c| c == '"' || c == '\'');
    if s.is_empty() {
        return None;
    }

    if s == r"\linewidth" || s == r"\textwidth" || s == "\\linewidth" || s == "\\textwidth" {
        return Some("100%".to_string());
    }

    if let Some(rest) = s.strip_suffix(r"\linewidth").or_else(|| s.strip_suffix(r"\textwidth"))
        && let Ok(factor) = rest.trim().parse::<f64>() {
            return Some(format!("{}%", (factor * 100.0).round() as i64));
        }

    if let Some(px_str) = s.strip_suffix("px")
        && let Ok(px) = px_str.trim().parse::<f64>() {
            return Some(format!("{}pt", (px * 0.75).round() as i64));
        }

    if s.ends_with('%')
        || s.ends_with("cm")
        || s.ends_with("mm")
        || s.ends_with("in")
        || s.ends_with("pt")
        || s.ends_with("em")
    {
        return Some(s.to_string());
    }

    if let Ok(n) = s.parse::<f64>() {
        if n <= 1.0 && n > 0.0 {
            return Some(format!("{}%", (n * 100.0).round() as i64));
        } else {
            return Some(format!("{s}pt"));
        }
    }

    Some(s.to_string())
}

pub fn parse_image_attributes(s: &str) -> (Option<String>, Option<String>, Option<String>) {
    let mut width = None;
    let mut height = None;
    let mut id = None;

    for token in s.split_whitespace() {
        let token = token.trim_matches(|c| c == ',' || c == '{' || c == '}');
        if token.starts_with('#') || token.starts_with(r"\#") {
            let raw_id = token.strip_prefix(r"\#").or_else(|| token.strip_prefix('#')).unwrap_or(token);
            let clean_id = raw_id.replace(':', "-");
            if !clean_id.is_empty() {
                id = Some(clean_id);
            }
        } else if let Some(stripped) = token.strip_prefix("id=") {
            let val = stripped.trim_matches('"').trim_matches('\'').replace(':', "-");
            if !val.is_empty() {
                id = Some(val);
            }
        } else if let Some(stripped) = token.strip_prefix("width=") {
            width = normalize_dimension(stripped);
        } else if let Some(stripped) = token.strip_prefix("height=") {
            height = normalize_dimension(stripped);
        }
    }

    (width, height, id)
}

pub fn parse_heading_attributes(s: &str) -> (String, bool, Option<String>) {
    let trimmed = s.trim();
    if let Some(brace_start) = trimmed.rfind('{')
        && trimmed.ends_with('}') && brace_start > 0 {
            let inside = trimmed[brace_start + 1..trimmed.len() - 1].trim();
            let mut is_unnumbered = false;
            let mut id = None;

            for token in inside.split_whitespace() {
                if token == "-" || token == ".unnumbered" || token == "unnumbered" {
                    is_unnumbered = true;
                } else if token.starts_with('#') || token.starts_with(r"\#") {
                    let raw_id = token.strip_prefix(r"\#").or_else(|| token.strip_prefix('#')).unwrap_or(token);
                    let clean_id = raw_id.replace(':', "-");
                    if !clean_id.is_empty() {
                        id = Some(clean_id);
                    }
                } else if let Some(val) = token.strip_prefix("id=") {
                    let val = val.trim_matches('"').trim_matches('\'');
                    let clean_id = val.replace(':', "-");
                    if !clean_id.is_empty() {
                        id = Some(clean_id);
                    }
                }
            }

            let clean_title = trimmed[..brace_start].trim().to_string();
            return (clean_title, is_unnumbered, id);
        }

    (trimmed.to_string(), false, None)
}

pub fn convert_cross_references(s: &str) -> String {
    let mut out = s.to_string();
    for prefix in &["@fig:", "@tbl:", "@sec:", "@eq:", "@lst:"] {
        if out.contains(prefix) {
            let target = match *prefix {
                "@fig:" => "@fig-",
                "@tbl:" => "@tbl-",
                "@sec:" => "@sec-",
                "@eq:" => "@eq-",
                "@lst:" => "@lst-",
                _ => continue,
            };
            out = out.replace(prefix, target);
        }
    }
    out
}

pub fn preprocess_pandoc(markdown: &str) -> String {
    let mut result = String::with_capacity(markdown.len());
    let mut in_code_fence = false;
    let mut fence_char = '`';
    let mut fence_len = 0;
    let mut in_github_callout = false;

    for line in markdown.lines() {
        let trimmed = line.trim_start();
        if !in_code_fence {
            let backticks = trimmed.chars().take_while(|&c| c == '`').count();
            let tildes = trimmed.chars().take_while(|&c| c == '~').count();
            if backticks >= 3 {
                if in_github_callout {
                    result.push_str("\n<!--botox:callout:end-->\n\n");
                    in_github_callout = false;
                }
                in_code_fence = true;
                fence_char = '`';
                fence_len = backticks;
                result.push_str(line);
                result.push('\n');
                continue;
            } else if tildes >= 3 {
                if in_github_callout {
                    result.push_str("\n<!--botox:callout:end-->\n\n");
                    in_github_callout = false;
                }
                in_code_fence = true;
                fence_char = '~';
                fence_len = tildes;
                result.push_str(line);
                result.push('\n');
                continue;
            }
        } else {
            let count = trimmed.chars().take_while(|&c| c == fence_char).count();
            if count >= fence_len && trimmed[count..].trim().is_empty() {
                in_code_fence = false;
            }
            result.push_str(line);
            result.push('\n');
            continue;
        }

        if in_github_callout {
            if let Some(after_gt) = trimmed.strip_prefix('>') {
                let inner = after_gt.trim_start();
                if let Some((next_kind, next_title)) = super::callouts::parse_github_callout_header(inner) {
                    result.push_str("\n<!--botox:callout:end-->\n\n");
                    result.push_str(&format!("<!--botox:callout:start:{next_kind}:{next_title}-->\n"));
                    continue;
                }
                let content = after_gt.strip_prefix(' ').unwrap_or(after_gt);
                result.push_str(content);
                result.push('\n');
                continue;
            } else {
                result.push_str("\n<!--botox:callout:end-->\n\n");
                in_github_callout = false;
                // fall through to process `line`
            }
        }

        if let Some(after_gt) = trimmed.strip_prefix('>') {
            let inner = after_gt.trim_start();
            if let Some((kind, title)) = super::callouts::parse_github_callout_header(inner) {
                in_github_callout = true;
                result.push_str(&format!("\n<!--botox:callout:start:{kind}:{title}-->\n"));
                continue;
            }
        }

        let line_trimmed = trimmed.trim_end();
        if line_trimmed == r"\pause" || line_trimmed == "<!-- pause -->" || line_trimmed == "<!--pause-->" || line_trimmed == "::: pause" {
            result.push_str("\n<!--botox:pause-->\n\n");
            continue;
        }
        if line.contains(r"\pause") {
            let line_mod = line.replace(r"\pause", "<!--botox:pause-->");
            result.push_str(&line_mod);
            result.push('\n');
            continue;
        }

        if let Some(toc_title_opt) = parse_toc_line(line_trimmed) {
            if in_github_callout {
                result.push_str("\n<!--botox:callout:end-->\n\n");
                in_github_callout = false;
            }
            if let Some(title) = toc_title_opt {
                result.push_str(&format!("\n<!--botox:toc:{title}-->\n\n"));
            } else {
                result.push_str("\n<!--botox:toc-->\n\n");
            }
            continue;
        }

        if let Some(ref_title_opt) = parse_ref_line(line_trimmed) {
            if in_github_callout {
                result.push_str("\n<!--botox:callout:end-->\n\n");
                in_github_callout = false;
            }
            if let Some(title) = ref_title_opt {
                result.push_str(&format!("\n<!--botox:ref:{title}-->\n\n"));
            } else {
                result.push_str("\n<!--botox:ref-->\n\n");
            }
            continue;
        }

        if trimmed.starts_with(":::") {
            let rest = trimmed.trim_start_matches(':').trim();
            if rest.is_empty() {
                result.push_str("\n<!--botox:callout:end-->\n\n");
            } else {
                let (kind, title) = super::callouts::parse_callout_header(rest);
                result.push_str(&format!("\n<!--botox:callout:start:{kind}:{title}-->\n"));
            }
            continue;
        }

        result.push_str(line);
        result.push('\n');
    }

    if in_github_callout {
        result.push_str("\n<!--botox:callout:end-->\n\n");
    }

    result
}
