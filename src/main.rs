mod config;
mod markdown;
mod document;
mod slides;
mod compiler;

use std::path::PathBuf;
use serde_yaml::Value;

fn show_help() {
    println!(r#"Usage: botox <input.md> [options]
       botox <pdf|slides> <input.md> [options]
       botox init <filename> [options]
       botox config

Pure Rust single native binary: Transforms Markdown into LaTeX-quality PDF documents or presentation slides.
Mode is detected automatically from frontmatter structure, or specified explicitly.

Commands:
  init <filename>         Initialize a new Markdown document or presentation slide deck
  config                  Display currently active configuration settings and loaded sources

Common Options:
  -o, --output <file>     Output PDF file path (default: <input>.pdf)
  --config <file>         Custom configuration YAML path

Documentation Options:
  --pdf                   Force PDF documentation mode
  --toc                   Generate automatic Table of Contents
  --no-toc                Disable Table of Contents
  --author <name>         Author name (overrides frontmatter & config)
  --font <name>           Main font (default: New Computer Modern)
  -N, --number-sections   Number section headings
  -b, --bibliography      Transform web links into an IEEE-standard Bibliography
  --no-bibliography       Disable automatic Bibliography generation

Slide Options:
  --slides                Force presentation slide deck mode
"#);
}

fn handle_init(args: &[String]) {
    if args.is_empty() || args[0] == "-h" || args[0] == "--help" {
        println!(r#"Usage: botox init <filename> [options]

Initialize a new Markdown document or presentation slide deck with active default settings.

Arguments:
  <filename>              Target file path to create (e.g. document.md, slides.md)

Options:
  --slides                Initialize as a presentation slide deck
  --doc, --pdf            Initialize as a publication-grade LaTeX PDF document
  -f, --force             Overwrite target file if it already exists
  --theme <theme>         Slide theme (default: from config or 'default')
"#);
        return;
    }

    let mut target_filename: Option<PathBuf> = None;
    let mut explicit_slides = false;
    let mut explicit_doc = false;
    let mut force = false;
    let mut cli_theme: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--slides" | "slides" => explicit_slides = true,
            "--doc" | "--pdf" | "pdf" | "doc" => explicit_doc = true,
            "-f" | "--force" => force = true,
            "--theme" => {
                i += 1;
                if i < args.len() {
                    cli_theme = Some(args[i].clone());
                }
            }
            other => {
                if !other.starts_with('-') && target_filename.is_none() {
                    let mut path = PathBuf::from(other);
                    if path.extension().is_none() {
                        path.set_extension("md");
                    }
                    target_filename = Some(path);
                }
            }
        }
        i += 1;
    }

    let target_path = match target_filename {
        Some(p) => p,
        None => {
            eprintln!("Error: 'botox init' requires a target filename.");
            eprintln!("Example: botox init document.md   or   botox init slides.md");
            std::process::exit(1);
        }
    };

    if target_path.exists() && !force {
        eprintln!(
            "Error: File '{}' already exists. Use --force to overwrite.",
            target_path.display()
        );
        std::process::exit(1);
    }

    let fname_lower = target_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    let is_slides = explicit_slides
        || (!explicit_doc && (fname_lower.contains("slide") || fname_lower.contains("pres")));

    let doc_dir = target_path.parent();
    let (resolved_cfg, _) = config::BotoxConfig::load(None, doc_dir);

    let content = generate_init_template(is_slides, &resolved_cfg, cli_theme.as_deref(), &target_path);

    if let Some(parent) = target_path.parent() {
        if !parent.as_os_str().is_empty() {
            let _ = std::fs::create_dir_all(parent);
        }
    }

    match std::fs::write(&target_path, content) {
        Ok(()) => {
            let doc_type = if is_slides { "slide presentation" } else { "PDF document" };
            println!("Initialized new {} -> '{}'", doc_type, target_path.display());
        }
        Err(e) => {
            eprintln!("Error writing '{}': {e}", target_path.display());
            std::process::exit(1);
        }
    }
}

