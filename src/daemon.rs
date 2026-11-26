use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crate::compiler::{self, BotoxHeadingInfo};
use crate::sync_map::{ClickJumpResult, CursorJumpResult, SyncIndex};

#[derive(Deserialize, Debug, Default)]
pub struct DaemonRequest {
    #[serde(default)]
    pub id: Option<u64>,
    pub action: String, // "ping", "compile", "compile_file", "jump_from_click", "jump_from_cursor", "shutdown"
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
    pub page: Option<usize>,
    pub x_pt: Option<f64>,
    pub y_pt: Option<f64>,
    pub line: Option<usize>,
    pub col: Option<usize>,
}

#[derive(Serialize, Debug, Default)]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jump: Option<ClickJumpResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<CursorJumpResult>,
}

pub fn handle_daemon(_args: &[String]) {
    compiler::preheat();

    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut reader = std::io::BufReader::new(stdin.lock());
    let mut writer = std::io::BufWriter::new(stdout.lock());
    let mut sync_cache: HashMap<String, SyncIndex> = HashMap::new();

    // Print daemon ready signal to stderr for process managers
    eprintln!("[BOTOX:DAEMON] ready v{}", env!("CARGO_PKG_VERSION"));

    let mut line_buffer = String::new();
    loop {
        line_buffer.clear();
        match reader.read_line(&mut line_buffer) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }
        let trimmed = line_buffer.trim();
        if trimmed.is_empty() {
            continue;
        }

        let req: DaemonRequest = match serde_json::from_str(trimmed) {
            Ok(r) => r,
            Err(e) => {
                let err_resp = DaemonResponse {
                    status: "error".to_string(),
                    error: Some(format!("Invalid JSON request: {e}")),
                    ..Default::default()
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
                ..Default::default()
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
                version: Some(env!("CARGO_PKG_VERSION")),
                ..Default::default()
            };
            if let Ok(json) = serde_json::to_string(&resp) {
                let _ = writeln!(writer, "{json}");
                let _ = writer.flush();
            }
            line_buffer.clear();
            continue;
        }

        if req.action == "jump_from_click" {
            let start_time = std::time::Instant::now();
            let key = req.source_file.as_deref().or(req.file.as_deref()).unwrap_or("__default__");
            let jump = sync_cache.get(key).and_then(|idx| {
                idx.jump_from_click(req.page.unwrap_or(1), req.x_pt.unwrap_or(0.0), req.y_pt.unwrap_or(0.0))
            });
            let resp = DaemonResponse {
                id: req.id,
                status: "ok".to_string(),
                jump,
                duration_ms: Some(start_time.elapsed().as_secs_f64() * 1000.0),
                ..Default::default()
            };
            if let Ok(json) = serde_json::to_string(&resp) {
                let _ = writeln!(writer, "{json}");
                let _ = writer.flush();
            }
            line_buffer.clear();
            continue;
        }

        if req.action == "jump_from_cursor" {
            let start_time = std::time::Instant::now();
            let key = req.source_file.as_deref().or(req.file.as_deref()).unwrap_or("__default__");
            let cursor = sync_cache.get(key).and_then(|idx| {
                idx.jump_from_cursor(req.line.unwrap_or(1), req.col.unwrap_or(1))
            });
            let resp = DaemonResponse {
                id: req.id,
                status: "ok".to_string(),
                cursor,
                duration_ms: Some(start_time.elapsed().as_secs_f64() * 1000.0),
                ..Default::default()
            };
            if let Ok(json) = serde_json::to_string(&resp) {
                let _ = writeln!(writer, "{json}");
                let _ = writer.flush();
            }
            line_buffer.clear();
            continue;
        }

        let start_time = std::time::Instant::now();
        let (resp, sync_idx) = process_daemon_request_with_sync(&req, start_time);
        if let Some(idx) = sync_idx {
            let key = req
                .source_file
                .clone()
                .or_else(|| req.file.clone())
                .unwrap_or_else(|| "__default__".to_string());
            if sync_cache.len() >= 8 && !sync_cache.contains_key(&key) {
                sync_cache.clear();
            }
            sync_cache.insert(key, idx);
        }
        if let Ok(json) = serde_json::to_string(&resp) {
            let _ = writeln!(writer, "{json}");
            let _ = writer.flush();
        }

        line_buffer.clear();
    }
}

#[allow(dead_code)]
fn process_daemon_request(req: &DaemonRequest, start_time: std::time::Instant) -> DaemonResponse {
    process_daemon_request_with_sync(req, start_time).0
}

