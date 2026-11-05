#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentTheme {
    Academic,
    Modern,
    Elegant,
    Technical,
    Compact,
    Minimal,
}

impl DocumentTheme {
    pub fn parse(name: &str) -> Self {
        match name.trim().to_lowercase().as_str() {
            "modern" => Self::Modern,
            "elegant" => Self::Elegant,
            "technical" | "tech" => Self::Technical,
            "compact" | "ieee" | "acm" => Self::Compact,
            "minimal" | "swiss" => Self::Minimal,
            _ => Self::Academic,
        }
    }

    pub fn default_font(&self) -> &'static str {
        match self {
            Self::Academic => "New Computer Modern",
            Self::Modern => "Liberation Sans",
            Self::Elegant => "Linux Libertine",
            Self::Technical => "Liberation Sans",
            Self::Compact => "Times New Roman",
            Self::Minimal => "Liberation Sans",
        }
    }

    pub fn default_fontsize(&self) -> &'static str {
        match self {
            Self::Academic => "11pt",
            Self::Modern => "10.5pt",
            Self::Elegant => "11pt",
            Self::Technical => "10pt",
            Self::Compact => "10pt",
            Self::Minimal => "11pt",
        }
    }

    pub fn default_margins(&self) -> (&'static str, &'static str) {
        match self {
            Self::Academic => ("2.5cm", "2.5cm"),
            Self::Modern => ("2.4cm", "2.4cm"),
            Self::Elegant => ("2.8cm", "2.8cm"),
            Self::Technical => ("2cm", "2cm"),
            Self::Compact => ("1.8cm", "1.8cm"),
            Self::Minimal => ("3cm", "3cm"),
        }
    }

    pub fn default_columns(&self) -> usize {
        match self {
            Self::Compact => 2,
            _ => 1,
        }
    }

    pub fn default_linkcolor(&self) -> &'static str {
        match self {
            Self::Academic => "#0284c7",
            Self::Modern => "#2563eb",
            Self::Elegant => "#9a3412",
            Self::Technical => "#0d9488",
            Self::Compact => "#0284c7",
            Self::Minimal => "#18181b",
        }
    }

    pub fn default_text_color(&self) -> Option<&'static str> {
        match self {
            Self::Academic => None,
            Self::Modern => Some("#1e293b"),
            Self::Elegant => Some("#262626"),
            Self::Technical => Some("#0f172a"),
            Self::Compact => None,
            Self::Minimal => Some("#18181b"),
        }
    }

    pub fn default_numbering(&self) -> bool {
        !matches!(self, Self::Minimal)
    }

    pub fn default_leading(&self) -> &'static str {
        match self {
            Self::Compact => "0.65em",
            Self::Technical => "0.68em",
            Self::Modern => "0.72em",
            Self::Academic => "0.7em",
            Self::Elegant => "0.8em",
            Self::Minimal => "0.8em",
        }
    }

    pub fn callout_preamble(&self) -> String {
        match self {
            Self::Minimal => {
                "#let botox_callout(kind, title, body) = {\n  let stroke_color = rgb(\"#71717a\");\n  let default_title = if kind == \"warning\" { \"Warning\" } else if kind == \"tip\" { \"Tip\" } else if kind == \"important\" { \"Important\" } else { \"Note\" };\n  let display_title = if title != \"\" { title } else { default_title };\n  block(\n    stroke: (left: 2pt + stroke_color),\n    inset: (left: 10pt, y: 4pt),\n    width: 100%,\n    above: 1.2em,\n    below: 1.2em,\n  )[\n    #text(weight: \"bold\", fill: stroke_color)[#display_title]\\\n    #v(0.2em)\n    #body\n  ]\n}\n\n".to_string()
            }
            _ => {
                let (note_stroke, note_fill) = match self {
                    Self::Modern => ("#2563eb", "#eff6ff"),
                    Self::Technical => ("#0284c7", "#f0f9ff"),
                    Self::Elegant => ("#0369a1", "#f0f9ff"),
                    _ => ("#0284c7", "#f1f5f9"),
                };
                let (tip_stroke, tip_fill) = match self {
                    Self::Technical => ("#0d9488", "#f0fdfa"),
                    Self::Elegant => ("#15803d", "#f0fdf4"),
                    _ => ("#16a34a", "#f0fdf4"),
                };
                let (warn_stroke, warn_fill) = match self {
                    Self::Elegant => ("#b45309", "#fffbeb"),
                    _ => ("#d97706", "#fffbeb"),
                };
                let (danger_stroke, danger_fill) = match self {
                    Self::Technical => ("#e11d48", "#fff1f2"),
                    Self::Elegant => ("#b91c1c", "#fef2f2"),
                    _ => ("#dc2626", "#fef2f2"),
                };
                let stroke_spec = match self {
                    Self::Technical => "1pt + stroke_color",
                    Self::Modern => "(left: 4pt + stroke_color)",
                    _ => "(left: 3.5pt + stroke_color)",
                };
                let radius_spec = match self {
                    Self::Modern => "radius: 4pt,\n",
                    Self::Technical => "radius: 2pt,\n",
                    Self::Elegant => "radius: (right: 3pt),\n",
                    _ => "radius: (right: 4pt),\n",
                };
                format!(
r#"#let botox_callout(kind, title, body) = {{
  let (stroke_color, fill_color, default_title) = if kind == "warning" {{
    (rgb("{warn_stroke}"), rgb("{warn_fill}"), "Warning")
  }} else if kind == "tip" {{
    (rgb("{tip_stroke}"), rgb("{tip_fill}"), "Tip")
  }} else if kind == "important" or kind == "caution" or kind == "danger" {{
    (rgb("{danger_stroke}"), rgb("{danger_fill}"), "Important")
  }} else {{
    (rgb("{note_stroke}"), rgb("{note_fill}"), "Note")
  }};
  let display_title = if title != "" {{ title }} else {{ default_title }};
  block(
    fill: fill_color,
    stroke: {stroke_spec},
    inset: (x: 12pt, y: 10pt),
    {radius_spec}    width: 100%,
    above: 1.2em,
    below: 1.2em,
  )[
    #text(weight: "bold", fill: stroke_color)[#display_title]\
    #v(0.3em)
    #body
  ]
}}

"#
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_theme_parsing_and_aliases() {
        assert_eq!(DocumentTheme::parse("academic"), DocumentTheme::Academic);
        assert_eq!(DocumentTheme::parse("ACADEMIC"), DocumentTheme::Academic);
        assert_eq!(DocumentTheme::parse("modern"), DocumentTheme::Modern);
        assert_eq!(DocumentTheme::parse("elegant"), DocumentTheme::Elegant);
        assert_eq!(DocumentTheme::parse("technical"), DocumentTheme::Technical);
        assert_eq!(DocumentTheme::parse("tech"), DocumentTheme::Technical);
        assert_eq!(DocumentTheme::parse("compact"), DocumentTheme::Compact);
        assert_eq!(DocumentTheme::parse("ieee"), DocumentTheme::Compact);
        assert_eq!(DocumentTheme::parse("acm"), DocumentTheme::Compact);
        assert_eq!(DocumentTheme::parse("minimal"), DocumentTheme::Minimal);
        assert_eq!(DocumentTheme::parse("swiss"), DocumentTheme::Minimal);
        assert_eq!(DocumentTheme::parse("unknown_theme"), DocumentTheme::Academic);
    }

    #[test]
    fn test_theme_properties_consistency() {
        let all_themes = [
            DocumentTheme::Academic,
            DocumentTheme::Modern,
            DocumentTheme::Elegant,
            DocumentTheme::Technical,
            DocumentTheme::Compact,
            DocumentTheme::Minimal,
        ];

        for t in &all_themes {
            assert!(!t.default_font().is_empty());
            assert!(!t.default_fontsize().is_empty());
            let (mx, my) = t.default_margins();
            assert!(!mx.is_empty() && !my.is_empty());
            assert!(t.default_linkcolor().starts_with('#'));
            assert!(!t.default_leading().is_empty());
        }

        assert_eq!(DocumentTheme::Compact.default_columns(), 2);
        assert_eq!(DocumentTheme::Academic.default_columns(), 1);
        assert!(!DocumentTheme::Minimal.default_numbering());
        assert!(DocumentTheme::Academic.default_numbering());
    }
}
