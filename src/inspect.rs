use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fmt::Write as _;
use typst_library::foundations::Value as TypstValue;
use typst_library::introspection::{MetadataElem, Tag};
use typst_library::layout::{Frame, FrameItem, Transform};
use typst_library::model::{HeadingElem, TableElem};

const PT_PER_MM: f64 = 72.0 / 25.4;
const PT_PER_CM: f64 = 72.0 / 2.54;

/// Axis-aligned bounding box in PDF points (`pt`), origin at top-left of page.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct BBox {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl BBox {
    pub fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        Self {
            x,
            y,
            w: w.max(0.0),
            h: h.max(0.0),
        }
    }

    pub fn right(self) -> f64 {
        self.x + self.w
    }

    pub fn bottom(self) -> f64 {
        self.y + self.h
    }

    pub fn area(self) -> f64 {
        self.w * self.h
    }

    pub fn union(self, other: Self) -> Self {
        let min_x = self.x.min(other.x);
        let min_y = self.y.min(other.y);
        let max_x = self.right().max(other.right());
        let max_y = self.bottom().max(other.bottom());
        Self::new(min_x, min_y, max_x - min_x, max_y - min_y)
    }

    pub fn intersection(self, other: Self) -> Option<Self> {
        let min_x = self.x.max(other.x);
        let min_y = self.y.max(other.y);
        let max_x = self.right().min(other.right());
        let max_y = self.bottom().min(other.bottom());
        if max_x > min_x && max_y > min_y {
            Some(Self::new(min_x, min_y, max_x - min_x, max_y - min_y))
        } else {
            None
        }
    }

    pub fn intersects(self, other: Self) -> bool {
        self.intersection(other).is_some()
    }

    pub fn to_array(self) -> [f64; 4] {
        [
            round2(self.x),
            round2(self.y),
            round2(self.w),
            round2(self.h),
        ]
    }
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

