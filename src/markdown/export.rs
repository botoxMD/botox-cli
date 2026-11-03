use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use pulldown_cmark::{Event, Parser, Tag, TagEnd};

use super::pandoc::{
    parse_heading_attributes, parse_image_attributes, parse_ref_line,
    parse_toc_line,
};
use super::references::url_matches_pattern;
use super::tables::parse_caption_and_label;
use crate::diagrams::{
    parse_code_fence_file, parse_diagram_fence, render_diagram, resolve_file_content,
};

pub fn extract_frontmatter_raw(content: &str) -> (&str, &str) {
    let trimmed_start = content.trim_start();
    if !trimmed_start.starts_with("---") {
        return ("", content);
    }

    let first_line_end = match trimmed_start.find('\n') {
        Some(idx) => idx,
        None => return ("", content),
    };

    let first_line = trimmed_start[..first_line_end].trim();
    if first_line != "---" {
        return ("", content);
    }

    let rest = &trimmed_start[first_line_end + 1..];
    let mut current_idx = 0;
    while current_idx < rest.len() {
        let line_end = rest[current_idx..].find('\n').map(|i| current_idx + i).unwrap_or(rest.len());
        let line = rest[current_idx..line_end].trim();
        if line == "---" || line == "..." {
            let fm = &rest[..current_idx];
            let body = if line_end < rest.len() {
                &rest[line_end + 1..]
            } else {
                ""
            };
            return (fm, body);
        }
        current_idx = if line_end < rest.len() { line_end + 1 } else { rest.len() };
    }

    ("", content)
}

#[derive(Debug, Clone, Default)]
pub struct MarkdownExportOptions<'a> {
    pub output_path: Option<&'a Path>,
    pub assets_dir: Option<&'a Path>,
    pub resource_dir: Option<&'a Path>,
    pub kroki_url: Option<&'a str>,
    pub bibliography: bool,
    pub lang: &'a str,
    pub biblio_title: Option<&'a str>,
    pub toc_title: Option<&'a str>,
    pub toc_depth: Option<usize>,
    pub ref_exclude: Option<&'a [String]>,
    pub ref_include: Option<&'a [String]>,
    pub generate_toc: bool,
    pub number_sections: bool,
}

#[derive(Debug, Clone)]
pub struct MarkdownExportResult {
    pub content: String,
    pub exported_assets: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
enum TargetKind {
    Section,
    Figure,
    Table,
    Equation,
}

#[derive(Debug, Clone)]
struct TargetInfo {
    kind: TargetKind,
    id: String,
    number: String,
    title: String,
}

fn slugify(s: &str) -> String {
    let mut slug = String::new();
    let mut prev_dash = false;
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash {
            slug.push('-');
            prev_dash = true;
        }
    }
    slug.trim_matches('-').to_string()
}

fn format_today_date(lang: &str) -> String {
    let now = chrono::Local::now();
    match lang {
        "fr" => now.format("%d %B %Y").to_string(),
        "de" => now.format("%d. %B %Y").to_string(),
        "es" => now.format("%d de %B de %Y").to_string(),
        "it" => now.format("%d %B %Y").to_string(),
        _ => now.format("%B %-d, %Y").to_string(),
    }
}

pub fn is_references_heading_title(title: &str, biblio_title: Option<&str>) -> bool {
    let lower = title.trim().to_lowercase();
    if let Some(bt) = biblio_title {
        if lower == bt.trim().to_lowercase() {
            return true;
        }
    }
    matches!(
        lower.as_str(),
        "references"
            | "reference"
            | "bibliography"
            | "works cited"
            | "sources"
            | "références"
            | "literaturverzeichnis"
            | "referencias"
            | "riferimenti bibliografici"
    )
}

pub fn detect_list_indent(lines: &[&str], current_line_idx: usize) -> Option<&'static str> {
    let mut k = current_line_idx;
    while k > 0 {
        k -= 1;
        let prev = lines[k].trim();
        if prev.is_empty() {
            continue;
        }
        let trimmed_start = lines[k].trim_start();
        if let Some(rest) = trimmed_start.strip_prefix(|c: char| c.is_ascii_digit()) {
            let after_digits = rest.trim_start_matches(|c: char| c.is_ascii_digit());
            if after_digits.starts_with(". ") || after_digits.starts_with(") ") {
                return Some("   ");
            }
        }
        if trimmed_start.starts_with("* ")
            || trimmed_start.starts_with("- ")
            || trimmed_start.starts_with("+ ")
        {
            return Some("  ");
        }
        break;
    }
    None
}

pub fn filter_frontmatter_for_markdown(raw_fm: &str) -> Option<String> {
    let val: serde_yaml::Value = serde_yaml::from_str(raw_fm).ok()?;
    let map = val.as_mapping()?;
    let mut new_map = serde_yaml::Mapping::new();

    let internal_keys = [
        "theme", "slides_theme", "slides-theme",
        "papersize", "paper", "pagesize", "page-size",
        "fontsize", "font-size", "font",
        "margin", "margins", "columns", "gutter",
        "linestretch", "line-spacing", "line_spacing",
        "pagebreak-before", "pagebreak_before",
        "kroki_url", "kroki-url", "kroki_endpoint", "kroki-endpoint",
        "toc", "toc-depth", "toc_depth", "toc-title", "toc_title",
        "number-sections", "number_sections",
        "bibliography", "bibliography-title", "biblio-title", "biblio_title", "references-title", "references_title",
        "links-as-references", "links_as_references", "cite-links", "cite_links",
        "exclude-references", "exclude_references", "exclude-links", "exclude_links",
        "include-references", "include_references", "include-links", "include_links",
        "marp", "slide", "slides", "presentation",
        "_class",
    ];

    for (k, v) in map {
        if let Some(key_str) = k.as_str() {
            let key_norm = key_str.to_ascii_lowercase().replace('_', "-");
            let is_internal = internal_keys.iter().any(|ik| {
                let ik_norm = ik.replace('_', "-");
                key_norm == ik_norm
            });
            if !is_internal {
                new_map.insert(k.clone(), v.clone());
            }
        } else {
            new_map.insert(k.clone(), v.clone());
        }
    }

    if new_map.is_empty() {
        None
    } else {
        let serialized = serde_yaml::to_string(&serde_yaml::Value::Mapping(new_map)).ok()?;
        Some(serialized.trim().to_string())
    }
}

