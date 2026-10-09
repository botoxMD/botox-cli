mod config;
mod markdown;
mod document;
mod slides;
mod compiler;
pub mod diagrams;

use std::path::PathBuf;
use serde_yaml::Value;

fn show_help() {
    println!(r#"Usage: botox <input.md> [options]
       botox <pdf|slides> <input.md> [options]
       botox init <filename> [options]
       botox config
       botox setup [options]

Pure Rust single native binary: Transforms Markdown into LaTeX-quality PDF documents or presentation slides.
Mode is detected automatically from frontmatter structure, or specified explicitly.

Commands:
  init <filename>         Initialize a new Markdown document or presentation slide deck
  config                  Display currently active configuration settings and loaded sources
  setup                   Interactive configuration wizard to configure user defaults

Common Options:
  -o, --output <file>     Output file path (default: .html for slides, .pdf for documents, or '-' for stdout)
  -w, --watch             Watch input file and directory for changes and recompile automatically
  --config <file>         Custom configuration YAML path

Documentation Options:
  --pdf                   Force PDF documentation mode
  --theme <theme>         Document theme: academic (default), modern, elegant, technical, compact, minimal
  --toc                   Generate automatic Table of Contents
  --no-toc                Disable Table of Contents
  --author <name>         Author name (overrides frontmatter & config)
  --font <name>           Main font (default: New Computer Modern)
  -N, --number-sections   Number section headings
  -b, --bibliography      Transform web links into an IEEE-standard Bibliography
  --no-bibliography       Disable automatic Bibliography generation
  --exclude-ref <pattern> Exclude matching URLs/domains from references (e.g. "github.com,x.com")
  --include-ref <pattern> Only include matching URLs/domains in references

Slide Options:
  --slides                Force presentation slide deck mode (default output: .html)
  --theme <theme>         Slide theme: default, academic, dark, nord
"#);
}

fn handle_setup(args: &[String]) {
    if args.iter().any(|a| a == "-h" || a == "--help") {
        println!(r#"Usage: botox setup [options]

Interactive configuration wizard to set global or local typesetting defaults.

Options:
  --global                Save to global configuration (~/.config/botox/config.yaml) [default]
  --local                 Save to current directory (./botox.yaml)
  --author <name>         Pre-set default author
  --affiliation <org>     Pre-set default affiliation
  --theme <theme>         Pre-set default document theme (academic, modern, elegant, technical, compact, minimal)
  --slides-theme <theme>  Pre-set default slide theme (default, academic, dark, nord)
  --papersize <size>      Pre-set default paper size (a4, us-letter)
  --toc                   Enable Table of Contents by default
  --no-toc                Disable Table of Contents by default
  --bibliography          Enable automatic Bibliography by default
  --no-bibliography       Disable automatic Bibliography by default
  --non-interactive, -y   Save configuration without interactive prompts
"#);
        return;
    }

    let mut force_global = false;
    let mut force_local = false;
    let mut non_interactive = false;

    let mut cli_author: Option<String> = None;
    let mut cli_affiliation: Option<String> = None;
    let mut cli_theme: Option<String> = None;
    let mut cli_slides_theme: Option<String> = None;
    let mut cli_papersize: Option<String> = None;
    let mut cli_toc: Option<bool> = None;
    let mut cli_bib: Option<bool> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--global" => force_global = true,
            "--local" => force_local = true,
            "--non-interactive" | "-y" => non_interactive = true,
            "--author" => {
                i += 1;
                if i < args.len() { cli_author = Some(args[i].clone()); }
            }
            "--affiliation" => {
                i += 1;
                if i < args.len() { cli_affiliation = Some(args[i].clone()); }
            }
            "--theme" => {
                i += 1;
                if i < args.len() { cli_theme = Some(args[i].clone()); }
            }
            "--slides-theme" => {
                i += 1;
                if i < args.len() { cli_slides_theme = Some(args[i].clone()); }
            }
            "--papersize" => {
                i += 1;
                if i < args.len() { cli_papersize = Some(args[i].clone()); }
            }
            "--toc" => cli_toc = Some(true),
            "--no-toc" => cli_toc = Some(false),
            "--bibliography" => cli_bib = Some(true),
            "--no-bibliography" => cli_bib = Some(false),
            _ => {}
        }
        i += 1;
    }

    // Load existing config to propose current values as defaults
    let (existing, _) = config::BotoxConfig::load(None, None);
    let existing_doc = existing.document.unwrap_or_default();
    let existing_slides = existing.slides.unwrap_or_default();

    let default_author = cli_author
        .or(existing_doc.author)
        .or_else(|| std::env::var("USER").or_else(|_| std::env::var("USERNAME")).ok())
        .unwrap_or_else(|| "Minus".to_string());

    let default_affiliation = cli_affiliation
        .or(existing_doc.affiliation)
        .unwrap_or_default();

    let default_doc_theme = cli_theme
        .or(existing_doc.theme)
        .unwrap_or_else(|| "academic".to_string());

    let default_slides_theme = cli_slides_theme
        .or(existing_slides.theme)
        .unwrap_or_else(|| "default".to_string());

    let default_papersize = cli_papersize
        .or(existing_doc.papersize)
        .unwrap_or_else(|| "a4".to_string());

    let default_toc = cli_toc
        .or(existing_doc.toc)
        .unwrap_or(false);

    let default_bib = cli_bib
        .or(existing_doc.bibliography)
        .unwrap_or(true);

    let (chosen_author, chosen_affiliation, chosen_doc_theme, chosen_slides_theme, chosen_papersize, chosen_toc, chosen_bib, is_global) = if non_interactive {
        (
            default_author,
            default_affiliation,
            default_doc_theme,
            default_slides_theme,
            default_papersize,
            default_toc,
            default_bib,
            !force_local,
        )
    } else {
        println!("\n╔══════════════════════════════════════════════════════════════╗");
        println!("║                Botox Setup & Defaults Wizard                 ║");
        println!("╚══════════════════════════════════════════════════════════════╝");
        println!("Configure your default author, themes, and typesetting styles.");
        println!("Press [Enter] to keep the current value shown in brackets.\n");

        let author = prompt_line("1. Default Author name", &default_author);
        let affiliation = prompt_line("2. Default Affiliation / Institution", &default_affiliation);
        let doc_theme = prompt_choice(
            "3. Default Document Theme (academic, modern, elegant, technical, compact, minimal)",
            &["academic", "modern", "elegant", "technical", "compact", "minimal"],
            &default_doc_theme,
        );
        let slides_theme = prompt_choice(
            "4. Default Slide Theme (default, academic, dark, nord)",
            &["default", "academic", "dark", "nord"],
            &default_slides_theme,
        );
        let papersize = prompt_choice(
            "5. Paper Size (a4, us-letter)",
            &["a4", "us-letter"],
            &default_papersize,
        );
        let toc = prompt_bool("6. Enable Table of Contents by default in new documents?", default_toc);
        let bib = prompt_bool("7. Enable automatic IEEE Bibliography from web links?", default_bib);

        let target_global = if force_global {
            true
        } else if force_local {
            false
        } else {
            let choice = prompt_line(
                "8. Save configuration scope:\n   [1] Global (~/.config/botox/config.yaml - all projects)\n   [2] Local  (./botox.yaml - current folder only)\n   Choose 1 or 2",
                "1",
            );
            choice.trim() != "2"
        };

        (author, affiliation, doc_theme, slides_theme, papersize, toc, bib, target_global)
    };

    let target_path = if is_global {
        config::global_config_path()
    } else {
        PathBuf::from("botox.yaml")
    };

    if let Some(parent) = target_path.parent()
        && !parent.as_os_str().is_empty() {
            let _ = std::fs::create_dir_all(parent);
        }

    let yaml_content = config::generate_config_yaml(
        &chosen_author,
        if chosen_affiliation.trim().is_empty() { None } else { Some(&chosen_affiliation) },
        &chosen_doc_theme,
        &chosen_slides_theme,
        &chosen_papersize,
        chosen_toc,
        chosen_bib,
    );

    match std::fs::write(&target_path, yaml_content) {
        Ok(()) => {
            println!("\n✓ Successfully saved Botox defaults to '{}'", target_path.display());
            println!("Summary of active defaults:");
            println!("  - Author:            {}", chosen_author);
            if !chosen_affiliation.trim().is_empty() {
                println!("  - Affiliation:       {}", chosen_affiliation);
            }
            println!("  - Document Theme:    {}", chosen_doc_theme);
            println!("  - Slide Theme:       {}", chosen_slides_theme);
            println!("  - Paper Size:        {}", chosen_papersize);
            println!("  - Table of Contents: {}", if chosen_toc { "Enabled" } else { "Disabled" });
            println!("  - Bibliography:      {}", if chosen_bib { "Enabled" } else { "Disabled" });
            println!("\nYou can re-run 'botox setup' anytime or edit the YAML file directly.\n");
        }
        Err(e) => {
            eprintln!("Error saving configuration to '{}': {e}", target_path.display());
            std::process::exit(1);
        }
    }
}

