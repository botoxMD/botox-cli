pub mod callouts;
pub mod escapes;
pub mod math;
pub mod pandoc;
pub mod references;
pub mod tables;

use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

pub use escapes::escape_typst_text;
pub use math::convert_latex_math_to_typst;
pub use pandoc::{
    parse_heading_attributes, parse_image_attributes, preprocess_pandoc,
};
pub use references::{format_references_block, url_matches_pattern};
pub use tables::parse_caption_and_label;

pub use crate::diagrams::{
    parse_code_fence_file, parse_diagram_fence, resolve_file_content,
};

#[derive(Debug, Clone, Default)]
pub struct MarkdownOptions<'a> {
    pub is_slides: bool,
    pub bibliography: bool,
    pub lang: &'a str,
    pub biblio_title: Option<&'a str>,
    pub toc_title: Option<&'a str>,
    pub toc_depth: Option<usize>,
    pub ref_exclude: Option<&'a [String]>,
    pub ref_include: Option<&'a [String]>,
    pub kroki_url: Option<&'a str>,
    pub resource_dir: Option<&'a std::path::Path>,
}

#[allow(clippy::too_many_arguments, dead_code)]
pub fn markdown_to_typst(
    markdown: &str,
    is_slides: bool,
    bibliography: bool,
    lang: &str,
    biblio_title: Option<&str>,
    toc_title: Option<&str>,
    toc_depth: Option<usize>,
    ref_exclude: Option<&[String]>,
    ref_include: Option<&[String]>,
) -> String {
    markdown_to_typst_with_options(
        markdown,
        &MarkdownOptions {
            is_slides,
            bibliography,
            lang,
            biblio_title,
            toc_title,
            toc_depth,
            ref_exclude,
            ref_include,
            kroki_url: None,
            resource_dir: None,
        },
    )
}

