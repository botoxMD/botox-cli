use crate::config::SlidesConfig;
use serde_yaml::Value;

pub fn wrap_slides(slides_typst: &str, fm: &Value, config: &SlidesConfig) -> String {
    let mut out = String::new();

    let theme = fm.get("theme")
        .and_then(|v| v.as_str())
        .or_else(|| config.theme.as_deref())
        .unwrap_or("default");

    let (default_bg, default_text) = match theme {
        "dark" => ("#0f172a", "#f8fafc"),
        "nord" => ("#2e3440", "#eceff4"),
        "academic" => ("#ffffff", "#1a1a1a"),
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
        _ => "#0f172a",
    };
    let muted_color = match theme {
        "dark" | "nord" => "#94a3b8",
        _ => "#475569",
    };

    let footer_code = if paginate {
        "  footer: context {\n    let p = counter(page).get().first()\n    if p > 1 {\n      align(right, text(size: 12pt, fill: rgb(\"#64748b\"))[#p])\n    }\n  },\n"
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

    let raw_slides: Vec<&str> = slides_typst.split("#pagebreak()").collect();

    for (i, slide) in raw_slides.iter().enumerate() {
        let trimmed = slide.trim();
        if trimmed.is_empty() {
            continue;
        }

        if i > 0 {
            out.push_str("\n#pagebreak()\n\n");
        }

        if i == 0 && is_title_slide(trimmed) {
            out.push_str(&format!(
                "#place(center + horizon)[\n  #align(center)[\n    #show heading.where(level: 1): it => block(below: 0.6em)[#text(size: 2.1em, weight: \"bold\", fill: rgb(\"{heading_color}\"))[#it.body]]\n    #show heading.where(level: 3): it => block(below: 0.4em)[#text(size: 1.15em, style: \"italic\", fill: rgb(\"{muted_color}\"))[#it.body]]\n    {trimmed}\n  ]\n]\n"
            ));
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
        let typst1 = wrap_slides("= Title Slide", &empty_fm, &default_cfg);
        assert!(typst1.contains("rgb(\"#f8fafc\")"), "Should take built-in default background");

        // 2. Folder defines theme: academic, file defines nothing -> takes folder's "academic" (#ffffff)
        let mut folder_cfg = SlidesConfig::defaults();
        folder_cfg.theme = Some("academic".to_string());
        let typst2 = wrap_slides("= Title Slide", &empty_fm, &folder_cfg);
        assert!(typst2.contains("rgb(\"#ffffff\")"), "Should take folder's academic background");

        // 3. Folder defines theme: academic, but file defines theme: nord -> takes file's "nord" (#2e3440)
        let nord_fm = serde_yaml::from_str("theme: nord").unwrap();
        let typst3 = wrap_slides("= Title Slide", &nord_fm, &folder_cfg);
        assert!(typst3.contains("rgb(\"#2e3440\")"), "File's nord theme should override folder's academic theme");

        // 4. Folder defines theme: academic, file overrides color only -> theme is still academic, text is overridden
        let custom_fm = serde_yaml::from_str("color: \"#123456\"").unwrap();
        let typst4 = wrap_slides("= Title Slide", &custom_fm, &folder_cfg);
        assert!(typst4.contains("rgb(\"#ffffff\")"), "Should keep folder's academic background");
        assert!(typst4.contains("rgb(\"#123456\")"), "Should take file's custom text color");
    }
}
