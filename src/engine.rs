//! Publishing Engine Module (Deep Module)
//!
//! Provides a unified, high-level interface for the Botox publishing pipeline:
//! coordinates document intake, configuration cascading, frontmatter analysis,
//! markdown transpilation, slide/document layout synthesis, and backend compilation.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use serde_yaml::Value;

use crate::compiler::{self, BotoxPagesOutput};
use crate::config::{self, BotoxConfig};
use crate::document;
use crate::markdown::{self, MarkdownExportOptions, MarkdownExportResult, MarkdownOptions};
use crate::slides;

/// Represents the source of a document to be published.
#[derive(Debug, Clone)]
pub enum DocumentSource<'a> {
    File {
        path: &'a Path,
        resource_dir: Option<&'a Path>,
    },
    Stdin {
        source_filename: Option<&'a str>,
        resource_dir: Option<&'a Path>,
    },
    Memory {
        content: &'a str,
        source_filename: Option<&'a str>,
        resource_dir: Option<&'a Path>,
    },
}

impl<'a> DocumentSource<'a> {
    pub fn file(path: &'a Path) -> Self {
        Self::File {
            path,
            resource_dir: None,
        }
    }

    pub fn file_with_resource_dir(path: &'a Path, resource_dir: Option<&'a Path>) -> Self {
        Self::File { path, resource_dir }
    }

    pub fn stdin(source_filename: Option<&'a str>, resource_dir: Option<&'a Path>) -> Self {
        Self::Stdin {
            source_filename,
            resource_dir,
        }
    }

    pub fn memory(content: &'a str) -> Self {
        Self::Memory {
            content,
            source_filename: None,
            resource_dir: None,
        }
    }

    pub fn memory_with_meta(
        content: &'a str,
        source_filename: Option<&'a str>,
        resource_dir: Option<&'a Path>,
    ) -> Self {
        Self::Memory {
            content,
            source_filename,
            resource_dir,
        }
    }

    pub fn read_content(&self) -> Result<String, String> {
        match self {
            DocumentSource::File { path, .. } => std::fs::read_to_string(path)
                .map_err(|e| format!("Error reading '{}': {e}", path.display())),
            DocumentSource::Stdin { .. } => {
                use std::io::Read;
                let mut buffer = String::new();
                std::io::stdin()
                    .read_to_string(&mut buffer)
                    .map_err(|e| format!("Error reading from stdin: {e}"))?;
                Ok(buffer)
            }
            DocumentSource::Memory { content, .. } => Ok(content.to_string()),
        }
    }

    pub fn source_path(&self) -> Option<&'a Path> {
        match self {
            DocumentSource::File { path, .. } => Some(path),
            DocumentSource::Stdin { source_filename, .. } => source_filename.map(Path::new),
            DocumentSource::Memory { source_filename, .. } => source_filename.map(Path::new),
        }
    }

    pub fn resource_dir(&self) -> Option<&'a Path> {
        match self {
            DocumentSource::File { path, resource_dir } => {
                resource_dir.or_else(|| path.parent())
            }
            DocumentSource::Stdin { resource_dir, .. } => *resource_dir,
            DocumentSource::Memory { resource_dir, .. } => *resource_dir,
        }
    }

    pub fn filename_display(&self) -> String {
        match self {
            DocumentSource::File { path, .. } => path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("document.md")
                .to_string(),
            DocumentSource::Stdin { source_filename, .. } => {
                source_filename.unwrap_or("document.md").to_string()
            }
            DocumentSource::Memory { source_filename, .. } => {
                source_filename.unwrap_or("document.md").to_string()
            }
        }
    }
}

/// Target format requested by the caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TargetFormat {
    #[default]
    Auto,
    Pdf,
    Slides,
    Markdown,
}

