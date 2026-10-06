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
        .and_then(|v| v.as_str())
        .unwrap_or(default_bg);

    let text_color = fm.get("color")
        .and_then(|v| v.as_str())
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
