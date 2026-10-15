use std::sync::LazyLock;
use std::time::Duration;

static KROKI_AGENT: LazyLock<ureq::Agent> = LazyLock::new(|| {
    ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_millis(4000)))
        .build()
        .new_agent()
});

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
        let target_url = format!("{}/{}/svg", endpoint.trim_end_matches('/'), norm_type);
        let kroki_res = KROKI_AGENT
            .post(&target_url)
            .header("Content-Type", "text/plain; charset=utf-8")
            .header("User-Agent", "botox/0.1")
            .send(canonical_code);

        match kroki_res {
            Ok(mut resp) => {
                let status = resp.status();
                if status.is_success() {
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
                } else {
                    errors.push(format!("{endpoint}: HTTP {status}"));
                }
            }
            Err(e) => {
                errors.push(format!("{endpoint}: connection error ({e})"));
            }
        }
    }

    Err(format!(
        "Kroki diagram rendering failed for {norm_type}: {}",
        errors.join("; ")
    ))
}


