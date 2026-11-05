pub fn extract_host(url: &str) -> Option<&str> {
    let without_scheme = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .or_else(|| url.strip_prefix("ftp://"))
        .unwrap_or(url);

    let host = without_scheme
        .split('/')
        .next()?
        .split(':')
        .next()?;

    if host.is_empty() {
        None
    } else {
        Some(host)
    }
}

pub fn glob_match(pattern: &str, text: &str) -> bool {
    let p_chars: Vec<char> = pattern.chars().collect();
    let t_chars: Vec<char> = text.chars().collect();
    let (mut pi, mut ti) = (0, 0);
    let (mut star_pi, mut star_ti) = (None, 0);

    while ti < t_chars.len() {
        if pi < p_chars.len() && (p_chars[pi] == '?' || p_chars[pi].eq_ignore_ascii_case(&t_chars[ti])) {
            pi += 1;
            ti += 1;
        } else if pi < p_chars.len() && p_chars[pi] == '*' {
            star_pi = Some(pi);
            pi += 1;
            star_ti = ti;
        } else if let Some(spi) = star_pi {
            pi = spi + 1;
            star_ti += 1;
            ti = star_ti;
        } else {
            return false;
        }
    }

    while pi < p_chars.len() && p_chars[pi] == '*' {
        pi += 1;
    }

    pi == p_chars.len()
}

pub fn url_matches_pattern(url: &str, pattern: &str) -> bool {
    let pat = pattern.trim();
    if pat.is_empty() {
        return false;
    }

    let clean_url = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .or_else(|| url.strip_prefix("ftp://"))
        .unwrap_or(url);

    // 1. Wildcard glob match against full url and scheme-stripped url
    if pat.contains('*') || pat.contains('?') {
        if glob_match(pat, url) || glob_match(pat, clean_url) {
            return true;
        }
        // Also try matching against host
        if let Some(host) = extract_host(url)
            && glob_match(pat, host) {
                return true;
            }
        return false;
    }

    // 2. Clean scheme from pattern if present
    let clean_pat = pat
        .strip_prefix("https://")
        .or_else(|| pat.strip_prefix("http://"))
        .unwrap_or(pat)
        .trim_end_matches('/');

    // If pattern contains a path (e.g. "github.com/myorg" or "example.com/docs")
    if clean_pat.contains('/') {
        return clean_url.to_ascii_lowercase().starts_with(&clean_pat.to_ascii_lowercase())
            || url.to_ascii_lowercase().contains(&pat.to_ascii_lowercase());
    }

    // If pattern is a domain name (e.g. "github.com", "x.com", "wikipedia.org")
    if let Some(host) = extract_host(url) {
        let host_lower = host.to_ascii_lowercase();
        let pat_lower = clean_pat.to_ascii_lowercase();
        return host_lower == pat_lower || host_lower.ends_with(&format!(".{pat_lower}"));
    }

    false
}

pub fn get_default_references_heading(lang: &str) -> &'static str {
    match lang {
        "fr" => "Références",
        "de" => "Literaturverzeichnis",
        "es" => "Referencias",
        "it" => "Riferimenti bibliografici",
        _ => "References",
    }
}

pub fn get_reference_labels(lang: &str) -> (&'static str, &'static str) {
    match lang {
        "fr" => ("[En ligne]", "Disponible sur :"),
        "de" => ("[Online]", "Verfügbar unter:"),
        "es" => ("[En línea]", "Disponible en:"),
        "it" => ("[Online]", "Disponibile su:"),
        _ => ("[Online]", "Available:"),
    }
}

