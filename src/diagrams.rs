use std::fs;
use std::path::PathBuf;
use std::time::Duration;
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

/// Fetch a Mermaid SVG directly from mermaid.ink as a companion fallback.
pub fn fetch_mermaid_ink(canonical_code: &str) -> Result<Vec<u8>, String> {
    use base64::prelude::*;
    let encoded = BASE64_URL_SAFE.encode(canonical_code.as_bytes());
    let url = format!("https://mermaid.ink/svg/{encoded}");

    let client = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_millis(4000)))
        .build()
        .new_agent();

    let mut resp = client
        .get(&url)
        .header("User-Agent", "botox/0.1")
        .call()
        .map_err(|e| format!("mermaid.ink connection failed: {e}"))?;

    let status = resp.status();
    if !status.is_success() {
        return Err(format!("mermaid.ink returned HTTP {status}"));
    }

    let svg_bytes = resp
        .body_mut()
        .read_to_vec()
        .map_err(|e| format!("Failed to read response from mermaid.ink: {e}"))?;

    if svg_bytes.is_empty() {
        return Err("mermaid.ink returned an empty SVG response".to_string());
    }

    Ok(svg_bytes)
}

/// Render a diagram using Kroki (or mermaid.ink fallback for Mermaid) with local caching.
/// If the diagram was already rendered and cached, it is returned immediately (0ms).
/// If not cached, it attempts an HTTP POST request to Kroki.
/// For Mermaid, if Kroki fails or times out, it automatically falls back to mermaid.ink.
/// On failure, returns an error message so the caller can render a fallback.
pub fn render_diagram(
    diagram_type: &str,
    code: &str,
    kroki_endpoint: Option<&str>,
) -> Result<PathBuf, String> {
    let clean_type = diagram_type.trim().to_lowercase();
    let norm_type = match clean_type.as_str() {
        "puml" => "plantuml",
        "uxf" => "umlet",
        other => other,
    };

    let canonical_code = canonicalize_diagram_source(norm_type, code);
    if canonical_code.is_empty() {
        return Err("Diagram source code is empty".to_string());
    }

    let cache_dir = get_cache_dir();
    if let Err(e) = fs::create_dir_all(&cache_dir) {
        return Err(format!("Failed to create diagram cache dir: {e}"));
    }

    let hash = compute_diagram_hash(norm_type, &canonical_code);
    let cache_file = cache_dir.join(format!("{hash}.svg"));

    // Check existing cache
    if cache_file.is_file()
        && let Ok(meta) = fs::metadata(&cache_file)
            && meta.len() > 0 {
                if let Ok(content) = fs::read_to_string(&cache_file)
                    && content.contains("<foreignObject") {
                        let sanitized = sanitize_svg_for_typesetting(&content);
                        let _ = fs::write(&cache_file, sanitized.as_bytes());
                }
                return Ok(cache_file);
            }

    // Resolve Kroki base URL
    let base_url = kroki_endpoint
        .map(|s| s.to_string())
        .or_else(|| std::env::var("KROKI_ENDPOINT").ok())
        .or_else(|| std::env::var("KROKI_URL").ok())
        .unwrap_or_else(|| "https://140.238.215.250.sslip.io".to_string());

    let target_url = format!("{}/{}/svg", base_url.trim_end_matches('/'), norm_type);

    let client = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_millis(3500)))
        .build()
        .new_agent();

    let kroki_res = client
        .post(&target_url)
        .header("Content-Type", "text/plain; charset=utf-8")
        .header("User-Agent", "botox/0.1")
        .send(&canonical_code);

    let svg_bytes = match kroki_res {
        Ok(mut resp) => {
            let status = resp.status();
            if !status.is_success() {
                if norm_type == "mermaid" {
                    // Try mermaid.ink fallback
                    fetch_mermaid_ink(&canonical_code)
                        .map_err(|e| format!("Kroki returned HTTP {status}; fallback failed: {e}"))?
                } else {
                    return Err(format!("Kroki returned HTTP {status}"));
                }
            } else {
                let bytes = resp
                    .body_mut()
                    .read_to_vec()
                    .map_err(|e| format!("Failed to read response from Kroki: {e}"))?;
                if bytes.is_empty() {
                    return Err("Kroki returned an empty SVG response".to_string());
                }
                bytes
            }
        }
        Err(e) => {
            if norm_type == "mermaid" {
                // Kroki connection failed or timed out, attempt mermaid.ink fallback
                fetch_mermaid_ink(&canonical_code)
                    .map_err(|ink_err| format!("Kroki unreachable ({e}); mermaid.ink failed ({ink_err})"))?
            } else {
                return Err(format!("Kroki connection failed: {e}"));
            }
        }
    };

    let is_svg = svg_bytes.windows(4).any(|w| w == b"<svg")
        || svg_bytes.windows(5).any(|w| w == b"<?xml");
    if !is_svg {
        let snippet = String::from_utf8_lossy(&svg_bytes[..svg_bytes.len().min(120)]);
        return Err(format!("Diagram rendering returned non-SVG content: {snippet}"));
    }

    let sanitized_svg = sanitize_svg_for_typesetting(&String::from_utf8_lossy(&svg_bytes));
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

