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

fn convert_sub_super(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let n = chars.len();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;

    while i < n {
        let c = chars[i];
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
                        out.push_str(&format!("#sub[{sub}]"));
                        i = j + 1;
                        continue;
                    }
                }
            }
            out.push('~');
            i += 1;
        } else if c == '^' {
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
                        out.push_str(&format!("#super[{sup}]"));
                        i = j + 1;
                        continue;
                    }
                }
            }
            out.push('^');
            i += 1;
        } else {
            out.push(c);
            i += 1;
        }
    }

    out
}

fn preprocess_pandoc(markdown: &str) -> String {
    let mut result = String::with_capacity(markdown.len());
    let mut in_code_fence = false;
    let mut in_display_math = false;

    for line in markdown.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
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

        let mut processed_line = String::with_capacity(line.len());
        let mut chars = line.chars().peekable();
        let mut text_segment = String::new();

        while let Some(c) = chars.next() {
            if in_display_math {
                if c == '$' && chars.peek() == Some(&'$') {
                    chars.next(); // consume second $
                    processed_line.push_str("$$");
                    in_display_math = false;
                } else {
                    processed_line.push(c);
                }
                continue;
            }

            if c == '`' {
                if !text_segment.is_empty() {
                    processed_line.push_str(&convert_sub_super(&text_segment));
                    text_segment.clear();
                }
                processed_line.push('`');
                while let Some(c2) = chars.next() {
                    processed_line.push(c2);
                    if c2 == '`' {
                        break;
                    }
                }
            } else if c == '$' {
                if chars.peek() == Some(&'$') {
                    // Display math starts
                    chars.next(); // consume second $
                    if !text_segment.is_empty() {
                        processed_line.push_str(&convert_sub_super(&text_segment));
                        text_segment.clear();
                    }
                    processed_line.push_str("$$");
                    let mut closed = false;
                    while let Some(c2) = chars.next() {
                        if c2 == '$' && chars.peek() == Some(&'$') {
                            chars.next(); // consume second $
                            processed_line.push_str("$$");
                            closed = true;
                            break;
                        } else {
                            processed_line.push(c2);
                        }
                    }
                    if !closed {
                        in_display_math = true;
                    }
                } else {
                    // Inline math starts with a single $
                    if !text_segment.is_empty() {
                        processed_line.push_str(&convert_sub_super(&text_segment));
                        text_segment.clear();
                    }
                    processed_line.push('$');
                    let mut prev_backslash = false;
                    while let Some(c2) = chars.next() {
                        processed_line.push(c2);
                        if c2 == '$' && !prev_backslash {
                            break;
                        }
                        prev_backslash = c2 == '\\' && !prev_backslash;
                    }
                }
            } else {
                text_segment.push(c);
            }
        }
        if !text_segment.is_empty() {
            processed_line.push_str(&convert_sub_super(&text_segment));
        }

        result.push_str(&processed_line);
        result.push('\n');
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
            if rest.starts_with('{') {
                if let Some(end2) = find_matching_brace(&s, end1 + 1) {
                    let den = s[end1 + 2..end2].to_string();
                    let num_conv = convert_latex_math_to_typst(&num);
                    let den_conv = convert_latex_math_to_typst(&den);
                    s.replace_range(pos..end2 + 1, &format!("({num_conv}) / ({den_conv})"));
                    continue;
                }
            }
        }
        break;
    }

    // Roots: \sqrt[n]{x} -> root(n, x) and \sqrt{x} -> sqrt(x)
    while let Some(pos) = s.find(r"\sqrt[") {
        if let Some(c1) = s[pos + 6..].find(']') {
            let n = s[pos + 6..pos + 6 + c1].to_string();
            let rest = &s[pos + 6 + c1 + 1..];
            if rest.starts_with('{') {
                if let Some(end) = find_matching_brace(&s, pos + 6 + c1 + 1) {
                    let arg = s[pos + 6 + c1 + 2..end].to_string();
                    let arg_conv = convert_latex_math_to_typst(&arg);
                    s.replace_range(pos..end + 1, &format!("root({n}, {arg_conv})"));
                    continue;
                }
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
                let space_before = if pos > 0 && s[..pos].chars().last().map_or(false, |c| c.is_alphanumeric()) {
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

pub fn markdown_to_typst(
    markdown: &str,
    is_slides: bool,
    bibliography: bool,
    lang: &str,
    biblio_title: Option<&str>,
) -> String {
    let preprocessed = preprocess_pandoc(markdown);
    let markdown = &preprocessed;

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_MATH);
    options.insert(Options::ENABLE_SUBSCRIPT);
    options.insert(Options::ENABLE_SUPERSCRIPT);
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
                    text.push_str(&t);
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

    let parser = Parser::new_ext(markdown, options);
    let mut typst = String::with_capacity(markdown.len() * 2);

    let mut list_depth: usize = 0;
    let mut in_table_header = false;
    let mut table_cells: Vec<String> = Vec::new();
    let mut table_col_count = 0;
    let mut table_alignments: Vec<&'static str> = Vec::new();
    let mut current_cell = String::new();
    let mut in_table = false;
    let mut in_code_block = false;
    let mut in_footnote_def = false;
    let mut references: Vec<(String, String)> = Vec::new();
    let mut current_link: Option<(String, String)> = None;
    let mut current_image: Option<(String, String)> = None;

    for event in parser {
        match event {
            Event::Start(tag) => {
                if let Tag::FootnoteDefinition(_) = tag {
                    in_footnote_def = true;
                    continue;
                }
                if in_footnote_def {
                    continue;
                }
                match tag {
                    Tag::Paragraph => {}
                Tag::Heading { level, .. } => {
                    let prefix = match level {
                        HeadingLevel::H1 => "=",
                        HeadingLevel::H2 => "==",
                        HeadingLevel::H3 => "===",
                        HeadingLevel::H4 => "====",
                        HeadingLevel::H5 => "=====",
                        HeadingLevel::H6 => "======",
                    };
                    typst.push_str(prefix);
                    typst.push(' ');
                }
                Tag::BlockQuote(_) => {
                    typst.push_str("#quote[");
                }
                Tag::CodeBlock(kind) => {
                    in_code_block = true;
                    typst.push_str("```");
                    match kind {
                        pulldown_cmark::CodeBlockKind::Fenced(lang) => {
                            typst.push_str(&lang);
                        }
                        pulldown_cmark::CodeBlockKind::Indented => {}
                    }
                    typst.push('\n');
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
                    if in_table { current_cell.push('_'); } else { typst.push('_'); }
                }
                Tag::Strong => {
                    if in_table { current_cell.push('*'); } else { typst.push('*'); }
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
                    if bibliography && is_external {
                        current_link = Some((dest_url.to_string(), String::new()));
                    } else {
                        current_link = None;
                    }
                    let link_code = format!("#link(\"{dest_url}\")[");
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
                    continue;
                }
                if in_footnote_def {
                    continue;
                }
                match tag {
                    TagEnd::Paragraph => {
                        typst.push_str("\n\n");
                    }
                    TagEnd::Heading(_) => {
                        typst.push_str("\n\n");
                    }
                    TagEnd::BlockQuote(_) => {
                        typst.push_str("]\n\n");
                    }
                    TagEnd::CodeBlock => {
                        in_code_block = false;
                        if !typst.ends_with('\n') {
                            typst.push('\n');
                        }
                        typst.push_str("```\n\n");
                    }
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
                        if in_table { current_cell.push('_'); } else { typst.push('_'); }
                    }
                    TagEnd::Strong => {
                        if in_table { current_cell.push('*'); } else { typst.push('*'); }
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
                        if let Some((url, alt)) = current_image.take() {
                            let alt = alt.trim();
                            let escaped_url = url.replace('\\', "/").replace('"', "\\\"");
                            if in_table {
                                current_cell.push_str(&format!("#image(\"{escaped_url}\")"));
                            } else if current_link.is_some() {
                                typst.push_str(&format!("#image(\"{escaped_url}\")"));
                            } else if is_slides {
                                if alt == "bg" || alt.starts_with("bg ") {
                                    typst.push_str(&format!("\n#place(top + left, dx: 0pt, dy: 0pt, image(\"{escaped_url}\", width: 100%, height: 100%, fit: \"cover\"))\n\n"));
                                } else if !alt.is_empty() {
                                    typst.push_str(&format!("\n#figure(image(\"{escaped_url}\"), caption: [{alt}])\n\n"));
                                } else {
                                    typst.push_str(&format!("\n#align(center)[#image(\"{escaped_url}\")]\n\n"));
                                }
                            } else {
                                if !alt.is_empty() {
                                    typst.push_str(&format!("\n#figure(image(\"{escaped_url}\"), caption: [{alt}])\n\n"));
                                } else {
                                    typst.push_str(&format!("\n#align(center)[#image(\"{escaped_url}\")]\n\n"));
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
                    typst.push_str(&format!("#table(\n  columns: {cols},\n  align: ({align_str}),\n"));
                    if table_col_count > 0 && table_cells.len() >= table_col_count {
                        let header_slice = &table_cells[..table_col_count];
                        let header_args: Vec<String> = header_slice
                            .iter()
                            .map(|c| format!("[*{c}*]"))
                            .collect();
                        typst.push_str(&format!("  table.header({}),\n", header_args.join(", ")));
                        for cell in &table_cells[table_col_count..] {
                            typst.push_str(&format!("  [{cell}],\n"));
                        }
                    } else {
                        for cell in &table_cells {
                            typst.push_str(&format!("  [{cell}],\n"));
                        }
                    }
                    typst.push_str(")\n\n");
                }
                _ => {}
                }
            }
            Event::Text(text) => {
                if in_footnote_def {
                    continue;
                }
                if let Some((_, ref mut img_alt)) = current_image {
                    img_alt.push_str(&text);
                    continue;
                }
                if let Some((_, ref mut link_text)) = current_link {
                    link_text.push_str(&text);
                }
                if in_table {
                    current_cell.push_str(&text);
                } else if in_code_block {
                    typst.push_str(&text);
                } else {
                    let mut s = convert_sub_super(&text);
                    if s.contains(r"\newpage") || s.contains(r"\pagebreak") || s.contains(r"\clearpage") {
                        s = s.replace(r"\newpage", "\n#pagebreak()\n")
                             .replace(r"\pagebreak", "\n#pagebreak()\n")
                             .replace(r"\clearpage", "\n#pagebreak()\n");
                    }
                    if s.contains(r"\tableofcontents") {
                        s = s.replace(r"\tableofcontents", "\n#outline(depth: 3)\n");
                    }
                    if s.contains(r"\listoffigures") {
                        s = s.replace(r"\listoffigures", "\n#outline(target: figure.where(kind: image))\n");
                    }
                    if s.contains(r"\listoftables") {
                        s = s.replace(r"\listoftables", "\n#outline(target: figure.where(kind: table))\n");
                    }
                    typst.push_str(&s);
                }
            }
            Event::Html(html) => {
                if in_footnote_def {
                    continue;
                }
                if html.contains("pagebreak") || html.contains("page-break") || html.contains("newpage") {
                    typst.push_str("\n#pagebreak()\n\n");
                }
            }
            Event::FootnoteReference(name) => {
                if in_footnote_def {
                    continue;
                }
                let body = footnote_defs.get(name.as_ref()).cloned().unwrap_or_default();
                typst.push_str(&format!("#footnote[{body}]"));
            }
            Event::TaskListMarker(checked) => {
                if in_footnote_def {
                    continue;
                }
                if checked {
                    typst.push_str("[x] ");
                } else {
                    typst.push_str("[ ] ");
                }
            }
            Event::Code(code) => {
                if in_footnote_def {
                    continue;
                }
                if let Some((_, ref mut img_alt)) = current_image {
                    img_alt.push('`');
                    img_alt.push_str(&code);
                    img_alt.push('`');
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
                    continue;
                }
                let converted = convert_latex_math_to_typst(&math);
                let s = format!("\n$ {} $\n\n", converted.trim());
                if in_table {
                    current_cell.push_str(&s);
                } else {
                    typst.push_str(&s);
                }
            }
            Event::Rule => {
                if in_footnote_def {
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
                    continue;
                }
                if in_table {
                    current_cell.push(' ');
                } else {
                    typst.push_str("\\\n");
                }
            }
            _ => {}
        }
    }

    if bibliography && !references.is_empty() {
        let default_heading = match lang {
            "fr" => "Références",
            "de" => "Literaturverzeichnis",
            "es" => "Referencias",
            "it" => "Riferimenti bibliografici",
            _ => "References",
        };
        let heading = biblio_title.unwrap_or(default_heading);

        let (online_label, available_label) = match lang {
            "fr" => ("[En ligne]", "Disponible sur :"),
            "de" => ("[Online]", "Verfügbar unter:"),
            "es" => ("[En línea]", "Disponible en:"),
            "it" => ("[Online]", "Disponibile su:"),
            _ => ("[Online]", "Available:"),
        };

        typst.push_str("\n\n#v(2em)\n");
        typst.push_str(&format!("#heading(numbering: none)[{heading}] <references>\n\n"));
        typst.push_str("#set par(hanging-indent: 1.8em, justify: false)\n\n");

        for (i, (url, label)) in references.iter().enumerate() {
            let idx = i + 1;

            let is_url_label = label.is_empty()
                || label == url
                || label.starts_with("http://")
                || label.starts_with("https://");

            if is_url_label {
                typst.push_str(&format!("#block[\\[{idx}\\] {online_label}. {available_label} #link(\"{url}\").] <bib-{idx}>\n\n"));
            } else {
                typst.push_str(&format!("#block[\\[{idx}\\] \"{label}\", {online_label}. {available_label} #link(\"{url}\").] <bib-{idx}>\n\n"));
            }
        }
    }

    typst
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sub_super_and_strikethrough() {
        let md = "H~2~O and 10^6^ with ~~strike~~ and `code_with_~_and_^`";
        let typst = markdown_to_typst(md, false, false, "en", None);
        assert!(typst.contains("#sub[2]"));
        assert!(typst.contains("#super[6]"));
        assert!(typst.contains("#strike[strike]"));
        assert!(typst.contains("`code_with_~_and_^`"));
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
        let md = "Before\n\n\\newpage\n\nAfter";
        let typst = markdown_to_typst(md, false, false, "en", None);
        assert!(typst.contains("#pagebreak()"));
    }

    #[test]
    fn test_bibliography_link_transformation() {
        let md = "See [Rust](https://www.rust-lang.org) and [LLVM](https://llvm.org). Also [Rust Lang](https://www.rust-lang.org).";
        let typst = markdown_to_typst(md, false, true, "en", None);
        assert!(typst.contains("#link(<bib-1>)[\\"));
        assert!(typst.contains("#link(<bib-2>)[\\"));
        assert!(typst.contains("#heading(numbering: none)[References] <references>"));
        assert!(typst.contains("<bib-1>"));
        assert!(typst.contains("\\[1\\] \"Rust\", [Online]. Available: #link(\"https://www.rust-lang.org\")"));
        assert!(typst.contains("<bib-2>"));
        assert!(typst.contains("\\[2\\] \"LLVM\", [Online]. Available: #link(\"https://llvm.org\")"));
    }

    #[test]
    fn test_display_math_and_latex_superscript() {
        let md = "$$\\int_{-\\infty}^{+\\infty} e^{-x^2} \\, dx = \\sqrt{\\pi}$$\n\n$$\n\\sum_{k=0}^\\infty \\frac{1}{k!}\n$$";
        let typst = markdown_to_typst(md, false, false, "en", None);
        assert!(!typst.contains("#super"), "Math must not contain #super: {typst}");
        assert!(!typst.contains("#sub"), "Math must not contain #sub: {typst}");
        assert!(typst.contains("oo"), "Infinity should be oo: {typst}");
        assert!(typst.contains("integral_(-oo)^(+oo)"));
    }

    #[test]
    fn test_image_rendering() {
        let md = "![Architecture Pipeline](figures/arch.png)\n\n![](logo.svg)";
        let typst = markdown_to_typst(md, false, false, "en", None);
        assert!(typst.contains("#figure(image(\"figures/arch.png\"), caption: [Architecture Pipeline])"));
        assert!(typst.contains("#align(center)[#image(\"logo.svg\")]"));
    }
}