pub fn normalize_markdown_blank_lines(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let mut in_fence = false;
    let mut fence_char = '`';
    let mut fence_len = 0;
    let mut consecutive_empty = 0;

    for line in content.lines() {
        let trimmed_end = line.trim_end();
        let trimmed_start = trimmed_end.trim_start();

        if !in_fence {
            let backticks = trimmed_start.chars().take_while(|&c| c == '`').count();
            let tildes = trimmed_start.chars().take_while(|&c| c == '~').count();
            if backticks >= 3 || tildes >= 3 {
                in_fence = true;
                fence_char = if backticks >= 3 { '`' } else { '~' };
                fence_len = backticks.max(tildes);
                consecutive_empty = 0;
                result.push_str(trimmed_end);
                result.push('\n');
                continue;
            }

            if trimmed_start.is_empty() {
                consecutive_empty += 1;
                if consecutive_empty <= 1 && !result.is_empty() {
                    result.push('\n');
                }
            } else {
                consecutive_empty = 0;
                result.push_str(trimmed_end);
                result.push('\n');
            }
        } else {
            let count = trimmed_start.chars().take_while(|&c| c == fence_char).count();
            if count >= fence_len && trimmed_start[count..].trim().is_empty() {
                in_fence = false;
            }
            result.push_str(line);
            result.push('\n');
        }
    }

    let trimmed = result.trim_end();
    if trimmed.is_empty() {
        String::new()
    } else {
        format!("{trimmed}\n")
    }
}

fn parse_display_math_block(
    lines: &[&str],
    start_idx: usize,
) -> (String, Option<String>, usize) {
    let first_line = lines[start_idx].trim();
    let mut math_code = String::new();
    let mut eq_id = None;
    let mut end_idx = start_idx;

    // Case A: Single line "$$ formula $$ {#eq:label}"
    if first_line.len() > 2 && first_line[2..].contains("$$") {
        let after_opening = &first_line[2..];
        let closing_pos = after_opening.find("$$").unwrap();
        let formula = after_opening[..closing_pos].trim();
        math_code.push_str(formula);

        let after_closing = after_opening[closing_pos + 2..].trim();
        if after_closing.starts_with("{#") {
            let end_brace = after_closing.find('}').unwrap_or(after_closing.len());
            let raw = &after_closing[1..end_brace].trim();
            eq_id = Some(raw.strip_prefix('#').unwrap_or(raw).to_string());
        }
        end_idx = start_idx;
    } else {
        // Case B: Multi-line "$$ \n ... \n $$ {#eq:label}"
        let first_body = first_line[2..].trim();
        if !first_body.is_empty() {
            math_code.push_str(first_body);
            math_code.push('\n');
        }

        let mut j = start_idx + 1;
        while j < lines.len() {
            let cur_line = lines[j];
            let cur_trimmed = cur_line.trim();

            if cur_trimmed.starts_with("$$") {
                let after_closing = cur_trimmed[2..].trim();
                if after_closing.starts_with("{#") {
                    let end_brace = after_closing.find('}').unwrap_or(after_closing.len());
                    let raw = &after_closing[1..end_brace].trim();
                    eq_id = Some(raw.strip_prefix('#').unwrap_or(raw).to_string());
                } else if after_closing.is_empty() && j + 1 < lines.len() {
                    let next_line = lines[j + 1].trim();
                    if next_line.starts_with("{#eq:") || next_line.starts_with("{#") {
                        let end_brace = next_line.find('}').unwrap_or(next_line.len());
                        let raw = &next_line[1..end_brace].trim();
                        eq_id = Some(raw.strip_prefix('#').unwrap_or(raw).to_string());
                        j += 1;
                    }
                }
                end_idx = j;
                break;
            } else if cur_trimmed.ends_with("$$") {
                let formula_part = cur_trimmed[..cur_trimmed.len() - 2].trim();
                if !formula_part.is_empty() {
                    math_code.push_str(formula_part);
                }
                end_idx = j;
                break;
            } else {
                math_code.push_str(cur_line);
                math_code.push('\n');
                j += 1;
                end_idx = j;
            }
        }
    }

    if eq_id.is_none() {
        if let Some(pos) = math_code.find(r"\label{") {
            if let Some(end) = math_code[pos..].find('}') {
                eq_id = Some(math_code[pos + 7..pos + end].to_string());
            }
        }
    }

    (math_code.trim().to_string(), eq_id, end_idx)
}

