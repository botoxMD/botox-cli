pub fn find_matching_brace(s: &str, open_pos: usize) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut depth = 0;
    for (i, c) in s[open_pos..].char_indices() {
        let abs = open_pos + i;
        if c == '{' {
            if abs > 0 && bytes[abs - 1] == b'\\' {
                // Escaped brace \{
                continue;
            }
            depth += 1;
        } else if c == '}' {
            if abs > 0 && bytes[abs - 1] == b'\\' {
                // Escaped brace \}
                continue;
            }
            depth -= 1;
            if depth == 0 {
                return Some(abs);
            }
        }
    }
    None
}

/// Unescape LaTeX text-mode escapes (`\_`, `\%`, `\&`, `\#`, `\$`, `\{`, `\}`) inside
/// `\text{...}`, `\mathrm{...}`, `\operatorname{...}`, etc. before emitting a Typst string literal.
fn unescape_latex_text(s: &str) -> String {
    s.replace(r"\_", "_")
        .replace(r"\%", "%")
        .replace(r"\&", "&")
        .replace(r"\#", "#")
        .replace(r"\$", "$")
        .replace(r"\{", "{")
        .replace(r"\}", "}")
}

/// Helper that replaces a LaTeX command while ensuring word boundaries so that
/// shorter command prefixes (e.g. `\le`, `\tan`, `\to`) do not corrupt longer
/// tokens (e.g. `\leftarrow`, `\tanh`, `\top`, `\tolerance`), and skips string literals.
fn replace_latex_command(s: &mut String, cmd: &str, replacement: &str) {
    let cmd_len = cmd.len();
    let ends_with_letter = cmd.chars().last().is_some_and(|c| c.is_ascii_alphabetic());
    let starts_with_letter = cmd.chars().next().is_some_and(|c| c.is_ascii_alphabetic());
    let repl_starts_alnum = replacement.chars().next().is_some_and(|c| c.is_ascii_alphanumeric());
    let repl_ends_alnum = replacement.chars().last().is_some_and(|c| c.is_ascii_alphanumeric());

    let mut idx = 0;
    let mut in_quote = false;

    while idx < s.len() {
        let ch = match s[idx..].chars().next() {
            Some(c) => c,
            None => break,
        };
        let ch_len = ch.len_utf8();

        if ch == '"' {
            in_quote = !in_quote;
            idx += ch_len;
            continue;
        }

        if in_quote {
            idx += ch_len;
            continue;
        }

        if s[idx..].starts_with(cmd) {
            let abs_pos = idx;
            let after_pos = abs_pos + cmd_len;

            // If the command starts with an ASCII letter, the preceding character must not be
            // an ASCII letter or backslash.
            if starts_with_letter
                && abs_pos > 0
                && let Some(prev_ch) = s[..abs_pos].chars().last()
                && (prev_ch.is_ascii_alphabetic() || prev_ch == '\\')
            {
                idx += ch_len;
                continue;
            }

            // If the command ends in an ASCII letter, the following character must not be
            // an ASCII letter (which would mean cmd is a prefix of a longer command name).
            if ends_with_letter
                && let Some(next_ch) = s[after_pos..].chars().next()
                && next_ch.is_ascii_alphabetic()
            {
                idx += ch_len;
                continue;
            }

            let need_leading_space = repl_starts_alnum
                && abs_pos > 0
                && s[..abs_pos].chars().last().is_some_and(|c| c.is_ascii_alphanumeric());
            let need_trailing_space = repl_ends_alnum
                && s[after_pos..].chars().next().is_some_and(|c| c.is_ascii_alphanumeric());

            let actual_repl = match (need_leading_space, need_trailing_space) {
                (true, true) => format!(" {replacement} "),
                (true, false) => format!(" {replacement}"),
                (false, true) => format!("{replacement} "),
                (false, false) => replacement.to_string(),
            };

            s.replace_range(abs_pos..after_pos, &actual_repl);
            idx = abs_pos + actual_repl.len();
        } else {
            idx += ch_len;
        }
    }
}

