pub fn extract_host(url: &str) -> Option<&str> {
    let without_scheme = if let Some(rest) = url.strip_prefix("https://") {
        rest
    } else if let Some(rest) = url.strip_prefix("http://") {
        rest
    } else if let Some(rest) = url.strip_prefix("ftp://") {
        rest
    } else {
        url
    };
    let host_and_port = without_scheme.split(&['/', '?', '#'][..]).next().unwrap_or("");
    let host = host_and_port.split(':').next().unwrap_or("");
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
        if pi < p_chars.len() && (p_chars[pi] == '?' || p_chars[pi] == t_chars[ti]) {
            pi += 1;
            ti += 1;
        } else if pi < p_chars.len() && p_chars[pi] == '*' {
            star_pi = Some(pi);
            star_ti = ti;
            pi += 1;
        } else if let Some(sp) = star_pi {
            pi = sp + 1;
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

    let url_lower = url.to_lowercase();
    let pat_lower = pat.to_lowercase();

    // 1. Exact match on URL
    if url_lower == pat_lower {
        return true;
    }

    // 2. Glob pattern with '*'
    if pat_lower.contains('*') {
        if glob_match(&pat_lower, &url_lower) {
            return true;
        }
        if let Some(host) = extract_host(&url_lower)
            && glob_match(&pat_lower, host) {
                return true;
            }
        return false;
    }

    // 3. Exact host match or domain suffix
    if let Some(host) = extract_host(&url_lower) {
        if host == pat_lower {
            return true;
        }
        if let Some(stripped) = host.strip_suffix(&pat_lower)
            && stripped.ends_with('.') {
                return true;
            }
    }

    // 4. Fallback: URL prefix
    if url_lower.starts_with(&pat_lower) {
        return true;
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
