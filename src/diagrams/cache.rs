use std::fs;
use std::path::PathBuf;
use sha2::{Digest, Sha256};

/// Determine the local cache directory for diagrams.
pub fn get_cache_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("BOTOX_CACHE_DIR") {
        return PathBuf::from(dir);
    }
    if let Ok(xdg) = std::env::var("XDG_CACHE_HOME") {
        return PathBuf::from(xdg).join("botox").join("diagrams");
    }
    if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
        return PathBuf::from(local_appdata).join("botox").join("diagrams");
    }
    if let Some(home) = crate::config::dirs_home() {
        return home.join(".cache").join("botox").join("diagrams");
    }
    std::env::temp_dir().join("botox_diagrams")
}

/// Minimize and canonicalize diagram source text so that stylistic variations
/// (such as added newlines, blank lines, indentation changes, CRLF vs LF, or comment lines)
/// do not alter the cache hash or trigger unnecessary Kroki requests.
pub fn canonicalize_diagram_source(diagram_type: &str, code: &str) -> String {
    let norm_type = diagram_type.trim().to_lowercase();
    let mut lines = Vec::new();

    for raw_line in code.lines() {
        let line = raw_line.trim();
        // Skip blank or whitespace-only lines
        if line.is_empty() {
            continue;
        }

        // Skip comments that have no effect on rendered output
        if norm_type == "mermaid" {
            // In Mermaid, %% is comment unless it's a %%{init: ...}%% directive
            if line.starts_with("%%") && !line.starts_with("%%{") {
                continue;
            }
        } else if (norm_type == "plantuml" || norm_type == "puml") && line.starts_with('\'') {
            // In PlantUML, lines starting with ' are comments
            continue;
        }

        lines.push(line);
    }

    lines.join("\n")
}

