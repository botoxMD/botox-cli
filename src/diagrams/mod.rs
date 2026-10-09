pub mod cache;
pub mod kroki;

use std::fs;
use std::path::{Path, PathBuf};

pub use cache::{
    canonicalize_diagram_source, compute_diagram_hash, get_cache_dir, sanitize_svg_for_typesetting,
};
pub use kroki::{fetch_kroki, fetch_mermaid_ink};

#[derive(Debug, Clone, PartialEq)]
pub struct DiagramSpec {
    pub diagram_type: String,
    pub caption: Option<String>,
    pub width: Option<String>,
    pub height: Option<String>,
    pub id: Option<String>,
    pub file: Option<String>,
}

pub fn resolve_file_content(path_str: &str, resource_dir: Option<&Path>) -> Option<String> {
    let trimmed = path_str.trim().trim_matches(|c| c == '"' || c == '\'');
    if trimmed.is_empty() {
        return None;
    }
    let p = Path::new(trimmed);
    if p.is_file() {
        return fs::read_to_string(p).ok();
    }
    if let Some(res_dir) = resource_dir {
        let joined = res_dir.join(p);
        if joined.is_file() {
            return fs::read_to_string(joined).ok();
        }
    }
    None
}

pub fn parse_code_fence_file(fence: &str) -> (String, Option<String>) {
    let trimmed = fence.trim();
    let inside = if trimmed.starts_with('{') && trimmed.ends_with('}') {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    };
    let mut parts = inside.split_whitespace();
    let first = parts.next().unwrap_or("").trim_start_matches('.');
    let (lang, colon_file) = if let Some((l, f)) = first.split_once(':') {
        (l.to_string(), Some(f.to_string()))
    } else {
        (first.to_string(), None)
    };

    let mut file = colon_file;
    if file.is_none() {
        for attr in ["file=", "src=", "path="] {
            if let Some(pos) = inside.find(attr) {
                let rest = inside[pos + attr.len()..].trim_start();
                if let Some(stripped) = rest.strip_prefix('"') {
                    if let Some(end) = stripped.find('"') {
                        file = Some(stripped[..end].to_string());
                        break;
                    }
                } else if let Some(stripped) = rest.strip_prefix('\'') {
                    if let Some(end) = stripped.find('\'') {
                        file = Some(stripped[..end].to_string());
                        break;
                    }
                } else {
                    let token = rest.split_whitespace().next().unwrap_or("").trim_end_matches('}');
                    if !token.is_empty() {
                        file = Some(token.to_string());
                        break;
                    }
                }
            }
        }
    }
    (lang, file)
}

fn parse_dimension(val: &str) -> Option<String> {
    let trimmed = val.trim().trim_matches(|c| c == '"' || c == '\'');
    if trimmed.is_empty() {
        return None;
    }
    if let Some(stripped) = trimmed.strip_prefix("0.")
        && let Some(rest) = stripped.strip_suffix(r"\linewidth")
            .or_else(|| stripped.strip_suffix(r"\textwidth"))
            .or_else(|| stripped.strip_suffix("\\columnwidth"))
    {
        let num_str = format!("0.{}", rest.trim());
        if let Ok(pct) = num_str.parse::<f64>() {
            return Some(format!("{}%", (pct * 100.0).round() as i64));
        }
    }
    if trimmed.ends_with(r"\linewidth")
        || trimmed.ends_with(r"\textwidth")
        || trimmed.ends_with("\\columnwidth")
    {
        return Some("100%".to_string());
    }
    Some(trimmed.to_string())
}

pub fn parse_diagram_fence_attributes(s: &str) -> (Option<String>, Option<String>, Option<String>) {
    let mut width = None;
    let mut height = None;
    let mut id = None;

    for token in s.split_whitespace() {
        let token = token.trim_matches(|c| c == ',' || c == '{' || c == '}');
        if token.starts_with('#') || token.starts_with(r"\#") {
            let raw_id = token.strip_prefix(r"\#").or_else(|| token.strip_prefix('#')).unwrap_or(token);
            let clean_id = raw_id.replace(':', "-");
            if !clean_id.is_empty() {
                id = Some(clean_id);
            }
        } else if let Some(stripped) = token.strip_prefix("id=") {
            let val = stripped.trim_matches('"').trim_matches('\'').replace(':', "-");
            if !val.is_empty() {
                id = Some(val);
            }
        } else if let Some(stripped) = token.strip_prefix("width=") {
            width = parse_dimension(stripped);
        } else if let Some(stripped) = token.strip_prefix("height=") {
            height = parse_dimension(stripped);
        }
    }

    (width, height, id)
}

