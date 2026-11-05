pub fn escape_typst_text(s: &str) -> String {
    if !s.bytes().any(|b| matches!(b, b'#' | b']' | b'[' | b'~' | b'^' | b'@' | b'/' | b'\\' | b'*' | b'_' | b'$' | b'<' | b'>' | b'=' | b'-' | b'+')) {
        return s.to_string();
    }

    let s = crate::markdown::pandoc::convert_cross_references(s);

    const LATEX_COMMANDS: &[(&str, &str)] = &[
        (r"\newpage landscape", "\n#pagebreak(weak: true)\n#set page(flipped: true)\n"),
        (r"\pagebreak landscape", "\n#pagebreak(weak: true)\n#set page(flipped: true)\n"),
        (r"\clearpage landscape", "\n#pagebreak(weak: true)\n#set page(flipped: true)\n"),
        (r"\newpage{landscape}", "\n#pagebreak(weak: true)\n#set page(flipped: true)\n"),
        (r"\pagebreak{landscape}", "\n#pagebreak(weak: true)\n#set page(flipped: true)\n"),
        (r"\clearpage{landscape}", "\n#pagebreak(weak: true)\n#set page(flipped: true)\n"),
        (r"\newpage portrait", "\n#pagebreak(weak: true)\n#set page(flipped: false)\n"),
        (r"\pagebreak portrait", "\n#pagebreak(weak: true)\n#set page(flipped: false)\n"),
        (r"\clearpage portrait", "\n#pagebreak(weak: true)\n#set page(flipped: false)\n"),
        (r"\newpage{portrait}", "\n#pagebreak(weak: true)\n#set page(flipped: false)\n"),
        (r"\pagebreak{portrait}", "\n#pagebreak(weak: true)\n#set page(flipped: false)\n"),
        (r"\clearpage{portrait}", "\n#pagebreak(weak: true)\n#set page(flipped: false)\n"),
        (r"\newpage", "\n#pagebreak()\n"),
        (r"\pagebreak", "\n#pagebreak()\n"),
        (r"\clearpage", "\n#pagebreak()\n"),
        (r"\columnbreak", "\n#colbreak()\n"),
        (r"\colbreak", "\n#colbreak()\n"),
        (r"\tableofcontents", "\n#outline(depth: 3)\n"),
        (r"\listoffigures", "\n#outline(target: figure.where(kind: image))\n"),
        (r"\listoftables", "\n#outline(target: figure.where(kind: table))\n"),
        (r"\today", "#datetime.today().display(\"[day] [month repr:long] [year]\")"),
        (r"\pause", "\n#botox_pause()\n"),
    ];

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
                        out.push_str("#sub[");
                        out.push_str(&escaped_sub);
                        out.push(']');
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
                        out.push_str("#super[");
                        out.push_str(&escaped_sup);
                        out.push(']');
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

        // Check for backslash and LaTeX commands
        if c == '\\' {
            let rest: String = chars[i..].iter().take(22).collect();
            let mut matched = false;
            for (cmd, typst_replacement) in LATEX_COMMANDS {
                if rest.starts_with(cmd) {
                    out.push_str(typst_replacement);
                    i += cmd.chars().count();
                    matched = true;
                    break;
                }
            }
            if matched {
                continue;
            }
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

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_typst_commands_preservation() {
        let text = "#sub[H2O] and #super[100] and #strike[deleted] and #pagebreak() and #colbreak() and #outline(depth: 2)";
        let escaped = escape_typst_text(text);
        assert!(escaped.contains("#sub[H2O]"));
        assert!(escaped.contains("#super[100]"));
        assert!(escaped.contains("#strike[deleted]"));
        assert!(escaped.contains("#pagebreak()"));
        assert!(escaped.contains("#colbreak()"));
        assert!(escaped.contains("#outline(depth: 2)"));

        let hashtags = "Check #123 and C# programming";
        let esc_hash = escape_typst_text(hashtags);
        assert!(esc_hash.contains(r"\#123"));
        assert!(esc_hash.contains(r"C\#"));
    }

    #[test]
    fn test_subscript_and_superscript_escaping() {
        let text = "Formula H~2~O and value 2^10^ alongside ~~strike~~";
        let escaped = escape_typst_text(text);
        assert!(escaped.contains("H#sub[2]O"));
        assert!(escaped.contains("2#super[10]"));
        assert!(escaped.contains(r"\~\~strike\~\~"));
    }

    #[test]
    fn test_bracket_balancing() {
        let text = "Array arr[0] and lone bracket ] here";
        let escaped = escape_typst_text(text);
        assert!(escaped.contains(r"arr\[0\]"));
        assert!(escaped.contains(r"\] here"));
    }

    #[test]
    fn test_escape_cross_references() {
        let text = "See @fig-1 and @tbl-2 vs email user@domain.com";
        let escaped = escape_typst_text(text);
        assert!(escaped.contains("@fig-1"));
        assert!(escaped.contains("@tbl-2"));
        assert!(escaped.contains(r"user\@domain.com"));
    }

    #[test]
    fn test_escape_comments_and_slashes() {
        let text = "// comment\n/* block */\nNormal a/b slash\n/ Description term";
        let escaped = escape_typst_text(text);
        assert!(escaped.contains(r"\// comment"));
        assert!(escaped.contains(r"/\* block"));
        assert!(escaped.contains("Normal a/b slash"));
        assert!(escaped.contains(r"\/ Description term"));
    }

    #[test]
    fn test_landscape_and_portrait_pagebreaks() {
        let text_land1 = "Start\\newpage landscapeEnd";
        assert!(escape_typst_text(text_land1).contains("#pagebreak(weak: true)\n#set page(flipped: true)"));

        let text_land2 = "Start\\pagebreak{landscape}End";
        assert!(escape_typst_text(text_land2).contains("#pagebreak(weak: true)\n#set page(flipped: true)"));

        let text_port = "Start\\clearpage portraitEnd";
        assert!(escape_typst_text(text_port).contains("#pagebreak(weak: true)\n#set page(flipped: false)"));

        let text_page = "Before\\pagebreak After";
        assert!(escape_typst_text(text_page).contains("\n#pagebreak()\n"));
    }

    #[test]
    fn test_breaks_and_helpers() {
        let text_col = "Col1\\columnbreak Col2\\colbreak Col3";
        let esc_col = escape_typst_text(text_col);
        assert!(esc_col.contains("\n#colbreak()\n"));

        let text_outlines = "\\tableofcontents\n\\listoffigures\n\\listoftables";
        let esc_outlines = escape_typst_text(text_outlines);
        assert!(esc_outlines.contains("\n#outline(depth: 3)\n"));
        assert!(esc_outlines.contains("#outline(target: figure.where(kind: image))"));
        assert!(esc_outlines.contains("#outline(target: figure.where(kind: table))"));

        let text_today = "Today is \\today";
        assert!(escape_typst_text(text_today).contains("#datetime.today().display("));

        let text_pause = "Slide 1\\pause Slide 2";
        assert!(escape_typst_text(text_pause).contains("\n#botox_pause()\n"));
    }

    #[test]
    fn test_line_start_markers() {
        let text = "= Title\n- Bullet\n+ Numbered";
        let escaped = escape_typst_text(text);
        assert!(escaped.starts_with(r"\= Title"));
        assert!(escaped.contains(r"\- Bullet"));
        assert!(escaped.contains(r"\+ Numbered"));
    }
}
