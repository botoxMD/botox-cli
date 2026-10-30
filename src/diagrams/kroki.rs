use std::sync::LazyLock;
use std::time::Duration;

static KROKI_AGENT: LazyLock<ureq::Agent> = LazyLock::new(|| {
    ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_millis(4000)))
        .build()
        .new_agent()
});

static KROKI_POST_AGENT: LazyLock<ureq::Agent> = LazyLock::new(|| {
    ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_millis(4000)))
        .max_redirects(0)
        .http_status_as_error(false)
        .build()
        .new_agent()
});

fn resolve_redirect_url(current_url: &str, location: &str) -> String {
    let loc = location.trim();
    if loc.starts_with("http://") || loc.starts_with("https://") {
        return loc.to_string();
    }
    if loc.starts_with('/')
        && let Some(scheme_end) = current_url.find("://")
    {
        let after_scheme = &current_url[scheme_end + 3..];
        let host_end = after_scheme.find('/').unwrap_or(after_scheme.len());
        let origin = &current_url[..scheme_end + 3 + host_end];
        return format!("{origin}{loc}");
    }
    loc.to_string()
}

/// Fetch a Mermaid SVG directly from mermaid.ink as a companion fallback.
pub fn fetch_mermaid_ink(canonical_code: &str) -> Result<Vec<u8>, String> {
    use base64::prelude::*;
    let encoded = BASE64_URL_SAFE.encode(canonical_code.as_bytes());
    let url = format!("https://mermaid.ink/svg/{encoded}");

    let mut resp = KROKI_AGENT
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

/// Request rendering from a user-configured Kroki endpoint.
/// By default, no remote endpoints are contacted unless configured.
pub fn fetch_kroki(
    norm_type: &str,
    canonical_code: &str,
    kroki_endpoint: Option<&str>,
) -> Result<Vec<u8>, String> {
    let custom_env = std::env::var("KROKI_ENDPOINT").ok().or_else(|| std::env::var("KROKI_URL").ok());
    let explicit_endpoint = kroki_endpoint.map(|s| s.trim().to_string()).or(custom_env);

    let endpoints: Vec<String> = match explicit_endpoint {
        Some(ep) if !ep.trim().is_empty() => {
            ep.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        }
        _ => Vec::new(),
    };

    if endpoints.is_empty() {
        return Err(
            "Kroki endpoint is not configured. Configure a Kroki endpoint (e.g. 'kroki_url: https://kroki.io' or a self-hosted instance 'http://localhost:8000') in frontmatter, botox.yaml, or via KROKI_ENDPOINT."
                .to_string(),
        );
    }

    let mut errors: Vec<String> = Vec::new();

    for endpoint in &endpoints {
        let mut current_url = format!("{}/{}/svg", endpoint.trim_end_matches('/'), norm_type);
        let mut redirects = 0;

        loop {
            let kroki_res = KROKI_POST_AGENT
                .post(&current_url)
                .header("Content-Type", "text/plain; charset=utf-8")
                .header("User-Agent", "botox/0.1")
                .send(canonical_code);

            match kroki_res {
                Ok(mut resp) => {
                    let status = resp.status();
                    let code = status.as_u16();
                    if matches!(code, 301 | 302 | 303 | 307 | 308) {
                        if redirects >= 5 {
                            errors.push(format!("{endpoint}: too many redirects (last HTTP {status} at {current_url})"));
                            break;
                        }
                        if let Some(loc) = resp.headers().get("location").and_then(|v| v.to_str().ok()) {
                            current_url = resolve_redirect_url(&current_url, loc);
                            redirects += 1;
                            continue;
                        } else {
                            errors.push(format!("{endpoint}: HTTP {status} redirect without Location header"));
                            break;
                        }
                    } else if status.is_success() {
                        match resp.body_mut().read_to_vec() {
                            Ok(bytes) => {
                                if !bytes.is_empty() {
                                    return Ok(bytes);
                                } else {
                                    errors.push(format!("{endpoint}: empty SVG response"));
                                }
                            }
                            Err(e) => {
                                errors.push(format!("{endpoint}: failed reading response ({e})"));
                            }
                        }
                        break;
                    } else {
                        errors.push(format!("{endpoint}: HTTP {status}"));
                        break;
                    }
                }
                Err(e) => {
                    errors.push(format!("{endpoint}: connection error ({e})"));
                    break;
                }
            }
        }
    }

    Err(format!(
        "Kroki diagram rendering failed for {norm_type}: {}",
        errors.join("; ")
    ))
}