/// Options to guide the publishing pipeline across the interface.
#[derive(Debug, Clone, Default)]
pub struct PublishOptions<'a> {
    pub target: TargetFormat,
    pub output_path: Option<&'a Path>,
    pub custom_config_path: Option<&'a Path>,
    pub theme: Option<&'a str>,
    pub author: Option<&'a str>,
    pub font: Option<&'a str>,
    pub toc: Option<bool>,
    pub bibliography: Option<bool>,
    pub exclude_ref: &'a [String],
    pub include_ref: &'a [String],
    pub assets_dir: Option<&'a Path>,
    pub number_sections: bool,
    pub no_frontmatter: bool,
}

/// Structured result of a publishing run.
#[derive(Debug)]
pub enum PublishResult {
    Pdf {
        output_path: Option<PathBuf>,
        is_slides: bool,
        duration: Duration,
    },
    PdfBytes {
        bytes: Vec<u8>,
        is_slides: bool,
        duration: Duration,
    },
    Pages {
        output: BotoxPagesOutput,
        duration: Duration,
    },
    Markdown {
        result: MarkdownExportResult,
        output_path: Option<PathBuf>,
        duration: Duration,
    },
}

impl PublishResult {
    pub fn is_slides(&self) -> bool {
        match self {
            PublishResult::Pdf { is_slides, .. } => *is_slides,
            PublishResult::PdfBytes { is_slides, .. } => *is_slides,
            PublishResult::Pages { output, .. } => output.is_slides,
            PublishResult::Markdown { .. } => false,
        }
    }

    pub fn duration(&self) -> Duration {
        match self {
            PublishResult::Pdf { duration, .. } => *duration,
            PublishResult::PdfBytes { duration, .. } => *duration,
            PublishResult::Pages { duration, .. } => *duration,
            PublishResult::Markdown { duration, .. } => *duration,
        }
    }
}

/// Extract YAML frontmatter strictly, failing on YAML syntax errors.
pub fn parse_frontmatter_strict(content: &str) -> Result<(Value, &str), String> {
    let (fm_raw, body) = markdown::export::extract_frontmatter_raw(content);
    if fm_raw.trim().is_empty() {
        return Ok((Value::Null, body));
    }
    match serde_yaml::from_str::<Value>(fm_raw) {
        Ok(val) => Ok((val, body)),
        Err(err) => {
            let sanitized = fm_raw.replace(r"\today", r#""\today""#);
            if let Ok(val) = serde_yaml::from_str::<Value>(&sanitized) {
                Ok((val, body))
            } else {
                Err(format!("{err}"))
            }
        }
    }
}

/// Extract YAML frontmatter permissively (fallback to Null on error).
pub fn extract_frontmatter(content: &str) -> (Value, &str) {
    let (fm_raw, body) = markdown::export::extract_frontmatter_raw(content);
    if fm_raw.trim().is_empty() {
        return (Value::Null, body);
    }
    match serde_yaml::from_str::<Value>(fm_raw) {
        Ok(val) => (val, body),
        Err(_) => {
            let sanitized = fm_raw.replace(r"\today", r#""\today""#);
            if let Ok(val) = serde_yaml::from_str::<Value>(&sanitized) {
                (val, body)
            } else {
                (Value::Null, body)
            }
        }
    }
}

/// Extract a string list from YAML value (handles sequences or single string).
pub fn extract_yaml_string_list(val: &Value) -> Vec<String> {
    if let Some(arr) = val.as_sequence() {
        arr.iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect()
    } else if let Some(s) = val.as_str() {
        vec![s.to_string()]
    } else {
        Vec::new()
    }
}

/// Detect whether document content intends to be a presentation slide deck.
pub fn detect_is_slides(
    fm: &Value,
    raw_content: &str,
    input_path: &Path,
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
                if (key_lower == "marp"
                    || key_lower == "slides"
                    || key_lower == "slide"
                    || key_lower == "presentation")
                    && (v.as_bool() == Some(true)
                        || v.as_str()
                            .map(|s| s.eq_ignore_ascii_case("true"))
                            .unwrap_or(false))
                {
                    return true;
                }
                if (key_lower == "type" || key_lower == "format" || key_lower == "document-type")
                    && let Some(s) = v.as_str()
                {
                    let s_lower = s.to_lowercase();
                    if s_lower == "slides"
                        || s_lower == "slide"
                        || s_lower == "presentation"
                        || s_lower == "deck"
                    {
                        return true;
                    }
                    if s_lower == "document"
                        || s_lower == "article"
                        || s_lower == "paper"
                        || s_lower == "report"
                    {
                        return false;
                    }
                }
                if key_lower == "_class" {
                    return true;
                }
            }
        }
    }

    // 2. Direct text scan fallback across frontmatter
    let trimmed = raw_content.trim_start();
    if trimmed.starts_with("---")
        && let Some(end_idx) = trimmed[3..].find("---")
    {
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
            if lower.starts_with("type:")
                && (lower.contains("slide")
                    || lower.contains("deck")
                    || lower.contains("presentation"))
            {
                return true;
            }
        }
    }

    // 3. Filename convention fallback
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

