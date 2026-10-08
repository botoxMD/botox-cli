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
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".cache").join("botox").join("diagrams");
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
    let canonical = canonicalize_diagram_source(diagram_type, code);
    let mut hasher = Sha256::new();
    hasher.update(diagram_type.trim().to_lowercase().as_bytes());
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
    if cache_file.is_file() {
        if let Ok(meta) = fs::metadata(&cache_file) {
            if meta.len() > 0 {
                return Ok(cache_file);
            }
        }
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

    fs::write(&cache_file, svg_bytes)
        .map_err(|e| format!("Failed to write diagram to cache: {e}"))?;

    Ok(cache_file)
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
}
