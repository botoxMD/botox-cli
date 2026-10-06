use std::path::{Path, PathBuf};
use typst_as_lib::file_resolver::FileResolver;
use typst_as_lib::TypstEngine;

#[derive(Debug, Clone)]
struct BotoxFileResolver {
    resource_dir: PathBuf,
}

impl FileResolver for BotoxFileResolver {
    fn resolve_binary(
        &self,
        id: typst::syntax::FileId,
    ) -> typst::diag::FileResult<std::borrow::Cow<'_, typst::foundations::Bytes>> {
        let vpath = id.vpath();
        let rel_path = Path::new(vpath.get_without_slash());
        let abs_candidate = Path::new(vpath.get_with_slash());

        let candidates = [
            self.resource_dir.join(rel_path),
            abs_candidate.to_path_buf(),
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(rel_path),
        ];

        for path in &candidates {
            if path.is_file() {
                if let Ok(bytes) = std::fs::read(path) {
                    return Ok(std::borrow::Cow::Owned(typst::foundations::Bytes::new(bytes)));
                }
            }
        }

        Err(typst::diag::FileError::NotFound(self.resource_dir.join(rel_path)))
    }

    fn resolve_source(
        &self,
        id: typst::syntax::FileId,
    ) -> typst::diag::FileResult<std::borrow::Cow<'_, typst::syntax::Source>> {
        let rel_path = Path::new(id.vpath().get_without_slash());
        Err(typst::diag::FileError::NotFound(self.resource_dir.join(rel_path)))
    }
}

#[derive(serde::Serialize)]
pub struct BotoxPagesOutput {
    pub num_pages: usize,
    pub pages: Vec<String>,
}

pub fn compile_typst(
    typst_markup: &str,
    output_path: &Path,
    resource_dir: Option<&Path>,
) -> Result<(), String> {
    let mut fonts: Vec<typst::text::Font> = Vec::new();

    // 1. Embedded fonts (New Computer Modern, Math, etc.)
    for font_bytes in typst_assets::fonts() {
        let bytes = typst::foundations::Bytes::new(font_bytes);
        for font in typst::text::Font::iter(bytes) {
            fonts.push(font);
        }
    }

    // 2. System fonts (discovers monospace, sans-serif, etc.)
    let mut db = fontdb::Database::new();
    db.load_system_fonts();
    for face in db.faces() {
        if let fontdb::Source::File(ref path) = face.source {
            if let Ok(data) = std::fs::read(path) {
                let bytes = typst::foundations::Bytes::new(data);
                if let Some(font) = typst::text::Font::new(bytes, face.index) {
                    fonts.push(font);
                }
            }
        }
    }

    let res_dir = resource_dir
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let engine = TypstEngine::builder()
        .main_file(typst_markup)
        .fonts(fonts)
        .add_file_resolver(BotoxFileResolver { resource_dir: res_dir })
        .build();

    let compilation_result = engine.compile();
    let doc: typst_layout::PagedDocument = compilation_result.output.map_err(|e| {
        let _ = std::fs::write("/tmp/debug_fail.typ", typst_markup);
        format!("Typst compilation error: {e:?}\n(Dumped markup to /tmp/debug_fail.typ)")
    })?;

    if let Some(parent) = output_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create output directory: {e}"))?;
        }
    }

    let ext = output_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

fn escape_xml_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

fn extract_frame_text(
    frame: &typst_library::layout::Frame,
    parent_ts: typst_library::layout::Transform,
    out: &mut Vec<(f64, f64, f64, f64, String, bool)>,
) {
    for (point, item) in frame.items() {
        let item_ts = parent_ts.pre_concat(typst_library::layout::Transform::translate(point.x, point.y));
        match item {
            typst_library::layout::FrameItem::Text(text_item) => {
                let x = item_ts.tx.to_pt();
                let y = item_ts.ty.to_pt();
                let size = (text_item.size.to_pt() * item_ts.sy.get()).abs();
                let width = (text_item.width().to_pt() * item_ts.sx.get()).abs();
                let is_heading = size >= 12.8;
                if !text_item.text.is_empty() {
                    out.push((x, y, size, width, text_item.text.to_string(), is_heading));
                }
            }
            typst_library::layout::FrameItem::Group(group) => {
                let group_ts = item_ts.pre_concat(group.transform);
                extract_frame_text(&group.frame, group_ts, out);
            }
            _ => {}
        }
    }
}

