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

    s = crate::markdown::pandoc::convert_cross_references(&s);

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
