pub mod columns;
pub mod metadata;
pub mod themes;

use crate::config::{DocumentConfig, MarginConfig};
use serde_yaml::Value;

pub use columns::sanitize_column_pagebreaks;
pub use metadata::{
    build_document_metadata, extract_authors, extract_date, extract_title,
};
pub use themes::DocumentTheme;

fn render_date_value(d: &str) -> String {
    let trimmed = d.trim();
    if trimmed == r"\today" || trimmed.eq_ignore_ascii_case("today") {
        "#datetime.today().display(\"[day] [month repr:long] [year]\")".to_string()
    } else {
        trimmed.to_string()
    }
}

#[allow(clippy::too_many_arguments)]
fn render_title_and_abstract(
    doc_theme: DocumentTheme,
    title: Option<&str>,
    subtitle: Option<&str>,
    authors: &[String],
    affiliation: Option<&str>,
    date_str: Option<&str>,
    abstract_text: Option<&str>,
    keywords_str: Option<&str>,
) -> String {
    if title.is_none() && authors.is_empty() && abstract_text.is_none() {
        return String::new();
    }

    let mut b = String::new();

    match doc_theme {
        DocumentTheme::Modern => {
            if title.is_some() || !authors.is_empty() {
                b.push_str("#block(\n  width: 100%,\n  stroke: (left: 4.5pt + rgb(\"#2563eb\")),\n  inset: (left: 14pt, top: 4pt, bottom: 4pt),\n)[\n  #set par(justify: false)\n");
                if let Some(t) = title {
                    b.push_str(&format!("  #text(size: 2em, weight: \"bold\", fill: rgb(\"#0f172a\"))[{t}]\n"));
                }
                if let Some(sub) = subtitle {
                    b.push_str(&format!("  #v(0.3em)\n  #text(size: 1.15em, fill: rgb(\"#475569\"))[{sub}]\n"));
                }
                if !authors.is_empty() || affiliation.is_some() || date_str.is_some() {
                    b.push_str("  #v(0.5em)\n");
                    if !authors.is_empty() {
                        let joined = authors.join(", ");
                        b.push_str(&format!("  #text(size: 1.05em, weight: \"bold\", fill: rgb(\"#334155\"))[{joined}]\n"));
                    }
                    if let Some(aff) = affiliation
                        && !aff.is_empty() {
                            b.push_str(&format!("  #v(0.2em)\n  #text(size: 0.95em, fill: rgb(\"#64748b\"))[{aff}]\n"));
                        }
                    if let Some(d) = date_str {
                        let date_typst = render_date_value(d);
                        b.push_str(&format!("  #v(0.2em)\n  #text(size: 0.9em, fill: rgb(\"#64748b\"))[{date_typst}]\n"));
                    }
                }
                b.push_str("]\n\n#v(1.4em)\n");
            }
            if abstract_text.is_some() || keywords_str.is_some() {
                b.push_str("#block(\n  fill: rgb(\"#f8fafc\"),\n  stroke: 0.5pt + rgb(\"#e2e8f0\"),\n  inset: (x: 14pt, y: 12pt),\n  radius: 4pt,\n  width: 100%,\n)[\n");
                b.push_str("  #set par(justify: true, leading: 0.65em)\n");
                if let Some(abs) = abstract_text {
                    b.push_str(&format!("  #text(weight: \"bold\", fill: rgb(\"#0f172a\"))[Abstract] -- #text(fill: rgb(\"#334155\"))[{abs}]\n"));
                }
                if let Some(kw) = keywords_str {
                    if abstract_text.is_some() {
                        b.push_str("  #v(0.5em)\n");
                    }
                    b.push_str(&format!("  #text(weight: \"bold\", fill: rgb(\"#0f172a\"))[Keywords] -- #text(fill: rgb(\"#475569\"))[{kw}]\n"));
                }
                b.push_str("]\n\n#v(1.4em)\n");
            }
        }
        DocumentTheme::Technical => {
            if title.is_some() || !authors.is_empty() {
                b.push_str("#block(\n  width: 100%,\n  stroke: 1pt + rgb(\"#0d9488\"),\n  fill: rgb(\"#f0fdfa\"),\n  inset: (x: 14pt, y: 12pt),\n  radius: 3pt,\n)[\n  #set par(justify: false)\n");
                b.push_str("  #text(size: 8.5pt, weight: \"bold\", fill: rgb(\"#0f766e\"))[ENGINEERING SPECIFICATION]\n");
                if let Some(t) = title {
                    b.push_str(&format!("  #v(0.3em)\n  #text(size: 1.8em, weight: \"bold\", fill: rgb(\"#0f172a\"))[{t}]\n"));
                }
                if let Some(sub) = subtitle {
                    b.push_str(&format!("  #v(0.2em)\n  #text(size: 1.05em, fill: rgb(\"#0f766e\"))[{sub}]\n"));
                }
                if !authors.is_empty() || date_str.is_some() || affiliation.is_some() {
                    b.push_str("  #v(0.4em)\n  #line(length: 100%, stroke: 0.5pt + rgb(\"#ccfbf1\"))\n  #v(0.3em)\n");
                    let authors_str = if !authors.is_empty() { authors.join(", ") } else { String::new() };
                    let aff_suffix = if let Some(aff) = affiliation { format!(" ({aff})") } else { String::new() };
                    let full_author = format!("{authors_str}{aff_suffix}");
                    let d_str = date_str.map(render_date_value).unwrap_or_default();
                    b.push_str(&format!("  #grid(columns: (1fr, auto), text(size: 9pt, fill: rgb(\"#334155\"))[{full_author}], text(size: 9pt, fill: rgb(\"#0f766e\"))[{d_str}])\n"));
                }
                b.push_str("]\n\n#v(1.4em)\n");
            }
            if abstract_text.is_some() || keywords_str.is_some() {
                b.push_str("#block(\n  stroke: (left: 3pt + rgb(\"#0d9488\")),\n  fill: rgb(\"#f8fafc\"),\n  inset: (x: 12pt, y: 10pt),\n  width: 100%,\n)[\n  #set par(justify: true, leading: 0.6em)\n");
                if let Some(abs) = abstract_text {
                    b.push_str(&format!("  #text(weight: \"bold\", fill: rgb(\"#0f766e\"))[Summary] -- #text(fill: rgb(\"#334155\"))[{abs}]\n"));
                }
                if let Some(kw) = keywords_str {
                    if abstract_text.is_some() {
                        b.push_str("  #v(0.5em)\n");
                    }
                    b.push_str(&format!("  #text(weight: \"bold\", fill: rgb(\"#0f766e\"))[Keywords] -- #text(fill: rgb(\"#475569\"))[{kw}]\n"));
                }
                b.push_str("]\n\n#v(1.4em)\n");
            }
        }
        DocumentTheme::Elegant => {
            if title.is_some() || !authors.is_empty() {
                b.push_str("#align(center)[\n  #set par(justify: false)\n");
                if let Some(t) = title {
                    b.push_str(&format!("  #text(size: 2em, weight: \"bold\", fill: rgb(\"#1c1917\"))[{t}]\n"));
                }
                if let Some(sub) = subtitle {
                    b.push_str(&format!("  #v(0.3em)\n  #text(size: 1.15em, style: \"italic\", fill: rgb(\"#44403c\"))[{sub}]\n"));
                }
                b.push_str("  #v(0.5em)\n  #line(length: 30%, stroke: 0.6pt + rgb(\"#a8a29e\"))\n  #v(0.5em)\n");
                if !authors.is_empty() {
                    let joined = authors.join(", ");
                    b.push_str(&format!("  #text(size: 1.05em, fill: rgb(\"#292524\"))[{joined}]\n"));
                }
                if let Some(aff) = affiliation
                    && !aff.is_empty() {
                        b.push_str(&format!("  #v(0.2em)\n  #text(size: 0.95em, style: \"italic\", fill: rgb(\"#78716c\"))[{aff}]\n"));
                    }
                if let Some(d) = date_str {
                    let date_typst = render_date_value(d);
                    b.push_str(&format!("  #v(0.3em)\n  #text(size: 0.9em, fill: rgb(\"#78716c\"))[{date_typst}]\n"));
                }
                b.push_str("]\n\n#v(1.6em)\n");
            }
            if abstract_text.is_some() || keywords_str.is_some() {
                b.push_str("#align(center)[\n  #block(width: 82%)[\n    #set par(justify: true, leading: 0.7em)\n");
                if let Some(abs) = abstract_text {
                    b.push_str(&format!("    #text(weight: \"bold\", fill: rgb(\"#1c1917\"))[Abstract] -- #text(style: \"italic\", fill: rgb(\"#44403c\"))[{abs}]\n"));
                }
                if let Some(kw) = keywords_str {
                    if abstract_text.is_some() {
                        b.push_str("    #v(0.5em)\n");
                    }
                    b.push_str(&format!("    #text(weight: \"bold\", fill: rgb(\"#1c1917\"))[Keywords] -- #text(style: \"italic\", fill: rgb(\"#78716c\"))[{kw}]\n"));
                }
                b.push_str("  ]\n]\n\n#v(1.6em)\n");
            }
        }
        DocumentTheme::Minimal => {
            if title.is_some() || !authors.is_empty() {
                b.push_str("#block(width: 100%)[\n  #set par(justify: false)\n");
                if let Some(t) = title {
                    b.push_str(&format!("  #text(size: 2.2em, weight: \"bold\", fill: rgb(\"#18181b\"))[{t}]\n"));
                }
                if let Some(sub) = subtitle {
                    b.push_str(&format!("  #v(0.3em)\n  #text(size: 1.15em, fill: rgb(\"#71717a\"))[{sub}]\n"));
                }
                b.push_str("  #v(0.8em)\n");
                if !authors.is_empty() {
                    let joined = authors.join(", ");
                    b.push_str(&format!("  #text(size: 1.0em, fill: rgb(\"#3f3f46\"))[{joined}]\n"));
                }
                if let Some(aff) = affiliation
                    && !aff.is_empty() {
                        b.push_str(&format!("  #v(0.15em)\n  #text(size: 0.9em, fill: rgb(\"#71717a\"))[{aff}]\n"));
                    }
                if let Some(d) = date_str {
                    let date_typst = render_date_value(d);
                    b.push_str(&format!("  #v(0.2em)\n  #text(size: 0.85em, fill: rgb(\"#71717a\"))[{date_typst}]\n"));
                }
                b.push_str("]\n\n#v(2.0em)\n");
            }
            if abstract_text.is_some() || keywords_str.is_some() {
                b.push_str("#block(width: 100%)[\n  #set par(justify: true, leading: 0.7em)\n");
                if let Some(abs) = abstract_text {
                    b.push_str(&format!("  #text(fill: rgb(\"#52525b\"))[{abs}]\n"));
                }
                if let Some(kw) = keywords_str {
                    if abstract_text.is_some() {
                        b.push_str("  #v(0.4em)\n");
                    }
                    b.push_str(&format!("  #text(size: 0.9em, fill: rgb(\"#71717a\"))[{kw}]\n"));
                }
                b.push_str("]\n\n#v(1.8em)\n");
            }
        }
        DocumentTheme::Academic | DocumentTheme::Compact => {
            if title.is_some() || !authors.is_empty() {
                b.push_str("#align(center)[\n  #set par(justify: false)\n");
                let title_size = if doc_theme == DocumentTheme::Compact { "1.6em" } else { "1.8em" };
                if let Some(t) = title {
                    b.push_str(&format!("  #text(size: {title_size}, weight: \"bold\")[{t}]\n\n"));
                }
                if let Some(sub) = subtitle {
                    b.push_str(&format!("  #v(0.3em)\n  #text(size: 1.15em, style: \"italic\", fill: rgb(\"#334155\"))[{sub}]\n\n"));
                }
                b.push_str("  #v(0.6em)\n");
                if !authors.is_empty() {
                    let joined = authors.join(", ");
                    b.push_str(&format!("  #text(size: 1.1em)[{joined}]\n"));
                }
                if let Some(aff) = affiliation
                    && !aff.is_empty() {
                        b.push_str(&format!("  #v(0.2em)\n  #text(size: 0.95em, style: \"italic\")[{aff}]\n"));
                    }
                if let Some(d) = date_str {
                    let date_typst = render_date_value(d);
                    b.push_str(&format!("  #v(0.4em)\n  #text(size: 0.9em, fill: rgb(\"#475569\"))[{date_typst}]\n"));
                }
                b.push_str("]\n\n#v(1.2em)\n");
            }
            if abstract_text.is_some() || keywords_str.is_some() {
                let abs_width = if doc_theme == DocumentTheme::Compact { "92%" } else { "88%" };
                b.push_str(&format!("#align(center)[\n  #block(width: {abs_width})[\n    #set par(justify: true, leading: 0.6em)\n"));
                if let Some(abs) = abstract_text {
                    b.push_str(&format!("    #text(weight: \"bold\")[Abstract] -- #text(style: \"italic\")[{abs}]\n"));
                }
                if let Some(kw) = keywords_str {
                    if abstract_text.is_some() {
                        b.push_str("    #v(0.6em)\n");
                    }
                    b.push_str(&format!("    #text(weight: \"bold\")[Keywords] -- #text(style: \"italic\")[{kw}]\n"));
                }
                b.push_str("  ]\n]\n\n#v(1.4em)\n");
            }
        }
    }

    b
}

