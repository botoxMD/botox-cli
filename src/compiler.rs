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

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct BotoxHeadingInfo {
    pub page_index: usize,
    pub text: String,
    pub y_ratio: f64,
}

#[derive(serde::Serialize)]
pub struct BotoxPagesOutput {
    pub num_pages: usize,
    pub pages: Vec<String>,
    pub headings: Vec<BotoxHeadingInfo>,
}

static EMBEDDED_FONTS: std::sync::LazyLock<Vec<typst::text::Font>> = std::sync::LazyLock::new(|| {
    let mut fonts = Vec::new();
    for font_bytes in typst_assets::fonts() {
        let bytes = typst::foundations::Bytes::new(font_bytes);
        for font in typst::text::Font::iter(bytes) {
            fonts.push(font);
        }
    }
    fonts
});

fn is_embedded_font(name: &str) -> bool {
    let n = name.trim().trim_matches('"').trim_matches('\'').to_lowercase();
    n == "new computer modern"
        || n == "new computer modern math"
        || n == "dejavu sans mono"
        || n == "libertinus serif"
        || n.is_empty()
}

fn load_needed_fonts(typst_markup: &str) -> Vec<typst::text::Font> {
    let mut fonts = EMBEDDED_FONTS.clone();

    // Check if custom fonts are referenced in typst_markup:
    // e.g. `font: "..."` or `font: ("...", "...")`
    let mut custom_fonts: Vec<String> = Vec::new();
    let mut idx = 0;
    while let Some(pos) = typst_markup[idx..].find("font:") {
        let start = idx + pos + 5;
        let rest = &typst_markup[start..];
        let end = rest.find(['\n', ';']).unwrap_or(rest.len().min(120));
        let slice = &rest[..end];

        let mut in_quote = false;
        let mut quote_char = '"';
        let mut cur = String::new();
        for ch in slice.chars() {
            if in_quote {
                if ch == quote_char {
                    in_quote = false;
                    let trimmed = cur.trim();
                    if !is_embedded_font(trimmed)
                        && !custom_fonts.iter().any(|f| f.eq_ignore_ascii_case(trimmed))
                    {
                        custom_fonts.push(trimmed.to_string());
                    }
                    cur.clear();
                } else {
                    cur.push(ch);
                }
            } else if ch == '"' || ch == '\'' {
                in_quote = true;
                quote_char = ch;
            }
        }
        idx = start + end;
    }

    if !custom_fonts.is_empty() {
        let mut db = fontdb::Database::new();
        db.load_system_fonts();
        for face in db.faces() {
            let matches_custom = face.families.iter().any(|(fam, _)| {
                custom_fonts.iter().any(|cf| fam.eq_ignore_ascii_case(cf))
            }) || custom_fonts
                .iter()
                .any(|cf| face.post_script_name.eq_ignore_ascii_case(cf));

            if matches_custom {
                if let fontdb::Source::File(ref path) = face.source {
                    if let Ok(data) = std::fs::read(path) {
                        let bytes = typst::foundations::Bytes::new(data);
                        if let Some(font) = typst::text::Font::new(bytes, face.index) {
                            fonts.push(font);
                        }
                    }
                }
            }
        }
    }

    fonts
}

fn format_compilation_error(e: &typst_as_lib::TypstAsLibError) -> String {
    match e {
        typst_as_lib::TypstAsLibError::TypstSource(diags) => {
            let mut msgs = Vec::new();
            for d in diags {
                let mut msg = d.message.to_string();
                if !d.hints.is_empty() {
                    let hints: Vec<String> = d.hints.iter().map(|h| h.v.to_string()).collect();
                    msg.push_str(&format!(" (hint: {})", hints.join("; ")));
                }
                msgs.push(msg);
            }
            if msgs.is_empty() {
                "Compilation failed with unspecified Typst error".to_string()
            } else {
                msgs.join("\n")
            }
        }
        _ => format!("Compilation error: {e}"),
    }
}