fn process_daemon_request_with_sync(
    req: &DaemonRequest,
    start_time: std::time::Instant,
) -> (DaemonResponse, Option<SyncIndex>) {
    let doc_dir_buf = req.resource_dir.as_ref().map(PathBuf::from).or_else(|| {
        req.file.as_ref().and_then(|f| Path::new(f).parent().map(|p| p.to_path_buf()))
    });
    let doc_dir = doc_dir_buf.as_deref();

    let source = if let Some(ref md) = req.markdown {
        crate::engine::DocumentSource::Memory {
            content: md,
            source_filename: req.source_file.as_deref().or(req.file.as_deref()),
            resource_dir: doc_dir,
        }
    } else if let Some(ref file_path) = req.file {
        let p = Path::new(file_path);
        if !compiler::is_path_allowed(p, None) {
            return (
                DaemonResponse {
                    id: req.id,
                    status: "error".to_string(),
                    error: Some(format!("Access denied to file outside allowed directory: '{file_path}'")),
                    duration_ms: Some(start_time.elapsed().as_secs_f64() * 1000.0),
                    ..Default::default()
                },
                None,
            );
        }
        crate::engine::DocumentSource::File {
            path: p,
            resource_dir: doc_dir,
        }
    } else {
        return (
            DaemonResponse {
                id: req.id,
                status: "error".to_string(),
                error: Some("Request must provide either 'markdown' or 'file'".to_string()),
                duration_ms: Some(start_time.elapsed().as_secs_f64() * 1000.0),
                ..Default::default()
            },
            None,
        );
    };

    let target = match req.is_slides {
        Some(true) => crate::engine::TargetFormat::Slides,
        Some(false) => crate::engine::TargetFormat::Pdf,
        None => crate::engine::TargetFormat::Auto,
    };

    let opts = crate::engine::PublishOptions {
        target,
        theme: req.theme.as_deref(),
        author: req.author.as_deref(),
        font: req.font.as_deref(),
        toc: req.toc,
        bibliography: req.bibliography,
        ..Default::default()
    };

    match crate::engine::publish_to_pages_with_sync(&source, &opts) {
        Ok((output, sync_idx)) => {
            let duration_ms = start_time.elapsed().as_secs_f64() * 1000.0;
            if let Some(ref out_file) = req.output_file {
                let p = Path::new(out_file);
                let check_target = if p.exists() { p } else { p.parent().unwrap_or(Path::new(".")) };
                if compiler::is_path_allowed(check_target, doc_dir) {
                    if let Some(parent) = p.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    if let Ok(json_str) = serde_json::to_string(&output) {
                        let _ = std::fs::write(p, json_str);
                    }
                }
            }
            (
                DaemonResponse {
                    id: req.id,
                    status: "ok".to_string(),
                    num_pages: Some(output.num_pages),
                    pages: Some(output.pages),
                    headings: Some(output.headings),
                    is_slides: Some(output.is_slides),
                    pause_indices: Some(output.pause_indices),
                    duration_ms: Some(duration_ms),
                    ..Default::default()
                },
                sync_idx,
            )
        }
        Err(e) => (
            DaemonResponse {
                id: req.id,
                status: "error".to_string(),
                error: Some(e),
                duration_ms: Some(start_time.elapsed().as_secs_f64() * 1000.0),
                ..Default::default()
            },
            None,
        ),
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
            ..Default::default()
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
            source_file: Some("live.md".to_string()),
            theme: Some("academic".to_string()),
            is_slides: Some(false),
            ..Default::default()
        };
        let (resp, sync_idx) = process_daemon_request_with_sync(&req, std::time::Instant::now());
        assert_eq!(resp.status, "ok");
        assert_eq!(resp.id, Some(42));
        assert_eq!(resp.num_pages, Some(1));
        assert!(resp.pages.is_some());
        let pages = resp.pages.unwrap();
        assert_eq!(pages.len(), 1);
        assert!(pages[0].contains("<svg"));
        assert!(resp.duration_ms.unwrap() > 0.0);

        let idx = sync_idx.expect("SyncIndex should be built on daemon compile");
        assert!(idx.glyph_count() > 20, "expected mapped glyphs, got {}", idx.glyph_count());
        let c = idx.jump_from_cursor(3, 6).expect("cursor jump on line 3 col 6");
        assert_eq!(c.page, 1);
        let back = idx.jump_from_click(c.page as usize, c.x_pt as f64 + 1.0, c.y_pt as f64 + 2.0)
            .expect("click jump back to source");
        assert_eq!(back.line, 3);
        assert!((back.col as i32 - 6).abs() <= 2, "expected col ~6, got {}", back.col);
    }

    #[test]
    fn test_daemon_compile_slides() {
        let req = DaemonRequest {
            id: Some(101),
            action: "compile".to_string(),
            markdown: Some("---\nslides: true\n---\n\n# Slide One\n\nBullet 1\n".to_string()),
            ..Default::default()
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
            source_file: Some("err.md".to_string()),
            ..Default::default()
        };
        let resp = process_daemon_request(&req, std::time::Instant::now());
        assert_eq!(resp.status, "error");
        assert_eq!(resp.id, Some(999));
        assert!(resp.error.is_some());
    }

    #[test]
    fn test_daemon_unauthorized_file_access() {
        let req = DaemonRequest {
            id: Some(101),
            action: "compile_file".to_string(),
            file: Some("/etc/shadow".to_string()),
            ..Default::default()
        };
        let resp = process_daemon_request(&req, std::time::Instant::now());
        assert_eq!(resp.status, "error");
        assert!(resp.error.unwrap().contains("Access denied"));
    }
}

