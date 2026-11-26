use crate::config::BotoxConfig;
use std::path::Path;

/// Metadata and preset definition for a Botox document/slide header template.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeaderTemplate {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub description: &'static str,
    pub layout_summary: &'static str,
    pub is_slides: bool,
}

pub const TEMPLATES: &[HeaderTemplate] = &[
    HeaderTemplate {
        name: "cheatsheet",
        aliases: &["cheat", "sheet", "refcard", "reference"],
        description: "Ultra-dense 2-column A4 reference sheet with minimal margins & compact code",
        layout_summary: "A4 2-col | margin 0.35cm | 8.1pt / 7pt code | compact",
        is_slides: false,
    },
    HeaderTemplate {
        name: "paper",
        aliases: &["academic", "ieee", "arxiv"],
        description: "Formal 2-column academic research paper with abstract & IEEE bibliography",
        layout_summary: "A4 2-col | margin 1.7x2cm | 10pt / 8pt code | academic",
        is_slides: false,
    },
    HeaderTemplate {
        name: "report",
        aliases: &["technical", "rfc", "spec", "design"],
        description: "Engineering design doc / RFC with TOC, numbered sections & Kroki diagrams",
        layout_summary: "A4 1-col | margin 2cm | 10pt / 8.5pt code | technical",
        is_slides: false,
    },
    HeaderTemplate {
        name: "notes",
        aliases: &["study", "lecture", "guide"],
        description: "Structured study guide or lecture notes with callouts & scannable TOC",
        layout_summary: "A4 1-col | margin 2.2x2cm | 10.5pt / 8.5pt code | modern",
        is_slides: false,
    },
    HeaderTemplate {
        name: "cv",
        aliases: &["resume"],
        description: "Crisp single-page resume / CV with tight margins & Swiss typography",
        layout_summary: "A4 1-col | margin 1.4x1.2cm | 9.5pt / 8pt code | minimal",
        is_slides: false,
    },
    HeaderTemplate {
        name: "essay",
        aliases: &["book", "thesis", "monograph"],
        description: "Long-form essay or manuscript with classic book margins & serif leading",
        layout_summary: "A4 1-col | margin 3x2.8cm | 11pt / 9pt code | elegant",
        is_slides: false,
    },
    HeaderTemplate {
        name: "memo",
        aliases: &["brief", "letter"],
        description: "Concise executive memo or brief with distraction-free hierarchy",
        layout_summary: "A4 1-col | margin 2.5x2.2cm | 10.5pt / 8.5pt code | minimal",
        is_slides: false,
    },
    HeaderTemplate {
        name: "landscape",
        aliases: &["matrix", "wide", "poster"],
        description: "Wide 3-column horizontal A4 layout for comparison matrices & system maps",
        layout_summary: "A4-L 3-col | margin 1x0.9cm | 8.5pt / 7.5pt code | compact",
        is_slides: false,
    },
    HeaderTemplate {
        name: "slides",
        aliases: &["deck", "presentation", "pitch"],
        description: "Interactive 16:9 presentation slide deck (supports --theme overrides)",
        layout_summary: "16:9 deck | 20pt / 15pt code | paginated",
        is_slides: true,
    },
];

pub fn find_template(name: &str) -> Option<&'static HeaderTemplate> {
    let norm = name.trim().to_ascii_lowercase();
    TEMPLATES
        .iter()
        .find(|t| t.name == norm || t.aliases.iter().any(|a| *a == norm))
}

pub fn format_template_list() -> String {
    let mut out = String::from("Available Botox header templates:\n\n");
    for t in TEMPLATES {
        out.push_str(&format!(
            "  {:<12} {}\n               [{}]\n",
            t.name, t.description, t.layout_summary
        ));
    }
    out.push_str("\nUsage:\n");
    out.push_str("  botox init <file.md> --template <name>   Create a new file from a template\n");
    out.push_str("  botox template <name>                    Print only the YAML frontmatter header\n");
    out
}

fn resolved_author(cfg: &BotoxConfig) -> String {
    cfg.document
        .as_ref()
        .and_then(|d| d.author.clone())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| {
            std::env::var("USER")
                .or_else(|_| std::env::var("USERNAME"))
                .unwrap_or_else(|_| "Author Name".to_string())
        })
}