fn prompt_line(prompt: &str, default: &str) -> String {
    use std::io::{self, Write};
    if default.is_empty() {
        print!("{prompt}: ");
    } else {
        print!("{prompt} [{default}]: ");
    }
    let _ = io::stdout().flush();
    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_ok() {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            default.to_string()
        } else {
            trimmed.to_string()
        }
    } else {
        default.to_string()
    }
}

fn prompt_bool(prompt: &str, default: bool) -> bool {
    let def_str = if default { "Y/n" } else { "y/N" };
    let prompt_full = format!("{prompt} [{def_str}]");
    let res = prompt_line(&prompt_full, if default { "y" } else { "n" });
    match res.trim().to_lowercase().as_str() {
        "y" | "yes" | "true" | "1" => true,
        "n" | "no" | "false" | "0" => false,
        _ => default,
    }
}

fn prompt_choice(prompt: &str, choices: &[&str], default: &str) -> String {
    let input = prompt_line(prompt, default);
    let trimmed = input.trim().to_lowercase();
    for c in choices {
        if trimmed == *c {
            return c.to_string();
        }
    }
    default.to_string()
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

    if let Some(parent) = target_path.parent()
        && !parent.as_os_str().is_empty() {
            let _ = std::fs::create_dir_all(parent);
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

Hyperlinks can be formatted into a bibliography using `\ref`:
Check out the [Botox Documentation](https://github.com) for more examples.

\ref References
"#,
            display_name = target_path.display()
        )
    }
}