fn generate_init_template(
    is_slides: bool,
    resolved_cfg: &config::BotoxConfig,
    cli_theme: Option<&str>,
    target_path: &std::path::Path,
) -> String {
    if is_slides {
        let slides_cfg = resolved_cfg.slides.clone().unwrap_or_default();
        let theme = cli_theme
            .map(|s| s.to_string())
            .or(slides_cfg.theme)
            .unwrap_or_else(|| "default".to_string());
        let paginate = slides_cfg.paginate.unwrap_or(true);

        format!(
            r#"---
marp: true
theme: {theme}
paginate: {paginate}
---

# Presentation Title
### Presentation Subtitle or Presenter

---

# Executive Summary

- High performance, single-binary Markdown typesetting
- Direct compilation to 16:9 presentation slides
- Full support for math, code, tables, and images

---

# Architecture Highlights

Mathematical equations and vector graphics render natively:

$$ \vec{{F}} = m \frac{{d \vec{{v}}}}{{d t}} $$

- Point 1: Direct bypass forwarding
- Point 2: Zero runtime dependencies
"#
        )
    } else {
        let doc_cfg = resolved_cfg.document.clone().unwrap_or_default();
        let author = doc_cfg.author.unwrap_or_else(|| {
            std::env::var("USER")
                .or_else(|_| std::env::var("USERNAME"))
                .unwrap_or_else(|_| "Minus".to_string())
        });
        let affiliation = doc_cfg
            .affiliation
            .unwrap_or_else(|| "Systems Engineering".to_string());
        let lang = doc_cfg.lang.unwrap_or_else(|| "en".to_string());
        let papersize = doc_cfg.papersize.unwrap_or_else(|| "a4".to_string());
        let fontsize = doc_cfg.fontsize.unwrap_or_else(|| "11pt".to_string());
        let mainfont = doc_cfg
            .mainfont
            .unwrap_or_else(|| "New Computer Modern".to_string());
        let monofont = doc_cfg
            .monofont
            .unwrap_or_else(|| "DejaVu Sans Mono".to_string());
        let mathfont = doc_cfg
            .mathfont
            .unwrap_or_else(|| "New Computer Modern Math".to_string());
        let num_sections = doc_cfg.section_numbering.unwrap_or(true);
        let toc = doc_cfg.toc.unwrap_or(true);
        let toc_depth = doc_cfg.toc_depth.unwrap_or(3);
        let bib = doc_cfg.bibliography.unwrap_or(true);

        format!(
            r#"---
title: "Document Title"
subtitle: "Document Subtitle"
author:
  - name: "{author}"
    affiliation: "{affiliation}"
date: \today
lang: {lang}
papersize: {papersize}
fontsize: {fontsize}
mainfont: "{mainfont}"
monofont: "{monofont}"
mathfont: "{mathfont}"
geometry: "margin=2.5cm"
number-sections: {num_sections}
table-of-contents: {toc}
toc-depth: {toc_depth}
bibliography: {bib}
---

\newpage

# Introduction

Welcome to your new document. This file was initialized with your active Botox default settings.

## Getting Started

You can write standard Markdown and compile it directly using:

```bash
botox {display_name} -o output.pdf
```

## Mathematical Modeling

LaTeX mathematics is supported natively:

$$\int_{{-\infty}}^{{+\infty}} e^{{-x^2}} \, dx = \sqrt{{\pi}}$$

## References

Hyperlinks are automatically transformed into an IEEE-standard Bibliography:
Check out the [Botox Documentation](https://github.com) for more examples.
"#,
            display_name = target_path.display()
        )
    }
}

fn extract_frontmatter(content: &str) -> (Value, &str) {
    if content.starts_with("---") {
        let parts: Vec<&str> = content.splitn(3, "---").collect();
        if parts.len() >= 3 {
            if let Ok(val) = serde_yaml::from_str::<Value>(parts[1]) {
                return (val, parts[2].trim_start());
            }
            return (Value::Null, parts[2].trim_start());
        }
    }
    (Value::Null, content)
}

fn detect_is_slides(fm: &Value, explicit_slides: bool, explicit_pdf: bool) -> bool {
    if explicit_slides {
        return true;
    }
    if explicit_pdf {
        return false;
    }

    if let Some(map) = fm.as_mapping() {
        let marp_key = Value::String("marp".to_string());
        if let Some(v) = map.get(&marp_key) {
            if v.as_bool() == Some(true) {
                return true;
            }
        }

        let theme_key = Value::String("theme".to_string());
        if let Some(v) = map.get(&theme_key) {
            if let Some(s) = v.as_str() {
                if matches!(s.to_lowercase().as_str(), "gaia" | "uncover" | "default" | "bespoke") {
                    return true;
                }
            }
        }

        let class_key = Value::String("_class".to_string());
        if map.contains_key(&class_key) {
            return true;
        }
    }

    false
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 || args[1] == "-h" || args[1] == "--help" {
        show_help();
        return;
    }

    if args[1] == "init" {
        handle_init(&args[2..]);
        return;
    }

    if args[1] == "config" {
        let (resolved_cfg, paths) = config::BotoxConfig::load(None, None);
        println!("Configuration status:");
        if !paths.is_empty() {
            println!("Loaded configuration sources (in priority order):");
            for p in &paths {
                println!("  - {}", p.display());
            }
        } else {
            println!("No custom configuration files found. Using internal defaults.");
        }
        println!("\nActive resolved configuration:");
        if let Ok(yaml) = serde_yaml::to_string(&resolved_cfg) {
            println!("{yaml}");
        }
        return;
    }

    let mut input_file: Option<PathBuf> = None;
    let mut output_file: Option<PathBuf> = None;
    let mut custom_config: Option<PathBuf> = None;
    let mut cli_toc: Option<bool> = None;
    let mut cli_author: Option<String> = None;
    let mut cli_font: Option<String> = None;
    let mut cli_bibliography: Option<bool> = None;
    let mut explicit_pdf = false;
    let mut explicit_slides = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "pdf" | "doc" => {
                explicit_pdf = true;
            }
            "slides" | "slide" => {
                explicit_slides = true;
            }
            "--pdf" => {
                explicit_pdf = true;
            }
            "--slides" => {
                explicit_slides = true;
            }
            "--toc" => {
                cli_toc = Some(true);
            }
            "--no-toc" => {
                cli_toc = Some(false);
            }
            "-b" | "--bib" | "--bibliography" => {
                cli_bibliography = Some(true);
            }
            "--no-bib" | "--no-bibliography" => {
                cli_bibliography = Some(false);
            }
            "-o" | "--output" => {
                i += 1;
                if i < args.len() {
                    output_file = Some(PathBuf::from(&args[i]));
                }
            }
            "--config" => {
                i += 1;
                if i < args.len() {
                    custom_config = Some(PathBuf::from(&args[i]));
                }
            }
            "--author" => {
                i += 1;
                if i < args.len() {
                    cli_author = Some(args[i].clone());
                }
            }
            "--font" => {
                i += 1;
                if i < args.len() {
                    cli_font = Some(args[i].clone());
                }
            }
            "-N" | "--number-sections" => {
                // Handled in document wrapper via flag or frontmatter
            }
            other => {
                if !other.starts_with('-') && input_file.is_none() {
                    input_file = Some(PathBuf::from(other));
                }
            }
        }
        i += 1;
    }

    let input_path = match input_file {
        Some(p) => p,
        None => {
            eprintln!("Error: No input file specified.");
            std::process::exit(1);
        }
    };

    if !input_path.is_file() {
        eprintln!("Error: Input file '{}' not found.", input_path.display());
        std::process::exit(1);
    }

    let output_path = output_file.unwrap_or_else(|| input_path.with_extension("pdf"));
    let (config_data, _) = config::BotoxConfig::load(custom_config.as_deref(), input_path.parent());
    let doc_config = config_data.document.unwrap_or_default();
    let slides_config = config_data.slides.unwrap_or_default();

    let raw_content = match std::fs::read_to_string(&input_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading '{}': {e}", input_path.display());
            std::process::exit(1);
        }
    };

    let (fm, body_md) = extract_frontmatter(&raw_content);
    let is_slides = detect_is_slides(&fm, explicit_slides, explicit_pdf);

    let lang = fm.get("lang")
        .and_then(|v| v.as_str())
        .or_else(|| doc_config.lang.as_deref())
        .unwrap_or("en");
    let biblio_title = fm.get("biblio-title")
        .or_else(|| fm.get("bibliography-title"))
        .or_else(|| fm.get("references-title"))
        .and_then(|v| v.as_str());

    let should_enable_bib = if let Some(cli) = cli_bibliography {
        cli
    } else if let Some(fm_bib) = fm.get("bibliography")
        .or_else(|| fm.get("links-as-references"))
        .or_else(|| fm.get("cite-links"))
    {
        match fm_bib {
            Value::Bool(b) => *b,
            Value::String(s) => !s.is_empty() && s != "false" && s != "no" && s != "off",
            _ => false,
        }
    } else {
        doc_config.bibliography.unwrap_or(true)
    };

    let typst_markup = if is_slides {
        let body_typst = markdown::markdown_to_typst(body_md, true, should_enable_bib, lang, biblio_title);
        slides::wrap_slides(&body_typst, &fm, &slides_config)
    } else {
        let body_typst = markdown::markdown_to_typst(body_md, false, should_enable_bib, lang, biblio_title);
        document::wrap_document(
            &body_typst,
            &fm,
            &doc_config,
            cli_toc,
            cli_author.as_deref(),
            cli_font.as_deref(),
        )
    };

    let start = std::time::Instant::now();
    let resource_dir = input_path.parent();
    match compiler::compile_typst_to_pdf(&typst_markup, &output_path, resource_dir) {
        Ok(()) => {
            let duration = start.elapsed();
            let mode_str = if is_slides { "Presentation slides" } else { "LaTeX PDF document" };
            println!("Compiled {} -> '{}' in {:.2?}", mode_str, output_path.display(), duration);
        }
        Err(e) => {
            eprintln!("Compilation failed: {e}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_generate_init_document_template() {
        let mut cfg = config::BotoxConfig::defaults();
        if let Some(ref mut doc) = cfg.document {
            doc.author = Some("Ada Lovelace".to_string());
            doc.affiliation = Some("Computing Institute".to_string());
        }
        let tpl = generate_init_template(false, &cfg, None, Path::new("report.md"));
        assert!(tpl.contains("name: \"Ada Lovelace\""));
        assert!(tpl.contains("affiliation: \"Computing Institute\""));
        assert!(tpl.contains("papersize: a4"));
        assert!(tpl.contains("number-sections: true"));
        assert!(tpl.contains("table-of-contents: true"));
        assert!(tpl.contains("toc-depth: 3"));
        assert!(tpl.contains("bibliography: true"));
        assert!(tpl.contains("botox report.md -o output.pdf"));
    }

    #[test]
    fn test_generate_init_slides_template() {
        let mut cfg = config::BotoxConfig::defaults();
        if let Some(ref mut s) = cfg.slides {
            s.theme = Some("academic".to_string());
        }
        let tpl = generate_init_template(true, &cfg, None, Path::new("talk.md"));
        assert!(tpl.contains("marp: true"));
        assert!(tpl.contains("theme: academic"));
        assert!(tpl.contains("paginate: true"));

        // CLI theme override
        let tpl2 = generate_init_template(true, &cfg, Some("nord"), Path::new("talk.md"));
        assert!(tpl2.contains("theme: nord"));
    }
}