pub fn format_references_block(
    custom_heading: Option<&str>,
    refs: &[(String, String)],
    at_start_of_slide: bool,
    is_slides: bool,
    lang: &str,
    biblio_title: Option<&str>,
    include_labels: bool,
) -> String {
    let default_heading = get_default_references_heading(lang);
    let heading = custom_heading.unwrap_or_else(|| biblio_title.unwrap_or(default_heading));
    let (online_label, available_label) = get_reference_labels(lang);

    let mut block = String::new();
    if is_slides {
        if !at_start_of_slide {
            block.push_str("\n\n#pagebreak()\n\n");
        }
    } else {
        block.push_str("\n\n#v(2em)\n");
    }
    
    if include_labels {
        block.push_str(&format!("#heading(numbering: none)[{heading}] <references>\n\n"));
    } else {
        block.push_str(&format!("#heading(numbering: none)[{heading}]\n\n"));
    }
    block.push_str("#set par(hanging-indent: 1.8em, justify: false)\n\n");

    for (i, (url, label)) in refs.iter().enumerate() {
        let idx = i + 1;

        let is_url_label = label.is_empty()
            || label == url
            || label.starts_with("http://")
            || label.starts_with("https://");

        let label_suffix = if include_labels {
            format!(" <bib-{idx}>")
        } else {
            String::new()
        };

        let safe_url = url.replace('\\', "/").replace('"', "\\\"");
        if is_url_label {
            block.push_str(&format!("#block[\\[{idx}\\] {online_label}. {available_label} #link(\"{safe_url}\").]{label_suffix}\n\n"));
        } else {
            let safe_label = crate::markdown::escapes::escape_typst_text(label).replace('"', "\\\"");
            block.push_str(&format!("#block[\\[{idx}\\] \"{safe_label}\", {online_label}. {available_label} #link(\"{safe_url}\").]{label_suffix}\n\n"));
        }
    }
    block
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_host() {
        assert_eq!(extract_host("https://github.com/rust-lang/rust"), Some("github.com"));
        assert_eq!(extract_host("http://localhost:8080/api/v1"), Some("localhost"));
        assert_eq!(extract_host("ftp://ftp.gnu.org/gnu"), Some("ftp.gnu.org"));
        assert_eq!(extract_host("crates.io/crates/typst"), Some("crates.io"));
        assert_eq!(extract_host(""), None);
    }

    #[test]
    fn test_glob_match() {
        assert!(glob_match("abc", "abc"));
        assert!(!glob_match("abc", "abcd"));
        assert!(glob_match("a?c", "abc"));
        assert!(glob_match("a?c", "axc"));
        assert!(!glob_match("a?c", "abbc"));

        assert!(glob_match("*.rs", "main.rs"));
        assert!(glob_match("test_*", "test_file.rs"));
        assert!(glob_match("*github.com*", "https://github.com/botoxMD"));
        assert!(glob_match("*.org", "WIKIPEDIA.ORG"));
    }

    #[test]
    fn test_url_matches_pattern_cases() {
        assert!(url_matches_pattern("https://x.com/username", "x.com"));
        assert!(url_matches_pattern("https://sub.domain.com/path", "domain.com"));
        assert!(url_matches_pattern("https://github.com/my-org/my-repo", "github.com/my-org"));
        assert!(!url_matches_pattern("https://other.com/my-org", "github.com/my-org"));
        assert!(!url_matches_pattern("https://latex.com/guide", "x.com"));
        assert!(!url_matches_pattern("https://example.com", ""));
    }

    #[test]
    fn test_format_references_block_multilingual() {
        let refs = vec![
            ("https://rust-lang.org".to_string(), "Rust".to_string()),
            ("https://crates.io".to_string(), "https://crates.io".to_string()),
        ];

        // English document
        let doc_en = format_references_block(None, &refs, false, false, "en", None, true);
        assert!(doc_en.contains("#heading(numbering: none)[References] <references>"));
        assert!(doc_en.contains("#v(2em)"));
        assert!(doc_en.contains(r#"[1\] "Rust", [Online]. Available: #link("https://rust-lang.org").] <bib-1>"#));
        assert!(doc_en.contains(r#"[2\] [Online]. Available: #link("https://crates.io").] <bib-2>"#));

        // French document
        let doc_fr = format_references_block(None, &refs, false, false, "fr", None, false);
        assert!(doc_fr.contains("#heading(numbering: none)[Références]"));
        assert!(!doc_fr.contains("<references>"));
        assert!(doc_fr.contains("[En ligne]. Disponible sur :"));

        // German document
        let doc_de = format_references_block(None, &refs, false, false, "de", None, false);
        assert!(doc_de.contains("#heading(numbering: none)[Literaturverzeichnis]"));
        assert!(doc_de.contains("[Online]. Verfügbar unter:"));

        // Spanish document
        let doc_es = format_references_block(None, &refs, false, false, "es", None, false);
        assert!(doc_es.contains("#heading(numbering: none)[Referencias]"));
        assert!(doc_es.contains("[En línea]. Disponible en:"));

        // Italian document
        let doc_it = format_references_block(None, &refs, false, false, "it", None, false);
        assert!(doc_it.contains("#heading(numbering: none)[Riferimenti bibliografici]"));
        assert!(doc_it.contains("[Online]. Disponibile su:"));

        // Custom heading override
        let doc_custom = format_references_block(Some("Sources Cited"), &refs, false, false, "en", None, false);
        assert!(doc_custom.contains("#heading(numbering: none)[Sources Cited]"));

        // Slides mode
        let slides = format_references_block(None, &refs, false, true, "en", None, false);
        assert!(slides.contains("#pagebreak()"));
    }
}