fn extract_frontmatter(content: &str) -> (Value, &str) {
    let trimmed = content.trim_start();
    if trimmed.starts_with("---") {
        let parts: Vec<&str> = trimmed.splitn(3, "---").collect();
        if parts.len() >= 3 {
            if let Ok(val) = serde_yaml::from_str::<Value>(parts[1]) {
                return (val, parts[2].trim_start());
            }
            // Fallback: sanitize unquoted \today which can cause YAML parser errors
            let sanitized = parts[1].replace(r"\today", r#""\today""#);
            if let Ok(val) = serde_yaml::from_str::<Value>(&sanitized) {
                return (val, parts[2].trim_start());
            }
            return (Value::Null, parts[2].trim_start());
        }
    }
    (Value::Null, content)
}

fn detect_is_slides(
    fm: &Value,
    raw_content: &str,
    input_path: &std::path::Path,
    explicit_slides: bool,
    explicit_pdf: bool,
) -> bool {
    if explicit_slides {
        return true;
    }
    if explicit_pdf {
        return false;
    }

    // 1. Check parsed YAML frontmatter mapping
    if let Some(map) = fm.as_mapping() {
        for (k, v) in map {
            if let Some(key_str) = k.as_str() {
                let key_lower = key_str.to_lowercase();
                if (key_lower == "marp" || key_lower == "slides" || key_lower == "slide" || key_lower == "presentation")
                    && (v.as_bool() == Some(true) || v.as_str().map(|s| s.eq_ignore_ascii_case("true")).unwrap_or(false)) {
                        return true;
                    }
                if (key_lower == "type" || key_lower == "format" || key_lower == "document-type")
                    && let Some(s) = v.as_str() {
                        let s_lower = s.to_lowercase();
                        if s_lower == "slides" || s_lower == "slide" || s_lower == "presentation" || s_lower == "deck" {
                            return true;
                        }
                        if s_lower == "document" || s_lower == "article" || s_lower == "paper" || s_lower == "report" {
                            return false;
                        }
                    }
                if key_lower == "_class" {
                    return true;
                }
            }
        }
    }

    // 2. Direct text scan fallback across frontmatter in case YAML parsing failed
    let trimmed = raw_content.trim_start();
    if trimmed.starts_with("---")
        && let Some(end_idx) = trimmed[3..].find("---") {
            let header = &trimmed[3..3 + end_idx];
            for line in header.lines() {
                let l = line.trim();
                let lower = l.to_lowercase();
                if lower.starts_with("marp:") && lower.contains("true") {
                    return true;
                }
                if lower.starts_with("slides:") && lower.contains("true") {
                    return true;
                }
                if lower.starts_with("presentation:") && lower.contains("true") {
                    return true;
                }
                if lower.starts_with("type:") && (lower.contains("slide") || lower.contains("deck") || lower.contains("presentation")) {
                    return true;
                }
            }
        }

    // 3. Filename convention fallback (e.g. slides.md, deck.md, presentation.md)
    let file_name = input_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();
    if file_name.contains("slide") || file_name.contains("deck") || file_name.contains("presentation") {
        return true;
    }

    false
}

fn get_timestamp() -> String {
    chrono::Local::now().format("%H:%M:%S").to_string()
}

fn is_relevant_watch_file(path: &std::path::Path, output_path: &std::path::Path) -> bool {
    if path == output_path {
        return false;
    }
    let file_name = match path.file_name().and_then(|n| n.to_str()) {
        Some(n) => n,
        None => return false,
    };
    if file_name.starts_with('.') || file_name.ends_with('~') {
        return false;
    }
    let ext = match path.extension().and_then(|e| e.to_str()) {
        Some(e) => e.to_lowercase(),
        None => return false,
    };
    matches!(
        ext.as_str(),
        "md" | "markdown" | "yaml" | "yml" | "typ" | "bib" | "png" | "jpg" | "jpeg" | "svg" | "webp" | "gif"
    )
}

fn take_watch_snapshot(
    input_path: &std::path::Path,
    output_path: &std::path::Path,
    doc_dir: Option<&std::path::Path>,
    custom_config: Option<&std::path::Path>,
) -> Vec<(PathBuf, std::time::SystemTime, u64)> {
    let mut list = Vec::new();

    if let Ok(meta) = std::fs::metadata(input_path) {
        let mtime = meta.modified().unwrap_or(std::time::UNIX_EPOCH);
        list.push((input_path.to_path_buf(), mtime, meta.len()));
    }

    if let Some(cfg) = custom_config
        && let Ok(meta) = std::fs::metadata(cfg) {
            let mtime = meta.modified().unwrap_or(std::time::UNIX_EPOCH);
            list.push((cfg.to_path_buf(), mtime, meta.len()));
        }

    let dir = doc_dir.or_else(|| input_path.parent());
    if let Some(d) = dir
        && let Ok(read_dir) = std::fs::read_dir(d) {
            for entry in read_dir.flatten() {
                let path = entry.path();
                if path != input_path && is_relevant_watch_file(&path, output_path)
                    && let Ok(meta) = entry.metadata()
                        && meta.is_file() {
                            let mtime = meta.modified().unwrap_or(std::time::UNIX_EPOCH);
                            list.push((path, mtime, meta.len()));
                        }
            }
        }

    list.sort_by(|a, b| a.0.cmp(&b.0));
    list
}

#[allow(clippy::too_many_arguments)]
fn compile_once(
    input_path: &std::path::Path,
    output_path: &std::path::Path,
    doc_dir: Option<&std::path::Path>,
    config_data: &config::BotoxConfig,
    explicit_pdf: bool,
    explicit_slides: bool,
    cli_toc: Option<bool>,
    cli_author: Option<&str>,
    cli_font: Option<&str>,
    cli_bibliography: Option<bool>,
    cli_theme: Option<&str>,
    cli_exclude_ref: &[String],
    cli_include_ref: &[String],
    is_stdin: bool,
) -> Result<(bool, std::time::Duration), String> {
    let raw_content = if is_stdin {
        use std::io::Read;
        let mut buffer = String::new();
        std::io::stdin()
            .read_to_string(&mut buffer)
            .map_err(|e| format!("Error reading from stdin: {e}"))?;
        buffer
    } else {
        std::fs::read_to_string(input_path)
            .map_err(|e| format!("Error reading '{}': {e}", input_path.display()))?
    };

    let (fm, body_md) = extract_frontmatter(&raw_content);
    let is_slides = detect_is_slides(&fm, &raw_content, input_path, explicit_slides, explicit_pdf);

    let doc_config = config_data.document.clone().unwrap_or_default();
    let slides_config = config_data.slides.clone().unwrap_or_default();

    let lang = fm.get("lang")
        .and_then(|v| v.as_str())
        .or(doc_config.lang.as_deref())
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
        doc_config.bibliography.unwrap_or(false)
    };

    let toc_title = fm.get("toc-title")
        .or_else(|| fm.get("toc_title"))
        .and_then(|v| v.as_str())
        .or(doc_config.toc_title.as_deref());

    let toc_depth = fm.get("toc-depth")
        .or_else(|| fm.get("toc_depth"))
        .and_then(|v| v.as_u64())
        .map(|n| n as usize)
        .or(doc_config.toc_depth);

    let extract_string_list = |val: &Value| -> Vec<String> {
        if let Some(arr) = val.as_sequence() {
            arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()
        } else if let Some(s) = val.as_str() {
            vec![s.to_string()]
        } else {
            Vec::new()
        }
    };

    let mut ref_exclude = Vec::new();
    ref_exclude.extend_from_slice(cli_exclude_ref);
    if let Some(ref cfg_ref) = config_data.references
        && let Some(ref exc) = cfg_ref.exclude {
            ref_exclude.extend(exc.clone());
        }
    if let Some(ref cfg_ref) = doc_config.references
        && let Some(ref exc) = cfg_ref.exclude {
            ref_exclude.extend(exc.clone());
        }
    if let Some(ref exc) = doc_config.exclude_references {
        ref_exclude.extend(exc.clone());
    }
    if let Some(v) = fm.get("references").and_then(|r| r.get("exclude")) {
        ref_exclude.extend(extract_string_list(v));
    }
    if let Some(v) = fm.get("exclude-references")
        .or_else(|| fm.get("exclude_references"))
        .or_else(|| fm.get("exclude-links"))
        .or_else(|| fm.get("exclude_links"))
    {
        ref_exclude.extend(extract_string_list(v));
    }

    let mut ref_include = Vec::new();
    ref_include.extend_from_slice(cli_include_ref);
    if let Some(ref cfg_ref) = config_data.references
        && let Some(ref inc) = cfg_ref.include {
            ref_include.extend(inc.clone());
        }
    if let Some(ref cfg_ref) = doc_config.references
        && let Some(ref inc) = cfg_ref.include {
            ref_include.extend(inc.clone());
        }
    if let Some(ref inc) = doc_config.include_references {
        ref_include.extend(inc.clone());
    }
    if let Some(v) = fm.get("references").and_then(|r| r.get("include")) {
        ref_include.extend(extract_string_list(v));
    }
    if let Some(v) = fm.get("include-references")
        .or_else(|| fm.get("include_references"))
        .or_else(|| fm.get("include-links"))
        .or_else(|| fm.get("include_links"))
    {
        ref_include.extend(extract_string_list(v));
    }

    let exclude_opt = if ref_exclude.is_empty() { None } else { Some(ref_exclude) };
    let include_opt = if ref_include.is_empty() { None } else { Some(ref_include) };

    let kroki_url = fm.get("krocki-url")
        .or_else(|| fm.get("kroki-url"))
        .or_else(|| fm.get("kroki_url"))
        .or_else(|| fm.get("krocki_url"))
        .and_then(|v| v.as_str())
        .or(doc_config.kroki_url.as_deref());

    let md_opts = markdown::MarkdownOptions {
        is_slides,
        bibliography: should_enable_bib,
        lang,
        biblio_title,
        toc_title,
        toc_depth,
        ref_exclude: exclude_opt.as_deref(),
        ref_include: include_opt.as_deref(),
        kroki_url,
    };
    let body_typst = markdown::markdown_to_typst_with_options(body_md, &md_opts);

    let typst_markup = if is_slides {
        slides::wrap_slides(&body_typst, &fm, &slides_config, cli_author, cli_theme)
    } else {
        document::wrap_document(
            &body_typst,
            &fm,
            &doc_config,
            cli_toc,
            cli_author,
            cli_font,
            cli_theme,
        )
    };

    let start = std::time::Instant::now();
    let resource_dir = doc_dir;
    compiler::compile_typst(&typst_markup, output_path, resource_dir)?;
    let duration = start.elapsed();
    Ok((is_slides, duration))
}