fn resolved_affiliation(cfg: &BotoxConfig) -> String {
    cfg.document
        .as_ref()
        .and_then(|d| d.affiliation.clone())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "Systems Engineering".to_string())
}

fn resolved_lang(cfg: &BotoxConfig) -> String {
    cfg.document
        .as_ref()
        .and_then(|d| d.lang.clone())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "en".to_string())
}

fn resolved_papersize(cfg: &BotoxConfig) -> String {
    cfg.document
        .as_ref()
        .and_then(|d| d.papersize.clone())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "a4".to_string())
}

fn resolved_kroki_url(cfg: &BotoxConfig) -> String {
    cfg.document
        .as_ref()
        .and_then(|d| d.kroki_url.clone())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "https://kroki.io".to_string())
}

/// Renders only the YAML frontmatter header block (`---\n...\n---\n`) for a template.
pub fn render_template_header(
    tpl: &HeaderTemplate,
    cfg: &BotoxConfig,
    cli_theme: Option<&str>,
) -> String {
    let author = resolved_author(cfg);
    let affiliation = resolved_affiliation(cfg);
    let lang = resolved_lang(cfg);
    let papersize = resolved_papersize(cfg);
    let kroki = resolved_kroki_url(cfg);

    match tpl.name {
        "cheatsheet" => {
            let theme = cli_theme.unwrap_or("compact");
            format!(
                r#"---
author: ""
lang: {lang}
papersize: {papersize}
fontsize: 8.1pt
code-fontsize: 7pt
columns: 2
column-gutter: 8pt
linestretch: 1.01
geometry: "margin=0.35cm"
theme: {theme}
mainfont: "Liberation Sans"
monofont: "DejaVu Sans Mono"
number-sections: false
table-of-contents: false
bibliography: false
kroki_url: "{kroki}"
---
"#
            )
        }
        "paper" => {
            let theme = cli_theme.unwrap_or("academic");
            format!(
                r#"---
title: "Paper Title"
subtitle: "Optional Subtitle"
author:
  - name: "{author}"
    affiliation: "{affiliation}"
date: \today
abstract: >
  Concise summary of the problem statement, methodology, and principal findings.
keywords: "keyword one, keyword two, keyword three"
lang: {lang}
papersize: {papersize}
theme: {theme}
columns: 2
column-gutter: 16pt
margin:
  x: "1.7cm"
  y: "2cm"
fontsize: 10pt
code-fontsize: 8pt
linestretch: 1.05
indent: true
mainfont: "New Computer Modern"
monofont: "DejaVu Sans Mono"
mathfont: "New Computer Modern Math"
number-sections: true
table-of-contents: false
bibliography: true
---
"#
            )
        }
        "report" => {
            let theme = cli_theme.unwrap_or("technical");
            format!(
                r#"---
title: "Technical Design Document"
subtitle: "RFC-001 — System Architecture & Implementation Plan"
author:
  - name: "{author}"
    affiliation: "{affiliation}"
date: \today
abstract: >
  Executive summary of the proposed architecture, trade-offs, and rollout strategy.
lang: {lang}
papersize: {papersize}
theme: {theme}
columns: 1
margin:
  x: "2cm"
  y: "2cm"
fontsize: 10pt
code-fontsize: 8.5pt
linestretch: 1.12
mainfont: "Liberation Sans"
monofont: "DejaVu Sans Mono"
number-sections: true
table-of-contents: true
toc-depth: 3
bibliography: true
kroki_url: "{kroki}"
---
"#
            )
        }
        "notes" => {
            let theme = cli_theme.unwrap_or("modern");
            format!(
                r#"---
title: "Course & Study Notes"
subtitle: "Topic Summary & Key Concepts"
author: "{author}"
date: \today
lang: {lang}
papersize: {papersize}
theme: {theme}
columns: 1
margin:
  x: "2.2cm"
  y: "2cm"
fontsize: 10.5pt
code-fontsize: 8.5pt
linestretch: 1.15
mainfont: "Liberation Sans"
monofont: "DejaVu Sans Mono"
number-sections: true
table-of-contents: true
toc-depth: 2
bibliography: false
kroki_url: "{kroki}"
---
"#
            )
        }
        "cv" => {
            let theme = cli_theme.unwrap_or("minimal");
            format!(
                r##"---
title: "{author}"
subtitle: "Role Title · email@example.com · github.com/username"
lang: {lang}
papersize: {papersize}
theme: {theme}
columns: 1
margin:
  x: "1.4cm"
  y: "1.2cm"
fontsize: 9.5pt
code-fontsize: 8pt
linestretch: 1.05
mainfont: "Liberation Sans"
monofont: "DejaVu Sans Mono"
linkcolor: "#18181b"
number-sections: false
table-of-contents: false
bibliography: false
---
"##
            )
        }
        "essay" => {
            let theme = cli_theme.unwrap_or("elegant");
            format!(
                r#"---
title: "Essay Title"
subtitle: "A Long-Form Exploration"
author: "{author}"
date: \today
lang: {lang}
papersize: {papersize}
theme: {theme}
columns: 1
margin:
  x: "3cm"
  y: "2.8cm"
fontsize: 11pt
code-fontsize: 9pt
linestretch: 1.3
indent: true
mainfont: "Linux Libertine"
mathfont: "New Computer Modern Math"
number-sections: false
table-of-contents: true
toc-depth: 2
bibliography: true
---
"#
            )
        }
        "memo" => {
            let theme = cli_theme.unwrap_or("minimal");
            format!(
                r#"---
title: "Executive Memo"
subtitle: "Briefing & Decision Summary"
author: "{author}"
date: \today
lang: {lang}
papersize: {papersize}
theme: {theme}
columns: 1
margin:
  x: "2.5cm"
  y: "2.2cm"
fontsize: 10.5pt
code-fontsize: 8.5pt
linestretch: 1.15
mainfont: "Liberation Sans"
number-sections: false
table-of-contents: false
bibliography: false
---
"#
            )
        }
        "landscape" => {
            let theme = cli_theme.unwrap_or("compact");
            format!(
                r#"---
title: "System Overview & Reference Matrix"
lang: {lang}
papersize: {papersize}
landscape: true
theme: {theme}
columns: 3
column-gutter: 12pt
margin:
  x: "1cm"
  y: "0.9cm"
fontsize: 8.5pt
code-fontsize: 7.5pt
linestretch: 1.05
mainfont: "Liberation Sans"
monofont: "DejaVu Sans Mono"
number-sections: false
table-of-contents: false
bibliography: false
kroki_url: "{kroki}"
---
"#
            )
        }
        "slides" => {
            let slides_cfg = cfg.slides.clone().unwrap_or_default();
            let theme = cli_theme
                .map(|s| s.to_string())
                .or(slides_cfg.theme)
                .unwrap_or_else(|| "default".to_string());
            let paginate = slides_cfg.paginate.unwrap_or(true);
            format!(
                r#"---
marp: true
title: "Presentation Title"
author: "{author}"
date: \today
theme: {theme}
paginate: {paginate}
font: "Liberation Sans"
code-fontsize: 15pt
---
"#
            )
        }
        _ => unreachable!(),
    }
}