/// Resolve reference filters from frontmatter, configuration, and command options.
pub fn resolve_reference_filters(
    fm: &Value,
    config_data: &BotoxConfig,
    cli_exclude_ref: &[String],
    cli_include_ref: &[String],
) -> (Vec<String>, Vec<String>) {
    let doc_config = config_data.document.clone().unwrap_or_default();
    let mut ref_exclude = Vec::new();
    ref_exclude.extend_from_slice(cli_exclude_ref);
    if let Some(ref cfg_ref) = config_data.references
        && let Some(ref exc) = cfg_ref.exclude
    {
        ref_exclude.extend(exc.clone());
    }
    if let Some(ref cfg_ref) = doc_config.references
        && let Some(ref exc) = cfg_ref.exclude
    {
        ref_exclude.extend(exc.clone());
    }
    if let Some(ref exc) = doc_config.exclude_references {
        ref_exclude.extend(exc.clone());
    }
    if let Some(v) = fm.get("references").and_then(|r| r.get("exclude")) {
        ref_exclude.extend(extract_yaml_string_list(v));
    }
    if let Some(v) = fm
        .get("exclude-references")
        .or_else(|| fm.get("exclude_references"))
        .or_else(|| fm.get("exclude-links"))
        .or_else(|| fm.get("exclude_links"))
    {
        ref_exclude.extend(extract_yaml_string_list(v));
    }

    let mut ref_include = Vec::new();
    ref_include.extend_from_slice(cli_include_ref);
    if let Some(ref cfg_ref) = config_data.references
        && let Some(ref inc) = cfg_ref.include
    {
        ref_include.extend(inc.clone());
    }
    if let Some(ref cfg_ref) = doc_config.references
        && let Some(ref inc) = cfg_ref.include
    {
        ref_include.extend(inc.clone());
    }
    if let Some(ref inc) = doc_config.include_references {
        ref_include.extend(inc.clone());
    }
    if let Some(v) = fm.get("references").and_then(|r| r.get("include")) {
        ref_include.extend(extract_yaml_string_list(v));
    }
    if let Some(v) = fm
        .get("include-references")
        .or_else(|| fm.get("include_references"))
        .or_else(|| fm.get("include-links"))
        .or_else(|| fm.get("include_links"))
    {
        ref_include.extend(extract_yaml_string_list(v));
    }

    (ref_exclude, ref_include)
}

