use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crate::compiler::{self, BotoxHeadingInfo};
use crate::config;

#[derive(Deserialize, Debug)]
pub struct DaemonRequest {
    #[serde(default)]
    pub id: Option<u64>,
    pub action: String, // "ping", "compile", "compile_file", "shutdown"
    pub markdown: Option<String>,
    pub file: Option<String>,
    pub resource_dir: Option<String>,
    pub source_file: Option<String>,
    pub theme: Option<String>,
    pub author: Option<String>,
    pub font: Option<String>,
    pub toc: Option<bool>,
    pub bibliography: Option<bool>,
    pub is_slides: Option<bool>,
    pub output_file: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct DaemonResponse {
    pub id: Option<u64>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_pages: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pages: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headings: Option<Vec<BotoxHeadingInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_slides: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pause_indices: Option<Vec<usize>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<&'static str>,
}

pub fn handle_daemon(_args: &[String]) {
    compiler::preheat();

    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut reader = std::io::BufReader::new(stdin.lock());
    let mut writer = std::io::BufWriter::new(stdout.lock());

    // Print daemon ready signal to stderr for process managers
    eprintln!("[BOTOX:DAEMON] ready v{}", env!("CARGO_PKG_VERSION"));

    let mut line_buffer = String::new();
    while reader.read_line(&mut line_buffer).is_ok() {
        let trimmed = line_buffer.trim();
        if trimmed.is_empty() {
            line_buffer.clear();
            continue;
        }

        let req: DaemonRequest = match serde_json::from_str(trimmed) {
            Ok(r) => r,
            Err(e) => {
                let err_resp = DaemonResponse {
                    id: None,
                    status: "error".to_string(),
                    error: Some(format!("Invalid JSON request: {e}")),
                    num_pages: None,
                    pages: None,
                    headings: None,
                    is_slides: None,
                    pause_indices: None,
                    duration_ms: None,
                    version: None,
                };
                if let Ok(json) = serde_json::to_string(&err_resp) {
                    let _ = writeln!(writer, "{json}");
                    let _ = writer.flush();
                }
                line_buffer.clear();
                continue;
            }
        };

        if req.action == "shutdown" {
            let resp = DaemonResponse {
                id: req.id,
                status: "ok".to_string(),
                error: None,
                num_pages: None,
                pages: None,
                headings: None,
                is_slides: None,
                pause_indices: None,
                duration_ms: None,
                version: None,
            };
            if let Ok(json) = serde_json::to_string(&resp) {
                let _ = writeln!(writer, "{json}");
                let _ = writer.flush();
            }
            break;
        }

        if req.action == "ping" {
            let resp = DaemonResponse {
                id: req.id,
                status: "ok".to_string(),
                error: None,
                num_pages: None,
                pages: None,
                headings: None,
                is_slides: None,
                pause_indices: None,
                duration_ms: None,
                version: Some(env!("CARGO_PKG_VERSION")),
            };
            if let Ok(json) = serde_json::to_string(&resp) {
                let _ = writeln!(writer, "{json}");
                let _ = writer.flush();
            }
            line_buffer.clear();
            continue;
        }

        let start_time = std::time::Instant::now();
        let resp = process_daemon_request(&req, start_time);
        if let Ok(json) = serde_json::to_string(&resp) {
            let _ = writeln!(writer, "{json}");
            let _ = writer.flush();
        }

        line_buffer.clear();
    }
}

fn process_daemon_request(req: &DaemonRequest, start_time: std::time::Instant) -> DaemonResponse {
    let raw_content = if let Some(ref md) = req.markdown {
        md.clone()
    } else if let Some(ref file_path) = req.file {
        match std::fs::read_to_string(file_path) {
            Ok(content) => content,
            Err(e) => {
                return DaemonResponse {
                    id: req.id,
                    status: "error".to_string(),
                    error: Some(format!("Error reading '{}': {e}", file_path)),
                    num_pages: None,
                    pages: None,
                    headings: None,
                    is_slides: None,
                    pause_indices: None,
                    duration_ms: Some(start_time.elapsed().as_secs_f64() * 1000.0),
                    version: None,
                };
            }
        }
    } else {
        return DaemonResponse {
            id: req.id,
            status: "error".to_string(),
            error: Some("Request must provide either 'markdown' or 'file'".to_string()),
            num_pages: None,
            pages: None,
            headings: None,
            is_slides: None,
            pause_indices: None,
            duration_ms: Some(start_time.elapsed().as_secs_f64() * 1000.0),
            version: None,
        };
    };

    let doc_dir_buf = req.resource_dir.as_ref().map(PathBuf::from).or_else(|| {
        req.file.as_ref().and_then(|f| Path::new(f).parent().map(|p| p.to_path_buf()))
    });
    let doc_dir = doc_dir_buf.as_deref();

    let (config_data, _) = config::BotoxConfig::load(None, doc_dir);
    let explicit_slides = req.is_slides == Some(true);
    let explicit_pdf = req.is_slides == Some(false);

    let src_path = req.file.as_deref().map(Path::new)
        .or_else(|| req.source_file.as_deref().map(Path::new));

    let (typst_markup, _) = crate::transpile_markdown_to_typst_markup(
        &raw_content,
        doc_dir,
        &config_data,
        explicit_pdf,
        explicit_slides,
        req.toc,
        req.author.as_deref(),
        req.font.as_deref(),
        req.bibliography,
        req.theme.as_deref(),
        &[],
        &[],
        src_path,
    );

    let src_filename = src_path
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("document.md");

    match compiler::compile_typst_to_pages(
        &typst_markup,
        doc_dir,
        Some(&raw_content),
        Some(src_filename),
    ) {
        Ok(output) => {
            let duration_ms = start_time.elapsed().as_secs_f64() * 1000.0;
            if let Some(ref out_file) = req.output_file {
                let p = Path::new(out_file);
                if let Some(parent) = p.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                if let Ok(json_str) = serde_json::to_string(&output) {
                    let _ = std::fs::write(p, json_str);
                }
            }
            DaemonResponse {
                id: req.id,
                status: "ok".to_string(),
                error: None,
                num_pages: Some(output.num_pages),
                pages: Some(output.pages),
                headings: Some(output.headings),
                is_slides: Some(output.is_slides),
                pause_indices: Some(output.pause_indices),
                duration_ms: Some(duration_ms),
                version: None,
            }
        }
        Err(e) => DaemonResponse {
            id: req.id,
            status: "error".to_string(),
            error: Some(e),
            num_pages: None,
            pages: None,
            headings: None,
            is_slides: None,
            pause_indices: None,
            duration_ms: Some(start_time.elapsed().as_secs_f64() * 1000.0),
            version: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daemon_ping() {
        let req = DaemonRequest {
            id: Some(10),
            action: "ping".to_string(),
            markdown: None,
            file: None,
            resource_dir: None,
            source_file: None,
            theme: None,
            author: None,
            font: None,
            toc: None,
            bibliography: None,
            is_slides: None,
            output_file: None,
        };
        let resp = process_daemon_request(&req, std::time::Instant::now());
        // For ping, handle_daemon handles it directly, but let's test process_daemon_request requirement validation
        assert_eq!(resp.status, "error");
        assert!(resp.error.unwrap().contains("must provide either"));
    }

    #[test]
    fn test_daemon_compile_memory_markdown() {
        let req = DaemonRequest {
            id: Some(42),
            action: "compile".to_string(),
            markdown: Some("# Daemon Heading\n\nFast preview test paragraph.\n".to_string()),
            file: None,
            resource_dir: None,
            source_file: Some("live.md".to_string()),
            theme: Some("academic".to_string()),
            author: None,
            font: None,
            toc: None,
            bibliography: None,
            is_slides: Some(false),
            output_file: None,
        };
        let resp = process_daemon_request(&req, std::time::Instant::now());
        assert_eq!(resp.status, "ok");
        assert_eq!(resp.id, Some(42));
        assert_eq!(resp.num_pages, Some(1));
        assert!(resp.pages.is_some());
        let pages = resp.pages.unwrap();
        assert_eq!(pages.len(), 1);
        assert!(pages[0].contains("<svg"));
        assert!(resp.duration_ms.unwrap() > 0.0);
    }

    #[test]
    fn test_daemon_compile_slides() {
        let req = DaemonRequest {
            id: Some(101),
            action: "compile".to_string(),
            markdown: Some("---\nslides: true\n---\n\n# Slide One\n\nBullet 1\n".to_string()),
            file: None,
            resource_dir: None,
            source_file: None,
            theme: None,
            author: None,
            font: None,
            toc: None,
            bibliography: None,
            is_slides: None,
            output_file: None,
        };
        let resp = process_daemon_request(&req, std::time::Instant::now());
        assert_eq!(resp.status, "ok");
        assert_eq!(resp.is_slides, Some(true));
        assert_eq!(resp.num_pages, Some(1));
    }

    #[test]
    fn test_daemon_compile_syntax_error_handled() {
        let req = DaemonRequest {
            id: Some(999),
            action: "compile".to_string(),
            markdown: Some("# Title\n\nSee @fig-nonexistent-diagram-label-xyz for details.\n".to_string()),
            file: None,
            resource_dir: None,
            source_file: Some("err.md".to_string()),
            theme: None,
            author: None,
            font: None,
            toc: None,
            bibliography: None,
            is_slides: None,
            output_file: None,
        };
        let resp = process_daemon_request(&req, std::time::Instant::now());
        assert_eq!(resp.status, "error");
        assert_eq!(resp.id, Some(999));
        assert!(resp.error.is_some());
    }
}

