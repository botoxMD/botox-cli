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
       botox config

Pure Rust single native binary: Transforms Markdown into LaTeX-quality PDF documents or presentation slides.
Mode is detected automatically from frontmatter structure, or specified explicitly.

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

Slide Options:
  --slides                Force presentation slide deck mode
"#);
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

    if args[1] == "config" {
        let (_, path) = config::BotoxConfig::load(None);
        println!("Configuration status:");
        if let Some(p) = path {
            println!("Active config: {}", p.display());
            if let Ok(c) = std::fs::read_to_string(&p) {
                println!("\n{c}");
            }
        } else {
            println!("No configuration file found. Using internal defaults.");
        }
        return;
    }

    let mut input_file: Option<PathBuf> = None;
    let mut output_file: Option<PathBuf> = None;
    let mut custom_config: Option<PathBuf> = None;
    let mut cli_toc: Option<bool> = None;
    let mut cli_author: Option<String> = None;
    let mut cli_font: Option<String> = None;
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
    let (config_data, _) = config::BotoxConfig::load(custom_config.as_deref());
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

    let typst_markup = if is_slides {
        let body_typst = markdown::markdown_to_typst(body_md, true);
        slides::wrap_slides(&body_typst, &fm, &slides_config)
    } else {
        let body_typst = markdown::markdown_to_typst(body_md, false);
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
    match compiler::compile_typst_to_pdf(&typst_markup, &output_path) {
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