pub fn export_to_normal_markdown(
    raw_content: &str,
    opts: &MarkdownExportOptions<'_>,
) -> Result<MarkdownExportResult, String> {
    let (fm_raw, body_md) = extract_frontmatter_raw(raw_content);

    let fm: serde_yaml::Value = serde_yaml::from_str(&fm_raw).unwrap_or(serde_yaml::Value::Null);

    let lang = fm.get("lang")
        .and_then(|v| v.as_str())
        .unwrap_or(if opts.lang.is_empty() { "en" } else { opts.lang });

    let biblio_title = fm.get("biblio-title")
        .or_else(|| fm.get("bibliography-title"))
        .or_else(|| fm.get("references-title"))
        .and_then(|v| v.as_str())
        .or(opts.biblio_title);

    let toc_title = fm.get("toc-title")
        .or_else(|| fm.get("toc_title"))
        .and_then(|v| v.as_str())
        .or(opts.toc_title)
        .unwrap_or(match lang {
            "fr" => "Table des matières",
            "de" => "Inhaltsverzeichnis",
            "es" => "Tabla de contenidos",
            "it" => "Indice dei contenuti",
            _ => "Table of Contents",
        });

    let toc_depth = fm.get("toc-depth")
        .or_else(|| fm.get("toc_depth"))
        .and_then(|v| v.as_u64())
        .map(|n| n as usize)
        .or(opts.toc_depth)
        .unwrap_or(3);

    let number_sections = if opts.number_sections {
        true
    } else if let Some(v) = fm.get("number-sections").or_else(|| fm.get("number_sections")) {
        match v {
            serde_yaml::Value::Bool(b) => *b,
            serde_yaml::Value::String(s) => s == "true" || s == "yes" || s == "1",
            _ => false,
        }
    } else {
        false
    };

    let kroki_url = fm.get("kroki-url")
        .or_else(|| fm.get("kroki_url"))
        .or_else(|| fm.get("kroki-endpoint"))
        .or_else(|| fm.get("kroki_endpoint"))
        .and_then(|v| v.as_str())
        .or(opts.kroki_url);

    let should_enable_bib = if opts.bibliography {
        true
    } else if let Some(fm_bib) = fm.get("bibliography")
        .or_else(|| fm.get("links-as-references"))
        .or_else(|| fm.get("cite-links"))
    {
        match fm_bib {
            serde_yaml::Value::Bool(b) => *b,
            serde_yaml::Value::String(s) => !s.is_empty() && s != "false" && s != "no" && s != "off",
            _ => false,
        }
    } else {
        false
    };

    // Determine target assets directory
    let (target_assets_dir, relative_assets_prefix) = if let Some(ad) = opts.assets_dir {
        let rel_prefix = if ad.is_absolute() {
            if let Some(out_p) = opts.output_path {
                let out_parent = if out_p.as_os_str() == "-" {
                    Path::new(".")
                } else {
                    out_p.parent().unwrap_or(Path::new("."))
                };
                if let (Ok(canon_ad), Ok(canon_out)) = (ad.canonicalize(), out_parent.canonicalize()) {
                    if let Ok(rel) = canon_ad.strip_prefix(&canon_out) {
                        rel.to_string_lossy().to_string()
                    } else {
                        ad.to_string_lossy().to_string()
                    }
                } else if let Ok(rel) = ad.strip_prefix(out_parent) {
                    rel.to_string_lossy().to_string()
                } else {
                    ad.to_string_lossy().to_string()
                }
            } else {
                ad.to_string_lossy().to_string()
            }
        } else {
            ad.to_string_lossy().to_string()
        };
        (ad.to_path_buf(), rel_prefix)
    } else if let Some(out_p) = opts.output_path {
        if out_p.as_os_str() == "-" {
            (PathBuf::from("assets"), "assets".to_string())
        } else {
            let parent = out_p.parent().unwrap_or(Path::new("."));
            let dir = parent.join("assets");
            (dir, "assets".to_string())
        }
    } else {
        (PathBuf::from("assets"), "assets".to_string())
    };

    // --- PASS 1: Index headings, tables, equations, diagrams, and citations ---
    let mut labels: HashMap<String, TargetInfo> = HashMap::new();
    let mut headings_list: Vec<(usize, String, String, String)> = Vec::new(); // (level, title, id, number)
    let mut section_counters = vec![0usize; 6];
    let mut fig_counter = 0usize;
    let mut tbl_counter = 0usize;
    let mut eq_counter = 0usize;

    // Scan lines for headings, tables, equations, diagrams
    let lines: Vec<&str> = body_md.lines().collect();
    let mut in_fence = false;
    let mut fence_char = '`';
    let mut fence_len = 0;

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim_start();

        if !in_fence {
            let backticks = trimmed.chars().take_while(|&c| c == '`').count();
            let tildes = trimmed.chars().take_while(|&c| c == '~').count();
            if backticks >= 3 || tildes >= 3 {
                in_fence = true;
                fence_char = if backticks >= 3 { '`' } else { '~' };
                fence_len = backticks.max(tildes);

                // Check diagram fence
                let fence_str = trimmed[fence_len..].trim();
                if let Some(spec) = parse_diagram_fence(fence_str) {
                    fig_counter += 1;
                    let id = spec.id.clone().unwrap_or_else(|| format!("fig-{fig_counter}"));
                    let clean_id = id.replace(':', "-");
                    let title = spec.caption.clone().unwrap_or_else(|| format!("Figure {fig_counter}"));
                    let info = TargetInfo {
                        kind: TargetKind::Figure,
                        id: clean_id.clone(),
                        number: format!("Figure {fig_counter}"),
                        title: title.clone(),
                    };
                    labels.insert(clean_id.clone(), info.clone());
                    if let Some(ref raw_id) = spec.id {
                        labels.insert(raw_id.clone(), info);
                    }
                }
                i += 1;
                continue;
            }
        } else {
            let count = trimmed.chars().take_while(|&c| c == fence_char).count();
            if count >= fence_len && trimmed[count..].trim().is_empty() {
                in_fence = false;
            }
            i += 1;
            continue;
        }

        // Headings
        if trimmed.starts_with('#') {
            let hashes = trimmed.chars().take_while(|&c| c == '#').count();
            if hashes <= 6 && trimmed.chars().nth(hashes).map(|c| c.is_whitespace()).unwrap_or(false) {
                let h_text = trimmed[hashes..].trim();
                let (title, mut is_unnumbered, id_opt) = parse_heading_attributes(h_text);
                if is_references_heading_title(&title, biblio_title) {
                    is_unnumbered = true;
                }
                let id = id_opt.unwrap_or_else(|| slugify(&title));
                let clean_id = id.replace(':', "-");

                let number_str = if number_sections && !is_unnumbered {
                    section_counters[hashes - 1] += 1;
                    for k in hashes..6 {
                        section_counters[k] = 0;
                    }
                    let parts: Vec<String> = section_counters[..hashes]
                        .iter()
                        .map(|n| n.to_string())
                        .collect();
                    parts.join(".")
                } else {
                    String::new()
                };

                let info = TargetInfo {
                    kind: TargetKind::Section,
                    id: clean_id.clone(),
                    number: number_str.clone(),
                    title: title.clone(),
                };
                labels.insert(clean_id.clone(), info.clone());
                if !id.is_empty() {
                    labels.insert(id.clone(), info);
                }

                headings_list.push((hashes, title, clean_id, number_str));
                i += 1;
                continue;
            }
        }

        // Table with caption: Table: ... {#tbl:targets}
        if trimmed.starts_with("Table:") || (trimmed.starts_with(':') && !trimmed.starts_with(":::")) {
            let (caption, id_opt) = parse_caption_and_label(trimmed);
            tbl_counter += 1;
            let id = id_opt.unwrap_or_else(|| format!("tbl-{tbl_counter}"));
            let clean_id = id.replace(':', "-");
            let info = TargetInfo {
                kind: TargetKind::Table,
                id: clean_id.clone(),
                number: format!("Table {tbl_counter}"),
                title: caption,
            };
            labels.insert(clean_id.clone(), info.clone());
            labels.insert(id, info);
            i += 1;
            continue;
        }

        // Display math with label: $$ ... $$ {#eq:gaussian} or \label{eq:gaussian}
        if trimmed.starts_with("$$") {
            let (_math_code, eq_id, end_j) = parse_display_math_block(&lines, i);
            if let Some(eid) = eq_id {
                eq_counter += 1;
                let clean_id = eid.replace(':', "-");
                let info = TargetInfo {
                    kind: TargetKind::Equation,
                    id: clean_id.clone(),
                    number: format!("({eq_counter})"),
                    title: String::new(),
                };
                labels.insert(clean_id.clone(), info.clone());
                labels.insert(eid, info);
            }
            i = end_j + 1;
            continue;
        }

        // Image with diagram extension: ![caption](arch.uxf){#fig:umlet-img}
        if trimmed.starts_with("![") && trimmed.contains('(') {
            if let Some(open_paren) = trimmed.find('(') {
                if let Some(close_paren) = trimmed[open_paren..].find(')') {
                    let url = &trimmed[open_paren + 1..open_paren + close_paren];
                    let norm = url.trim().to_lowercase();
                    if norm.ends_with(".uxf")
                        || norm.ends_with(".umlet")
                        || norm.ends_with(".puml")
                        || norm.ends_with(".plantuml")
                        || norm.ends_with(".mmd")
                        || norm.ends_with(".mermaid")
                        || norm.ends_with(".dot")
                    {
                        fig_counter += 1;
                        let mut fig_id = None;
                        let after_paren = &trimmed[open_paren + close_paren + 1..].trim();
                        if after_paren.starts_with('{') {
                            let (_, _, id_opt) = parse_image_attributes(&after_paren[1..after_paren.len().saturating_sub(1)]);
                            fig_id = id_opt;
                        }
                        let id = fig_id.unwrap_or_else(|| format!("fig-{fig_counter}"));
                        let clean_id = id.replace(':', "-");
                        let alt = trimmed[2..open_paren].trim_end_matches(']').trim();
                        let title = if alt.is_empty() { format!("Figure {fig_counter}") } else { alt.to_string() };
                        let info = TargetInfo {
                            kind: TargetKind::Figure,
                            id: clean_id.clone(),
                            number: format!("Figure {fig_counter}"),
                            title,
                        };
                        labels.insert(clean_id.clone(), info.clone());
                        labels.insert(id, info);
                    }
                }
            }
        }

        i += 1;
    }

    // --- Bibliography URL collection ---
    let mut references: Vec<(String, String)> = Vec::new();
    let has_ref_command = body_md.lines().any(|l| parse_ref_line(l).is_some());
    let active_bibliography = should_enable_bib || has_ref_command;

    if active_bibliography {
        // Collect links using pulldown-cmark
        let parser = Parser::new(body_md);
        let mut cur_link_url: Option<String> = None;
        let mut cur_link_text = String::new();

        for event in parser {
            match event {
                Event::Start(Tag::Link { dest_url, .. }) => {
                    cur_link_url = Some(dest_url.to_string());
                    cur_link_text.clear();
                }
                Event::Text(t) => {
                    if cur_link_url.is_some() {
                        cur_link_text.push_str(&t);
                    }
                }
                Event::Code(c) => {
                    if cur_link_url.is_some() {
                        cur_link_text.push('`');
                        cur_link_text.push_str(&c);
                        cur_link_text.push('`');
                    }
                }
                Event::End(TagEnd::Link) => {
                    if let Some(url) = cur_link_url.take() {
                        let trimmed_url = url.trim();
                        if (trimmed_url.starts_with("http://") || trimmed_url.starts_with("https://"))
                            && !trimmed_url.starts_with('#')
                        {
                            let mut excluded = false;
                            if let Some(exc_patterns) = opts.ref_exclude {
                                for pat in exc_patterns {
                                    if url_matches_pattern(trimmed_url, pat) {
                                        excluded = true;
                                        break;
                                    }
                                }
                            }
                            let mut included = true;
                            if let Some(inc_patterns) = opts.ref_include {
                                included = false;
                                for pat in inc_patterns {
                                    if url_matches_pattern(trimmed_url, pat) {
                                        included = true;
                                        break;
                                    }
                                }
                            }
                            if !excluded && included {
                                let trimmed_text = cur_link_text.trim();
                                let is_badge = trimmed_text.is_empty() || trimmed_text.starts_with("![");
                                let is_numeric = trimmed_text.starts_with('[') && trimmed_text.ends_with(']');
                                if !is_badge && !is_numeric {
                                    if !references.iter().any(|(u, _)| u == trimmed_url) {
                                        references.push((trimmed_url.to_string(), trimmed_text.to_string()));
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // --- PASS 2: Line by line transformation ---
    let mut out = String::with_capacity(body_md.len() * 2);
    let mut exported_assets = Vec::new();
    let mut in_callout = false;
    let mut cur_fig_num = 0usize;
    let mut cur_tbl_num = 0usize;
    let mut cur_eq_num = 0usize;
    let mut has_rendered_toc = false;
    let mut has_rendered_ref = false;

    // Output frontmatter if present (cleaned of internal compiler keys)
    if !fm_raw.trim().is_empty() {
        if let Some(clean_fm) = filter_frontmatter_for_markdown(fm_raw) {
            out.push_str("---\n");
            out.push_str(&clean_fm);
            out.push_str("\n---\n\n");
        }
    }

    // If explicit --toc was requested and no \toc in body, prepend TOC
    let doc_has_toc_marker = body_md.lines().any(|l| parse_toc_line(l).is_some());
    if (opts.generate_toc || fm.get("toc").and_then(|v| v.as_bool()).unwrap_or(false)) && !doc_has_toc_marker {
        out.push_str(&render_toc(&headings_list, toc_title, toc_depth));
        out.push_str("\n\n");
        has_rendered_toc = true;
    }

    in_fence = false;
    fence_char = '`';
    fence_len = 0;

    let mut line_idx = 0;
    while line_idx < lines.len() {
        let line = lines[line_idx];
        let trimmed = line.trim_start();

        // 1. Check code fences
        if !in_fence {
            let backticks = trimmed.chars().take_while(|&c| c == '`').count();
            let tildes = trimmed.chars().take_while(|&c| c == '~').count();
            if backticks >= 3 || tildes >= 3 {
                in_fence = true;
                fence_char = if backticks >= 3 { '`' } else { '~' };
                fence_len = backticks.max(tildes);

                let fence_str = trimmed[fence_len..].trim();

                // A. Check diagram fence first
                if let Some(spec) = parse_diagram_fence(fence_str) {
                    let fence_start_idx = line_idx;
                    cur_fig_num += 1;
                    // Gather diagram source code
                    let mut diag_code = String::new();
                    let mut j = line_idx + 1;
                    while j < lines.len() {
                        let l = lines[j];
                        let l_trimmed = l.trim_start();
                        let count = l_trimmed.chars().take_while(|&c| c == fence_char).count();
                        if count >= fence_len && l_trimmed[count..].trim().is_empty() {
                            break;
                        }
                        diag_code.push_str(l);
                        diag_code.push('\n');
                        j += 1;
                    }
                    line_idx = j; // pointing to closing fence
                    in_fence = false;

                    // Resolve external diagram file if any
                    let mut file_to_load = spec.file.clone();
                    if file_to_load.is_none() {
                        let trimmed_diag = diag_code.trim();
                        if let Some(rest) = trimmed_diag.strip_prefix("!include ") {
                            file_to_load = Some(rest.trim().to_string());
                        } else if let Some(rest) = trimmed_diag.strip_prefix("file: ") {
                            file_to_load = Some(rest.trim().to_string());
                        } else if !trimmed_diag.contains('\n')
                            && (trimmed_diag.ends_with(".uxf")
                                || trimmed_diag.ends_with(".umlet")
                                || trimmed_diag.ends_with(".puml")
                                || trimmed_diag.ends_with(".plantuml")
                                || trimmed_diag.ends_with(".mmd")
                                || trimmed_diag.ends_with(".mermaid")
                                || trimmed_diag.ends_with(".dot"))
                        {
                            file_to_load = Some(trimmed_diag.to_string());
                        }
                    }

                    if let Some(ref cf) = file_to_load {
                        if let Some(loaded) = resolve_file_content(cf, opts.resource_dir) {
                            diag_code = loaded;
                        }
                    }

                    // Render diagram via Kroki / cache
                    let id_str = spec.id.clone().unwrap_or_else(|| format!("fig-{cur_fig_num}"));
                    let clean_id = id_str.replace(':', "-");
                    let asset_filename = format!("{clean_id}.svg");

                    let mut export_success = false;
                    if let Ok(cached_svg_path) = render_diagram(&spec.diagram_type, &diag_code, kroki_url) {
                        let _ = fs::create_dir_all(&target_assets_dir);
                        let target_asset_path = target_assets_dir.join(&asset_filename);
                        if let Ok(_) = fs::copy(&cached_svg_path, &target_asset_path) {
                            exported_assets.push(target_asset_path);
                            export_success = true;
                        }
                    }

                    // Emit Markdown
                    let indent = detect_list_indent(&lines, fence_start_idx).unwrap_or("");
                    let alt_text = spec.caption.as_deref().unwrap_or("Diagram");

                    if export_success {
                        let rel_path = format!("{relative_assets_prefix}/{asset_filename}");
                        let width_attr = if let Some(ref w) = spec.width {
                            format!(" width=\"{w}\"")
                        } else {
                            String::new()
                        };
                        out.push_str(&format!("{indent}<p align=\"center\" id=\"{clean_id}\">\n"));
                        out.push_str(&format!("{indent}  <img src=\"{rel_path}\" alt=\"{alt_text}\"{width_attr} style=\"border-radius: 8px;\" />"));
                        if let Some(ref cap) = spec.caption {
                            out.push_str(&format!("<br>\n{indent}  <em>Figure {cur_fig_num}: {cap}</em>\n"));
                        } else {
                            out.push('\n');
                        }
                        out.push_str(&format!("{indent}</p>\n\n"));
                    } else {
                        // Fallback: If Mermaid, output native GitHub ```mermaid block!
                        let fence_prefix = if indent.is_empty() { String::new() } else { indent.to_string() };
                        let indented_code: String = diag_code.lines()
                            .map(|l| if l.is_empty() { String::new() } else { format!("{fence_prefix}{l}") })
                            .collect::<Vec<_>>()
                            .join("\n");
                        let diag_type = &spec.diagram_type;
                        out.push_str(&format!("{fence_prefix}```{diag_type}\n{indented_code}\n{fence_prefix}```\n\n"));
                        if let Some(ref cap) = spec.caption {
                            out.push_str(&format!("{indent}<p align=\"center\" id=\"{clean_id}\"><em>Figure {cur_fig_num}: {cap}</em></p>\n\n"));
                        }
                    }

                    line_idx += 1;
                    continue;
                }

                // B. Check external code referencing: ```lang file="path.rs"
                let (lang_clean, code_file) = parse_code_fence_file(fence_str);
                if let Some(cf) = code_file {
                    let mut file_content = resolve_file_content(&cf, opts.resource_dir)
                        .unwrap_or_else(|| format!("// Could not load referenced file: {cf}\n"));
                    if !file_content.ends_with('\n') {
                        file_content.push('\n');
                    }
                    out.push_str(&format!("```{lang_clean}\n{file_content}```\n\n"));

                    // Fast forward past fence contents if any
                    line_idx += 1;
                    while line_idx < lines.len() {
                        let l = lines[line_idx].trim_start();
                        let count = l.chars().take_while(|&c| c == fence_char).count();
                        if count >= fence_len && l[count..].trim().is_empty() {
                            in_fence = false;
                            break;
                        }
                        line_idx += 1;
                    }
                    line_idx += 1;
                    continue;
                }

                // Regular code block: write unchanged
                out.push_str(line);
                out.push('\n');
                line_idx += 1;
                continue;
            }
        } else {
            let count = trimmed.chars().take_while(|&c| c == fence_char).count();
            if count >= fence_len && trimmed[count..].trim().is_empty() {
                in_fence = false;
            }
            out.push_str(line);
            out.push('\n');
            line_idx += 1;
            continue;
        }

        // 2. Pandoc ::: callouts & orientation divs
        if trimmed.starts_with(":::") {
            let rest = trimmed.trim_start_matches(':').trim();
            let clean = rest.trim_start_matches('{').trim_end_matches('}').trim();
            let lower = clean.to_ascii_lowercase();
            let lower_ident = lower.strip_prefix('.').unwrap_or(&lower).trim();

            if lower_ident == "landscape" || lower_ident == "horizontal" || lower_ident == "portrait" || lower_ident == "vertical" {
                out.push_str(line);
                out.push('\n');
                line_idx += 1;
                continue;
            }

            if rest.is_empty() {
                if in_callout {
                    in_callout = false;
                    out.push('\n');
                } else {
                    out.push_str(":::\n\n");
                }
            } else {
                in_callout = true;
                let (kind, title) = super::callouts::parse_callout_header(rest);
                let gh_kind = match kind.to_ascii_lowercase().as_str() {
                    "note" | "info" => "NOTE",
                    "tip" | "hint" => "TIP",
                    "warning" => "WARNING",
                    "caution" | "danger" => "CAUTION",
                    "important" => "IMPORTANT",
                    _ => "NOTE",
                };
                out.push_str(&format!("> [!{gh_kind}]\n"));
                if !title.is_empty() {
                    out.push_str(&format!("> **{title}**\n>\n"));
                }
            }
            line_idx += 1;
            continue;
        }

        // Inside ::: callout
        if in_callout {
            out.push_str(&format!("> {line}\n"));
            line_idx += 1;
            continue;
        }

        // 3. Headings with Pandoc attributes: # Heading {#sec:intro}
        if trimmed.starts_with('#') {
            let hashes = trimmed.chars().take_while(|&c| c == '#').count();
            if hashes <= 6 && trimmed.chars().nth(hashes).map(|c| c.is_whitespace()).unwrap_or(false) {
                let h_text = trimmed[hashes..].trim();
                let (title, _is_unnumbered, id_opt) = parse_heading_attributes(h_text);
                let clean_id = id_opt.clone().unwrap_or_else(|| slugify(&title)).replace(':', "-");

                let hashes_str = "#".repeat(hashes);

                let target_info = labels.get(&clean_id);
                let num_prefix = if let Some(t) = target_info
                    && !t.number.is_empty() {
                        format!("{} ", t.number)
                    } else {
                        String::new()
                    };

                let anchor_suffix = if id_opt.is_some() {
                    format!(" <a id=\"{clean_id}\"></a>")
                } else {
                    String::new()
                };

                out.push_str(&format!("{hashes_str} {num_prefix}{title}{anchor_suffix}\n\n"));
                line_idx += 1;
                continue;
            }
        }

        // 4. Tables with Pandoc captions: Table: ... {#tbl:targets}
        if trimmed.starts_with("Table:") || (trimmed.starts_with(':') && !trimmed.starts_with(":::")) {
            cur_tbl_num += 1;
            let (caption, id_opt) = parse_caption_and_label(trimmed);
            let id = id_opt.unwrap_or_else(|| format!("tbl-{cur_tbl_num}"));
            let clean_id = id.replace(':', "-");

            out.push_str(&format!("<p align=\"center\" id=\"{clean_id}\"><em>Table {cur_tbl_num}: {caption}</em></p>\n\n"));
            line_idx += 1;
            continue;
        }

        // 5. Display Equations with Pandoc labels: $$ ... $$ {#eq:gaussian}
        if trimmed.starts_with("$$") {
            let (math_code, eq_id, end_j) = parse_display_math_block(&lines, line_idx);
            if let Some(eid) = eq_id {
                cur_eq_num += 1;
                let clean_id = eid.replace(':', "-");
                out.push_str(&format!("<a id=\"{clean_id}\"></a>\n"));
                let tag_str = if !math_code.contains(r"\tag{") {
                    format!(" \\tag{{{cur_eq_num}}}")
                } else {
                    String::new()
                };
                out.push_str(&format!("$$\n{math_code}{tag_str}\n$$\n\n"));
            } else {
                out.push_str(&format!("$$\n{math_code}\n$$\n\n"));
            }

            line_idx = end_j + 1;
            continue;
        }

        // 6. Image with diagram extension: ![caption](arch.uxf){#fig:umlet-img}
        if trimmed.starts_with("![") && trimmed.contains('(') {
            if let Some(open_paren) = trimmed.find('(') {
                if let Some(close_paren) = trimmed[open_paren..].find(')') {
                    let url = &trimmed[open_paren + 1..open_paren + close_paren];
                    let norm = url.trim().to_lowercase();
                    if norm.ends_with(".uxf")
                        || norm.ends_with(".umlet")
                        || norm.ends_with(".puml")
                        || norm.ends_with(".plantuml")
                        || norm.ends_with(".mmd")
                        || norm.ends_with(".mermaid")
                        || norm.ends_with(".dot")
                    {
                        cur_fig_num += 1;
                        let mut fig_id = None;
                        let mut fig_width = None;
                        let after_paren = &trimmed[open_paren + close_paren + 1..].trim();
                        if after_paren.starts_with('{') {
                            let (w, _, id_opt) = parse_image_attributes(&after_paren[1..after_paren.len().saturating_sub(1)]);
                            fig_id = id_opt;
                            fig_width = w;
                        }
                        let id = fig_id.unwrap_or_else(|| format!("fig-{cur_fig_num}"));
                        let clean_id = id.replace(':', "-");
                        let alt = trimmed[2..open_paren].trim_end_matches(']').trim();
                        let title = if alt.is_empty() { format!("Figure {cur_fig_num}") } else { alt.to_string() };

                        let diag_type = if norm.ends_with(".uxf") || norm.ends_with(".umlet") {
                            "umlet"
                        } else if norm.ends_with(".puml") || norm.ends_with(".plantuml") {
                            "plantuml"
                        } else if norm.ends_with(".mmd") || norm.ends_with(".mermaid") {
                            "mermaid"
                        } else {
                            "graphviz"
                        };

                        let asset_filename = format!("{clean_id}.svg");
                        let mut export_success = false;
                        if let Some(diag_code) = resolve_file_content(url, opts.resource_dir) {
                            if let Ok(cached_svg) = render_diagram(diag_type, &diag_code, kroki_url) {
                                let _ = fs::create_dir_all(&target_assets_dir);
                                let target_asset_path = target_assets_dir.join(&asset_filename);
                                if let Ok(_) = fs::copy(&cached_svg, &target_asset_path) {
                                    exported_assets.push(target_asset_path);
                                    export_success = true;
                                }
                            }
                        }

                        let indent = detect_list_indent(&lines, line_idx).unwrap_or("");
                        let width_attr = if let Some(ref w) = fig_width {
                            format!(" width=\"{w}\"")
                        } else {
                            String::new()
                        };

                        let img_src = if export_success {
                            format!("{relative_assets_prefix}/{asset_filename}")
                        } else {
                            url.to_string()
                        };

                        out.push_str(&format!("{indent}<p align=\"center\" id=\"{clean_id}\">\n"));
                        out.push_str(&format!("{indent}  <img src=\"{img_src}\" alt=\"{title}\"{width_attr} style=\"border-radius: 8px;\" />"));
                        if !title.is_empty() {
                            out.push_str(&format!("<br>\n{indent}  <em>Figure {cur_fig_num}: {title}</em>\n"));
                        } else {
                            out.push('\n');
                        }
                        out.push_str(&format!("{indent}</p>\n\n"));

                        line_idx += 1;
                        continue;
                    }
                }
            }
        }

        // 7. Table of contents line: \toc, \tableofcontents, <!-- toc -->
        if let Some(custom_toc_title) = parse_toc_line(trimmed) {
            if !has_rendered_toc {
                let title_to_use = custom_toc_title.as_deref().unwrap_or(toc_title);
                out.push_str(&render_toc(&headings_list, title_to_use, toc_depth));
                out.push_str("\n\n");
                has_rendered_toc = true;
            }
            line_idx += 1;
            continue;
        }

        // 8. References line: \ref "Works Cited", \references, \bibliography
        if let Some(custom_bib_title) = parse_ref_line(trimmed) {
            let title_to_use = custom_bib_title.as_deref().unwrap_or(biblio_title.unwrap_or("References"));
            let prev_heading_is_ref = headings_list.iter().rev().find(|(lvl, _, _, _)| *lvl == 1)
                .map(|(_, title, _, _)| is_references_heading_title(title, biblio_title))
                .unwrap_or(false);
            let heading_level = if custom_bib_title.is_some() {
                if prev_heading_is_ref { 2 } else { 1 }
            } else if prev_heading_is_ref {
                0
            } else {
                1
            };
            out.push_str(&render_references_section(title_to_use, &references, lang, heading_level));
            out.push_str("\n\n");
            has_rendered_ref = true;
            line_idx += 1;
            continue;
        }

        // 9. Page breaks and date macros
        let mut processed_line = line.to_string();

        if processed_line.trim() == r"\pagebreak"
            || processed_line.trim() == r"\newpage"
            || processed_line.trim() == "<!-- pagebreak -->"
            || processed_line.trim() == "<!--pagebreak-->"
            || processed_line.trim() == "==="
        {
            out.push_str("---\n\n");
            line_idx += 1;
            continue;
        }

        // Remove pause tokens
        if processed_line.trim() == r"\pause"
            || processed_line.trim() == "<!-- pause -->"
            || processed_line.trim() == "<!--pause-->"
            || processed_line.trim() == "::: pause"
        {
            line_idx += 1;
            continue;
        }
        if processed_line.contains(r"\pause") {
            processed_line = processed_line.replace(r"\pause", "");
        }

        // Replace \today macro
        if processed_line.contains(r"\today") {
            let today_str = format_today_date(lang);
            processed_line = processed_line.replace(r"\today", &today_str);
        }

        // 10. Transform cross-references and citation links in inline text
        let final_line = transform_inline_cross_refs_and_citations(
            &processed_line,
            &labels,
            &references,
            active_bibliography,
        );

        out.push_str(&final_line);
        out.push('\n');
        line_idx += 1;
    }

    // 11. Append bibliography at bottom if active and not explicitly placed
    if active_bibliography && !has_rendered_ref && !references.is_empty() {
        let title_to_use = biblio_title.unwrap_or(match lang {
            "fr" => "Références",
            "de" => "Literaturverzeichnis",
            "es" => "Referencias",
            "it" => "Riferimenti bibliografici",
            _ => "References",
        });
        let last_heading_is_ref = headings_list.last()
            .map(|(_, title, _, _)| is_references_heading_title(title, biblio_title))
            .unwrap_or(false);
        let heading_level = if last_heading_is_ref { 0 } else { 1 };
        out.push_str("\n");
        out.push_str(&render_references_section(title_to_use, &references, lang, heading_level));
        out.push('\n');
    }

    let normalized = normalize_markdown_blank_lines(&out);

    Ok(MarkdownExportResult {
        content: normalized,
        exported_assets,
    })
}

fn render_toc(
    headings: &[(usize, String, String, String)],
    toc_title: &str,
    max_depth: usize,
) -> String {
    let mut s = format!("## {toc_title}\n\n");
    for (level, title, id, number) in headings {
        if *level > max_depth {
            continue;
        }
        let indent = "  ".repeat(level.saturating_sub(1));
        let num_str = if number.is_empty() {
            String::new()
        } else {
            format!("{number} ")
        };
        s.push_str(&format!("{indent}- [{num_str}{title}](#{id})\n"));
    }
    s
}

fn render_references_section(
    title: &str,
    refs: &[(String, String)],
    lang: &str,
    heading_level: usize,
) -> String {
    let (online_label, available_label) = match lang {
        "fr" => ("[En ligne]", "Disponible sur :"),
        "de" => ("[Online]", "Verfügbar unter:"),
        "es" => ("[En línea]", "Disponible en:"),
        "it" => ("[Online]", "Disponibile su:"),
        _ => ("[Online]", "Available:"),
    };

    let mut s = String::new();
    if heading_level > 0 && !title.is_empty() {
        let hashes = "#".repeat(heading_level);
        s.push_str(&format!("{hashes} {title}\n\n"));
    }

    for (i, (url, label)) in refs.iter().enumerate() {
        let idx = i + 1;
        let is_url_label = label.is_empty()
            || label == url
            || label.starts_with("http://")
            || label.starts_with("https://");

        if is_url_label {
            s.push_str(&format!(
                "<a id=\"bib-{idx}\"></a>[{idx}] {online_label}. {available_label} <{url}>.\n\n"
            ));
        } else {
            s.push_str(&format!(
                "<a id=\"bib-{idx}\"></a>[{idx}] \"{label}\", {online_label}. {available_label} <{url}>.\n\n"
            ));
        }
    }
    s
}

fn transform_inline_cross_refs_and_citations(
    line: &str,
    labels: &HashMap<String, TargetInfo>,
    references: &[(String, String)],
    active_bibliography: bool,
) -> String {
    let mut result = String::with_capacity(line.len() * 2);
    let chars: Vec<char> = line.chars().collect();
    let n = chars.len();
    let mut i = 0;

    while i < n {
        // A. Check for Pandoc cross references: @sec:..., @fig:..., @tbl:..., @eq:..., @lst:...
        if chars[i] == '@' && (i == 0 || chars[i - 1].is_whitespace() || chars[i - 1] == '(' || chars[i - 1] == '[') {
            let mut j = i + 1;
            while j < n && (chars[j].is_alphanumeric() || chars[j] == ':' || chars[j] == '-' || chars[j] == '_') {
                j += 1;
            }
            let key: String = chars[i + 1..j].iter().collect();
            let clean_key = key.replace(':', "-");

            // Look up target
            if let Some(target) = labels.get(&clean_key).or_else(|| labels.get(&key)) {
                // Determine preceding context: did preceding text say "Figure ", "Table ", "Equation ", etc.?
                let prev_text = result.trim_end();
                let link_text = match target.kind {
                    TargetKind::Figure => {
                        if prev_text.ends_with("Figure") || prev_text.ends_with("Fig.") {
                            target.number.replace("Figure ", "")
                        } else {
                            target.number.clone()
                        }
                    }
                    TargetKind::Table => {
                        if prev_text.ends_with("Table") || prev_text.ends_with("Tab.") {
                            target.number.replace("Table ", "")
                        } else {
                            target.number.clone()
                        }
                    }
                    TargetKind::Equation => {
                        if prev_text.ends_with("Equation") || prev_text.ends_with("Eq.") {
                            target.number.clone()
                        } else {
                            format!("Equation {}", target.number)
                        }
                    }
                    TargetKind::Section => {
                        if prev_text.ends_with("Section") || prev_text.ends_with("Sec.") {
                            if target.number.is_empty() {
                                target.title.clone()
                            } else {
                                target.number.clone()
                            }
                        } else if target.number.is_empty() {
                            target.title.clone()
                        } else {
                            format!("Section {}", target.number)
                        }
                    }
                };
                result.push_str(&format!("[{link_text}](#{})", target.id));
                i = j;
                continue;
            } else if key.starts_with("sec:") || key.starts_with("fig:") || key.starts_with("tbl:") || key.starts_with("eq:") {
                // Unknown label, link to slug anyway
                result.push_str(&format!("[{clean_key}](#{clean_key})"));
                i = j;
                continue;
            }
        }

        // B. Check for web links to add bibliography citation tags: [Text](URL)
        if active_bibliography && chars[i] == '[' && (i == 0 || chars[i - 1] != '!') {
            let mut close_bracket = None;
            let mut j = i + 1;
            while j < n {
                if chars[j] == ']' {
                    close_bracket = Some(j);
                    break;
                }
                j += 1;
            }
            if let Some(cb) = close_bracket {
                if cb + 1 < n && chars[cb + 1] == '(' {
                    let mut close_paren = None;
                    let mut k = cb + 2;
                    while k < n {
                        if chars[k] == ')' {
                            close_paren = Some(k);
                            break;
                        }
                        k += 1;
                    }
                    if let Some(cp) = close_paren {
                        let link_text: String = chars[i + 1..cb].iter().collect();
                        let link_url: String = chars[cb + 2..cp].iter().collect();
                        let trimmed_url = link_url.trim();

                        if let Some(pos) = references.iter().position(|(u, _)| u == trimmed_url) {
                            let idx = pos + 1;
                            // Check if next characters already contain citation e.g. [[1]] or [1]
                            let remainder: String = chars[cp + 1..].iter().collect();
                            let rem_trimmed = remainder.trim_start();
                            let already_cited = rem_trimmed.starts_with(&format!("[[{idx}]]"))
                                || rem_trimmed.starts_with(&format!("[{idx}]"));

                            if !already_cited {
                                result.push_str(&format!("[{link_text}]({link_url}) [[{idx}]](#bib-{idx})"));
                                i = cp + 1;
                                continue;
                            }
                        }
                    }
                }
            }
        }

        result.push(chars[i]);
        i += 1;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_basic_markdown_with_headings_and_today() {
        let md = "# Introduction {#sec:intro}\n\nDocument generated on \\today.\n\n# Mathematics {#sec:math}\n\nContent.";
        let opts = MarkdownExportOptions {
            number_sections: true,
            ..Default::default()
        };
        let res = export_to_normal_markdown(md, &opts).unwrap();
        assert!(res.content.contains("<a id=\"sec-intro\"></a>"));
        assert!(res.content.contains("# 1 Introduction"));
        assert!(res.content.contains("# 2 Mathematics"));
        assert!(!res.content.contains(r"\today"));
    }

    #[test]
    fn test_export_callouts_conversion() {
        let md = "::: important\nThis is an important message.\n:::\n\n> [!NOTE]\n> Keep this note.";
        let opts = MarkdownExportOptions::default();
        let res = export_to_normal_markdown(md, &opts).unwrap();
        assert!(res.content.contains("> [!IMPORTANT]"));
        assert!(res.content.contains("> This is an important message."));
        assert!(res.content.contains("> [!NOTE]"));
        assert!(res.content.contains("> Keep this note."));
    }

    #[test]
    fn test_export_cross_references() {
        let md = "# Introduction {#sec:intro}\n\nTable: Targets {#tbl:targets}\n\n| A | B |\n|---|---|\n\nAs shown in Table @tbl:targets and Section @sec:intro.";
        let opts = MarkdownExportOptions {
            number_sections: true,
            ..Default::default()
        };
        let res = export_to_normal_markdown(md, &opts).unwrap();
        assert!(res.content.contains("id=\"tbl-targets\""));
        assert!(res.content.contains("[1](#tbl-targets)"));
        assert!(res.content.contains("[1](#sec-intro)"));
    }

    #[test]
    fn test_export_bibliography() {
        let md = "Read [Typst Docs](https://typst.app/docs) for details.\n\n\\ref \"Works Cited\"";
        let opts = MarkdownExportOptions {
            bibliography: true,
            ..Default::default()
        };
        let res = export_to_normal_markdown(md, &opts).unwrap();
        assert!(res.content.contains("[Typst Docs](https://typst.app/docs) [[1]](#bib-1)"));
        assert!(res.content.contains("# Works Cited"));
        assert!(res.content.contains("<a id=\"bib-1\"></a>[1] \"Typst Docs\", [Online]. Available: <https://typst.app/docs>."));
    }

    #[test]
    fn test_export_table_of_contents() {
        let md = "\\toc\n\n# Introduction {#sec:intro}\n\n## Details {#sec:details}";
        let opts = MarkdownExportOptions {
            number_sections: true,
            ..Default::default()
        };
        let res = export_to_normal_markdown(md, &opts).unwrap();
        assert!(res.content.contains("## Table of Contents"));
        assert!(res.content.contains("- [1 Introduction](#sec-intro)"));
        assert!(res.content.contains("  - [1.1 Details](#sec-details)"));
    }

    #[test]
    fn test_export_diagram_figure_formatting() {
        let md = "```mermaid {caption=\"Compiler Pipeline\" width=70% #fig:mermaid-pipeline}\ngraph LR\n  A --> B\n```";
        let opts = MarkdownExportOptions::default();
        let res = export_to_normal_markdown(md, &opts).unwrap();
        assert!(res.content.contains("id=\"fig-mermaid-pipeline\""));
        assert!(res.content.contains("<p align=\"center\" id=\"fig-mermaid-pipeline\">"));
        assert!(res.content.contains("Figure 1: Compiler Pipeline"));
        assert!(res.content.contains("</p>"));
    }

    #[test]
    fn test_export_frontmatter_cleaning() {
        let md = "---\ntitle: My Document\ntheme: elegant\npapersize: a4\ntoc: true\nauthor: Alice\n---\n\n# Body";
        let opts = MarkdownExportOptions::default();
        let res = export_to_normal_markdown(md, &opts).unwrap();
        assert!(res.content.contains("title: My Document"));
        assert!(res.content.contains("author: Alice"));
        assert!(!res.content.contains("theme: elegant"));
        assert!(!res.content.contains("papersize: a4"));
        assert!(!res.content.contains("toc: true"));
    }

    #[test]
    fn test_export_ordered_list_with_diagram() {
        let md = "1. First item:\n\n```mermaid {caption=\"Step 1\" #fig:step1}\ngraph LR\n  A --> B\n```\n\n2. Second item.";
        let opts = MarkdownExportOptions::default();
        let res = export_to_normal_markdown(md, &opts).unwrap();
        assert!(res.content.contains("1. First item:"));
        assert!(res.content.contains("   <p align=\"center\" id=\"fig-step1\">"));
        assert!(res.content.contains("2. Second item."));
    }

    #[test]
    fn test_export_alt_text_preservation() {
        let md = "![UMLet Architecture via Image Syntax](architecture.uxf){#fig:arch width=60%}";
        let opts = MarkdownExportOptions::default();
        let res = export_to_normal_markdown(md, &opts).unwrap();
        assert!(res.content.contains("alt=\"UMLet Architecture via Image Syntax\""));
        assert!(res.content.contains("Figure 1: UMLet Architecture via Image Syntax"));
    }

    #[test]
    fn test_extract_frontmatter_raw_with_dashes_in_fields() {
        let md = "---\ntitle: Guide\ndescription: Zero-cost --- Abstractions\n---\n# Body\nHere is content with --- inside body.\n";
        let (fm, body) = extract_frontmatter_raw(md);
        assert!(fm.contains("title: Guide"));
        assert!(fm.contains("description: Zero-cost --- Abstractions"));
        assert!(body.starts_with("# Body"));
    }
}