/// Renders a complete starter document (YAML frontmatter + concise tailored body).
pub fn render_template_document(
    tpl: &HeaderTemplate,
    cfg: &BotoxConfig,
    cli_theme: Option<&str>,
    target_path: &Path,
) -> String {
    let header = render_template_header(tpl, cfg, cli_theme);
    let display_name = target_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("document.md");

    let body = match tpl.name {
        "cheatsheet" => format!(
            r#"
## Core Concepts

- **Item 1:** Concise definition or invariant
- **Item 2:** Complexity $O(n \log n)$ time, $O(n)$ space

```bash
botox {display_name} -o cheatsheet.pdf
```

## Quick Commands

| Command | Purpose |
| :--- | :--- |
| `botox {display_name}` | Compile to PDF |
| `botox -w {display_name}` | Watch & auto-recompile |
"#
        ),
        "paper" => r#"
# Introduction

State the research problem, motivation, and main contributions.

# Methodology

Define the formal model:

$$ \mathcal{L}(\theta) = \mathbb{E}_{x \sim p}\left[ -\log p_\theta(x) \right] $$

# Evaluation & Conclusion

Summarize experimental results and future work.
"#
        .to_string(),
        "report" => format!(
            r#"
\newpage

# Context & Goals

Describe the engineering problem, constraints, and non-goals.

# Proposed Architecture

```rust
fn main() {{
    println!("Compile with: botox {display_name}");
}}
```

# Trade-offs & Rollout Plan

Document risks, alternatives considered, and phased rollout milestones.
"#
        ),
        "notes" => r#"
# Key Concepts

> [!NOTE]
> Use callouts (`NOTE`, `TIP`, `IMPORTANT`, `WARNING`) to highlight definitions and exam traps.

## Formulas & Theorems

- Fundamental identity: $e^{i\pi} + 1 = 0$

## Summary Checklist

- Review core definitions
- Practice worked examples
"#
        .to_string(),
        "cv" => r#"
## Summary

Systems-focused engineer building reliable, high-performance software.

## Experience

**Senior Software Engineer** — Company Name *(2023 – Present)*
- Designed and shipped core infrastructure reducing latency by 40%.

## Education & Skills

- **M.S. Computer Science** — University Name
- **Languages:** Rust, TypeScript, Python, SQL
"#
        .to_string(),
        "essay" => r#"
# Opening

Begin with the central thesis and context.

# Development

Unfold the primary arguments with supporting evidence and citations.

# Conclusion

Synthesize the thesis and its broader implications.
"#
        .to_string(),
        "memo" => r#"
## Executive Summary

State the decision or recommendation in 2–3 sentences.

## Background

Key facts and context needed to evaluate the proposal.

## Next Steps

- **Owner:** Action item and target date
"#
        .to_string(),
        "landscape" => r#"
## Column 1 — Overview

High-level summary and invariants.

## Column 2 — Comparison Matrix

| Option | Pros | Cons |
| :--- | :--- | :--- |
| A | Fast | Higher memory |
| B | Compact | Higher latency |

## Column 3 — Key Commands

```bash size=7pt
botox landscape.md -o matrix.pdf
```
"#
        .to_string(),
        "slides" => r#"
# Presentation Title
### Subtitle or Presenter Name

---

# Agenda

- Context and motivation
- Architecture & key results
- Next steps

---

# Code & Math Highlights

```rust size=13pt
fn main() {
    println!("Hello from Botox slides!");
}
```

$$ \vec{F} = m \frac{d\vec{v}}{dt} $$
"#
        .to_string(),
        _ => unreachable!(),
    };

    format!("{header}{body}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_templates_compile_to_pdf() {
        let cfg = BotoxConfig::defaults();
        for tpl in TEMPLATES {
            let md = render_template_document(tpl, &cfg, None, Path::new("sample.md"));
            let (typst_markup, is_slides) = crate::engine::transpile_markdown_to_typst_markup(
                &md,
                None,
                &cfg,
                false,
                false,
                None,
                None,
                None,
                None,
                None,
                &[],
                &[],
                Some(Path::new("sample.md")),
            )
            .unwrap_or_else(|e| panic!("Template '{}' failed to transpile: {e}", tpl.name));
            assert_eq!(
                is_slides, tpl.is_slides,
                "Template '{}' slides detection mismatch",
                tpl.name
            );
            let pdf_res = crate::compiler::compile_typst_to_pdf_bytes(&typst_markup, None);
            assert!(
                pdf_res.is_ok(),
                "Template '{}' failed to compile to PDF: {:?}",
                tpl.name,
                pdf_res.err()
            );
        }
    }

    #[test]
    fn test_template_lookup_and_aliases() {
        assert_eq!(find_template("cheatsheet").unwrap().name, "cheatsheet");
        assert_eq!(find_template("cheat").unwrap().name, "cheatsheet");
        assert_eq!(find_template("resume").unwrap().name, "cv");
        assert_eq!(find_template("IEEE").unwrap().name, "paper");
        assert_eq!(find_template("rfc").unwrap().name, "report");
        assert!(find_template("nonexistent").is_none());
    }
}