fn format_font_stack(font_raw: &str) -> String {
    if font_raw.eq_ignore_ascii_case("Times New Roman") || font_raw.eq_ignore_ascii_case("Times") {
        "(\"Times New Roman\", \"Nimbus Roman\", \"Liberation Serif\", \"New Computer Modern\")".to_string()
    } else if font_raw.eq_ignore_ascii_case("New Computer Modern") {
        "(\"New Computer Modern\",)".to_string()
    } else if font_raw.eq_ignore_ascii_case("Liberation Sans")
        || font_raw.eq_ignore_ascii_case("Helvetica")
        || font_raw.eq_ignore_ascii_case("Arial")
        || font_raw.eq_ignore_ascii_case("Sans")
    {
        "(\"Liberation Sans\", \"Adwaita Sans\", \"DejaVu Sans\", \"Helvetica Neue\", \"Arial\", \"New Computer Modern\")".to_string()
    } else if font_raw.eq_ignore_ascii_case("Linux Libertine") || font_raw.eq_ignore_ascii_case("Libertinus Serif") {
        "(\"Linux Libertine\", \"Libertinus Serif\", \"Nimbus Roman\", \"Times New Roman\", \"New Computer Modern\")".to_string()
    } else {
        format!("(\"{font_raw}\", \"New Computer Modern\")")
    }
}