/// Compute a unique deterministic cache hash for a diagram.
pub fn compute_diagram_hash(diagram_type: &str, code: &str) -> String {
    let clean_type = diagram_type.trim().to_lowercase();
    let norm_type = match clean_type.as_str() {
        "puml" => "plantuml",
        "uxf" => "umlet",
        other => other,
    };
    let canonical = canonicalize_diagram_source(norm_type, code);
    let mut hasher = Sha256::new();
    hasher.update(norm_type.as_bytes());
    hasher.update(b":");
    hasher.update(canonical.as_bytes());
    let result = hasher.finalize();
    result.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Sanitize SVG markup so that rasterizers and PDF generators like Typst (which rely on resvg)
/// can properly render text labels.
pub fn sanitize_svg_for_typesetting(svg: &str) -> String {
    if !svg.contains("<foreignObject") {
        return svg.to_string();
    }

    let mut result = String::with_capacity(svg.len());
    let mut remaining = svg;

    while let Some(start_idx) = remaining.find("<foreignObject") {
        result.push_str(&remaining[..start_idx]);
        let after_start = &remaining[start_idx..];

        // Find end of <foreignObject ...> opening tag
        let tag_close = match after_start.find('>') {
            Some(idx) => idx,
            None => {
                result.push_str(after_start);
                return result;
            }
        };

        let fo_tag = &after_start[..tag_close + 1];
        let content_start = tag_close + 1;

        // Find closing </foreignObject>
        let end_idx = match after_start[content_start..].find("</foreignObject>") {
            Some(idx) => content_start + idx,
            None => {
                result.push_str(after_start);
                return result;
            }
        };

        let inner_content = &after_start[content_start..end_idx];
        remaining = &after_start[end_idx + "</foreignObject>".len()..];

        // Parse attributes from <foreignObject ...>
        let parse_attr = |attr_name: &str| -> Option<f64> {
            let pat_double = format!("{attr_name}=\"");
            if let Some(pos) = fo_tag.find(&pat_double) {
                let val_start = pos + pat_double.len();
                if let Some(val_end) = fo_tag[val_start..].find('"') {
                    return fo_tag[val_start..val_start + val_end].parse::<f64>().ok();
                }
            }
            let pat_single = format!("{attr_name}='");
            if let Some(pos) = fo_tag.find(&pat_single) {
                let val_start = pos + pat_single.len();
                if let Some(val_end) = fo_tag[val_start..].find('\'') {
                    return fo_tag[val_start..val_start + val_end].parse::<f64>().ok();
                }
            }
            None
        };

        let x_attr = parse_attr("x").unwrap_or(0.0);
        let y_attr = parse_attr("y").unwrap_or(0.0);
        let width = parse_attr("width").unwrap_or(0.0);
        let height = parse_attr("height").unwrap_or(0.0);

        // Determine text alignment
        let (anchor, x_pos) = if inner_content.contains("text-align: left") || inner_content.contains("text-align:left") {
            ("start", x_attr + 4.0)
        } else if inner_content.contains("text-align: right") || inner_content.contains("text-align:right") {
            ("end", x_attr + width - 4.0)
        } else {
            ("middle", x_attr + width / 2.0)
        };

        // Extract color if present
        let fill_color = if let Some(color_pos) = inner_content.find("color:") {
            let after_col = inner_content[color_pos + 6..].trim_start();
            let end_col = after_col.find([';', '"', '\'']).unwrap_or(after_col.len());
            let col = after_col[..end_col].trim();
            if col.starts_with('#') || col.starts_with("rgb") {
                col
            } else {
                "#333333"
            }
        } else {
            "#333333"
        };

        // Extract lines of text from HTML
        let clean_html = inner_content
            .replace("<br>", "\n")
            .replace("<br/>", "\n")
            .replace("<br />", "\n")
            .replace("</p>", "\n")
            .replace("</div>", "\n");

        // Strip HTML tags
        let mut in_tag = false;
        let mut text_buf = String::new();
        for ch in clean_html.chars() {
            if ch == '<' {
                in_tag = true;
            } else if ch == '>' {
                in_tag = false;
            } else if !in_tag {
                text_buf.push(ch);
            }
        }

        // Decode HTML entities
        let decoded = text_buf
            .replace("&nbsp;", " ")
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'")
            .replace("&apos;", "'");

        let lines: Vec<&str> = decoded
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();

        if lines.is_empty() {
            continue;
        }

        let escape_xml = |s: &str| -> String {
            s.replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;")
                .replace('"', "&quot;")
        };

        let font_family = "DejaVu Sans, Liberation Sans, -apple-system, Segoe UI, sans-serif";
        let font_size = 14;
        let line_height = 18.0;

        if lines.len() == 1 {
            let y_pos = y_attr + height / 2.0;
            let line_esc = escape_xml(lines[0]);
            result.push_str(&format!(
                r#"<text x="{x_pos:.1}" y="{y_pos:.1}" text-anchor="{anchor}" dominant-baseline="central" fill="{fill_color}" font-family="{font_family}" font-size="{font_size}">{line_esc}</text>"#
            ));
        } else {
            let total_height = (lines.len() as f64 - 1.0) * line_height;
            let start_y = (y_attr + height / 2.0) - (total_height / 2.0);
            let mut tspans = String::new();
            for (idx, line) in lines.iter().enumerate() {
                let line_esc = escape_xml(line);
                if idx == 0 {
                    tspans.push_str(&format!(
                        r#"<tspan x="{x_pos:.1}" y="{start_y:.1}">{line_esc}</tspan>"#
                    ));
                } else {
                    tspans.push_str(&format!(
                        r#"<tspan x="{x_pos:.1}" dy="{line_height:.1}">{line_esc}</tspan>"#
                    ));
                }
            }
            result.push_str(&format!(
                r#"<text text-anchor="{anchor}" dominant-baseline="central" fill="{fill_color}" font-family="{font_family}" font-size="{font_size}">{tspans}</text>"#
            ));
        }
    }

    result.push_str(remaining);
    result
}

pub fn save_to_cache(hash: &str, svg_bytes: &[u8]) -> Result<PathBuf, String> {
    let cache_dir = get_cache_dir();
    fs::create_dir_all(&cache_dir).map_err(|e| format!("Failed to create diagram cache dir: {e}"))?;
    let cache_file = cache_dir.join(format!("{hash}.svg"));

    let sanitized_svg = sanitize_svg_for_typesetting(&String::from_utf8_lossy(svg_bytes));
    let final_bytes = sanitized_svg.into_bytes();

    let tmp_file = cache_dir.join(format!("{hash}.{}.tmp", std::process::id()));
    if let Err(e) = fs::write(&tmp_file, &final_bytes) {
        return Err(format!("Failed to write diagram to cache: {e}"));
    }
    if let Err(e) = fs::rename(&tmp_file, &cache_file) {
        let _ = fs::remove_file(&tmp_file);
        fs::write(&cache_file, &final_bytes)
            .map_err(|e2| format!("Failed to write diagram cache: {e2} (rename failed: {e})"))?;
    }

    Ok(cache_file)
}
