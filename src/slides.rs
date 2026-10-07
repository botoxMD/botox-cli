use crate::config::SlidesConfig;
use serde_yaml::Value;

pub fn wrap_slides(
    slides_typst: &str,
    fm: &Value,
    config: &SlidesConfig,
    cli_author: Option<&str>,
    cli_theme: Option<&str>,
) -> String {
    let mut out = String::new();

    // 0. PDF Document Metadata
    let title = crate::document::extract_title(fm).or_else(|| {
        let first_slide = slides_typst.split("#pagebreak()").next()?;
        for line in first_slide.lines() {
            let trimmed = line.trim();
            if let Some(h) = trimmed.strip_prefix("= ") {
                return Some(h.trim().to_string());
            }
        }
        None
    });
    let authors = crate::document::extract_authors(fm, config.author.as_deref(), cli_author);
    let date_str = crate::document::extract_date(fm);

    if let Some(doc_meta) = crate::document::build_document_metadata(title.as_deref(), &authors, date_str.as_deref()) {
        out.push_str(&doc_meta);
    }

    let theme = cli_theme
        .or_else(|| fm.get("theme").and_then(|v| v.as_str()))
        .or_else(|| config.theme.as_deref())
        .unwrap_or("default");

    let (default_bg, default_text) = match theme {
        "dark" => ("#0f172a", "#f8fafc"),
        "nord" => ("#2e3440", "#eceff4"),
        "academic" => ("#ffffff", "#1a1a1a"),
        "gaia" => ("#fbfbf8", "#333333"),
        "uncover" => ("#fafafa", "#0f172a"),
        _ => ("#f8fafc", "#0f172a"),
    };

    let bg_color = fm.get("backgroundColor")
        .or_else(|| fm.get("background_color"))
        .or_else(|| fm.get("background-color"))
        .and_then(|v| v.as_str())
        .or_else(|| config.background_color.as_deref())
        .unwrap_or(default_bg);

    let text_color = fm.get("color")
        .and_then(|v| v.as_str())
        .or_else(|| config.color.as_deref())
        .unwrap_or(default_text);

    let font = fm.get("font")
        .and_then(|v| v.as_str())
        .or_else(|| config.font.as_deref())
        .unwrap_or("New Computer Modern");

    let paginate = fm.get("paginate")
        .and_then(|v| v.as_bool())
        .or(config.paginate)
        .unwrap_or(true);

    let heading_color = match theme {
        "dark" | "nord" => "#f8fafc",
        "gaia" => "#903020",
        "uncover" => "#0284c7",
        _ => "#0f172a",
    };
    let muted_color = match theme {
        "dark" | "nord" => "#94a3b8",
        "gaia" => "#85756c",
        "uncover" => "#64748b",
        _ => "#475569",
    };

    let footer_code = if paginate {
        "  footer: context {\n    let p = counter(\"slide\").get().first()\n    if p > 1 {\n      align(right, text(size: 12pt, fill: rgb(\"#64748b\"))[#p])\n    }\n  },\n"
    } else {
        ""
    };

    out.push_str(&format!(
        "#set page(\n  paper: \"presentation-16-9\",\n  margin: (x: 2.4cm, top: 2.2cm, bottom: 2cm),\n  fill: rgb(\"{bg_color}\"),\n{footer_code})\n"
    ));

    out.push_str(&format!(
        "#set text(\n  font: (\"{font}\", \"New Computer Modern\"),\n  size: 20pt,\n  fill: rgb(\"{text_color}\"),\n)\n"
    ));

    out.push_str("#set par(justify: false, leading: 0.75em)\n");
    out.push_str("#set list(spacing: 1.1em, marker: [•])\n");
    out.push_str("#set enum(spacing: 1.1em)\n\n");

    // Headings on content slides
    out.push_str(&format!(
        "#show heading.where(level: 1): it => block(below: 1.2em)[#text(size: 1.5em, weight: \"bold\", fill: rgb(\"{heading_color}\"))[#it.body]]\n"
    ));
    out.push_str(&format!(
        "#show heading.where(level: 2): it => block(below: 1.0em)[#text(size: 1.25em, weight: \"bold\", fill: rgb(\"{heading_color}\"))[#it.body]]\n"
    ));
    out.push_str(&format!(
        "#show heading.where(level: 3): it => block(below: 0.8em)[#text(size: 1.1em, weight: \"medium\", fill: rgb(\"{muted_color}\"))[#it.body]]\n\n"
    ));

    // Code blocks in slides
    out.push_str("#show raw: set text(font: \"DejaVu Sans Mono\")\n");
    out.push_str("#show raw.where(block: true): it => block(\n");
    out.push_str("  fill: rgb(\"#ffffff\").transparentize(60%),\n");
    out.push_str("  stroke: 0.5pt + rgb(\"#cbd5e1\"),\n");
    out.push_str("  inset: (x: 18pt, y: 14pt),\n");
    out.push_str("  radius: 6pt,\n");
    out.push_str("  width: 100%,\n");
    out.push_str(")[#set text(size: 15pt); #it]\n");
    out.push_str("#show raw.where(block: false): it => box(\n");
    out.push_str("  fill: rgb(\"#ffffff\").transparentize(40%),\n");
    out.push_str("  inset: (x: 4pt, y: 2pt),\n");
    out.push_str("  radius: 3pt,\n");
    out.push_str(")[#set text(size: 0.9em); #it]\n\n");

    // Callouts prelude for slides
    out.push_str("#let botox_callout(kind, title, body) = {\n");
    out.push_str("  let (stroke_color, default_title) = if kind == \"warning\" {\n");
    out.push_str("    (rgb(\"#d97706\"), \"Warning\")\n");
    out.push_str("  } else if kind == \"tip\" {\n");
    out.push_str("    (rgb(\"#16a34a\"), \"Tip\")\n");
    out.push_str("  } else if kind == \"important\" or kind == \"caution\" or kind == \"danger\" {\n");
    out.push_str("    (rgb(\"#dc2626\"), \"Important\")\n");
    out.push_str("  } else {\n");
    out.push_str("    (rgb(\"#0284c7\"), \"Note\")\n");
    out.push_str("  };\n");
    out.push_str("  let display_title = if title != \"\" { title } else { default_title };\n");
    out.push_str("  block(\n");
    out.push_str("    fill: stroke_color.transparentize(88%),\n");
    out.push_str("    stroke: (left: 4pt + stroke_color),\n");
    out.push_str("    inset: (x: 16pt, y: 12pt),\n");
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

    out.push_str("#let botox_slide_counter = counter(\"slide\")\n");
    out.push_str("#botox_slide_counter.step()\n\n");

    let raw_slides: Vec<&str> = slides_typst.split("#pagebreak()").collect();
    let mut first_emitted = false;

    for (i, slide) in raw_slides.iter().enumerate() {
        let trimmed = slide.trim();
        if trimmed.is_empty() {
            continue;
        }

        let is_first_logical_slide = !first_emitted;
        first_emitted = true;

        if !is_first_logical_slide {
            out.push_str("\n#pagebreak()\n#botox_slide_counter.step()\n\n");
        }

        if i == 0 && is_title_slide(trimmed) {
            out.push_str(&format!(
                "#place(center + horizon)[\n  #align(center)[\n    #show heading.where(level: 1): it => block(below: 0.6em)[#text(size: 2.1em, weight: \"bold\", fill: rgb(\"{heading_color}\"))[#it.body]]\n    #show heading.where(level: 3): it => block(below: 0.4em)[#text(size: 1.15em, style: \"italic\", fill: rgb(\"{muted_color}\"))[#it.body]]\n    {trimmed}\n  ]\n]\n"
            ));
        } else if trimmed.contains("#botox_pause()") {
            let chunks: Vec<&str> = trimmed.split("#botox_pause()").collect();
            let mut accumulated = String::new();
            for (step_idx, chunk) in chunks.iter().enumerate() {
                let c = chunk.trim();
                if c.is_empty() && step_idx > 0 {
                    continue;
                }
                if step_idx > 0 {
                    out.push_str("\n#pagebreak()\n\n");
                }
                if !accumulated.is_empty() {
                    accumulated.push_str("\n\n");
                }
                accumulated.push_str(c);
                out.push_str(&accumulated);
            }
            out.push('\n');
        } else {
            out.push_str(trimmed);
            out.push('\n');
        }
    }

    out
}

