use std::path::{Path, PathBuf};
use typst::LibraryExt;

enum BotoxFonts {
    Default,
    Custom {
        fonts: Vec<typst::text::Font>,
        book: typst::utils::LazyHash<typst::text::FontBook>,
    },
}

impl BotoxFonts {
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        match self {
            BotoxFonts::Default => DEFAULT_EMBEDDED_FONTS.len(),
            BotoxFonts::Custom { fonts, .. } => fonts.len(),
        }
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

struct BotoxWorld {
    main_id: typst::syntax::FileId,
    main_source: typst::syntax::Source,
    resource_dir: PathBuf,
    fonts: BotoxFonts,
    now: chrono::DateTime<chrono::Utc>,
}

impl typst::World for BotoxWorld {
    fn library(&self) -> &typst::utils::LazyHash<typst::Library> {
        &DEFAULT_LIBRARY
    }

    fn book(&self) -> &typst::utils::LazyHash<typst::text::FontBook> {
        match &self.fonts {
            BotoxFonts::Default => &DEFAULT_FONT_BOOK,
            BotoxFonts::Custom { book, .. } => book,
        }
    }

    fn main(&self) -> typst::syntax::FileId {
        self.main_id
    }

    fn source(&self, id: typst::syntax::FileId) -> typst::diag::FileResult<typst::syntax::Source> {
        if id == self.main_id {
            return Ok(self.main_source.clone());
        }
        let vpath = id.vpath();
        let rel_path = Path::new(vpath.get_without_slash());
        let abs_candidate = Path::new(vpath.get_with_slash());

        if abs_candidate.is_absolute()
            && let Ok(content) = std::fs::read_to_string(abs_candidate) {
                return Ok(typst::syntax::Source::new(id, content));
            }

        let rel_cand = self.resource_dir.join(rel_path);
        if let Ok(content) = std::fs::read_to_string(&rel_cand) {
            return Ok(typst::syntax::Source::new(id, content));
        }

        if let Ok(content) = std::fs::read_to_string(rel_path) {
            return Ok(typst::syntax::Source::new(id, content));
        }

        Err(typst::diag::FileError::NotFound(self.resource_dir.join(rel_path)))
    }

    fn file(&self, id: typst::syntax::FileId) -> typst::diag::FileResult<typst::foundations::Bytes> {
        if id == self.main_id {
            return Ok(typst::foundations::Bytes::new(self.main_source.text().as_bytes().to_vec()));
        }
        let vpath = id.vpath();
        let rel_path = Path::new(vpath.get_without_slash());
        let abs_candidate = Path::new(vpath.get_with_slash());

        if abs_candidate.is_absolute()
            && let Ok(bytes) = std::fs::read(abs_candidate) {
                return Ok(typst::foundations::Bytes::new(bytes));
            }

        let rel_cand = self.resource_dir.join(rel_path);
        if let Ok(bytes) = std::fs::read(&rel_cand) {
            return Ok(typst::foundations::Bytes::new(bytes));
        }

        if let Ok(bytes) = std::fs::read(rel_path) {
            return Ok(typst::foundations::Bytes::new(bytes));
        }

        Err(typst::diag::FileError::NotFound(self.resource_dir.join(rel_path)))
    }

    fn font(&self, id: usize) -> Option<typst::text::Font> {
        match &self.fonts {
            BotoxFonts::Default => DEFAULT_EMBEDDED_FONTS.get(id).cloned(),
            BotoxFonts::Custom { fonts, .. } => fonts.get(id).cloned(),
        }
    }

