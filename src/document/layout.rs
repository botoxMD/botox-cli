//! Document Layout Coordinator
//!
//! Coordinates document layout, multi-column flow, title/abstract headers,
//! and column pagebreak sanitization according to the selected `DocumentTheme`.

use crate::document::columns::sanitize_column_pagebreaks;
use crate::document::themes::DocumentTheme;
use crate::emitter::render_date_value;

/// Layout coordinator for standard articles, reports, and academic papers.
#[derive(Debug, Clone)]
pub struct DocumentLayout {
    pub theme: DocumentTheme,
    pub columns: usize,
    pub column_gutter: String,
}

impl DocumentLayout {
    pub fn new(theme: DocumentTheme, columns: usize, column_gutter: String) -> Self {
        Self {
            theme,
            columns,
            column_gutter,
        }
    }

    /// Renders the title block, authors, date, and abstract styled by theme.
    /// Floats across the top of columns when in multi-column mode.
    #[allow(clippy::too_many_arguments)]
    pub fn render_header(
        &self,
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

        let title_esc = title.map(crate::markdown::escapes::escape_typst_text);
        let title = title_esc.as_deref();
        let subtitle_esc = subtitle.map(crate::markdown::escapes::escape_typst_text);
        let subtitle = subtitle_esc.as_deref();
        let authors_escaped: Vec<String> = authors
            .iter()
            .map(|a| crate::markdown::escapes::escape_typst_text(a))
            .collect();
        let authors = &authors_escaped;
        let affiliation_esc = affiliation.map(crate::markdown::escapes::escape_typst_text);
        let affiliation = affiliation_esc.as_deref();
        let abstract_text_esc = abstract_text.map(crate::markdown::escapes::escape_typst_text);
        let abstract_text = abstract_text_esc.as_deref();
        let keywords_str_esc = keywords_str.map(crate::markdown::escapes::escape_typst_text);
        let keywords_str = keywords_str_esc.as_deref();

        let mut b = String::new();

        match self.theme {
            DocumentTheme::Modern => {
                if title.is_some() || !authors.is_empty() {
                    b.push_str("#block(\n  width: 100%,\n  stroke: (left: 4.5pt + rgb(\"#2563eb\")),\n  inset: (left: 14pt, top: 4pt, bottom: 4pt),\n)[\n  #set par(justify: false)\n");
                    if let Some(t) = title {
                        b.push_str(&format!(
                            "  #text(size: 2em, weight: \"bold\", fill: rgb(\"#0f172a\"))[{t}]\n"
                        ));
                    }
                    if let Some(sub) = subtitle {
                        b.push_str(&format!(
                            "  #v(0.3em)\n  #text(size: 1.15em, fill: rgb(\"#475569\"))[{sub}]\n"
                        ));
                    }
                    if !authors.is_empty() || affiliation.is_some() || date_str.is_some() {
                        b.push_str("  #v(0.5em)\n");
                        if !authors.is_empty() {
                            let joined = authors.join(", ");
                            b.push_str(&format!("  #text(size: 1.05em, weight: \"bold\", fill: rgb(\"#334155\"))[{joined}]\n"));
                        }
                        if let Some(aff) = affiliation
                            && !aff.is_empty()
                        {
                            b.push_str(&format!(
                                "  #v(0.2em)\n  #text(size: 0.95em, fill: rgb(\"#64748b\"))[{aff}]\n"
                            ));
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
                        b.push_str(&format!(
                            "  #v(0.2em)\n  #text(size: 1.05em, fill: rgb(\"#0f766e\"))[{sub}]\n"
                        ));
                    }
                    if !authors.is_empty() || date_str.is_some() || affiliation.is_some() {
                        b.push_str("  #v(0.4em)\n  #line(length: 100%, stroke: 0.5pt + rgb(\"#ccfbf1\"))\n  #v(0.3em)\n");
                        let authors_str = if !authors.is_empty() {
                            authors.join(", ")
                        } else {
                            String::new()
                        };
                        let aff_suffix = if let Some(aff) = affiliation {
                            format!(" ({aff})")
                        } else {
                            String::new()
                        };
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
                        b.push_str(&format!(
                            "  #text(size: 2em, weight: \"bold\", fill: rgb(\"#1c1917\"))[{t}]\n"
                        ));
                    }
                    if let Some(sub) = subtitle {
                        b.push_str(&format!("  #v(0.3em)\n  #text(size: 1.15em, style: \"italic\", fill: rgb(\"#44403c\"))[{sub}]\n"));
                    }
                    b.push_str(
                        "  #v(0.5em)\n  #line(length: 30%, stroke: 0.6pt + rgb(\"#a8a29e\"))\n  #v(0.5em)\n",
                    );
                    if !authors.is_empty() {
                        let joined = authors.join(", ");
                        b.push_str(&format!(
                            "  #text(size: 1.05em, fill: rgb(\"#292524\"))[{joined}]\n"
                        ));
                    }
                    if let Some(aff) = affiliation
                        && !aff.is_empty()
                    {
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
                        b.push_str(&format!(
                            "  #text(size: 2.2em, weight: \"bold\", fill: rgb(\"#18181b\"))[{t}]\n"
                        ));
                    }
                    if let Some(sub) = subtitle {
                        b.push_str(&format!(
                            "  #v(0.3em)\n  #text(size: 1.15em, fill: rgb(\"#71717a\"))[{sub}]\n"
                        ));
                    }
                    b.push_str("  #v(0.8em)\n");
                    if !authors.is_empty() {
                        let joined = authors.join(", ");
                        b.push_str(&format!(
                            "  #text(size: 1.0em, fill: rgb(\"#3f3f46\"))[{joined}]\n"
                        ));
                    }
                    if let Some(aff) = affiliation
                        && !aff.is_empty()
                    {
                        b.push_str(&format!(
                            "  #v(0.15em)\n  #text(size: 0.9em, fill: rgb(\"#71717a\"))[{aff}]\n"
                        ));
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
                        b.push_str(&format!(
                            "  #text(size: 0.9em, fill: rgb(\"#71717a\"))[{kw}]\n"
                        ));
                    }
                    b.push_str("]\n\n#v(1.8em)\n");
                }
            }
            DocumentTheme::Academic | DocumentTheme::Compact => {
                if title.is_some() || !authors.is_empty() {
                    b.push_str("#align(center)[\n  #set par(justify: false)\n");
                    let title_size = if self.theme == DocumentTheme::Compact {
                        "1.6em"
                    } else {
                        "1.8em"
                    };
                    if let Some(t) = title {
                        b.push_str(&format!(
                            "  #text(size: {title_size}, weight: \"bold\")[{t}]\n\n"
                        ));
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
                        && !aff.is_empty()
                    {
                        b.push_str(&format!(
                            "  #v(0.2em)\n  #text(size: 0.95em, style: \"italic\")[{aff}]\n"
                        ));
                    }
                    if let Some(d) = date_str {
                        let date_typst = render_date_value(d);
                        b.push_str(&format!("  #v(0.4em)\n  #text(size: 0.9em, fill: rgb(\"#475569\"))[{date_typst}]\n"));
                    }
                    b.push_str("]\n\n#v(1.2em)\n");
                }
                if abstract_text.is_some() || keywords_str.is_some() {
                    let abs_width = if self.theme == DocumentTheme::Compact {
                        "92%"
                    } else {
                        "88%"
                    };
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

        if self.columns > 1 && !b.trim().is_empty() {
            format!("#place(top, float: true, scope: \"parent\")[\n{b}]\n\n")
        } else {
            b
        }
    }

    /// Sanitizes body content for multi-column or single-column layout.
    pub fn prepare_body(&self, body_typst: &str) -> String {
        let clean_body = body_typst.replace("#botox_pause()", "");
        sanitize_column_pagebreaks(&clean_body, self.columns)
    }
}
