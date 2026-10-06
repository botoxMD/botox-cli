use crate::config::{DocumentConfig, MarginConfig};
use serde_yaml::Value;

pub fn wrap_document(body_typst: &str, fm: &Value, config: &DocumentConfig, cli_toc: Option<bool>, cli_author: Option<&str>, cli_font: Option<&str>) -> String {
    let mut out = String::new();

    // 1. Paper and Margins
    let papersize = fm.get("papersize")
        .and_then(|v| v.as_str())
        .or_else(|| config.papersize.as_deref())
        .unwrap_or("a4");

    let (margin_x, margin_y) = parse_margins(fm, config);

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
        .or(config.columns)
        .unwrap_or(1);

    out.push_str(&format!(
        "#set page(\n  paper: \"{papersize}\",\n  margin: (x: {margin_x}, y: {margin_y}),\n  columns: {columns},\n  numbering: \"1\",\n)\n"
    ));

    // 2. Typography
    let lang = fm.get("lang")
        .and_then(|v| v.as_str())
        .or_else(|| config.lang.as_deref())
        .unwrap_or("en");

    let font_raw = cli_font
        .or_else(|| fm.get("mainfont").and_then(|v| v.as_str()))
        .or_else(|| config.mainfont.as_deref())
        .unwrap_or("New Computer Modern");

    let font_family = if font_raw.eq_ignore_ascii_case("Times New Roman") || font_raw.eq_ignore_ascii_case("Times") {
        "(\"Times New Roman\", \"Nimbus Roman\", \"Liberation Serif\", \"New Computer Modern\")".to_string()
    } else if font_raw.eq_ignore_ascii_case("New Computer Modern") {
        "(\"New Computer Modern\",)".to_string()
    } else {
        format!("(\"{font_raw}\", \"New Computer Modern\")")
    };

    let monofont_raw = fm.get("monofont")
        .and_then(|v| v.as_str())
        .or_else(|| config.monofont.as_deref())
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
            } else if let Some(n) = v.as_f64() {
                Some(format!("{n}pt"))
            } else {
                None
            }
        })
        .or_else(|| config.fontsize.clone())
        .unwrap_or_else(|| "11pt".to_string());

    out.push_str(&format!(
        "#set text(font: {font_family}, size: {fontsize}, lang: \"{lang}\")\n"
    ));

    // Math font & equations
    let mathfont = fm.get("mathfont")
        .and_then(|v| v.as_str())
        .or_else(|| config.mathfont.as_deref())
        .unwrap_or("New Computer Modern Math");
    out.push_str(&format!(
        "#show math.equation: set text(font: \"{mathfont}\")\n"
    ));
    out.push_str("#show math.equation.where(block: true): it => block(above: 1.2em, below: 1.2em)[#it]\n");
    out.push_str("#set math.equation(numbering: \"(1)\")\n\n");

    // Pandoc callouts prelude
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

    // Link color
    let linkcolor = fm.get("linkcolor")
        .and_then(|v| v.as_str())
        .or_else(|| config.linkcolor.as_deref());

    if let Some(lc) = linkcolor {
        let color_val = if lc.starts_with('#') {
            format!("rgb(\"{lc}\")")
        } else if matches!(lc, "blue" | "red" | "green" | "navy" | "maroon" | "purple" | "teal" | "olive" | "gray" | "black" | "orange") {
            lc.to_string()
        } else {
            format!("rgb(\"{lc}\")")
        };
        out.push_str(&format!("#show link: set text(fill: {color_val})\n"));
    }

    // Paragraph & Base Typography styling
    let linestretch = fm.get("linestretch")
        .or_else(|| fm.get("line-spacing"))
        .and_then(|v| v.as_f64());
    let leading_em = if let Some(ls) = linestretch {
        format!("{:.3}em", 0.7 * ls)
    } else {
        "0.7em".to_string()
    };

    let parindent = fm.get("indent")
        .or_else(|| fm.get("parindent"))
        .and_then(|v| {
            if let Some(b) = v.as_bool() {
                if b { Some("1.5em".to_string()) } else { None }
            } else if let Some(s) = v.as_str() {
                Some(s.to_string())
            } else {
                None
            }
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
    out.push_str("#show raw.where(block: true): it => block(\n");
    out.push_str("  fill: rgb(\"#f8fafc\"),\n");
    out.push_str("  stroke: 0.5pt + rgb(\"#e2e8f0\"),\n");
    out.push_str("  inset: (x: 12pt, y: 10pt),\n");
    out.push_str("  radius: 4pt,\n");
    out.push_str("  width: 100%,\n");
    out.push_str(")[#set text(size: 8.5pt); #it]\n");
    out.push_str("#show raw.where(block: false): it => box(\n");
    out.push_str("  fill: rgb(\"#f1f5f9\"),\n");
    out.push_str("  inset: (x: 3pt, y: 1.5pt),\n");
    out.push_str("  radius: 2pt,\n");
    out.push_str(")[#set text(size: 0.9em); #it]\n\n");

    // Academic Table styling (booktabs inspired)
    out.push_str("#show table: set table(\n");
    out.push_str("  inset: (x: 10pt, y: 7pt),\n");
    out.push_str("  stroke: (x, y) => if y == 0 { (bottom: 1.2pt + black) } else { (bottom: 0.5pt + rgb(\"#cbd5e1\")) },\n");
    out.push_str(")\n");
    out.push_str("#show table.cell.where(y: 0): set text(weight: \"bold\")\n\n");

    // Section numbering
    let numbering = fm.get("numbersections")
        .or_else(|| fm.get("number-sections"))
        .and_then(|v| v.as_bool())
        .or(config.section_numbering)
        .unwrap_or(true);

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

    out.push_str("#show heading.where(level: 1): it => block(above: 1.8em, below: 0.9em)[#it]\n");
    out.push_str("#show heading.where(level: 2): it => block(above: 1.4em, below: 0.7em)[#it]\n");
    out.push_str("#show heading.where(level: 3): it => block(above: 1.1em, below: 0.6em)[#it]\n\n");

    // 3. Title block
    let title = fm.get("title").and_then(|v| v.as_str());
    let subtitle = fm.get("subtitle").and_then(|v| v.as_str());

    let authors: Vec<String> = if let Some(ca) = cli_author {
        vec![ca.to_string()]
    } else if let Some(a) = fm.get("author") {
        if let Some(s) = a.as_str() {
            vec![s.to_string()]
        } else if let Some(arr) = a.as_sequence() {
            arr.iter().filter_map(|item| {
                if let Some(s) = item.as_str() {
                    Some(s.to_string())
                } else if let Some(name) = item.get("name").and_then(|v| v.as_str()) {
                    Some(name.to_string())
                } else {
                    None
                }
            }).collect()
        } else {
            Vec::new()
        }
    } else if let Some(ref ca) = config.author {
        vec![ca.clone()]
    } else {
        Vec::new()
    };

    let affiliation = if let Some(a) = fm.get("author") {
        if let Some(arr) = a.as_sequence() {
            arr.first().and_then(|item| item.get("affiliation").and_then(|v| v.as_str()))
        } else {
            fm.get("affiliation").and_then(|v| v.as_str())
        }
    } else {
        fm.get("affiliation").and_then(|v| v.as_str()).or_else(|| config.affiliation.as_deref())
    };

    let date_str = fm.get("date").and_then(|v| {
        if let Some(s) = v.as_str() {
            Some(s.to_string())
        } else if let Some(n) = v.as_i64() {
            Some(n.to_string())
        } else {
            None
        }
    });

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

    if title.is_some() || !authors.is_empty() {
        out.push_str("#align(center)[\n  #set par(justify: false)\n");
        if let Some(t) = title {
            out.push_str(&format!("  #text(size: 1.8em, weight: \"bold\")[{t}]\n\n"));
        }
        if let Some(sub) = subtitle {
            out.push_str(&format!("  #v(0.3em)\n  #text(size: 1.15em, style: \"italic\", fill: rgb(\"#334155\"))[{sub}]\n\n"));
        }
        out.push_str("  #v(0.6em)\n");
        if !authors.is_empty() {
            let joined = authors.join(", ");
            out.push_str(&format!("  #text(size: 1.1em)[{joined}]\n"));
        }
        if let Some(aff) = affiliation {
            if !aff.is_empty() {
                out.push_str(&format!("  #v(0.2em)\n  #text(size: 0.95em, style: \"italic\")[{aff}]\n"));
            }
        }
        if let Some(ref d) = date_str {
            let trimmed = d.trim();
            if trimmed == r"\today" || trimmed == "\\today" || trimmed.eq_ignore_ascii_case("today") {
                out.push_str("  #v(0.4em)\n  #text(size: 0.9em, fill: rgb(\"#475569\"))[#datetime.today().display(\"[day] [month repr:long] [year]\")]\n");
            } else {
                out.push_str(&format!("  #v(0.4em)\n  #text(size: 0.9em, fill: rgb(\"#475569\"))[{trimmed}]\n"));
            }
        }
        out.push_str("]\n\n#v(1.2em)\n");
    }

    if abstract_text.is_some() || keywords_str.is_some() {
        out.push_str("#align(center)[\n  #block(width: 88%)[\n    #set par(justify: true, leading: 0.6em)\n");
        if let Some(abs) = abstract_text {
            out.push_str(&format!("    #text(weight: \"bold\")[Abstract] -- #text(style: \"italic\")[{abs}]\n"));
        }
        if let Some(ref kw) = keywords_str {
            if abstract_text.is_some() {
                out.push_str("    #v(0.6em)\n");
            }
            out.push_str(&format!("    #text(weight: \"bold\")[Keywords] -- #text(style: \"italic\")[{kw}]\n"));
        }
        out.push_str("  ]\n]\n\n#v(1.4em)\n");
    }

    // 4. Table of Contents
    let should_include_toc = if let Some(cli) = cli_toc {
        cli
    } else if let Some(fm_toc) = fm.get("toc").or_else(|| fm.get("table-of-contents")).and_then(|v| v.as_bool()) {
        fm_toc
    } else {
        config.toc.unwrap_or(true)
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
        .or_else(|| config.toc_title.as_deref());

    if should_include_toc {
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
    out.push_str(body_typst);

    out
}

fn parse_margins(fm: &Value, config: &DocumentConfig) -> (String, String) {
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
                mx.unwrap_or_else(|| "2.5cm".to_string()),
                my.unwrap_or_else(|| "2.5cm".to_string()),
            );
        }
    }

    if let Some(ref m) = config.margin {
        match m {
            MarginConfig::Uniform(s) => (s.clone(), s.clone()),
            MarginConfig::Axes { x, y } => (
                x.clone().unwrap_or_else(|| "2.5cm".to_string()),
                y.clone().unwrap_or_else(|| "2.5cm".to_string()),
            ),
        }
    } else {
        ("2.5cm".to_string(), "2.5cm".to_string())
    }
}