fn format_mode_str(is_slides: bool, output_path: &std::path::Path) -> &'static str {
    let ext = output_path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    if is_slides {
        if ext == "html" || ext == "htm" {
            "HTML presentation slides"
        } else if ext == "svg" {
            "SVG presentation slides"
        } else {
            "Presentation slides"
        }
    } else {
        if ext == "html" || ext == "htm" {
            "HTML document"
        } else if ext == "svg" {
            "SVG document"
        } else {
            "LaTeX PDF document"
        }
    }
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

    if args[1] == "setup" {
        handle_setup(&args[2..]);
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
    let mut cli_theme: Option<String> = None;
    let mut cli_bibliography: Option<bool> = None;
    let mut cli_resource_dir: Option<PathBuf> = None;
    let mut cli_exclude_ref: Vec<String> = Vec::new();
    let mut cli_include_ref: Vec<String> = Vec::new();
    let mut explicit_pdf = false;
    let mut explicit_slides = false;
    let mut watch_mode = false;

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
            "-w" | "--watch" => {
                watch_mode = true;
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
            "--exclude-ref" | "--exclude-references" | "--exclude-links" => {
                i += 1;
                if i < args.len() {
                    for part in args[i].split(',') {
                        let trimmed = part.trim();
                        if !trimmed.is_empty() {
                            cli_exclude_ref.push(trimmed.to_string());
                        }
                    }
                }
            }
            "--include-ref" | "--include-references" | "--include-links" => {
                i += 1;
                if i < args.len() {
                    for part in args[i].split(',') {
                        let trimmed = part.trim();
                        if !trimmed.is_empty() {
                            cli_include_ref.push(trimmed.to_string());
                        }
                    }
                }
            }
            "--theme" => {
                i += 1;
                if i < args.len() {
                    cli_theme = Some(args[i].clone());
                }
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
            "--resource-dir" => {
                i += 1;
                if i < args.len() {
                    cli_resource_dir = Some(PathBuf::from(&args[i]));
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
                if other == "-" && input_file.is_none() {
                    input_file = Some(PathBuf::from("-"));
                } else if !other.starts_with('-') && input_file.is_none() {
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

    let is_stdin = input_path.as_os_str() == "-";
    if !is_stdin && !input_path.is_file() {
        eprintln!("Error: Input file '{}' not found.", input_path.display());
        std::process::exit(1);
    }

    if is_stdin && watch_mode {
        eprintln!("Error: Cannot use watch mode when reading from stdin.");
        std::process::exit(1);
    }

    let is_preliminary_slides = if explicit_slides {
        true
    } else if explicit_pdf {
        false
    } else if !is_stdin && input_path.is_file() {
        if let Ok(prelim_content) = std::fs::read_to_string(&input_path) {
            let (fm, _) = extract_frontmatter(&prelim_content);
            detect_is_slides(&fm, &prelim_content, &input_path, false, false)
        } else {
            let name = input_path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_lowercase();
            name.contains("slide") || name.contains("deck") || name.contains("presentation")
        }
    } else {
        false
    };

    let default_ext = if is_preliminary_slides { "html" } else { "pdf" };
    let default_output = if is_stdin {
        PathBuf::from(format!("output.{default_ext}"))
    } else {
        input_path.with_extension(default_ext)
    };
    let output_path = output_file.unwrap_or(default_output);
    let is_stdout = output_path.as_os_str() == "-";

    let doc_dir = cli_resource_dir.or_else(|| {
        if is_stdin {
            std::env::current_dir().ok()
        } else {
            input_path.parent().map(|p| p.to_path_buf())
        }
    });

    let (config_data, _) = config::BotoxConfig::load(custom_config.as_deref(), doc_dir.as_deref());

    if !watch_mode {
        match compile_once(
            &input_path,
            &output_path,
            doc_dir.as_deref(),
            &config_data,
            explicit_pdf,
            explicit_slides,
            cli_toc,
            cli_author.as_deref(),
            cli_font.as_deref(),
            cli_bibliography,
            cli_theme.as_deref(),
            &cli_exclude_ref,
            &cli_include_ref,
            is_stdin,
        ) {
            Ok((is_slides, duration)) => {
                if !is_stdout {
                    let mode_str = format_mode_str(is_slides, &output_path);
                    println!("Compiled {} -> '{}' in {:.2?}", mode_str, output_path.display(), duration);
                }
            }
            Err(e) => {
                eprintln!("Compilation failed: {e}");
                std::process::exit(1);
            }
        }
        return;
    }

    // Watch mode: compile once, then poll for changes
    match compile_once(
        &input_path,
        &output_path,
        doc_dir.as_deref(),
        &config_data,
        explicit_pdf,
        explicit_slides,
        cli_toc,
        cli_author.as_deref(),
        cli_font.as_deref(),
        cli_bibliography,
        cli_theme.as_deref(),
        &cli_exclude_ref,
        &cli_include_ref,
        false,
    ) {
        Ok((is_slides, duration)) => {
            let mode_str = format_mode_str(is_slides, &output_path);
            if is_stdout {
                eprintln!("[{}] Compiled {} -> stdout in {:.2?}", get_timestamp(), mode_str, duration);
            } else {
                println!("Compiled {} -> '{}' in {:.2?}", mode_str, output_path.display(), duration);
            }
        }
        Err(e) => {
            eprintln!("[{}] Initial compilation failed: {e}", get_timestamp());
        }
    }

    let start_time = get_timestamp();
    if is_stdout {
        eprintln!("[{start_time}] Watching '{}' for changes (Ctrl+C to stop)...", input_path.display());
    } else {
        println!("[{start_time}] Watching '{}' for changes (Ctrl+C to stop)...", input_path.display());
    }

    let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let r = running.clone();
    let _ = ctrlc::set_handler(move || {
        r.store(false, std::sync::atomic::Ordering::SeqCst);
    });

    let mut last_snapshot = take_watch_snapshot(&input_path, &output_path, doc_dir.as_deref(), custom_config.as_deref());

    while running.load(std::sync::atomic::Ordering::SeqCst) {
        for _ in 0..5 {
            if !running.load(std::sync::atomic::Ordering::SeqCst) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        if !running.load(std::sync::atomic::Ordering::SeqCst) {
            break;
        }

        let current_snapshot = take_watch_snapshot(&input_path, &output_path, doc_dir.as_deref(), custom_config.as_deref());
        if current_snapshot != last_snapshot {
            std::thread::sleep(std::time::Duration::from_millis(50));
            let current_snapshot = take_watch_snapshot(&input_path, &output_path, doc_dir.as_deref(), custom_config.as_deref());
            last_snapshot = current_snapshot;

            let (updated_cfg, _) = config::BotoxConfig::load(custom_config.as_deref(), doc_dir.as_deref());

            match compile_once(
                &input_path,
                &output_path,
                doc_dir.as_deref(),
                &updated_cfg,
                explicit_pdf,
                explicit_slides,
                cli_toc,
                cli_author.as_deref(),
                cli_font.as_deref(),
                cli_bibliography,
                cli_theme.as_deref(),
                &cli_exclude_ref,
                &cli_include_ref,
                false,
            ) {
                Ok((is_slides, duration)) => {
                    let mode_str = format_mode_str(is_slides, &output_path);
                    let now = get_timestamp();
                    let target = if is_stdout { "stdout".to_string() } else { format!("'{}'", output_path.display()) };
                    let msg = format!("[{now}] Recompiled {mode_str} -> {target} in {:.2?}", duration);
                    if is_stdout {
                        eprintln!("{msg}");
                    } else {
                        println!("{msg}");
                    }
                }
                Err(e) => {
                    let now = get_timestamp();
                    eprintln!("[{now}] Compilation failed: {e}");
                }
            }
        }
    }

    if is_stdout {
        eprintln!("\nWatch mode stopped.");
    } else {
        println!("\nWatch mode stopped.");
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
        assert!(tpl.contains("table-of-contents: false"));
        assert!(tpl.contains("toc-depth: 3"));
        assert!(tpl.contains("bibliography: false"));
        assert!(tpl.contains(r"\ref References"));
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

    #[test]
    fn test_handle_setup_non_interactive_local() {
        let temp_dir = std::env::temp_dir().join("botox_setup_test");
        let _ = std::fs::create_dir_all(&temp_dir);
        let orig_dir = std::env::current_dir().unwrap();
        let _ = std::env::set_current_dir(&temp_dir);

        let args = vec![
            "--non-interactive".to_string(),
            "--local".to_string(),
            "--author".to_string(),
            "Grace Hopper".to_string(),
            "--theme".to_string(),
            "modern".to_string(),
            "--slides-theme".to_string(),
            "nord".to_string(),
            "--papersize".to_string(),
            "us-letter".to_string(),
            "--toc".to_string(),
            "--bibliography".to_string(),
        ];
        handle_setup(&args);

        let local_cfg = temp_dir.join("botox.yaml");
        assert!(local_cfg.is_file(), "Local config should be generated");
        let content = std::fs::read_to_string(&local_cfg).unwrap();
        assert!(content.contains("author: \"Grace Hopper\""));
        assert!(content.contains("theme: \"modern\""));
        assert!(content.contains("theme: \"nord\""));
        assert!(content.contains("papersize: \"us-letter\""));
        assert!(content.contains("toc: true"));
        assert!(content.contains("bibliography: true"));

        let _ = std::fs::remove_file(local_cfg);
        let _ = std::env::set_current_dir(orig_dir);
        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_default_output_extension_slides_vs_document() {
        let slides_md = "---\nmarp: true\n---\n# Slide 1";
        let (fm, _) = extract_frontmatter(slides_md);
        assert!(detect_is_slides(&fm, slides_md, Path::new("test.md"), false, false));

        let doc_md = "---\ntitle: Doc\n---\n# Intro";
        let (fm_doc, _) = extract_frontmatter(doc_md);
        assert!(!detect_is_slides(&fm_doc, doc_md, Path::new("test.md"), false, false));

        // When explicit_pdf is set, even marp: true is treated as pdf
        assert!(!detect_is_slides(&fm, slides_md, Path::new("test.md"), false, true));
    }
}

