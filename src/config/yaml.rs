pub fn generate_config_yaml(
    author: &str,
    affiliation: Option<&str>,
    doc_theme: &str,
    slide_theme: &str,
    papersize: &str,
    toc: bool,
    bib: bool,
) -> String {
    let aff_line = match affiliation {
        Some(a) if !a.trim().is_empty() => format!("  affiliation: \"{}\"\n", a.trim()),
        _ => String::new(),
    };

    format!(
r#"# Botox Global Configuration
# Default style, author, and formatting for PDF documents and slides.

document:
  author: "{author}"
{aff_line}  theme: "{doc_theme}"                    # academic, modern, elegant, technical, compact, minimal
  fontsize: "11pt"
  mainfont: "New Computer Modern"       # True LaTeX font
  mathfont: "New Computer Modern Math"  # True LaTeX math font
  monofont: "DejaVu Sans Mono"          # Monospace font
  papersize: "{papersize}"              # a4 or us-letter
  columns: 1                            # 1 or 2 (multi-column)
  margin:
    x: "2.5cm"
    y: "2.5cm"
  section_numbering: true               # Numbered sections: 1, 1.1
  toc: {toc}                           # Enable Table of Contents by default
  bibliography: {bib}                   # Transform web links into an automatic IEEE bibliography
  lang: "en"

slides:
  theme: "{slide_theme}"                # default, academic, nord, dark
  author: "{author}"
  paginate: true
  font: "New Computer Modern"
"#
    )
}