fn is_known_typst_math_symbol(s: &str) -> bool {
    matches!(
        s,
        // Functions / decorators
        "cal" | "bb" | "bold" | "italic" | "upright" | "mono" | "sans" | "frak" | "scr"
            | "op" | "root" | "sqrt" | "arrow" | "hat" | "macron" | "dot" | "ddot"
            | "tilde" | "overline" | "underline" | "overbrace" | "underbrace"
            | "mat" | "cases" | "binom" | "abs" | "norm" | "floor" | "ceil" | "round"
            | "cancel" | "display" | "inline" | "script" | "sscript" | "class" | "rect"
            // Differentials & calculus
            | "dif" | "diff" | "dx" | "dy" | "dz" | "dt" | "dr" | "du" | "dv"
            // Operators & constants
            | "sum" | "product" | "integral" | "oo" | "lim" | "det" | "max" | "min"
            | "sup" | "inf" | "log" | "ln" | "exp"
            | "sin" | "cos" | "tan" | "sinh" | "cosh" | "tanh"
            | "arcsin" | "arccos" | "arctan" | "arg" | "deg" | "dim" | "gcd" | "hom"
            | "ker" | "mod" | "sec" | "csc" | "cot"
            | "plus" | "minus" | "times" | "div" | "star" | "ast" | "circle" | "cross"
            | "equiv" | "approx" | "ne" | "neq" | "lt" | "gt" | "le" | "ge"
            | "in" | "notin" | "subset" | "supset" | "sect" | "inter" | "union" | "without"
            | "forall" | "exists" | "not" | "and" | "or" | "top" | "bot" | "parallel" | "perp"
            | "emptyset" | "prop" | "ell" | "aleph" | "wp" | "Re" | "Im" | "angle" | "prime"
            | "backslash" | "diamond" | "square" | "bullet" | "dagger"
            | "space" | "thin" | "med" | "thick" | "quad" | "qquad" | "wide"
            | "dots" | "partial" | "nabla" | "delim" | "comma" | "semi" | "colon"
            // Dotted symbols
            | "subset.eq" | "supset.eq" | "subset.neq" | "supset.neq"
            | "in.not" | "plus.minus" | "minus.plus" | "plus.circle" | "times.circle" | "minus.circle" | "dot.circle"
            | "dots.v" | "dots.down" | "dots.h" | "dots.c"
            | "floor.l" | "floor.r" | "ceil.l" | "ceil.r" | "chevron.l" | "chevron.r"
            | "tilde.equiv" | "tilde.eq" | "union.big" | "sect.big"
            | "integral.cont" | "integral.double" | "integral.triple"
            | "epsilon.alt" | "phi.alt" | "theta.alt" | "rho.alt" | "sigma.alt" | "pi.alt"
            | "planck.reduce" | "arrow.t" | "arrow.b" | "arrow.l" | "arrow.r" | "triangle.t" | "circle.small"
            // Greek letters (lowercase)
            | "alpha" | "beta" | "gamma" | "delta" | "epsilon" | "zeta" | "eta" | "theta"
            | "iota" | "kappa" | "lambda" | "mu" | "nu" | "xi" | "omicron" | "pi" | "rho"
            | "sigma" | "tau" | "upsilon" | "phi" | "chi" | "psi" | "omega"
            // Greek letters (uppercase)
            | "Gamma" | "Delta" | "Theta" | "Lambda" | "Xi" | "Pi" | "Sigma"
            | "Upsilon" | "Phi" | "Psi" | "Omega"
    )
}

/// Convert single-argument decorator function content so that top-level commas `,`
/// are rendered as ` comma `, preventing Typst from misinterpreting them as multiple arguments.
fn escape_commas_in_single_arg_func(s: &str) -> String {
    let mut res = String::with_capacity(s.len() + 8);
    let mut depth: usize = 0;
    let mut in_quotes = false;
    let chars: Vec<char> = s.chars().collect();
    let n = chars.len();
    let mut i = 0;

    while i < n {
        let ch = chars[i];
        if ch == '"' {
            in_quotes = !in_quotes;
            res.push(ch);
            i += 1;
        } else if in_quotes {
            res.push(ch);
            i += 1;
        } else {
            match ch {
                '(' | '[' | '{' => {
                    depth += 1;
                    res.push(ch);
                    i += 1;
                }
                ')' | ']' | '}' => {
                    depth = depth.saturating_sub(1);
                    res.push(ch);
                    i += 1;
                }
                ',' if depth == 0 => {
                    res.push_str(" comma ");
                    i += 1;
                    while i < n && chars[i] == ' ' {
                        i += 1;
                    }
                }
                _ => {
                    res.push(ch);
                    i += 1;
                }
            }
        }
    }
    res
}