/// Transpiles raw markdown content into compiled Typst markup.
#[allow(clippy::too_many_arguments)]
pub fn transpile_markdown_to_typst_markup(
    raw_content: &str,
    doc_dir: Option<&Path>,
    config_data: &BotoxConfig,
    explicit_pdf: bool,
    explicit_slides: bool,
    cli_toc: Option<bool>,
    cli_author: Option<&str>,
    cli_font: Option<&str>,
    cli_bibliography: Option<bool>,
    cli_theme: Option<&str>,
    cli_exclude_ref: &[String],
    cli_include_ref: &[String],
    source_path: Option<&Path>,
) -> Result<(String, bool), String> {
    let (fm, body_md) = parse_frontmatter_strict(raw_content)?;
    let fallback = source_path.unwrap_or(Path::new("document.md"));
    let is_slides = detect_is_slides(&fm, raw_content, fallback, explicit_slides, explicit_pdf);

    let doc_config = config_data.document.clone().unwrap_or_default();
    let slides_config = config_data.slides.clone().unwrap_or_default();

    let lang = fm
        .get("lang")
        .and_then(|v| v.as_str())
        .or(doc_config.lang.as_deref())
        .unwrap_or("en");
    let biblio_title = fm
        .get("biblio-title")
        .or_else(|| fm.get("bibliography-title"))
        .or_else(|| fm.get("references-title"))
        .and_then(|v| v.as_str());

    let should_enable_bib = if let Some(cli) = cli_bibliography {
        cli
    } else if let Some(fm_bib) = fm
        .get("bibliography")
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

    let toc_title = fm
        .get("toc-title")
        .or_else(|| fm.get("toc_title"))
        .and_then(|v| v.as_str())
        .or(doc_config.toc_title.as_deref());

    let toc_depth = fm
        .get("toc-depth")
        .or_else(|| fm.get("toc_depth"))
        .and_then(|v| v.as_u64())
        .map(|n| n as usize)
        .or(doc_config.toc_depth);

    let (ref_exclude, ref_include) =
        resolve_reference_filters(&fm, config_data, cli_exclude_ref, cli_include_ref);
    let exclude_opt = if ref_exclude.is_empty() {
        None
    } else {
        Some(ref_exclude)
    };
    let include_opt = if ref_include.is_empty() {
        None
    } else {
        Some(ref_include)
    };

    let kroki_url = fm
        .get("kroki-url")
        .or_else(|| fm.get("kroki_url"))
        .or_else(|| fm.get("kroki-endpoint"))
        .or_else(|| fm.get("kroki_endpoint"))
        .and_then(|v| v.as_str())
        .or(doc_config.kroki_url.as_deref());

    let classoption = fm
        .get("classoption")
        .and_then(|v| {
            if let Some(s) = v.as_str() {
                Some(s.to_string())
            } else if let Some(arr) = v.as_sequence() {
                Some(
                    arr.iter()
                        .filter_map(|x| x.as_str())
                        .collect::<Vec<_>>()
                        .join(" "),
                )
            } else {
                None
            }
        })
        .unwrap_or_default();

    let is_landscape = fm
        .get("landscape")
        .and_then(|v| v.as_bool())
        .or_else(|| fm.get("flipped").and_then(|v| v.as_bool()))
        .or_else(|| {
            fm.get("orientation").and_then(|v| v.as_str()).map(|s| {
                s.eq_ignore_ascii_case("landscape") || s.eq_ignore_ascii_case("horizontal")
            })
        })
        .unwrap_or_else(|| classoption.contains("landscape"));

    let theme_explicit = cli_theme.is_some() || fm.get("theme").is_some();
    let theme_raw = cli_theme
        .or_else(|| fm.get("theme").and_then(|v| v.as_str()))
        .or(doc_config.theme.as_deref())
        .unwrap_or("academic");
    let doc_theme = document::DocumentTheme::parse(theme_raw);
    let columns = fm
        .get("columns")
        .and_then(|v| v.as_u64())
        .map(|n| n as usize)
        .or_else(|| {
            if classoption.contains("twocolumn") {
                Some(2)
            } else {
                None
            }
        })
        .or_else(|| {
            if theme_explicit {
                Some(doc_theme.default_columns())
            } else {
                doc_config
                    .columns
                    .or_else(|| Some(doc_theme.default_columns()))
            }
        })
        .unwrap_or_else(|| doc_theme.default_columns());

    let md_opts = MarkdownOptions {
        is_slides,
        bibliography: should_enable_bib,
        lang,
        biblio_title,
        toc_title,
        toc_depth,
        ref_exclude: exclude_opt.as_deref(),
        ref_include: include_opt.as_deref(),
        kroki_url,
        resource_dir: doc_dir,
        base_columns: columns,
        is_landscape,
    };
    let body_typst = markdown::markdown_to_typst_with_options(body_md, &md_opts);

    let emitter: Box<dyn crate::emitter::DocumentEmitter> = if is_slides {
        Box::new(slides::SlideEmitter {
            fm: &fm,
            config: &slides_config,
            cli_author,
            cli_theme,
        })
    } else {
        Box::new(document::ArticleEmitter {
            fm: &fm,
            config: &doc_config,
            cli_toc,
            cli_author,
            cli_font,
            cli_theme,
        })
    };
    let typst_markup = emitter.emit(&body_typst);

    Ok((typst_markup, is_slides))
}