pub fn make_page_svg_with_text(page: &typst_layout::Page, opts: &typst_svg::SvgOptions) -> String {
    let mut base_svg = typst_svg::svg(page, opts);
    let mut text_runs = Vec::new();
    extract_frame_text(&page.frame, typst_library::layout::Transform::identity(), &mut text_runs);

    if text_runs.is_empty() {
        return base_svg;
    }

    let mut text_layer = String::from(r#"<g class="selectable-text" style="cursor: text; fill: transparent; stroke: none; fill-opacity: 0;">"#);
    for (x, y, size, width, text, is_heading) in text_runs {
        let escaped = escape_xml_text(&text);
        let heading_attr = if is_heading { r#" data-heading="true""# } else { "" };
        if width > 0.0 {
            use std::fmt::Write;
            let _ = write!(
                text_layer,
                r#"<text x="{x:.2}" y="{y:.2}" font-size="{size:.2}" data-size="{size:.2}" textLength="{width:.2}" lengthAdjust="spacingAndGlyphs"{heading_attr}>{escaped}</text>"#
            );
        } else {
            use std::fmt::Write;
            let _ = write!(
                text_layer,
                r#"<text x="{x:.2}" y="{y:.2}" font-size="{size:.2}" data-size="{size:.2}"{heading_attr}>{escaped}</text>"#
            );
        }
    }
    text_layer.push_str("</g>");

    if let Some(pos) = base_svg.rfind("</svg>") {
        base_svg.insert_str(pos, &text_layer);
    } else {
        base_svg.push_str(&text_layer);
    }

    base_svg
}

    match ext.as_str() {
        "svg" => {
            let svg_opts = typst_svg::SvgOptions::default();
            let svg_content = if doc.pages().len() <= 1 {
                if let Some(first_page) = doc.pages().first() {
                    make_page_svg_with_text(first_page, &svg_opts)
                } else {
                    String::from("<svg></svg>")
                }
            } else {
                typst_svg::svg_merged(&doc, &svg_opts, typst::layout::Abs::pt(20.0))
            };
            std::fs::write(output_path, svg_content)
                .map_err(|e| format!("Failed to write output SVG file '{}': {e}", output_path.display()))?;
        }
        "json" => {
            let svg_opts = typst_svg::SvgOptions::default();
            let pages: Vec<String> = doc
                .pages()
                .iter()
                .map(|p| make_page_svg_with_text(p, &svg_opts))
                .collect();
            let output_struct = BotoxPagesOutput {
                num_pages: pages.len(),
                pages,
            };
            let json_str = serde_json::to_string(&output_struct)
                .map_err(|e| format!("Failed to serialize pages to JSON: {e}"))?;
            std::fs::write(output_path, json_str)
                .map_err(|e| format!("Failed to write output JSON file '{}': {e}", output_path.display()))?;
        }
        _ => {
            let pdf_bytes = typst_pdf::pdf(&doc, &typst_pdf::PdfOptions::default())
                .map_err(|e| format!("PDF export error: {e:?}"))?;
            std::fs::write(output_path, pdf_bytes)
                .map_err(|e| format!("Failed to write output file '{}': {e}", output_path.display()))?;
        }
    }

    Ok(())
}

#[allow(dead_code)]
pub fn compile_typst_to_pdf(
    typst_markup: &str,
    output_path: &Path,
    resource_dir: Option<&Path>,
) -> Result<(), String> {
    compile_typst(typst_markup, output_path, resource_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_typst_to_svg_and_json() {
        let markup = "= Hello World\nThis is pure vector text.\n$x^2 + y^2 = z^2$";
        let tmp_svg = std::env::temp_dir().join("test_botox_vector.svg");
        let tmp_json = std::env::temp_dir().join("test_botox_pages.json");

        assert!(compile_typst(markup, &tmp_svg, None).is_ok());
        let svg_str = std::fs::read_to_string(&tmp_svg).expect("Read SVG");
        assert!(svg_str.contains("<svg"));
        assert!(svg_str.contains("Hello World") || svg_str.contains("<path") || svg_str.contains("<text"));

        assert!(compile_typst(markup, &tmp_json, None).is_ok());
        let json_str = std::fs::read_to_string(&tmp_json).expect("Read JSON");
        assert!(json_str.contains("\"num_pages\":1"));
        assert!(json_str.contains("\"pages\":[\"<svg"));

        let _ = std::fs::remove_file(tmp_svg);
        let _ = std::fs::remove_file(tmp_json);
    }
}