fn parse_margins(fm: &Value, config: &DocumentConfig, default_x: &str, default_y: &str, theme_explicit: bool) -> (String, String) {
    if let Some(fm_m) = fm.get("margin") {
        if let Some(s) = fm_m.as_str() {
            return (s.to_string(), s.to_string());
        } else if let Some(x) = fm_m.get("x").and_then(|v| v.as_str()) {
            let y = fm_m.get("y").and_then(|v| v.as_str()).unwrap_or(x);
            return (x.to_string(), y.to_string());
        }
    }

    if let Some(geom) = fm.get("geometry") {
        let geom_strs: Vec<String> = if let Some(s) = geom.as_str() {
            s.split(',').map(|p| p.trim().to_string()).collect()
        } else if let Some(arr) = geom.as_sequence() {
            arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()
        } else {
            Vec::new()
        };

        let mut mx = None;
        let mut my = None;
        for part in geom_strs {
            if let Some(val) = part.strip_prefix("margin=") {
                mx = Some(val.to_string());
                my = Some(val.to_string());
            } else if let Some(val) = part.strip_prefix("left=") {
                mx = Some(val.to_string());
            } else if let Some(val) = part.strip_prefix("right=") {
                mx = Some(val.to_string());
            } else if let Some(val) = part.strip_prefix("top=") {
                my = Some(val.to_string());
            } else if let Some(val) = part.strip_prefix("bottom=") {
                my = Some(val.to_string());
            }
        }
        if mx.is_some() || my.is_some() {
            return (
                mx.unwrap_or_else(|| default_x.to_string()),
                my.unwrap_or_else(|| default_y.to_string()),
            );
        }
    }

    if !theme_explicit
        && let Some(ref m) = config.margin {
            return match m {
                MarginConfig::Uniform(s) => (s.clone(), s.clone()),
                MarginConfig::Axes { x, y } => (
                    x.clone().unwrap_or_else(|| default_x.to_string()),
                    y.clone().unwrap_or_else(|| default_y.to_string()),
                ),
            };
        }

    (default_x.to_string(), default_y.to_string())
}