/// Primary publishing entrypoint: executes the end-to-end publishing pipeline.
pub fn publish(
    source: &DocumentSource<'_>,
    options: &PublishOptions<'_>,
) -> Result<PublishResult, String> {
    let raw_content = source.read_content()?;
    let doc_dir = source.resource_dir();
    let src_path = source.source_path();
    let src_filename = source.filename_display();

    let (config_data, _) = config::BotoxConfig::load(options.custom_config_path, doc_dir);

    // Markdown export mode
    if options.target == TargetFormat::Markdown {
        return publish_markdown(&raw_content, source, &config_data, options);
    }

    let explicit_slides = options.target == TargetFormat::Slides;
    let explicit_pdf = options.target == TargetFormat::Pdf;

    let t_transpile_start = Instant::now();
    let (typst_markup, is_slides) = transpile_markdown_to_typst_markup(
        &raw_content,
        doc_dir,
        &config_data,
        explicit_pdf,
        explicit_slides,
        options.toc,
        options.author,
        options.font,
        options.bibliography,
        options.theme,
        options.exclude_ref,
        options.include_ref,
        src_path,
    )
    .map_err(|e| format!("[YAML Frontmatter Error] {src_filename}: {e}"))?;
    let t_transpile = t_transpile_start.elapsed();

    let start_compile = Instant::now();
    if let Some(out_path) = options.output_path {
        compiler::compile_typst_with_source(
            &typst_markup,
            out_path,
            doc_dir,
            Some(&raw_content),
            Some(&src_filename),
        )?;
        let duration = start_compile.elapsed();

        if std::env::var("BOTOX_PROFILE").is_ok() {
            eprintln!(
                "[PROFILE:PIPELINE] markdown_transpile: {t_transpile:.2?}, typst_backend: {duration:.2?}"
            );
        }

        Ok(PublishResult::Pdf {
            output_path: Some(out_path.to_path_buf()),
            is_slides,
            duration,
        })
    } else {
        let bytes = compiler::compile_typst_to_pdf_bytes(&typst_markup, doc_dir)?;
        let duration = start_compile.elapsed();
        Ok(PublishResult::PdfBytes {
            bytes,
            is_slides,
            duration,
        })
    }
}

/// Publishes document to vector preview pages (for daemon / editor preview).
pub fn publish_to_pages(
    source: &DocumentSource<'_>,
    options: &PublishOptions<'_>,
) -> Result<BotoxPagesOutput, String> {
    let raw_content = source.read_content()?;
    let doc_dir = source.resource_dir();
    let src_path = source.source_path();
    let src_filename = source.filename_display();

    let (config_data, _) = config::BotoxConfig::load(options.custom_config_path, doc_dir);
    let explicit_slides = options.target == TargetFormat::Slides;
    let explicit_pdf = options.target == TargetFormat::Pdf;

    let (typst_markup, _) = transpile_markdown_to_typst_markup(
        &raw_content,
        doc_dir,
        &config_data,
        explicit_pdf,
        explicit_slides,
        options.toc,
        options.author,
        options.font,
        options.bibliography,
        options.theme,
        options.exclude_ref,
        options.include_ref,
        src_path,
    )
    .map_err(|e| format!("[YAML Frontmatter Error] {src_filename}: {e}"))?;

    compiler::compile_typst_to_pages(
        &typst_markup,
        doc_dir,
        Some(&raw_content),
        Some(&src_filename),
    )
}