/// Converts single-quoted string literals in LaTeX math expressions (`'John'`, `'Smith'`)
/// to double-quoted string literals (`"John"`, `"Smith"`), while preserving prime notation
/// (e.g. `f'`, `y''`, `(x+y)'`).
fn convert_latex_single_quoted_strings(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    let n = chars.len();
    let mut i = 0;

    while i < n {
        let ch = chars[i];
        if ch == '"' {
            result.push('"');
            i += 1;
            while i < n {
                result.push(chars[i]);
                if chars[i] == '"' {
                    i += 1;
                    break;
                }
                i += 1;
            }
            continue;
        }

        if ch == '\'' {
            let is_prime = if i > 0 {
                let prev = chars[i - 1];
                prev.is_alphanumeric() || prev == ')' || prev == ']' || prev == '}' || prev == '\''
            } else {
                false
            };

            if is_prime {
                result.push('\'');
                i += 1;
                continue;
            }

            let mut j = i + 1;
            let mut found_close = None;
            while j < n {
                if chars[j] == '\'' {
                    found_close = Some(j);
                    break;
                } else if chars[j] == '\n' || chars[j] == '"' {
                    break;
                }
                j += 1;
            }

            if let Some(close_idx) = found_close {
                let content: String = chars[i + 1..close_idx].iter().collect();
                if !content.is_empty() {
                    result.push('"');
                    result.push_str(&content);
                    result.push('"');
                    i = close_idx + 1;
                    continue;
                }
            }
        }

        result.push(ch);
        i += 1;
    }

    result
}

fn split_unknown_math_identifiers(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 16);
    let chars: Vec<char> = s.chars().collect();
    let n = chars.len();
    let mut i = 0;
    let mut in_quotes = false;
    let mut brace_depth: usize = 0;

    while i < n {
        let ch = chars[i];

        // Handle string quotes: "..."
        if ch == '"' {
            in_quotes = !in_quotes;
            result.push('"');
            i += 1;
            continue;
        }

        if in_quotes {
            result.push(ch);
            i += 1;
            continue;
        }

        // Handle LaTeX command if any remaining (e.g. \foo, \{, \}, \_)
        if ch == '\\' {
            result.push('\\');
            i += 1;
            if i < n && (chars[i] == '{' || chars[i] == '}' || chars[i] == '_') {
                result.push(chars[i]);
                i += 1;
                continue;
            }
            while i < n && chars[i].is_ascii_alphabetic() {
                result.push(chars[i]);
                i += 1;
            }
            continue;
        }

        // Track braces { ... }
        if ch == '{' {
            brace_depth += 1;
            result.push('{');
            i += 1;
            continue;
        } else if ch == '}' {
            brace_depth = brace_depth.saturating_sub(1);
            result.push('}');
            i += 1;
            continue;
        }

        // Skip Typst code expressions in math mode (e.g. #box(...)[...], #rect(...), etc.)
        if ch == '#' {
            result.push('#');
            i += 1;
            while i < n && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                result.push(chars[i]);
                i += 1;
            }
            if i < n && chars[i] == '(' {
                let mut depth = 0;
                while i < n {
                    result.push(chars[i]);
                    if chars[i] == '(' {
                        depth += 1;
                    } else if chars[i] == ')' {
                        depth -= 1;
                        if depth == 0 {
                            i += 1;
                            break;
                        }
                    }
                    i += 1;
                }
            }
            if i < n && chars[i] == '[' {
                let mut depth = 0;
                while i < n {
                    result.push(chars[i]);
                    if chars[i] == '[' {
                        depth += 1;
                    } else if chars[i] == ']' {
                        depth -= 1;
                        if depth == 0 {
                            i += 1;
                            break;
                        }
                    }
                    i += 1;
                }
            }
            continue;
        }

        // If inside braces, don't split identifiers on outer pass (handled in sub/super or command args)
        if brace_depth > 0 {
            result.push(ch);
            i += 1;
            continue;
        }

        // Check for ASCII alphabetic word
        if ch.is_ascii_alphabetic() {
            let start = i;
            while i < n && chars[i].is_ascii_alphabetic() {
                i += 1;
            }
            let word_end = i;

            // Check if followed by '.' and another word (e.g. subset.eq, floor.l, dots.v)
            if i < n && chars[i] == '.' {
                let dot_pos = i;
                let mut after_dot = i + 1;
                while after_dot < n && chars[after_dot].is_ascii_alphabetic() {
                    after_dot += 1;
                }
                if after_dot > dot_pos + 1 {
                    let compound: String = chars[start..after_dot].iter().collect();
                    if is_known_typst_math_symbol(&compound) {
                        result.push_str(&compound);
                        i = after_dot;
                        continue;
                    }
                }
            }

            let word: String = chars[start..word_end].iter().collect();
            let next_is_digit = i < n && chars[i].is_ascii_digit();
            let prev_is_digit = start > 0 && chars[start - 1].is_ascii_digit();

            if prev_is_digit {
                result.push(' ');
            }

            if is_known_typst_math_symbol(&word) {
                result.push_str(&word);
                if next_is_digit {
                    result.push(' ');
                }
            } else if word.len() == 1 {
                result.push_str(&word);
                if next_is_digit {
                    result.push(' ');
                }
            } else {
                // Unknown multi-letter identifier: split into spaced individual characters
                // e.g. "pk" -> "p k", "attr" -> "a t t r", "type" -> "t y p e", "dept" -> "d e p t"
                let spaced = word
                    .chars()
                    .map(|c| c.to_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                result.push_str(&spaced);
                if next_is_digit {
                    result.push(' ');
                }
            }
        } else {
            result.push(ch);
            i += 1;
        }
    }

    result
}

