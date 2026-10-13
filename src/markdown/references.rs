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

pub fn format_references_block(
    custom_heading: Option<&str>,
    refs: &[(String, String)],
    at_start_of_slide: bool,
    is_slides: bool,
    lang: &str,
    biblio_title: Option<&str>,
    include_labels: bool,
) -> String {
    let default_heading = match lang {
        "fr" => "Références",
        "de" => "Literaturverzeichnis",
        "es" => "Referencias",
        "it" => "Riferimenti bibliografici",
        _ => "References",
    };
    let heading = custom_heading.unwrap_or_else(|| biblio_title.unwrap_or(default_heading));

    let (online_label, available_label) = match lang {
        "fr" => ("[En ligne]", "Disponible sur :"),
        "de" => ("[Online]", "Verfügbar unter:"),
        "es" => ("[En línea]", "Disponible en:"),
        "it" => ("[Online]", "Disponibile su:"),
        _ => ("[Online]", "Available:"),
    };

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

        if is_url_label {
            block.push_str(&format!("#block[\\[{idx}\\] {online_label}. {available_label} #link(\"{url}\").]{label_suffix}\n\n"));
        } else {
            block.push_str(&format!("#block[\\[{idx}\\] \"{label}\", {online_label}. {available_label} #link(\"{url}\").]{label_suffix}\n\n"));
        }
    }
    block
}