/// Publishes document directly to PDF byte vector in-memory.
pub fn publish_to_pdf_bytes(
    source: &DocumentSource<'_>,
    options: &PublishOptions<'_>,
) -> Result<Vec<u8>, String> {
    let raw_content = source.read_content()?;
    let doc_dir = source.resource_dir();
    let src_path = source.source_path();
    let src_filename = source.filename_display();

    let (config_data, _) = config::BotoxConfig::load(options.custom_config_path, doc_dir);
    let explicit_slides = options.target == TargetFormat::Slides;
    let explicit_pdf = options.target == TargetFormat::Pdf;

    let (typst_markup, _) = transpile_markdown_to_typst_markup(
        &raw_content,
        doc_dir,
        &config_data,
        explicit_pdf,
        explicit_slides,
        options.toc,
        options.author,
        options.font,
        options.bibliography,
        options.theme,
        options.exclude_ref,
        options.include_ref,
        src_path,
    )
    .map_err(|e| format!("[YAML Frontmatter Error] {src_filename}: {e}"))?;

    compiler::compile_typst_to_pdf_bytes(&typst_markup, doc_dir)
}

/// Publishes document to clean standard Markdown with resolved assets.
fn publish_markdown(
    raw_content: &str,
    source: &DocumentSource<'_>,
    config_data: &BotoxConfig,
    options: &PublishOptions<'_>,
) -> Result<PublishResult, String> {
    let start = Instant::now();
    let (fm_raw, _) = markdown::export::extract_frontmatter_raw(raw_content);
    let fm: Value = serde_yaml::from_str(fm_raw).unwrap_or(Value::Null);
    let doc_config = config_data.document.clone().unwrap_or_default();

    let lang = fm
        .get("lang")
        .and_then(|v| v.as_str())
        .or(doc_config.lang.as_deref())
        .unwrap_or("en");

    let biblio_title = fm
        .get("biblio-title")
        .or_else(|| fm.get("bibliography-title"))
        .or_else(|| fm.get("references-title"))
        .and_then(|v| v.as_str());

    let should_enable_bib = if let Some(cli) = options.bibliography {
        cli
    } else if let Some(fm_bib) = fm
        .get("bibliography")
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

    let toc_title = fm
        .get("toc-title")
        .or_else(|| fm.get("toc_title"))
        .and_then(|v| v.as_str())
        .or(doc_config.toc_title.as_deref());

    let toc_depth = fm
        .get("toc-depth")
        .or_else(|| fm.get("toc_depth"))
        .and_then(|v| v.as_u64())
        .map(|n| n as usize)
        .or(doc_config.toc_depth);

    let kroki_url = fm
        .get("kroki-url")
        .or_else(|| fm.get("kroki_url"))
        .or_else(|| fm.get("kroki-endpoint"))
        .or_else(|| fm.get("kroki_endpoint"))
        .and_then(|v| v.as_str())
        .or(doc_config.kroki_url.as_deref());

    let (ref_exclude, ref_include) =
        resolve_reference_filters(&fm, config_data, options.exclude_ref, options.include_ref);
    let exclude_opt = if ref_exclude.is_empty() {
        None
    } else {
        Some(ref_exclude)
    };
    let include_opt = if ref_include.is_empty() {
        None
    } else {
        Some(ref_include)
    };

    let export_opts = MarkdownExportOptions {
        output_path: options.output_path,
        assets_dir: options.assets_dir,
        resource_dir: source.resource_dir(),
        kroki_url,
        bibliography: should_enable_bib,
        lang,
        biblio_title,
        toc_title,
        toc_depth,
        ref_exclude: exclude_opt.as_deref(),
        ref_include: include_opt.as_deref(),
        generate_toc: options.toc.unwrap_or(false),
        number_sections: options.number_sections,
        no_frontmatter: options.no_frontmatter,
    };

    let result = markdown::export_to_normal_markdown(raw_content, &export_opts)?;

    if let Some(out_path) = options.output_path {
        if out_path.as_os_str() == "-" {
            use std::io::Write;
            std::io::stdout()
                .write_all(result.content.as_bytes())
                .map_err(|e| format!("Error writing to stdout: {e}"))?;
        } else {
            if let Some(parent) = out_path.parent()
                && !parent.as_os_str().is_empty()
            {
                std::fs::create_dir_all(parent).map_err(|e| {
                    format!(
                        "Failed to create output directory '{}': {e}",
                        parent.display()
                    )
                })?;
            }
            std::fs::write(out_path, &result.content)
                .map_err(|e| format!("Error writing '{}': {e}", out_path.display()))?;
        }
    }

    let duration = start.elapsed();
    Ok(PublishResult::Markdown {
        result,
        output_path: options.output_path.map(|p| p.to_path_buf()),
        duration,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_detect_is_slides() {
        let slides_md = "---\nmarp: true\n---\n# Slide 1";
        let (fm, _) = extract_frontmatter(slides_md);
        assert!(detect_is_slides(&fm, slides_md, Path::new("test.md"), false, false));

        let doc_md = "---\ntitle: Doc\n---\n# Heading";
        let (fm_doc, _) = extract_frontmatter(doc_md);
        assert!(!detect_is_slides(&fm_doc, doc_md, Path::new("test.md"), false, false));
    }

    #[test]
    fn test_engine_publish_in_memory_pdf() {
        let content = "# Test Document\n\nHello from the deep publishing engine!";
        let source = DocumentSource::memory_with_meta(content, Some("test.md"), None);
        let bytes = publish_to_pdf_bytes(&source, &PublishOptions::default())
            .expect("In-memory publication to PDF bytes should succeed");
        assert!(bytes.starts_with(b"%PDF-"));
    }

    #[test]
    fn test_engine_publish_slides_to_pages() {
        let content = "---\nslides: true\n---\n# Slide 1\nContent 1\n\n---\n# Slide 2\nContent 2";
        let source = DocumentSource::memory_with_meta(content, Some("slides.md"), None);
        let opts = PublishOptions {
            target: TargetFormat::Slides,
            ..Default::default()
        };
        let pages = publish_to_pages(&source, &opts)
            .expect("In-memory publication of slides to pages should succeed");
        assert!(pages.is_slides);
        assert!(pages.num_pages >= 2);
    }

    #[test]
    fn test_engine_publish_markdown_export() {
        let content = "---\ntitle: Sample\n---\n# Header\nParagraph with [Link](https://example.com).";
        let source = DocumentSource::memory_with_meta(content, Some("doc.md"), None);
        let opts = PublishOptions {
            target: TargetFormat::Markdown,
            ..Default::default()
        };
        let res = publish(&source, &opts).expect("Markdown export should succeed");
        match res {
            PublishResult::Markdown { result, .. } => {
                assert!(result.content.contains("# Header"));
                assert!(result.content.contains("https://example.com"));
            }
            _ => panic!("Expected PublishResult::Markdown"),
        }
    }

    #[test]
    fn test_engine_yaml_frontmatter_error() {
        let malformed = "---\ntitle: [unclosed list\n---\n# Heading";
        let source = DocumentSource::memory_with_meta(malformed, Some("bad.md"), None);
        let err = publish_to_pdf_bytes(&source, &PublishOptions::default()).unwrap_err();
        assert!(err.contains("[YAML Frontmatter Error]"));
    }
}
