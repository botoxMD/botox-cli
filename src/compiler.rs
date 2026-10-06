use std::path::Path;
use typst_as_lib::TypstEngine;

pub fn compile_typst_to_pdf(typst_markup: &str, output_path: &Path) -> Result<(), String> {
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

    let engine = TypstEngine::builder()
        .main_file(typst_markup)
        .fonts(fonts)
        .build();

    let compilation_result = engine.compile();
    let doc = compilation_result.output.map_err(|e| {
        let _ = std::fs::write("/tmp/debug_fail.typ", typst_markup);
        format!("Typst compilation error: {e:?}\n(Dumped markup to /tmp/debug_fail.typ)")
    })?;

    let pdf_bytes = typst_pdf::pdf(&doc, &typst_pdf::PdfOptions::default())
        .map_err(|e| format!("PDF export error: {e:?}"))?;

    if let Some(parent) = output_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create output directory: {e}"))?;
        }
    }

    std::fs::write(output_path, pdf_bytes)
        .map_err(|e| format!("Failed to write output file '{}': {e}", output_path.display()))?;

    Ok(())
}
