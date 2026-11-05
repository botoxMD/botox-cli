mod config;
mod markdown;
mod document;
mod slides;
mod compiler;
pub mod diagrams;
mod skills;
pub mod daemon;
pub mod engine;
pub mod emitter;

use std::path::{Path, PathBuf};

pub use engine::{
    detect_is_slides, extract_frontmatter, extract_yaml_string_list, parse_frontmatter_strict,
    resolve_reference_filters, transpile_markdown_to_typst_markup, DocumentSource, PublishOptions,
    PublishResult, TargetFormat,
};

fn show_help() {
    println!(r#"Usage: botox <input.md|input.btx|input.btx.md> [options]
       botox <pdf|slides|markdown> <input> [options]
       botox init <filename> [options]
       botox config
       botox setup [options]
       botox skills [options]
       botox daemon [options]

Pure Rust single native binary: Transforms Markdown and Botox (.btx, .btx.md) into LaTeX-quality PDF documents, presentation slides, or clean standard Markdown.
Mode is detected automatically from frontmatter structure and file extensions, or specified explicitly.

Commands:
  init <filename>         Initialize a new Markdown/Botox document or presentation slide deck
  markdown, md            Export to clean, portable standard Markdown with assets & resolved citations
  config                  Display currently active configuration settings and loaded sources
  setup                   Interactive configuration wizard to configure user defaults
  skills, --skills        Generate agent skill definition (SKILL.md) for AI coding agents
  daemon, --daemon        Start persistent preview compiler daemon for instant editor live updates

Common Options:
  -o, --output <file>     Output file path (default: .html for slides, .pdf for documents, .md for markdown export, or '-' for stdout)
  -w, --watch             Watch input file and directory for changes and recompile automatically
  --config <file>         Custom configuration YAML path
  -V, --version           Print version information and exit
  -h, --help              Print this help message and exit

Markdown Export Options:
  --markdown, --md        Force normal Markdown export mode
  --assets-dir <dir>      Directory for exported diagram images (default: 'assets')
  --no-frontmatter        Omit YAML frontmatter in exported Markdown

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
  --theme <theme>         Slide theme: default, academic, gaia, uncover, nord, dark
"#);
}

fn handle_setup(args: &[String]) {
    handle_setup_in_dir(args, None);
}

fn handle_setup_in_dir(args: &[String], local_dir: Option<&Path>) {
    if args.iter().any(|a| a == "-h" || a == "--help") {
        println!(r#"Usage: botox setup [options]

Interactive configuration wizard to set global or local typesetting defaults.

Options:
  --global                Save to global configuration (~/.config/botox/config.yaml) [default]
  --local                 Save to current directory (./botox.yaml)
  --author <name>         Pre-set default author
  --affiliation <org>     Pre-set default affiliation
  --theme <theme>         Pre-set default document theme (academic, modern, elegant, technical, compact, minimal)
  --slides-theme <theme>  Pre-set default slide theme (default, academic, gaia, uncover, nord, dark)
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
            "4. Default Slide Theme (default, academic, gaia, uncover, nord, dark)",
            &["default", "academic", "gaia", "uncover", "nord", "dark"],
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
    } else if let Some(dir) = local_dir {
        dir.join("botox.yaml")
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
    let name_lower = file_name.to_lowercase();
    if name_lower.ends_with(".btx.md") || name_lower.ends_with(".bbl.md") {
        return true;
    }
    let ext = match path.extension().and_then(|e| e.to_str()) {
        Some(e) => e.to_lowercase(),
        None => return false,
    };
    matches!(
        ext.as_str(),
        "md" | "markdown" | "btx" | "bbl" | "yaml" | "yml" | "typ" | "bib" | "png" | "jpg" | "jpeg" | "svg" | "webp" | "gif"
    )
}

pub fn compute_default_output_path(input_path: &Path, target_ext: &str) -> PathBuf {
    let file_name = input_path.file_name().and_then(|n| n.to_str()).unwrap_or("document");
    let stem = if let Some(stripped) = file_name.strip_suffix(".btx.md") {
        stripped
    } else if let Some(stripped) = file_name.strip_suffix(".bbl.md") {
        stripped
    } else if let Some(stripped) = file_name.strip_suffix(".btx") {
        stripped
    } else if let Some(stripped) = file_name.strip_suffix(".md") {
        stripped
    } else if let Some(stripped) = file_name.strip_suffix(".markdown") {
        stripped
    } else {
        input_path.file_stem().and_then(|s| s.to_str()).unwrap_or(file_name)
    };

    let final_stem = if target_ext == "md" && (file_name.ends_with(".md") && !file_name.ends_with(".btx.md") && !file_name.ends_with(".bbl.md")) {
        format!("{stem}.export")
    } else {
        stem.to_string()
    };

    let parent = input_path.parent();
    match parent {
        Some(p) if !p.as_os_str().is_empty() => p.join(format!("{final_stem}.{target_ext}")),
        _ => PathBuf::from(format!("{final_stem}.{target_ext}")),
    }
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
    _config_data: &config::BotoxConfig,
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
    cli_source_file: Option<&str>,
) -> Result<(bool, std::time::Duration), String> {
    let source = if is_stdin {
        engine::DocumentSource::stdin(cli_source_file, doc_dir)
    } else {
        engine::DocumentSource::file_with_resource_dir(input_path, doc_dir)
    };

    let target = if explicit_slides {
        engine::TargetFormat::Slides
    } else if explicit_pdf {
        engine::TargetFormat::Pdf
    } else {
        engine::TargetFormat::Auto
    };

    let opts = engine::PublishOptions {
        target,
        output_path: Some(output_path),
        theme: cli_theme,
        author: cli_author,
        font: cli_font,
        toc: cli_toc,
        bibliography: cli_bibliography,
        exclude_ref: cli_exclude_ref,
        include_ref: cli_include_ref,
        ..Default::default()
    };

    let res = engine::publish(&source, &opts)?;
    Ok((res.is_slides(), res.duration()))
}

#[allow(clippy::too_many_arguments)]
fn compile_markdown_once(
    input_path: &std::path::Path,
    output_path: &std::path::Path,
    doc_dir: Option<&std::path::Path>,
    _config_data: &config::BotoxConfig,
    cli_assets_dir: Option<&std::path::Path>,
    cli_toc: Option<bool>,
    cli_author: Option<&str>,
    cli_bibliography: Option<bool>,
    cli_number_sections: bool,
    cli_exclude_ref: &[String],
    cli_include_ref: &[String],
    is_stdin: bool,
    cli_no_frontmatter: bool,
) -> Result<(usize, std::time::Duration), String> {
    let source = if is_stdin {
        engine::DocumentSource::stdin(None, doc_dir)
    } else {
        engine::DocumentSource::file_with_resource_dir(input_path, doc_dir)
    };

    let opts = engine::PublishOptions {
        target: engine::TargetFormat::Markdown,
        output_path: Some(output_path),
        assets_dir: cli_assets_dir,
        toc: cli_toc,
        author: cli_author,
        bibliography: cli_bibliography,
        number_sections: cli_number_sections,
        exclude_ref: cli_exclude_ref,
        include_ref: cli_include_ref,
        no_frontmatter: cli_no_frontmatter,
        ..Default::default()
    };

    let res = engine::publish(&source, &opts)?;
    match res {
        engine::PublishResult::Markdown { result, duration, .. } => {
            Ok((result.exported_assets.len(), duration))
        }
        _ => Err("Expected markdown export output".to_string()),
    }
}

fn format_mode_str(is_slides: bool, output_path: &std::path::Path) -> &'static str {
    let ext = output_path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    if ext == "md" || ext == "markdown" {
        "Normal Markdown"
    } else if is_slides {
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

    if args[1] == "-V" || args[1] == "--version" || args[1] == "version" {
        println!("botox {}", env!("CARGO_PKG_VERSION"));
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

    if args[1] == "skills" || args[1] == "--skills" {
        skills::handle_skills(&args[2..]);
        return;
    }

    if args[1] == "daemon" || args[1] == "--daemon" || args[1] == "serve" {
        daemon::handle_daemon(&args[2..]);
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
    let mut cli_source_file: Option<String> = None;
    let mut explicit_pdf = false;
    let mut explicit_slides = false;
    let mut explicit_markdown = false;
    let mut cli_assets_dir: Option<PathBuf> = None;
    let mut cli_number_sections = false;
    let mut cli_no_frontmatter = false;
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
            "markdown" | "md" | "--markdown" | "--md" => {
                explicit_markdown = true;
            }
            "--pdf" => {
                explicit_pdf = true;
            }
            "--slides" => {
                explicit_slides = true;
            }
            "--skills" | "skills" => {
                let remaining: Vec<String> = args[1..]
                    .iter()
                    .filter(|&a| a != "--skills" && a != "skills")
                    .cloned()
                    .collect();
                skills::handle_skills(&remaining);
                return;
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
            "--source-file" | "--filename" => {
                i += 1;
                if i < args.len() {
                    cli_source_file = Some(args[i].clone());
                }
            }
            "--assets-dir" => {
                i += 1;
                if i < args.len() {
                    cli_assets_dir = Some(PathBuf::from(&args[i]));
                }
            }
            "-N" | "--number-sections" => {
                cli_number_sections = true;
            }
            "--no-frontmatter" => {
                cli_no_frontmatter = true;
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

    let is_markdown_mode = if explicit_markdown {
        true
    } else if let Some(ref out_f) = output_file {
        let ext = out_f.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        (ext == "md" || ext == "markdown") && !explicit_pdf && !explicit_slides
    } else {
        false
    };

    let is_preliminary_slides = if is_markdown_mode {
        false
    } else if explicit_slides {
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

    let default_ext = if is_markdown_mode {
        "md"
    } else if is_preliminary_slides {
        "html"
    } else {
        "pdf"
    };
    let default_output = if is_stdin {
        if is_markdown_mode {
            PathBuf::from("-")
        } else {
            PathBuf::from(format!("output.{default_ext}"))
        }
    } else {
        compute_default_output_path(&input_path, default_ext)
    };
    let output_path = output_file.unwrap_or(default_output);
    let is_stdout = output_path.as_os_str() == "-";

    let doc_dir = cli_resource_dir.or_else(|| {
        if is_stdin {
            cli_source_file
                .as_ref()
                .and_then(|sf| std::path::Path::new(sf).parent().map(|p| p.to_path_buf()))
                .or_else(|| std::env::current_dir().ok())
        } else {
            input_path.parent().map(|p| p.to_path_buf())
        }
    });

    let (config_data, _) = config::BotoxConfig::load(custom_config.as_deref(), doc_dir.as_deref());

    if is_markdown_mode {
        if !watch_mode {
            match compile_markdown_once(
                &input_path,
                &output_path,
                doc_dir.as_deref(),
                &config_data,
                cli_assets_dir.as_deref(),
                cli_toc,
                cli_author.as_deref(),
                cli_bibliography,
                cli_number_sections,
                &cli_exclude_ref,
                &cli_include_ref,
                is_stdin,
                cli_no_frontmatter,
            ) {
                Ok((assets_count, duration)) => {
                    if !is_stdout {
                        let assets_info = if assets_count > 0 {
                            format!(" (exported {assets_count} diagram assets)")
                        } else {
                            String::new()
                        };
                        println!("Compiled Normal Markdown -> '{}' in {:.2?}{assets_info}", output_path.display(), duration);
                    }
                }
                Err(e) => {
                    eprintln!("Compilation failed: {e}");
                    std::process::exit(1);
                }
            }
            return;
        }

        // Markdown Watch mode:
        match compile_markdown_once(
            &input_path,
            &output_path,
            doc_dir.as_deref(),
            &config_data,
            cli_assets_dir.as_deref(),
            cli_toc,
            cli_author.as_deref(),
            cli_bibliography,
            cli_number_sections,
            &cli_exclude_ref,
            &cli_include_ref,
            false,
            cli_no_frontmatter,
        ) {
            Ok((assets_count, duration)) => {
                let assets_info = if assets_count > 0 {
                    format!(" (exported {assets_count} diagram assets)")
                } else {
                    String::new()
                };
                if is_stdout {
                    eprintln!("[{}] Compiled Normal Markdown -> stdout in {:.2?}{assets_info}", get_timestamp(), duration);
                } else {
                    println!("Compiled Normal Markdown -> '{}' in {:.2?}{assets_info}", output_path.display(), duration);
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

                let now = get_timestamp();
                match compile_markdown_once(
                    &input_path,
                    &output_path,
                    doc_dir.as_deref(),
                    &updated_cfg,
                    cli_assets_dir.as_deref(),
                    cli_toc,
                    cli_author.as_deref(),
                    cli_bibliography,
                    cli_number_sections,
                    &cli_exclude_ref,
                    &cli_include_ref,
                    false,
                    cli_no_frontmatter,
                ) {
                    Ok((assets_count, duration)) => {
                        let assets_info = if assets_count > 0 {
                            format!(" (exported {assets_count} diagram assets)")
                        } else {
                            String::new()
                        };
                        let target = if is_stdout { "stdout".to_string() } else { format!("'{}'", output_path.display()) };
                        let msg = format!("[{now}] Recompiled Normal Markdown -> {target} in {:.2?}{assets_info}", duration);
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
        return;
    }

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
            cli_source_file.as_deref(),
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
        cli_source_file.as_deref(),
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
                cli_source_file.as_deref(),
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
        handle_setup_in_dir(&args, Some(&temp_dir));

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

        // When explicit_slides is set, even normal document is treated as slides
        assert!(detect_is_slides(&fm_doc, doc_md, Path::new("test.md"), true, false));

        // String "true" for slides
        let slides_str_md = "---\nslides: \"true\"\n---\n# Slide";
        let (fm_str, _) = extract_frontmatter(slides_str_md);
        assert!(detect_is_slides(&fm_str, slides_str_md, Path::new("talk.md"), false, false));

        // Presentation: true
        let pres_md = "---\npresentation: true\n---\n# Slide";
        let (fm_pres, _) = extract_frontmatter(pres_md);
        assert!(detect_is_slides(&fm_pres, pres_md, Path::new("talk.md"), false, false));

        // type: deck vs type: report
        let deck_md = "---\ntype: deck\n---\n# Slide";
        let (fm_deck, _) = extract_frontmatter(deck_md);
        assert!(detect_is_slides(&fm_deck, deck_md, Path::new("talk.md"), false, false));

        let report_md = "---\ntype: report\n---\n# Section";
        let (fm_rep, _) = extract_frontmatter(report_md);
        assert!(!detect_is_slides(&fm_rep, report_md, Path::new("talk.md"), false, false));

        // _class: invert
        let class_md = "---\n_class: invert\n---\n# Slide";
        let (fm_cls, _) = extract_frontmatter(class_md);
        assert!(detect_is_slides(&fm_cls, class_md, Path::new("talk.md"), false, false));

        // Filename convention fallback
        let empty_fm = serde_yaml::Value::Mapping(serde_yaml::Mapping::new());
        assert!(detect_is_slides(&empty_fm, "# Title", Path::new("lecture_slides.md"), false, false));
        assert!(detect_is_slides(&empty_fm, "# Title", Path::new("pitch_deck.md"), false, false));
        assert!(detect_is_slides(&empty_fm, "# Title", Path::new("keynote_presentation.md"), false, false));
        assert!(!detect_is_slides(&empty_fm, "# Title", Path::new("thesis.md"), false, false));
    }

    #[test]
    fn test_compute_default_output_path_btx_and_markdown() {
        // .btx.md inputs
        assert_eq!(
            compute_default_output_path(Path::new("paper.btx.md"), "pdf"),
            PathBuf::from("paper.pdf")
        );
        assert_eq!(
            compute_default_output_path(Path::new("deck.btx.md"), "html"),
            PathBuf::from("deck.html")
        );
        assert_eq!(
            compute_default_output_path(Path::new("paper.btx.md"), "md"),
            PathBuf::from("paper.md")
        );

        // .btx inputs
        assert_eq!(
            compute_default_output_path(Path::new("doc.btx"), "pdf"),
            PathBuf::from("doc.pdf")
        );
        assert_eq!(
            compute_default_output_path(Path::new("doc.btx"), "md"),
            PathBuf::from("doc.md")
        );

        // standard .md inputs
        assert_eq!(
            compute_default_output_path(Path::new("report.md"), "pdf"),
            PathBuf::from("report.pdf")
        );
        assert_eq!(
            compute_default_output_path(Path::new("report.md"), "md"),
            PathBuf::from("report.export.md")
        );

        // directory preserving
        assert_eq!(
            compute_default_output_path(Path::new("examples/paper.btx.md"), "pdf"),
            PathBuf::from("examples/paper.pdf")
        );
        assert_eq!(
            compute_default_output_path(Path::new("examples/paper.btx.md"), "md"),
            PathBuf::from("examples/paper.md")
        );
    }

    #[test]
    fn test_is_relevant_watch_file_supports_btx() {
        let out = Path::new("doc.pdf");
        assert!(is_relevant_watch_file(Path::new("document.btx"), out));
        assert!(is_relevant_watch_file(Path::new("document.btx.md"), out));
        assert!(is_relevant_watch_file(Path::new("slides.btx"), out));
        assert!(is_relevant_watch_file(Path::new("slides.btx.md"), out));
        assert!(is_relevant_watch_file(Path::new("sub/module.btx"), out));
        assert!(is_relevant_watch_file(Path::new("diagram.svg"), out));
        assert!(is_relevant_watch_file(Path::new("picture.png"), out));
        assert!(is_relevant_watch_file(Path::new("bibliography.bib"), out));
        assert!(is_relevant_watch_file(Path::new("custom.typ"), out));
        assert!(is_relevant_watch_file(Path::new("botox.yaml"), out));

        // Output path ignored
        assert!(!is_relevant_watch_file(out, out));
        // Hidden files ignored
        assert!(!is_relevant_watch_file(Path::new(".hidden.btx"), out));
        assert!(!is_relevant_watch_file(Path::new(".git/HEAD"), out));
        // Temp/backup files ending with ~ ignored
        assert!(!is_relevant_watch_file(Path::new("document.md~"), out));
        // Irrelevant extensions ignored
        assert!(!is_relevant_watch_file(Path::new("build.rs"), out));
        assert!(!is_relevant_watch_file(Path::new("app.exe"), out));
    }

    #[test]
    fn test_technical_document_end_to_end_pdf_compilation() {
        let md = r#"---
title: "Technical Specification Summary"
theme: technical
---

# Math Layout Cases

1. UTF-8 inside `\text{...}`: $\frac{\text{observé}}{\text{attendu}}$
2. Thin spaces between digits: $1\,000\,000$
3. Backslash-space in math: $\approx 20 \text{ to } 100\ \mu\text{s}$
4. Floor and modulo: $\lfloor N / K \rfloor$ and $N \bmod K$
5. Escaped underscore in text: $K = \min(\text{NB\_THREADS}, N)$

# Nested Bullet Lists

- **Parent Bullet Title**:
  - First nested sub-item
  - Second nested sub-item

# Table With Trailing Caption & `<br>`

| Primitive | Necessary Conditions |
| :--- | :--- |
| Mutex | 1. Mutual exclusion<br>2. Hold and wait |
| Semaphore | 1. Resource pool<br>2. Atomic counter |
Table: Synchronization Primitives Comparison {#tbl:sync}

See @tbl:sync for details.
"#;
        let cfg = config::BotoxConfig::default();
        let (typst_markup, is_slides) = transpile_markdown_to_typst_markup(
            md,
            None,
            &cfg,
            true,
            false,
            None,
            None,
            None,
            None,
            None,
            &[],
            &[],
            Some(Path::new("technical_summary.md")),
        ).expect("transpile should succeed");
        assert!(!is_slides);
        let pdf_bytes = compiler::compile_typst_to_pdf_bytes(&typst_markup, None)
            .expect("Technical document should compile cleanly to PDF");
        assert!(pdf_bytes.starts_with(b"%PDF-"));
    }

    #[test]
    fn test_compile_ex_c_end_to_end() {
        let content = std::fs::read_to_string("ex-c.md").unwrap();
        let cfg = config::BotoxConfig::default();
        let (typst_markup, is_slides) = transpile_markdown_to_typst_markup(
            &content,
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
            Some(Path::new("ex-c.md")),
        ).expect("transpile should succeed");
        assert!(!is_slides);
        let res = compiler::compile_typst_to_pdf_bytes(&typst_markup, None);
        if let Err(ref e) = res {
            eprintln!("EX-C COMPILATION ERROR:\n{e}");
        }
        assert!(res.is_ok());
    }

    #[test]
    fn test_individual_flipped_pages_end_to_end() {
        let content = r#"---
title: Flipped Test
papersize: a4
---

# Page 1 Portrait
Standard portrait introduction text.

::: landscape
# Page 2 Landscape
Wide data table across landscape canvas.
:::

# Page 3 Portrait
Standard portrait conclusion text.
"#;
        let cfg = config::BotoxConfig::default();
        let (typst_markup, is_slides) = transpile_markdown_to_typst_markup(
            content,
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
            None,
        ).expect("transpile should succeed");
        assert!(!is_slides);
        assert!(typst_markup.contains("#pagebreak(weak: true)\n#set page(flipped: true)"));
        assert!(typst_markup.contains("#pagebreak(weak: true)\n#set page(flipped: false)"));
        let pdf_bytes = compiler::compile_typst_to_pdf_bytes(&typst_markup, None)
            .expect("Should compile cleanly to PDF with flipped pages");
        assert!(pdf_bytes.starts_with(b"%PDF-"));
    }
}