fn is_title_slide(slide_text: &str) -> bool {
    let has_h1 = slide_text.lines().any(|l| l.starts_with("= "));
    let has_h2 = slide_text.lines().any(|l| l.starts_with("== "));
    has_h1 && !has_h2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slides_settings_cascade() {
        // 1. Neither folder nor file defines theme -> built-in default ("default" -> fill #f8fafc)
        let default_cfg = SlidesConfig::defaults();
        let empty_fm = serde_yaml::from_str("{}").unwrap();
        let typst1 = wrap_slides("= Title Slide", &empty_fm, &default_cfg, None, None);
        assert!(typst1.contains("rgb(\"#f8fafc\")"), "Should take built-in default background");

        // 2. Folder defines theme: academic, file defines nothing -> takes folder's "academic" (#ffffff)
        let mut folder_cfg = SlidesConfig::defaults();
        folder_cfg.theme = Some("academic".to_string());
        let typst2 = wrap_slides("= Title Slide", &empty_fm, &folder_cfg, None, None);
        assert!(typst2.contains("rgb(\"#ffffff\")"), "Should take folder's academic background");

        // 3. Folder defines theme: academic, but file defines theme: nord -> takes file's "nord" (#2e3440)
        let nord_fm = serde_yaml::from_str("theme: nord").unwrap();
        let typst3 = wrap_slides("= Title Slide", &nord_fm, &folder_cfg, None, None);
        assert!(typst3.contains("rgb(\"#2e3440\")"), "File's nord theme should override folder's academic theme");

        // 4. Folder defines theme: academic, file overrides color only -> theme is still academic, text is overridden
        let custom_fm = serde_yaml::from_str("color: \"#123456\"").unwrap();
        let typst4 = wrap_slides("= Title Slide", &custom_fm, &folder_cfg, None, None);
        assert!(typst4.contains("rgb(\"#ffffff\")"), "Should keep folder's academic background");
        assert!(typst4.contains("rgb(\"#123456\")"), "Should take file's custom text color");

        // 5. CLI theme overrides everything
        let typst5 = wrap_slides("= Title Slide", &nord_fm, &folder_cfg, None, Some("dark"));
        assert!(typst5.contains("rgb(\"#0f172a\")"), "CLI dark theme should override everything");
    }

    #[test]
    fn test_slides_document_metadata() {
        let default_cfg = SlidesConfig::defaults();

        // 1. Title, author, date from YAML frontmatter
        let fm: Value = serde_yaml::from_str(r#"
title: "AI Operating Systems"
author: "Minus"
date: "2026-10-07"
"#).unwrap();
        let typst = wrap_slides("= Slide 1\nContent", &fm, &default_cfg, None, None);
        assert!(typst.contains(r#"#set document(title: "AI Operating Systems", author: "Minus", date: datetime(year: 2026, month: 10, day: 7))"#));

        // 2. Title from first slide H1 when absent in frontmatter
        let empty_fm: Value = serde_yaml::from_str("{}").unwrap();
        let typst2 = wrap_slides("= First Slide Heading\nBody text", &empty_fm, &default_cfg, Some("CLI Presenter"), None);
        assert!(typst2.contains(r#"#set document(title: "First Slide Heading", author: "CLI Presenter")"#));

        // 3. Config author fallback
        let mut cfg_with_author = SlidesConfig::defaults();
        cfg_with_author.author = Some("Slides Config Author".to_string());
        let fm_title_only: Value = serde_yaml::from_str("title: Deck").unwrap();
        let typst3 = wrap_slides("Body", &fm_title_only, &cfg_with_author, None, None);
        assert!(typst3.contains(r#"#set document(title: "Deck", author: "Slides Config Author")"#));
    }

    #[test]
    fn test_slides_pause_incremental_stepping() {
        let default_cfg = SlidesConfig::defaults();
        let fm: Value = serde_yaml::from_str("{}").unwrap();
        let slide_content = "== Features\n- Point 1\n#botox_pause()\n- Point 2";
        let typst = wrap_slides(slide_content, &fm, &default_cfg, None, None);
        assert!(typst.contains("botox_slide_counter"));
        assert!(typst.contains("counter(\"slide\")"));
        // Sub-slide 0 should contain Point 1, Sub-slide 1 should contain Point 1 and Point 2
        assert!(typst.contains("== Features\n- Point 1"));
        assert!(typst.contains("== Features\n- Point 1\n\n- Point 2"));
    }
}
