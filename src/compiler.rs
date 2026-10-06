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

pub fn compile_typst_to_pdf(
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
