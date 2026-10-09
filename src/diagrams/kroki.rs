use std::time::Duration;

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

/// Request rendering from a Kroki endpoint.
pub fn fetch_kroki(
    norm_type: &str,
    canonical_code: &str,
    kroki_endpoint: Option<&str>,
) -> Result<Vec<u8>, String> {
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
        .send(canonical_code);

    match kroki_res {
        Ok(mut resp) => {
            let status = resp.status();
            if !status.is_success() {
                if norm_type == "mermaid" {
                    // Try mermaid.ink fallback
                    fetch_mermaid_ink(canonical_code)
                        .map_err(|e| format!("Kroki returned HTTP {status}; fallback failed: {e}"))
                } else {
                    Err(format!("Kroki returned HTTP {status}"))
                }
            } else {
                let bytes = resp
                    .body_mut()
                    .read_to_vec()
                    .map_err(|e| format!("Failed to read response from Kroki: {e}"))?;
                if bytes.is_empty() {
                    return Err("Kroki returned an empty SVG response".to_string());
                }
                Ok(bytes)
            }
        }
        Err(e) => {
            if norm_type == "mermaid" {
                // Kroki connection failed or timed out, attempt mermaid.ink fallback
                fetch_mermaid_ink(canonical_code)
                    .map_err(|ink_err| format!("Kroki unreachable ({e}); mermaid.ink failed ({ink_err})"))
            } else {
                Err(format!("Kroki connection failed: {e}"))
            }
        }
    }
}