pub const TYPST_MATH_PREAMBLE: &str = r#"
// Botox LaTeX Math Bridge Shims
#let frac(num, den) = $(num)/(den)$
#let sect = math.inter
#let cfrac(num, den) = $display((num)/(den))$
#let dfrac(num, den) = $display((num)/(den))$
#let tfrac(num, den) = $inline((num)/(den))$
#let stackrel(sup, base) = $limits(base)^(sup)$
#let overset(sup, base) = $limits(base)^(sup)$
#let underset(sub, base) = $limits(base)_(sub)$
#let boxed(it) = box(stroke: 0.5pt, inset: (x: 8pt, y: 6pt), it)
#let underline(..args) = {
  let items = args.pos()
  let joined = ()
  for (i, it) in items.enumerate() {
    if i > 0 { joined.push(math.comma); joined.push([ ]) }
    joined.push(it)
  }
  math.underline(joined.join())
}
#let pmatrix(..args) = math.mat(delim: "(", ..args)
#let bmatrix(..args) = math.mat(delim: "[", ..args)
#let Bmatrix(..args) = math.mat(delim: "{", ..args)
#let vmatrix(..args) = math.mat(delim: "|", ..args)
#let Vmatrix(..args) = math.mat(delim: "||", ..args)
#let matrix(..args) = math.mat(delim: none, ..args)
#let aligned(it) = it
#let alignedat(..args) = args.pos().last()
#let textmath(it) = it
#let mitexmathbf(it) = math.bold(math.upright(it))
#let mitexsqrt(..args) = {
  if args.pos().len() == 1 {
    math.sqrt(args.pos().at(0))
  } else if args.pos().len() == 2 {
    math.root(args.pos().at(0), args.pos().at(1))
  }
}
"#;

fn normalize_mitex_ast(ast: &mut String) {
    if ast.starts_with("aligned(") && ast.ends_with(')') {
        let inner = &ast[8..ast.len() - 1];
        *ast = inner.trim().to_string();
    }
    *ast = ast
        .replace("mitexsqrt(", "sqrt(")
        .replace("mitexmathbf(", "bold(")
        .replace("sect ", "inter ")
        .replace("sect)", "inter)")
        .replace("sect,", "inter,")
        .replace("pmatrix(", "mat(delim: \"(\", ")
        .replace("bmatrix(", "mat(delim: \"[\", ")
        .replace("vmatrix(", "mat(delim: \"|\", ")
        .replace("Vmatrix(", "mat(delim: \"||\", ")
        .replace("Bmatrix(", "mat(delim: \"{\", ")
        .replace("matrix(", "mat(delim: #none, ");
}