    fn today(&self, offset: Option<typst::foundations::Duration>) -> Option<typst::foundations::Datetime> {
        let mut now = self.now;
        if let Some(offset) = offset {
            now += chrono::Duration::hours(offset.hours() as i64);
        }
        let date = now.date_naive();
        use chrono::Datelike;
        typst::foundations::Datetime::from_ymd(date.year(), (date.month0() + 1) as u8, (date.day0() + 1) as u8)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct BotoxHeadingInfo {
    pub page_index: usize,
    pub text: String,
    pub y_ratio: f64,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct BotoxPagesOutput {
    pub num_pages: usize,
    pub pages: Vec<String>,
    pub headings: Vec<BotoxHeadingInfo>,
    #[serde(default)]
    pub is_slides: bool,
    #[serde(default)]
    pub pause_indices: Vec<usize>,
}

static DEFAULT_EMBEDDED_FONTS: std::sync::LazyLock<Vec<typst::text::Font>> = std::sync::LazyLock::new(|| {
    let font_bytes_list: Vec<&'static [u8]> = typst_assets::fonts().skip(6).collect();
    std::thread::scope(|s| {
        let handles: Vec<_> = font_bytes_list
            .into_iter()
            .map(|fb| {
                s.spawn(move || {
                    let bytes = typst::foundations::Bytes::new(fb);
                    typst::text::Font::iter(bytes).collect::<Vec<_>>()
                })
            })
            .collect();
        handles.into_iter().flat_map(|h| h.join().ok()).flatten().collect()
    })
});

static LIBERTINE_FONTS: std::sync::LazyLock<Vec<typst::text::Font>> = std::sync::LazyLock::new(|| {
    let font_bytes_list: Vec<&'static [u8]> = typst_assets::fonts().take(6).collect();
    std::thread::scope(|s| {
        let handles: Vec<_> = font_bytes_list
            .into_iter()
            .map(|fb| {
                s.spawn(move || {
                    let bytes = typst::foundations::Bytes::new(fb);
                    typst::text::Font::iter(bytes).collect::<Vec<_>>()
                })
            })
            .collect();
        handles.into_iter().flat_map(|h| h.join().ok()).flatten().collect()
    })
});

static DEFAULT_FONT_BOOK: std::sync::LazyLock<typst::utils::LazyHash<typst::text::FontBook>> =
    std::sync::LazyLock::new(|| {
        let mut book = typst::text::FontBook::new();
        for f in DEFAULT_EMBEDDED_FONTS.iter() {
            book.push(f.info().clone());
        }
        typst::utils::LazyHash::new(book)
    });

static DEFAULT_LIBRARY: std::sync::LazyLock<typst::utils::LazyHash<typst::Library>> =
    std::sync::LazyLock::new(|| {
        typst::utils::LazyHash::new(typst::Library::builder().build())
    });

fn is_embedded_font(name: &str) -> bool {
    let n = name.trim().trim_matches('"').trim_matches('\'').to_lowercase();
    n == "new computer modern"
        || n == "new computer modern math"
        || n == "dejavu sans mono"
        || n == "libertinus serif"
        || n.is_empty()
}

fn load_needed_fonts(typst_markup: &str) -> BotoxFonts {
    let lower_markup = typst_markup.to_ascii_lowercase();
    let has_libertine = lower_markup.contains("libertin") || lower_markup.contains("biolinum");

    // Check if custom fonts are referenced in typst_markup:
    // e.g. `font: "..."` or `font: ("...", "...")`
    let mut custom_fonts: Vec<String> = Vec::new();
    let mut idx = 0;
    while let Some(pos) = typst_markup[idx..].find("font:") {
        let start = idx + pos + 5;
        let rest = typst_markup[start..].trim_start();
        idx = start + 5;
        if rest.starts_with('(') {
            if let Some(end_paren) = rest.find(')') {
                let content = &rest[1..end_paren];
                let mut in_quote = false;
                let mut quote_char = '"';
                let mut cur = String::new();
                for ch in content.chars() {
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
            }
        } else if (rest.starts_with('"') || rest.starts_with('\''))
            && let Some(quote_char) = rest.chars().next() {
                let after_quote = &rest[1..];
                if let Some(end_q) = after_quote.find(quote_char) {
                    let trimmed = after_quote[..end_q].trim();
                    if !is_embedded_font(trimmed)
                        && !custom_fonts.iter().any(|f| f.eq_ignore_ascii_case(trimmed))
                    {
                        custom_fonts.push(trimmed.to_string());
                    }
                }
            }
    }

    if !has_libertine && custom_fonts.is_empty() {
        return BotoxFonts::Default;
    }

    let mut fonts = DEFAULT_EMBEDDED_FONTS.clone();
    if has_libertine {
        fonts.extend(LIBERTINE_FONTS.iter().cloned());
    }

    if !custom_fonts.is_empty() {
        static SYSTEM_FONTS_DB: std::sync::OnceLock<fontdb::Database> = std::sync::OnceLock::new();
        let db = SYSTEM_FONTS_DB.get_or_init(|| {
            let mut db = fontdb::Database::new();
            db.load_system_fonts();
            db
        });
        for face in db.faces() {
            let matches_custom = face.families.iter().any(|(fam, _)| {
                custom_fonts.iter().any(|cf| fam.eq_ignore_ascii_case(cf))
            }) || custom_fonts
                .iter()
                .any(|cf| face.post_script_name.eq_ignore_ascii_case(cf));

            if matches_custom
                && let fontdb::Source::File(ref path) = face.source
                    && let Ok(data) = std::fs::read(path) {
                        let bytes = typst::foundations::Bytes::new(data);
                        if let Some(font) = typst::text::Font::new(bytes, face.index) {
                            fonts.push(font);
                        }
                    }
        }
    }

    let mut book = typst::text::FontBook::new();
    for f in &fonts {
        book.push(f.info().clone());
    }
    BotoxFonts::Custom {
        fonts,
        book: typst::utils::LazyHash::new(book),
    }
}

fn format_compilation_error(
    diags: &[typst::diag::SourceDiagnostic],
    typst_markup: &str,
    markdown_source: Option<&str>,
    source_filename: Option<&str>,
) -> String {
    let source = typst::syntax::Source::detached(typst_markup);
    let mut msgs = Vec::new();
    for d in diags {
                let mut msg = d.message.to_string();
                let range = match d.span.get() {
                    typst::syntax::DiagSpanKind::Number { num, sub_range, .. } => {
                        source.range(num, sub_range)
                    }
                    typst::syntax::DiagSpanKind::Range { range, .. } => Some(range),
                    typst::syntax::DiagSpanKind::Detached => None,
                };

                let typst_snippet = range
                    .as_ref()
                    .and_then(|r| typst_markup.get(r.clone()))
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty());

                let file_not_found_token = if d.message.contains("file not found") {
                    if let Some(pos) = d.message.find("searched at ") {
                        let path_part = d.message[pos + 12..].trim_end_matches(')');
                        std::path::Path::new(path_part)
                            .file_name()
                            .and_then(|n| n.to_str())
                    } else {
                        None
                    }
                } else {
                    None
                };

                let target_token = file_not_found_token.or(typst_snippet);

                let preamble_line_count = typst_markup
                    .lines()
                    .take_while(|l| !l.starts_with("// BOTOX_BODY_START"))
                    .count();

                let mut found_md_loc: Option<(usize, usize)> = None;
                if let Some(md) = markdown_source {
                    let md_source = typst::syntax::Source::detached(md);
                    let md_line_count = md.lines().count();

                    let fm_lines = {
                        let trimmed = md.trim_start();
                        if let Some(stripped) = trimmed.strip_prefix("---") {
                            if let Some(end_idx) = stripped.find("---") {
                                let header = &trimmed[..3 + end_idx + 3];
                                header.lines().count()
                            } else {
                                0
                            }
                        } else {
                            0
                        }
                    };

                    let typst_line_opt = range.as_ref().and_then(|r| {
                        source.lines().byte_to_line_column(r.start)
                    });

                    // 1. Body line offset mapping with neighborhood search
                    if let Some((typst_line, typst_col)) = typst_line_opt {
                        if typst_line > preamble_line_count {
                            let body_line_idx = typst_line.saturating_sub(preamble_line_count + 1);
                            let target_md_line = (fm_lines + body_line_idx).min(md_line_count.saturating_sub(1));

                            let search_start = target_md_line.saturating_sub(5);
                            let search_end = (target_md_line + 5).min(md_line_count.saturating_sub(1));
                            let typst_line_text = source.text().lines().nth(typst_line).unwrap_or("");
                            let typst_trimmed = typst_line_text.trim();

                            let mut best_line = target_md_line;
                            let mut best_col = typst_col;
                            let mut found_match = false;

                            // First preference: if target_token is found in neighbor lines
                            if let Some(tok) = target_token {
                                for l_idx in search_start..=search_end {
                                    if let Some(line) = md.lines().nth(l_idx)
                                        && let Some(c_idx) = line.find(tok)
                                    {
                                        best_line = l_idx;
                                        best_col = c_idx;
                                        found_match = true;
                                        break;
                                    }
                                }
                            }

                            // Second preference: line text similarity
                            if !found_match {
                                for l_idx in search_start..=search_end {
                                    if let Some(line) = md.lines().nth(l_idx) {
                                        let line_trim = line.trim();
                                        if !line_trim.is_empty()
                                            && (line_trim == typst_trimmed
                                                || typst_trimmed.contains(line_trim)
                                                || line_trim.contains(typst_trimmed))
                                        {
                                            best_line = l_idx;
                                            break;
                                        }
                                    }
                                }
                            }

                            found_md_loc = Some((best_line, best_col));
                        } else {
                            let fm_line = typst_line.min(fm_lines.saturating_sub(1));
                            found_md_loc = Some((fm_line, typst_col));
                        }
                    }

                    // Fallback: only if no line mapping was found, try specific file/token search
                    if found_md_loc.is_none()
                        && let Some(token) = file_not_found_token.or(target_token)
                        && (token.len() >= 3 || token.contains('.'))
                        && let Some(idx) = md.find(token)
                    {
                        found_md_loc = md_source.lines().byte_to_line_column(idx);
                    }
                }

                let is_latex = if d.message.contains("label") || d.message.contains("bib-") {
                    false
                } else if let (Some(md), Some((l, _))) = (markdown_source, found_md_loc) {
                    if let Some(target_line) = md.lines().nth(l) {
                        let trimmed_target = target_line.trim_start();
                        if trimmed_target.starts_with(":::") || trimmed_target.starts_with("```") || trimmed_target.starts_with(r"\ref") {
                            false
                        } else {
                            let mut in_math_block = false;
                            for (idx, line) in md.lines().enumerate() {
                                let trimmed = line.trim();
                                if trimmed == "$$" {
                                    in_math_block = !in_math_block;
                                } else if trimmed.starts_with("$$") && !trimmed.ends_with("$$") {
                                    in_math_block = true;
                                } else if !trimmed.starts_with("$$") && trimmed.ends_with("$$") {
                                    in_math_block = false;
                                }
                                if idx == l {
                                    break;
                                }
                            }
                            let has_inline_math = target_line.contains('$') && !target_line.contains(r"\$");
                            let has_latex_cmd = target_line.contains(r"\frac")
                                || target_line.contains(r"\sqrt")
                                || target_line.contains(r"\sum")
                                || target_line.contains(r"\int")
                                || target_line.contains(r"\mathbf")
                                || target_line.contains(r"\alpha")
                                || target_line.contains(r"\beta");
                            in_math_block || has_inline_math || has_latex_cmd
                        }
                    } else {
                        false
                    }
                } else {
                    d.message.contains("math") || d.message.contains("equation")
                };

                let category = if is_latex {
                    "LaTeX Error"
                } else if d.message.contains("file not found") {
                    "Resource Error"
                } else {
                    "Typst Error"
                };

                let fname = source_filename.unwrap_or("document.md");
                let loc_str = if let Some((l, c)) = found_md_loc {
                    format!("{fname}:{}:{}: ", l + 1, c + 1)
                } else if let Some(r) = range.as_ref() {
                    if let Some((line, col)) = source.lines().byte_to_line_column(r.start) {
                        format!("line {}:{}: ", line + 1, col + 1)
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };

                let snippet_suffix = if let Some(token) = target_token {
                    if !msg.contains(token) {
                        format!(" (near '{token}')")
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };

                msg = format!("[{category}] {loc_str}{msg}{snippet_suffix}");

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

pub fn compile_typst(
    typst_markup: &str,
    output_path: &Path,
    resource_dir: Option<&Path>,
) -> Result<(), String> {
    compile_typst_with_source(typst_markup, output_path, resource_dir, None, None)
}

pub fn compile_typst_with_source(
    typst_markup: &str,
    output_path: &Path,
    resource_dir: Option<&Path>,
    markdown_source: Option<&str>,
    source_filename: Option<&str>,
) -> Result<(), String> {
    let t0 = std::time::Instant::now();
    let fonts = load_needed_fonts(typst_markup);
    let t_fonts = t0.elapsed();

    let res_dir = resource_dir
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let t1 = std::time::Instant::now();
    let main_source = typst::syntax::Source::detached(typst_markup.to_string());
    let world = BotoxWorld {
        main_id: main_source.id(),
        main_source,
        resource_dir: res_dir,
        fonts,
        now: chrono::Utc::now(),
    };
    let t_world = t1.elapsed();

    let t2 = std::time::Instant::now();
    let typst::diag::Warned { output, warnings: _ } = typst::compile(&world);
    let t_compile = t2.elapsed();

    let doc: typst_layout::PagedDocument = output.map_err(|diags| {
        if std::env::var("BOTOX_DEBUG").is_ok() {
            let debug_path = crate::diagrams::get_cache_dir().join("debug_fail.typ");
            let _ = std::fs::write(debug_path, typst_markup);
        }
        format_compilation_error(&diags, typst_markup, markdown_source, source_filename)
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

    if let Some(parent) = output_path.parent()
        && !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create output directory: {e}"))?;
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

    struct PageWalkContext<'a> {
        page_idx: usize,
        page_height: f64,
        check_pause: bool,
        text_out: &'a mut String,
        is_pause_out: &'a mut bool,
        headings_out: &'a mut Vec<BotoxHeadingInfo>,
    }

    fn walk_frame_data(
        frame: &typst_library::layout::Frame,
        ts: typst_library::layout::Transform,
        ctx: &mut PageWalkContext<'_>,
    ) {
        for (point, item) in frame.items() {
            let item_ts = ts.pre_concat(typst_library::layout::Transform::translate(point.x, point.y));
            match item {
                typst_library::layout::FrameItem::Text(text_item) => {
                    let text = text_item.text.as_str();
                    let trimmed = text.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    if ctx.check_pause && text.contains("botox-pause-step") {
                        *ctx.is_pause_out = true;
                        continue;
                    }

                    let x = item_ts.tx.to_pt();
                    let y = item_ts.ty.to_pt();
                    let size = (text_item.size.to_pt() * item_ts.sy.get()).abs();
                    let width = (text_item.width().to_pt() * item_ts.sx.get()).abs();
                    let escaped = xml_escape(text);

                    use std::fmt::Write;
                    let _ = write!(
                        ctx.text_out,
                        r#"<text x="{x:.2}" y="{y:.2}" font-size="{size:.2}pt" textLength="{width:.2}" lengthAdjust="spacingAndGlyphs" fill="transparent" stroke="none" style="cursor: text; user-select: text;">{escaped}</text>"#
                    );

                    if size >= 12.8 {
                        let cur_ratio = (y / ctx.page_height).clamp(0.0, 1.0);
                        if let Some(last) = ctx.headings_out.last_mut() {
                            if last.page_index == ctx.page_idx && (last.y_ratio - cur_ratio).abs() < 0.008 {
                                last.text.push(' ');
                                last.text.push_str(trimmed);
                            } else {
                                ctx.headings_out.push(BotoxHeadingInfo {
                                    page_index: ctx.page_idx,
                                    text: trimmed.to_string(),
                                    y_ratio: cur_ratio,
                                });
                            }
                        } else {
                            ctx.headings_out.push(BotoxHeadingInfo {
                                page_index: ctx.page_idx,
                                text: trimmed.to_string(),
                                y_ratio: cur_ratio,
                            });
                        }
                    }
                }
                typst_library::layout::FrameItem::Group(group) => {
                    let group_ts = item_ts.pre_concat(group.transform);
                    walk_frame_data(&group.frame, group_ts, ctx);
                }
                _ => {}
            }
        }
    }

    fn process_page_vector(
        page: &typst_layout::Page,
        page_idx: usize,
        svg_opts: &typst_svg::SvgOptions,
        check_pause: bool,
    ) -> (String, bool, Vec<BotoxHeadingInfo>) {
        let mut text_elements = String::new();
        let mut is_pause = false;
        let mut headings = Vec::new();
        let page_height = page.frame.height().to_pt().max(1.0);

        let mut ctx = PageWalkContext {
            page_idx,
            page_height,
            check_pause,
            text_out: &mut text_elements,
            is_pause_out: &mut is_pause,
            headings_out: &mut headings,
        };

        walk_frame_data(
            &page.frame,
            typst_library::layout::Transform::identity(),
            &mut ctx,
        );

        let mut svg = typst_svg::svg(page, svg_opts);
        if !text_elements.is_empty()
            && let Some(idx) = svg.rfind("</svg>") {
                svg.insert_str(
                    idx,
                    &format!(r#"<g class="botox-text-layer" style="user-select: text; -webkit-user-select: text; pointer-events: auto;">{text_elements}</g>"#),
                );
            }

        if is_pause
            && let Some(pos) = svg.find("<svg") {
                let insert_pos = pos + 4;
                svg.insert_str(insert_pos, r#" data-pause-step="true" class="pause-step""#);
            }

        (svg, is_pause, headings)
    }

    match ext.as_str() {
        "svg" => {
            let svg_opts = typst_svg::SvgOptions::default();
            let svg_content = if doc.pages().len() <= 1 {
                if let Some(first_page) = doc.pages().first() {
                    process_page_vector(first_page, 0, &svg_opts, false).0
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
            let t_svg_start = std::time::Instant::now();
            let svg_opts = typst_svg::SvgOptions::default();

            let is_slides = doc.pages().first().map(|p| {
                p.frame.width().to_pt() > p.frame.height().to_pt()
            }).unwrap_or(false);

            let num_pages = doc.pages().len();
            let (pages, pause_indices, headings) = if num_pages <= 1 {
                let mut pages = Vec::with_capacity(num_pages);
                let mut pause_indices = Vec::new();
                let mut headings = Vec::new();
                for (idx, p) in doc.pages().iter().enumerate() {
                    let (svg, is_pause, h) = process_page_vector(p, idx, &svg_opts, true);
                    if is_pause {
                        pause_indices.push(idx);
                    }
                    pages.push(svg);
                    headings.extend(h);
                }
                (pages, pause_indices, headings)
            } else {
                let num_threads = std::thread::available_parallelism()
                    .map(|n| n.get())
                    .unwrap_or(4)
                    .min(num_pages);
                let chunk_size = num_pages.div_ceil(num_threads);
                let pages_slice = doc.pages();
                let results: Vec<_> = std::thread::scope(|s| {
                    let mut handles = Vec::new();
                    for (chunk_idx, chunk) in pages_slice.chunks(chunk_size).enumerate() {
                        let start_page_idx = chunk_idx * chunk_size;
                        let svg_opts_ref = &svg_opts;
                        handles.push(s.spawn(move || {
                            let mut chunk_res = Vec::with_capacity(chunk.len());
                            for (rel_idx, page) in chunk.iter().enumerate() {
                                let abs_idx = start_page_idx + rel_idx;
                                chunk_res.push((abs_idx, process_page_vector(page, abs_idx, svg_opts_ref, true)));
                            }
                            chunk_res
                        }));
                    }
                    handles.into_iter().flat_map(|h| h.join().unwrap()).collect()
                });

                let mut pages = Vec::with_capacity(num_pages);
                let mut pause_indices = Vec::new();
                let mut headings = Vec::new();
                for (abs_idx, (svg, is_pause, h)) in results {
                    if is_pause {
                        pause_indices.push(abs_idx);
                    }
                    pages.push(svg);
                    headings.extend(h);
                }
                (pages, pause_indices, headings)
            };
            let t_svg_pages = t_svg_start.elapsed();

            let output_struct = BotoxPagesOutput {
                num_pages: pages.len(),
                pages,
                headings,
                is_slides,
                pause_indices,
            };
            let t_ser_start = std::time::Instant::now();
            let json_str = serde_json::to_string(&output_struct)
                .map_err(|e| format!("Failed to serialize pages to JSON: {e}"))?;
            let t_ser = t_ser_start.elapsed();

            let t_write_start = std::time::Instant::now();
            std::fs::write(output_path, json_str)
                .map_err(|e| format!("Failed to write output JSON file '{}': {e}", output_path.display()))?;
            let t_write = t_write_start.elapsed();

            if std::env::var("BOTOX_PROFILE").is_ok() {
                eprintln!("[PROFILE:JSON] load_fonts: {t_fonts:.2?}, world_init: {t_world:.2?}, typst_compile: {t_compile:.2?}, svg_pages: {t_svg_pages:.2?}, serde_json: {t_ser:.2?}, write_fs: {t_write:.2?}");
            }
        }
        "html" | "htm" => {
            let svg_opts = typst_svg::SvgOptions::default();
            let is_presentation = doc.pages().first().map(|p| {
                p.frame.width().to_pt() > p.frame.height().to_pt()
            }).unwrap_or(false);

            let pages_slice = doc.pages();
            let pages: Vec<String> = if pages_slice.len() <= 1 {
                pages_slice
                    .iter()
                    .enumerate()
                    .map(|(idx, p)| process_page_vector(p, idx, &svg_opts, is_presentation).0)
                    .collect()
            } else {
                let num_threads = std::thread::available_parallelism()
                    .map(|n| n.get())
                    .unwrap_or(4)
                    .min(pages_slice.len());
                let chunk_size = pages_slice.len().div_ceil(num_threads);
                std::thread::scope(|s| {
                    let mut handles = Vec::new();
                    for (chunk_idx, chunk) in pages_slice.chunks(chunk_size).enumerate() {
                        let start_idx = chunk_idx * chunk_size;
                        let svg_opts_ref = &svg_opts;
                        handles.push(s.spawn(move || {
                            chunk
                                .iter()
                                .enumerate()
                                .map(|(rel_idx, page)| {
                                    process_page_vector(page, start_idx + rel_idx, svg_opts_ref, is_presentation).0
                                })
                                .collect::<Vec<_>>()
                        }));
                    }
                    handles.into_iter().flat_map(|h| h.join().unwrap()).collect()
                })
            };

            let html_content = if is_presentation {
                render_slides_html(&pages)
            } else {
                render_document_html(&pages)
            };

            std::fs::write(output_path, html_content)
                .map_err(|e| format!("Failed to write output HTML file '{}': {e}", output_path.display()))?;
        }
        _ => {
            let t_pdf_start = std::time::Instant::now();
            let pdf_bytes = typst_pdf::pdf(&doc, &typst_pdf::PdfOptions::default())
                .map_err(|e| format!("PDF export error: {e:?}"))?;
            let t_pdf = t_pdf_start.elapsed();

            let t_write_start = std::time::Instant::now();
            std::fs::write(output_path, pdf_bytes)
                .map_err(|e| format!("Failed to write output file '{}': {e}", output_path.display()))?;
            let t_write = t_write_start.elapsed();

            if std::env::var("BOTOX_PROFILE").is_ok() {
                eprintln!("[PROFILE] load_fonts: {t_fonts:.2?}, world_init: {t_world:.2?}, typst_compile: {t_compile:.2?}, pdf_export: {t_pdf:.2?}, write_fs: {t_write:.2?}");
            }
        }
    }

    Ok(())
}

fn render_slides_html(pages: &[String]) -> String {
    let mut slides_html = String::new();
    let mut overview_html = String::new();

    for (i, svg) in pages.iter().enumerate() {
        let idx = i + 1;
        let active_cls = if i == 0 { " active" } else { "" };
        slides_html.push_str(&format!(
            "<div class=\"slide{active_cls}\" data-slide=\"{idx}\">{svg}</div>\n"
        ));
        overview_html.push_str(&format!(
            "<div class=\"overview-item{active_cls}\" data-target=\"{idx}\">{svg}<span class=\"overview-badge\">{idx}</span></div>\n"
        ));
    }

    let total = pages.len();

    format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Botox Presentation</title>
<style>
* {{ box-sizing: border-box; margin: 0; padding: 0; }}
body {{
  background: #0f172a;
  color: #f8fafc;
  font-family: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  overflow: hidden;
  height: 100vh;
  width: 100vw;
  user-select: none;
}}
.progress-bar {{
  position: fixed;
  top: 0;
  left: 0;
  height: 3px;
  background: #3b82f6;
  width: 0%;
  transition: width 0.25s ease;
  z-index: 1000;
}}
.viewport {{
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100vh;
  width: 100vw;
}}
.stage {{
  width: min(96vw, calc(96vh * (16 / 9)));
  height: min(96vh, calc(96vw * (9 / 16)));
  aspect-ratio: 16 / 9;
  position: relative;
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.7);
  border-radius: 6px;
  overflow: hidden;
  background: #000;
}}
.slide {{
  display: none;
  width: 100%;
  height: 100%;
}}
.slide.active {{
  display: block;
}}
.slide svg {{
  width: 100%;
  height: 100%;
  display: block;
}}
/* Overview Grid */
.overview {{
  display: none;
  position: fixed;
  inset: 0;
  background: rgba(15, 23, 42, 0.95);
  backdrop-filter: blur(12px);
  z-index: 500;
  overflow-y: auto;
  padding: 40px 24px;
}}
.overview.active {{
  display: block;
}}
.overview-header {{
  max-width: 1300px;
  margin: 0 auto 24px auto;
  display: flex;
  justify-content: space-between;
  align-items: center;
}}
.overview-title {{
  font-size: 1.25rem;
  font-weight: 600;
  color: #e2e8f0;
}}
.overview-close {{
  background: rgba(255, 255, 255, 0.1);
  border: none;
  color: #fff;
  padding: 6px 14px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
}}
.overview-close:hover {{
  background: rgba(255, 255, 255, 0.2);
}}
.overview-grid {{
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 20px;
  max-width: 1300px;
  margin: 0 auto;
}}
.overview-item {{
  aspect-ratio: 16 / 9;
  border-radius: 6px;
  overflow: hidden;
  cursor: pointer;
  position: relative;
  box-shadow: 0 4px 12px rgba(0,0,0,0.5);
  transition: transform 0.15s ease, box-shadow 0.15s ease, outline 0.15s ease;
  background: #111;
}}
.overview-item:hover {{
  transform: translateY(-3px) scale(1.02);
  box-shadow: 0 8px 24px rgba(59, 130, 246, 0.4);
}}
.overview-item.active {{
  outline: 3px solid #3b82f6;
}}
.overview-item svg {{
  width: 100%;
  height: 100%;
  pointer-events: none;
}}
.overview-badge {{
  position: absolute;
  bottom: 8px;
  right: 8px;
  background: rgba(0, 0, 0, 0.75);
  color: #fff;
  font-size: 11px;
  font-weight: 600;
  padding: 2px 7px;
  border-radius: 4px;
}}
/* On-Screen Controls */
.osc {{
  position: fixed;
  bottom: 24px;
  left: 50%;
  transform: translateX(-50%);
  background: rgba(30, 41, 59, 0.85);
  backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 30px;
  padding: 6px 14px;
  display: flex;
  align-items: center;
  gap: 8px;
  color: #cbd5e1;
  font-size: 13px;
  z-index: 200;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
  opacity: 0.25;
  transition: opacity 0.3s ease;
}}
.osc:hover, body.active-controls .osc {{
  opacity: 1;
}}
.osc-btn {{
  background: transparent;
  border: none;
  color: #cbd5e1;
  cursor: pointer;
  padding: 5px 9px;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 500;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.15s ease, color 0.15s ease;
}}
.osc-btn:hover {{
  background: rgba(255, 255, 255, 0.12);
  color: #fff;
}}
.osc-counter {{
  padding: 0 6px;
  font-weight: 600;
  color: #94a3b8;
}}
@media print {{
  body {{
    background: #fff;
    overflow: visible;
    height: auto;
  }}
  .progress-bar, .osc, .overview {{ display: none !important; }}
  .viewport {{
    display: block;
    height: auto;
    width: auto;
  }}
  .stage {{
    width: 100%;
    height: auto;
    box-shadow: none;
    border-radius: 0;
    overflow: visible;
  }}
  .slide {{
    display: block !important;
    page-break-after: always;
    break-after: page;
    height: 100vh;
  }}
}}
</style>
</head>
<body>
<div class="progress-bar" id="progressBar"></div>

<div class="viewport">
  <div class="stage">
    {slides_html}
  </div>
</div>

<div class="osc">
  <button class="osc-btn" id="prevBtn" title="Previous Slide (Left/Up)">&#9664;</button>
  <span class="osc-counter" id="counter">1 / {total}</span>
  <button class="osc-btn" id="nextBtn" title="Next Slide (Right/Down/Space)">&#9654;</button>
  <button class="osc-btn" id="overviewBtn" title="Toggle Overview (O / Esc)">&#8862; Grid</button>
  <button class="osc-btn" id="fsBtn" title="Toggle Fullscreen (F)">&#x26F6;</button>
</div>

<div class="overview" id="overview">
  <div class="overview-header">
    <span class="overview-title">Slide Overview ({total} slides)</span>
    <button class="overview-close" id="closeOverviewBtn">Close (Esc)</button>
  </div>
  <div class="overview-grid">
    {overview_html}
  </div>
</div>

<script>
(function() {{
  const total = {total};
  let current = 1;
  const slides = document.querySelectorAll('.slide');
  const overviewItems = document.querySelectorAll('.overview-item');
  const counter = document.getElementById('counter');
  const progressBar = document.getElementById('progressBar');
  const overview = document.getElementById('overview');

  function update() {{
    slides.forEach((s, idx) => {{
      s.classList.toggle('active', idx + 1 === current);
    }});
    overviewItems.forEach((item, idx) => {{
      item.classList.toggle('active', idx + 1 === current);
    }});
    counter.textContent = current + ' / ' + total;
    progressBar.style.width = ((current / total) * 100) + '%';
    window.location.hash = current;
  }}

  function gotoSlide(n) {{
    if (n < 1) n = 1;
    if (n > total) n = total;
    current = n;
    update();
  }}

  function next() {{ gotoSlide(current + 1); }}
  function prev() {{ gotoSlide(current - 1); }}

  function toggleOverview() {{
    const isOpen = overview.classList.toggle('active');
    if (isOpen) {{
      const activeItem = overview.querySelector('.overview-item.active');
      if (activeItem) activeItem.scrollIntoView({{ block: 'nearest' }});
    }}
  }}

  function toggleFullscreen() {{
    if (!document.fullscreenElement) {{
      document.documentElement.requestFullscreen().catch(() => {{}});
    }} else {{
      document.exitFullscreen().catch(() => {{}});
    }}
  }}

  document.getElementById('nextBtn').addEventListener('click', next);
  document.getElementById('prevBtn').addEventListener('click', prev);
  document.getElementById('overviewBtn').addEventListener('click', toggleOverview);
  document.getElementById('closeOverviewBtn').addEventListener('click', toggleOverview);
  document.getElementById('fsBtn').addEventListener('click', toggleFullscreen);

  overviewItems.forEach(item => {{
    item.addEventListener('click', () => {{
      const target = parseInt(item.getAttribute('data-target'), 10);
      gotoSlide(target);
      overview.classList.remove('active');
    }});
  }});

  // Keyboard navigation
  window.addEventListener('keydown', (e) => {{
    if (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA') return;
    switch(e.key) {{
      case 'ArrowRight':
      case 'ArrowDown':
      case 'PageDown':
      case ' ':
      case 'Enter':
      case 'n':
      case 'N':
        if (!overview.classList.contains('active')) next();
        break;
      case 'ArrowLeft':
      case 'ArrowUp':
      case 'PageUp':
      case 'Backspace':
      case 'p':
      case 'P':
        if (!overview.classList.contains('active')) prev();
        break;
      case 'Home':
        gotoSlide(1);
        break;
      case 'End':
        gotoSlide(total);
        break;
      case 'f':
      case 'F':
        toggleFullscreen();
        break;
      case 'o':
      case 'O':
        toggleOverview();
        break;
      case 'Escape':
        if (overview.classList.contains('active')) {{
          overview.classList.remove('active');
        }}
        break;
    }}
  }});

  // Touch Swipe navigation
  let touchStartX = 0;
  window.addEventListener('touchstart', e => {{
    touchStartX = e.changedTouches[0].screenX;
  }}, {{ passive: true }});
  window.addEventListener('touchend', e => {{
    const diff = e.changedTouches[0].screenX - touchStartX;
    if (Math.abs(diff) > 40) {{
      if (diff < 0) next(); else prev();
    }}
  }}, {{ passive: true }});

  // Fade controls on idle
  let mouseTimer = null;
  window.addEventListener('mousemove', () => {{
    document.body.classList.add('active-controls');
    clearTimeout(mouseTimer);
    mouseTimer = setTimeout(() => {{
      document.body.classList.remove('active-controls');
    }}, 2500);
  }});

  // Read initial slide from hash
  const hash = parseInt(window.location.hash.replace('#', ''), 10);
  if (!isNaN(hash) && hash >= 1 && hash <= total) {{
    current = hash;
  }}
  update();
}})();
</script>
</body>
</html>"#)
}

fn render_document_html(pages: &[String]) -> String {
    let mut pages_html = String::new();
    for (i, svg) in pages.iter().enumerate() {
        let idx = i + 1;
        pages_html.push_str(&format!(
            "<div class=\"page\" id=\"page-{idx}\">{svg}<div class=\"page-number\">{idx}</div></div>\n"
        ));
    }

    format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Botox Document</title>
<style>
* {{ box-sizing: border-box; margin: 0; padding: 0; }}
body {{
  background: #f1f5f9;
  color: #0f172a;
  font-family: system-ui, -apple-system, sans-serif;
  padding: 32px 16px;
  min-height: 100vh;
}}
.container {{
  max-width: 860px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: 24px;
}}
.page {{
  background: #fff;
  border-radius: 4px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.08), 0 1px 3px rgba(0, 0, 0, 0.04);
  position: relative;
  overflow: hidden;
}}
.page svg {{
  width: 100%;
  height: auto;
  display: block;
}}
.page-number {{
  position: absolute;
  bottom: 8px;
  right: 12px;
  font-size: 11px;
  color: #94a3b8;
  pointer-events: none;
}}
@media print {{
  body {{
    background: #fff;
    padding: 0;
  }}
  .container {{
    max-width: none;
    margin: 0;
    gap: 0;
  }}
  .page {{
    box-shadow: none;
    border-radius: 0;
    page-break-after: always;
    break-after: page;
  }}
  .page-number {{ display: none; }}
}}
</style>
</head>
<body>
<div class="container">
  {pages_html}
</div>
</body>
</html>"#)
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

    let main_source = typst::syntax::Source::detached(typst_markup.to_string());
    let world = BotoxWorld {
        main_id: main_source.id(),
        main_source,
        resource_dir: res_dir,
        fonts,
        now: chrono::Utc::now(),
    };

    let typst::diag::Warned { output, warnings: _ } = typst::compile(&world);
    let doc: typst_layout::PagedDocument = output.map_err(|diags| {
        format_compilation_error(&diags, typst_markup, None, None)
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

        let tmp_html = std::env::temp_dir().join("test_botox_page.html");
        assert!(compile_typst(markup, &tmp_html, None).is_ok());
        let html_str = std::fs::read_to_string(&tmp_html).expect("Read HTML");
        assert!(html_str.contains("<!DOCTYPE html>"));
        assert!(html_str.contains("<svg"));

        let _ = std::fs::remove_file(tmp_svg);
        let _ = std::fs::remove_file(tmp_json);
        let _ = std::fs::remove_file(tmp_html);
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
#set page(paper: "a4", margin: (x: 1.8cm, y: 1.8cm), numbering: "1")
#align(center)[#text(size: 18pt, weight: "bold")[Conference Paper Title]]
#v(1.5em)
#show: columns.with(2, gutter: 14pt)
= Section 1
First column body text.
#colbreak()
= Section 2
Second column body text.
"#;
        let tmp_json = std::env::temp_dir().join("test_botox_2col.json");
        let res = compile_typst(markup, &tmp_json, None);
        assert!(res.is_ok(), "Typst compile error: {:?}", res.err());
        let json_str = std::fs::read_to_string(&tmp_json).unwrap();
        let json_val: serde_json::Value = serde_json::from_str(&json_str).expect("Valid JSON");
        let svg = json_val["pages"][0].as_str().expect("SVG string");
        let mut found_col1 = false;
        let mut found_col2 = false;
        for line in svg.split("<text").skip(1) {
            if let Some(x_pos) = line.find("x=\"") {
                let rest = &line[x_pos + 3..];
                if let Some(quote_end) = rest.find('"')
                    && let Ok(x) = rest[..quote_end].parse::<f64>() {
                        if (x - 51.02).abs() < 5.0 {
                            found_col1 = true;
                        } else if (280.0..350.0).contains(&x) {
                            found_col2 = true;
                        }
                    }
            }
        }
        assert!(found_col1, "Should find text in column 1 (x ~ 51)");
        assert!(found_col2, "Should find text in column 2 (x ~ 300)");
        let _ = std::fs::remove_file(tmp_json);
    }

    #[test]
    fn test_compact_document_with_pagebreak_compilation() {
        let md = "---\ntitle: Two Column Test\ntheme: compact\n---\n\n# Section 1\nFirst column text\n\n\\newpage\n\n# Section 2\nSecond column text\n";
        let (fm, body_md) = crate::extract_frontmatter(md);
        let body_typst = crate::markdown::markdown_to_typst(body_md, false, false, "en", None, None, None, None, None);
        let config = crate::config::DocumentConfig::defaults();
        let typst_markup = crate::document::wrap_document(&body_typst, &fm, &config, None, None, None, None);
        let tmp_pdf = std::env::temp_dir().join("test_botox_2col_pagebreak.pdf");
        let res = compile_typst(&typst_markup, &tmp_pdf, None);
        assert!(res.is_ok(), "Typst compile error: {:?}", res.err());
        let _ = std::fs::remove_file(tmp_pdf);
    }

    #[test]
    fn test_pause_step_tagging_in_json() {
        let markup = "= Slide 1\n#place(top + left)[#text(size: 0.001pt, fill: rgb(0, 0, 0, 0))[botox-pause-step]]\n#pagebreak()\n= Slide 1 Complete";
        let tmp_json = std::env::temp_dir().join("test_pause_tag.json");
        assert!(compile_typst(markup, &tmp_json, None).is_ok());
        let json_str = std::fs::read_to_string(&tmp_json).expect("Read JSON");
        let parsed: BotoxPagesOutput = serde_json::from_str(&json_str).expect("Valid JSON");
        assert_eq!(parsed.num_pages, 2);
        assert_eq!(parsed.pause_indices, vec![0]);
        assert!(parsed.pages[0].contains("data-pause-step=\"true\""));
        assert!(!parsed.pages[1].contains("data-pause-step=\"true\""));
        let _ = std::fs::remove_file(tmp_json);
    }

    #[test]
    fn test_callout_with_codeblock_compilation() {
        let callout_markup = "#botox_callout(\"warning\", \"Diagram rendering failed: Kroki unreachable\")[\n```mermaid\ngraph TD;\n  A-->B;\n```\n]";
        let fm = serde_yaml::Value::Mapping(serde_yaml::Mapping::new());
        let config = crate::config::DocumentConfig::defaults();
        let typst_markup = crate::document::wrap_document(callout_markup, &fm, &config, None, None, None, None);
        let tmp_pdf = std::env::temp_dir().join("test_callout_codeblock.pdf");
        let res = compile_typst(&typst_markup, &tmp_pdf, None);
        assert!(res.is_ok(), "Typst compile error: {:?}", res.err());
        let _ = std::fs::remove_file(tmp_pdf);
    }

    #[test]
    fn test_load_needed_fonts_defaults_and_custom() {
        let default_markup = r#"
            #set text(font: ("New Computer Modern",), size: 11pt, lang: "en")
            #show math.equation: set text(font: "New Computer Modern Math")
            #show raw: set text(font: ("DejaVu Sans Mono",))
        "#;
        let fonts = load_needed_fonts(default_markup);
        // Default fonts should contain only embedded New Computer Modern, Math, and DejaVu variants
        assert!(!fonts.is_empty());
        assert!(fonts.len() <= 12, "Should only load default embedded fonts, got {}", fonts.len());

        let libertine_markup = r#"
            #set text(font: ("Libertinus Serif",), size: 11pt)
        "#;
        let lib_fonts = load_needed_fonts(libertine_markup);
        assert!(lib_fonts.len() > fonts.len(), "Libertine markup should load additional Libertinus fonts");
    }

    #[test]
    fn test_format_compilation_error_location() {
        let invalid_markup = "= Valid Title\n\n#nonexistent_function_xyz(123)\n";
        let tmp_pdf = std::env::temp_dir().join("test_err_loc.pdf");
        let res = compile_typst(invalid_markup, &tmp_pdf, None);
        assert!(res.is_err());
        let err = res.err().unwrap();
        assert!(err.contains("line 3:"), "Error message must contain line number: {err}");
        let _ = std::fs::remove_file(tmp_pdf);
    }
}