pub fn compile_typst(
    typst_markup: &str,
    output_path: &Path,
    resource_dir: Option<&Path>,
) -> Result<(), String> {
    let fonts = load_needed_fonts(typst_markup);

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
        format_compilation_error(&e)
    })?;

    if output_path.as_os_str() == "-" {
        let pdf_bytes = typst_pdf::pdf(&doc, &typst_pdf::PdfOptions::default())
            .map_err(|e| format!("PDF export error: {e:?}"))?;
        use std::io::Write;
        let mut stdout = std::io::stdout().lock();
        stdout.write_all(&pdf_bytes)
            .map_err(|e| format!("Failed to write PDF to stdout: {e}"))?;
        stdout.flush()
            .map_err(|e| format!("Failed to flush stdout: {e}"))?;
        return Ok(());
    }

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

    fn xml_escape(s: &str) -> String {
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

    fn extract_text_layer(
        frame: &typst_library::layout::Frame,
        parent_ts: typst_library::layout::Transform,
        out: &mut String,
    ) {
        for (point, item) in frame.items() {
            let item_ts = parent_ts.pre_concat(typst_library::layout::Transform::translate(point.x, point.y));
            match item {
                typst_library::layout::FrameItem::Text(text_item) => {
                    let text = text_item.text.as_str();
                    if text.trim().is_empty() {
                        continue;
                    }
                    let x = item_ts.tx.to_pt();
                    let y = item_ts.ty.to_pt();
                    let size = (text_item.size.to_pt() * item_ts.sy.get()).abs();
                    let width = (text_item.width().to_pt() * item_ts.sx.get()).abs();
                    let escaped = xml_escape(text);

                    use std::fmt::Write;
                    let _ = write!(
                        out,
                        r#"<text x="{x:.2}" y="{y:.2}" font-size="{size:.2}pt" textLength="{width:.2}" lengthAdjust="spacingAndGlyphs" fill="transparent" stroke="none" style="cursor: text; user-select: text;">{escaped}</text>"#
                    );
                }
                typst_library::layout::FrameItem::Group(group) => {
                    let group_ts = item_ts.pre_concat(group.transform);
                    extract_text_layer(&group.frame, group_ts, out);
                }
                _ => {}
            }
        }
    }

    fn inject_text_layer(mut svg: String, frame: &typst_library::layout::Frame) -> String {
        let mut text_elements = String::new();
        extract_text_layer(frame, typst_library::layout::Transform::identity(), &mut text_elements);
        if !text_elements.is_empty() {
            if let Some(idx) = svg.rfind("</svg>") {
                svg.insert_str(
                    idx,
                    &format!(r#"<g class="botox-text-layer" style="user-select: text; -webkit-user-select: text; pointer-events: auto;">{text_elements}</g>"#),
                );
            }
        }
        svg
    }

    fn extract_frame_headings(
        frame: &typst_library::layout::Frame,
        parent_ts: typst_library::layout::Transform,
        page_index: usize,
        page_height: f64,
        out: &mut Vec<BotoxHeadingInfo>,
    ) {
        for (point, item) in frame.items() {
            let item_ts = parent_ts.pre_concat(typst_library::layout::Transform::translate(point.x, point.y));
            match item {
                typst_library::layout::FrameItem::Text(text_item) => {
                    let y = item_ts.ty.to_pt();
                    let size = (text_item.size.to_pt() * item_ts.sy.get()).abs();
                    let is_heading = size >= 12.8;
                    let trimmed = text_item.text.trim();
                    if is_heading && !trimmed.is_empty() {
                        let cur_ratio = (y / page_height).clamp(0.0, 1.0);
                        if let Some(last) = out.last_mut() {
                            if last.page_index == page_index && (last.y_ratio - cur_ratio).abs() < 0.008 {
                                last.text.push(' ');
                                last.text.push_str(trimmed);
                            } else {
                                out.push(BotoxHeadingInfo {
                                    page_index,
                                    text: trimmed.to_string(),
                                    y_ratio: cur_ratio,
                                });
                            }
                        } else {
                            out.push(BotoxHeadingInfo {
                                page_index,
                                text: trimmed.to_string(),
                                y_ratio: cur_ratio,
                            });
                        }
                    }
                }
                typst_library::layout::FrameItem::Group(group) => {
                    let group_ts = item_ts.pre_concat(group.transform);
                    extract_frame_headings(&group.frame, group_ts, page_index, page_height, out);
                }
                _ => {}
            }
        }
    }

    match ext.as_str() {
        "svg" => {
            let svg_opts = typst_svg::SvgOptions::default();
            let svg_content = if doc.pages().len() <= 1 {
                if let Some(first_page) = doc.pages().first() {
                    inject_text_layer(typst_svg::svg(first_page, &svg_opts), &first_page.frame)
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
                .map(|p| inject_text_layer(typst_svg::svg(p, &svg_opts), &p.frame))
                .collect();

            let mut headings = Vec::new();
            for (page_idx, page) in doc.pages().iter().enumerate() {
                let page_height = page.frame.height().to_pt().max(1.0);
                extract_frame_headings(
                    &page.frame,
                    typst_library::layout::Transform::identity(),
                    page_idx,
                    page_height,
                    &mut headings,
                );
            }

            let output_struct = BotoxPagesOutput {
                num_pages: pages.len(),
                pages,
                headings,
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

#[allow(dead_code)]
pub fn compile_typst_to_pdf_bytes(
    typst_markup: &str,
    resource_dir: Option<&Path>,
) -> Result<Vec<u8>, String> {
    let fonts = load_needed_fonts(typst_markup);

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
        format_compilation_error(&e)
    })?;

    typst_pdf::pdf(&doc, &typst_pdf::PdfOptions::default())
        .map_err(|e| format!("PDF export error: {e:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_typst_to_pdf_bytes() {
        let markup = "= Test PDF Stream\nPipeline verification";
        let res = compile_typst_to_pdf_bytes(markup, None);
        assert!(res.is_ok(), "PDF compile error: {:?}", res.err());
        let bytes = res.unwrap();
        assert!(bytes.starts_with(b"%PDF-"), "Stream must begin with %PDF- header");
    }

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

    #[test]
    fn test_typst_escapes() {
        let markup = r#"
= Test Escapes
Multiplicity 1..\*
Email user\@example.com
Issue \#42
Brackets \[test\]
Dollar \$100
Underscore \_foo\_
Tilde \~tilde
Slash \/\/comment
Backslash \\
Angle \<stdio.h\>
"#;
        let tmp_json = std::env::temp_dir().join("test_botox_escapes.json");
        let res = compile_typst(markup, &tmp_json, None);
        assert!(res.is_ok(), "Typst compile error: {:?}", res.err());
        let _ = std::fs::remove_file(tmp_json);
    }

    #[test]
    fn test_compact_two_column_float() {
        let markup = r#"
#set page(paper: "a4", margin: (x: 1.8cm, y: 1.8cm), columns: 2, numbering: "1")
#place(top, float: true, scope: "parent")[
  #align(center)[#text(size: 16pt, weight: "bold")[My 2-Column Title]]
  #v(1em)
]
= Section 1
This is body text in column 1.
"#;
        let tmp_json = std::env::temp_dir().join("test_botox_2col.json");
        let res = compile_typst(markup, &tmp_json, None);
        assert!(res.is_ok(), "Typst compile error: {:?}", res.err());
        let _ = std::fs::remove_file(tmp_json);
    }
}