pub fn markdown_to_typst_with_options(
    markdown: &str,
    opts: &MarkdownOptions<'_>,
) -> String {
    let is_slides = opts.is_slides;
    let bibliography = opts.bibliography;
    let lang = opts.lang;
    let biblio_title = opts.biblio_title;
    let toc_title = opts.toc_title;
    let toc_depth = opts.toc_depth;
    let ref_exclude = opts.ref_exclude;
    let ref_include = opts.ref_include;

    let preprocessed = preprocess_pandoc(markdown);
    let markdown = &preprocessed;
    let has_ref_command = preprocessed.contains("<!--botox:ref");
    let should_index_citations = bibliography || has_ref_command;

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_MATH);
    options.insert(Options::ENABLE_DEFINITION_LIST);

    // Pass 1: Parse and collect footnote definitions
    let parser1 = Parser::new_ext(markdown, options);
    let mut footnote_defs: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut current_footnote: Option<(String, String)> = None;

    for event in parser1 {
        match event {
            Event::Start(Tag::FootnoteDefinition(name)) => {
                current_footnote = Some((name.to_string(), String::new()));
            }
            Event::End(TagEnd::FootnoteDefinition) => {
                if let Some((name, text)) = current_footnote.take() {
                    footnote_defs.insert(name, text.trim().to_string());
                }
            }
            Event::Text(t) => {
                if let Some((_, ref mut text)) = current_footnote {
                    text.push_str(&escape_typst_text(&t));
                }
            }
            Event::Code(c) => {
                if let Some((_, ref mut text)) = current_footnote {
                    text.push('`');
                    text.push_str(&c);
                    text.push('`');
                }
            }
            _ => {}
        }
    }

    let mut events: Vec<Event> = Parser::new_ext(markdown, options).collect();
    let mut typst = String::with_capacity(markdown.len() * 2);

    let mut list_depth: usize = 0;
    let mut in_table_header = false;
    let mut table_cells: Vec<String> = Vec::new();
    let mut table_col_count = 0;
    let mut table_alignments: Vec<&'static str> = Vec::new();
    let mut current_cell = String::new();
    let mut in_table = false;
    let mut in_footnote_def = false;
    let mut references: Vec<(String, String)> = Vec::new();
    let mut in_link = false;
    let mut current_link: Option<(String, String)> = None;
    let mut current_image: Option<(String, String)> = None;
    let mut current_heading: Option<(usize, String)> = None;
    let mut pending_table_caption: Option<(String, Option<String>)> = None;

    let num_events = events.len();
    let mut i = 0;
    while i < num_events {
        let event = std::mem::replace(&mut events[i], Event::Text("".into()));
        match event {
            Event::Start(tag) => {
                if let Tag::FootnoteDefinition(_) = tag {
                    in_footnote_def = true;
                    i += 1;
                    continue;
                }
                if in_footnote_def {
                    i += 1;
                    continue;
                }
                match tag {
                    Tag::Paragraph => {
                        if let Some(Event::Text(next_text)) = events.get(i + 1) {
                            let trimmed = next_text.trim();
                            if (trimmed.starts_with("Table:") || trimmed.starts_with(": ")) && trimmed.len() > 2 {
                                let mut j = i + 2;
                                let mut is_tbl = false;
                                while j < num_events {
                                    match &events[j] {
                                        Event::End(TagEnd::Paragraph) => {
                                            if let Some(Event::Start(Tag::Table(_))) = events.get(j + 1) {
                                                is_tbl = true;
                                            }
                                            break;
                                        }
                                        _ => j += 1,
                                    }
                                }
                                if is_tbl {
                                    let (caption, label) = parse_caption_and_label(trimmed);
                                    pending_table_caption = Some((caption, label));
                                    i = j + 1;
                                    continue;
                                }
                            }
                        }
                    }
                    Tag::Heading { level, .. } => {
                        let lvl = match level {
                            HeadingLevel::H1 => 1,
                            HeadingLevel::H2 => 2,
                            HeadingLevel::H3 => 3,
                            HeadingLevel::H4 => 4,
                            HeadingLevel::H5 => 5,
                            HeadingLevel::H6 => 6,
                        };
                        current_heading = Some((lvl, String::new()));
                    }
                    Tag::BlockQuote(_) => {
                        typst.push_str("#quote(block: true)[");
                    }
                    Tag::CodeBlock(kind) => {
                        let mut code = String::new();
                        let mut j = i + 1;
                        while j < num_events {
                            match &events[j] {
                                Event::Text(t) => {
                                    code.push_str(t);
                                    j += 1;
                                }
                                Event::End(TagEnd::CodeBlock) => {
                                    break;
                                }
                                _ => {
                                    j += 1;
                                }
                            }
                        }
                        i = j + 1;

                        if let pulldown_cmark::CodeBlockKind::Fenced(ref fence) = kind
                            && let Some(spec) = parse_diagram_fence(fence) {
                                let mut diag_code = code.clone();
                                let mut file_to_load = spec.file.clone();
                                if file_to_load.is_none() {
                                    let trimmed = code.trim();
                                    if let Some(rest) = trimmed.strip_prefix("!include ") {
                                        file_to_load = Some(rest.trim().to_string());
                                    } else if let Some(rest) = trimmed.strip_prefix("include: ") {
                                        file_to_load = Some(rest.trim().to_string());
                                    } else if let Some(rest) = trimmed.strip_prefix("file: ") {
                                        file_to_load = Some(rest.trim().to_string());
                                    } else if !trimmed.contains('\n')
                                        && (trimmed.ends_with(".uxf")
                                            || trimmed.ends_with(".umlet")
                                            || trimmed.ends_with(".puml")
                                            || trimmed.ends_with(".plantuml")
                                            || trimmed.ends_with(".mmd")
                                            || trimmed.ends_with(".mermaid")
                                            || trimmed.ends_with(".dot"))
                                    {
                                        file_to_load = Some(trimmed.to_string());
                                    }
                                }

                                if let Some(ref file_path) = file_to_load {
                                    if let Some(loaded) = resolve_file_content(file_path, opts.resource_dir) {
                                        diag_code = loaded;
                                    } else {
                                        typst.push_str(&format!(
                                            "\n#botox_callout(\"warning\", \"Diagram file not found: {}\")[\nCould not load referenced diagram file `{}`.\n]\n\n",
                                            escape_typst_text(file_path),
                                            escape_typst_text(file_path),
                                        ));
                                        continue;
                                    }
                                }

                                match crate::diagrams::render_diagram(&spec.diagram_type, &diag_code, opts.kroki_url) {
                                    Ok(cached_path) => {
                                        let path_str = cached_path.to_string_lossy().replace('\\', "/");
                                        let mut img_args = vec![format!("\"{path_str}\"")];
                                        if let Some(ref w) = spec.width {
                                            img_args.push(format!("width: {w}"));
                                        }
                                        if let Some(ref h) = spec.height {
                                            img_args.push(format!("height: {h}"));
                                        }
                                        let img_call = format!("image({})", img_args.join(", "));

                                        if spec.caption.is_some() || spec.id.is_some() {
                                            let mut fig = format!("\n#figure({img_call}");
                                            if let Some(ref cap) = spec.caption {
                                                let escaped_cap = escape_typst_text(cap);
                                                fig.push_str(&format!(", caption: [{escaped_cap}]"));
                                            }
                                            fig.push(')');
                                            if let Some(ref label) = spec.id {
                                                fig.push_str(&format!(" <{label}>"));
                                            }
                                            fig.push_str("\n\n");
                                            typst.push_str(&fig);
                                        } else {
                                            typst.push_str(&format!("\n#align(center)[#{img_call}]\n\n"));
                                        }
                                    }
                                    Err(err) => {
                                        let is_not_configured = err.contains("not configured");
                                        let title = if is_not_configured {
                                            "Diagram rendering requires Kroki configuration".to_string()
                                        } else {
                                            "Diagram rendering failed: Kroki unreachable".to_string()
                                        };
                                        let instruction = if is_not_configured {
                                            "\nTo render this diagram, configure a Kroki server endpoint.\n\nYou can configure the public service (e.g. `kroki_url: https://kroki.io` or `https://demo.kroki.io`) or run a self-hosted instance (e.g. `http://localhost:8000`) in `botox.yaml`, document frontmatter, or via the `KROKI_ENDPOINT` environment variable.\n\n"
                                        } else {
                                            ""
                                        };
                                        let raw_lang = &spec.diagram_type;
                                        let clean_code = if diag_code.ends_with('\n') {
                                            diag_code
                                        } else {
                                            format!("{diag_code}\n")
                                        };
                                        let mut max_ticks = 0;
                                        let mut cur_ticks = 0;
                                        for ch in clean_code.chars() {
                                            if ch == '`' {
                                                cur_ticks += 1;
                                                if cur_ticks > max_ticks { max_ticks = cur_ticks; }
                                            } else {
                                                cur_ticks = 0;
                                            }
                                        }
                                        let fence_str = "`".repeat((max_ticks + 1).max(3));
                                        typst.push_str(&format!(
                                            "\n#botox_callout(\"warning\", \"{title}\")[\n{instruction}{fence_str}{raw_lang}\n{clean_code}{fence_str}\n]\n\n"
                                        ));
                                    }
                                }
                                continue;
                            }

                        // Regular code block (fenced or indented)
                        let mut final_code = code;
                        let lang = match &kind {
                            pulldown_cmark::CodeBlockKind::Fenced(l) => {
                                let (clean_lang, code_file) = parse_code_fence_file(l.as_ref());
                                if let Some(ref cf) = code_file {
                                    if let Some(loaded) = resolve_file_content(cf, opts.resource_dir) {
                                        final_code = loaded;
                                    } else {
                                        typst.push_str(&format!(
                                            "\n#botox_callout(\"warning\", \"Code file not found: {}\")[\nCould not load referenced code file `{}`.\n]\n\n",
                                            escape_typst_text(cf),
                                            escape_typst_text(cf),
                                        ));
                                        continue;
                                    }
                                }
                                clean_lang
                            }
                            pulldown_cmark::CodeBlockKind::Indented => String::new(),
                        };
                        let mut max_ticks = 0;
                        let mut cur_ticks = 0;
                        for ch in final_code.chars() {
                            if ch == '`' {
                                cur_ticks += 1;
                                if cur_ticks > max_ticks { max_ticks = cur_ticks; }
                            } else {
                                cur_ticks = 0;
                            }
                        }
                        let fence_str = "`".repeat((max_ticks + 1).max(3));
                        let clean_code = if final_code.ends_with('\n') {
                            final_code
                        } else {
                            format!("{final_code}\n")
                        };
                        typst.push_str(&format!("{fence_str}{lang}\n{clean_code}{fence_str}\n\n"));
                        continue;
                    }
                    Tag::List(_) => {
                        list_depth += 1;
                    }
                    Tag::Item => {
                        let indent = "  ".repeat(list_depth.saturating_sub(1));
                        typst.push_str(&indent);
                        typst.push_str("- ");
                    }
                    Tag::Emphasis => {
                        if let Some((_, ref mut h_buf)) = current_heading {
                            h_buf.push('_');
                        } else if in_table {
                            current_cell.push('_');
                        } else {
                            typst.push('_');
                        }
                    }
                    Tag::Strong => {
                        if let Some((_, ref mut h_buf)) = current_heading {
                            h_buf.push('*');
                        } else if in_table {
                            current_cell.push('*');
                        } else {
                            typst.push('*');
                        }
                    }
                    Tag::Strikethrough => {
                        if in_table { current_cell.push_str("#strike["); } else { typst.push_str("#strike["); }
                    }
                    Tag::Subscript => {
                        if in_table { current_cell.push_str("#sub["); } else { typst.push_str("#sub["); }
                    }
                    Tag::Superscript => {
                        if in_table { current_cell.push_str("#super["); } else { typst.push_str("#super["); }
                    }
                    Tag::DefinitionList => {
                        typst.push('\n');
                    }
                    Tag::DefinitionListTitle => {
                        typst.push_str("\n/ ");
                    }
                    Tag::DefinitionListDefinition => {}
                    Tag::FootnoteDefinition(_) => {
                        in_footnote_def = true;
                    }
                    Tag::Link { dest_url, .. } => {
                        in_link = true;
                        let is_external = dest_url.starts_with("http://")
                            || dest_url.starts_with("https://")
                            || dest_url.starts_with("ftp://");

                        let is_excluded = if is_external {
                            let not_in_include = if let Some(inc) = ref_include {
                                !inc.iter().any(|pat| url_matches_pattern(&dest_url, pat))
                            } else {
                                false
                            };
                            let in_exclude = if let Some(exc) = ref_exclude {
                                exc.iter().any(|pat| url_matches_pattern(&dest_url, pat))
                            } else {
                                false
                            };
                            not_in_include || in_exclude
                        } else {
                            true
                        };

                        if should_index_citations && is_external && !is_excluded {
                            current_link = Some((dest_url.to_string(), String::new()));
                        } else {
                            current_link = None;
                        }
                        let safe_url = dest_url.replace('\\', "/").replace('"', "\\\"");
                        let link_code = format!("#link(\"{safe_url}\")[");
                        if in_table { current_cell.push_str(&link_code); } else { typst.push_str(&link_code); }
                    }
                    Tag::Image { dest_url, .. } => {
                        current_image = Some((dest_url.to_string(), String::new()));
                    }
                    Tag::Table(alignments) => {
                        in_table = true;
                        table_cells.clear();
                        table_col_count = 0;
                        table_alignments = alignments.into_iter().map(|a| match a {
                            pulldown_cmark::Alignment::Center => "center",
                            pulldown_cmark::Alignment::Right => "right",
                            _ => "left",
                        }).collect();
                    }
                    Tag::TableHead => {
                        in_table_header = true;
                    }
                    Tag::TableRow => {}
                    Tag::TableCell => {
                        current_cell.clear();
                    }
                    _ => {}
                }
            }
            Event::End(tag) => {
                if let TagEnd::FootnoteDefinition = tag {
                    in_footnote_def = false;
                    i += 1;
                    continue;
                }
                if in_footnote_def {
                    i += 1;
                    continue;
                }
                match tag {
                    TagEnd::Paragraph => {
                        typst.push_str("\n\n");
                    }
                    TagEnd::Heading(_) => {
                        if let Some((lvl, h_text)) = current_heading.take() {
                            let (clean_title, is_unnumbered, id) = parse_heading_attributes(&h_text);
                            if is_unnumbered {
                                typst.push_str(&format!("#heading(level: {lvl}, numbering: none)[{clean_title}]"));
                            } else {
                                let prefix = match lvl {
                                    1 => "=",
                                    2 => "==",
                                    3 => "===",
                                    4 => "====",
                                    5 => "=====",
                                    _ => "======",
                                };
                                typst.push_str(&format!("{prefix} {clean_title}"));
                            }
                            if let Some(ref label) = id {
                                typst.push_str(&format!(" <{label}>"));
                            }
                            typst.push_str("\n\n");
                        } else {
                            typst.push_str("\n\n");
                        }
                    }
                    TagEnd::BlockQuote(_) => {
                        typst.push_str("]\n\n");
                    }
                    TagEnd::CodeBlock => {}
                    TagEnd::List(_) => {
                        list_depth = list_depth.saturating_sub(1);
                        if list_depth == 0 {
                            typst.push('\n');
                        }
                    }
                    TagEnd::Item => {
                        typst.push('\n');
                    }
                    TagEnd::Emphasis => {
                        if let Some((_, ref mut h_buf)) = current_heading {
                            h_buf.push('_');
                        } else if in_table {
                            current_cell.push('_');
                        } else {
                            typst.push('_');
                        }
                    }
                    TagEnd::Strong => {
                        if let Some((_, ref mut h_buf)) = current_heading {
                            h_buf.push('*');
                        } else if in_table {
                            current_cell.push('*');
                        } else {
                            typst.push('*');
                        }
                    }
                    TagEnd::Strikethrough => {
                        if in_table { current_cell.push(']'); } else { typst.push(']'); }
                    }
                    TagEnd::Subscript => {
                        if in_table { current_cell.push(']'); } else { typst.push(']'); }
                    }
                    TagEnd::Superscript => {
                        if in_table { current_cell.push(']'); } else { typst.push(']'); }
                    }
                    TagEnd::Link => {
                        in_link = false;
                        if in_table { current_cell.push(']'); } else { typst.push(']'); }
                        if let Some((url, link_text)) = current_link.take() {
                            let trimmed = link_text.trim();
                            // Do not append citation bracket [1] if link only wraps an image badge or is empty
                            let is_image_only = trimmed.is_empty() || trimmed.contains("#image(");
                            let already_citation = (trimmed.starts_with('[') && trimmed.ends_with(']'))
                                || trimmed.parse::<usize>().is_ok();

                            if !is_image_only && !already_citation {
                                let idx = if let Some(pos) = references.iter().position(|(u, _)| u == &url) {
                                    pos + 1
                                } else {
                                    references.push((url.clone(), trimmed.to_string()));
                                    references.len()
                                };
                                let cite_code = format!(" #link(<bib-{idx}>)[\\[{idx}\\]]");
                                if in_table {
                                    current_cell.push_str(&cite_code);
                                } else {
                                    typst.push_str(&cite_code);
                                }
                            }
                        }
                    }
                    TagEnd::Image => {
                        let mut image_attrs = (None, None, None);
                        if let Some(Event::Text(next_text)) = events.get_mut(i + 1) {
                            let trimmed = next_text.trim_start();
                            if trimmed.starts_with('{')
                                && let Some(brace_end) = trimmed.find('}') {
                                    let attr_part = &trimmed[1..brace_end];
                                    image_attrs = parse_image_attributes(attr_part);
                                    let remainder = trimmed[brace_end + 1..].to_string();
                                    *next_text = remainder.into();
                                }
                        }

                        if let Some((url, alt)) = current_image.take() {
                            let alt = alt.trim();
                            let (width, height, id) = image_attrs;

                            let norm_url = url.trim().to_lowercase();
                            let is_diagram_file = norm_url.ends_with(".uxf")
                                || norm_url.ends_with(".umlet")
                                || norm_url.ends_with(".puml")
                                || norm_url.ends_with(".plantuml")
                                || norm_url.ends_with(".mmd")
                                || norm_url.ends_with(".mermaid")
                                || norm_url.ends_with(".dot")
                                || norm_url.ends_with(".gv")
                                || norm_url.ends_with(".ditaa")
                                || norm_url.ends_with(".wavedrom")
                                || norm_url.ends_with(".bytefield")
                                || norm_url.ends_with(".bpmn");

                            let final_img_path = if is_diagram_file {
                                let diag_type = if norm_url.ends_with(".uxf") || norm_url.ends_with(".umlet") {
                                    "umlet"
                                } else if norm_url.ends_with(".puml") || norm_url.ends_with(".plantuml") {
                                    "plantuml"
                                } else if norm_url.ends_with(".mmd") || norm_url.ends_with(".mermaid") {
                                    "mermaid"
                                } else if norm_url.ends_with(".dot") || norm_url.ends_with(".gv") {
                                    "graphviz"
                                } else if norm_url.ends_with(".ditaa") {
                                    "ditaa"
                                } else if norm_url.ends_with(".wavedrom") {
                                    "wavedrom"
                                } else if norm_url.ends_with(".bytefield") {
                                    "bytefield"
                                } else if norm_url.ends_with(".bpmn") {
                                    "bpmn"
                                } else {
                                    "mermaid"
                                };

                                if let Some(content) = resolve_file_content(&url, opts.resource_dir) {
                                    match crate::diagrams::render_diagram(diag_type, &content, opts.kroki_url) {
                                        Ok(cached_path) => cached_path.to_string_lossy().replace('\\', "/"),
                                        Err(err) => {
                                            let is_not_configured = err.contains("not configured");
                                            let (title, message) = if is_not_configured {
                                                (
                                                    "Diagram rendering requires Kroki configuration".to_string(),
                                                    format!(
                                                        "To render referenced diagram `{}` (type `{}`), configure a Kroki server endpoint.\n\nYou can configure the public service (e.g. `kroki_url: https://kroki.io` or `https://demo.kroki.io`) or a self-hosted instance (e.g. `http://localhost:8000`) in `botox.yaml`, document frontmatter, or via the `KROKI_ENDPOINT` environment variable.",
                                                        escape_typst_text(&url),
                                                        diag_type
                                                    ),
                                                )
                                            } else {
                                                (
                                                    format!("Diagram rendering failed: {}", escape_typst_text(&url)),
                                                    format!("Could not render diagram file `{}` via Kroki: {err}", escape_typst_text(&url)),
                                                )
                                            };
                                            typst.push_str(&format!(
                                                "\n#botox_callout(\"warning\", \"{title}\")[\n{message}\n]\n\n"
                                            ));
                                            continue;
                                        }
                                    }
                                } else {
                                    typst.push_str(&format!(
                                        "\n#botox_callout(\"warning\", \"Diagram file not found: {}\")[\nCould not find diagram file `{}`.\n]\n\n",
                                        escape_typst_text(&url),
                                        escape_typst_text(&url),
                                    ));
                                    continue;
                                }
                            } else if url.starts_with("http://") || url.starts_with("https://") {
                                match crate::diagrams::fetch_and_cache_remote_image(&url) {
                                    Ok(cached_path) => cached_path.to_string_lossy().replace('\\', "/"),
                                    Err(_) => {
                                        let fallback_text = if !alt.is_empty() {
                                            escape_typst_text(alt)
                                        } else {
                                            escape_typst_text(&url)
                                        };
                                        if in_table {
                                            current_cell.push_str(&fallback_text);
                                        } else if current_link.is_some() {
                                            typst.push_str(&fallback_text);
                                        } else {
                                            typst.push_str(&format!("[{fallback_text}]"));
                                        }
                                        continue;
                                    }
                                }
                            } else {
                                url.replace('\\', "/").replace('"', "\\\"")
                            };

                            let mut img_args = vec![format!("\"{final_img_path}\"")];
                            if let Some(ref w) = width {
                                img_args.push(format!("width: {w}"));
                            }
                            if let Some(ref h) = height {
                                img_args.push(format!("height: {h}"));
                            }
                            let img_call = format!("image({})", img_args.join(", "));

                            if in_table {
                                current_cell.push_str(&format!("#{img_call}"));
                            } else if in_link {
                                let alt_attr = if !alt.is_empty() {
                                    format!(", alt: \"{}\"", alt.replace('\\', "/").replace('"', "\\\""))
                                } else {
                                    String::new()
                                };
                                let mut link_img_args = vec![format!("\"{final_img_path}\"{alt_attr}")];
                                if let Some(ref w) = width {
                                    link_img_args.push(format!("width: {w}"));
                                }
                                if let Some(ref h) = height {
                                    link_img_args.push(format!("height: {h}"));
                                } else if width.is_none() {
                                    link_img_args.push("height: 1.35em".to_string());
                                }
                                let link_img_call = format!("image({})", link_img_args.join(", "));
                                let boxed_img = format!("#box(baseline: 20%, {link_img_call}) ");
                                typst.push_str(&boxed_img);
                                if let Some((_, ref mut l_text)) = current_link {
                                    l_text.push_str(&boxed_img);
                                }
                            } else if is_slides {
                                if alt == "bg" || alt.starts_with("bg ") {
                                    typst.push_str(&format!("\n#place(top + left, dx: 0pt, dy: 0pt, image(\"{final_img_path}\", width: 100%, height: 100%, fit: \"cover\"))\n\n"));
                                } else if !alt.is_empty() || id.is_some() {
                                    let mut fig = format!("\n#figure({img_call}");
                                    if !alt.is_empty() {
                                        fig.push_str(&format!(", caption: [{alt}]"));
                                    }
                                    fig.push(')');
                                    if let Some(ref label) = id {
                                        fig.push_str(&format!(" <{label}>"));
                                    }
                                    fig.push_str("\n\n");
                                    typst.push_str(&fig);
                                } else {
                                    typst.push_str(&format!("\n#align(center)[#{img_call}]\n\n"));
                                }
                            } else {
                                if !alt.is_empty() || id.is_some() {
                                    let mut fig = format!("\n#figure({img_call}");
                                    if !alt.is_empty() {
                                        fig.push_str(&format!(", caption: [{alt}]"));
                                    }
                                    fig.push(')');
                                    if let Some(ref label) = id {
                                        fig.push_str(&format!(" <{label}>"));
                                    }
                                    fig.push_str("\n\n");
                                    typst.push_str(&fig);
                                } else {
                                    typst.push_str(&format!("\n#align(center)[#{img_call}]\n\n"));
                                }
                            }
                        }
                    }
                    TagEnd::DefinitionList => {
                        typst.push('\n');
                    }
                    TagEnd::DefinitionListTitle => {
                        typst.push_str(": ");
                    }
                    TagEnd::DefinitionListDefinition => {
                        typst.push_str("\n\n");
                    }
                    TagEnd::TableHead => {
                        in_table_header = false;
                    }
                    TagEnd::TableRow => {}
                    TagEnd::TableCell => {
                        if in_table_header {
                            table_col_count += 1;
                        }
                        table_cells.push(current_cell.trim().to_string());
                        current_cell.clear();
                    }
                    TagEnd::Table => {
                        in_table = false;
                        let cols = if table_col_count > 0 { table_col_count } else { 1 };
                        let align_str = if !table_alignments.is_empty() {
                            let items: Vec<&str> = table_alignments.iter().copied().take(cols).collect();
                            items.join(", ")
                        } else {
                            "left".to_string()
                        };
                        let mut tbl_str = format!("table(\n  columns: {cols},\n  align: ({align_str}),\n");
                        if table_col_count > 0 && table_cells.len() >= table_col_count {
                            let header_slice = &table_cells[..table_col_count];
                            let header_args: Vec<String> = header_slice
                                .iter()
                                .map(|c| format!("[*{c}*]"))
                                .collect();
                            tbl_str.push_str(&format!("  table.header({}),\n", header_args.join(", ")));
                            for cell in &table_cells[table_col_count..] {
                                tbl_str.push_str(&format!("  [{cell}],\n"));
                            }
                        } else {
                            for cell in &table_cells {
                                tbl_str.push_str(&format!("  [{cell}],\n"));
                            }
                        }
                        tbl_str.push(')');

                        if let Some((caption, label)) = pending_table_caption.take() {
                            let escaped_caption = escape_typst_text(&caption);
                            let mut fig = format!("\n#figure(\n  {tbl_str},\n  caption: [{escaped_caption}],\n)");
                            if let Some(ref l) = label {
                                fig.push_str(&format!(" <{l}>"));
                            }
                            fig.push_str("\n\n");
                            typst.push_str(&fig);
                        } else {
                            typst.push_str(&format!("#{tbl_str}\n\n"));
                        }
                    }
                    _ => {}
                }
            }
            Event::Text(text) => {
                if in_footnote_def {
                    i += 1;
                    continue;
                }
                let escaped = escape_typst_text(&text);
                if let Some((_, ref mut h_buf)) = current_heading {
                    h_buf.push_str(&escaped);
                    i += 1;
                    continue;
                }
                if let Some((_, ref mut img_alt)) = current_image {
                    img_alt.push_str(&escaped);
                    i += 1;
                    continue;
                }
                if let Some((_, ref mut link_text)) = current_link {
                    link_text.push_str(&escaped);
                }
                if in_table {
                    current_cell.push_str(&escaped);
                } else {
                    typst.push_str(&escaped);
                }
            }
            Event::Html(html) | Event::InlineHtml(html) => {
                if in_footnote_def {
                    i += 1;
                    continue;
                }
                if html.contains("<!--botox:callout:start:") {
                    let rest = html.split("<!--botox:callout:start:").nth(1).unwrap_or("");
                    if let Some(content) = rest.split("-->").next() {
                        let mut parts = content.splitn(2, ':');
                        let kind = parts.next().unwrap_or("note");
                        let title = parts.next().unwrap_or("");
                        let escaped_title = escape_typst_text(title);
                        typst.push_str(&format!("\n#botox_callout(\"{kind}\", \"{escaped_title}\")[\n"));
                    }
                } else if html.contains("<!--botox:callout:end-->") {
                    typst.push_str("\n]\n\n");
                } else if html.contains("<!--botox:pause-->")
                    || html.trim() == "<!-- pause -->"
                    || html.trim() == "<!--pause-->"
                    || html.contains("class=\"pause\"")
                    || html.contains("class='pause'")
                {
                    typst.push_str("\n#botox_pause()\n\n");
                } else if html.contains("<!--botox:colbreak-->")
                    || html.trim() == "<!-- colbreak -->"
                    || html.trim() == "<!--colbreak-->"
                    || html.trim() == "<!-- columnbreak -->"
                    || html.trim() == "<!--columnbreak-->"
                    || html.contains("class=\"colbreak\"")
                    || html.contains("class=\"columnbreak\"")
                    || html.contains("class='colbreak'")
                    || html.contains("class='columnbreak'")
                {
                    typst.push_str("\n#colbreak()\n\n");
                } else if html.contains("<!--botox:pagebreak-->")
                    || html.trim() == "<!-- pagebreak -->"
                    || html.trim() == "<!--pagebreak-->"
                    || html.trim() == "<!-- newpage -->"
                    || html.trim() == "<!--newpage-->"
                    || html.contains("page-break-after")
                    || html.contains("page-break-before")
                    || html.contains("class=\"pagebreak\"")
                    || html.contains("class=\"newpage\"")
                    || html.contains("class='pagebreak'")
                    || html.contains("class='newpage'")
                {
                    typst.push_str("\n#pagebreak()\n\n");
                } else if html.contains("<!--botox:toc") {
                    let custom_title = if html.contains("<!--botox:toc:") {
                        let rest = html.split("<!--botox:toc:").nth(1).unwrap_or("");
                        let inner = rest.split("-->").next().unwrap_or("").trim();
                        if !inner.is_empty() {
                            Some(inner.to_string())
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    let title_to_use = custom_title.as_deref().or(toc_title);
                    let depth = toc_depth.unwrap_or(3);
                    if let Some(t) = title_to_use {
                        let escaped_t = escape_typst_text(t);
                        typst.push_str(&format!("\n#outline(title: \"{escaped_t}\", depth: {depth})\n#v(1.5em)\n\n"));
                    } else {
                        typst.push_str(&format!("\n#outline(depth: {depth})\n#v(1.5em)\n\n"));
                    }
                } else if html.contains("<!--botox:ref") {
                    let custom_title = if html.contains("<!--botox:ref:") {
                        let rest = html.split("<!--botox:ref:").nth(1).unwrap_or("");
                        let inner = rest.split("-->").next().unwrap_or("").trim();
                        inner.to_string()
                    } else {
                        String::new()
                    };
                    typst.push_str(&format!("\nBOTOX_REF_PLACEHOLDER_START:{custom_title}:BOTOX_REF_PLACEHOLDER_END\n\n"));
                } else if html.trim().starts_with("<!--") && html.trim().ends_with("-->") {
                    // Raw HTML comment: ignore
                } else {
                    let trimmed = html.trim().to_lowercase();
                    if trimmed == "<br>" || trimmed == "<br/>" || trimmed == "<br />" {
                        typst.push_str(" \\ \n");
                    } else {
                        let escaped = escape_typst_text(&html);
                        if in_table {
                            current_cell.push_str(&escaped);
                        } else {
                            typst.push_str(&escaped);
                        }
                    }
                }
            }
            Event::FootnoteReference(name) => {
                if in_footnote_def {
                    i += 1;
                    continue;
                }
                let body = footnote_defs.get(name.as_ref()).cloned().unwrap_or_default();
                typst.push_str(&format!("#footnote[{body}]"));
            }
            Event::TaskListMarker(checked) => {
                if in_footnote_def {
                    i += 1;
                    continue;
                }
                if checked {
                    let s = r"\[x\] ";
                    if in_table { current_cell.push_str(s); } else { typst.push_str(s); }
                } else {
                    let s = r"\[ \] ";
                    if in_table { current_cell.push_str(s); } else { typst.push_str(s); }
                }
            }
            Event::Code(code) => {
                if in_footnote_def {
                    i += 1;
                    continue;
                }
                if let Some((_, ref mut h_buf)) = current_heading {
                    h_buf.push('`');
                    h_buf.push_str(&code);
                    h_buf.push('`');
                    i += 1;
                    continue;
                }
                if let Some((_, ref mut img_alt)) = current_image {
                    img_alt.push('`');
                    img_alt.push_str(&code);
                    img_alt.push('`');
                    i += 1;
                    continue;
                }
                if let Some((_, ref mut link_text)) = current_link {
                    link_text.push('`');
                    link_text.push_str(&code);
                    link_text.push('`');
                }
                if in_table {
                    current_cell.push('`');
                    current_cell.push_str(&code);
                    current_cell.push('`');
                } else {
                    typst.push('`');
                    typst.push_str(&code);
                    typst.push('`');
                }
            }
            Event::InlineMath(math) => {
                if in_footnote_def {
                    i += 1;
                    continue;
                }
                if let Some((_, ref mut h_buf)) = current_heading {
                    let converted = convert_latex_math_to_typst(&math);
                    h_buf.push('$');
                    h_buf.push_str(converted.trim());
                    h_buf.push('$');
                    i += 1;
                    continue;
                }
                let converted = convert_latex_math_to_typst(&math);
                let s = format!("${}$", converted.trim());
                if in_table {
                    current_cell.push_str(&s);
                } else {
                    typst.push_str(&s);
                }
            }
            Event::DisplayMath(math) => {
                if in_footnote_def {
                    i += 1;
                    continue;
                }
                let mut eq_label = None;
                if let Some(Event::Text(next_text)) = events.get_mut(i + 1) {
                    let trimmed = next_text.trim_start();
                    if trimmed.starts_with('{')
                        && let Some(brace_end) = trimmed.find('}') {
                            let attr = trimmed[1..brace_end].trim();
                            if let Some(stripped) = attr.strip_prefix('#') {
                                eq_label = Some(stripped.replace(':', "-"));
                                let remainder = trimmed[brace_end + 1..].to_string();
                                *next_text = remainder.into();
                            }
                        }
                }
                let mut math_str = math.to_string();
                if let Some(pos) = math_str.find(r"\label{")
                    && let Some(end) = math_str[pos..].find('}') {
                        let raw_label = &math_str[pos + 7..pos + end];
                        if eq_label.is_none() {
                            eq_label = Some(raw_label.replace(':', "-"));
                        }
                        math_str.replace_range(pos..pos + end + 1, "");
                    }
                let converted = convert_latex_math_to_typst(&math_str);
                let mut s = format!("\n$ {} $", converted.trim());
                if let Some(ref label) = eq_label {
                    s.push_str(&format!(" <{label}>"));
                }
                s.push_str("\n\n");
                if in_table {
                    current_cell.push_str(&s);
                } else {
                    typst.push_str(&s);
                }
            }
            Event::Rule => {
                if in_footnote_def {
                    i += 1;
                    continue;
                }
                if is_slides {
                    typst.push_str("\n#pagebreak()\n\n");
                } else {
                    typst.push_str("\n#line(length: 100%, stroke: 0.5pt + luma(180))\n\n");
                }
            }
            Event::SoftBreak => {
                if in_footnote_def {
                    i += 1;
                    continue;
                }
                if in_table {
                    current_cell.push(' ');
                } else {
                    typst.push('\n');
                }
            }
            Event::HardBreak => {
                if in_footnote_def {
                    i += 1;
                    continue;
                }
                if in_table {
                    current_cell.push(' ');
                } else {
                    typst.push_str("\\\n");
                }
            }
        }
        i += 1;
    }

    let mut rendered_first = false;
    while let Some(start_pos) = typst.find("BOTOX_REF_PLACEHOLDER_START:") {
        if let Some(end_pos) = typst[start_pos..].find(":BOTOX_REF_PLACEHOLDER_END") {
            let full_end = start_pos + end_pos + ":BOTOX_REF_PLACEHOLDER_END".len();
            if !rendered_first {
                let title_raw = &typst[start_pos + "BOTOX_REF_PLACEHOLDER_START:".len()..start_pos + end_pos];
                let custom_heading = if !title_raw.trim().is_empty() {
                    Some(title_raw.trim())
                } else {
                    None
                };
                let at_start_of_slide = typst[..start_pos].trim_end().ends_with("#pagebreak()");
                let ref_rendered = format_references_block(
                    custom_heading,
                    &references,
                    at_start_of_slide,
                    is_slides,
                    lang,
                    biblio_title,
                    true,
                );
                typst.replace_range(start_pos..full_end, &ref_rendered);
                rendered_first = true;
            } else {
                // Strip duplicate \ref placeholders so duplicate <bib-n> labels are not emitted
                typst.replace_range(start_pos..full_end, "");
            }
        } else {
            break;
        }
    }

    if !has_ref_command && bibliography && !references.is_empty() {
        let ref_rendered = format_references_block(
            None,
            &references,
            false,
            is_slides,
            lang,
            biblio_title,
            true,
        );
        typst.push_str(&ref_rendered);
    }

    typst
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sub_super_and_strikethrough() {
        let md = "H~2~O and 10^6^ with ~~strike~~ and `code_with_~_and_^`";
        let typst = markdown_to_typst(md, false, false, "en", None, None, None, None, None);
        assert!(typst.contains("#sub[2]"));
        assert!(typst.contains("#super[6]"));
        assert!(typst.contains("#strike[strike]"));
        assert!(typst.contains("`code_with_~_and_^`"));
    }

    #[test]
    fn test_typst_syntax_escaping() {
        let md = r#"
# Multiplicity 1..* and Comparisons: x < 5 and y > 2

Multiplicity is 1..* in text.
Contact user@domain.com or see issue #42 and C#.
Array index arr[0] and lone bracket ] alone.
Price is $100 and $200.
Code comment // not comment and /* unclosed
Path is C:\Users\Name and home ~/Desktop.
Variable foo_bar_baz.
/ Term: this is not a typst description list
Include <stdio.h> and break line<br>next line.
- [ ] Incomplete task
- [x] Done task
Check [link](https://example.com/api?q="quoted") here.
"#;
        let typst = markdown_to_typst(md, false, false, "en", None, None, None, None, None);
        assert!(typst.contains(r"1..\*"));
        assert!(typst.contains(r"x \< 5 and y \> 2"));
        assert!(typst.contains(r"user\@domain.com"));
        assert!(typst.contains(r"\#42"));
        assert!(typst.contains(r"C\#"));
        assert!(typst.contains(r"arr\[0\]"));
        assert!(typst.contains(r"\] alone"));
        assert!(typst.contains(r"\$100"));
        assert!(typst.contains(r"\$200"));
        assert!(typst.contains(r"\// not comment"));
        assert!(typst.contains(r"/\* unclosed"));
        assert!(typst.contains(r"C:\\Users\\Name"));
        assert!(typst.contains(r"\~/Desktop"));
        assert!(typst.contains(r"foo\_bar\_baz"));
        assert!(typst.contains(r"\/ Term:"));
        assert!(typst.contains(r"\<stdio.h\>"));
        assert!(typst.contains(r"\[ \]"));
        assert!(typst.contains(r"\[x\]"));
        assert!(typst.contains(r#"#link("https://example.com/api?q=\"quoted\"")"#));

        // Verify it compiles into PDF/JSON through Typst engine without error!
        let tmp_json = std::env::temp_dir().join("test_escaped_compilation.json");
        let res = crate::compiler::compile_typst(&typst, &tmp_json, None);
        assert!(res.is_ok(), "Typst compilation failed on escaped syntax: {:?}", res.err());
        let _ = std::fs::remove_file(tmp_json);
    }

    #[test]
    fn test_math_matrix_and_subscripts() {
        let latex = r"\begin{bmatrix} a_{11} & a_{12} \\ a_{21} & a_{22} \end{bmatrix}";
        let converted = convert_latex_math_to_typst(latex);
        assert!(converted.contains("mat(delim: \"[\""));
        assert!(converted.contains("a_(11)"));
        assert!(converted.contains("a_(12)"));
    }

    #[test]
    fn test_pagebreaks() {
        let md = "Before\n\n\\newpage\n\nMiddle\n\n\\columnbreak\n\nAfter";
        let typst = markdown_to_typst(md, false, false, "en", None, None, None, None, None);
        assert!(typst.contains("#pagebreak()"));
        assert!(typst.contains("#colbreak()"));
    }

    #[test]
    fn test_bibliography_link_transformation() {
        let md = "See [Rust](https://www.rust-lang.org) and [LLVM](https://llvm.org). Also [Rust Lang](https://www.rust-lang.org).";
        let typst = markdown_to_typst(md, false, true, "en", None, None, None, None, None);
        assert!(typst.contains("#link(<bib-1>)[\\"));
        assert!(typst.contains("#link(<bib-2>)[\\"));
        assert!(typst.contains("#heading(numbering: none)[References] <references>"));
        assert!(typst.contains("<bib-1>"));
        assert!(typst.contains("\\[1\\] \"Rust\", [Online]. Available: #link(\"https://www.rust-lang.org\")"));
        assert!(typst.contains("<bib-2>"));
        assert!(typst.contains("\\[2\\] \"LLVM\", [Online]. Available: #link(\"https://llvm.org\")"));
    }

    #[test]
    fn test_explicit_toc_spawning() {
        // Plain \toc
        let md1 = "# Chapter 1\n\n\\toc\n\n# Chapter 2";
        let typ1 = markdown_to_typst(md1, false, false, "en", None, None, Some(3), None, None);
        assert!(typ1.contains("#outline(depth: 3)"));

        // \tableofcontents
        let md2 = "# Chapter 1\n\n\\tableofcontents\n\n# Chapter 2";
        let typ2 = markdown_to_typst(md2, false, false, "en", None, None, Some(2), None, None);
        assert!(typ2.contains("#outline(depth: 2)"));

        // Custom titled TOC <!-- toc: Quick Navigation -->
        let md3 = "# Chapter 1\n\n<!-- toc: Quick Navigation -->\n\n# Chapter 2";
        let typ3 = markdown_to_typst(md3, false, false, "en", None, None, Some(4), None, None);
        assert!(typ3.contains("#outline(title: \"Quick Navigation\", depth: 4)"));

        // LaTeX syntax with custom title: \toc{Table des matières}
        let md4 = "# Chapter 1\n\n\\toc{Table des matières}\n\n# Chapter 2";
        let typ4 = markdown_to_typst(md4, false, false, "fr", None, None, Some(3), None, None);
        assert!(typ4.contains("#outline(title: \"Table des matières\", depth: 3)"));
    }

    #[test]
    fn test_remote_image_badge_caching_and_fallback() {
        let md = "Check build: [![Build](https://img.shields.io/badge/build-passing-brightgreen)](https://github.com)";
        let typst = markdown_to_typst(md, false, false, "en", None, None, None, None, None);
        assert!(typst.contains("#link(\"https://github.com\")"));
        assert!(typst.contains("#box(baseline: 20%, image(") || typst.contains("Build"));
    }

    #[test]
    fn test_explicit_ref_spawning_and_removal() {
        let md = "# Title\nCheck [Google](https://google.com)\n\n\\ref Custom Bibliography\n\nMore text and duplicate \\ref";
        let typst = markdown_to_typst(md, false, false, "en", None, None, None, None, None);
        assert!(typst.contains("#heading(numbering: none)[Custom Bibliography] <references>"));
        assert!(typst.contains("<bib-1>"));
        assert!(!typst.contains("BOTOX_REF_PLACEHOLDER"));
        assert!(!typst.contains("<bib-2>"));
    }

    #[test]
    fn test_file_referencing_for_diagrams_and_code() {
        let temp_dir = std::env::temp_dir().join(format!("botox_md_ref_test_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let sample_puml = temp_dir.join("sample.puml");
        std::fs::write(&sample_puml, "@startuml\nAlice -> Bob: Hello\n@enduml").unwrap();
        let sample_rs = temp_dir.join("sample.rs");
        std::fs::write(&sample_rs, "fn main() { println!(\"Hi\"); }").unwrap();

        let md = "```plantuml:sample.puml\n```\n\n```rust:sample.rs\n```\n\n![Architecture](sample.puml)";

        let opts = MarkdownOptions {
            resource_dir: Some(&temp_dir),
            ..Default::default()
        };
        let typst = markdown_to_typst_with_options(md, &opts);
        assert!(typst.contains("fn main() { println!(\\\"Hi\\\"); }") || typst.contains("fn main()"));
        assert!(
            typst.contains("#image(")
                || typst.contains("Diagram rendering failed")
                || typst.contains("Diagram rendering requires Kroki configuration")
        );

        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_nested_code_fence_with_fig_attribute() {
        let md = "````markdown\n```mermaid {caption=\"High-Level Architecture\" width=75% #fig:arch}\ngraph TD\n  A --> B\n```\n````";
        let typst = markdown_to_typst(md, false, false, "en", None, None, None, None, None);
        assert!(typst.starts_with("````markdown\n"));
        assert!(typst.ends_with("````\n\n"));

        let fm = serde_yaml::Value::Mapping(serde_yaml::Mapping::new());
        let config = crate::config::DocumentConfig::defaults();
        let typst_markup = crate::document::wrap_document(&typst, &fm, &config, None, None, None, None);
        let tmp_pdf = std::env::temp_dir().join("test_nested_fence.pdf");
        let res = crate::compiler::compile_typst(&typst_markup, &tmp_pdf, None);
        assert!(res.is_ok(), "Typst compile error: {:?}", res.err());
        let _ = std::fs::remove_file(tmp_pdf);
    }

    #[test]
    fn test_diagram_markdown_cached_and_fallback() {
        let diag_code = "graph LR\n  Client --> Server";
        let hash = crate::diagrams::compute_diagram_hash("mermaid", diag_code);
        let cache_dir = crate::diagrams::get_cache_dir();
        let _ = std::fs::create_dir_all(&cache_dir);
        let cache_file = cache_dir.join(format!("{hash}.svg"));
        std::fs::write(&cache_file, b"<svg><text>Mock Mermaid</text></svg>").unwrap();

        let md_cached = format!("```mermaid {{caption=\"Client-Server Architecture\" #fig:arch}}\n{diag_code}\n```");
        let typst_cached = markdown_to_typst(&md_cached, false, false, "en", None, None, None, None, None);
        assert!(typst_cached.contains("#figure(image("));
        assert!(typst_cached.contains("caption: [Client-Server Architecture]"));
        assert!(typst_cached.contains("<fig-arch>"));

        let md_with_newlines = "```mermaid {caption=\"Client-Server Architecture\" #fig:arch}\n\n  graph LR  \n\n    Client --> Server\n\n\n```";
        let typst_with_newlines = markdown_to_typst(md_with_newlines, false, false, "en", None, None, None, None, None);
        assert_eq!(typst_cached, typst_with_newlines);

        let _ = std::fs::remove_file(cache_file);

        let md_fallback = "```plantuml\nAlice -> Bob : Ping\n```";
        let typst_fallback = markdown_to_typst_with_options(
            md_fallback,
            &MarkdownOptions {
                kroki_url: Some("http://127.0.0.1:1"),
                ..Default::default()
            },
        );

        assert!(typst_fallback.contains("#botox_callout(\"warning\", \"Diagram rendering failed: Kroki unreachable\")"));
        assert!(typst_fallback.contains("```plantuml\nAlice -> Bob : Ping\n```"));

        // Unconfigured Kroki renders configuration guidance callout proposing https://kroki.io
        let md_unconfigured = "```plantuml\nAlice -> Bob : Hello\n```";
        let typst_unconfigured = markdown_to_typst(md_unconfigured, false, false, "en", None, None, None, None, None);
        assert!(typst_unconfigured.contains("#botox_callout(\"warning\", \"Diagram rendering requires Kroki configuration\")"));
        assert!(typst_unconfigured.contains("https://kroki.io"));
    }

    #[test]
    fn test_display_math_and_latex_superscript() {
        let md = "$$\\int_{-\\infty}^{+\\infty} e^{-x^2} \\, dx = \\sqrt{\\pi}$$\n\n$$\n\\sum_{k=0}^\\infty \\frac{1}{k!}\n$$";
        let typst = markdown_to_typst(md, false, false, "en", None, None, None, None, None);
        assert!(!typst.contains("#super"), "Math must not contain #super: {typst}");
        assert!(!typst.contains("#sub"), "Math must not contain #sub: {typst}");
        assert!(typst.contains("oo"), "Infinity should be oo: {typst}");
        assert!(typst.contains("integral_(-oo)^(+oo)"));
    }

    #[test]
    fn test_github_callouts() {
        let md = "> [!NOTE] Custom Note\n> Important info here\n\n> [!WARNING]\n> Danger ahead";
        let typst = markdown_to_typst(md, false, false, "en", None, None, None, None, None);
        assert!(typst.contains(r#"#botox_callout("note", "Custom Note")"#));
        assert!(typst.contains(r#"#botox_callout("warning", "")"#));
    }

    #[test]
    fn test_pandoc_callout_divs() {
        let md = "::: {.warning title=\"Critical Alert\"}\nSystem check required\n:::";
        let typst = markdown_to_typst(md, false, false, "en", None, None, None, None, None);
        assert!(typst.contains(r#"#botox_callout("warning", "Critical Alert")"#));
        assert!(typst.contains("System check required"));
    }

    #[test]
    fn test_nested_code_fences_pandoc_directives() {
        let md = "````markdown\n```rust\n// \\pause inside code\n// ::: note inside code\n```\n````";
        let preprocessed = preprocess_pandoc(md);
        assert!(!preprocessed.contains("<!--botox:pause-->"));
        assert!(!preprocessed.contains("<!--botox:callout:start"));
        assert!(preprocessed.contains("// \\pause inside code"));
        assert!(preprocessed.contains("// ::: note inside code"));
    }

    #[test]
    fn test_pandoc_heading_attributes() {
        let md = "# Introduction {#sec:intro .unnumbered}\n\n## Methodology {#sec:methods}";
        let typst = markdown_to_typst(md, false, false, "en", None, None, None, None, None);
        assert!(typst.contains("#heading(level: 1, numbering: none)[Introduction] <sec-intro>"));
        assert!(typst.contains("== Methodology <sec-methods>"));
    }

    #[test]
    fn test_pandoc_image_attributes() {
        let md = "![Diagram](arch.png){#fig:arch width=80% height=400px}";
        let typst = markdown_to_typst(md, false, false, "en", None, None, None, None, None);
        assert!(typst.contains("width: 80%"));
        assert!(typst.contains("height: 300pt"));
        assert!(typst.contains("<fig-arch>"));
    }

    #[test]
    fn test_pandoc_cross_references() {
        let md = "See @fig:arch and @sec:intro and @tbl:results and @eq:euler";
        let typst = markdown_to_typst(md, false, false, "en", None, None, None, None, None);
        assert!(typst.contains("@fig-arch"));
        assert!(typst.contains("@sec-intro"));
        assert!(typst.contains("@tbl-results"));
        assert!(typst.contains("@eq-euler"));
    }

    #[test]
    fn test_pandoc_display_math_label() {
        let md = "$$ e^{i \\pi} + 1 = 0 $$ {#eq:euler}";
        let typst = markdown_to_typst(md, false, false, "en", None, None, None, None, None);
        assert!(typst.contains("<eq-euler>"));
    }

    #[test]
    fn test_pandoc_table_caption_and_label() {
        let md = "Table: Experimental Results {#tbl:results}\n\n| Param | Value |\n|---|---|\n| A | 1 |";
        let typst = markdown_to_typst(md, false, false, "en", None, None, None, None, None);
        assert!(typst.contains("#figure("));
        assert!(typst.contains("caption: [Experimental Results]"));
        assert!(typst.contains("<tbl-results>"));
    }

    #[test]
    fn test_references_exclusion_and_inclusion() {
        let md = "See [GitHub](https://github.com/rust-lang/rust), [Twitter](https://x.com/rustlang), and [Rust Lang](https://rust-lang.org).\n\n\\ref";

        // Exclude github.com and x.com:
        let exc = vec!["github.com".to_string(), "x.com".to_string()];
        let typ = markdown_to_typst(md, false, false, "en", None, None, None, Some(&exc), None);
        // Only Rust Lang should be cited in bibliography as [1]
        assert!(typ.contains("#link(<bib-1>)["));
        assert!(!typ.contains("#link(<bib-2>)["));
        assert!(typ.contains("<bib-1>"));
        assert!(typ.contains("https://rust-lang.org"));
        assert!(!typ.contains("<bib-2>"));
        // GitHub and Twitter should be normal inline links without citation markers
        assert!(typ.contains("#link(\"https://github.com/rust-lang/rust\")[GitHub]"));
        assert!(typ.contains("#link(\"https://x.com/rustlang\")[Twitter]"));

        // Inclusion whitelist: only rust-lang.org
        let inc = vec!["rust-lang.org".to_string()];
        let typ_inc = markdown_to_typst(md, false, false, "en", None, None, None, None, Some(&inc));
        assert!(typ_inc.contains("#link(<bib-1>)["));
        assert!(!typ_inc.contains("<bib-2>"));
        assert!(typ_inc.contains("https://rust-lang.org"));
    }

    #[test]
    fn test_today_replacement() {
        let md = "Date of publication: \\today in document.";
        let typst = markdown_to_typst(md, false, false, "en", None, None, None, None, None);
        assert!(typst.contains("#datetime.today().display("));
    }

    #[test]
    fn test_url_matches_pattern() {
        assert!(url_matches_pattern("https://github.com/rust-lang/rust", "github.com"));
        assert!(url_matches_pattern("https://www.github.com/rust-lang/rust", "github.com"));
        assert!(url_matches_pattern("https://github.com", "*github.com*"));
        assert!(url_matches_pattern("https://x.com/profile", "x.com"));
        // Avoid false positive on substring in host:
        assert!(!url_matches_pattern("https://latex.com/page", "x.com"));
        // Glob matching:
        assert!(url_matches_pattern("https://en.wikipedia.org/wiki/Rust", "*.org"));
        assert!(url_matches_pattern("https://my-company.com/internal/docs", "my-company.com/*"));
        assert!(!url_matches_pattern("https://other.com/internal/docs", "my-company.com/*"));
    }

    #[test]
    fn test_pause_extraction() {
        let md = "# Slide 1\nPoint 1\n\n\\pause\n\nPoint 2";
        let typst = markdown_to_typst(md, true, false, "en", None, None, None, None, None);
        assert!(typst.contains("#botox_pause()"));
    }
}