/// Convert LaTeX math syntax to Typst math syntax.
/// Uses an AST-based parser (`mitex`) as the primary bridge, with normalization
/// into native Typst math and fallback to legacy converter if needed.
pub fn convert_latex_math_to_typst(math: &str) -> String {
    let trimmed = math.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    if matches!(trimmed, "dt" | "dx" | "dy" | "dz" | "dr" | "du" | "dv") {
        return format!("dif {}", &trimmed[1..]);
    }

    let preprocessed = convert_latex_single_quoted_strings(trimmed);

    // Try AST-based LaTeX parser first
    if let Ok(mut ast) = mitex::convert_math(&preprocessed, None) {
        normalize_mitex_ast(&mut ast);
        return ast.trim().to_string();
    }

    // Fallback to legacy regex pipeline
    convert_latex_math_legacy(trimmed)
}

pub fn convert_latex_math_legacy(math: &str) -> String {
    let mut s = math.trim().to_string();

    if matches!(s.as_str(), "dt" | "dx" | "dy" | "dz" | "dr" | "du" | "dv") {
        return format!("dif {}", &s[1..]);
    }

    s = convert_latex_single_quoted_strings(&s);

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
    for (from, to) in [
        (r"\mathrm{d}x", "dif x"),
        (r"\mathrm{d}y", "dif y"),
        (r"\mathrm{d}t", "dif t"),
        (r"\mathrm{d}z", "dif z"),
        (r"\text{d}x", "dif x"),
        (r"\text{d}y", "dif y"),
        (r"\text{d}t", "dif t"),
        (r"\text{d}z", "dif z"),
    ] {
        replace_latex_command(&mut s, from, to);
    }

    while let Some(pos) = s.find(r"\mathit{") {
        if let Some(end) = find_matching_brace(&s, pos + 7) {
            let inner = &s[pos + 8..end];
            let inner_conv = convert_latex_math_to_typst(inner);
            s.replace_range(pos..end + 1, &inner_conv);
        } else {
            break;
        }
    }
    while let Some(pos) = s.find(r"\mathrm{") {
        if let Some(end) = find_matching_brace(&s, pos + 7) {
            let inner = unescape_latex_text(&s[pos + 8..end]);
            s.replace_range(pos..end + 1, &format!("upright(\"{inner}\")"));
        } else {
            break;
        }
    }
    while let Some(pos) = s.find(r"\operatorname{") {
        if let Some(end) = find_matching_brace(&s, pos + 13) {
            let inner = unescape_latex_text(&s[pos + 14..end]);
            s.replace_range(pos..end + 1, &format!("op(\"{inner}\")"));
        } else {
            break;
        }
    }
    for (cmd, wrapper) in [
        (r"\text{", ""),
        (r"\textbf{", "bold"),
        (r"\textit{", "italic"),
        (r"\texttt{", "mono"),
        (r"\mathtt{", "mono"),
        (r"\mathsf{", "sans"),
    ] {
        let brace_offset = cmd.len() - 1;
        while let Some(pos) = s.find(cmd) {
            if let Some(end) = find_matching_brace(&s, pos + brace_offset) {
                let inner = unescape_latex_text(&s[pos + cmd.len()..end]);
                let repl = if wrapper.is_empty() {
                    format!("\"{inner}\"")
                } else {
                    format!("{wrapper}(\"{inner}\")")
                };
                s.replace_range(pos..end + 1, &repl);
            } else {
                break;
            }
        }
    }

    // Fractions: \frac{a}{b}, \dfrac{a}{b}, \tfrac{a}{b} -> (a) / (b)
    for frac_cmd in [r"\frac{", r"\dfrac{", r"\tfrac{"] {
        let brace_offset = frac_cmd.len() - 1;
        while let Some(pos) = s.find(frac_cmd) {
            if let Some(end1) = find_matching_brace(&s, pos + brace_offset) {
                let num = s[pos + frac_cmd.len()..end1].to_string();
                let rest = &s[end1 + 1..];
                if rest.starts_with('{')
                    && let Some(end2) = find_matching_brace(&s, end1 + 1)
                {
                    let den = s[end1 + 2..end2].to_string();
                    let num_conv = convert_latex_math_to_typst(&num);
                    let den_conv = convert_latex_math_to_typst(&den);
                    s.replace_range(pos..end2 + 1, &format!("({num_conv}) / ({den_conv})"));
                    continue;
                }
            }
            break;
        }
    }

    // Binomial: \binom{a}{b} -> binom(a, b)
    while let Some(pos) = s.find(r"\binom{") {
        if let Some(end1) = find_matching_brace(&s, pos + 6) {
            let n = s[pos + 7..end1].to_string();
            let rest = &s[end1 + 1..];
            if rest.starts_with('{')
                && let Some(end2) = find_matching_brace(&s, end1 + 1)
            {
                let k = s[end1 + 2..end2].to_string();
                let n_conv = convert_latex_math_to_typst(&n);
                let k_conv = convert_latex_math_to_typst(&k);
                s.replace_range(pos..end2 + 1, &format!("binom({n_conv}, {k_conv})"));
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
        (r"\overrightarrow{", "arrow"),
        (r"\hat{", "hat"),
        (r"\widehat{", "hat"),
        (r"\bar{", "macron"),
        (r"\dot{", "dot"),
        (r"\ddot{", "ddot"),
        (r"\tilde{", "tilde"),
        (r"\widetilde{", "tilde"),
        (r"\overline{", "overline"),
        (r"\underline{", "underline"),
        (r"\overbrace{", "overbrace"),
        (r"\underbrace{", "underbrace"),
        (r"\cancel{", "cancel"),
    ] {
        while let Some(pos) = s.find(cmd) {
            if let Some(end) = find_matching_brace(&s, pos + cmd.len() - 1) {
                let inner = s[pos + cmd.len()..end].to_string();
                let mut inner_conv = convert_latex_math_to_typst(&inner);
                if matches!(func, "underline" | "overline" | "cancel") {
                    inner_conv = escape_commas_in_single_arg_func(&inner_conv);
                }
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

    // Boxed expressions \boxed{...} -> #box(stroke: 0.5pt, inset: (x: 8pt, y: 6pt))[$...$]
    // Must be tight without spaces directly inside [$...$] so Typst treats it as an inline
    // equation (block: false) and does NOT expand to 100% width or spawn colliding equation numbers.
    while let Some(pos) = s.find(r"\boxed{") {
        if let Some(end) = find_matching_brace(&s, pos + 6) {
            let inner = s[pos + 7..end].to_string();
            let inner_conv = convert_latex_math_to_typst(&inner);
            let inner_trimmed = inner_conv.trim();
            let space_before = if pos > 0 && s[..pos].chars().last().is_some_and(|c| c.is_alphanumeric()) {
                " "
            } else {
                ""
            };
            let space_after = if end + 1 < s.len() && s[end + 1..].chars().next().is_some_and(|c| c.is_alphanumeric()) {
                " "
            } else {
                ""
            };
            s.replace_range(
                pos..end + 1,
                &format!("{space_before}#box(stroke: 0.5pt, inset: (x: 8pt, y: 6pt))[${inner_trimmed}$]{space_after}"),
            );
        } else {
            break;
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

    // Stripping scaling delimiters \left, \right, \big, \Big, \bigg, \Bigg
    s = s.replace(r"\left.", "").replace(r"\right.", "");
    for delim_mod in [r"\left", r"\right", r"\Bigg", r"\bigg", r"\Big", r"\big"] {
        replace_latex_command(&mut s, delim_mod, "");
    }

    // Operators and symbols
    let mut replacements = [
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
        (r"\top", "top"),
        (r"\bot", "bot"),
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
        (r"\cap", "inter"),
        (r"\bigcap", "inter.big"),
        (r"\cup", "union"),
        (r"\bigcup", "union.big"),
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
        (r"\sec", "sec"),
        (r"\csc", "csc"),
        (r"\cot", "cot"),
        (r"\arcsin", "arcsin"),
        (r"\arccos", "arccos"),
        (r"\arctan", "arctan"),
        (r"\deg", "deg"),
        (r"\dim", "dim"),
        (r"\gcd", "gcd"),
        (r"\hom", "hom"),
        (r"\ker", "ker"),
        (r"\arg", "arg"),
        (r"\wedge", "and"),
        (r"\land", "and"),
        (r"\vee", "or"),
        (r"\lor", "or"),
        (r"\neg", "not"),
        (r"\lfloor", "floor.l"),
        (r"\rfloor", "floor.r"),
        (r"\lceil", "ceil.l"),
        (r"\rceil", "ceil.r"),
        (r"\bmod", "mod"),
        (r"\pmod", "mod"),
        (r"\;", "thick"),
        (r"\:", "med"),
        (r"\ ", "space"),
        (r"\, dx", " dif x"),
        (r"\, dy", " dif y"),
        (r"\, dt", " dif t"),
        (r"\, dz", " dif z"),
        (r"\, dr", " dif r"),
        (r"\, du", " dif u"),
        (r"\, dv", " dif v"),
        // Greek letters
        (r"\alpha", "alpha"),
        (r"\beta", "beta"),
        (r"\gamma", "gamma"),
        (r"\delta", "delta"),
        (r"\varepsilon", "epsilon.alt"),
        (r"\epsilon", "epsilon"),
        (r"\zeta", "zeta"),
        (r"\eta", "eta"),
        (r"\vartheta", "theta.alt"),
        (r"\theta", "theta"),
        (r"\iota", "iota"),
        (r"\kappa", "kappa"),
        (r"\lambda", "lambda"),
        (r"\mu", "mu"),
        (r"\nu", "nu"),
        (r"\xi", "xi"),
        (r"\varpi", "pi.alt"),
        (r"\pi", "pi"),
        (r"\varrho", "rho.alt"),
        (r"\rho", "rho"),
        (r"\varsigma", "sigma.alt"),
        (r"\sigma", "sigma"),
        (r"\tau", "tau"),
        (r"\upsilon", "upsilon"),
        (r"\varphi", "phi.alt"),
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
        (r"\Upsilon", "Upsilon"),
        (r"\Phi", "Phi"),
        (r"\Psi", "Psi"),
        (r"\Omega", "Omega"),
    ];

    // Sort replacements by descending pattern length so that longer, more specific
    // commands (e.g. \leftarrow, \tanh, \subseteq) are matched before shorter prefixes.
    replacements.sort_by_key(|a| std::cmp::Reverse(a.0.len()));

    for (from, to) in replacements {
        replace_latex_command(&mut s, from, to);
    }

    s = split_unknown_math_identifiers(&s);

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
                | "out" | "air" | "net" | "avg" | "pot" | "kin" | "eff" | "ref" | "col" | "row"
                | "mid" | "top" | "bot" | "len" | "val" | "key" | "err" | "pos" | "src" | "dst"
                | "in" | "on" | "th" | "eq" | "ex" | "id" | "up" | "bg" | "op"
                | "total"
        );

        if is_known || inner.chars().all(|c| c.is_ascii_digit()) {
            inner.to_string()
        } else {
            split_unknown_math_identifiers(inner)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_boundary_and_shadowing() {
        assert_eq!(convert_latex_math_to_typst(r"\leftarrow"), "<-");
        assert_eq!(convert_latex_math_to_typst(r"\le"), "<=");
        assert_eq!(convert_latex_math_to_typst(r"\tanh"), "tanh");
        assert_eq!(convert_latex_math_to_typst(r"\tan"), "tan");
        assert_eq!(convert_latex_math_to_typst(r"\sinh"), "sinh");
        assert_eq!(convert_latex_math_to_typst(r"\sin"), "sin");
        assert_eq!(convert_latex_math_to_typst(r"\cosh"), "cosh");
        assert_eq!(convert_latex_math_to_typst(r"\cos"), "cos");
        assert_eq!(convert_latex_math_to_typst(r"\subseteq"), "subset.eq");
        assert_eq!(convert_latex_math_to_typst(r"\subset"), "subset");
        assert_eq!(convert_latex_math_to_typst(r"\cdots"), "dots.h.c");
        assert_eq!(convert_latex_math_to_typst(r"\cdot"), "dot.c");
        assert_eq!(convert_latex_math_to_typst(r"\neq"), "!=");
        assert_eq!(convert_latex_math_to_typst(r"\ne"), "!=");
        assert_eq!(convert_latex_math_to_typst(r"\top"), "top");
    }

    #[test]
    fn test_subscripts_descriptive_names() {
        assert_eq!(convert_latex_math_to_typst(r"x_{total}"), "x _(t o t a l )");
        assert_eq!(convert_latex_math_to_typst(r"a_{11}"), "a _(1 1 )");
        assert_eq!(convert_latex_math_to_typst(r"x_{ij}"), "x _(i j )");
        assert_eq!(convert_latex_math_to_typst(r"V_{out}"), "V _(o u t )");
        assert_eq!(convert_latex_math_to_typst(r"T_{air}"), "T _(a i r )");
        assert_eq!(convert_latex_math_to_typst(r"P_{net}"), "P _(n e t )");
        assert_eq!(convert_latex_math_to_typst(r"y + dx"), "y  +  d x");
        assert_eq!(convert_latex_math_to_typst(r"\int f(x)\, dx"), "integral  f \\(x \\)thin  d x");
    }

    #[test]
    fn test_escaped_braces() {
        assert_eq!(convert_latex_math_to_typst(r"\frac{\{x\}}{y}"), r"frac(\{ x \} ,y )");
    }

    #[test]
    fn test_multi_letter_identifiers_split() {
        assert_eq!(
            convert_latex_math_to_typst(r"\mathbf{Relation}(\underline{pk_1}, \dots, attr, fk, \dots)"),
            "bold(R e l a t i o n )\\(underline(p k _(1 ))\\, dots.h \\, a t t r \\, f k \\, dots.h \\)"
        );
        assert_eq!(
            convert_latex_math_to_typst(r"A(\underline{a_1}, a_2, b_1, c_1, type)"),
            "A \\(underline(a _(1 ))\\, a _(2 )\\, b _(1 )\\, c _(1 )\\, t y p e \\)"
        );
        assert_eq!(convert_latex_math_to_typst("pk_1"), "p k _(1 )");
        assert_eq!(convert_latex_math_to_typst("E = mc^2"), "E  =  m c ^(2 )");
        assert_eq!(convert_latex_math_to_typst(r"\mathit{pk_1}"), "italic(p k _(1 ))");
    }

    #[test]
    fn test_feedback_report_math_fixes() {
        // UTF-8 inside \text{...}
        assert_eq!(
            convert_latex_math_to_typst(r"\frac{\text{observé}}{\text{attendu}}"),
            "frac(#textmath[observé];,#textmath[attendu];)"
        );

        // Thin space \, with adjacent digits
        assert_eq!(
            convert_latex_math_to_typst(r"1\,000\,000"),
            "1 thin 0 0 0 thin 0 0 0"
        );

        // Backslash-space \ in math mode
        assert_eq!(
            convert_latex_math_to_typst(r"\approx 20 \text{ to } 100\ \mu\text{s}"),
            "approx  2 0  #textmath[ to ]; 1 0 0 thick  mu #textmath[s];"
        );

        // \lfloor, \rfloor, \lceil, \rceil, \bmod
        assert_eq!(
            convert_latex_math_to_typst(r"\lfloor N / K \rfloor"),
            "⌊  N  \\/ K  ⌋"
        );
        assert_eq!(
            convert_latex_math_to_typst(r"\lceil N / K \rceil"),
            "⌈  N  \\/ K  ⌉"
        );
        assert_eq!(convert_latex_math_to_typst(r"N \bmod K"), "N  mod  K");

        // Escaped underscore \_ inside \text{...}
        assert_eq!(
            convert_latex_math_to_typst(r"K = \min(\text{NB\_THREADS}, N)"),
            "K  =  min \\(#textmath[NB\\_ THREADS];\\, N \\)"
        );
    }

    #[test]
    fn test_relational_algebra_math() {
        // Underline with composite keys containing commas
        assert_eq!(
            convert_latex_math_to_typst(r"\underline{dnumber, dlocation}"),
            "underline(d n u m b e r \\, d l o c a t i o n )"
        );

        // Boxed expressions
        assert_eq!(
            convert_latex_math_to_typst(r"\boxed{R \leftarrow ssn\_dept5 \cup superviseurs}"),
            "boxed(R  <-  s s n \\_ d e p t 5  union  s u p e r v i s e u r s )"
        );

        // Single-quoted strings in conditions
        assert_eq!(
            convert_latex_math_to_typst(r"\sigma_{fname='John' \;\wedge\; lname='Smith'}(employee)"),
            "sigma _(f n a m e = \\\"J o h n \\\" thick and thick  l n a m e = \\\"S m i t h \\\")\\(e m p l o y e e \\)"
        );

        // Subscripts with conditions and projections
        assert_eq!(
            convert_latex_math_to_typst(r"\sigma_{dno=5}(employee)"),
            "sigma _(d n o = 5 )\\(e m p l o y e e \\)"
        );
        assert_eq!(
            convert_latex_math_to_typst(r"\pi_{essn, pno}(works\_on)"),
            "pi _(e s s n \\, p n o )\\(w o r k s \\_ o n \\)"
        );
    }
}
