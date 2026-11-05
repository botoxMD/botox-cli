
pub fn clean_command_title(mut rest: &str) -> Option<String> {
    rest = rest.trim();
    if rest.starts_with(':') {
        rest = rest[1..].trim();
    }
    if (rest.starts_with('"') && rest.ends_with('"'))
        || (rest.starts_with('\'') && rest.ends_with('\''))
        || (rest.starts_with('{') && rest.ends_with('}'))
    {
        if rest.len() >= 2 {
            rest = rest[1..rest.len() - 1].trim();
        } else {
            rest = "";
        }
    } else if (rest.starts_with('{') && !rest.contains('}'))
        || (rest.starts_with('"') && !rest[1..].contains('"'))
        || (rest.starts_with('\'') && !rest[1..].contains('\''))
    {
        rest = rest[1..].trim();
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
    if !s.contains('@') {
        return s.to_string();
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DivKind {
    Callout,
    Landscape,
    Portrait,
}

fn classify_div_header(rest: &str) -> (DivKind, String, String) {
    let clean = rest.trim_start_matches('{').trim_end_matches('}').trim();
    let lower = clean.to_ascii_lowercase();
    let lower_ident = lower.strip_prefix('.').unwrap_or(&lower).trim();

    if lower_ident == "landscape" || lower_ident == "horizontal" {
        (DivKind::Landscape, String::new(), String::new())
    } else if lower_ident == "portrait" || lower_ident == "vertical" {
        (DivKind::Portrait, String::new(), String::new())
    } else {
        let (kind, title) = super::callouts::parse_callout_header(rest);
        (DivKind::Callout, kind, title)
    }
}

pub fn preprocess_pandoc(markdown: &str) -> String {
    let mut result = String::with_capacity(markdown.len());
    let mut in_code_fence = false;
    let mut fence_char = '`';
    let mut fence_len = 0;
    let mut in_github_callout = false;
    let mut div_stack: Vec<DivKind> = Vec::new();
    let mut in_landscape_page = false;

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

        if line_trimmed.starts_with('\\') || line_trimmed.starts_with("<!--") {
            let line_trimmed_lower = line_trimmed.to_ascii_lowercase();
            let words: Vec<&str> = line_trimmed_lower.split_whitespace().collect();

            let is_newpage_landscape = (words.len() == 2
                && (words[0] == r"\newpage" || words[0] == r"\pagebreak" || words[0] == r"\clearpage")
                && (words[1] == "landscape" || words[1] == "horizontal"))
                || line_trimmed == r"\begin{landscape}"
                || line_trimmed == r"\begin{horizontal}"
                || line_trimmed == r"\landscape"
                || line_trimmed == r"\newpage{landscape}"
                || line_trimmed == r"\pagebreak{landscape}"
                || line_trimmed_lower == "<!-- landscape -->"
                || line_trimmed_lower == "<!--landscape-->"
                || line_trimmed_lower == "<!-- horizontal -->"
                || line_trimmed_lower == "<!-- horizontal:start -->"
                || line_trimmed_lower == "<!-- landscape:start -->"
                || line_trimmed_lower == "<!-- page: landscape -->"
                || line_trimmed_lower == "<!-- page:landscape -->"
                || line_trimmed_lower == "<!-- page: horizontal -->"
                || line_trimmed_lower == "<!-- page:horizontal -->";

            if is_newpage_landscape {
                if in_github_callout {
                    result.push_str("\n<!--botox:callout:end-->\n\n");
                    in_github_callout = false;
                }
                in_landscape_page = true;
                result.push_str("\n<!--botox:page:landscape:start-->\n\n");
                continue;
            }

            let is_newpage_portrait = (words.len() == 2
                && (words[0] == r"\newpage" || words[0] == r"\pagebreak" || words[0] == r"\clearpage")
                && (words[1] == "portrait" || words[1] == "vertical"))
                || line_trimmed == r"\begin{portrait}"
                || line_trimmed == r"\portrait"
                || line_trimmed == r"\newpage{portrait}"
                || line_trimmed == r"\pagebreak{portrait}"
                || line_trimmed == r"\end{landscape}"
                || line_trimmed == r"\end{horizontal}"
                || line_trimmed == r"\endlandscape"
                || line_trimmed == r"\end{portrait}"
                || line_trimmed == r"\endportrait"
                || line_trimmed_lower == "<!-- /landscape -->"
                || line_trimmed_lower == "<!--/landscape-->"
                || line_trimmed_lower == "<!-- /horizontal -->"
                || line_trimmed_lower == "<!-- /horizontal:end -->"
                || line_trimmed_lower == "<!-- landscape:end -->"
                || line_trimmed_lower == "<!-- endlandscape -->"
                || line_trimmed_lower == "<!-- portrait -->"
                || line_trimmed_lower == "<!--portrait-->"
                || line_trimmed_lower == "<!-- /portrait -->"
                || line_trimmed_lower == "<!--/portrait-->"
                || line_trimmed_lower == "<!-- page: portrait -->"
                || line_trimmed_lower == "<!-- page:portrait -->";

            if is_newpage_portrait {
                if in_github_callout {
                    result.push_str("\n<!--botox:callout:end-->\n\n");
                    in_github_callout = false;
                }
                in_landscape_page = false;
                result.push_str("\n<!--botox:page:landscape:end-->\n\n");
                continue;
            }

            if line_trimmed == r"\newpage"
                || line_trimmed == r"\pagebreak"
                || line_trimmed == r"\clearpage"
                || line_trimmed_lower == "<!-- newpage -->"
                || line_trimmed_lower == "<!--newpage-->"
                || line_trimmed_lower == "<!-- pagebreak -->"
                || line_trimmed_lower == "<!--pagebreak-->"
            {
                if in_github_callout {
                    result.push_str("\n<!--botox:callout:end-->\n\n");
                    in_github_callout = false;
                }
                if in_landscape_page {
                    in_landscape_page = false;
                    result.push_str("\n<!--botox:page:landscape:end-->\n\n");
                } else {
                    result.push_str("\n<!--botox:pagebreak-->\n\n");
                }
                continue;
            }
        }

        if trimmed.starts_with(":::") {
            let rest = trimmed.trim_start_matches(':').trim();
            if rest.is_empty() {
                match div_stack.pop().unwrap_or(DivKind::Callout) {
                    DivKind::Landscape => result.push_str("\n<!--botox:page:landscape:end-->\n\n"),
                    DivKind::Portrait => result.push_str("\n<!--botox:page:portrait:end-->\n\n"),
                    DivKind::Callout => result.push_str("\n<!--botox:callout:end-->\n\n"),
                }
            } else {
                let (kind, callout_kind, callout_title) = classify_div_header(rest);
                div_stack.push(kind);
                match kind {
                    DivKind::Landscape => result.push_str("\n<!--botox:page:landscape:start-->\n\n"),
                    DivKind::Portrait => result.push_str("\n<!--botox:page:portrait:start-->\n\n"),
                    DivKind::Callout => {
                        result.push_str(&format!("\n<!--botox:callout:start:{callout_kind}:{callout_title}-->\n"));
                    }
                }
            }
            continue;
        }

        if (trimmed.starts_with("Table:") || trimmed.starts_with(": ")) && trimmed.len() > 2 {
            if !result.is_empty() && !result.ends_with("\n\n") {
                if !result.ends_with('\n') {
                    result.push('\n');
                }
                result.push('\n');
            }
            result.push_str(line);
            result.push_str("\n\n");
            continue;
        }

        result.push_str(line);
        result.push('\n');
    }

    if in_github_callout {
        result.push_str("\n<!--botox:callout:end-->\n\n");
    }

    while let Some(top) = div_stack.pop() {
        match top {
            DivKind::Landscape => result.push_str("\n<!--botox:page:landscape:end-->\n\n"),
            DivKind::Portrait => result.push_str("\n<!--botox:page:portrait:end-->\n\n"),
            DivKind::Callout => result.push_str("\n<!--botox:callout:end-->\n\n"),
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_command_title() {
        assert_eq!(clean_command_title(""), None);
        assert_eq!(clean_command_title("   "), None);
        assert_eq!(clean_command_title(r#""""#), None);
        assert_eq!(clean_command_title("''"), None);
        assert_eq!(clean_command_title("{}"), None);
        assert_eq!(clean_command_title(":"), None);

        assert_eq!(clean_command_title("Table of Contents").as_deref(), Some("Table of Contents"));
        assert_eq!(clean_command_title(r#""Detailed Overview""#).as_deref(), Some("Detailed Overview"));
        assert_eq!(clean_command_title("'Quick Links'").as_deref(), Some("Quick Links"));
        assert_eq!(clean_command_title("{Chapter Index}").as_deref(), Some("Chapter Index"));
        assert_eq!(clean_command_title(r#": "Prefixed Title""#).as_deref(), Some("Prefixed Title"));
        assert_eq!(clean_command_title(": Simple Title").as_deref(), Some("Simple Title"));
    }

    #[test]
    fn test_parse_toc_line() {
        assert_eq!(parse_toc_line("<!-- toc -->"), Some(None));
        assert_eq!(parse_toc_line("<!-- tableofcontents -->"), Some(None));
        assert_eq!(parse_toc_line("<!-- toc: Quick Nav -->"), Some(Some("Quick Nav".to_string())));
        assert_eq!(parse_toc_line(r#"<!-- tableofcontents "Custom Index" -->"#), Some(Some("Custom Index".to_string())));

        assert_eq!(parse_toc_line(r"\toc"), Some(None));
        assert_eq!(parse_toc_line(r"\tableofcontents"), Some(None));
        assert_eq!(parse_toc_line(r"\toc Summary"), Some(Some("Summary".to_string())));
        assert_eq!(parse_toc_line(r"\toc: Summary"), Some(Some("Summary".to_string())));
        assert_eq!(parse_toc_line(r"\toc{Summary}"), Some(Some("Summary".to_string())));
        assert_eq!(parse_toc_line(r"\tableofcontents Detailed"), Some(Some("Detailed".to_string())));
        assert_eq!(parse_toc_line(r"\tableofcontents: Detailed"), Some(Some("Detailed".to_string())));
        assert_eq!(parse_toc_line(r"\tableofcontents{Detailed}"), Some(Some("Detailed".to_string())));

        assert_eq!(parse_toc_line("Just normal text"), None);
        assert_eq!(parse_toc_line("<!-- other comment -->"), None);
    }

    #[test]
    fn test_parse_ref_line() {
        assert_eq!(parse_ref_line("<!-- ref -->"), Some(None));
        assert_eq!(parse_ref_line("<!-- references -->"), Some(None));
        assert_eq!(parse_ref_line("<!-- bibliography -->"), Some(None));
        assert_eq!(parse_ref_line("<!-- ref: Works Cited -->"), Some(Some("Works Cited".to_string())));
        assert_eq!(parse_ref_line(r#"<!-- bibliography "Sources" -->"#), Some(Some("Sources".to_string())));

        assert_eq!(parse_ref_line(r"\ref"), Some(None));
        assert_eq!(parse_ref_line(r"\references"), Some(None));
        assert_eq!(parse_ref_line(r"\bibliography"), Some(None));
        assert_eq!(parse_ref_line(r"\ref Sources"), Some(Some("Sources".to_string())));
        assert_eq!(parse_ref_line(r"\ref: Sources"), Some(Some("Sources".to_string())));
        assert_eq!(parse_ref_line(r"\ref{Sources}"), Some(Some("Sources".to_string())));
        assert_eq!(parse_ref_line(r"\references Custom Ref"), Some(Some("Custom Ref".to_string())));
        assert_eq!(parse_ref_line(r"\bibliography{Custom Bib}"), Some(Some("Custom Bib".to_string())));

        // Cross-reference label should be ignored by parse_ref_line
        assert_eq!(parse_ref_line(r"\ref{sec:intro}"), None);
        assert_eq!(parse_ref_line("Regular line of text"), None);
    }

    #[test]
    fn test_normalize_dimension() {
        assert_eq!(normalize_dimension(""), None);
        assert_eq!(normalize_dimension(r"\linewidth").as_deref(), Some("100%"));
        assert_eq!(normalize_dimension(r"\textwidth").as_deref(), Some("100%"));
        assert_eq!(normalize_dimension(r"0.75\linewidth").as_deref(), Some("75%"));
        assert_eq!(normalize_dimension(r"0.5\textwidth").as_deref(), Some("50%"));
        assert_eq!(normalize_dimension("400px").as_deref(), Some("300pt"));
        assert_eq!(normalize_dimension("80%").as_deref(), Some("80%"));
        assert_eq!(normalize_dimension("15cm").as_deref(), Some("15cm"));
        assert_eq!(normalize_dimension("20mm").as_deref(), Some("20mm"));
        assert_eq!(normalize_dimension("3in").as_deref(), Some("3in"));
        assert_eq!(normalize_dimension("250pt").as_deref(), Some("250pt"));
        assert_eq!(normalize_dimension("2em").as_deref(), Some("2em"));
        assert_eq!(normalize_dimension("0.65").as_deref(), Some("65%"));
        assert_eq!(normalize_dimension("350").as_deref(), Some("350pt"));
    }

    #[test]
    fn test_parse_image_attributes() {
        let (w, h, id) = parse_image_attributes("{#fig:arch width=80% height=200px}");
        assert_eq!(w.as_deref(), Some("80%"));
        assert_eq!(h.as_deref(), Some("150pt"));
        assert_eq!(id.as_deref(), Some("fig-arch"));

        let (w2, h2, id2) = parse_image_attributes(r#"id="fig:diagram" width=0.5\linewidth"#);
        assert_eq!(w2.as_deref(), Some("50%"));
        assert_eq!(h2, None);
        assert_eq!(id2.as_deref(), Some("fig-diagram"));
    }

    #[test]
    fn test_parse_heading_attributes() {
        let (title, unnumbered, id) = parse_heading_attributes("Introduction {#sec:intro .unnumbered}");
        assert_eq!(title, "Introduction");
        assert!(unnumbered);
        assert_eq!(id.as_deref(), Some("sec-intro"));

        let (title2, unnumbered2, id2) = parse_heading_attributes("Methodology {-}");
        assert_eq!(title2, "Methodology");
        assert!(unnumbered2);
        assert_eq!(id2, None);

        let (title3, unnumbered3, id3) = parse_heading_attributes("Results {#sec:results}");
        assert_eq!(title3, "Results");
        assert!(!unnumbered3);
        assert_eq!(id3.as_deref(), Some("sec-results"));

        let (title4, unnumbered4, id4) = parse_heading_attributes("Normal Heading Without Attributes");
        assert_eq!(title4, "Normal Heading Without Attributes");
        assert!(!unnumbered4);
        assert_eq!(id4, None);
    }

    #[test]
    fn test_convert_cross_references() {
        assert_eq!(convert_cross_references("See @fig:1 and @tbl:2"), "See @fig-1 and @tbl-2");
        assert_eq!(convert_cross_references("In @sec:intro and @eq:euler and @lst:main"), "In @sec-intro and @eq-euler and @lst-main");
    }

    #[test]
    fn test_preprocess_pandoc_fenced_divs() {
        let md = "::: horizontal\nContent in landscape\n:::\n\n::: {.note title=\"Important\"}\nTake note\n:::";
        let res = preprocess_pandoc(md);
        assert!(res.contains("<!--botox:page:landscape:start-->"));
        assert!(res.contains("<!--botox:page:landscape:end-->"));
        assert!(res.contains("<!--botox:callout:start:note:Important-->"));
        assert!(res.contains("<!--botox:callout:end-->"));
    }

    #[test]
    fn test_preprocess_pandoc_latex_landscape() {
        let md = "\\begin{landscape}\nWide table\n\\end{landscape}";
        let res = preprocess_pandoc(md);
        assert!(res.contains("<!--botox:page:landscape:start-->"));
        assert!(res.contains("<!--botox:page:landscape:end-->"));
    }

    #[test]
    fn test_clean_command_title_malformed_no_panic() {
        assert_eq!(clean_command_title("\""), None);
        assert_eq!(clean_command_title("'"), None);
        assert_eq!(clean_command_title("{"), None);
        assert_eq!(clean_command_title("\"\""), None);
        assert_eq!(clean_command_title("\"Valid Title\""), Some("Valid Title".to_string()));
    }
}