/// Sanitize SVG markup so that rasterizers and PDF generators like Typst (which rely on resvg)
/// can properly render text labels.
///
/// Mermaid and some other diagram engines embed labels inside `<foreignObject>` containing HTML.
/// Because resvg strictly adheres to static SVG and lacks an HTML layout engine, `<foreignObject>`
/// elements are ignored, leaving boxes and nodes without visible text.
///
/// This function translates `<foreignObject>` elements into standard SVG `<text>` (and `<tspan>`) elements.
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

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_canonicalize_and_hash_newline_invariant() {
        let code1 = "graph TD\n  A --> B";
        let code2 = "\n\n  graph TD  \n\n\n    A --> B\n\n";
        let code3 = "%% comment\ngraph TD\n\n  A --> B\n";
        assert_eq!(canonicalize_diagram_source("mermaid", code1), "graph TD\nA --> B");
        assert_eq!(canonicalize_diagram_source("mermaid", code2), "graph TD\nA --> B");
        assert_eq!(canonicalize_diagram_source("mermaid", code3), "graph TD\nA --> B");
        assert_eq!(compute_diagram_hash("mermaid", code1), compute_diagram_hash("mermaid", code2));
        assert_eq!(compute_diagram_hash("mermaid", code1), compute_diagram_hash("mermaid", code3));

        let puml1 = "@startuml\nClient -> Server : Ping\n@enduml";
        let puml2 = "\n\n@startuml\n\n  Client -> Server : Ping  \n\n@enduml\n\n";
        let puml3 = "@startuml\n' some developer comment\nClient -> Server : Ping\n@enduml";
        assert_eq!(canonicalize_diagram_source("plantuml", puml1), canonicalize_diagram_source("plantuml", puml2));
        assert_eq!(canonicalize_diagram_source("plantuml", puml1), canonicalize_diagram_source("plantuml", puml3));
        assert_eq!(compute_diagram_hash("plantuml", puml1), compute_diagram_hash("plantuml", puml2));
        assert_eq!(compute_diagram_hash("plantuml", puml1), compute_diagram_hash("plantuml", puml3));
    }

    #[test]
    fn test_compute_diagram_hash_deterministic() {
        let h1 = compute_diagram_hash("mermaid", "graph TD; A-->B;");
        let h2 = compute_diagram_hash("mermaid", "graph TD; A-->B;");
        let h3 = compute_diagram_hash("mermaid", "graph TD; A-->C;");
        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
    }

    #[test]
    fn test_render_diagram_cached_hit() {
        let cache_dir = get_cache_dir();
        let _ = fs::create_dir_all(&cache_dir);
        let test_code = "test_diagram_cached_hit_mock";
        let hash = compute_diagram_hash("mermaid", test_code);
        let cache_file = cache_dir.join(format!("{hash}.svg"));
        
        // Populate cache manually
        let dummy_svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\"><text>Mock Diagram</text></svg>";
        fs::write(&cache_file, dummy_svg).expect("Write mock cache");

        // Request render with an invalid endpoint to prove network is NOT hit when cached
        let res = render_diagram("mermaid", test_code, Some("http://invalid.local.domain.does.not.exist"));
        assert!(res.is_ok());
        let path = res.unwrap();
        assert_eq!(path, cache_file);

        let _ = fs::remove_file(cache_file);
    }

    #[test]
    fn test_render_diagram_unreachable_endpoint_fallback() {
        let test_code = "unique_uncached_diagram_test_code_12345";
        let res = render_diagram("plantuml", test_code, Some("http://127.0.0.1:1"));
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(err.contains("Kroki connection failed") || err.contains("connection refused") || err.contains("failed"));
    }

    #[test]
    fn test_fetch_mermaid_ink_direct() {
        let code = "graph TD\n  A --> B";
        if let Ok(svg) = fetch_mermaid_ink(code) {
            let svg_str = String::from_utf8_lossy(&svg);
            assert!(svg_str.contains("<svg"));
            assert!(svg_str.contains("</svg>"));
        }
    }

    #[test]
    fn test_sanitize_svg_single_line_foreignobject() {
        let raw_svg = r#"<svg><g class="label" transform="translate(-60, -12)"><foreignObject width="120" height="24"><div xmlns="http://www.w3.org/1999/xhtml"><span class="nodeLabel"><p>Hello World</p></span></div></foreignObject></g></svg>"#;
        let sanitized = sanitize_svg_for_typesetting(raw_svg);
        assert!(!sanitized.contains("<foreignObject"));
        assert!(sanitized.contains("<text"));
        assert!(sanitized.contains("Hello World"));
        assert!(sanitized.contains("x=\"60.0\""));
        assert!(sanitized.contains("y=\"12.0\""));
    }

    #[test]
    fn test_sanitize_svg_multiline_foreignobject() {
        let raw_svg = r#"<svg><foreignObject width="100" height="50"><div><p>First Line<br/>Second Line</p></div></foreignObject></svg>"#;
        let sanitized = sanitize_svg_for_typesetting(raw_svg);
        assert!(!sanitized.contains("<foreignObject"));
        assert!(sanitized.contains("<text"));
        assert!(sanitized.contains("<tspan"));
        assert!(sanitized.contains("First Line"));
        assert!(sanitized.contains("Second Line"));
    }

    #[test]
    fn test_sanitize_svg_entity_decoding() {
        let raw_svg = r#"<svg><foreignObject width="80" height="20"><div><span>A &amp; B &gt; C</span></div></foreignObject></svg>"#;
        let sanitized = sanitize_svg_for_typesetting(raw_svg);
        assert!(!sanitized.contains("<foreignObject"));
        assert!(sanitized.contains("A &amp; B &gt; C"));
    }
}