#[derive(Debug, Clone)]
struct RawTextRun {
    x: f64,
    y_baseline: f64,
    bbox: BBox,
    text: String,
    font_name: String,
    font_size_pt: f64,
    has_missing_glyph: bool,
    is_math: bool,
    in_table: bool,
    col: usize,
    src_line: Option<usize>,
    src_kind: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderedContainer {
    pub kind: String, // "shape" | "image"
    pub page: usize,
    pub col: usize,
    pub bbox: BBox,
    pub is_filled_rect: bool,
    pub src_line: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextLine {
    pub page: usize,
    pub col: usize,
    pub y_baseline: f64,
    pub bbox: BBox,
    pub text: String,
    pub word_count: usize,
    pub font_size_pt: f64,
    #[serde(default)]
    pub is_math: bool,
    #[serde(default)]
    pub in_table: bool,
    pub src_line: Option<usize>,
    pub src_kind: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutBlock {
    pub id: usize,
    pub kind: String,
    pub page: usize,
    pub col: usize,
    pub bbox: BBox,
    pub line_count: usize,
    pub last_line_fill_pct: f64,
    pub last_line_words: usize,
    pub median_leading_pt: f64,
    pub text_preview: String,
    pub src_line: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnStats {
    pub col: usize,
    pub x_min_pt: f64,
    pub x_max_pt: f64,
    pub content_bottom_y: f64,
    pub fill_pct: f64,
    pub bottom_dead_space_pt: f64,
    pub block_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageSummary {
    pub page: usize,
    pub width_pt: f64,
    pub height_pt: f64,
    pub content_bbox_pt: [f64; 4],
    pub fill_pct: f64,
    pub bottom_dead_space_pt: f64,
    pub columns: Vec<ColumnStats>,
    pub block_count: usize,
    pub line_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontTelemetry {
    pub family: String,
    pub sizes_pt: Vec<f64>,
    pub glyph_runs: usize,
    pub missing_glyph_runs: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LayoutIssue {
    pub code: String,
    pub severity: String,
    pub page: usize,
    pub col: usize,
    pub bbox_pt: [f64; 4],
    pub src_line: Option<usize>,
    pub text_excerpt: String,
    pub reclaimable_pt: Option<f64>,
    pub metrics: serde_json::Value,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IssueCounts {
    pub error: usize,
    pub warn: usize,
    pub hint: usize,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReclaimPlan {
    pub target_pages: usize,
    pub current_pages: usize,
    pub overflow_pt: f64,
    pub total_reclaimable_pt: f64,
    pub achievable: bool,
    pub steps: Vec<LayoutIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectReport {
    pub file: String,
    pub page_count: usize,
    pub last_page_fill_pct: f64,
    pub fonts_used: Vec<FontTelemetry>,
    pub pages: Vec<PageSummary>,
    pub issue_counts: IssueCounts,
    pub issues: Vec<LayoutIssue>,
    pub reclaim_plan: Option<ReclaimPlan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareDelta {
    pub baseline_pages: usize,
    pub current_pages: usize,
    pub pages_delta: isize,
    pub baseline_last_fill_pct: f64,
    pub current_last_fill_pct: f64,
    pub last_fill_delta_pct: f64,
    pub baseline_issues: usize,
    pub current_issues: usize,
    pub issues_delta: isize,
    pub resolved_codes: BTreeMap<String, usize>,
    pub added_codes: BTreeMap<String, usize>,
}

#[derive(Debug, Clone)]
pub struct ExtractedDocument {
    pub pages: Vec<PageSummary>,
    pub lines: Vec<TextLine>,
    pub blocks: Vec<LayoutBlock>,
    pub containers: Vec<RenderedContainer>,
    pub fonts_used: Vec<FontTelemetry>,
    raw_runs_by_page: Vec<Vec<RawTextRun>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActiveTagKind {
    Table,
    Heading,
    Other,
}

struct FrameWalkState {
    cur_src_line: Option<usize>,
    cur_src_kind: Option<String>,
    tag_stack: Vec<ActiveTagKind>,
    in_table_depth: usize,
    runs: Vec<RawTextRun>,
    containers: Vec<RenderedContainer>,
}

fn walk_inspect_frame(
    frame: &Frame,
    ts: Transform,
    page_num: usize,
    state: &mut FrameWalkState,
) {
    for (point, item) in frame.items() {
        let item_ts = ts.pre_concat(Transform::translate(point.x, point.y));
        match item {
            FrameItem::Tag(Tag::Start(elem, _)) => {
                if let Some(meta) = elem.to_packed::<MetadataElem>()
                    && let TypstValue::Str(s) = &meta.value
                    && let Some(rest) = s.as_str().strip_prefix("botox-src:")
                {
                    let mut parts = rest.splitn(2, ':');
                    if let Some(line_str) = parts.next()
                        && let Ok(line_num) = line_str.parse::<usize>()
                    {
                        let kind = parts.next().map(|k| k.to_string());
                        if line_num == 0 && kind.as_deref() == Some("end") {
                            state.cur_src_line = None;
                            state.cur_src_kind = None;
                        } else {
                            state.cur_src_line = Some(line_num);
                            state.cur_src_kind = kind;
                        }
                    }
                    state.tag_stack.push(ActiveTagKind::Other);
                } else if elem.to_packed::<TableElem>().is_some() {
                    state.in_table_depth += 1;
                    state.tag_stack.push(ActiveTagKind::Table);
                } else if elem.to_packed::<HeadingElem>().is_some() {
                    state.cur_src_kind = Some("heading".to_string());
                    state.tag_stack.push(ActiveTagKind::Heading);
                } else {
                    state.tag_stack.push(ActiveTagKind::Other);
                }
            }
            FrameItem::Tag(Tag::End(_, _, _)) => {
                if let Some(kind) = state.tag_stack.pop() {
                    match kind {
                        ActiveTagKind::Table => {
                            state.in_table_depth = state.in_table_depth.saturating_sub(1);
                        }
                        ActiveTagKind::Heading => {
                            state.cur_src_kind = None;
                        }
                        ActiveTagKind::Other => {}
                    }
                }
            }
            FrameItem::Text(text_item) => {
                let text = text_item.text.as_str();
                if text.trim().is_empty() {
                    continue;
                }
                let x = item_ts.tx.to_pt();
                let y_base = item_ts.ty.to_pt();
                let size = (text_item.size.to_pt() * item_ts.sy.get()).abs().max(0.5);
                let width = (text_item.width().to_pt() * item_ts.sx.get()).abs();
                let top_y = y_base - 0.78 * size;
                let bbox = BBox::new(x, top_y, width, size);
                let font_name = text_item.font.info().family.to_string();
                let has_missing_glyph = text_item.glyphs.iter().any(|g| g.id == 0);
                let is_math = font_name.contains("Math") || font_name.contains("Symbol");

                state.runs.push(RawTextRun {
                    x,
                    y_baseline: y_base,
                    bbox,
                    text: text.to_string(),
                    font_name,
                    font_size_pt: round2(size),
                    has_missing_glyph,
                    is_math,
                    in_table: state.in_table_depth > 0,
                    col: 1,
                    src_line: state.cur_src_line,
                    src_kind: state.cur_src_kind.clone(),
                });
            }
            FrameItem::Shape(shape, _) => {
                let r = shape.bbox(false);
                let sz = r.size();
                let x = item_ts.tx.to_pt() + r.min.x.to_pt() * item_ts.sx.get();
                let y = item_ts.ty.to_pt() + r.min.y.to_pt() * item_ts.sy.get();
                let w = (sz.x.to_pt() * item_ts.sx.get()).abs();
                let h = (sz.y.to_pt() * item_ts.sy.get()).abs();
                if w > 0.5 || h > 0.5 {
                    state.containers.push(RenderedContainer {
                        kind: "shape".to_string(),
                        page: page_num,
                        col: 1,
                        bbox: BBox::new(x, y, w, h),
                        is_filled_rect: shape.fill.is_some() && w > 8.0 && h > 6.0,
                        src_line: state.cur_src_line,
                    });
                }
            }
            FrameItem::Image(_, size, _) => {
                let x = item_ts.tx.to_pt();
                let y = item_ts.ty.to_pt();
                let w = (size.x.to_pt() * item_ts.sx.get()).abs();
                let h = (size.y.to_pt() * item_ts.sy.get()).abs();
                state.containers.push(RenderedContainer {
                    kind: "image".to_string(),
                    page: page_num,
                    col: 1,
                    bbox: BBox::new(x, y, w, h),
                    is_filled_rect: false,
                    src_line: state.cur_src_line,
                });
            }
            FrameItem::Group(group) => {
                let group_ts = item_ts.pre_concat(group.transform);
                walk_inspect_frame(&group.frame, group_ts, page_num, state);
            }
            _ => {}
        }
    }
}

/// Extracts structured geometry, lines, blocks, columns, and font telemetry from a compiled Typst `PagedDocument`.
pub fn extract_document_geometry(
    doc: &typst_layout::PagedDocument,
    expected_columns: usize,
) -> ExtractedDocument {
    let mut pages_summary = Vec::new();
    let mut all_lines = Vec::new();
    let mut all_blocks = Vec::new();
    let mut all_containers = Vec::new();
    let mut raw_runs_by_page = Vec::new();
    let mut font_map: BTreeMap<String, (Vec<f64>, usize, usize)> = BTreeMap::new();
    let mut next_block_id = 1usize;

    let mut carry_src_line: Option<usize> = None;
    let mut carry_src_kind: Option<String> = None;
    let mut carry_table_depth: usize = 0;

    for (page_idx, page) in doc.pages().iter().enumerate() {
        let page_num = page_idx + 1;
        let width_pt = page.frame.width().to_pt().max(1.0);
        let height_pt = page.frame.height().to_pt().max(1.0);

        let mut state = FrameWalkState {
            cur_src_line: carry_src_line,
            cur_src_kind: carry_src_kind.clone(),
            tag_stack: (0..carry_table_depth).map(|_| ActiveTagKind::Table).collect(),
            in_table_depth: carry_table_depth,
            runs: Vec::new(),
            containers: Vec::new(),
        };
        walk_inspect_frame(&page.frame, Transform::identity(), page_num, &mut state);
        carry_src_line = state.cur_src_line;
        carry_src_kind = state.cur_src_kind.clone();
        carry_table_depth = state.in_table_depth;

        for run in &state.runs {
            let entry = font_map
                .entry(run.font_name.clone())
                .or_insert_with(|| (Vec::new(), 0, 0));
            if !entry.0.iter().any(|&s| (s - run.font_size_pt).abs() < 0.05) {
                entry.0.push(run.font_size_pt);
            }
            entry.1 += 1;
            if run.has_missing_glyph {
                entry.2 += 1;
            }
        }

        // Detect column boundaries directly from raw text runs (excluding page numbers) BEFORE line grouping
        let col_bounds = detect_column_bounds_from_runs(&state.runs, width_pt, height_pt, expected_columns);

        // Assign column to each raw run & container before line grouping so adjacent columns NEVER merge
        for run in &mut state.runs {
            run.col = assign_column(run.bbox.x + run.bbox.w * 0.25, &col_bounds);
        }
        for c in &mut state.containers {
            c.col = assign_column(c.bbox.x + c.bbox.w * 0.5, &col_bounds);
        }

        // Group text runs into visual lines per column
        let page_lines_with_col = group_runs_into_lines(&state.runs, page_num);

        // Group non-page-number lines into logical blocks per column using true column bounds
        let content_lines: Vec<TextLine> = page_lines_with_col
            .iter()
            .filter(|l| !is_likely_page_number(l, height_pt))
            .cloned()
            .collect();
        let page_blocks = group_lines_into_blocks(&content_lines, &col_bounds, &mut next_block_id);

        // Compute content bounding box (filtering out tiny page numbers near bottom edge if content exists above)
        let content_bbox = compute_page_content_bbox(&page_lines_with_col, &state.containers, width_pt, height_pt);

        let usable_top = content_bbox.y.min(height_pt * 0.12);
        let usable_bottom = (height_pt - usable_top).max(height_pt * 0.85);
        let usable_h = (usable_bottom - usable_top).max(1.0);

        let mut col_stats = Vec::new();
        for (idx, &(c_min, c_max)) in col_bounds.iter().enumerate() {
            let col_num = idx + 1;
            let col_bottom = page_lines_with_col
                .iter()
                .filter(|l| l.col == col_num && !is_likely_page_number(l, height_pt))
                .map(|l| l.bbox.bottom())
                .chain(
                    state
                        .containers
                        .iter()
                        .filter(|c| c.col == col_num && c.kind == "image")
                        .map(|c| c.bbox.bottom()),
                )
                .fold(usable_top, f64::max);

            let used_h = (col_bottom - usable_top).clamp(0.0, usable_h);
            let fill_pct = round2((used_h / usable_h) * 100.0);
            let dead_pt = round2((usable_bottom - col_bottom).max(0.0));
            let block_count = page_blocks.iter().filter(|b| b.col == col_num).count();

            col_stats.push(ColumnStats {
                col: col_num,
                x_min_pt: round2(c_min),
                x_max_pt: round2(c_max),
                content_bottom_y: round2(col_bottom),
                fill_pct,
                bottom_dead_space_pt: dead_pt,
                block_count,
            });
        }

        let page_fill_pct = if col_stats.is_empty() {
            0.0
        } else {
            let sum_fill: f64 = col_stats.iter().map(|c| c.fill_pct).sum();
            round2((sum_fill / col_stats.len() as f64).clamp(0.0, 100.0))
        };
        let page_dead_pt = if col_stats.is_empty() {
            round2(usable_h)
        } else {
            let sum_dead: f64 = col_stats.iter().map(|c| c.bottom_dead_space_pt).sum();
            round2((sum_dead / col_stats.len() as f64).max(0.0))
        };

        pages_summary.push(PageSummary {
            page: page_num,
            width_pt: round2(width_pt),
            height_pt: round2(height_pt),
            content_bbox_pt: content_bbox.to_array(),
            fill_pct: page_fill_pct,
            bottom_dead_space_pt: page_dead_pt,
            columns: col_stats,
            block_count: page_blocks.len(),
            line_count: page_lines_with_col.len(),
        });

        all_lines.extend(page_lines_with_col);
        all_blocks.extend(page_blocks);
        all_containers.extend(state.containers);
        raw_runs_by_page.push(state.runs);
    }

    let fonts_used = font_map
        .into_iter()
        .map(|(family, (mut sizes_pt, glyph_runs, missing_glyph_runs))| {
            sizes_pt.sort_by(|a, b| a.total_cmp(b));
            FontTelemetry {
                family,
                sizes_pt,
                glyph_runs,
                missing_glyph_runs,
            }
        })
        .collect();

    ExtractedDocument {
        pages: pages_summary,
        lines: all_lines,
        blocks: all_blocks,
        containers: all_containers,
        fonts_used,
        raw_runs_by_page,
    }
}

fn is_likely_page_number(line: &TextLine, page_height: f64) -> bool {
    line.y_baseline > page_height * 0.92
        && line.text.trim().len() <= 5
        && line.text.trim().chars().all(|c| c.is_ascii_digit())
}

fn is_likely_page_number_run(run: &RawTextRun, page_height: f64) -> bool {
    run.y_baseline > page_height * 0.92
        && run.text.trim().len() <= 5
        && run.text.trim().chars().all(|c| c.is_ascii_digit())
}

fn is_bullet_marker(s: &str) -> bool {
    matches!(s.trim(), "•" | "‣" | "◦" | "▪" | "–" | "-")
}

fn group_runs_into_lines(runs: &[RawTextRun], page_num: usize) -> Vec<TextLine> {
    if runs.is_empty() {
        return Vec::new();
    }
    let mut sorted: Vec<&RawTextRun> = runs.iter().collect();
    sorted.sort_by(|a, b| {
        a.col
            .cmp(&b.col)
            .then(a.y_baseline.total_cmp(&b.y_baseline))
            .then(a.x.total_cmp(&b.x))
    });

    // Cluster runs within the SAME column that share a baseline (within 2.2pt) and reasonable horizontal gap
    let mut lines: Vec<Vec<&RawTextRun>> = Vec::new();
    for run in sorted {
        let max_gap = (run.font_size_pt * 2.2).max(18.0);
        let matched = lines.iter_mut().rev().take(8).find(|grp| {
            let first = grp[0];
            let last = grp[grp.len() - 1];
            first.col == run.col
                && (first.y_baseline - run.y_baseline).abs() <= 2.2
                && run.x >= last.bbox.right() - 2.0
                && (run.x - last.bbox.right()) <= max_gap
        });
        if let Some(grp) = matched {
            grp.push(run);
        } else {
            lines.push(vec![run]);
        }
    }

    let mut out = Vec::with_capacity(lines.len());
    for mut grp in lines {
        grp.sort_by(|a, b| a.x.total_cmp(&b.x));
        let mut bbox = grp[0].bbox;
        let mut text = String::new();
        let mut prev_right = grp[0].bbox.x;
        let mut y_sum = 0.0;
        let mut size_max = 0.0f64;
        let mut is_math = false;
        let mut in_table = false;
        let mut src_line = None;
        let mut src_kind = None;

        for (idx, r) in grp.iter().enumerate() {
            bbox = bbox.union(r.bbox);
            if idx > 0 && r.bbox.x - prev_right > r.font_size_pt * 0.12 {
                text.push(' ');
            }
            text.push_str(r.text.trim());
            prev_right = r.bbox.right();
            y_sum += r.y_baseline;
            size_max = size_max.max(r.font_size_pt);
            if r.is_math {
                is_math = true;
            }
            if r.in_table {
                in_table = true;
            }
            // Prefer non-bullet runs for source-line attribution so `- #metadata(...)` binds to the item's line
            if !is_bullet_marker(&r.text) && r.src_line.is_some() && src_line.is_none() {
                src_line = r.src_line;
                src_kind = r.src_kind.clone();
            }
        }
        if src_line.is_none() {
            src_line = grp[0].src_line;
            src_kind = grp[0].src_kind.clone();
        }
        let trimmed = text.trim().to_string();
        let word_count = trimmed.split_whitespace().count();
        out.push(TextLine {
            page: page_num,
            col: grp[0].col,
            y_baseline: round2(y_sum / grp.len() as f64),
            bbox,
            text: trimmed,
            word_count,
            font_size_pt: round2(size_max),
            is_math,
            in_table,
            src_line,
            src_kind,
        });
    }
    out
}

fn detect_column_bounds_from_runs(
    runs: &[RawTextRun],
    page_width: f64,
    page_height: f64,
    expected_columns: usize,
) -> Vec<(f64, f64)> {
    let content_runs: Vec<&RawTextRun> = runs
        .iter()
        .filter(|r| !is_likely_page_number_run(r, page_height))
        .collect();

    let cols = expected_columns.max(1);

    let min_x = content_runs
        .iter()
        .map(|r| r.bbox.x)
        .fold(f64::INFINITY, f64::min);
    let max_x = content_runs.iter().map(|r| r.bbox.right()).fold(0.0f64, f64::max);
    let left_margin = if min_x.is_finite() && min_x < page_width * 0.25 {
        min_x
    } else {
        20.0
    };
    let right_edge = (page_width - left_margin).max(max_x);

    if cols <= 1 {
        return vec![(left_margin, right_edge)];
    }

    let span = (right_edge - left_margin).max(page_width * 0.6);
    let col_w = span / cols as f64;
    (0..cols)
        .map(|c| {
            let x0 = left_margin + c as f64 * col_w;
            let x1 = x0 + col_w;
            (x0, x1)
        })
        .collect()
}

fn assign_column(x_probe: f64, bounds: &[(f64, f64)]) -> usize {
    for (idx, &(x0, x1)) in bounds.iter().enumerate() {
        if x_probe >= x0 && x_probe <= x1 {
            return idx + 1;
        }
    }
    if x_probe < bounds.first().map(|b| b.0).unwrap_or(0.0) {
        1
    } else {
        bounds.len().max(1)
    }
}

fn group_lines_into_blocks(
    lines: &[TextLine],
    col_bounds: &[(f64, f64)],
    next_id: &mut usize,
) -> Vec<LayoutBlock> {
    let mut by_col: BTreeMap<usize, Vec<&TextLine>> = BTreeMap::new();
    for line in lines {
        by_col.entry(line.col).or_default().push(line);
    }

    let mut blocks = Vec::new();
    for (col, mut col_lines) in by_col {
        col_lines.sort_by(|a, b| a.y_baseline.total_cmp(&b.y_baseline));
        if col_lines.is_empty() {
            continue;
        }
        let col_width = col_bounds
            .get(col.saturating_sub(1))
            .map(|(x0, x1)| (x1 - x0).max(40.0))
            .unwrap_or_else(|| {
                let x0 = col_lines.iter().map(|l| l.bbox.x).fold(f64::INFINITY, f64::min);
                let x1 = col_lines.iter().map(|l| l.bbox.right()).fold(0.0f64, f64::max);
                (x1 - x0).max(40.0)
            });

        let mut groups: Vec<Vec<&TextLine>> = vec![vec![col_lines[0]]];
        for &line in col_lines.iter().skip(1) {
            let prev_grp = groups.last().unwrap();
            let prev = prev_grp.last().unwrap();
            let dy = line.y_baseline - prev.y_baseline;
            let same_src = line.src_line.is_some() && line.src_line == prev.src_line;
            let starts_bullet = line.text.starts_with('•')
                || line.text.starts_with('-')
                || line.text.starts_with('‣')
                || line.text.starts_with('◦');
            let font_changed = (line.font_size_pt - prev.font_size_pt).abs() > 1.0;
            let big_gap = dy > prev.font_size_pt * 1.75;
            let different_src =
                line.src_line.is_some() && prev.src_line.is_some() && line.src_line != prev.src_line;

            if same_src || (!starts_bullet && !font_changed && !big_gap && !different_src) {
                groups.last_mut().unwrap().push(line);
            } else {
                groups.push(vec![line]);
            }
        }

        for grp in groups {
            let mut bbox = grp[0].bbox;
            for l in grp.iter().skip(1) {
                bbox = bbox.union(l.bbox);
            }
            let last = grp.last().unwrap();
            let last_fill = round2(((last.bbox.w / col_width) * 100.0).clamp(0.0, 100.0));
            let mut deltas: Vec<f64> = grp
                .windows(2)
                .map(|w| (w[1].y_baseline - w[0].y_baseline).max(0.0))
                .collect();
            deltas.sort_by(|a, b| a.total_cmp(b));
            let median_leading = if deltas.is_empty() {
                0.0
            } else {
                round2(deltas[deltas.len() / 2])
            };

            let kind = grp[0]
                .src_kind
                .clone()
                .unwrap_or_else(|| {
                    if grp[0].text.starts_with('•') || grp[0].text.starts_with('‣') {
                        "list_item".to_string()
                    } else if grp[0].font_size_pt >= 11.5 && grp.len() <= 2 {
                        "heading".to_string()
                    } else {
                        "paragraph".to_string()
                    }
                });

            let joined: String = grp
                .iter()
                .map(|l| l.text.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            let preview: String = joined.chars().take(72).collect();

            blocks.push(LayoutBlock {
                id: *next_id,
                kind,
                page: grp[0].page,
                col,
                bbox,
                line_count: grp.len(),
                last_line_fill_pct: last_fill,
                last_line_words: last.word_count,
                median_leading_pt: median_leading,
                text_preview: preview,
                src_line: grp[0].src_line,
            });
            *next_id += 1;
        }
    }

    blocks.sort_by(|a, b| {
        a.page
            .cmp(&b.page)
            .then(a.col.cmp(&b.col))
            .then(a.bbox.y.total_cmp(&b.bbox.y))
    });
    blocks
}

fn compute_page_content_bbox(
    lines: &[TextLine],
    containers: &[RenderedContainer],
    page_w: f64,
    page_h: f64,
) -> BBox {
    let mut acc: Option<BBox> = None;
    for l in lines {
        if is_likely_page_number(l, page_h) {
            continue;
        }
        acc = Some(match acc {
            Some(b) => b.union(l.bbox),
            None => l.bbox,
        });
    }
    for c in containers {
        if c.kind == "image" || c.is_filled_rect {
            acc = Some(match acc {
                Some(b) => b.union(c.bbox),
                None => c.bbox,
            });
        }
    }
    acc.unwrap_or_else(|| BBox::new(0.0, 0.0, page_w, page_h))
}

/// Runs full wasted-space and layout-health diagnostics on an extracted document.
pub fn analyze_layout(
    doc: &ExtractedDocument,
    file_label: &str,
    target_pages: Option<usize>,
) -> InspectReport {
    analyze_layout_with_source(doc, file_label, target_pages, None)
}

/// Runs full wasted-space and layout-health diagnostics with optional raw Markdown source
/// (used to attribute explicit `\newpage` / `\columnbreak` causes on `UNDERFULL_COLUMN`).
pub fn analyze_layout_with_source(
    doc: &ExtractedDocument,
    file_label: &str,
    target_pages: Option<usize>,
    raw_markdown: Option<&str>,
) -> InspectReport {
    let mut issues: Vec<LayoutIssue> = Vec::new();

    for page in &doc.pages {
        let page_lines: Vec<&TextLine> = doc
            .lines
            .iter()
            .filter(|l| l.page == page.page && !is_likely_page_number(l, page.height_pt))
            .collect();
        let page_blocks: Vec<&LayoutBlock> = doc
            .blocks
            .iter()
            .filter(|b| b.page == page.page)
            .collect();

        // 1. SHORT_LAST_LINE (widow words at end of multi-line paragraph/bullet)
        for blk in &page_blocks {
            if blk.line_count >= 2
                && blk.kind != "heading"
                && blk.kind != "code"
                && (blk.last_line_words <= 2 || blk.last_line_fill_pct < 24.0)
            {
                let lines_in_blk: Vec<&&TextLine> = page_lines
                    .iter()
                    .filter(|l| l.col == blk.col && l.bbox.y >= blk.bbox.y - 1.0 && l.bbox.bottom() <= blk.bbox.bottom() + 1.0)
                    .collect();
                if lines_in_blk.last().is_some_and(|l| l.is_math || l.in_table) {
                    continue;
                }
                let last_text = lines_in_blk
                    .last()
                    .map(|l| l.text.clone())
                    .unwrap_or_else(|| blk.text_preview.clone());
                let line_h = if blk.median_leading_pt > 0.0 {
                    blk.median_leading_pt
                } else {
                    10.0
                };
                let words = blk.last_line_words.max(1);
                issues.push(LayoutIssue {
                    code: "SHORT_LAST_LINE".to_string(),
                    severity: "hint".to_string(),
                    page: page.page,
                    col: blk.col,
                    bbox_pt: blk.bbox.to_array(),
                    src_line: blk.src_line,
                    text_excerpt: last_text.clone(),
                    reclaimable_pt: Some(round2(line_h)),
                    metrics: serde_json::json!({
                        "lines": blk.line_count,
                        "last_line_words": blk.last_line_words,
                        "last_line_fill_pct": blk.last_line_fill_pct,
                    }),
                    suggestion: format!(
                        "Trim ~{} word(s) (\"{}\"){} to reclaim 1 full line ({:.1}pt).",
                        words,
                        last_text,
                        blk.src_line
                            .map(|n| format!(" near line {n}"))
                            .unwrap_or_default(),
                        line_h
                    ),
                });
            }
        }

        // 2. INFLATED_LEADING (inline code padding or tall math stretching a line)
        let all_deltas: Vec<f64> = page_blocks
            .iter()
            .filter(|b| b.median_leading_pt > 0.0)
            .map(|b| b.median_leading_pt)
            .collect();
        let global_median_leading = median_f64(&all_deltas);
        if global_median_leading > 0.0 {
            let mut by_col: BTreeMap<usize, Vec<&TextLine>> = BTreeMap::new();
            for l in &page_lines {
                by_col.entry(l.col).or_default().push(l);
            }
            for (col, mut clines) in by_col {
                clines.sort_by(|a, b| a.y_baseline.total_cmp(&b.y_baseline));
                for w in clines.windows(2) {
                    if w[0].in_table || w[1].in_table {
                        continue;
                    }
                    let dy = w[1].y_baseline - w[0].y_baseline;
                    let same_block = (w[1].font_size_pt - w[0].font_size_pt).abs() < 0.8
                        && (w[1].src_line == w[0].src_line || w[1].src_line.is_none());
                    if same_block && dy > global_median_leading * 1.28 && dy < global_median_leading * 2.2 {
                        let wasted = round2(dy - global_median_leading);
                        issues.push(LayoutIssue {
                            code: "INFLATED_LEADING".to_string(),
                            severity: "hint".to_string(),
                            page: page.page,
                            col,
                            bbox_pt: w[1].bbox.to_array(),
                            src_line: w[1].src_line,
                            text_excerpt: w[1].text.chars().take(60).collect(),
                            reclaimable_pt: Some(wasted),
                            metrics: serde_json::json!({
                                "line_delta_pt": round2(dy),
                                "median_leading_pt": round2(global_median_leading),
                                "wasted_pt": wasted,
                            }),
                            suggestion: format!(
                                "Line spacing stretched to {:.1}pt vs {:.1}pt baseline (likely tall inline code/math); {:.1}pt extra vertical height.",
                                dy, global_median_leading, wasted
                            ),
                        });
                    }
                }
            }
        }

        // 3. CONTENT_OVERFLOW & GUTTER_INTRUSION
        for c_stat in &page.columns {
            let col_limit_x = c_stat.x_max_pt + 2.0;
            for line in page_lines.iter().filter(|l| l.col == c_stat.col) {
                if line.bbox.right() > page.width_pt - 4.0 {
                    issues.push(LayoutIssue {
                        code: "CONTENT_OVERFLOW".to_string(),
                        severity: "error".to_string(),
                        page: page.page,
                        col: c_stat.col,
                        bbox_pt: line.bbox.to_array(),
                        src_line: line.src_line,
                        text_excerpt: line.text.chars().take(60).collect(),
                        reclaimable_pt: None,
                        metrics: serde_json::json!({
                            "right_pt": round2(line.bbox.right()),
                            "page_width_pt": page.width_pt,
                        }),
                        suggestion: "Text extends past page edge; reduce font/code size or wrap long tokens.".to_string(),
                    });
                } else if page.columns.len() > 1
                    && c_stat.col < page.columns.len()
                    && line.bbox.right() > col_limit_x
                {
                    issues.push(LayoutIssue {
                        code: "GUTTER_INTRUSION".to_string(),
                        severity: "warn".to_string(),
                        page: page.page,
                        col: c_stat.col,
                        bbox_pt: line.bbox.to_array(),
                        src_line: line.src_line,
                        text_excerpt: line.text.chars().take(60).collect(),
                        reclaimable_pt: None,
                        metrics: serde_json::json!({
                            "right_pt": round2(line.bbox.right()),
                            "col_max_pt": c_stat.x_max_pt,
                        }),
                        suggestion: "Line crosses into column gutter; shorten unbreakable inline code or shrink code-fontsize.".to_string(),
                    });
                }
            }
        }

        // 4. COLUMN_IMBALANCE & BOTTOM_DEAD_SPACE
        if page.columns.len() >= 2 {
            let min_fill = page.columns.iter().map(|c| c.fill_pct).fold(100.0f64, f64::min);
            let max_fill = page.columns.iter().map(|c| c.fill_pct).fold(0.0f64, f64::max);
            if max_fill - min_fill > 22.0 && page.page < doc.pages.len() {
                issues.push(LayoutIssue {
                    code: "COLUMN_IMBALANCE".to_string(),
                    severity: "warn".to_string(),
                    page: page.page,
                    col: 1,
                    bbox_pt: page.content_bbox_pt,
                    src_line: None,
                    text_excerpt: format!("Columns fill range: {:.1}% .. {:.1}%", min_fill, max_fill),
                    reclaimable_pt: None,
                    metrics: serde_json::json!({
                        "min_fill_pct": min_fill,
                        "max_fill_pct": max_fill,
                        "delta_pct": round2(max_fill - min_fill),
                    }),
                    suggestion: "Column heights are imbalanced; consider inserting `\\columnbreak` or adjusting block sizes.".to_string(),
                });
            }
        }

        // 4b. UNDERFULL_COLUMN on non-final columns with large trailing dead space (> 80pt & < 88% fill)
        for col_stat in &page.columns {
            let is_final_col_of_doc =
                page.page == doc.pages.len() && col_stat.col == page.columns.len();
            if is_final_col_of_doc || col_stat.block_count == 0 {
                continue;
            }
            if col_stat.bottom_dead_space_pt > 80.0 && col_stat.fill_pct < 88.0 {
                let mut col_blks: Vec<&&LayoutBlock> = page_blocks
                    .iter()
                    .filter(|b| b.col == col_stat.col)
                    .collect();
                col_blks.sort_by(|a, b| a.bbox.y.total_cmp(&b.bbox.y));
                let last_blk_line = col_blks.last().and_then(|b| b.src_line);

                // Find the next column's blocks (either next col on same page, or col 1 on next page)
                let (next_page, next_col) = if col_stat.col < page.columns.len() {
                    (page.page, col_stat.col + 1)
                } else {
                    (page.page + 1, 1)
                };
                let mut next_col_blks: Vec<&LayoutBlock> = doc
                    .blocks
                    .iter()
                    .filter(|b| b.page == next_page && b.col == next_col)
                    .collect();
                next_col_blks.sort_by(|a, b| a.bbox.y.total_cmp(&b.bbox.y));
                let next_first = next_col_blks.first().copied();
                let next_first_line = next_first.and_then(|b| b.src_line);

                // Check raw Markdown between last_blk_line and next_first_line for explicit break commands
                let mut explicit_break: Option<(usize, &'static str)> = None;
                if let (Some(md), Some(start_ln)) = (raw_markdown, last_blk_line) {
                    let end_ln = next_first_line.unwrap_or(start_ln + 40);
                    for (idx, line_text) in md.lines().enumerate() {
                        let ln = idx + 1;
                        if ln <= start_ln || ln >= end_ln {
                            continue;
                        }
                        let trimmed = line_text.trim();
                        if trimmed == r"\newpage" {
                            explicit_break = Some((ln, r"\newpage"));
                            break;
                        } else if trimmed == r"\pagebreak" {
                            explicit_break = Some((ln, r"\pagebreak"));
                            break;
                        } else if trimmed == r"\columnbreak" {
                            explicit_break = Some((ln, r"\columnbreak"));
                            break;
                        }
                    }
                }

                let (cause_src_line, cause_str) = if let Some((brk_ln, brk_cmd)) = explicit_break {
                    (
                        Some(brk_ln),
                        format!("explicit `{brk_cmd}` at md:{brk_ln}"),
                    )
                } else if let Some(nf) = next_first {
                    // If the next column starts with a sticky heading followed by a table/code/para,
                    // estimate the combined indivisible height.
                    let (unit_kind, unit_h) = if nf.kind == "heading" {
                        if let Some(second) = next_col_blks.get(1) {
                            let combined = (second.bbox.bottom() - nf.bbox.y).max(nf.bbox.h);
                            (format!("heading + {}", second.kind), round2(combined))
                        } else {
                            (nf.kind.clone(), round2(nf.bbox.h))
                        }
                    } else {
                        (nf.kind.clone(), round2(nf.bbox.h))
                    };
                    let loc = nf.src_line.map(|l| format!(" at md:{l}")).unwrap_or_default();
                    (
                        nf.src_line.or(last_blk_line),
                        format!("next indivisible {unit_kind} (~{unit_h:.1}pt{loc}) didn't fit in {:.1}pt", col_stat.bottom_dead_space_pt),
                    )
                } else {
                    (last_blk_line, "large trailing gap before next column/page".to_string())
                };

                let severity = if col_stat.bottom_dead_space_pt >= 110.0 || col_stat.fill_pct < 82.0 {
                    "warn"
                } else {
                    "hint"
                };

                let dead_bbox = BBox::new(
                    col_stat.x_min_pt,
                    col_stat.content_bottom_y,
                    (col_stat.x_max_pt - col_stat.x_min_pt).max(10.0),
                    col_stat.bottom_dead_space_pt,
                );
                issues.push(LayoutIssue {
                    code: "UNDERFULL_COLUMN".to_string(),
                    severity: severity.to_string(),
                    page: page.page,
                    col: col_stat.col,
                    bbox_pt: dead_bbox.to_array(),
                    src_line: cause_src_line,
                    text_excerpt: format!(
                        "p{} col {} is {:.1}% full ({:.1}pt dead space)",
                        page.page, col_stat.col, col_stat.fill_pct, col_stat.bottom_dead_space_pt
                    ),
                    reclaimable_pt: Some(round2(col_stat.bottom_dead_space_pt)),
                    metrics: serde_json::json!({
                        "fill_pct": col_stat.fill_pct,
                        "bottom_dead_space_pt": col_stat.bottom_dead_space_pt,
                        "explicit_break": explicit_break.map(|(_, c)| c),
                    }),
                    suggestion: format!(
                        "Column has {:.1}pt dead space ({:.1}% fill) — cause: {cause_str}. Remove blunt breaks, reorder content, or allow splitting.",
                        col_stat.bottom_dead_space_pt, col_stat.fill_pct
                    ),
                });
            }
        }

        // 5. ORPHAN_HEADING (heading near bottom of column with no following block in same column)
        for col_stat in &page.columns {
            let mut col_blks: Vec<&&LayoutBlock> = page_blocks
                .iter()
                .filter(|b| b.col == col_stat.col)
                .collect();
            col_blks.sort_by(|a, b| a.bbox.y.total_cmp(&b.bbox.y));
            if let Some(last_blk) = col_blks.last()
                && last_blk.kind == "heading"
                && last_blk.line_count <= 2
                && last_blk.bbox.h <= 42.0
                && (page.page < doc.pages.len() || col_stat.col < page.columns.len())
            {
                issues.push(LayoutIssue {
                    code: "ORPHAN_HEADING".to_string(),
                    severity: "warn".to_string(),
                    page: page.page,
                    col: col_stat.col,
                    bbox_pt: last_blk.bbox.to_array(),
                    src_line: last_blk.src_line,
                    text_excerpt: last_blk.text_preview.clone(),
                    reclaimable_pt: None,
                    metrics: serde_json::json!({ "y_pt": round2(last_blk.bbox.y) }),
                    suggestion: "Heading appears stranded at the bottom of the column/page with no following body content.".to_string(),
                });
            }
        }

        // 6. OVERSIZED_PADDING on filled containers (code blocks / callouts)
        for c in doc.containers.iter().filter(|c| c.page == page.page && c.is_filled_rect) {
            let inner_lines: Vec<&&TextLine> = page_lines
                .iter()
                .filter(|l| c.bbox.intersects(l.bbox))
                .collect();
            if !inner_lines.is_empty() {
                let mut inner_box = inner_lines[0].bbox;
                for l in inner_lines.iter().skip(1) {
                    inner_box = inner_box.union(l.bbox);
                }
                let pad_h = (c.bbox.h - inner_box.h).max(0.0);
                let pad_ratio = pad_h / c.bbox.h.max(1.0);
                if pad_ratio > 0.55 && pad_h > 14.0 && c.bbox.h > 24.0 {
                    issues.push(LayoutIssue {
                        code: "OVERSIZED_PADDING".to_string(),
                        severity: "hint".to_string(),
                        page: page.page,
                        col: c.col,
                        bbox_pt: c.bbox.to_array(),
                        src_line: inner_lines[0].src_line.or(c.src_line),
                        text_excerpt: inner_lines[0].text.chars().take(50).collect(),
                        reclaimable_pt: Some(round2(pad_h * 0.5)),
                        metrics: serde_json::json!({
                            "container_h_pt": round2(c.bbox.h),
                            "text_h_pt": round2(inner_box.h),
                            "padding_pct": round2(pad_ratio * 100.0),
                        }),
                        suggestion: format!(
                            "Container padding uses {:.0}% ({:.1}pt) of block height; consider tighter spacing or inline code.",
                            pad_ratio * 100.0, pad_h
                        ),
                    });
                }
            }
        }

        // 7. TINY_IMAGE / CLIPPED_IMAGE
        for img in doc.containers.iter().filter(|c| c.page == page.page && c.kind == "image") {
            if img.bbox.w < 24.0 || img.bbox.h < 24.0 {
                issues.push(LayoutIssue {
                    code: "TINY_IMAGE".to_string(),
                    severity: "warn".to_string(),
                    page: page.page,
                    col: img.col,
                    bbox_pt: img.bbox.to_array(),
                    src_line: img.src_line,
                    text_excerpt: format!("Image {:.1}x{:.1}pt", img.bbox.w, img.bbox.h),
                    reclaimable_pt: None,
                    metrics: serde_json::json!({ "w_pt": round2(img.bbox.w), "h_pt": round2(img.bbox.h) }),
                    suggestion: "Image/diagram is rendered very small (<24pt); check width/height attributes.".to_string(),
                });
            }
            if img.bbox.right() > page.width_pt - 2.0 || img.bbox.bottom() > page.height_pt - 2.0 {
                issues.push(LayoutIssue {
                    code: "CLIPPED_IMAGE".to_string(),
                    severity: "error".to_string(),
                    page: page.page,
                    col: img.col,
                    bbox_pt: img.bbox.to_array(),
                    src_line: img.src_line,
                    text_excerpt: format!("Image overflows page at ({:.1}, {:.1})", img.bbox.right(), img.bbox.bottom()),
                    reclaimable_pt: None,
                    metrics: serde_json::json!({ "bbox": img.bbox.to_array() }),
                    suggestion: "Image extends beyond page bounds; constrain `width` or switch section to `::: landscape`.".to_string(),
                });
            }
        }

        // 8. TEXT_OVERLAP (intersecting non-math lines in same page & column with substantial overlap)
        for i in 0..page_lines.len() {
            let a = page_lines[i];
            if a.is_math {
                continue;
            }
            for b in page_lines.iter().skip(i + 1) {
                if b.is_math || a.col != b.col {
                    continue;
                }
                if let Some(inter) = a.bbox.intersection(b.bbox)
                    && inter.w > 4.0
                    && inter.h > a.font_size_pt.min(b.font_size_pt) * 0.60
                {
                    let excerpt_a: String = a.text.chars().take(32).collect();
                    let excerpt_b: String = b.text.chars().take(32).collect();
                    issues.push(LayoutIssue {
                        code: "TEXT_OVERLAP".to_string(),
                        severity: "error".to_string(),
                        page: page.page,
                        col: a.col,
                        bbox_pt: inter.to_array(),
                        src_line: a.src_line.or(b.src_line),
                        text_excerpt: format!("\"{excerpt_a}\" overlaps \"{excerpt_b}\""),
                        reclaimable_pt: None,
                        metrics: serde_json::json!({
                            "overlap_area_pt2": round2(inter.area()),
                            "overlap_bbox_pt": inter.to_array(),
                            "text_a": excerpt_a,
                            "text_b": excerpt_b,
                        }),
                        suggestion: format!(
                            "Text overlap at ({:.1}, {:.1}, {:.1}x{:.1}pt): \"{}\" overlaps \"{}\"; check negative spacing, wide math, or long tokens.",
                            inter.x, inter.y, inter.w, inter.h, excerpt_a, excerpt_b
                        ),
                    });
                }
            }
        }
    }

    // 8b. SPLIT_CODE_BLOCK (fenced code block split across columns or pages)
    let mut code_lines_by_src: BTreeMap<usize, Vec<&TextLine>> = BTreeMap::new();
    for line in &doc.lines {
        if line.src_kind.as_deref() == Some("code")
            && let Some(sl) = line.src_line
        {
            code_lines_by_src.entry(sl).or_default().push(line);
        }
    }
    for (src_line, clines) in code_lines_by_src {
        let mut loc_counts: BTreeMap<(usize, usize), usize> = BTreeMap::new();
        for l in &clines {
            *loc_counts.entry((l.page, l.col)).or_default() += 1;
        }
        if loc_counts.len() >= 2 {
            let first = clines[0];
            let breakdown: Vec<String> = loc_counts
                .iter()
                .map(|((p, c), cnt)| {
                    format!(
                        "{cnt} line{} on p{p} col{c}",
                        if *cnt == 1 { "" } else { "s" }
                    )
                })
                .collect();
            let split_summary = breakdown.join(", ");
            let preview: String = first.text.chars().take(50).collect();
            issues.push(LayoutIssue {
                code: "SPLIT_CODE_BLOCK".to_string(),
                severity: "warn".to_string(),
                page: first.page,
                col: first.col,
                bbox_pt: first.bbox.to_array(),
                src_line: Some(src_line),
                text_excerpt: preview.clone(),
                reclaimable_pt: None,
                metrics: serde_json::json!({
                    "fragments": loc_counts.len(),
                    "total_lines": clines.len(),
                    "split": split_summary,
                }),
                suggestion: format!(
                    "Code block at md:{src_line} (\"{}\") splits across column/page break ({}); consider adding `\\columnbreak`, trimming preceding lines, or shrinking `code-fontsize`.",
                    preview, split_summary
                ),
            });
        }
    }

    // 9. MISSING_GLYPH telemetry check
    for f in &doc.fonts_used {
        if f.missing_glyph_runs > 0 {
            issues.push(LayoutIssue {
                code: "MISSING_GLYPH".to_string(),
                severity: "warn".to_string(),
                page: 1,
                col: 1,
                bbox_pt: [0.0, 0.0, 0.0, 0.0],
                src_line: None,
                text_excerpt: format!("Font '{}' had {} run(s) with .notdef glyphs", f.family, f.missing_glyph_runs),
                reclaimable_pt: None,
                metrics: serde_json::json!({ "font": f.family, "runs": f.missing_glyph_runs }),
                suggestion: "Some characters rendered as missing glyph boxes; verify Unicode support in selected mainfont/monofont.".to_string(),
            });
        }
    }

    // 10. PAGE_SPILLOVER (final page underfilled)
    let last_page_fill_pct = doc.pages.last().map(|p| p.fill_pct).unwrap_or(0.0);
    if doc.pages.len() > 1 && last_page_fill_pct < 28.0 {
        let last = doc.pages.last().unwrap();
        let spill_pt = round2((last.fill_pct / 100.0) * (last.height_pt * 0.88));
        issues.push(LayoutIssue {
            code: "PAGE_SPILLOVER".to_string(),
            severity: "warn".to_string(),
            page: last.page,
            col: 1,
            bbox_pt: last.content_bbox_pt,
            src_line: None,
            text_excerpt: format!("Page {} is only {:.1}% full ({} lines)", last.page, last.fill_pct, last.line_count),
            reclaimable_pt: None,
            metrics: serde_json::json!({
                "last_page": last.page,
                "fill_pct": last.fill_pct,
                "spill_height_pt": spill_pt,
                "spilled_lines": last.line_count,
            }),
            suggestion: format!(
                "Final page only uses {:.1}% (~{:.1}pt, {} lines). Run with `--target-pages {}` for an exact reclaim checklist.",
                last.fill_pct, spill_pt, last.line_count, doc.pages.len() - 1
            ),
        });
    }

    // Build ReclaimPlan if --target-pages N is requested and exceeded
    let reclaim_plan = target_pages.map(|target| {
        if doc.pages.len() <= target {
            return ReclaimPlan {
                target_pages: target,
                current_pages: doc.pages.len(),
                overflow_pt: 0.0,
                total_reclaimable_pt: 0.0,
                achievable: true,
                steps: Vec::new(),
            };
        }
        // Sum vertical height spilled onto pages > target
        let overflow_pt: f64 = doc
            .pages
            .iter()
            .skip(target)
            .map(|p| {
                let col_sum: f64 = p
                    .columns
                    .iter()
                    .map(|c| (c.fill_pct / 100.0) * (p.height_pt * 0.85))
                    .sum();
                col_sum.max((p.fill_pct / 100.0) * (p.height_pt * 0.85))
            })
            .sum();
        let overflow_pt = round2(overflow_pt);

        let mut candidates: Vec<LayoutIssue> = issues
            .iter()
            .filter(|i| i.reclaimable_pt.unwrap_or(0.0) > 0.0)
            .cloned()
            .collect();
        candidates.sort_by(|a, b| {
            b.reclaimable_pt
                .unwrap_or(0.0)
                .total_cmp(&a.reclaimable_pt.unwrap_or(0.0))
        });

        let mut selected = Vec::new();
        let mut acc = 0.0;
        for c in candidates {
            acc += c.reclaimable_pt.unwrap_or(0.0);
            selected.push(c);
            if acc >= overflow_pt {
                break;
            }
        }
        ReclaimPlan {
            target_pages: target,
            current_pages: doc.pages.len(),
            overflow_pt,
            total_reclaimable_pt: round2(acc),
            achievable: acc >= overflow_pt,
            steps: selected,
        }
    });

    let mut counts = IssueCounts::default();
    for iss in &issues {
        match iss.severity.as_str() {
            "error" => counts.error += 1,
            "warn" => counts.warn += 1,
            _ => counts.hint += 1,
        }
        counts.total += 1;
    }

    InspectReport {
        file: file_label.to_string(),
        page_count: doc.pages.len(),
        last_page_fill_pct,
        fonts_used: doc.fonts_used.clone(),
        pages: doc.pages.clone(),
        issue_counts: counts,
        issues,
        reclaim_plan,
    }
}

fn median_f64(vals: &[f64]) -> f64 {
    if vals.is_empty() {
        return 0.0;
    }
    let mut v = vals.to_vec();
    v.sort_by(|a, b| a.total_cmp(b));
    v[v.len() / 2]
}

pub fn compare_reports(baseline: &InspectReport, current: &InspectReport) -> CompareDelta {
    let mut base_counts: HashMap<String, usize> = HashMap::new();
    for i in &baseline.issues {
        *base_counts.entry(i.code.clone()).or_default() += 1;
    }
    let mut cur_counts: HashMap<String, usize> = HashMap::new();
    for i in &current.issues {
        *cur_counts.entry(i.code.clone()).or_default() += 1;
    }

    let mut resolved = BTreeMap::new();
    let mut added = BTreeMap::new();
    for (code, &bc) in &base_counts {
        let cc = cur_counts.get(code).copied().unwrap_or(0);
        if bc > cc {
            resolved.insert(code.clone(), bc - cc);
        }
    }
    for (code, &cc) in &cur_counts {
        let bc = base_counts.get(code).copied().unwrap_or(0);
        if cc > bc {
            added.insert(code.clone(), cc - bc);
        }
    }

    CompareDelta {
        baseline_pages: baseline.page_count,
        current_pages: current.page_count,
        pages_delta: current.page_count as isize - baseline.page_count as isize,
        baseline_last_fill_pct: baseline.last_page_fill_pct,
        current_last_fill_pct: current.last_page_fill_pct,
        last_fill_delta_pct: round2(current.last_page_fill_pct - baseline.last_page_fill_pct),
        baseline_issues: baseline.issue_counts.total,
        current_issues: current.issue_counts.total,
        issues_delta: current.issue_counts.total as isize - baseline.issue_counts.total as isize,
        resolved_codes: resolved,
        added_codes: added,
    }
}

pub fn format_inspect_human(report: &InspectReport) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "Botox Layout Inspection: {} ({} page{}, last page {:.1}% full)",
        report.file,
        report.page_count,
        if report.page_count == 1 { "" } else { "s" },
        report.last_page_fill_pct
    );
    let _ = writeln!(
        out,
        "Issues: {} total ({} error, {} warn, {} hint)\n",
        report.issue_counts.total,
        report.issue_counts.error,
        report.issue_counts.warn,
        report.issue_counts.hint
    );

    for p in &report.pages {
        let cols_str: Vec<String> = p
            .columns
            .iter()
            .map(|c| format!("col {}: {:.1}% (dead {:.1}pt)", c.col, c.fill_pct, c.bottom_dead_space_pt))
            .collect();
        let min_col_fill = p
            .columns
            .iter()
            .map(|c| c.fill_pct)
            .fold(100.0f64, f64::min);
        let fill_display = if p.columns.len() >= 2 && min_col_fill < p.fill_pct - 0.5 {
            format!("{:.1}% (min {:.1}%)", p.fill_pct, min_col_fill)
        } else {
            format!("{:.1}%", p.fill_pct)
        };
        let _ = writeln!(
            out,
            "  Page {} ({:.0}x{:.0}pt): fill {} | {}",
            p.page,
            p.width_pt,
            p.height_pt,
            fill_display,
            cols_str.join(", ")
        );
    }

    if !report.issues.is_empty() {
        out.push('\n');
        for iss in &report.issues {
            let src = iss
                .src_line
                .map(|l| format!(" md:{l}"))
                .unwrap_or_default();
            let rec = iss
                .reclaimable_pt
                .map(|r| format!(" [reclaim ~{r:.1}pt]"))
                .unwrap_or_default();
            let _ = writeln!(
                out,
                "  [{}] {} (p{} col{}{}{}): {}",
                iss.severity.to_uppercase(),
                iss.code,
                iss.page,
                iss.col,
                src,
                rec,
                iss.suggestion
            );
        }
    }

    if let Some(ref plan) = report.reclaim_plan {
        let _ = writeln!(
            out,
            "\nReclaim Plan (target {} pages, currently {}, overflow ~{:.1}pt, reclaimable ~{:.1}pt, achievable: {}):",
            plan.target_pages,
            plan.current_pages,
            plan.overflow_pt,
            plan.total_reclaimable_pt,
            plan.achievable
        );
        for (idx, step) in plan.steps.iter().enumerate() {
            let _ = writeln!(
                out,
                "  {}. [p{} col{} +~{:.1}pt] {}",
                idx + 1,
                step.page,
                step.col,
                step.reclaimable_pt.unwrap_or(0.0),
                step.suggestion
            );
        }
    }

    out
}

// ============================================================================
// Spatial Probing (`botox probe`)
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct ProbeQuery {
    pub page: Option<usize>,
    pub region: Option<String>,
    pub zone: Option<String>,
    pub around: Option<String>,
    pub radius_pt: f64,
    pub at_line: Option<usize>,
    pub measure_gap: Option<(String, String)>,
    pub density_grid: Option<(usize, usize)>,
    pub list_blocks: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProbeMatch {
    pub kind: String,
    pub page: usize,
    pub col: usize,
    pub bbox_pt: [f64; 4],
    pub src_line: Option<usize>,
    pub text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GapMeasurement {
    pub from_text: String,
    pub to_text: String,
    pub same_page: bool,
    pub dx_pt: f64,
    pub dy_pt: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DensityGridOutput {
    pub page: usize,
    pub cols: usize,
    pub rows: usize,
    pub ascii: Vec<String>,
    pub matrix: Vec<Vec<f64>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProbeReport {
    pub file: String,
    pub query: String,
    pub page: Option<usize>,
    pub probed_bbox_pt: Option<[f64; 4]>,
    pub coverage_pct: Option<f64>,
    pub empty_space_pct: Option<f64>,
    pub matches: Vec<ProbeMatch>,
    pub blocks: Vec<LayoutBlock>,
    pub gap: Option<GapMeasurement>,
    pub density_grid: Option<DensityGridOutput>,
}

fn parse_dim(token: &str, axis_len: f64) -> Result<f64, String> {
    let t = token.trim().to_ascii_lowercase();
    if let Some(p) = t.strip_suffix('%') {
        let pct: f64 = p.parse().map_err(|_| format!("Invalid percentage '{token}'"))?;
        Ok((pct / 100.0) * axis_len)
    } else if let Some(mm) = t.strip_suffix("mm") {
        let v: f64 = mm.parse().map_err(|_| format!("Invalid mm '{token}'"))?;
        Ok(v * PT_PER_MM)
    } else if let Some(cm) = t.strip_suffix("cm") {
        let v: f64 = cm.parse().map_err(|_| format!("Invalid cm '{token}'"))?;
        Ok(v * PT_PER_CM)
    } else if let Some(pt) = t.strip_suffix("pt") {
        pt.parse().map_err(|_| format!("Invalid pt '{token}'"))
    } else {
        t.parse().map_err(|_| format!("Invalid dimension '{token}'"))
    }
}

pub fn resolve_zone_bbox(zone: &str, pages: &[PageSummary]) -> Result<(usize, BBox), String> {
    let parts: Vec<&str> = zone.trim().split(':').collect();
    if parts.is_empty() || parts[0].is_empty() {
        return Err("Empty zone specification".to_string());
    }
    let (page_num, tail_parts) = if parts.len() >= 2 && parts[0].eq_ignore_ascii_case("page") {
        let pn = if parts[1].eq_ignore_ascii_case("last") {
            pages.len().max(1)
        } else {
            parts[1]
                .parse::<usize>()
                .map_err(|_| format!("Invalid page in zone '{zone}'"))?
        };
        (pn, &parts[2..])
    } else {
        (1, &parts[..])
    };
    let p = pages
        .iter()
        .find(|ps| ps.page == page_num)
        .ok_or_else(|| format!("Page {page_num} does not exist (document has {} pages)", pages.len()))?;

    let tail = if tail_parts.is_empty() {
        "full".to_string()
    } else {
        tail_parts.join(":")
    };

    let bbox = match tail.to_ascii_lowercase().as_str() {
        "full" => BBox::new(0.0, 0.0, p.width_pt, p.height_pt),
        "top" => BBox::new(0.0, 0.0, p.width_pt, p.height_pt * 0.5),
        "bottom" | "footer" => BBox::new(0.0, p.height_pt * 0.75, p.width_pt, p.height_pt * 0.25),
        "header" => BBox::new(0.0, 0.0, p.width_pt, p.height_pt * 0.15),
        "q1" => BBox::new(0.0, 0.0, p.width_pt * 0.5, p.height_pt * 0.5),
        "q2" => BBox::new(p.width_pt * 0.5, 0.0, p.width_pt * 0.5, p.height_pt * 0.5),
        "q3" => BBox::new(0.0, p.height_pt * 0.5, p.width_pt * 0.5, p.height_pt * 0.5),
        "q4" => BBox::new(p.width_pt * 0.5, p.height_pt * 0.5, p.width_pt * 0.5, p.height_pt * 0.5),
        "gutter" => {
            if p.columns.len() >= 2 {
                let x0 = p.columns[0].x_max_pt - 8.0;
                BBox::new(x0.max(0.0), 0.0, 16.0, p.height_pt)
            } else {
                BBox::new(p.width_pt * 0.48, 0.0, p.width_pt * 0.04, p.height_pt)
            }
        }
        other if other.starts_with("col:") => {
            let sub: Vec<&str> = other.split(':').collect();
            let col_num: usize = sub
                .get(1)
                .ok_or_else(|| format!("Missing column index in zone '{zone}'"))?
                .parse::<usize>()
                .map_err(|_| format!("Invalid column index in zone '{zone}'"))?;
            if col_num == 0 {
                return Err(format!("Column index must be >= 1 in zone '{zone}'"));
            }
            let col_idx = col_num - 1;
            let col = p
                .columns
                .get(col_idx)
                .ok_or_else(|| format!("Column {col_num} not found on page {}", p.page))?;
            let (y0, h) = match sub.get(2).copied() {
                Some("top") => (0.0, p.height_pt * 0.5),
                Some("bottom") => (p.height_pt * 0.75, p.height_pt * 0.25),
                Some(extra) => return Err(format!("Unknown column sub-zone '{extra}' in '{zone}'")),
                None => (0.0, p.height_pt),
            };
            BBox::new(col.x_min_pt, y0, (col.x_max_pt - col.x_min_pt).max(1.0), h)
        }
        _ => return Err(format!("Unknown zone suffix '{tail}' in '{zone}'")),
    };

    Ok((page_num, bbox))
}

pub fn run_probe(doc: &ExtractedDocument, file_label: &str, q: &ProbeQuery) -> Result<ProbeReport, String> {
    if doc.pages.is_empty() {
        return Err("Compiled document has 0 pages".to_string());
    }

    // 1. Measure gap between two text anchors
    if let Some((ref a_pat, ref b_pat)) = q.measure_gap {
        let a_low = a_pat.to_ascii_lowercase();
        let b_low = b_pat.to_ascii_lowercase();
        let a_line = doc
            .lines
            .iter()
            .find(|l| l.text.to_ascii_lowercase().contains(&a_low))
            .ok_or_else(|| format!("Anchor '{a_pat}' not found in rendered text"))?;
        let b_line = doc
            .lines
            .iter()
            .find(|l| l.text.to_ascii_lowercase().contains(&b_low))
            .ok_or_else(|| format!("Anchor '{b_pat}' not found in rendered text"))?;

        let dx = if b_line.bbox.x > a_line.bbox.right() {
            b_line.bbox.x - a_line.bbox.right()
        } else if a_line.bbox.x > b_line.bbox.right() {
            a_line.bbox.x - b_line.bbox.right()
        } else {
            0.0
        };
        let dy = b_line.bbox.y - a_line.bbox.bottom();
        return Ok(ProbeReport {
            file: file_label.to_string(),
            query: format!("measure-gap \"{a_pat}\" -> \"{b_pat}\""),
            page: Some(a_line.page),
            probed_bbox_pt: Some(a_line.bbox.union(b_line.bbox).to_array()),
            coverage_pct: None,
            empty_space_pct: None,
            matches: vec![
                ProbeMatch {
                    kind: "anchor_a".to_string(),
                    page: a_line.page,
                    col: a_line.col,
                    bbox_pt: a_line.bbox.to_array(),
                    src_line: a_line.src_line,
                    text: a_line.text.clone(),
                },
                ProbeMatch {
                    kind: "anchor_b".to_string(),
                    page: b_line.page,
                    col: b_line.col,
                    bbox_pt: b_line.bbox.to_array(),
                    src_line: b_line.src_line,
                    text: b_line.text.clone(),
                },
            ],
            blocks: Vec::new(),
            gap: Some(GapMeasurement {
                from_text: a_line.text.clone(),
                to_text: b_line.text.clone(),
                same_page: a_line.page == b_line.page,
                dx_pt: round2(dx),
                dy_pt: round2(dy),
            }),
            density_grid: None,
        });
    }

    // 2. Locate by Markdown source line (--at-line N)
    if let Some(target_line) = q.at_line {
        let mut effective_line = target_line;
        let mut matched_blocks: Vec<LayoutBlock> = doc
            .blocks
            .iter()
            .filter(|b| b.src_line == Some(target_line))
            .cloned()
            .collect();
        if matched_blocks.is_empty() {
            // Fall back to nearest previous tagged source line
            if let Some((sl, blk)) = doc
                .blocks
                .iter()
                .filter_map(|b| b.src_line.filter(|&sl| sl <= target_line).map(|sl| (sl, b)))
                .max_by_key(|(sl, _)| *sl)
            {
                effective_line = sl;
                matched_blocks.push(blk.clone());
            }
        }
        let matches: Vec<ProbeMatch> = doc
            .lines
            .iter()
            .filter(|l| l.src_line == Some(effective_line))
            .map(|l| ProbeMatch {
                kind: l.src_kind.clone().unwrap_or_else(|| "line".to_string()),
                page: l.page,
                col: l.col,
                bbox_pt: l.bbox.to_array(),
                src_line: l.src_line,
                text: l.text.clone(),
            })
            .collect();
        let page = matched_blocks
            .first()
            .map(|b| b.page)
            .or_else(|| matches.first().map(|m| m.page));
        let bbox = matched_blocks.first().map(|b| b.bbox.to_array());
        return Ok(ProbeReport {
            file: file_label.to_string(),
            query: format!("at-line {target_line}"),
            page,
            probed_bbox_pt: bbox,
            coverage_pct: None,
            empty_space_pct: None,
            matches,
            blocks: matched_blocks,
            gap: None,
            density_grid: None,
        });
    }

    // 3. Determine (page, bbox) from --around, --zone, --region, or --page
    let (target_page, probe_bbox, query_desc) = if let Some(ref anchor) = q.around {
        let low = anchor.to_ascii_lowercase();
        let hit = doc
            .lines
            .iter()
            .find(|l| l.text.to_ascii_lowercase().contains(&low))
            .ok_or_else(|| format!("Text anchor '{anchor}' not found in document"))?;
        let r = q.radius_pt.max(10.0);
        let p = &doc.pages[hit.page - 1];
        let bx = (hit.bbox.x - r).max(0.0);
        let by = (hit.bbox.y - r).max(0.0);
        let br = (hit.bbox.right() + r).min(p.width_pt);
        let bb = (hit.bbox.bottom() + r).min(p.height_pt);
        (
            hit.page,
            BBox::new(bx, by, br - bx, bb - by),
            format!("around \"{anchor}\" (r={r:.1}pt)"),
        )
    } else if let Some(ref zone_spec) = q.zone {
        let (pn, bb) = resolve_zone_bbox(zone_spec, &doc.pages)?;
        (pn, bb, format!("zone {zone_spec}"))
    } else if let Some(ref reg_spec) = q.region {
        let pn = q.page.unwrap_or(1);
        let p = doc
            .pages
            .iter()
            .find(|ps| ps.page == pn)
            .ok_or_else(|| format!("Page {pn} not found"))?;
        let coords: Vec<&str> = reg_spec.split(',').collect();
        if coords.len() != 4 {
            return Err("Region must have 4 comma-separated values: x,y,w,h".to_string());
        }
        let x = parse_dim(coords[0], p.width_pt)?;
        let y = parse_dim(coords[1], p.height_pt)?;
        let w = parse_dim(coords[2], p.width_pt)?;
        let h = parse_dim(coords[3], p.height_pt)?;
        if w <= 0.0 || h <= 0.0 {
            return Err(format!("Region width and height must be positive (got w={w}, h={h})"));
        }
        (pn, BBox::new(x, y, w, h), format!("region {reg_spec}"))
    } else {
        let pn = q.page.unwrap_or(1);
        let p = doc
            .pages
            .iter()
            .find(|ps| ps.page == pn)
            .ok_or_else(|| format!("Page {pn} not found"))?;
        (pn, BBox::new(0.0, 0.0, p.width_pt, p.height_pt), format!("page {pn}"))
    };

    let _page_summary = &doc.pages[target_page - 1];
    let matching_lines: Vec<ProbeMatch> = doc
        .lines
        .iter()
        .filter(|l| l.page == target_page && l.bbox.intersects(probe_bbox))
        .map(|l| ProbeMatch {
            kind: l.src_kind.clone().unwrap_or_else(|| "line".to_string()),
            page: l.page,
            col: l.col,
            bbox_pt: l.bbox.to_array(),
            src_line: l.src_line,
            text: l.text.clone(),
        })
        .collect();

    let matching_blocks: Vec<LayoutBlock> = doc
        .blocks
        .iter()
        .filter(|b| b.page == target_page && b.bbox.intersects(probe_bbox))
        .cloned()
        .collect();

    // Compute ink coverage within probe_bbox
    let probe_area = probe_bbox.area().max(1.0);
    let mut covered_area = 0.0;
    for l in doc
        .lines
        .iter()
        .filter(|l| l.page == target_page)
    {
        if let Some(inter) = l.bbox.intersection(probe_bbox) {
            covered_area += inter.area();
        }
    }
    let coverage_pct = round2(((covered_area / probe_area) * 100.0).clamp(0.0, 100.0));
    let empty_space_pct = round2((100.0 - coverage_pct).max(0.0));

    let density_grid = q.density_grid.map(|(cols, rows)| {
        build_density_grid(
            &doc.raw_runs_by_page[target_page - 1],
            &doc.containers,
            target_page,
            probe_bbox,
            cols.clamp(4, 64),
            rows.clamp(4, 64),
        )
    });

    Ok(ProbeReport {
        file: file_label.to_string(),
        query: query_desc,
        page: Some(target_page),
        probed_bbox_pt: Some(probe_bbox.to_array()),
        coverage_pct: Some(coverage_pct),
        empty_space_pct: Some(empty_space_pct),
        matches: matching_lines,
        blocks: if q.list_blocks || q.around.is_some() || q.zone.is_some() || q.region.is_some() {
            matching_blocks
        } else {
            Vec::new()
        },
        gap: None,
        density_grid,
    })
}

fn build_density_grid(
    runs: &[RawTextRun],
    containers: &[RenderedContainer],
    page_num: usize,
    region: BBox,
    cols: usize,
    rows: usize,
) -> DensityGridOutput {
    let cell_w = region.w.max(1.0) / cols as f64;
    let cell_h = region.h.max(1.0) / rows as f64;
    let cell_area = (cell_w * cell_h).max(1.0);
    let mut matrix = vec![vec![0.0f64; cols]; rows];
    let mut ascii = Vec::with_capacity(rows);

    for (r, mat_row) in matrix.iter_mut().enumerate().take(rows) {
        let mut row_str = String::with_capacity(cols);
        for (c, mat_cell) in mat_row.iter_mut().enumerate().take(cols) {
            let cell = BBox::new(
                region.x + c as f64 * cell_w,
                region.y + r as f64 * cell_h,
                cell_w,
                cell_h,
            );
            let mut text_area = 0.0;
            for run in runs {
                if let Some(inter) = run.bbox.intersection(cell) {
                    text_area += inter.area();
                }
            }
            let has_container = containers
                .iter()
                .any(|ct| ct.page == page_num && ct.is_filled_rect && ct.bbox.intersects(cell));

            let ratio = (text_area / cell_area).clamp(0.0, 1.0);
            *mat_cell = round2(ratio);
            let ch = if ratio >= 0.45 {
                '█'
            } else if ratio >= 0.20 {
                '▓'
            } else if ratio > 0.02 {
                '▒'
            } else if has_container {
                '░'
            } else {
                '.'
            };
            row_str.push(ch);
        }
        ascii.push(row_str);
    }

    DensityGridOutput {
        page: page_num,
        cols,
        rows,
        ascii,
        matrix,
    }
}

pub fn format_probe_human(report: &ProbeReport) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "Botox Spatial Probe: {} [{}]", report.file, report.query);
    if let (Some(p), Some(bb)) = (report.page, report.probed_bbox_pt) {
        let _ = writeln!(
            out,
            "Page {} | Region [{:.1}, {:.1}, {:.1}x{:.1}pt]",
            p, bb[0], bb[1], bb[2], bb[3]
        );
    }
    if let (Some(cov), Some(emp)) = (report.coverage_pct, report.empty_space_pct) {
        let _ = writeln!(out, "Text ink coverage: {:.1}% | Empty space: {:.1}%", cov, emp);
    }
    if let Some(ref g) = report.gap {
        let _ = writeln!(
            out,
            "Gap: dx={:.1}pt, dy={:.1}pt (same_page: {})\n  From: \"{}\"\n  To:   \"{}\"",
            g.dx_pt, g.dy_pt, g.same_page, g.from_text, g.to_text
        );
    }
    if let Some(ref grid) = report.density_grid {
        let _ = writeln!(out, "\nDensity Grid ({}x{}, page {}):", grid.cols, grid.rows, grid.page);
        for row in &grid.ascii {
            let _ = writeln!(out, "  |{row}|");
        }
    }
    if !report.blocks.is_empty() {
        let _ = writeln!(out, "\nBlocks ({}):", report.blocks.len());
        for b in &report.blocks {
            let src = b.src_line.map(|l| format!(" md:{l}")).unwrap_or_default();
            let _ = writeln!(
                out,
                "  #{} [{} col{}{}] lines={} last_fill={:.0}% y={:.1}..{:.1}pt | {}",
                b.id,
                b.kind,
                b.col,
                src,
                b.line_count,
                b.last_line_fill_pct,
                b.bbox.y,
                b.bbox.bottom(),
                b.text_preview
            );
        }
    } else if !report.matches.is_empty() {
        let _ = writeln!(out, "\nMatched Lines ({}):", report.matches.len());
        for m in report.matches.iter().take(30) {
            let src = m.src_line.map(|l| format!(" md:{l}")).unwrap_or_default();
            let _ = writeln!(
                out,
                "  [p{} col{}{}] ({:.1},{:.1}) \"{}\"",
                m.page, m.col, src, m.bbox_pt[0], m.bbox_pt[1], m.text
            );
        }
        if report.matches.len() > 30 {
            let _ = writeln!(out, "  ... ({} more lines)", report.matches.len() - 30);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::BotoxConfig;
    use std::path::Path;

    #[test]
    fn test_metadata_source_tags_do_not_alter_layout_geometry() {
        let md = r#"---
theme: compact
columns: 2
margin: "0.35cm"
fontsize: 8.1pt
---

## Section One

- **Branch:** Isolated parallel line of code; work safely without touching `main`, then merge back.
- **Three areas:** Working Directory $\longrightarrow$ Staging (Index) $\longrightarrow$ Local Repository $\longrightarrow$ Remote.
"#;
        let cfg = BotoxConfig::defaults();
        let (plain_markup, _) = crate::engine::transpile_markdown_to_typst_markup(
            md, None, &cfg, false, false, None, None, None, None, None, &[], &[], Some(Path::new("t.md")),
        ).unwrap();
        let plain_doc = crate::compiler::compile_typst_to_document(&plain_markup, None, Some(md), Some("t.md")).unwrap();
        let (tagged_doc, _, cols) = crate::engine::compile_for_inspection(
            md, None, &cfg, false, false, None, None, None, None, None, &[], &[], Some(Path::new("t.md")), "t.md",
        ).unwrap();

        assert_eq!(plain_doc.pages().len(), tagged_doc.pages().len());
        let plain_ext = extract_document_geometry(&plain_doc, cols);
        let tagged_ext = extract_document_geometry(&tagged_doc, cols);
        assert_eq!(plain_ext.lines.len(), tagged_ext.lines.len());
        assert!(tagged_ext.lines.iter().any(|l| l.src_line.is_some()), "Expected source lines attached");
        for (a, b) in plain_ext.lines.iter().zip(tagged_ext.lines.iter()) {
            assert!((a.y_baseline - b.y_baseline).abs() < 0.01, "Metadata changed baseline y!");
        }
    }

    #[test]
    fn test_inspect_detects_short_last_line_and_reclaim_plan() {
        let md = r#"---
theme: compact
columns: 2
margin: "0.35cm"
fontsize: 8.1pt
---

## Git Reference

- **Branch:** Isolated parallel line of code; work safely without touching `main`, then merge back.
- **Three areas:** Working Directory then Staging Index then Local Repository then Remote.
"#;
        let cfg = BotoxConfig::defaults();
        let (doc, _, cols) = crate::engine::compile_for_inspection(
            md, None, &cfg, false, false, None, None, None, None, None, &[], &[], Some(Path::new("cheat.md")), "cheat.md",
        ).unwrap();
        let extracted = extract_document_geometry(&doc, cols);
        let report = analyze_layout(&extracted, "cheat.md", Some(1));
        assert!(
            report.issues.iter().any(|i| i.code == "SHORT_LAST_LINE"),
            "Expected SHORT_LAST_LINE issue, got: {:?}",
            report.issues
        );
        assert!(report.reclaim_plan.is_some());
    }

    #[test]
    fn test_probe_zone_around_at_line_and_density_grid() {
        let md = r#"---
theme: compact
columns: 2
margin: "0.5cm"
---

# Git Basics

- **Branch:** Work safely without touching `main`, then merge back.
- **Commit:** Record snapshot in history.
"#;
        let cfg = BotoxConfig::defaults();
        let (doc, _, cols) = crate::engine::compile_for_inspection(
            md, None, &cfg, false, false, None, None, None, None, None, &[], &[], Some(Path::new("p.md")), "p.md",
        ).unwrap();
        let extracted = extract_document_geometry(&doc, cols);

        let z = run_probe(&extracted, "p.md", &ProbeQuery {
            zone: Some("page:1:col:1".to_string()),
            density_grid: Some((12, 12)),
            ..Default::default()
        }).unwrap();
        assert_eq!(z.page, Some(1));
        assert!(z.density_grid.is_some());
        assert!(!z.blocks.is_empty());

        let around = run_probe(&extracted, "p.md", &ProbeQuery {
            around: Some("Branch".to_string()),
            radius_pt: 40.0,
            ..Default::default()
        }).unwrap();
        assert!(!around.matches.is_empty());

        let at_line = run_probe(&extracted, "p.md", &ProbeQuery {
            at_line: Some(9),
            ..Default::default()
        }).unwrap();
        assert!(!at_line.matches.is_empty());
    }

    #[test]
    fn test_narrow_two_column_gutter_does_not_merge_columns() {
        let mut md = String::from(
            "---\nauthor: \"\"\ntheme: compact\ncolumns: 2\ncolumn-gutter: 8pt\ngeometry: \"margin=0.35cm\"\nfontsize: 8.1pt\nnumber-sections: false\ntable-of-contents: false\nbibliography: false\n---\n\n## Cheatsheet\n\n",
        );
        for idx in 1..=70 {
            md.push_str(&format!(
                "- **Item {idx}:** Detailed compact cheatsheet bullet filling almost the full column width cleanly.\n"
            ));
        }
        let cfg = BotoxConfig::defaults();
        let (doc, _, cols) = crate::engine::compile_for_inspection(
            &md, None, &cfg, false, false, None, None, None, None, None, &[], &[], Some(Path::new("sheet.md")), "sheet.md",
        ).unwrap();
        let extracted = extract_document_geometry(&doc, cols);
        let col1_count = extracted.lines.iter().filter(|l| l.page == 1 && l.col == 1).count();
        let col2_count = extracted.lines.iter().filter(|l| l.page == 1 && l.col == 2).count();
        assert!(col1_count >= 20 && col2_count >= 10, "Expected both columns populated, got col1={col1_count} col2={col2_count}");

        let report = analyze_layout(&extracted, "sheet.md", Some(1));
        assert!(
            !report.issues.iter().any(|i| i.code == "GUTTER_INTRUSION"),
            "Unexpected GUTTER_INTRUSION on normal 8pt-gutter cheatsheet: {:?}",
            report.issues.iter().filter(|i| i.code == "GUTTER_INTRUSION").collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_math_stress_document_has_zero_false_text_overlap() {
        let md = std::fs::read_to_string("tests/math_stress_test.md").unwrap();
        let cfg = BotoxConfig::defaults();
        let (doc, _, cols) = crate::engine::compile_for_inspection(
            &md, None, &cfg, false, false, None, None, None, None, None, &[], &[], Some(Path::new("tests/math_stress_test.md")), "math_stress_test.md",
        ).unwrap();
        let extracted = extract_document_geometry(&doc, cols);
        let report = analyze_layout(&extracted, "math_stress_test.md", None);
        let overlaps: Vec<_> = report.issues.iter().filter(|i| i.code == "TEXT_OVERLAP").collect();
        assert!(
            overlaps.is_empty(),
            "Expected 0 false TEXT_OVERLAP errors on math_stress_test.md, got {} (first: {:?})",
            overlaps.len(),
            overlaps.first()
        );
    }

    #[test]
    fn test_resolve_zone_bbox_bare_and_errors() {
        let pages = vec![PageSummary {
            page: 1,
            width_pt: 595.0,
            height_pt: 842.0,
            content_bbox_pt: [20.0, 20.0, 555.0, 800.0],
            fill_pct: 80.0,
            bottom_dead_space_pt: 120.0,
            columns: vec![
                ColumnStats { col: 1, x_min_pt: 20.0, x_max_pt: 290.0, content_bottom_y: 700.0, fill_pct: 80.0, bottom_dead_space_pt: 120.0, block_count: 5 },
                ColumnStats { col: 2, x_min_pt: 305.0, x_max_pt: 575.0, content_bottom_y: 500.0, fill_pct: 60.0, bottom_dead_space_pt: 320.0, block_count: 3 },
            ],
            block_count: 8,
            line_count: 40,
        }];
        let (p, bb) = resolve_zone_bbox("col:2:bottom", &pages).unwrap();
        assert_eq!(p, 1);
        assert!((bb.x - 305.0).abs() < 0.1);
        assert!((bb.y - 842.0 * 0.75).abs() < 0.5);

        let (p2, q4) = resolve_zone_bbox("q4", &pages).unwrap();
        assert_eq!(p2, 1);
        assert!((q4.x - 297.5).abs() < 0.1);

        assert!(resolve_zone_bbox("col:0", &pages).is_err());
        assert!(resolve_zone_bbox("col:abc", &pages).is_err());
        assert!(resolve_zone_bbox("page:99:bottom", &pages).is_err());
    }

    #[test]
    fn test_probe_measure_gap_region_and_compare_reports() {
        let md = "---\ntheme: compact\nmargin: \"1cm\"\n---\n\n## Alpha\n\nFirst paragraph text.\n\n## Beta\n\nSecond paragraph text.\n";
        let cfg = BotoxConfig::defaults();
        let (doc, _, cols) = crate::engine::compile_for_inspection(
            md, None, &cfg, false, false, None, None, None, None, None, &[], &[], Some(Path::new("g.md")), "g.md",
        ).unwrap();
        let extracted = extract_document_geometry(&doc, cols);

        let gap_rep = run_probe(&extracted, "g.md", &ProbeQuery {
            measure_gap: Some(("Alpha".to_string(), "Beta".to_string())),
            ..Default::default()
        }).unwrap();
        let g = gap_rep.gap.expect("Expected gap measurement");
        assert!(g.same_page && g.dy_pt > 0.0);

        let reg_rep = run_probe(&extracted, "g.md", &ProbeQuery {
            page: Some(1),
            region: Some("0%,0%,100%,50%".to_string()),
            density_grid: Some((8, 8)),
            ..Default::default()
        }).unwrap();
        assert!(reg_rep.coverage_pct.is_some());
        assert_eq!(reg_rep.density_grid.unwrap().cols, 8);

        assert!(run_probe(&extracted, "g.md", &ProbeQuery {
            region: Some("0,0,0,10".to_string()),
            ..Default::default()
        }).is_err());

        let r1 = analyze_layout(&extracted, "g.md", None);
        let delta = compare_reports(&r1, &r1);
        assert_eq!(delta.pages_delta, 0);
        assert_eq!(delta.issues_delta, 0);
    }

    #[test]
    fn test_page_spillover_and_real_reclaim_plan() {
        // Generate a 2-column cheatsheet that spills just a few lines onto page 2
        let mut md = String::from(
            "---\nauthor: \"\"\ntheme: compact\ncolumns: 2\ncolumn-gutter: 8pt\ngeometry: \"margin=0.35cm\"\nfontsize: 8.1pt\nlinestretch: 1.01\nnumber-sections: false\ntable-of-contents: false\nbibliography: false\n---\n\n## Dense Sheet\n\n",
        );
        for idx in 1..=72 {
            md.push_str(&format!(
                "- **Rule {idx}:** Isolated parallel line of code; work safely without touching `main`, then merge back.\n"
            ));
        }
        let cfg = BotoxConfig::defaults();
        let (doc, _, cols) = crate::engine::compile_for_inspection(
            &md, None, &cfg, false, false, None, None, None, None, None, &[], &[], Some(Path::new("spill.md")), "spill.md",
        ).unwrap();
        let extracted = extract_document_geometry(&doc, cols);
        assert!(extracted.pages.len() >= 2, "Expected document to spill onto page 2, got {} pages", extracted.pages.len());

        let report = analyze_layout(&extracted, "spill.md", Some(1));
        assert!(
            report.issues.iter().any(|i| i.code == "PAGE_SPILLOVER"),
            "Expected PAGE_SPILLOVER on lightly filled page 2 (last fill: {}%)",
            report.last_page_fill_pct
        );
        let plan = report.reclaim_plan.as_ref().expect("Expected reclaim plan");
        assert_eq!(plan.target_pages, 1);
        assert!(plan.overflow_pt > 0.0);
        assert!(!plan.steps.is_empty(), "Expected actionable reclaim steps");
        assert!(plan.total_reclaimable_pt > 0.0);

        let human = format_inspect_human(&report);
        assert!(human.contains("Reclaim Plan"));
        assert!(human.contains("PAGE_SPILLOVER"));
    }

    #[test]
    fn test_compare_reports_detects_real_widow_reduction() {
        let md_before = "---\nauthor: \"\"\ntheme: compact\ncolumns: 2\ngeometry: \"margin=0.35cm\"\nfontsize: 8.1pt\n---\n\n- **Branch:** Isolated parallel line of code; work safely without touching `main`, then merge back.\n- **Merge conflict:** Concurrent edits to the same lines, or an edit versus delete on the same file.\n";
        let md_after = "---\nauthor: \"\"\ntheme: compact\ncolumns: 2\ngeometry: \"margin=0.35cm\"\nfontsize: 8.1pt\n---\n\n- **Branch:** Isolated parallel line; work safely off `main`.\n- **Merge conflict:** Concurrent edits or edit vs delete.\n";

        let cfg = BotoxConfig::defaults();
        let (d1, _, c1) = crate::engine::compile_for_inspection(
            md_before, None, &cfg, false, false, None, None, None, None, None, &[], &[], Some(Path::new("a.md")), "a.md",
        ).unwrap();
        let (d2, _, c2) = crate::engine::compile_for_inspection(
            md_after, None, &cfg, false, false, None, None, None, None, None, &[], &[], Some(Path::new("b.md")), "b.md",
        ).unwrap();

        let r1 = analyze_layout(&extract_document_geometry(&d1, c1), "a.md", None);
        let r2 = analyze_layout(&extract_document_geometry(&d2, c2), "b.md", None);
        let delta = compare_reports(&r1, &r2);
        assert!(
            delta.resolved_codes.get("SHORT_LAST_LINE").copied().unwrap_or(0) >= 1,
            "Expected SHORT_LAST_LINE issues resolved, got delta: {:?}",
            delta
        );
        assert!(delta.issues_delta < 0);
    }

    #[test]
    fn test_gutter_intrusion_and_content_overflow_detection() {
        let md = "---\nauthor: \"\"\ntheme: compact\ncolumns: 2\ncolumn-gutter: 8pt\ngeometry: \"margin=0.35cm\"\nfontsize: 9pt\n---\n\n- `supercalifragilisticexpialidocious_unbreakable_identifier_that_overflows_column_gutter_easily_1234567890`\n";
        let cfg = BotoxConfig::defaults();
        let (doc, _, cols) = crate::engine::compile_for_inspection(
            md, None, &cfg, false, false, None, None, None, None, None, &[], &[], Some(Path::new("ov.md")), "ov.md",
        ).unwrap();
        let report = analyze_layout(&extract_document_geometry(&doc, cols), "ov.md", None);
        assert!(
            report.issues.iter().any(|i| i.code == "GUTTER_INTRUSION" || i.code == "CONTENT_OVERFLOW"),
            "Expected GUTTER_INTRUSION or CONTENT_OVERFLOW for unbreakable token, got: {:?}",
            report.issues
        );
    }

    #[test]
    fn test_orphan_heading_and_column_imbalance_detection() {
        let md = "---\nauthor: \"\"\ntheme: compact\ncolumns: 2\ngeometry: \"margin=1cm\"\n---\n\n## Section 1\n\nShort paragraph.\n\n## Stranded Heading\n\n\\columnbreak\n\nBody moved to column 2.\n\n\\newpage\n\nPage 2 content.\n";
        let cfg = BotoxConfig::defaults();
        let (doc, _, cols) = crate::engine::compile_for_inspection(
            md, None, &cfg, false, false, None, None, None, None, None, &[], &[], Some(Path::new("orph.md")), "orph.md",
        ).unwrap();
        let report = analyze_layout(&extract_document_geometry(&doc, cols), "orph.md", None);
        assert!(
            report.issues.iter().any(|i| i.code == "ORPHAN_HEADING"),
            "Expected ORPHAN_HEADING for heading before \\columnbreak, got: {:?}",
            report.issues
        );
    }

    #[test]
    fn test_text_overlap_positive_detection_on_synthetic_collision() {
        let extracted = ExtractedDocument {
            pages: vec![PageSummary {
                page: 1,
                width_pt: 595.0,
                height_pt: 842.0,
                content_bbox_pt: [20.0, 20.0, 555.0, 200.0],
                fill_pct: 25.0,
                bottom_dead_space_pt: 600.0,
                columns: vec![ColumnStats {
                    col: 1,
                    x_min_pt: 20.0,
                    x_max_pt: 575.0,
                    content_bottom_y: 220.0,
                    fill_pct: 25.0,
                    bottom_dead_space_pt: 600.0,
                    block_count: 1,
                }],
                block_count: 1,
                line_count: 2,
            }],
            lines: vec![
                TextLine {
                    page: 1,
                    col: 1,
                    y_baseline: 100.0,
                    bbox: BBox::new(50.0, 92.0, 120.0, 10.0),
                    text: "Colliding line A".to_string(),
                    word_count: 3,
                    font_size_pt: 10.0,
                    is_math: false,
                    in_table: false,
                    src_line: Some(5),
                    src_kind: Some("para".to_string()),
                },
                TextLine {
                    page: 1,
                    col: 1,
                    y_baseline: 101.0,
                    bbox: BBox::new(70.0, 93.0, 120.0, 10.0),
                    text: "Colliding line B".to_string(),
                    word_count: 3,
                    font_size_pt: 10.0,
                    is_math: false,
                    in_table: false,
                    src_line: Some(6),
                    src_kind: Some("para".to_string()),
                },
            ],
            blocks: Vec::new(),
            containers: Vec::new(),
            fonts_used: Vec::new(),
            raw_runs_by_page: vec![Vec::new()],
        };
        let report = analyze_layout(&extracted, "collide.md", None);
        let overlap = report
            .issues
            .iter()
            .find(|i| i.code == "TEXT_OVERLAP")
            .expect("Expected TEXT_OVERLAP on colliding lines");
        assert!(
            overlap.suggestion.contains("Colliding line A") && overlap.suggestion.contains("Colliding line B"),
            "TEXT_OVERLAP suggestion should include both colliding strings: {}",
            overlap.suggestion
        );
        assert!(
            overlap.suggestion.contains("70.0, 93.0"),
            "TEXT_OVERLAP suggestion should include overlap bbox coordinates: {}",
            overlap.suggestion
        );
    }

    #[test]
    fn test_probe_at_line_exact_and_fallback_and_units() {
        let md = "---\ntheme: modern\n---\n\n# Title Heading\n\nParagraph on line 7.\n\n- Bullet on line 9\n";
        let cfg = BotoxConfig::defaults();
        let (doc, _, cols) = crate::engine::compile_for_inspection(
            md, None, &cfg, false, false, None, None, None, None, None, &[], &[], Some(Path::new("lines.md")), "lines.md",
        ).unwrap();
        let extracted = extract_document_geometry(&doc, cols);

        // Exact line 7 (paragraph)
        let p7 = run_probe(&extracted, "lines.md", &ProbeQuery {
            at_line: Some(7),
            ..Default::default()
        }).unwrap();
        assert!(p7.matches.iter().any(|m| m.text.contains("Paragraph on line 7")));

        // Blank line 8 should fall back to line 7 and still populate matches
        let p8 = run_probe(&extracted, "lines.md", &ProbeQuery {
            at_line: Some(8),
            ..Default::default()
        }).unwrap();
        assert!(!p8.blocks.is_empty());
        assert!(!p8.matches.is_empty());

        // Exact line 9 (bullet)
        let p9 = run_probe(&extracted, "lines.md", &ProbeQuery {
            at_line: Some(9),
            ..Default::default()
        }).unwrap();
        assert!(p9.matches.iter().any(|m| m.text.contains("Bullet on line 9")));

        // Region with mm / cm / pt units
        let r_mm = run_probe(&extracted, "lines.md", &ProbeQuery {
            page: Some(1),
            region: Some("10mm,1cm,180mm,100pt".to_string()),
            ..Default::default()
        }).unwrap();
        assert!(r_mm.probed_bbox_pt.is_some());
        let human = format_probe_human(&r_mm);
        assert!(human.contains("Botox Spatial Probe"));
    }

    #[test]
    fn test_math_equations_not_flagged_as_short_last_line() {
        let md = "---\ntheme: academic\n---\n\n$$\n\\frac{\\partial f}{\\partial x} + \\frac{\\partial g}{\\partial y} = 0\n$$\n";
        let cfg = BotoxConfig::defaults();
        let (doc, _, cols) = crate::engine::compile_for_inspection(
            md, None, &cfg, false, false, None, None, None, None, None, &[], &[], Some(Path::new("m.md")), "m.md",
        ).unwrap();
        let report = analyze_layout(&extract_document_geometry(&doc, cols), "m.md", None);
        assert!(
            !report.issues.iter().any(|i| i.code == "SHORT_LAST_LINE"),
            "Math denominators should not trigger SHORT_LAST_LINE: {:?}",
            report.issues
        );
    }

    #[test]
    fn test_table_rows_do_not_trigger_inflated_leading() {
        let md = r#"---
author: ""
theme: compact
columns: 2
geometry: "margin=0.35cm"
fontsize: 8.25pt
---

## Commands

Regular body paragraph with multiple lines of text to establish a clean median body leading across the column before the table appears.

| Command | Purpose |
| :--- | :--- |
| `git status` | Working tree state |
| `git diff` | Unstaged changes |
| `git log` | Commit history |
| `git rebase` | Replay commits |
| `git bisect` | Binary search bugs |
"#;
        let cfg = BotoxConfig::defaults();
        let (doc, _, cols) = crate::engine::compile_for_inspection(
            md, None, &cfg, false, false, None, None, None, None, None, &[], &[], Some(Path::new("tbl.md")), "tbl.md",
        ).unwrap();
        let extracted = extract_document_geometry(&doc, cols);
        assert!(
            extracted.lines.iter().any(|l| l.in_table),
            "Expected table lines to be flagged with in_table=true"
        );
        let report = analyze_layout(&extracted, "tbl.md", None);
        let inflated: Vec<_> = report.issues.iter().filter(|i| i.code == "INFLATED_LEADING").collect();
        assert!(
            inflated.is_empty(),
            "Table rows should not trigger INFLATED_LEADING, got: {:?}",
            inflated
        );
    }

    #[test]
    fn test_split_code_block_across_columns_detected() {
        // Fill ~85% of Column 1, then place a 12-line fenced code block so it splits across Col 1 -> Col 2
        let mut md = String::from(
            "---\nauthor: \"\"\ntheme: compact\ncolumns: 2\ncolumn-gutter: 8pt\ngeometry: \"margin=0.35cm\"\nfontsize: 8.25pt\ncode-fontsize: 7.3pt\nnumber-sections: false\ntable-of-contents: false\nbibliography: false\n---\n\n## TCP Client\n\n",
        );
        for idx in 1..=58 {
            md.push_str(&format!("- **Bullet {idx}:** Compact summary line.\n"));
        }
        md.push_str("\n```python\n");
        for idx in 1..=20 {
            md.push_str(&format!("line_{idx} = send_packet({idx})\n"));
        }
        md.push_str("```\n");

        let cfg = BotoxConfig::defaults();
        let (doc, _, cols) = crate::engine::compile_for_inspection(
            &md, None, &cfg, false, false, None, None, None, None, None, &[], &[], Some(Path::new("split.md")), "split.md",
        ).unwrap();
        let extracted = extract_document_geometry(&doc, cols);
        let report = analyze_layout(&extracted, "split.md", None);
        let split_issue = report
            .issues
            .iter()
            .find(|i| i.code == "SPLIT_CODE_BLOCK")
            .expect("Expected SPLIT_CODE_BLOCK warning when fenced code splits across columns");
        assert_eq!(split_issue.severity, "warn");
        assert!(split_issue.suggestion.contains("col1") && split_issue.suggestion.contains("col2"));
    }

    #[test]
    fn test_underfull_column_detects_explicit_newpage_and_averages_page_fill() {
        let mut md = String::from("---\ntitle: Underfull Col Test\ntheme: compact\ncolumns: 2\nfontsize: 9pt\n---\n\n");
        for i in 1..=26 {
            md.push_str(&format!("Paragraph {i} with enough text to fill column one completely to the bottom margin of page one.\n\n"));
        }
        md.push_str("## Col 2 Start\n\nShort text in col 2.\n\n\\newpage\n\n## Page 2 Start\n\nContent on page 2.\n");

        let cfg = BotoxConfig::defaults();
        let (doc, _, cols) = crate::engine::compile_for_inspection(
            &md, None, &cfg,
            false, false, None, None, None, None, None, &[], &[], None, "uf.md",
        ).expect("compile ok");
        let extracted = extract_document_geometry(&doc, cols);
        let report = analyze_layout_with_source(&extracted, "uf.md", None, Some(&md));

        assert!(report.page_count >= 2, "expected >= 2 pages");
        let p1 = &report.pages[0];
        let avg: f64 = p1.columns.iter().map(|c| c.fill_pct).sum::<f64>() / p1.columns.len() as f64;
        assert!((p1.fill_pct - avg).abs() < 0.2, "page fill {} should equal avg {}", p1.fill_pct, avg);

        let uf: Vec<_> = report.issues.iter().filter(|i| i.code == "UNDERFULL_COLUMN").collect();
        assert!(!uf.is_empty(), "expected UNDERFULL_COLUMN on underfull non-final column");
        assert!(
            uf.iter().any(|i| i.suggestion.contains(r"\newpage")),
            "expected explicit \\newpage cause in suggestion, got: {:?}",
            uf.iter().map(|i| &i.suggestion).collect::<Vec<_>>()
        );

        let text = format_inspect_human(&report);
        assert!(text.contains("(min "), "expected '(min ...%)' in multi-col text output:\n{text}");
    }

    #[test]
    fn test_sticky_headings_emitted_in_document_preamble() {
        let md = "---\ntitle: Sticky Test\ntheme: compact\n---\n\n# Heading\n\nBody.\n";
        let cfg = BotoxConfig::defaults();
        let (typst, _) = crate::engine::transpile_markdown_to_typst_markup(
            md, None, &cfg,
            false, false, None, None, None, None, None, &[], &[], None,
        ).expect("transpile ok");
        assert!(typst.contains("#show heading: set block(sticky: true)"));
        assert!(typst.contains("sticky: true"));
    }
}