pub fn wrap_document(
    body_typst: &str,
    fm: &Value,
    config: &DocumentConfig,
    cli_toc: Option<bool>,
    cli_author: Option<&str>,
    cli_font: Option<&str>,
    cli_theme: Option<&str>,
) -> String {
    let mut out = String::new();

    let theme_explicit = cli_theme.is_some() || fm.get("theme").is_some();
    let theme_raw = cli_theme
        .or_else(|| fm.get("theme").and_then(|v| v.as_str()))
        .or(config.theme.as_deref())
        .unwrap_or("academic");
    let doc_theme = DocumentTheme::parse(theme_raw);

    let title = extract_title(fm);
    let authors = extract_authors(fm, config.author.as_deref(), cli_author);
    let date_str = extract_date(fm);

    // 0. PDF Document Metadata
    if let Some(doc_meta) = build_document_metadata(title.as_deref(), &authors, date_str.as_deref()) {
        out.push_str(&doc_meta);
    }

    // 1. Paper and Margins
    let papersize = fm.get("papersize")
        .and_then(|v| v.as_str())
        .or(config.papersize.as_deref())
        .unwrap_or("a4");

    let (def_mx, def_my) = doc_theme.default_margins();
    let (margin_x, margin_y) = parse_margins(fm, config, def_mx, def_my, theme_explicit);

    let classoption = fm.get("classoption").and_then(|v| {
        if let Some(s) = v.as_str() {
            Some(s.to_string())
        } else if let Some(arr) = v.as_sequence() {
            Some(arr.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>().join(" "))
        } else {
            None
        }
    }).unwrap_or_default();

    let columns = fm.get("columns")
        .and_then(|v| v.as_u64())
        .map(|n| n as usize)
        .or_else(|| if classoption.contains("twocolumn") { Some(2) } else { None })
        .or_else(|| {
            if theme_explicit {
                Some(doc_theme.default_columns())
            } else {
                config.columns.or_else(|| Some(doc_theme.default_columns()))
            }
        })
        .unwrap_or_else(|| doc_theme.default_columns());

    out.push_str(&format!(
        "#set page(\n  paper: \"{papersize}\",\n  margin: (x: {margin_x}, y: {margin_y}),\n  numbering: \"1\",\n)\n"
    ));

    // 2. Typography
    let lang = fm.get("lang")
        .and_then(|v| v.as_str())
        .or(config.lang.as_deref())
        .unwrap_or("en");

    let font_raw = cli_font
        .or_else(|| fm.get("mainfont").and_then(|v| v.as_str()))
        .or_else(|| {
            if theme_explicit {
                Some(doc_theme.default_font())
            } else {
                config.mainfont.as_deref().or_else(|| Some(doc_theme.default_font()))
            }
        })
        .unwrap_or_else(|| doc_theme.default_font());

    let font_family = format_font_stack(font_raw);

    let monofont_raw = fm.get("monofont")
        .and_then(|v| v.as_str())
        .or(config.monofont.as_deref())
        .unwrap_or("DejaVu Sans Mono");

    let monofont_family = if monofont_raw.eq_ignore_ascii_case("Courier New") || monofont_raw.eq_ignore_ascii_case("Courier") {
        "(\"Courier New\", \"Liberation Mono\", \"DejaVu Sans Mono\")".to_string()
    } else if monofont_raw.eq_ignore_ascii_case("DejaVu Sans Mono") {
        "(\"DejaVu Sans Mono\",)".to_string()
    } else {
        format!("(\"{monofont_raw}\", \"DejaVu Sans Mono\")")
    };

    let fontsize = fm.get("fontsize")
        .and_then(|v| {
            if let Some(s) = v.as_str() {
                Some(s.to_string())
            } else if let Some(n) = v.as_i64() {
                Some(format!("{n}pt"))
            } else { v.as_f64().map(|n| format!("{n}pt")) }
        })
        .or_else(|| {
            if theme_explicit {
                Some(doc_theme.default_fontsize().to_string())
            } else {
                config.fontsize.clone().or_else(|| Some(doc_theme.default_fontsize().to_string()))
            }
        })
        .unwrap_or_else(|| doc_theme.default_fontsize().to_string());

    if let Some(text_color) = doc_theme.default_text_color() {
        out.push_str(&format!(
            "#set text(font: {font_family}, size: {fontsize}, lang: \"{lang}\", fill: rgb(\"{text_color}\"))\n"
        ));
    } else {
        out.push_str(&format!(
            "#set text(font: {font_family}, size: {fontsize}, lang: \"{lang}\")\n"
        ));
    }

    // Math font & equations
    let mathfont = fm.get("mathfont")
        .and_then(|v| v.as_str())
        .or(config.mathfont.as_deref())
        .unwrap_or("New Computer Modern Math");
    out.push_str(&format!(
        "#show math.equation: set text(font: \"{mathfont}\")\n"
    ));
    out.push_str("#show math.equation.where(block: true): it => block(above: 1.2em, below: 1.2em)[#it]\n");
    out.push_str("#set math.equation(numbering: \"(1)\")\n\n");

    // Pandoc callouts prelude styled by theme
    match doc_theme {
        DocumentTheme::Modern => {
            out.push_str("#let botox_callout(kind, title, body) = {\n");
            out.push_str("  let (stroke_color, fill_color, default_title) = if kind == \"warning\" {\n");
            out.push_str("    (rgb(\"#d97706\"), rgb(\"#fffbeb\"), \"Warning\")\n");
            out.push_str("  } else if kind == \"tip\" {\n");
            out.push_str("    (rgb(\"#16a34a\"), rgb(\"#f0fdf4\"), \"Tip\")\n");
            out.push_str("  } else if kind == \"important\" or kind == \"caution\" or kind == \"danger\" {\n");
            out.push_str("    (rgb(\"#dc2626\"), rgb(\"#fef2f2\"), \"Important\")\n");
            out.push_str("  } else {\n");
            out.push_str("    (rgb(\"#2563eb\"), rgb(\"#eff6ff\"), \"Note\")\n");
            out.push_str("  };\n");
            out.push_str("  let display_title = if title != \"\" { title } else { default_title };\n");
            out.push_str("  block(\n");
            out.push_str("    fill: fill_color,\n");
            out.push_str("    stroke: (left: 4pt + stroke_color),\n");
            out.push_str("    inset: (x: 12pt, y: 10pt),\n");
            out.push_str("    radius: 4pt,\n");
            out.push_str("    width: 100%,\n");
            out.push_str("    above: 1.2em,\n");
            out.push_str("    below: 1.2em,\n");
            out.push_str("  )[\n");
            out.push_str("    #text(weight: \"bold\", fill: stroke_color)[#display_title]\\\n");
            out.push_str("    #v(0.3em)\n");
            out.push_str("    #body\n");
            out.push_str("  ]\n");
            out.push_str("}\n\n");
        }
        DocumentTheme::Technical => {
            out.push_str("#let botox_callout(kind, title, body) = {\n");
            out.push_str("  let (stroke_color, fill_color, default_title) = if kind == \"warning\" {\n");
            out.push_str("    (rgb(\"#d97706\"), rgb(\"#fffbeb\"), \"Warning\")\n");
            out.push_str("  } else if kind == \"tip\" {\n");
            out.push_str("    (rgb(\"#0d9488\"), rgb(\"#f0fdfa\"), \"Tip\")\n");
            out.push_str("  } else if kind == \"important\" or kind == \"caution\" or kind == \"danger\" {\n");
            out.push_str("    (rgb(\"#e11d48\"), rgb(\"#fff1f2\"), \"Important\")\n");
            out.push_str("  } else {\n");
            out.push_str("    (rgb(\"#0284c7\"), rgb(\"#f0f9ff\"), \"Note\")\n");
            out.push_str("  };\n");
            out.push_str("  let display_title = if title != \"\" { title } else { default_title };\n");
            out.push_str("  block(\n");
            out.push_str("    fill: fill_color,\n");
            out.push_str("    stroke: 1pt + stroke_color,\n");
            out.push_str("    inset: (x: 12pt, y: 10pt),\n");
            out.push_str("    radius: 2pt,\n");
            out.push_str("    width: 100%,\n");
            out.push_str("    above: 1.2em,\n");
            out.push_str("    below: 1.2em,\n");
            out.push_str("  )[\n");
            out.push_str("    #text(weight: \"bold\", fill: stroke_color)[#display_title]\\\n");
            out.push_str("    #v(0.3em)\n");
            out.push_str("    #body\n");
            out.push_str("  ]\n");
            out.push_str("}\n\n");
        }
        DocumentTheme::Minimal => {
            out.push_str("#let botox_callout(kind, title, body) = {\n");
            out.push_str("  let stroke_color = rgb(\"#71717a\");\n");
            out.push_str("  let default_title = if kind == \"warning\" { \"Warning\" } else if kind == \"tip\" { \"Tip\" } else if kind == \"important\" { \"Important\" } else { \"Note\" };\n");
            out.push_str("  let display_title = if title != \"\" { title } else { default_title };\n");
            out.push_str("  block(\n");
            out.push_str("    stroke: (left: 2pt + stroke_color),\n");
            out.push_str("    inset: (left: 10pt, y: 4pt),\n");
            out.push_str("    width: 100%,\n");
            out.push_str("    above: 1.2em,\n");
            out.push_str("    below: 1.2em,\n");
            out.push_str("  )[\n");
            out.push_str("    #text(weight: \"bold\", fill: stroke_color)[#display_title]\\\n");
            out.push_str("    #v(0.2em)\n");
            out.push_str("    #body\n");
            out.push_str("  ]\n");
            out.push_str("}\n\n");
        }
        DocumentTheme::Elegant => {
            out.push_str("#let botox_callout(kind, title, body) = {\n");
            out.push_str("  let (stroke_color, fill_color, default_title) = if kind == \"warning\" {\n");
            out.push_str("    (rgb(\"#b45309\"), rgb(\"#fffbeb\"), \"Warning\")\n");
            out.push_str("  } else if kind == \"tip\" {\n");
            out.push_str("    (rgb(\"#15803d\"), rgb(\"#f0fdf4\"), \"Tip\")\n");
            out.push_str("  } else if kind == \"important\" or kind == \"caution\" or kind == \"danger\" {\n");
            out.push_str("    (rgb(\"#b91c1c\"), rgb(\"#fef2f2\"), \"Important\")\n");
            out.push_str("  } else {\n");
            out.push_str("    (rgb(\"#0369a1\"), rgb(\"#f0f9ff\"), \"Note\")\n");
            out.push_str("  };\n");
            out.push_str("  let display_title = if title != \"\" { title } else { default_title };\n");
            out.push_str("  block(\n");
            out.push_str("    fill: fill_color,\n");
            out.push_str("    stroke: (left: 3.5pt + stroke_color),\n");
            out.push_str("    inset: (x: 12pt, y: 10pt),\n");
            out.push_str("    radius: (right: 3pt),\n");
            out.push_str("    width: 100%,\n");
            out.push_str("    above: 1.2em,\n");
            out.push_str("    below: 1.2em,\n");
            out.push_str("  )[\n");
            out.push_str("    #text(weight: \"bold\", fill: stroke_color)[#display_title]\\\n");
            out.push_str("    #v(0.3em)\n");
            out.push_str("    #body\n");
            out.push_str("  ]\n");
            out.push_str("}\n\n");
        }
        _ => {
            // Academic & Compact
            out.push_str("#let botox_callout(kind, title, body) = {\n");
            out.push_str("  let (stroke_color, fill_color, default_title) = if kind == \"warning\" {\n");
            out.push_str("    (rgb(\"#d97706\"), rgb(\"#fffbeb\"), \"Warning\")\n");
            out.push_str("  } else if kind == \"tip\" {\n");
            out.push_str("    (rgb(\"#16a34a\"), rgb(\"#f0fdf4\"), \"Tip\")\n");
            out.push_str("  } else if kind == \"important\" or kind == \"caution\" or kind == \"danger\" {\n");
            out.push_str("    (rgb(\"#dc2626\"), rgb(\"#fef2f2\"), \"Important\")\n");
            out.push_str("  } else {\n");
            out.push_str("    (rgb(\"#0284c7\"), rgb(\"#f1f5f9\"), \"Note\")\n");
            out.push_str("  };\n");
            out.push_str("  let display_title = if title != \"\" { title } else { default_title };\n");
            out.push_str("  block(\n");
            out.push_str("    fill: fill_color,\n");
            out.push_str("    stroke: (left: 3.5pt + stroke_color),\n");
            out.push_str("    inset: (x: 12pt, y: 10pt),\n");
            out.push_str("    radius: (right: 4pt),\n");
            out.push_str("    width: 100%,\n");
            out.push_str("    above: 1.2em,\n");
            out.push_str("    below: 1.2em,\n");
            out.push_str("  )[\n");
            out.push_str("    #text(weight: \"bold\", fill: stroke_color)[#display_title]\\\n");
            out.push_str("    #v(0.3em)\n");
            out.push_str("    #body\n");
            out.push_str("  ]\n");
            out.push_str("}\n\n");
        }
    }

    // Link color
    let linkcolor = fm.get("linkcolor")
        .and_then(|v| v.as_str())
        .or(config.linkcolor.as_deref())
        .unwrap_or_else(|| doc_theme.default_linkcolor());

    let color_val = if linkcolor.starts_with('#') {
        format!("rgb(\"{linkcolor}\")")
    } else if matches!(linkcolor, "blue" | "red" | "green" | "navy" | "maroon" | "purple" | "teal" | "olive" | "gray" | "black" | "orange") {
        linkcolor.to_string()
    } else {
        format!("rgb(\"{linkcolor}\")")
    };
    out.push_str(&format!("#show link: set text(fill: {color_val})\n"));

    // Paragraph & Base Typography styling
    let linestretch = fm.get("linestretch")
        .or_else(|| fm.get("line-spacing"))
        .and_then(|v| v.as_f64());
    let leading_em = if let Some(ls) = linestretch {
        format!("{:.3}em", 0.7 * ls)
    } else {
        doc_theme.default_leading().to_string()
    };

    let parindent = fm.get("indent")
        .or_else(|| fm.get("parindent"))
        .and_then(|v| {
            if let Some(b) = v.as_bool() {
                if b { Some("1.5em".to_string()) } else { None }
            } else { v.as_str().map(|s| s.to_string()) }
        });
    let indent_clause = if let Some(ind) = parindent {
        format!(", first-line-indent: {ind}")
    } else {
        String::new()
    };

    out.push_str(&format!("#set par(justify: true, leading: {leading_em}{indent_clause})\n"));
    out.push_str("#show heading: set par(justify: false)\n");
    out.push_str("#show list: set par(justify: false)\n");
    out.push_str("#show enum: set par(justify: false)\n");
    out.push_str("#show table.cell: set par(justify: false)\n\n");

    // Code styling
    out.push_str(&format!("#show raw: set text(font: {monofont_family})\n"));
    match doc_theme {
        DocumentTheme::Technical => {
            out.push_str("#show raw.where(block: true): it => block(\n");
            out.push_str("  fill: rgb(\"#f8fafc\"),\n");
            out.push_str("  stroke: (left: 3pt + rgb(\"#0d9488\"), rest: 0.5pt + rgb(\"#e2e8f0\")),\n");
            out.push_str("  inset: (x: 12pt, y: 10pt),\n");
            out.push_str("  radius: (right: 4pt),\n");
            out.push_str("  width: 100%,\n");
            out.push_str(")[#set text(size: 8.5pt); #it]\n");
        }
        DocumentTheme::Minimal => {
            out.push_str("#show raw.where(block: true): it => block(\n");
            out.push_str("  fill: rgb(\"#f4f4f5\"),\n");
            out.push_str("  inset: (x: 12pt, y: 10pt),\n");
            out.push_str("  radius: 3pt,\n");
            out.push_str("  width: 100%,\n");
            out.push_str(")[#set text(size: 8.5pt); #it]\n");
        }
        DocumentTheme::Elegant => {
            out.push_str("#show raw.where(block: true): it => block(\n");
            out.push_str("  fill: rgb(\"#fafaf9\"),\n");
            out.push_str("  stroke: 0.5pt + rgb(\"#e7e5e4\"),\n");
            out.push_str("  inset: (x: 12pt, y: 10pt),\n");
            out.push_str("  radius: 4pt,\n");
            out.push_str("  width: 100%,\n");
            out.push_str(")[#set text(size: 8.5pt); #it]\n");
        }
        DocumentTheme::Compact => {
            out.push_str("#show raw.where(block: true): it => block(\n");
            out.push_str("  fill: rgb(\"#f8fafc\"),\n");
            out.push_str("  stroke: 0.5pt + rgb(\"#e2e8f0\"),\n");
            out.push_str("  inset: (x: 8pt, y: 7pt),\n");
            out.push_str("  radius: 3pt,\n");
            out.push_str("  width: 100%,\n");
            out.push_str(")[#set text(size: 8pt); #it]\n");
        }
        _ => {
            // Academic & Modern
            out.push_str("#show raw.where(block: true): it => block(\n");
            out.push_str("  fill: rgb(\"#f8fafc\"),\n");
            out.push_str("  stroke: 0.5pt + rgb(\"#e2e8f0\"),\n");
            out.push_str("  inset: (x: 12pt, y: 10pt),\n");
            out.push_str("  radius: 4pt,\n");
            out.push_str("  width: 100%,\n");
            out.push_str(")[#set text(size: 8.5pt); #it]\n");
        }
    }
    out.push_str("#show raw.where(block: false): it => box(\n");
    out.push_str("  fill: rgb(\"#f1f5f9\"),\n");
    out.push_str("  inset: (x: 3pt, y: 1.5pt),\n");
    out.push_str("  radius: 2pt,\n");
    out.push_str(")[#set text(size: 0.9em); #it]\n\n");

    // Blockquote styling
    let (quote_fill, quote_border, quote_text) = match doc_theme {
        DocumentTheme::Technical => ("rgb(\"#f8fafc\")", "rgb(\"#0d9488\")", "rgb(\"#334155\")"),
        DocumentTheme::Elegant => ("rgb(\"#fafaf9\")", "rgb(\"#a8a29e\")", "rgb(\"#44403c\")"),
        DocumentTheme::Minimal => ("rgb(\"#fafafa\")", "rgb(\"#71717a\")", "rgb(\"#3f3f46\")"),
        _ => ("rgb(\"#f8fafc\")", "rgb(\"#94a3b8\")", "rgb(\"#334155\")"),
    };
    out.push_str(&format!(
        "#show quote.where(block: true): it => block(\n  fill: {quote_fill},\n  stroke: (left: 3pt + {quote_border}),\n  inset: (left: 14pt, y: 8pt, right: 10pt),\n  radius: (right: 3pt),\n  width: 100%,\n  above: 1.2em,\n  below: 1.2em,\n)[\n  #set text(style: \"italic\", fill: {quote_text})\n  #it.body\n]\n\n"
    ));

    // Table styling per theme
    match doc_theme {
        DocumentTheme::Modern => {
            out.push_str("#show table: set table(\n");
            out.push_str("  inset: (x: 10pt, y: 7pt),\n");
            out.push_str("  fill: (x, y) => if y == 0 { rgb(\"#f1f5f9\") } else { none },\n");
            out.push_str("  stroke: (x, y) => if y == 0 { (bottom: 1.5pt + rgb(\"#2563eb\")) } else { (bottom: 0.5pt + rgb(\"#e2e8f0\")) },\n");
            out.push_str(")\n");
            out.push_str("#show table.cell.where(y: 0): set text(weight: \"bold\", fill: rgb(\"#0f172a\"))\n\n");
        }
        DocumentTheme::Technical => {
            out.push_str("#show table: set table(\n");
            out.push_str("  inset: (x: 10pt, y: 7pt),\n");
            out.push_str("  fill: (x, y) => if y == 0 { rgb(\"#f0fdfa\") } else { none },\n");
            out.push_str("  stroke: (x, y) => if y == 0 { (bottom: 1.5pt + rgb(\"#0d9488\"), top: 1.5pt + rgb(\"#0d9488\")) } else { (bottom: 0.5pt + rgb(\"#ccfbf1\")) },\n");
            out.push_str(")\n");
            out.push_str("#show table.cell.where(y: 0): set text(weight: \"bold\", fill: rgb(\"#0f766e\"))\n\n");
        }
        DocumentTheme::Elegant => {
            out.push_str("#show table: set table(\n");
            out.push_str("  inset: (x: 10pt, y: 7pt),\n");
            out.push_str("  stroke: (x, y) => if y == 0 { (bottom: 1.2pt + rgb(\"#292524\"), top: 1.2pt + rgb(\"#292524\")) } else { (bottom: 0.5pt + rgb(\"#d6d3d1\")) },\n");
            out.push_str(")\n");
            out.push_str("#show table.cell.where(y: 0): set text(weight: \"bold\", fill: rgb(\"#1c1917\"))\n\n");
        }
        DocumentTheme::Minimal => {
            out.push_str("#show table: set table(\n");
            out.push_str("  inset: (x: 10pt, y: 7pt),\n");
            out.push_str("  stroke: (x, y) => if y == 0 { (bottom: 1pt + rgb(\"#18181b\"), top: 1pt + rgb(\"#18181b\")) } else { (bottom: 0.5pt + rgb(\"#e4e4e7\")) },\n");
            out.push_str(")\n");
            out.push_str("#show table.cell.where(y: 0): set text(weight: \"bold\", fill: rgb(\"#18181b\"))\n\n");
        }
        _ => {
            // Academic & Compact
            out.push_str("#show table: set table(\n");
            out.push_str("  inset: (x: 10pt, y: 7pt),\n");
            out.push_str("  stroke: (x, y) => if y == 0 { (bottom: 1.2pt + black) } else { (bottom: 0.5pt + rgb(\"#cbd5e1\")) },\n");
            out.push_str(")\n");
            out.push_str("#show table.cell.where(y: 0): set text(weight: \"bold\")\n\n");
        }
    }

    // Section numbering
    let numbering = fm.get("numbersections")
        .or_else(|| fm.get("number-sections"))
        .and_then(|v| v.as_bool())
        .or(config.section_numbering)
        .unwrap_or_else(|| doc_theme.default_numbering());

    if numbering {
        out.push_str("#set heading(numbering: \"1.1\")\n");
    }

    // Line numbering (Pandoc linenumbers / line-numbers)
    let linenumbers = fm.get("linenumbers")
        .or_else(|| fm.get("line-numbers"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if linenumbers {
        out.push_str("#set par.line(numbering: \"1\")\n");
    }

    // Heading styling per theme
    match doc_theme {
        DocumentTheme::Modern => {
            out.push_str("#show heading.where(level: 1): it => block(above: 1.9em, below: 1.1em, width: 100%)[\n");
            out.push_str("  #text(fill: rgb(\"#0f172a\"), size: 1.35em, weight: \"bold\")[#it]\n");
            out.push_str("  #v(0.35em)\n");
            out.push_str("  #line(length: 100%, stroke: 1.2pt + rgb(\"#2563eb\"))\n");
            out.push_str("]\n");
            out.push_str("#show heading.where(level: 2): it => block(above: 1.4em, below: 0.7em)[#text(fill: rgb(\"#0f172a\"), size: 1.18em, weight: \"bold\")[#it]]\n");
            out.push_str("#show heading.where(level: 3): it => block(above: 1.1em, below: 0.5em)[#text(fill: rgb(\"#334155\"), size: 1.05em, weight: \"bold\")[#it]]\n\n");
        }
        DocumentTheme::Technical => {
            out.push_str("#show heading.where(level: 1): it => block(above: 1.7em, below: 0.8em, width: 100%)[\n");
            out.push_str("  #text(fill: rgb(\"#0f766e\"), size: 1.3em, weight: \"bold\")[#it]\n");
            out.push_str("  #v(0.3em)\n");
            out.push_str("  #line(length: 100%, stroke: 1pt + rgb(\"#0d9488\"))\n");
            out.push_str("]\n");
            out.push_str("#show heading.where(level: 2): it => block(above: 1.3em, below: 0.6em)[#text(fill: rgb(\"#0f766e\"), size: 1.15em, weight: \"bold\")[#it]]\n");
            out.push_str("#show heading.where(level: 3): it => block(above: 1.0em, below: 0.5em)[#text(fill: rgb(\"#0369a1\"), size: 1.0em, weight: \"bold\")[#it]]\n\n");
        }
        DocumentTheme::Elegant => {
            out.push_str("#show heading.where(level: 1): it => block(above: 2.2em, below: 1.0em)[#text(fill: rgb(\"#1c1917\"), size: 1.35em, weight: \"bold\")[#it]]\n");
            out.push_str("#show heading.where(level: 2): it => block(above: 1.6em, below: 0.7em)[#text(fill: rgb(\"#292524\"), size: 1.15em, weight: \"bold\")[#it]]\n");
            out.push_str("#show heading.where(level: 3): it => block(above: 1.2em, below: 0.5em)[#text(fill: rgb(\"#44403c\"), size: 1.05em, style: \"italic\")[#it]]\n\n");
        }
        DocumentTheme::Compact => {
            out.push_str("#show heading.where(level: 1): it => block(above: 1.2em, below: 0.5em)[#text(size: 1.15em, weight: \"bold\")[#it]]\n");
            out.push_str("#show heading.where(level: 2): it => block(above: 1.0em, below: 0.4em)[#text(size: 1.05em, weight: \"bold\")[#it]]\n");
            out.push_str("#show heading.where(level: 3): it => block(above: 0.8em, below: 0.3em)[#text(size: 1.0em, weight: \"bold\", style: \"italic\")[#it]]\n\n");
        }
        DocumentTheme::Minimal => {
            out.push_str("#show heading.where(level: 1): it => block(above: 2.4em, below: 1.0em)[#text(size: 1.5em, weight: \"bold\", fill: rgb(\"#18181b\"))[#it]]\n");
            out.push_str("#show heading.where(level: 2): it => block(above: 1.8em, below: 0.8em)[#text(size: 1.2em, weight: \"bold\", fill: rgb(\"#27272a\"))[#it]]\n");
            out.push_str("#show heading.where(level: 3): it => block(above: 1.4em, below: 0.6em)[#text(size: 1.05em, weight: \"medium\", fill: rgb(\"#3f3f46\"))[#it]]\n\n");
        }
        DocumentTheme::Academic => {
            out.push_str("#show heading.where(level: 1): it => block(above: 1.8em, below: 0.9em)[#it]\n");
            out.push_str("#show heading.where(level: 2): it => block(above: 1.4em, below: 0.7em)[#it]\n");
            out.push_str("#show heading.where(level: 3): it => block(above: 1.1em, below: 0.6em)[#it]\n\n");
        }
    }

    // 3. Title block and Abstract
    let subtitle = fm.get("subtitle").and_then(|v| v.as_str());

    let affiliation = if let Some(a) = fm.get("author").or_else(|| fm.get("authors")) {
        if let Some(arr) = a.as_sequence() {
            arr.first().and_then(|item| item.get("affiliation").and_then(|v| v.as_str()))
        } else {
            fm.get("affiliation").and_then(|v| v.as_str())
        }
    } else {
        fm.get("affiliation").and_then(|v| v.as_str()).or(config.affiliation.as_deref())
    };

    let abstract_text = fm.get("abstract").and_then(|v| v.as_str());
    let keywords_str: Option<String> = fm.get("keywords").and_then(|v| {
        if let Some(s) = v.as_str() {
            Some(s.to_string())
        } else if let Some(arr) = v.as_sequence() {
            let kw: Vec<String> = arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect();
            Some(kw.join(", "))
        } else {
            None
        }
    });

    let rendered_header = render_title_and_abstract(
        doc_theme,
        title.as_deref(),
        subtitle,
        &authors,
        affiliation,
        date_str.as_deref(),
        abstract_text,
        keywords_str.as_deref(),
    );
    out.push_str(&rendered_header);

    if columns > 1 {
        let gutter = fm.get("column-gutter")
            .or_else(|| fm.get("column_gutter"))
            .and_then(|v| {
                if let Some(s) = v.as_str() {
                    Some(s.to_string())
                } else if let Some(n) = v.as_i64() {
                    Some(format!("{n}pt"))
                } else { v.as_f64().map(|n| format!("{n}pt")) }
            })
            .unwrap_or_else(|| "14pt".to_string());
        out.push_str(&format!("#show: columns.with({columns}, gutter: {gutter})\n\n"));
    }

    // 4. Table of Contents
    let should_include_toc = if let Some(cli) = cli_toc {
        cli
    } else if let Some(fm_toc) = fm.get("toc").or_else(|| fm.get("table-of-contents")).and_then(|v| v.as_bool()) {
        fm_toc
    } else {
        config.toc.unwrap_or(false)
    };

    let toc_depth = fm.get("toc-depth")
        .or_else(|| fm.get("toc_depth"))
        .and_then(|v| v.as_u64())
        .map(|n| n as usize)
        .or(config.toc_depth)
        .unwrap_or(3);

    let toc_title = fm.get("toc-title")
        .or_else(|| fm.get("toc_title"))
        .and_then(|v| v.as_str())
        .or(config.toc_title.as_deref());

    if should_include_toc && !body_typst.contains("#outline") {
        if let Some(title) = toc_title {
            out.push_str(&format!("#outline(title: \"{title}\", depth: {toc_depth})\n#v(1.5em)\n"));
        } else {
            out.push_str(&format!("#outline(depth: {toc_depth})\n#v(1.5em)\n"));
        }
    }

    let lof = fm.get("lof").and_then(|v| v.as_bool()).unwrap_or(false);
    if lof {
        let lof_title = match lang {
            "fr" => "Table des figures",
            "de" => "Abbildungsverzeichnis",
            "es" => "Índice de figuras",
            _ => "List of Figures",
        };
        out.push_str(&format!("#outline(title: \"{lof_title}\", target: figure.where(kind: image))\n#v(1.5em)\n"));
    }

    let lot = fm.get("lot").and_then(|v| v.as_bool()).unwrap_or(false);
    if lot {
        let lot_title = match lang {
            "fr" => "Liste des tableaux",
            "de" => "Tabellenverzeichnis",
            "es" => "Índice de cuadros",
            _ => "List of Tables",
        };
        out.push_str(&format!("#outline(title: \"{lot_title}\", target: figure.where(kind: table))\n#v(1.5em)\n"));
    }

    // 5. Body
    let clean_body = body_typst.replace("#botox_pause()", "");
    let processed_body = sanitize_column_pagebreaks(&clean_body, columns);
    out.push_str("// BOTOX_BODY_START\n");
    out.push_str(&processed_body);

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::metadata::{escape_typst_string, format_typst_date};

    #[test]
    fn test_escape_typst_string() {
        assert_eq!(escape_typst_string("Normal Title"), "Normal Title");
        assert_eq!(
            escape_typst_string(r#"Quotes "and" Backslashes \here\"#),
            r#"Quotes \"and\" Backslashes \\here\\"#
        );
        assert_eq!(escape_typst_string("Line1\nLine2\r"), r"Line1\nLine2");
    }

    #[test]
    fn test_format_typst_date() {
        assert_eq!(format_typst_date(r"\today").as_deref(), Some("datetime.today()"));
        assert_eq!(format_typst_date("today").as_deref(), Some("datetime.today()"));
        assert_eq!(format_typst_date("2026-10-07").as_deref(), Some("datetime(year: 2026, month: 10, day: 7)"));
        assert_eq!(format_typst_date("2026-10").as_deref(), Some("datetime(year: 2026, month: 10, day: 1)"));
        assert_eq!(format_typst_date("2026").as_deref(), Some("datetime(year: 2026, month: 1, day: 1)"));
        assert_eq!(format_typst_date("auto").as_deref(), Some("auto"));
        assert_eq!(format_typst_date("none").as_deref(), Some("none"));
        assert_eq!(format_typst_date("October 2026").as_deref(), Some("auto"));
        assert_eq!(format_typst_date(""), None);
    }

    #[test]
    fn test_document_metadata_emission() {
        let default_cfg = DocumentConfig::defaults();

        // 1. Title, single author, ISO date
        let fm: Value = serde_yaml::from_str(r#"
title: "Autonomous Agents"
author: "Ada Lovelace"
date: "2026-10-07"
"#).unwrap();
        let doc = wrap_document("Hello world", &fm, &default_cfg, None, None, None, None);
        assert!(doc.starts_with("#set document(title: \"Autonomous Agents\", author: \"Ada Lovelace\", date: datetime(year: 2026, month: 10, day: 7))\n"));

        // 2. Escaped quotes and backslashes in title, multiple authors
        let fm_esc: Value = serde_yaml::from_str(r#"
title: 'The "Art" of \Coding\'
author:
  - "Alice Smith"
  - "Bob Jones"
date: \today
"#).unwrap();
        let doc_esc = wrap_document("Hello world", &fm_esc, &default_cfg, None, None, None, None);
        assert!(doc_esc.contains(r#"#set document(title: "The \"Art\" of \\Coding\\", author: ("Alice Smith", "Bob Jones"), date: datetime.today())"#));

        // 3. Fallback to config author when frontmatter author is absent
        let mut cfg_with_author = DocumentConfig::defaults();
        cfg_with_author.author = Some("Config Author".to_string());
        let fm_no_author: Value = serde_yaml::from_str(r#"
title: "Config Fallback"
"#).unwrap();
        let doc_cfg_auth = wrap_document("Hello world", &fm_no_author, &cfg_with_author, None, None, None, None);
        assert!(doc_cfg_auth.contains(r#"#set document(title: "Config Fallback", author: "Config Author")"#));

        // 4. CLI author overrides everything
        let doc_cli_auth = wrap_document("Hello world", &fm, &cfg_with_author, None, Some("CLI Override"), None, None);
        assert!(doc_cli_auth.contains(r#"author: "CLI Override""#));

        // 5. Empty metadata produces no #set document
        let empty_fm: Value = serde_yaml::from_str("{}").unwrap();
        let empty_cfg = DocumentConfig {
            author: None,
            ..DocumentConfig::defaults()
        };
        let doc_empty = wrap_document("Hello world", &empty_fm, &empty_cfg, None, None, None, None);
        assert!(!doc_empty.contains("#set document("));
    }

    #[test]
    fn test_document_themes_cascade_and_styles() {
        let default_cfg = DocumentConfig::defaults();

        // 1. Default is academic
        let empty_fm: Value = serde_yaml::from_str("{}").unwrap();
        let doc_acad = wrap_document("= Heading\nContent", &empty_fm, &default_cfg, None, None, None, None);
        assert!(doc_acad.contains("New Computer Modern"));
        assert!(doc_acad.contains("margin: (x: 2.5cm, y: 2.5cm)"));
        assert!(!doc_acad.contains("#show: columns.with"));

        // 2. Frontmatter theme: modern
        let modern_fm: Value = serde_yaml::from_str("theme: modern\ntitle: Modern Doc").unwrap();
        let doc_modern = wrap_document("= Modern Heading\nContent", &modern_fm, &default_cfg, None, None, None, None);
        assert!(doc_modern.contains("Liberation Sans"));
        assert!(doc_modern.contains("margin: (x: 2.4cm, y: 2.4cm)"));
        assert!(doc_modern.contains("rgb(\"#2563eb\")")); // Modern accent color
        assert!(doc_modern.contains("line(length: 100%, stroke: 1.2pt + rgb(\"#2563eb\"))"));

        // 3. CLI override theme
        let doc_cli_tech = wrap_document("= Tech Heading\nContent", &modern_fm, &default_cfg, None, None, None, Some("technical"));
        assert!(doc_cli_tech.contains("ENGINEERING SPECIFICATION"));
        assert!(doc_cli_tech.contains("margin: (x: 2cm, y: 2cm)"));
        assert!(doc_cli_tech.contains("rgb(\"#0f766e\")"));

        // 4. Compact theme (2 columns with gutter)
        let compact_fm: Value = serde_yaml::from_str("theme: compact\ntitle: Conference Paper").unwrap();
        let doc_compact = wrap_document("= Section\nBody text", &compact_fm, &default_cfg, None, None, None, None);
        assert!(doc_compact.contains("#show: columns.with(2, gutter: 14pt)"));
        assert!(doc_compact.contains("margin: (x: 1.8cm, y: 1.8cm)"));
        assert!(!doc_compact.contains("#place(top, float: true, scope: \"parent\")"));

        // 5. Minimal theme (unnumbered Swiss typography, 3cm margin)
        let minimal_fm: Value = serde_yaml::from_str("theme: minimal\ntitle: Swiss Typo").unwrap();
        let doc_minimal = wrap_document("= Section\nBody text", &minimal_fm, &default_cfg, None, None, None, None);
        assert!(doc_minimal.contains("margin: (x: 3cm, y: 3cm)"));
        assert!(!doc_minimal.contains("#set heading(numbering: \"1.1\")"));

        // 6. Elegant theme (2.8cm margins, warm styling)
        let elegant_fm: Value = serde_yaml::from_str("theme: elegant\ntitle: Monograph").unwrap();
        let doc_elegant = wrap_document("= Section\nBody text", &elegant_fm, &default_cfg, None, None, None, None);
        assert!(doc_elegant.contains("Linux Libertine"));
        assert!(doc_elegant.contains("margin: (x: 2.8cm, y: 2.8cm)"));
        assert!(doc_elegant.contains("rgb(\"#1c1917\")"));
    }

    #[test]
    fn test_custom_column_gutter() {
        let default_cfg = DocumentConfig::defaults();
        let fm: Value = serde_yaml::from_str("columns: 3\ncolumn-gutter: 20pt\ntitle: Tri-Column").unwrap();
        let doc = wrap_document("= Section\nBody", &fm, &default_cfg, None, None, None, None);
        assert!(doc.contains("#show: columns.with(3, gutter: 20pt)"));
    }

    #[test]
    fn test_column_pagebreak_sanitization() {
        let default_cfg = DocumentConfig::defaults();
        let fm: Value = serde_yaml::from_str("theme: compact\ntitle: Two Column").unwrap();
        let doc = wrap_document("= Col 1\n#pagebreak()\n= Col 2", &fm, &default_cfg, None, None, None, None);
        assert!(!doc.contains("#pagebreak()"));
        assert!(doc.contains("#colbreak()"));

        // Single column converts colbreak to pagebreak
        let fm_single: Value = serde_yaml::from_str("columns: 1").unwrap();
        let doc_single = wrap_document("= Sec 1\n#colbreak()\n= Sec 2", &fm_single, &default_cfg, None, None, None, None);
        assert!(!doc_single.contains("#colbreak()"));
        assert!(doc_single.contains("#pagebreak()"));
    }
}