pub fn parse_diagram_fence(fence: &str) -> Option<DiagramSpec> {
    let trimmed = fence.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut lang_candidate = String::new();
    let mut attr_str = "";
    let mut pre_brace_attr = "";
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        let inside = trimmed[1..trimmed.len() - 1].trim();
        for token in inside.split_whitespace() {
            if token.starts_with('.') {
                lang_candidate = token.trim_start_matches('.').to_string();
                break;
            }
        }
        attr_str = inside;
    } else if let Some(brace_pos) = trimmed.find('{') {
        let before_brace = trimmed[..brace_pos].trim();
        let mut parts = before_brace.splitn(2, |c: char| c.is_whitespace());
        lang_candidate = parts.next().unwrap_or("").trim_start_matches('.').to_string();
        pre_brace_attr = parts.next().unwrap_or("").trim();

        let rest = &trimmed[brace_pos + 1..];
        if let Some(end_brace) = rest.find('}') {
            attr_str = &rest[..end_brace];
        } else {
            attr_str = rest;
        }
    } else {
        let mut parts = trimmed.splitn(2, |c: char| c.is_whitespace());
        lang_candidate = parts.next().unwrap_or("").trim_start_matches('.').to_string();
        if let Some(rest) = parts.next() {
            attr_str = rest;
        }
    }

    let (lang_only, colon_file) = if let Some((l, f)) = lang_candidate.split_once(':') {
        (l.to_string(), Some(f.to_string()))
    } else {
        (lang_candidate, None)
    };

    let norm_lang = lang_only
        .strip_prefix("diagram-")
        .unwrap_or(&lang_only)
        .to_lowercase();

    let diagram_type = match norm_lang.as_str() {
        "mermaid" | "mmd" => "mermaid".to_string(),
        "plantuml" | "puml" => "plantuml".to_string(),
        "umlet" | "uxf" => "umlet".to_string(),
        "graphviz" | "dot" => "graphviz".to_string(),
        "ditaa" => "ditaa".to_string(),
        "wavedrom" => "wavedrom".to_string(),
        "bytefield" => "bytefield".to_string(),
        "bpmn" => "bpmn".to_string(),
        _ => return None,
    };

    let mut caption = None;
    if let Some(cap_start) = attr_str.find("caption=") {
        let rest = attr_str[cap_start + 8..].trim_start();
        if let Some(stripped) = rest.strip_prefix('"') {
            if let Some(cap_end) = stripped.find('"') {
                caption = Some(stripped[..cap_end].to_string());
            }
        } else if let Some(stripped) = rest.strip_prefix('\'')
            && let Some(cap_end) = stripped.find('\'') {
                caption = Some(stripped[..cap_end].to_string());
            }
    } else if let Some(title_start) = attr_str.find("title=") {
        let rest = attr_str[title_start + 6..].trim_start();
        if let Some(stripped) = rest.strip_prefix('"') {
            if let Some(title_end) = stripped.find('"') {
                caption = Some(stripped[..title_end].to_string());
            }
        } else if let Some(stripped) = rest.strip_prefix('\'')
            && let Some(title_end) = stripped.find('\'') {
                caption = Some(stripped[..title_end].to_string());
            }
    }

    let (width, height, id) = parse_diagram_fence_attributes(attr_str);

    let mut file = colon_file;
    if file.is_none() {
        let combined_attrs = if pre_brace_attr.is_empty() {
            attr_str.to_string()
        } else {
            format!("{pre_brace_attr} {attr_str}")
        };

        for attr in ["file=", "src=", "path="] {
            if let Some(pos) = combined_attrs.find(attr) {
                let rest = combined_attrs[pos + attr.len()..].trim_start();
                if let Some(stripped) = rest.strip_prefix('"') {
                    if let Some(end) = stripped.find('"') {
                        file = Some(stripped[..end].to_string());
                        break;
                    }
                } else if let Some(stripped) = rest.strip_prefix('\'') {
                    if let Some(end) = stripped.find('\'') {
                        file = Some(stripped[..end].to_string());
                        break;
                    }
                } else {
                    let token = rest.split_whitespace().next().unwrap_or("").trim_end_matches('}');
                    if !token.is_empty() {
                        file = Some(token.to_string());
                        break;
                    }
                }
            }
        }

        if file.is_none() {
            for search_str in [&pre_brace_attr, &attr_str] {
                let trimmed_search = search_str.trim().trim_matches(|c| c == '"' || c == '\'' || c == '{' || c == '}');
                for token in trimmed_search.split_whitespace() {
                    let clean = token.trim_matches(|c| c == '"' || c == '\'' || c == '{' || c == '}');
                    if clean.ends_with(".uxf")
                        || clean.ends_with(".umlet")
                        || clean.ends_with(".puml")
                        || clean.ends_with(".plantuml")
                        || clean.ends_with(".mmd")
                        || clean.ends_with(".mermaid")
                        || clean.ends_with(".dot")
                        || clean.ends_with(".gv")
                        || clean.ends_with(".ditaa")
                    {
                        file = Some(clean.to_string());
                        break;
                    }
                }
                if file.is_some() {
                    break;
                }
            }
        }
    }

    Some(DiagramSpec {
        diagram_type,
        caption,
        width,
        height,
        id,
        file,
    })
}

/// Render a diagram using Kroki (or mermaid.ink fallback for Mermaid) with local caching.
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
    fs::create_dir_all(&cache_dir).map_err(|e| format!("Failed to create diagram cache dir: {e}"))?;

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

    let svg_bytes = fetch_kroki(norm_type, &canonical_code, kroki_endpoint)?;

    let is_svg = svg_bytes.windows(4).any(|w| w == b"<svg")
        || svg_bytes.windows(5).any(|w| w == b"<?xml");
    if !is_svg {
        let snippet = String::from_utf8_lossy(&svg_bytes[..svg_bytes.len().min(120)]);
        return Err(format!("Diagram rendering returned non-SVG content: {snippet}"));
    }

    cache::save_to_cache(&hash, &svg_bytes)
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
