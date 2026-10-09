use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

fn find_matching_brace(s: &str, open_pos: usize) -> Option<usize> {
    let mut depth = 0;
    for (i, c) in s[open_pos..].char_indices() {
        if c == '{' {
            depth += 1;
        } else if c == '}' {
            depth -= 1;
            if depth == 0 {
                return Some(open_pos + i);
            }
        }
    }
    None
}


fn parse_callout_header(rest: &str) -> (String, String) {
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

fn parse_github_callout_header(s: &str) -> Option<(String, String)> {
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

fn clean_command_title(mut rest: &str) -> Option<String> {
    rest = rest.trim();
    if rest.starts_with(':') {
        rest = rest[1..].trim();
    }
    if (rest.starts_with('"') && rest.ends_with('"') && rest.len() >= 2)
        || (rest.starts_with('\'') && rest.ends_with('\'') && rest.len() >= 2)
        || (rest.starts_with('{') && rest.ends_with('}') && rest.len() >= 2)
    {
        rest = &rest[1..rest.len() - 1];
    }
    let trimmed = rest.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn parse_toc_line(line: &str) -> Option<Option<String>> {
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

fn parse_ref_line(line: &str) -> Option<Option<String>> {
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

fn preprocess_pandoc(markdown: &str) -> String {
    let mut result = String::with_capacity(markdown.len());
    let mut in_code_fence = false;
    let mut in_github_callout = false;

    for line in markdown.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            if in_github_callout {
                result.push_str("\n<!--botox:callout:end-->\n\n");
                in_github_callout = false;
            }
            in_code_fence = !in_code_fence;
            result.push_str(line);
            result.push('\n');
            continue;
        }

        if in_code_fence {
            result.push_str(line);
            result.push('\n');
            continue;
        }

        if in_github_callout {
            if let Some(after_gt) = trimmed.strip_prefix('>') {
                let inner = after_gt.trim_start();
                if let Some((next_kind, next_title)) = parse_github_callout_header(inner) {
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
            if let Some((kind, title)) = parse_github_callout_header(inner) {
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
                let (kind, title) = parse_callout_header(rest);
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

pub fn convert_latex_math_to_typst(math: &str) -> String {
    let mut s = math.trim().to_string();

    if matches!(s.as_str(), "dt" | "dx" | "dy" | "dz" | "dr" | "du" | "dv") {
        return format!("dif {}", &s[1..]);
    }

    // Text & Font styles
    while let Some(pos) = s.find(r"\mathcal{") {
        if let Some(end) = find_matching_brace(&s, pos + 8) {
            let inner = &s[pos + 9..end];
            let spaced = inner
                .chars()
                .filter(|c| !c.is_whitespace())
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            s.replace_range(pos..end + 1, &format!("cal({spaced})"));
        } else {
            break;
        }
    }
    while let Some(pos) = s.find(r"\mathbb{") {
        if let Some(end) = find_matching_brace(&s, pos + 7) {
            let inner = &s[pos + 8..end];
            let spaced = inner
                .chars()
                .filter(|c| !c.is_whitespace())
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            s.replace_range(pos..end + 1, &format!("bb({spaced})"));
        } else {
            break;
        }
    }
    while let Some(pos) = s.find(r"\mathbf{") {
        if let Some(end) = find_matching_brace(&s, pos + 7) {
            let inner = &s[pos + 8..end];
            let spaced = inner
                .chars()
                .filter(|c| !c.is_whitespace())
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            s.replace_range(pos..end + 1, &format!("bold({spaced})"));
        } else {
            break;
        }
    }
    while let Some(pos) = s.find(r"\mathrm{") {
        if let Some(end) = find_matching_brace(&s, pos + 7) {
            let inner = &s[pos + 8..end];
            s.replace_range(pos..end + 1, &format!("upright(\"{inner}\")"));
        } else {
            break;
        }
    }
    while let Some(pos) = s.find(r"\operatorname{") {
        if let Some(end) = find_matching_brace(&s, pos + 13) {
            let inner = &s[pos + 14..end];
            s.replace_range(pos..end + 1, &format!("op(\"{inner}\")"));
        } else {
            break;
        }
    }
    while let Some(pos) = s.find(r"\text{") {
        if let Some(end) = find_matching_brace(&s, pos + 5) {
            let inner = &s[pos + 6..end];
            s.replace_range(pos..end + 1, &format!("\"{inner}\""));
        } else {
            break;
        }
    }

    // Fractions: \frac{a}{b} -> (a) / (b)
    while let Some(pos) = s.find(r"\frac{") {
        if let Some(end1) = find_matching_brace(&s, pos + 5) {
            let num = s[pos + 6..end1].to_string();
            let rest = &s[end1 + 1..];
            if rest.starts_with('{')
                && let Some(end2) = find_matching_brace(&s, end1 + 1) {
                    let den = s[end1 + 2..end2].to_string();
                    let num_conv = convert_latex_math_to_typst(&num);
                    let den_conv = convert_latex_math_to_typst(&den);
                    s.replace_range(pos..end2 + 1, &format!("({num_conv}) / ({den_conv})"));
                    continue;
                }
        }
        break;
    }

    // Roots: \sqrt[n]{x} -> root(n, x) and \sqrt{x} -> sqrt(x)
    while let Some(pos) = s.find(r"\sqrt[") {
        if let Some(c1) = s[pos + 6..].find(']') {
            let n = s[pos + 6..pos + 6 + c1].to_string();
            let rest = &s[pos + 6 + c1 + 1..];
            if rest.starts_with('{')
                && let Some(end) = find_matching_brace(&s, pos + 6 + c1 + 1) {
                    let arg = s[pos + 6 + c1 + 2..end].to_string();
                    let arg_conv = convert_latex_math_to_typst(&arg);
                    s.replace_range(pos..end + 1, &format!("root({n}, {arg_conv})"));
                    continue;
                }
        }
        break;
    }
    while let Some(pos) = s.find(r"\sqrt{") {
        if let Some(end) = find_matching_brace(&s, pos + 5) {
            let inner = s[pos + 6..end].to_string();
            let inner_conv = convert_latex_math_to_typst(&inner);
            s.replace_range(pos..end + 1, &format!("sqrt({inner_conv})"));
        } else {
            break;
        }
    }

    // Accents & Decorations
    for (cmd, func) in [
        (r"\vec{", "arrow"),
        (r"\hat{", "hat"),
        (r"\bar{", "macron"),
        (r"\dot{", "dot"),
        (r"\ddot{", "ddot"),
        (r"\tilde{", "tilde"),
        (r"\overline{", "overline"),
        (r"\underline{", "underline"),
    ] {
        while let Some(pos) = s.find(cmd) {
            if let Some(end) = find_matching_brace(&s, pos + cmd.len() - 1) {
                let inner = s[pos + cmd.len()..end].to_string();
                let inner_conv = convert_latex_math_to_typst(&inner);
                let space_before = if pos > 0 && s[..pos].chars().last().is_some_and(|c| c.is_alphanumeric()) {
                    " "
                } else {
                    ""
                };
                s.replace_range(pos..end + 1, &format!("{space_before}{func}({inner_conv})"));
            } else {
                break;
            }
        }
    }

    // Matrix environments: pmatrix, bmatrix, vmatrix, matrix
    for (env, delim) in [
        ("pmatrix", Some("\"(\"")),
        ("bmatrix", Some("\"[\"")),
        ("vmatrix", Some("\"|\"")),
        ("matrix", None),
    ] {
        let begin_tag = format!(r"\begin{{{env}}}");
        let end_tag = format!(r"\end{{{env}}}");
        while let Some(start) = s.find(&begin_tag) {
            if let Some(end) = s[start..].find(&end_tag) {
                let inner = &s[start + begin_tag.len()..start + end];
                let formatted_inner = inner.replace(r"\\", ";").replace('&', ",");
                let delim_arg = if let Some(d) = delim {
                    format!("delim: {d}, ")
                } else {
                    String::new()
                };
                let replacement = format!("mat({delim_arg}{formatted_inner})");
                s.replace_range(start..start + end + end_tag.len(), &replacement);
            } else {
                break;
            }
        }
    }

    // Cases environment
    let begin_cases = r"\begin{cases}";
    let end_cases = r"\end{cases}";
    while let Some(start) = s.find(begin_cases) {
        if let Some(end) = s[start..].find(end_cases) {
            let inner = &s[start + begin_cases.len()..start + end];
            let formatted_inner = inner.replace(r"\\", ",");
            let replacement = format!("cases({formatted_inner})");
            s.replace_range(start..start + end + end_cases.len(), &replacement);
        } else {
            break;
        }
    }

    // Stripping scaling delimiters \left and \right
    s = s.replace(r"\left", "").replace(r"\right", "");

    // Operators and symbols
    let replacements = [
        (r"\sum", "sum"),
        (r"\prod", "product"),
        (r"\int", "integral"),
        (r"\infty", "oo"),
        (r"\ge", ">="),
        (r"\le", "<="),
        (r"\ne", "!="),
        (r"\neq", "!="),
        (r"\iff", "<=>"),
        (r"\Rightarrow", "=>"),
        (r"\to", "->"),
        (r"\rightarrow", "->"),
        (r"\leftarrow", "<-"),
        (r"\subset", "subset"),
        (r"\subseteq", "subset.eq"),
        (r"\in", "in"),
        (r"\notin", "in.not"),
        (r"\times", "times"),
        (r"\cdot", "dot"),
        (r"\parallel", "parallel"),
        (r"\sim", "tilde"),
        (r"\forall", "forall"),
        (r"\exists", "exists"),
        (r"\approx", "approx"),
        (r"\equiv", "equiv"),
        (r"\pm", "plus.minus"),
        (r"\mp", "minus.plus"),
        (r"\cap", "sect"),
        (r"\cup", "union"),
        (r"\setminus", "without"),
        (r"\partial", "partial"),
        (r"\nabla", "nabla"),
        (r"\perp", "perp"),
        (r"\quad", "space"),
        (r"\qquad", "space space"),
        (r"\,", "thin"),
        (r"\dots", "..."),
        (r"\cdots", "..."),
        (r"\ldots", "..."),
        (r"\vdots", "dots.v"),
        (r"\ddots", "dots.down"),
        (r"\lim", "lim"),
        (r"\det", "det"),
        (r"\max", "max"),
        (r"\min", "min"),
        (r"\sup", "sup"),
        (r"\inf", "inf"),
        (r"\log", "log"),
        (r"\ln", "ln"),
        (r"\exp", "exp"),
        (r"\sin", "sin"),
        (r"\cos", "cos"),
        (r"\tan", "tan"),
        (r"\sinh", "sinh"),
        (r"\cosh", "cosh"),
        (r"\tanh", "tanh"),
        (r"\, dx", " dif x"),
        (r"\, dy", " dif y"),
        (r"\, dt", " dif t"),
        (r"\, dz", " dif z"),
        (r"\, dr", " dif r"),
        (r"\, du", " dif u"),
        (r"\, dv", " dif v"),
        (r" dx", " dif x"),
        (r" dy", " dif y"),
        (r" dt", " dif t"),
        // Greek letters
        (r"\alpha", "alpha"),
        (r"\beta", "beta"),
        (r"\gamma", "gamma"),
        (r"\delta", "delta"),
        (r"\epsilon", "epsilon"),
        (r"\zeta", "zeta"),
        (r"\eta", "eta"),
        (r"\theta", "theta"),
        (r"\iota", "iota"),
        (r"\kappa", "kappa"),
        (r"\lambda", "lambda"),
        (r"\mu", "mu"),
        (r"\nu", "nu"),
        (r"\xi", "xi"),
        (r"\pi", "pi"),
        (r"\rho", "rho"),
        (r"\sigma", "sigma"),
        (r"\tau", "tau"),
        (r"\upsilon", "upsilon"),
        (r"\phi", "phi"),
        (r"\chi", "chi"),
        (r"\psi", "psi"),
        (r"\omega", "omega"),
        (r"\Gamma", "Gamma"),
        (r"\Delta", "Delta"),
        (r"\Theta", "Theta"),
        (r"\Lambda", "Lambda"),
        (r"\Xi", "Xi"),
        (r"\Pi", "Pi"),
        (r"\Sigma", "Sigma"),
        (r"\Phi", "Phi"),
        (r"\Psi", "Psi"),
        (r"\Omega", "Omega"),
    ];

    for (from, to) in replacements {
        s = s.replace(from, to);
    }

    // Convert grouping braces in subscripts and superscripts: _{...} -> _(...) and ^{...} -> ^(...)
    let format_sub_super_inner = |inner: &str| -> String {
        let trimmed = inner.trim();
        let is_known = matches!(
            trimmed,
            "oo" | "+oo" | "-oo" | "inf" | "sup" | "max" | "min" | "lim"
                | "alpha" | "beta" | "gamma" | "delta" | "epsilon" | "zeta" | "eta" | "theta"
                | "iota" | "kappa" | "lambda" | "mu" | "nu" | "xi" | "pi" | "rho" | "sigma"
                | "tau" | "upsilon" | "phi" | "chi" | "psi" | "omega"
                | "Gamma" | "Delta" | "Theta" | "Lambda" | "Xi" | "Pi" | "Sigma" | "Phi" | "Psi" | "Omega"
        );

        if is_known
            || inner.chars().all(|c| c.is_ascii_digit())
            || inner.contains(' ')
            || inner.contains(',')
            || inner.contains('+')
            || inner.contains('-')
            || inner.contains('=')
            || inner.contains('<')
            || inner.contains('>')
            || inner.contains('/')
            || inner.contains('*')
        {
            inner.to_string()
        } else {
            inner.chars().map(|c| c.to_string()).collect::<Vec<_>>().join(" ")
        }
    };

    while let Some(pos) = s.find("_{") {
        if let Some(end) = find_matching_brace(&s, pos + 1) {
            let inner = &s[pos + 2..end];
            let formatted_inner = format_sub_super_inner(inner);
            s.replace_range(pos..end + 1, &format!("_({formatted_inner})"));
        } else {
            break;
        }
    }
    while let Some(pos) = s.find("^{") {
        if let Some(end) = find_matching_brace(&s, pos + 1) {
            let inner = &s[pos + 2..end];
            let formatted_inner = format_sub_super_inner(inner);
            s.replace_range(pos..end + 1, &format!("^({formatted_inner})"));
        } else {
            break;
        }
    }

    s
}

fn normalize_dimension(val: &str) -> Option<String> {
    let s = val.trim();
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

fn parse_image_attributes(s: &str) -> (Option<String>, Option<String>, Option<String>) {
    let mut width = None;
    let mut height = None;
    let mut id = None;

    let clean = s.replace(',', " ");
    for token in clean.split_whitespace() {
        let token = token.trim();
        if token.starts_with('#') || token.starts_with(r"\#") {
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
        } else if let Some(val) = token.strip_prefix("width=") {
            let val = val.trim_matches('"').trim_matches('\'');
            if let Some(parsed) = normalize_dimension(val) {
                width = Some(parsed);
            }
        } else if let Some(val) = token.strip_prefix("height=") {
            let val = val.trim_matches('"').trim_matches('\'');
            if let Some(parsed) = normalize_dimension(val) {
                height = Some(parsed);
            }
        }
    }

    (width, height, id)
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiagramSpec {
    pub diagram_type: String,
    pub caption: Option<String>,
    pub width: Option<String>,
    pub height: Option<String>,
    pub id: Option<String>,
    pub file: Option<String>,
}

pub fn resolve_file_content(path_str: &str, resource_dir: Option<&std::path::Path>) -> Option<String> {
    let trimmed = path_str.trim().trim_matches(|c| c == '"' || c == '\'');
    if trimmed.is_empty() {
        return None;
    }
    let p = std::path::Path::new(trimmed);
    if p.is_file() {
        return std::fs::read_to_string(p).ok();
    }
    if let Some(res_dir) = resource_dir {
        let joined = res_dir.join(p);
        if joined.is_file() {
            return std::fs::read_to_string(joined).ok();
        }
    }
    None
}

pub fn parse_code_fence_file(fence: &str) -> (String, Option<String>) {
    let trimmed = fence.trim();
    let inside = if trimmed.starts_with('{') && trimmed.ends_with('}') {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    };
    let mut parts = inside.split_whitespace();
    let first = parts.next().unwrap_or("").trim_start_matches('.');
    let (lang, colon_file) = if let Some((l, f)) = first.split_once(':') {
        (l.to_string(), Some(f.to_string()))
    } else {
        (first.to_string(), None)
    };

    let mut file = colon_file;
    if file.is_none() {
        for attr in ["file=", "src=", "path="] {
            if let Some(pos) = inside.find(attr) {
                let rest = inside[pos + attr.len()..].trim_start();
                if let Some(stripped) = rest.strip_prefix('"') {
                    if let Some(end) = stripped.find('"') {
                        file = Some(stripped[..end].to_string());
                        break;
                    }
                } else if let Some(stripped) = rest.strip_prefix('\'') {
                    if let Some(end) = stripped.find('\'') {
                        file = Some(stripped[..end].to_string());
                        break;
                    }
                } else {
                    let token = rest.split_whitespace().next().unwrap_or("").trim_end_matches('}');
                    if !token.is_empty() {
                        file = Some(token.to_string());
                        break;
                    }
                }
            }
        }
    }
    (lang, file)
}

pub fn parse_diagram_fence(fence: &str) -> Option<DiagramSpec> {
    let trimmed = fence.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut lang_candidate = String::new();
    let mut attr_str = "";
    let mut pre_brace_attr = "";
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        let inside = trimmed[1..trimmed.len() - 1].trim();
        for token in inside.split_whitespace() {
            if token.starts_with('.') {
                lang_candidate = token.trim_start_matches('.').to_string();
                break;
            }
        }
        attr_str = inside;
    } else if let Some(brace_pos) = trimmed.find('{') {
        let before_brace = trimmed[..brace_pos].trim();
        let mut parts = before_brace.splitn(2, |c: char| c.is_whitespace());
        lang_candidate = parts.next().unwrap_or("").trim_start_matches('.').to_string();
        pre_brace_attr = parts.next().unwrap_or("").trim();

        let rest = &trimmed[brace_pos + 1..];
        if let Some(end_brace) = rest.find('}') {
            attr_str = &rest[..end_brace];
        } else {
            attr_str = rest;
        }
    } else {
        let mut parts = trimmed.splitn(2, |c: char| c.is_whitespace());
        lang_candidate = parts.next().unwrap_or("").trim_start_matches('.').to_string();
        if let Some(rest) = parts.next() {
            attr_str = rest;
        }
    }

    let (lang_only, colon_file) = if let Some((l, f)) = lang_candidate.split_once(':') {
        (l.to_string(), Some(f.to_string()))
    } else {
        (lang_candidate, None)
    };

    let norm_lang = lang_only
        .strip_prefix("diagram-")
        .unwrap_or(&lang_only)
        .to_lowercase();

    let diagram_type = match norm_lang.as_str() {
        "mermaid" | "mmd" => "mermaid".to_string(),
        "plantuml" | "puml" => "plantuml".to_string(),
        "umlet" | "uxf" => "umlet".to_string(),
        "graphviz" | "dot" => "graphviz".to_string(),
        "ditaa" => "ditaa".to_string(),
        "wavedrom" => "wavedrom".to_string(),
        "bytefield" => "bytefield".to_string(),
        "bpmn" => "bpmn".to_string(),
        _ => return None,
    };

    let mut caption = None;
    if let Some(cap_start) = attr_str.find("caption=") {
        let rest = attr_str[cap_start + 8..].trim_start();
        if let Some(stripped) = rest.strip_prefix('"') {
            if let Some(cap_end) = stripped.find('"') {
                caption = Some(stripped[..cap_end].to_string());
            }
        } else if let Some(stripped) = rest.strip_prefix('\'')
            && let Some(cap_end) = stripped.find('\'') {
                caption = Some(stripped[..cap_end].to_string());
            }
    } else if let Some(title_start) = attr_str.find("title=") {
        let rest = attr_str[title_start + 6..].trim_start();
        if let Some(stripped) = rest.strip_prefix('"') {
            if let Some(title_end) = stripped.find('"') {
                caption = Some(stripped[..title_end].to_string());
            }
        } else if let Some(stripped) = rest.strip_prefix('\'')
            && let Some(title_end) = stripped.find('\'') {
                caption = Some(stripped[..title_end].to_string());
            }
    }

    let (width, height, id) = parse_image_attributes(attr_str);

    let mut file = colon_file;
    if file.is_none() {
        let combined_attrs = if pre_brace_attr.is_empty() {
            attr_str.to_string()
        } else {
            format!("{pre_brace_attr} {attr_str}")
        };

        for attr in ["file=", "src=", "path="] {
            if let Some(pos) = combined_attrs.find(attr) {
                let rest = combined_attrs[pos + attr.len()..].trim_start();
                if let Some(stripped) = rest.strip_prefix('"') {
                    if let Some(end) = stripped.find('"') {
                        file = Some(stripped[..end].to_string());
                        break;
                    }
                } else if let Some(stripped) = rest.strip_prefix('\'') {
                    if let Some(end) = stripped.find('\'') {
                        file = Some(stripped[..end].to_string());
                        break;
                    }
                } else {
                    let token = rest.split_whitespace().next().unwrap_or("").trim_end_matches('}');
                    if !token.is_empty() {
                        file = Some(token.to_string());
                        break;
                    }
                }
            }
        }

        if file.is_none() {
            for search_str in [&pre_brace_attr, &attr_str] {
                let trimmed_search = search_str.trim().trim_matches(|c| c == '"' || c == '\'' || c == '{' || c == '}');
                for token in trimmed_search.split_whitespace() {
                    let clean = token.trim_matches(|c| c == '"' || c == '\'' || c == '{' || c == '}');
                    if clean.ends_with(".uxf")
                        || clean.ends_with(".umlet")
                        || clean.ends_with(".puml")
                        || clean.ends_with(".plantuml")
                        || clean.ends_with(".mmd")
                        || clean.ends_with(".mermaid")
                        || clean.ends_with(".dot")
                        || clean.ends_with(".gv")
                        || clean.ends_with(".ditaa")
                    {
                        file = Some(clean.to_string());
                        break;
                    }
                }
                if file.is_some() {
                    break;
                }
            }
        }
    }

    Some(DiagramSpec {
        diagram_type,
        caption,
        width,
        height,
        id,
        file,
    })
}

fn parse_heading_attributes(s: &str) -> (String, bool, Option<String>) {
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

fn parse_caption_and_label(s: &str) -> (String, Option<String>) {
    let rest = if let Some(stripped) = s.strip_prefix("Table:") {
        stripped.trim()
    } else if let Some(stripped) = s.strip_prefix(':') {
        stripped.trim()
    } else {
        s.trim()
    };
    let (caption, _, id) = parse_heading_attributes(rest);
    (caption, id)
}

fn convert_cross_references(s: &str) -> String {
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

pub fn escape_typst_text(s: &str) -> String {
    let mut s = s.to_string();
    let has_newpage = s.contains(r"\newpage") || s.contains(r"\pagebreak") || s.contains(r"\clearpage");
    let has_colbreak = s.contains(r"\columnbreak") || s.contains(r"\colbreak");
    let has_toc = s.contains(r"\tableofcontents");
    let has_lof = s.contains(r"\listoffigures");
    let has_lot = s.contains(r"\listoftables");
    let has_today = s.contains(r"\today");
    let has_pause = s.contains(r"\pause");

    if has_newpage {
        s = s.replace(r"\newpage", "BOTOXCMDNEWPAGEBOTOX")
             .replace(r"\pagebreak", "BOTOXCMDNEWPAGEBOTOX")
             .replace(r"\clearpage", "BOTOXCMDNEWPAGEBOTOX");
    }
    if has_colbreak {
        s = s.replace(r"\columnbreak", "BOTOXCMDCOLBREAKBOTOX")
             .replace(r"\colbreak", "BOTOXCMDCOLBREAKBOTOX");
    }
    if has_toc {
        s = s.replace(r"\tableofcontents", "BOTOXCMDTOCBOTOX");
    }
    if has_lof {
        s = s.replace(r"\listoffigures", "BOTOXCMDLOFBOTOX");
    }
    if has_lot {
        s = s.replace(r"\listoftables", "BOTOXCMDLOTBOTOX");
    }
    if has_today {
        s = s.replace(r"\today", "BOTOXCMDTODAYBOTOX");
    }
    if has_pause {
        s = s.replace(r"\pause", "BOTOXCMDPAUSEBOTOX");
    }

    s = convert_cross_references(&s);

    let chars: Vec<char> = s.chars().collect();
    let n = chars.len();
    let mut out = String::with_capacity(s.len() + 32);
    let mut i = 0;

    let mut cmd_bracket_depth = 0;

    while i < n {
        let c = chars[i];

        // Check for recognized Botox / Typst commands that we generated
        if c == '#' {
            let rest: String = chars[i..].iter().take(16).collect();
            if rest.starts_with("#sub[") {
                out.push_str("#sub[");
                i += 5;
                cmd_bracket_depth += 1;
                continue;
            } else if rest.starts_with("#super[") {
                out.push_str("#super[");
                i += 7;
                cmd_bracket_depth += 1;
                continue;
            } else if rest.starts_with("#strike[") {
                out.push_str("#strike[");
                i += 8;
                cmd_bracket_depth += 1;
                continue;
            } else if rest.starts_with("#pagebreak()") || rest.starts_with("#colbreak()") || rest.starts_with("#outline(") || rest.starts_with("#botox_callout(") || rest.starts_with("#heading(") || rest.starts_with("#figure(") {
                out.push('#');
                i += 1;
                continue;
            } else {
                out.push_str(r"\#");
                i += 1;
                continue;
            }
        }

        // Check for brackets
        if c == ']' {
            if cmd_bracket_depth > 0 {
                cmd_bracket_depth -= 1;
                out.push(']');
            } else {
                out.push_str(r"\]");
            }
            i += 1;
            continue;
        }

        if c == '[' {
            out.push_str(r"\[");
            i += 1;
            continue;
        }

        // Check for subscript: ~text~
        if c == '~' {
            let prev_is_tilde = i > 0 && chars[i - 1] == '~';
            let next_is_tilde = i + 1 < n && chars[i + 1] == '~';
            if !prev_is_tilde && !next_is_tilde {
                let mut j = i + 1;
                while j < n && chars[j] != '~' && chars[j] != ' ' && chars[j] != '\n' && chars[j] != '\t' {
                    j += 1;
                }
                if j < n && chars[j] == '~' {
                    let closing_next_is_tilde = j + 1 < n && chars[j + 1] == '~';
                    if !closing_next_is_tilde && j > i + 1 {
                        let sub: String = chars[i + 1..j].iter().collect();
                        let escaped_sub = escape_typst_text(&sub);
                        out.push_str("BOTOXSUBSTART");
                        out.push_str(&escaped_sub);
                        out.push_str("BOTOXSUBEND");
                        i = j + 1;
                        continue;
                    }
                }
            }
            out.push_str(r"\~");
            i += 1;
            continue;
        }

        // Check for superscript: ^text^
        if c == '^' {
            let prev_is_hat = i > 0 && chars[i - 1] == '^';
            let next_is_hat = i + 1 < n && chars[i + 1] == '^';
            if !prev_is_hat && !next_is_hat {
                let mut j = i + 1;
                while j < n && chars[j] != '^' && chars[j] != ' ' && chars[j] != '\n' && chars[j] != '\t' {
                    j += 1;
                }
                if j < n && chars[j] == '^' {
                    let closing_next_is_hat = j + 1 < n && chars[j + 1] == '^';
                    if !closing_next_is_hat && j > i + 1 {
                        let sup: String = chars[i + 1..j].iter().collect();
                        let escaped_sup = escape_typst_text(&sup);
                        out.push_str("BOTOXSUPERSTART");
                        out.push_str(&escaped_sup);
                        out.push_str("BOTOXSUPEREND");
                        i = j + 1;
                        continue;
                    }
                }
            }
            out.push('^');
            i += 1;
            continue;
        }

        // Check for cross references
        if c == '@' {
            let rest: String = chars[i..].iter().take(6).collect();
            if rest.starts_with("@fig-") || rest.starts_with("@tbl-") || rest.starts_with("@sec-") || rest.starts_with("@eq-") || rest.starts_with("@lst-") {
                out.push('@');
            } else {
                out.push_str(r"\@");
            }
            i += 1;
            continue;
        }

        // Check for comments (// or /*) or line-start description list marker (/ , /\n, or standalone /)
        if c == '/' {
            if (i + 1 < n && (chars[i + 1] == '/' || chars[i + 1] == '*'))
                || ((i == 0 || chars[i - 1] == '\n') && (i + 1 == n || chars[i + 1] == ' ' || chars[i + 1] == '\n'))
            {
                out.push_str(r"\/");
            } else {
                out.push('/');
            }
            i += 1;
            continue;
        }

        // Check for backslash
        if c == '\\' {
            out.push_str(r"\\");
            i += 1;
            continue;
        }

        // Check for list/heading start markers at line starts
        if (c == '=' || c == '-' || c == '+') && (i == 0 || chars[i - 1] == '\n') && i + 1 < n && chars[i + 1] == ' ' {
            out.push('\\');
            out.push(c);
            i += 1;
            continue;
        }

        match c {
            '*' => out.push_str(r"\*"),
            '_' => out.push_str(r"\_"),
            '$' => out.push_str(r"\$"),
            '<' => out.push_str(r"\<"),
            '>' => out.push_str(r"\>"),
            _ => out.push(c),
        }
        i += 1;
    }

    let mut final_out = out
        .replace("BOTOXSUBSTART", "#sub[")
        .replace("BOTOXSUBEND", "]")
        .replace("BOTOXSUPERSTART", "#super[")
        .replace("BOTOXSUPEREND", "]");

    if has_newpage {
        final_out = final_out.replace("BOTOXCMDNEWPAGEBOTOX", "\n#pagebreak()\n");
    }
    if has_colbreak {
        final_out = final_out.replace("BOTOXCMDCOLBREAKBOTOX", "\n#colbreak()\n");
    }
    if has_toc {
        final_out = final_out.replace("BOTOXCMDTOCBOTOX", "\n#outline(depth: 3)\n");
    }
    if has_lof {
        final_out = final_out.replace("BOTOXCMDLOFBOTOX", "\n#outline(target: figure.where(kind: image))\n");
    }
    if has_lot {
        final_out = final_out.replace("BOTOXCMDLOTBOTOX", "\n#outline(target: figure.where(kind: table))\n");
    }
    if has_today {
        final_out = final_out.replace("BOTOXCMDTODAYBOTOX", "#datetime.today().display(\"[day] [month repr:long] [year]\")");
    }
    if has_pause {
        final_out = final_out.replace("BOTOXCMDPAUSEBOTOX", "\n#botox_pause()\n");
    }

    final_out
}

#[derive(Debug, Clone, Default)]
pub struct MarkdownOptions<'a> {
    pub is_slides: bool,
    pub bibliography: bool,
    pub lang: &'a str,
    pub biblio_title: Option<&'a str>,
    pub toc_title: Option<&'a str>,
    pub toc_depth: Option<usize>,
    pub ref_exclude: Option<&'a [String]>,
    pub ref_include: Option<&'a [String]>,
    pub kroki_url: Option<&'a str>,
    pub resource_dir: Option<&'a std::path::Path>,
}

#[allow(clippy::too_many_arguments, dead_code)]
pub fn markdown_to_typst(
    markdown: &str,
    is_slides: bool,
    bibliography: bool,
    lang: &str,
    biblio_title: Option<&str>,
    toc_title: Option<&str>,
    toc_depth: Option<usize>,
    ref_exclude: Option<&[String]>,
    ref_include: Option<&[String]>,
) -> String {
    markdown_to_typst_with_options(
        markdown,
        &MarkdownOptions {
            is_slides,
            bibliography,
            lang,
            biblio_title,
            toc_title,
            toc_depth,
            ref_exclude,
            ref_include,
            kroki_url: None,
            resource_dir: None,
        },
    )
}

pub fn markdown_to_typst_with_options(
    markdown: &str,
    opts: &MarkdownOptions<'_>,
) -> String {
    let is_slides = opts.is_slides;
    let bibliography = opts.bibliography;
    let lang = opts.lang;
    let biblio_title = opts.biblio_title;
    let toc_title = opts.toc_title;
    let toc_depth = opts.toc_depth;
    let ref_exclude = opts.ref_exclude;
    let ref_include = opts.ref_include;

    let preprocessed = preprocess_pandoc(markdown);
    let markdown = &preprocessed;
    let has_ref_command = preprocessed.contains("<!--botox:ref");
    let should_index_citations = bibliography || has_ref_command;

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_MATH);
    options.insert(Options::ENABLE_DEFINITION_LIST);

    // Pass 1: Parse and collect footnote definitions
    let parser1 = Parser::new_ext(markdown, options);
    let mut footnote_defs: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut current_footnote: Option<(String, String)> = None;

    for event in parser1 {
        match event {
            Event::Start(Tag::FootnoteDefinition(name)) => {
                current_footnote = Some((name.to_string(), String::new()));
            }
            Event::End(TagEnd::FootnoteDefinition) => {
                if let Some((name, text)) = current_footnote.take() {
                    footnote_defs.insert(name, text.trim().to_string());
                }
            }
            Event::Text(t) => {
                if let Some((_, ref mut text)) = current_footnote {
                    text.push_str(&escape_typst_text(&t));
                }
            }
            Event::Code(c) => {
                if let Some((_, ref mut text)) = current_footnote {
                    text.push('`');
                    text.push_str(&c);
                    text.push('`');
                }
            }
            _ => {}
        }
    }

    let mut events: Vec<Event> = Parser::new_ext(markdown, options).collect();
    let mut typst = String::with_capacity(markdown.len() * 2);

    let mut list_depth: usize = 0;
    let mut in_table_header = false;
    let mut table_cells: Vec<String> = Vec::new();
    let mut table_col_count = 0;
    let mut table_alignments: Vec<&'static str> = Vec::new();
    let mut current_cell = String::new();
    let mut in_table = false;
    let mut in_footnote_def = false;
    let mut references: Vec<(String, String)> = Vec::new();
    let mut current_link: Option<(String, String)> = None;
    let mut current_image: Option<(String, String)> = None;
    let mut current_heading: Option<(usize, String)> = None;
    let mut pending_table_caption: Option<(String, Option<String>)> = None;

    let num_events = events.len();
    let mut i = 0;
    while i < num_events {
        let event = std::mem::replace(&mut events[i], Event::Text("".into()));
        match event {
            Event::Start(tag) => {
                if let Tag::FootnoteDefinition(_) = tag {
                    in_footnote_def = true;
                    i += 1;
                    continue;
                }
                if in_footnote_def {
                    i += 1;
                    continue;
                }
                match tag {
                    Tag::Paragraph => {
                        if let Some(Event::Text(next_text)) = events.get(i + 1) {
                            let trimmed = next_text.trim();
                            if (trimmed.starts_with("Table:") || trimmed.starts_with(": ")) && trimmed.len() > 2 {
                                let mut j = i + 2;
                                let mut is_tbl = false;
                                while j < num_events {
                                    match &events[j] {
                                        Event::End(TagEnd::Paragraph) => {
                                            if let Some(Event::Start(Tag::Table(_))) = events.get(j + 1) {
                                                is_tbl = true;
                                            }
                                            break;
                                        }
                                        _ => j += 1,
                                    }
                                }
                                if is_tbl {
                                    let (caption, label) = parse_caption_and_label(trimmed);
                                    pending_table_caption = Some((caption, label));
                                    i = j + 1;
                                    continue;
                                }
                            }
                        }
                    }
                    Tag::Heading { level, .. } => {
                        let lvl = match level {
                            HeadingLevel::H1 => 1,
                            HeadingLevel::H2 => 2,
                            HeadingLevel::H3 => 3,
                            HeadingLevel::H4 => 4,
                            HeadingLevel::H5 => 5,
                            HeadingLevel::H6 => 6,
                        };
                        current_heading = Some((lvl, String::new()));
                    }
                    Tag::BlockQuote(_) => {
                        typst.push_str("#quote[");
                    }
                    Tag::CodeBlock(kind) => {
                        let mut code = String::new();
                        let mut j = i + 1;
                        while j < num_events {
                            match &events[j] {
                                Event::Text(t) => {
                                    code.push_str(t);
                                    j += 1;
                                }
                                Event::End(TagEnd::CodeBlock) => {
                                    break;
                                }
                                _ => {
                                    j += 1;
                                }
                            }
                        }
                        i = j + 1;

                        if let pulldown_cmark::CodeBlockKind::Fenced(ref fence) = kind
                            && let Some(spec) = parse_diagram_fence(fence) {
                                let mut diag_code = code.clone();
                                let mut file_to_load = spec.file.clone();
                                if file_to_load.is_none() {
                                    let trimmed = code.trim();
                                    if let Some(rest) = trimmed.strip_prefix("!include ") {
                                        file_to_load = Some(rest.trim().to_string());
                                    } else if let Some(rest) = trimmed.strip_prefix("include: ") {
                                        file_to_load = Some(rest.trim().to_string());
                                    } else if let Some(rest) = trimmed.strip_prefix("file: ") {
                                        file_to_load = Some(rest.trim().to_string());
                                    } else if !trimmed.contains('\n')
                                        && (trimmed.ends_with(".uxf")
                                            || trimmed.ends_with(".umlet")
                                            || trimmed.ends_with(".puml")
                                            || trimmed.ends_with(".plantuml")
                                            || trimmed.ends_with(".mmd")
                                            || trimmed.ends_with(".mermaid")
                                            || trimmed.ends_with(".dot"))
                                    {
                                        file_to_load = Some(trimmed.to_string());
                                    }
                                }

                                if let Some(ref file_path) = file_to_load {
                                    if let Some(loaded) = resolve_file_content(file_path, opts.resource_dir) {
                                        diag_code = loaded;
                                    } else {
                                        typst.push_str(&format!(
                                            "\n#botox_callout(\"warning\", \"Diagram file not found: {}\")[\nCould not load referenced diagram file `{}`.\n]\n\n",
                                            escape_typst_text(file_path),
                                            escape_typst_text(file_path),
                                        ));
                                        continue;
                                    }
                                }

                                match crate::diagrams::render_diagram(&spec.diagram_type, &diag_code, opts.kroki_url) {
                                    Ok(cached_path) => {
                                        let path_str = cached_path.to_string_lossy().replace('\\', "/");
                                        let mut img_args = vec![format!("\"{path_str}\"")];
                                        if let Some(ref w) = spec.width {
                                            img_args.push(format!("width: {w}"));
                                        }
                                        if let Some(ref h) = spec.height {
                                            img_args.push(format!("height: {h}"));
                                        }
                                        let img_call = format!("image({})", img_args.join(", "));

                                        if spec.caption.is_some() || spec.id.is_some() {
                                            let mut fig = format!("\n#figure({img_call}");
                                            if let Some(ref cap) = spec.caption {
                                                let escaped_cap = escape_typst_text(cap);
                                                fig.push_str(&format!(", caption: [{escaped_cap}]"));
                                            }
                                            fig.push(')');
                                            if let Some(ref label) = spec.id {
                                                fig.push_str(&format!(" <{label}>"));
                                            }
                                            fig.push_str("\n\n");
                                            typst.push_str(&fig);
                                        } else {
                                            typst.push_str(&format!("\n#align(center)[#{img_call}]\n\n"));
                                        }
                                    }
                                    Err(_) => {
                                        let title = "Diagram rendering failed: Kroki unreachable";
                                        let raw_lang = &spec.diagram_type;
                                        let clean_code = if diag_code.ends_with('\n') {
                                            diag_code
                                        } else {
                                            format!("{diag_code}\n")
                                        };
                                        let mut max_ticks = 0;
                                        let mut cur_ticks = 0;
                                        for ch in clean_code.chars() {
                                            if ch == '`' {
                                                cur_ticks += 1;
                                                if cur_ticks > max_ticks { max_ticks = cur_ticks; }
                                            } else {
                                                cur_ticks = 0;
                                            }
                                        }
                                        let fence_str = "`".repeat((max_ticks + 1).max(3));
                                        typst.push_str(&format!(
                                            "\n#botox_callout(\"warning\", \"{title}\")[\n{fence_str}{raw_lang}\n{clean_code}{fence_str}\n]\n\n"
                                        ));
                                    }
                                }
                                continue;
                            }

                        // Regular code block (fenced or indented)
                        let mut final_code = code;
                        let lang = match &kind {
                            pulldown_cmark::CodeBlockKind::Fenced(l) => {
                                let (clean_lang, code_file) = parse_code_fence_file(l.as_ref());
                                if let Some(ref cf) = code_file {
                                    if let Some(loaded) = resolve_file_content(cf, opts.resource_dir) {
                                        final_code = loaded;
                                    } else {
                                        typst.push_str(&format!(
                                            "\n#botox_callout(\"warning\", \"Code file not found: {}\")[\nCould not load referenced code file `{}`.\n]\n\n",
                                            escape_typst_text(cf),
                                            escape_typst_text(cf),
                                        ));
                                        continue;
                                    }
                                }
                                clean_lang
                            }
                            pulldown_cmark::CodeBlockKind::Indented => String::new(),
                        };
                        let mut max_ticks = 0;
                        let mut cur_ticks = 0;
                        for ch in final_code.chars() {
                            if ch == '`' {
                                cur_ticks += 1;
                                if cur_ticks > max_ticks { max_ticks = cur_ticks; }
                            } else {
                                cur_ticks = 0;
                            }
                        }
                        let fence_str = "`".repeat((max_ticks + 1).max(3));
                        let clean_code = if final_code.ends_with('\n') {
                            final_code
                        } else {
                            format!("{final_code}\n")
                        };
                        typst.push_str(&format!("{fence_str}{lang}\n{clean_code}{fence_str}\n\n"));
                        continue;
                    }
                    Tag::List(_) => {
                        list_depth += 1;
                    }
                    Tag::Item => {
                        let indent = "  ".repeat(list_depth.saturating_sub(1));
                        typst.push_str(&indent);
                        typst.push_str("- ");
                    }
                    Tag::Emphasis => {
                        if let Some((_, ref mut h_buf)) = current_heading {
                            h_buf.push('_');
                        } else if in_table {
                            current_cell.push('_');
                        } else {
                            typst.push('_');
                        }
                    }
                    Tag::Strong => {
                        if let Some((_, ref mut h_buf)) = current_heading {
                            h_buf.push('*');
                        } else if in_table {
                            current_cell.push('*');
                        } else {
                            typst.push('*');
                        }
                    }
                    Tag::Strikethrough => {
                        if in_table { current_cell.push_str("#strike["); } else { typst.push_str("#strike["); }
                    }
                    Tag::Subscript => {
                        if in_table { current_cell.push_str("#sub["); } else { typst.push_str("#sub["); }
                    }
                    Tag::Superscript => {
                        if in_table { current_cell.push_str("#super["); } else { typst.push_str("#super["); }
                    }
                    Tag::DefinitionList => {
                        typst.push('\n');
                    }
                    Tag::DefinitionListTitle => {
                        typst.push_str("\n/ ");
                    }
                    Tag::DefinitionListDefinition => {}
                    Tag::FootnoteDefinition(_) => {
                        in_footnote_def = true;
                    }
                    Tag::Link { dest_url, .. } => {
                        let is_external = dest_url.starts_with("http://")
                            || dest_url.starts_with("https://")
                            || dest_url.starts_with("ftp://");

                        let is_excluded = if is_external {
                            let not_in_include = if let Some(inc) = ref_include {
                                !inc.iter().any(|pat| url_matches_pattern(&dest_url, pat))
                            } else {
                                false
                            };
                            let in_exclude = if let Some(exc) = ref_exclude {
                                exc.iter().any(|pat| url_matches_pattern(&dest_url, pat))
                            } else {
                                false
                            };
                            not_in_include || in_exclude
                        } else {
                            true
                        };

                        if should_index_citations && is_external && !is_excluded {
                            current_link = Some((dest_url.to_string(), String::new()));
                        } else {
                            current_link = None;
                        }
                        let safe_url = dest_url.replace('\\', "/").replace('"', "\\\"");
                        let link_code = format!("#link(\"{safe_url}\")[");
                        if in_table { current_cell.push_str(&link_code); } else { typst.push_str(&link_code); }
                    }
                    Tag::Image { dest_url, .. } => {
                        current_image = Some((dest_url.to_string(), String::new()));
                    }
                    Tag::Table(alignments) => {
                        in_table = true;
                        table_cells.clear();
                        table_col_count = 0;
                        table_alignments = alignments.into_iter().map(|a| match a {
                            pulldown_cmark::Alignment::Center => "center",
                            pulldown_cmark::Alignment::Right => "right",
                            _ => "left",
                        }).collect();
                    }
                    Tag::TableHead => {
                        in_table_header = true;
                    }
                    Tag::TableRow => {}
                    Tag::TableCell => {
                        current_cell.clear();
                    }
                    _ => {}
                }
            }
            Event::End(tag) => {
                if let TagEnd::FootnoteDefinition = tag {
                    in_footnote_def = false;
                    i += 1;
                    continue;
                }
                if in_footnote_def {
                    i += 1;
                    continue;
                }
                match tag {
                    TagEnd::Paragraph => {
                        typst.push_str("\n\n");
                    }
                    TagEnd::Heading(_) => {
                        if let Some((lvl, h_text)) = current_heading.take() {
                            let (clean_title, is_unnumbered, id) = parse_heading_attributes(&h_text);
                            if is_unnumbered {
                                typst.push_str(&format!("#heading(level: {lvl}, numbering: none)[{clean_title}]"));
                            } else {
                                let prefix = match lvl {
                                    1 => "=",
                                    2 => "==",
                                    3 => "===",
                                    4 => "====",
                                    5 => "=====",
                                    _ => "======",
                                };
                                typst.push_str(&format!("{prefix} {clean_title}"));
                            }
                            if let Some(ref label) = id {
                                typst.push_str(&format!(" <{label}>"));
                            }
                            typst.push_str("\n\n");
                        } else {
                            typst.push_str("\n\n");
                        }
                    }
                    TagEnd::BlockQuote(_) => {
                        typst.push_str("]\n\n");
                    }
                    TagEnd::CodeBlock => {}
                    TagEnd::List(_) => {
                        list_depth = list_depth.saturating_sub(1);
                        if list_depth == 0 {
                            typst.push('\n');
                        }
                    }
                    TagEnd::Item => {
                        typst.push('\n');
                    }
                    TagEnd::Emphasis => {
                        if let Some((_, ref mut h_buf)) = current_heading {
                            h_buf.push('_');
                        } else if in_table {
                            current_cell.push('_');
                        } else {
                            typst.push('_');
                        }
                    }
                    TagEnd::Strong => {
                        if let Some((_, ref mut h_buf)) = current_heading {
                            h_buf.push('*');
                        } else if in_table {
                            current_cell.push('*');
                        } else {
                            typst.push('*');
                        }
                    }
                    TagEnd::Strikethrough => {
                        if in_table { current_cell.push(']'); } else { typst.push(']'); }
                    }
                    TagEnd::Subscript => {
                        if in_table { current_cell.push(']'); } else { typst.push(']'); }
                    }
                    TagEnd::Superscript => {
                        if in_table { current_cell.push(']'); } else { typst.push(']'); }
                    }
                    TagEnd::Link => {
                        if in_table { current_cell.push(']'); } else { typst.push(']'); }
                        if let Some((url, link_text)) = current_link.take() {
                            let idx = if let Some(pos) = references.iter().position(|(u, _)| u == &url) {
                                pos + 1
                            } else {
                                references.push((url.clone(), link_text.trim().to_string()));
                                references.len()
                            };

                            let trimmed = link_text.trim();
                            let already_citation = (trimmed.starts_with('[') && trimmed.ends_with(']'))
                                || trimmed.parse::<usize>().is_ok();

                            if !already_citation {
                                let cite_code = format!(" #link(<bib-{idx}>)[\\[{idx}\\]]");
                                if in_table {
                                    current_cell.push_str(&cite_code);
                                } else {
                                    typst.push_str(&cite_code);
                                }
                            }
                        }
                    }
                    TagEnd::Image => {
                        let mut image_attrs = (None, None, None);
                        if let Some(Event::Text(next_text)) = events.get_mut(i + 1) {
                            let trimmed = next_text.trim_start();
                            if trimmed.starts_with('{')
                                && let Some(brace_end) = trimmed.find('}') {
                                    let attr_part = &trimmed[1..brace_end];
                                    image_attrs = parse_image_attributes(attr_part);
                                    let remainder = trimmed[brace_end + 1..].to_string();
                                    *next_text = remainder.into();
                                }
                        }

                        if let Some((url, alt)) = current_image.take() {
                            let alt = alt.trim();
                            let (width, height, id) = image_attrs;

                            let norm_url = url.trim().to_lowercase();
                            let is_diagram_file = norm_url.ends_with(".uxf")
                                || norm_url.ends_with(".umlet")
                                || norm_url.ends_with(".puml")
                                || norm_url.ends_with(".plantuml")
                                || norm_url.ends_with(".mmd")
                                || norm_url.ends_with(".mermaid")
                                || norm_url.ends_with(".dot")
                                || norm_url.ends_with(".gv")
                                || norm_url.ends_with(".ditaa")
                                || norm_url.ends_with(".wavedrom")
                                || norm_url.ends_with(".bytefield")
                                || norm_url.ends_with(".bpmn");

                            let final_img_path = if is_diagram_file {
                                let diag_type = if norm_url.ends_with(".uxf") || norm_url.ends_with(".umlet") {
                                    "umlet"
                                } else if norm_url.ends_with(".puml") || norm_url.ends_with(".plantuml") {
                                    "plantuml"
                                } else if norm_url.ends_with(".mmd") || norm_url.ends_with(".mermaid") {
                                    "mermaid"
                                } else if norm_url.ends_with(".dot") || norm_url.ends_with(".gv") {
                                    "graphviz"
                                } else if norm_url.ends_with(".ditaa") {
                                    "ditaa"
                                } else if norm_url.ends_with(".wavedrom") {
                                    "wavedrom"
                                } else if norm_url.ends_with(".bytefield") {
                                    "bytefield"
                                } else if norm_url.ends_with(".bpmn") {
                                    "bpmn"
                                } else {
                                    "mermaid"
                                };

                                if let Some(content) = resolve_file_content(&url, opts.resource_dir) {
                                    match crate::diagrams::render_diagram(diag_type, &content, opts.kroki_url) {
                                        Ok(cached_path) => cached_path.to_string_lossy().replace('\\', "/"),
                                        Err(_) => {
                                            typst.push_str(&format!(
                                                "\n#botox_callout(\"warning\", \"Diagram rendering failed: {}\")[\nCould not render diagram file `{}` via Kroki.\n]\n\n",
                                                escape_typst_text(&url),
                                                escape_typst_text(&url),
                                            ));
                                            continue;
                                        }
                                    }
                                } else {
                                    typst.push_str(&format!(
                                        "\n#botox_callout(\"warning\", \"Diagram file not found: {}\")[\nCould not find diagram file `{}`.\n]\n\n",
                                        escape_typst_text(&url),
                                        escape_typst_text(&url),
                                    ));
                                    continue;
                                }
                            } else {
                                url.replace('\\', "/").replace('"', "\\\"")
                            };

                            let mut img_args = vec![format!("\"{final_img_path}\"")];
                            if let Some(ref w) = width {
                                img_args.push(format!("width: {w}"));
                            }
                            if let Some(ref h) = height {
                                img_args.push(format!("height: {h}"));
                            }
                            let img_call = format!("image({})", img_args.join(", "));

                            if in_table {
                                current_cell.push_str(&format!("#{img_call}"));
                            } else if current_link.is_some() {
                                typst.push_str(&format!("#{img_call}"));
                            } else if is_slides {
                                if alt == "bg" || alt.starts_with("bg ") {
                                    typst.push_str(&format!("\n#place(top + left, dx: 0pt, dy: 0pt, image(\"{final_img_path}\", width: 100%, height: 100%, fit: \"cover\"))\n\n"));
                                } else if !alt.is_empty() || id.is_some() {
                                    let mut fig = format!("\n#figure({img_call}");
                                    if !alt.is_empty() {
                                        fig.push_str(&format!(", caption: [{alt}]"));
                                    }
                                    fig.push(')');
                                    if let Some(ref label) = id {
                                        fig.push_str(&format!(" <{label}>"));
                                    }
                                    fig.push_str("\n\n");
                                    typst.push_str(&fig);
                                } else {
                                    typst.push_str(&format!("\n#align(center)[#{img_call}]\n\n"));
                                }
                            } else {
                                if !alt.is_empty() || id.is_some() {
                                    let mut fig = format!("\n#figure({img_call}");
                                    if !alt.is_empty() {
                                        fig.push_str(&format!(", caption: [{alt}]"));
                                    }
                                    fig.push(')');
                                    if let Some(ref label) = id {
                                        fig.push_str(&format!(" <{label}>"));
                                    }
                                    fig.push_str("\n\n");
                                    typst.push_str(&fig);
                                } else {
                                    typst.push_str(&format!("\n#align(center)[#{img_call}]\n\n"));
                                }
                            }
                        }
                    }
                    TagEnd::DefinitionList => {
                        typst.push('\n');
                    }
                    TagEnd::DefinitionListTitle => {
                        typst.push_str(": ");
                    }
                    TagEnd::DefinitionListDefinition => {
                        typst.push_str("\n\n");
                    }
                    TagEnd::TableHead => {
                        in_table_header = false;
                    }
                    TagEnd::TableRow => {}
                    TagEnd::TableCell => {
                        if in_table_header {
                            table_col_count += 1;
                        }
                        table_cells.push(current_cell.trim().to_string());
                        current_cell.clear();
                    }
                    TagEnd::Table => {
                        in_table = false;
                        let cols = if table_col_count > 0 { table_col_count } else { 1 };
                        let align_str = if !table_alignments.is_empty() {
                            let items: Vec<&str> = table_alignments.iter().copied().take(cols).collect();
                            items.join(", ")
                        } else {
                            "left".to_string()
                        };
                        let mut tbl_str = format!("table(\n  columns: {cols},\n  align: ({align_str}),\n");
                        if table_col_count > 0 && table_cells.len() >= table_col_count {
                            let header_slice = &table_cells[..table_col_count];
                            let header_args: Vec<String> = header_slice
                                .iter()
                                .map(|c| format!("[*{c}*]"))
                                .collect();
                            tbl_str.push_str(&format!("  table.header({}),\n", header_args.join(", ")));
                            for cell in &table_cells[table_col_count..] {
                                tbl_str.push_str(&format!("  [{cell}],\n"));
                            }
                        } else {
                            for cell in &table_cells {
                                tbl_str.push_str(&format!("  [{cell}],\n"));
                            }
                        }
                        tbl_str.push(')');

                        if let Some((caption, label)) = pending_table_caption.take() {
                            let escaped_caption = escape_typst_text(&caption);
                            let mut fig = format!("\n#figure(\n  {tbl_str},\n  caption: [{escaped_caption}],\n)");
                            if let Some(ref l) = label {
                                fig.push_str(&format!(" <{l}>"));
                            }
                            fig.push_str("\n\n");
                            typst.push_str(&fig);
                        } else {
                            typst.push_str(&format!("#{tbl_str}\n\n"));
                        }
                    }
                    _ => {}
                }
            }
            Event::Text(text) => {
                if in_footnote_def {
                    i += 1;
                    continue;
                }
                let escaped = escape_typst_text(&text);
                if let Some((_, ref mut h_buf)) = current_heading {
                    h_buf.push_str(&escaped);
                    i += 1;
                    continue;
                }
                if let Some((_, ref mut img_alt)) = current_image {
                    img_alt.push_str(&escaped);
                    i += 1;
                    continue;
                }
                if let Some((_, ref mut link_text)) = current_link {
                    link_text.push_str(&escaped);
                }
                if in_table {
                    current_cell.push_str(&escaped);
                } else {
                    typst.push_str(&escaped);
                }
            }
            Event::Html(html) | Event::InlineHtml(html) => {
                if in_footnote_def {
                    i += 1;
                    continue;
                }
                if html.contains("<!--botox:callout:start:") {
                    let rest = html.split("<!--botox:callout:start:").nth(1).unwrap_or("");
                    if let Some(content) = rest.split("-->").next() {
                        let mut parts = content.splitn(2, ':');
                        let kind = parts.next().unwrap_or("note");
                        let title = parts.next().unwrap_or("");
                        let escaped_title = escape_typst_text(title);
                        typst.push_str(&format!("\n#botox_callout(\"{kind}\", \"{escaped_title}\")[\n"));
                    }
                } else if html.contains("<!--botox:callout:end-->") {
                    typst.push_str("\n]\n\n");
                } else if html.contains("pagebreak") || html.contains("page-break") || html.contains("newpage") {
                    typst.push_str("\n#pagebreak()\n\n");
                } else if html.contains("colbreak") || html.contains("columnbreak") || html.contains("column-break") {
                    typst.push_str("\n#colbreak()\n\n");
                } else if html.contains("<!--botox:pause-->") || html.contains("pause") {
                    typst.push_str("\n#botox_pause()\n\n");
                } else if html.contains("<!--botox:toc") {
                    let custom_title = if html.contains("<!--botox:toc:") {
                        let rest = html.split("<!--botox:toc:").nth(1).unwrap_or("");
                        let inner = rest.split("-->").next().unwrap_or("").trim();
                        if !inner.is_empty() {
                            Some(inner.to_string())
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    let title_to_use = custom_title.as_deref().or(toc_title);
                    let depth = toc_depth.unwrap_or(3);
                    if let Some(t) = title_to_use {
                        let escaped_t = escape_typst_text(t);
                        typst.push_str(&format!("\n#outline(title: \"{escaped_t}\", depth: {depth})\n#v(1.5em)\n\n"));
                    } else {
                        typst.push_str(&format!("\n#outline(depth: {depth})\n#v(1.5em)\n\n"));
                    }
                } else if html.contains("<!--botox:ref") {
                    let custom_title = if html.contains("<!--botox:ref:") {
                        let rest = html.split("<!--botox:ref:").nth(1).unwrap_or("");
                        let inner = rest.split("-->").next().unwrap_or("").trim();
                        inner.to_string()
                    } else {
                        String::new()
                    };
                    typst.push_str(&format!("\nBOTOX_REF_PLACEHOLDER_START:{custom_title}:BOTOX_REF_PLACEHOLDER_END\n\n"));
                } else if html.trim().starts_with("<!--") && html.trim().ends_with("-->") {
                    // Raw HTML comment: ignore
                } else {
                    let trimmed = html.trim().to_lowercase();
                    if trimmed == "<br>" || trimmed == "<br/>" || trimmed == "<br />" {
                        typst.push_str(" \\ \n");
                    } else {
                        let escaped = escape_typst_text(&html);
                        if in_table {
                            current_cell.push_str(&escaped);
                        } else {
                            typst.push_str(&escaped);
                        }
                    }
                }
            }
            Event::FootnoteReference(name) => {
                if in_footnote_def {
                    i += 1;
                    continue;
                }
                let body = footnote_defs.get(name.as_ref()).cloned().unwrap_or_default();
                typst.push_str(&format!("#footnote[{body}]"));
            }
            Event::TaskListMarker(checked) => {
                if in_footnote_def {
                    i += 1;
                    continue;
                }
                if checked {
                    let s = r"\[x\] ";
                    if in_table { current_cell.push_str(s); } else { typst.push_str(s); }
                } else {
                    let s = r"\[ \] ";
                    if in_table { current_cell.push_str(s); } else { typst.push_str(s); }
                }
            }
            Event::Code(code) => {
                if in_footnote_def {
                    i += 1;
                    continue;
                }
                if let Some((_, ref mut h_buf)) = current_heading {
                    h_buf.push('`');
                    h_buf.push_str(&code);
                    h_buf.push('`');
                    i += 1;
                    continue;
                }
                if let Some((_, ref mut img_alt)) = current_image {
                    img_alt.push('`');
                    img_alt.push_str(&code);
                    img_alt.push('`');
                    i += 1;
                    continue;
                }
                if let Some((_, ref mut link_text)) = current_link {
                    link_text.push('`');
                    link_text.push_str(&code);
                    link_text.push('`');
                }
                if in_table {
                    current_cell.push('`');
                    current_cell.push_str(&code);
                    current_cell.push('`');
                } else {
                    typst.push('`');
                    typst.push_str(&code);
                    typst.push('`');
                }
            }
            Event::InlineMath(math) => {
                if in_footnote_def {
                    i += 1;
                    continue;
                }
                if let Some((_, ref mut h_buf)) = current_heading {
                    let converted = convert_latex_math_to_typst(&math);
                    h_buf.push('$');
                    h_buf.push_str(converted.trim());
                    h_buf.push('$');
                    i += 1;
                    continue;
                }
                let converted = convert_latex_math_to_typst(&math);
                let s = format!("${}$", converted.trim());
                if in_table {
                    current_cell.push_str(&s);
                } else {
                    typst.push_str(&s);
                }
            }
            Event::DisplayMath(math) => {
                if in_footnote_def {
                    i += 1;
                    continue;
                }
                let mut eq_label = None;
                if let Some(Event::Text(next_text)) = events.get_mut(i + 1) {
                    let trimmed = next_text.trim_start();
                    if trimmed.starts_with('{')
                        && let Some(brace_end) = trimmed.find('}') {
                            let attr = trimmed[1..brace_end].trim();
                            if let Some(stripped) = attr.strip_prefix('#') {
                                eq_label = Some(stripped.replace(':', "-"));
                                let remainder = trimmed[brace_end + 1..].to_string();
                                *next_text = remainder.into();
                            }
                        }
                }
                let mut math_str = math.to_string();
                if let Some(pos) = math_str.find(r"\label{")
                    && let Some(end) = math_str[pos..].find('}') {
                        let raw_label = &math_str[pos + 7..pos + end];
                        if eq_label.is_none() {
                            eq_label = Some(raw_label.replace(':', "-"));
                        }
                        math_str.replace_range(pos..pos + end + 1, "");
                    }
                let converted = convert_latex_math_to_typst(&math_str);
                let mut s = format!("\n$ {} $", converted.trim());
                if let Some(ref label) = eq_label {
                    s.push_str(&format!(" <{label}>"));
                }
                s.push_str("\n\n");
                if in_table {
                    current_cell.push_str(&s);
                } else {
                    typst.push_str(&s);
                }
            }
            Event::Rule => {
                if in_footnote_def {
                    i += 1;
                    continue;
                }
                if is_slides {
                    typst.push_str("\n#pagebreak()\n\n");
                } else {
                    typst.push_str("\n#line(length: 100%, stroke: 0.5pt + luma(180))\n\n");
                }
            }
            Event::SoftBreak => {
                if in_footnote_def {
                    i += 1;
                    continue;
                }
                if in_table {
                    current_cell.push(' ');
                } else {
                    typst.push('\n');
                }
            }
            Event::HardBreak => {
                if in_footnote_def {
                    i += 1;
                    continue;
                }
                if in_table {
                    current_cell.push(' ');
                } else {
                    typst.push_str("\\\n");
                }
            }
        }
        i += 1;
    }

    let format_references_block = |custom_heading: Option<&str>, refs: &[(String, String)], at_start_of_slide: bool| -> String {
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
        block.push_str(&format!("#heading(numbering: none)[{heading}] <references>\n\n"));
        block.push_str("#set par(hanging-indent: 1.8em, justify: false)\n\n");

        for (i, (url, label)) in refs.iter().enumerate() {
            let idx = i + 1;

            let is_url_label = label.is_empty()
                || label == url
                || label.starts_with("http://")
                || label.starts_with("https://");

            if is_url_label {
                block.push_str(&format!("#block[\\[{idx}\\] {online_label}. {available_label} #link(\"{url}\").] <bib-{idx}>\n\n"));
            } else {
                block.push_str(&format!("#block[\\[{idx}\\] \"{label}\", {online_label}. {available_label} #link(\"{url}\").] <bib-{idx}>\n\n"));
            }
        }
        block
    };

    let mut rendered_first = false;
    while let Some(start_pos) = typst.find("BOTOX_REF_PLACEHOLDER_START:") {
        if let Some(end_pos) = typst[start_pos..].find(":BOTOX_REF_PLACEHOLDER_END") {
            let full_end = start_pos + end_pos + ":BOTOX_REF_PLACEHOLDER_END".len();
            if !rendered_first {
                let title_raw = &typst[start_pos + "BOTOX_REF_PLACEHOLDER_START:".len()..start_pos + end_pos];
                let custom_heading = if !title_raw.trim().is_empty() {
                    Some(title_raw.trim())
                } else {
                    None
                };
                let at_start_of_slide = typst[..start_pos].trim_end().ends_with("#pagebreak()");
                let ref_rendered = format_references_block(custom_heading, &references, at_start_of_slide);
                typst.replace_range(start_pos..full_end, &ref_rendered);
                rendered_first = true;
            } else {
                // Strip duplicate \ref placeholders so duplicate <bib-n> labels are not emitted
                typst.replace_range(start_pos..full_end, "");
            }
        } else {
            break;
        }
    }

    if !has_ref_command && bibliography && !references.is_empty() {
        let ref_rendered = format_references_block(None, &references, false);
        typst.push_str(&ref_rendered);
    }

    typst
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sub_super_and_strikethrough() {
        let md = "H~2~O and 10^6^ with ~~strike~~ and `code_with_~_and_^`";
        let typst = markdown_to_typst(md, false, false, "en", None, None, None, None, None);
        println!("TYPST RESULT: {:?}", typst);
        assert!(typst.contains("#sub[2]"));
        assert!(typst.contains("#super[6]"));
        assert!(typst.contains("#strike[strike]"));
        assert!(typst.contains("`code_with_~_and_^`"));
    }

    #[test]
    fn test_typst_syntax_escaping() {
        let md = r#"
# Multiplicity 1..* and Comparisons: x < 5 and y > 2

Multiplicity is 1..* in text.
Contact user@domain.com or see issue #42 and C#.
Array index arr[0] and lone bracket ] alone.
Price is $100 and $200.
Code comment // not comment and /* unclosed
Path is C:\Users\Name and home ~/Desktop.
Variable foo_bar_baz.
/ Term: this is not a typst description list
Include <stdio.h> and break line<br>next line.
- [ ] Incomplete task
- [x] Done task
Check [link](https://example.com/api?q="quoted") here.
"#;
        let typst = markdown_to_typst(md, false, false, "en", None, None, None, None, None);
        assert!(typst.contains(r"1..\*"));
        assert!(typst.contains(r"x \< 5 and y \> 2"));
        assert!(typst.contains(r"user\@domain.com"));
        assert!(typst.contains(r"\#42"));
        assert!(typst.contains(r"C\#"));
        assert!(typst.contains(r"arr\[0\]"));
        assert!(typst.contains(r"\] alone"));
        assert!(typst.contains(r"\$100"));
        assert!(typst.contains(r"\$200"));
        assert!(typst.contains(r"\// not comment"));
        assert!(typst.contains(r"/\* unclosed"));
        assert!(typst.contains(r"C:\\Users\\Name"));
        assert!(typst.contains(r"\~/Desktop"));
        assert!(typst.contains(r"foo\_bar\_baz"));
        assert!(typst.contains(r"\/ Term:"));
        assert!(typst.contains(r"\<stdio.h\>"));
        assert!(typst.contains(r"\[ \]"));
        assert!(typst.contains(r"\[x\]"));
        assert!(typst.contains(r#"#link("https://example.com/api?q=\"quoted\"")"#));

        // Verify it compiles into PDF/JSON through Typst engine without error!
        let tmp_json = std::env::temp_dir().join("test_escaped_compilation.json");
        let res = crate::compiler::compile_typst(&typst, &tmp_json, None);
        assert!(res.is_ok(), "Typst compilation failed on escaped syntax: {:?}", res.err());
        let _ = std::fs::remove_file(tmp_json);
    }

    #[test]
    fn test_math_matrix_and_subscripts() {
        let latex = r"\begin{bmatrix} a_{11} & a_{12} \\ a_{21} & a_{22} \end{bmatrix}";
        let converted = convert_latex_math_to_typst(latex);
        assert!(converted.contains("mat(delim: \"[\""));
        assert!(converted.contains("a_(11)"));
        assert!(converted.contains("a_(12)"));
    }

    #[test]
    fn test_pagebreaks() {
        let md = "Before\n\n\\newpage\n\nMiddle\n\n\\columnbreak\n\nAfter";
        let typst = markdown_to_typst(md, false, false, "en", None, None, None, None, None);
        assert!(typst.contains("#pagebreak()"));
        assert!(typst.contains("#colbreak()"));
    }

    #[test]
    fn test_bibliography_link_transformation() {
        let md = "See [Rust](https://www.rust-lang.org) and [LLVM](https://llvm.org). Also [Rust Lang](https://www.rust-lang.org).";
        let typst = markdown_to_typst(md, false, true, "en", None, None, None, None, None);
        assert!(typst.contains("#link(<bib-1>)[\\"));
        assert!(typst.contains("#link(<bib-2>)[\\"));
        assert!(typst.contains("#heading(numbering: none)[References] <references>"));
        assert!(typst.contains("<bib-1>"));
        assert!(typst.contains("\\[1\\] \"Rust\", [Online]. Available: #link(\"https://www.rust-lang.org\")"));
        assert!(typst.contains("<bib-2>"));
        assert!(typst.contains("\\[2\\] \"LLVM\", [Online]. Available: #link(\"https://llvm.org\")"));
    }

    #[test]
    fn test_explicit_toc_spawning() {
        // Plain \toc
        let md1 = "# Chapter 1\n\n\\toc\n\n# Chapter 2";
        let typ1 = markdown_to_typst(md1, false, false, "en", None, None, Some(3), None, None);
        assert!(typ1.contains("#outline(depth: 3)"));

        // \toc with title
        let md2 = "# Intro\n\n\\toc Table of Contents\n\n# Main";
        let typ2 = markdown_to_typst(md2, false, false, "en", None, None, Some(2), None, None);
        assert!(typ2.contains("#outline(title: \"Table of Contents\", depth: 2)"));

        // \toc with quotes
        let md3 = "\\toc \"Agenda Overview\"";
        let typ3 = markdown_to_typst(md3, false, false, "en", None, None, None, None, None);
        assert!(typ3.contains("#outline(title: \"Agenda Overview\", depth: 3)"));

        // \toc with braces
        let md4 = "\\toc {Document Outline}";
        let typ4 = markdown_to_typst(md4, false, false, "en", None, None, None, None, None);
        assert!(typ4.contains("#outline(title: \"Document Outline\", depth: 3)"));

        // \tableofcontents
        let md5 = "\\tableofcontents Sommaire";
        let typ5 = markdown_to_typst(md5, false, false, "fr", None, None, None, None, None);
        assert!(typ5.contains("#outline(title: \"Sommaire\", depth: 3)"));
    }

    #[test]
    fn test_explicit_ref_spawning_and_removal() {
        // Document without \ref and bibliography=false: NO references rendered, normal links
        let md_clean = "Visit [Google](https://google.com) for searching.";
        let typ_clean = markdown_to_typst(md_clean, false, false, "en", None, None, None, None, None);
        assert!(!typ_clean.contains("<references>"));
        assert!(!typ_clean.contains("<bib-1>"));
        assert!(typ_clean.contains("#link(\"https://google.com\")[Google]"));

        // Document with explicit \ref Works Cited
        let md_ref = "# Introduction\n\nCheck out [Rust](https://rust-lang.org) and [Typst](https://typst.app).\n\n\\ref Works Cited";
        let typ_ref = markdown_to_typst(md_ref, false, false, "en", None, None, None, None, None);
        assert!(typ_ref.contains("#heading(numbering: none)[Works Cited] <references>"));
        assert!(typ_ref.contains("#link(<bib-1>)["));
        assert!(typ_ref.contains("<bib-1>"));
        assert!(typ_ref.contains("<bib-2>"));
        assert!(typ_ref.contains("https://rust-lang.org"));
        assert!(typ_ref.contains("https://typst.app"));

        // Document with \ref Sources in quotes
        let md_quotes = "See [Site](https://example.com).\n\n\\ref \"Sources and References\"";
        let typ_quotes = markdown_to_typst(md_quotes, false, false, "en", None, None, None, None, None);
        assert!(typ_quotes.contains("#heading(numbering: none)[Sources and References] <references>"));

        // Document with plain \ref
        let md_plain = "See [Site](https://example.com).\n\n\\ref";
        let typ_plain = markdown_to_typst(md_plain, false, false, "en", None, None, None, None, None);
        assert!(typ_plain.contains("#heading(numbering: none)[References] <references>"));

        // LaTeX cross-reference \ref{sec:intro} inside text must NOT trigger references block
        let md_latex = "As shown in section \\ref{sec:intro}.";
        let typ_latex = markdown_to_typst(md_latex, false, false, "en", None, None, None, None, None);
        assert!(!typ_latex.contains("<references>"));
    }

    #[test]
    fn test_url_matches_pattern() {
        assert!(url_matches_pattern("https://github.com/rust-lang/rust", "github.com"));
        assert!(url_matches_pattern("https://www.github.com/rust-lang/rust", "github.com"));
        assert!(url_matches_pattern("https://github.com", "*github.com*"));
        assert!(url_matches_pattern("https://x.com/profile", "x.com"));
        // Avoid false positive on substring in host:
        assert!(!url_matches_pattern("https://latex.com/page", "x.com"));
        // Glob matching:
        assert!(url_matches_pattern("https://en.wikipedia.org/wiki/Rust", "*.org"));
        assert!(url_matches_pattern("https://my-company.com/internal/docs", "my-company.com/*"));
        assert!(!url_matches_pattern("https://other.com/internal/docs", "my-company.com/*"));
    }

    #[test]
    fn test_references_exclusion_and_inclusion() {
        let md = "See [GitHub](https://github.com/rust-lang/rust), [Twitter](https://x.com/rustlang), and [Rust Lang](https://rust-lang.org).\n\n\\ref";

        // Exclude github.com and x.com:
        let exc = vec!["github.com".to_string(), "x.com".to_string()];
        let typ = markdown_to_typst(md, false, false, "en", None, None, None, Some(&exc), None);
        // Only Rust Lang should be cited in bibliography as [1]
        assert!(typ.contains("#link(<bib-1>)["));
        assert!(!typ.contains("#link(<bib-2>)["));
        assert!(typ.contains("<bib-1>"));
        assert!(typ.contains("https://rust-lang.org"));
        assert!(!typ.contains("<bib-2>"));
        // GitHub and Twitter should be normal inline links without citation markers
        assert!(typ.contains("#link(\"https://github.com/rust-lang/rust\")[GitHub]"));
        assert!(typ.contains("#link(\"https://x.com/rustlang\")[Twitter]"));

        // Inclusion whitelist: only rust-lang.org
        let inc = vec!["rust-lang.org".to_string()];
        let typ_inc = markdown_to_typst(md, false, false, "en", None, None, None, None, Some(&inc));
        assert!(typ_inc.contains("#link(<bib-1>)["));
        assert!(!typ_inc.contains("<bib-2>"));
        assert!(typ_inc.contains("https://rust-lang.org"));
    }

    #[test]
    fn test_display_math_and_latex_superscript() {
        let md = "$$\\int_{-\\infty}^{+\\infty} e^{-x^2} \\, dx = \\sqrt{\\pi}$$\n\n$$\n\\sum_{k=0}^\\infty \\frac{1}{k!}\n$$";
        let typst = markdown_to_typst(md, false, false, "en", None, None, None, None, None);
        assert!(!typst.contains("#super"), "Math must not contain #super: {typst}");
        assert!(!typst.contains("#sub"), "Math must not contain #sub: {typst}");
        assert!(typst.contains("oo"), "Infinity should be oo: {typst}");
        assert!(typst.contains("integral_(-oo)^(+oo)"));
    }

    #[test]
    fn test_image_rendering() {
        let md = "![Architecture Pipeline](figures/arch.png)\n\n![](logo.svg)";
        let typst = markdown_to_typst(md, false, false, "en", None, None, None, None, None);
        assert!(typst.contains("#figure(image(\"figures/arch.png\"), caption: [Architecture Pipeline])"));
        assert!(typst.contains("#align(center)[#image(\"logo.svg\")]"));
    }

    #[test]
    fn test_pandoc_image_attributes() {
        let md = "![Architecture Pipeline](figures/arch.png){width=50% #fig:pipeline}\n\n![](logo.svg){width=10cm height=5cm}\n\n![Relative](banner.png){width=0.8\\linewidth}";
        let typst = markdown_to_typst(md, false, false, "en", None, None, None, None, None);
        assert!(typst.contains("#figure(image(\"figures/arch.png\", width: 50%), caption: [Architecture Pipeline]) <fig-pipeline>"));
        assert!(typst.contains("#align(center)[#image(\"logo.svg\", width: 10cm, height: 5cm)]"));
        assert!(typst.contains("#figure(image(\"banner.png\", width: 80%), caption: [Relative])"));
    }

    #[test]
    fn test_pandoc_cross_references() {
        let md = "As seen in @fig:pipeline and @tbl:results, we refer to @sec:intro and @eq:euler.";
        let typst = markdown_to_typst(md, false, false, "en", None, None, None, None, None);
        assert!(typst.contains("@fig-pipeline"));
        assert!(typst.contains("@tbl-results"));
        assert!(typst.contains("@sec-intro"));
        assert!(typst.contains("@eq-euler"));
    }

    #[test]
    fn test_pandoc_heading_attributes() {
        let md = "# Introduction {#sec:intro}\n\n## Appendix {-}\n\n### Extra Notes {.unnumbered #sec:extra}";
        let typst = markdown_to_typst(md, false, false, "en", None, None, None, None, None);
        assert!(typst.contains("= Introduction <sec-intro>"));
        assert!(typst.contains("#heading(level: 2, numbering: none)[Appendix]"));
        assert!(typst.contains("#heading(level: 3, numbering: none)[Extra Notes] <sec-extra>"));
    }

    #[test]
    fn test_pandoc_callout_divs() {
        let md = "::: note\nThis is a standard note.\n:::\n\n::: {.warning title=\"Caution Alert\"}\nDanger ahead!\n:::";
        let typst = markdown_to_typst(md, false, false, "en", None, None, None, None, None);
        assert!(typst.contains("#botox_callout(\"note\", \"\")[\nThis is a standard note."));
        assert!(typst.contains("#botox_callout(\"warning\", \"Caution Alert\")[\nDanger ahead!"));
    }

    #[test]
    fn test_github_callouts() {
        let md = "> [!NOTE]\n> This is a GitHub note.\n\n> [!TIP] Pro Tip\n> Remember to save your work.\n\n> [!IMPORTANT]\n> Critical instruction here.\n\n> [!WARNING]\n> High voltage!";
        let typst = markdown_to_typst(md, false, false, "en", None, None, None, None, None);
        assert!(typst.contains("#botox_callout(\"note\", \"\")[\nThis is a GitHub note."));
        assert!(typst.contains("#botox_callout(\"tip\", \"Pro Tip\")[\nRemember to save your work."));
        assert!(typst.contains("#botox_callout(\"important\", \"\")[\nCritical instruction here."));
        assert!(typst.contains("#botox_callout(\"warning\", \"\")[\nHigh voltage!"));
    }

    #[test]
    fn test_pandoc_display_math_label() {
        let md = "$$ E = m c^2 $$ {#eq:einstein}\n\n$$ a^2 + b^2 = c^2 \\label{eq:pythagoras} $$";
        let typst = markdown_to_typst(md, false, false, "en", None, None, None, None, None);
        assert!(typst.contains("<eq-einstein>"));
        assert!(typst.contains("<eq-pythagoras>"));
    }

    #[test]
    fn test_pandoc_table_caption_and_label() {
        let md = "Table: Forwarding performance summary. {#tbl:perf}\n\n| Technique | Speedup |\n| --- | --- |\n| Full | 1.45x |";
        let typst = markdown_to_typst(md, false, false, "en", None, None, None, None, None);
        assert!(typst.contains("caption: [Forwarding performance summary.]"));
        assert!(typst.contains("<tbl-perf>"));
    }

    #[test]
    fn test_today_replacement() {
        let md = "Today's date is \\today in presentation.";
        let typst = markdown_to_typst(md, false, false, "en", None, None, None, None, None);
        assert!(typst.contains(r#"#datetime.today().display("[day] [month repr:long] [year]")"#));
    }

    #[test]
    fn test_pause_extraction() {
        let md = "Point 1\n\\pause\nPoint 2\n<!-- pause -->\nPoint 3";
        let typst = markdown_to_typst(md, true, false, "en", None, None, None, None, None);
        assert!(typst.contains("#botox_pause()"));
    }

    #[test]
    fn test_parse_diagram_fence() {
        let spec1 = parse_diagram_fence("mermaid").expect("mermaid");
        assert_eq!(spec1.diagram_type, "mermaid");
        assert_eq!(spec1.caption, None);

        let spec2 = parse_diagram_fence("plantuml {caption=\"System Model\" width=80% #fig:arch}").expect("plantuml");
        assert_eq!(spec2.diagram_type, "plantuml");
        assert_eq!(spec2.caption.as_deref(), Some("System Model"));
        assert_eq!(spec2.width.as_deref(), Some("80%"));
        assert_eq!(spec2.id.as_deref(), Some("fig-arch"));

        let spec3 = parse_diagram_fence("puml {title='Auth Sequence'}").expect("puml");
        assert_eq!(spec3.diagram_type, "plantuml");
        assert_eq!(spec3.caption.as_deref(), Some("Auth Sequence"));

        let spec4 = parse_diagram_fence("{.mermaid width=10cm}").expect(".mermaid");
        assert_eq!(spec4.diagram_type, "mermaid");
        assert_eq!(spec4.width.as_deref(), Some("10cm"));

        let spec_umlet = parse_diagram_fence("umlet file=diagram.uxf {caption=\"Architecture Model\" #fig:arch}").expect("umlet");
        assert_eq!(spec_umlet.diagram_type, "umlet");
        assert_eq!(spec_umlet.file.as_deref(), Some("diagram.uxf"));
        assert_eq!(spec_umlet.caption.as_deref(), Some("Architecture Model"));
        assert_eq!(spec_umlet.id.as_deref(), Some("fig-arch"));

        let spec_uxf = parse_diagram_fence("uxf:model.uxf").expect("uxf colon");
        assert_eq!(spec_uxf.diagram_type, "umlet");
        assert_eq!(spec_uxf.file.as_deref(), Some("model.uxf"));

        assert!(parse_diagram_fence("rust").is_none());
        assert!(parse_diagram_fence("python").is_none());
    }

    #[test]
    fn test_file_referencing_for_diagrams_and_code() {
        let temp_dir = std::env::temp_dir().join(format!("botox_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        let sample_uxf = r#"<diagram program="umlet"><zoom_level>10</zoom_level><element><id>UMLClass</id><coordinates><x>20</x><y>20</y><w>100</w><h>40</h></coordinates><panel_attributes>User</panel_attributes></element></diagram>"#;
        let uxf_path = temp_dir.join("sample.uxf");
        std::fs::write(&uxf_path, sample_uxf).unwrap();

        let code_path = temp_dir.join("main.rs");
        std::fs::write(&code_path, "fn main() {\n    println!(\"Hello Botox\");\n}\n").unwrap();

        // Pre-cache hash for UMLet diagram to verify mock rendering
        let hash = crate::diagrams::compute_diagram_hash("umlet", sample_uxf);
        let cache_dir = crate::diagrams::get_cache_dir();
        let _ = std::fs::create_dir_all(&cache_dir);
        let cache_file = cache_dir.join(format!("{hash}.svg"));
        std::fs::write(&cache_file, b"<svg><text>Mock UMLet</text></svg>").unwrap();

        // 1. Diagram fence referencing file attribute
        let md_fence = "```umlet file=sample.uxf {caption=\"UMLet Class Model\" #fig:umlet}\n```";
        let opts = MarkdownOptions {
            is_slides: false,
            bibliography: false,
            lang: "en",
            biblio_title: None,
            toc_title: None,
            toc_depth: None,
            ref_exclude: None,
            ref_include: None,
            kroki_url: None,
            resource_dir: Some(&temp_dir),
        };
        let typ_fence = markdown_to_typst_with_options(md_fence, &opts);
        assert!(typ_fence.contains("#figure(image("));
        assert!(typ_fence.contains("caption: [UMLet Class Model]"));
        assert!(typ_fence.contains("<fig-umlet>"));

        // 2. Diagram image syntax referencing .uxf file
        let md_img = "![Class Diagram](sample.uxf){#fig:umlet-img}";
        let typ_img = markdown_to_typst_with_options(md_img, &opts);
        assert!(typ_img.contains("#figure(image("));
        assert!(typ_img.contains("caption: [Class Diagram]"));
        assert!(typ_img.contains("<fig-umlet-img>"));

        // 3. Regular code block referencing file
        let md_code = "```rust file=main.rs\n```";
        let typ_code = markdown_to_typst_with_options(md_code, &opts);
        assert!(typ_code.contains("```rust\nfn main() {\n    println!(\"Hello Botox\");\n}\n```"));

        // Clean up
        let _ = std::fs::remove_file(cache_file);
        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_diagram_markdown_cached_and_fallback() {
        // Test 1: Cached diagram produces #figure(image(...))
        let diag_code = "graph LR\n  Client --> Server";
        let hash = crate::diagrams::compute_diagram_hash("mermaid", diag_code);
        let cache_dir = crate::diagrams::get_cache_dir();
        let _ = std::fs::create_dir_all(&cache_dir);
        let cache_file = cache_dir.join(format!("{hash}.svg"));
        std::fs::write(&cache_file, b"<svg><text>Mock Mermaid</text></svg>").unwrap();

        let md_cached = format!("```mermaid {{caption=\"Client-Server Architecture\" #fig:arch}}\n{diag_code}\n```");
        let typst_cached = markdown_to_typst(&md_cached, false, false, "en", None, None, None, None, None);
        assert!(typst_cached.contains("#figure(image("));
        assert!(typst_cached.contains("caption: [Client-Server Architecture]"));
        assert!(typst_cached.contains("<fig-arch>"));

        // Adding extra blank lines, spaces, and indentation hits the exact same cache file and produces identical output
        let md_with_newlines = "```mermaid {caption=\"Client-Server Architecture\" #fig:arch}\n\n  graph LR  \n\n    Client --> Server\n\n\n```";
        let typst_with_newlines = markdown_to_typst(md_with_newlines, false, false, "en", None, None, None, None, None);
        assert_eq!(typst_cached, typst_with_newlines);

        let _ = std::fs::remove_file(cache_file);

        // Test 2: Fallback when Kroki is unreachable produces warning callout with raw code
        unsafe { std::env::set_var("KROKI_ENDPOINT", "http://127.0.0.1:1"); }
        let md_fallback = "```plantuml\nAlice -> Bob : Ping\n```";
        let typst_fallback = markdown_to_typst(md_fallback, false, false, "en", None, None, None, None, None);
        unsafe { std::env::remove_var("KROKI_ENDPOINT"); }

        assert!(typst_fallback.contains("#botox_callout(\"warning\", \"Diagram rendering failed: Kroki unreachable\")"));
        assert!(typst_fallback.contains("```plantuml\nAlice -> Bob : Ping\n```"));
    }

    #[test]
    fn test_nested_code_fence_with_fig_attribute() {
        let md = "````markdown\n```mermaid {caption=\"High-Level Architecture\" width=75% #fig:arch}\ngraph TD\n  A --> B\n```\n````";
        let typst = markdown_to_typst(md, false, false, "en", None, None, None, None, None);
        assert!(typst.starts_with("````markdown\n"));
        assert!(typst.ends_with("````\n\n"));

        let fm = serde_yaml::Value::Mapping(serde_yaml::Mapping::new());
        let config = crate::config::DocumentConfig::defaults();
        let typst_markup = crate::document::wrap_document(&typst, &fm, &config, None, None, None, None);
        let tmp_pdf = std::env::temp_dir().join("test_nested_fence.pdf");
        let res = crate::compiler::compile_typst(&typst_markup, &tmp_pdf, None);
        assert!(res.is_ok(), "Typst compile error: {:?}", res.err());
        let _ = std::fs::remove_file(tmp_pdf);
    }
}
