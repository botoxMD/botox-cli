//! Character-Level Two-Way Source Sync Module
//!
//! Maps byte offsets between Markdown source and generated Typst markup, then
//! indexes every rendered glyph in a compiled Typst `PagedDocument` so that:
//! - Preview clicks `(page, x_pt, y_pt)` resolve to exact 1-based `(line, col)` in Markdown.
//! - Editor cursors `(line, col)` resolve to exact `(page, x_pt, y_pt, w_pt, h_pt, y_ratio)` in the PDF.

use serde::Serialize;
use typst::layout::{Frame, FrameItem};
use typst::syntax::Source;
use typst_layout::PagedDocument;

/// A contiguous byte mapping from generated Typst markup back to the original Markdown document.
#[derive(Debug, Clone, Copy)]
pub struct SourceSegment {
    pub typst_start: usize,
    pub typst_end: usize,
    pub md_start: usize,
    pub md_end: usize,
}

/// Maps preprocessed Markdown byte offsets back to raw Markdown body byte offsets.
pub struct PreprocessOffsetMap {
    /// For each line in `preprocessed`: `(prep_line_start, raw_line_start, raw_line_len)`
    lines: Vec<(usize, usize, usize)>,
}

impl PreprocessOffsetMap {
    pub fn build(raw_body: &str, preprocessed: &str) -> Self {
        let raw_lines = split_lines_with_offsets(raw_body);
        let prep_lines = split_lines_with_offsets(preprocessed);

        let mut mapped = Vec::with_capacity(prep_lines.len());
        let mut raw_cursor: usize = 0;
        let mut last_raw_start: usize = 0;
        let mut last_raw_len: usize = 0;

        for (prep_start, prep_text) in prep_lines {
            let prep_trimmed = prep_text.trim();

            // Never advance `raw_cursor` on blank lines or synthetic preprocessor markers;
            // otherwise extra blank lines inserted around callouts/tables eat real lines ahead!
            if prep_trimmed.is_empty()
                || (prep_trimmed.starts_with("<!--botox:") && prep_trimmed.ends_with("-->"))
            {
                mapped.push((prep_start, last_raw_start, last_raw_len));
                continue;
            }

            let mut found = None;
            let search_end = (raw_cursor + 64).min(raw_lines.len());

            // 1. Exact line match
            for (idx, &(r_start, r_text)) in raw_lines.iter().enumerate().take(search_end).skip(raw_cursor) {
                if r_text == prep_text {
                    found = Some((idx, r_start, r_text.len()));
                    break;
                }
            }
            // 2. Trimmed or substring match (handles `> ` stripped from GitHub callouts!)
            if found.is_none() {
                for (idx, &(r_start, r_text)) in raw_lines.iter().enumerate().take(search_end).skip(raw_cursor) {
                    if let Some(sub_pos) = r_text.find(prep_trimmed) {
                        let prep_lead = prep_text.len() - prep_text.trim_start().len();
                        let adj_start = r_start + sub_pos.saturating_sub(prep_lead);
                        let adj_len = r_text.len().saturating_sub(sub_pos.saturating_sub(prep_lead));
                        found = Some((idx, adj_start, adj_len));
                        break;
                    }
                }
            }

            if let Some((idx, r_start, r_len)) = found {
                raw_cursor = idx + 1;
                last_raw_start = r_start;
                last_raw_len = r_len;
                mapped.push((prep_start, r_start, r_len));
            } else {
                // Synthetic line inserted by preprocessor: anchor to last matched raw line
                mapped.push((prep_start, last_raw_start, last_raw_len));
            }
        }

        Self { lines: mapped }
    }

    /// Map a byte offset in `preprocessed` to a byte offset in `raw_body`.
    pub fn map_offset(&self, prep_byte: usize) -> usize {
        if self.lines.is_empty() {
            return prep_byte;
        }
        let idx = self
            .lines
            .partition_point(|&(prep_start, _, _)| prep_start <= prep_byte)
            .saturating_sub(1);
        let (prep_start, raw_start, raw_len) = self.lines[idx];
        let col_in_line = prep_byte.saturating_sub(prep_start);
        raw_start + col_in_line.min(raw_len)
    }

    pub fn map_range(&self, prep_start: usize, prep_end: usize, base_offset: usize) -> (usize, usize) {
        let s = base_offset + self.map_offset(prep_start);
        let e = base_offset + self.map_offset(prep_end);
        if e >= s { (s, e) } else { (s, s) }
    }
}

fn split_lines_with_offsets(s: &str) -> Vec<(usize, &str)> {
    let mut out = Vec::new();
    let mut start = 0;
    for part in s.split('\n') {
        let clean = part.strip_suffix('\r').unwrap_or(part);
        out.push((start, clean));
        start += part.len() + 1;
    }
    out
}

/// Complete Markdown <-> Typst byte source map for a single compiled document.
#[derive(Debug, Clone)]
pub struct SourceMap {
    pub segments: Vec<SourceSegment>,
    md_line_starts: Vec<usize>,
    md_text: String,
}

impl SourceMap {
    pub fn new(raw_markdown: &str, mut segments: Vec<SourceSegment>) -> Self {
        segments.sort_by_key(|s| (s.typst_start, s.typst_end));
        let mut md_line_starts = vec![0usize];
        for (i, b) in raw_markdown.bytes().enumerate() {
            if b == b'\n' {
                md_line_starts.push(i + 1);
            }
        }
        Self {
            segments,
            md_line_starts,
            md_text: raw_markdown.to_string(),
        }
    }

    /// Shift all Typst byte offsets by `delta` (e.g., preamble length before body).
    pub fn shift_typst_offsets(&mut self, delta: usize) {
        if delta == 0 {
            return;
        }
        for seg in &mut self.segments {
            seg.typst_start += delta;
            seg.typst_end += delta;
        }
    }

    /// Map a byte offset in the generated Typst source to an exact byte offset in raw Markdown.
    pub fn typst_byte_to_md_byte(&self, typst_byte: usize) -> Option<usize> {
        if self.segments.is_empty() {
            return None;
        }
        let idx = self
            .segments
            .partition_point(|s| s.typst_start <= typst_byte)
            .saturating_sub(1);
        let seg = self.segments.get(idx)?;
        if typst_byte < seg.typst_start || typst_byte > seg.typst_end {
            return None;
        }
        let typst_len = seg.typst_end.saturating_sub(seg.typst_start);
        let md_len = seg.md_end.saturating_sub(seg.md_start);
        if typst_len == 0 || md_len == 0 {
            return Some(seg.md_start);
        }
        let rel = typst_byte - seg.typst_start;
        let mapped = seg.md_start + (rel.saturating_mul(md_len) / typst_len).min(md_len);
        Some(mapped.min(self.md_text.len()))
    }

    /// Map a byte offset in generated Typst to the starting Markdown byte of its owning segment
    /// (ideal for atomic non-text elements like images, diagrams, rules, and callout boxes).
    pub fn typst_byte_to_segment_md_start(&self, typst_byte: usize) -> Option<usize> {
        if self.segments.is_empty() {
            return None;
        }
        let idx = self
            .segments
            .partition_point(|s| s.typst_start <= typst_byte)
            .saturating_sub(1);
        let seg = self.segments.get(idx)?;
        if typst_byte < seg.typst_start || typst_byte > seg.typst_end {
            return None;
        }
        Some(seg.md_start.min(self.md_text.len()))
    }

    /// Convert a Markdown byte offset to 1-based `(line, utf16_col)` for VS Code.
    pub fn md_byte_to_line_col(&self, md_byte: usize) -> (u32, u32) {
        let clamped = md_byte.min(self.md_text.len());
        let line_idx = self
            .md_line_starts
            .partition_point(|&s| s <= clamped)
            .saturating_sub(1);
        let line_start = self.md_line_starts[line_idx];
        // Snap `clamped` to a valid UTF-8 char boundary
        let mut safe_end = clamped;
        while safe_end > line_start && !self.md_text.is_char_boundary(safe_end) {
            safe_end -= 1;
        }
        let slice = &self.md_text[line_start..safe_end];
        let utf16_col = slice.encode_utf16().count();
        ((line_idx + 1) as u32, (utf16_col + 1) as u32)
    }

    /// Convert 1-based `(line, utf16_col)` from VS Code to a Markdown byte offset.
    pub fn line_col_to_md_byte(&self, line: usize, col: usize) -> usize {
        if self.md_line_starts.is_empty() {
            return 0;
        }
        let line_idx = line.saturating_sub(1).min(self.md_line_starts.len() - 1);
        let line_start = self.md_line_starts[line_idx];
        let line_end = self
            .md_line_starts
            .get(line_idx + 1)
            .map(|&n| n.saturating_sub(1))
            .unwrap_or(self.md_text.len());
        let line_str = &self.md_text[line_start..line_end.min(self.md_text.len())];

        let target_utf16 = col.saturating_sub(1);
        let mut seen_utf16 = 0usize;
        let mut byte_off = 0usize;
        for ch in line_str.chars() {
            if seen_utf16 >= target_utf16 {
                break;
            }
            seen_utf16 += ch.len_utf16();
            byte_off += ch.len_utf8();
        }
        line_start + byte_off
    }
}

/// A single rendered glyph mapped back to its Markdown character coordinate.
#[derive(Debug, Clone, Copy)]
pub struct GlyphSpot {
    pub page: u16,
    pub x_pt: f32,
    pub y_pt: f32,
    pub w_pt: f32,
    pub h_pt: f32,
    pub md_byte: u32,
    pub line: u32,
    pub col: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClickJumpResult {
    pub line: u32,
    pub col: u32,
    pub x_pt: f32,
    pub y_pt: f32,
    pub w_pt: f32,
    pub h_pt: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct CursorJumpResult {
    pub page: u16,
    pub x_pt: f32,
    pub y_pt: f32,
    pub w_pt: f32,
    pub h_pt: f32,
    pub y_ratio: f32,
}

/// In-memory glyph- and element-level spatial & source index for a compiled document.
#[derive(Debug, Clone)]
pub struct SyncIndex {
    source_map: SourceMap,
    page_heights_pt: Vec<f32>,
    spots_by_page: Vec<Vec<GlyphSpot>>,
    elements_by_page: Vec<Vec<GlyphSpot>>,
    spots_by_md_byte: Vec<GlyphSpot>,
}

impl SyncIndex {
    pub fn build(doc: &PagedDocument, source: &Source, source_map: SourceMap) -> Self {
        let num_pages = doc.pages().len();
        let mut page_heights_pt = Vec::with_capacity(num_pages);
        let mut spots_by_page = vec![Vec::new(); num_pages];
        let mut elements_by_page = vec![Vec::new(); num_pages];
        let mut all_entries: Vec<(u8, GlyphSpot)> = Vec::new();

        for (page_idx, page) in doc.pages().iter().enumerate() {
            page_heights_pt.push(page.frame.height().to_pt() as f32);
            collect_frame_items(
                &page.frame,
                0.0,
                0.0,
                (page_idx + 1) as u16,
                source,
                &source_map,
                &mut spots_by_page[page_idx],
                &mut elements_by_page[page_idx],
            );
            for s in &spots_by_page[page_idx] {
                all_entries.push((0, *s));
            }
            for e in &elements_by_page[page_idx] {
                all_entries.push((1, *e));
            }
        }

        // Prefer glyphs over container shapes at the same md_byte, and prefer the highest page
        // when the same Markdown byte is rendered multiple times (e.g. TOC on Page 1 vs body).
        all_entries.sort_by_key(|(is_elem, s)| (s.md_byte, *is_elem, std::cmp::Reverse(s.page)));
        all_entries.dedup_by_key(|(_, s)| s.md_byte);
        let spots_by_md_byte = all_entries.into_iter().map(|(_, s)| s).collect();

        Self {
            source_map,
            page_heights_pt,
            spots_by_page,
            elements_by_page,
            spots_by_md_byte,
        }
    }

    pub fn glyph_count(&self) -> usize {
        self.spots_by_md_byte.len()
    }

    /// Resolve a preview click `(page: 1-based, x_pt, y_pt)` to an exact Markdown `(line, col)`
    /// for text glyphs, or to the owning Markdown line for non-text elements (images, diagrams,
    /// callout boxes, code blocks, horizontal rules, table borders).
    pub fn jump_from_click(&self, page: usize, x_pt: f64, y_pt: f64) -> Option<ClickJumpResult> {
        let page_idx = page.saturating_sub(1);
        let page_spots = self.spots_by_page.get(page_idx)?;
        let page_elems = self.elements_by_page.get(page_idx).map(|v| v.as_slice()).unwrap_or(&[]);
        if page_spots.is_empty() && page_elems.is_empty() {
            return None;
        }
        let px = x_pt as f32;
        let py = y_pt as f32;

        // 1. Check text glyphs first
        let mut best_glyph: Option<(f32, &GlyphSpot)> = None;
        for spot in page_spots {
            let dx = if px < spot.x_pt {
                spot.x_pt - px
            } else if px > spot.x_pt + spot.w_pt {
                px - (spot.x_pt + spot.w_pt)
            } else {
                0.0
            };
            let dy = if py < spot.y_pt {
                spot.y_pt - py
            } else if py > spot.y_pt + spot.h_pt {
                py - (spot.y_pt + spot.h_pt)
            } else {
                0.0
            };
            let score = dx * dx + (dy * 3.0) * (dy * 3.0);
            if best_glyph.is_none_or(|(b_score, _)| score < b_score) {
                best_glyph = Some((score, spot));
                if score == 0.0 {
                    break;
                }
            }
        }

        // Direct hit or immediate vicinity of a text glyph (~8pt horizontally / ~2.6pt vertically)
        if let Some((score, spot)) = best_glyph
            && score <= 64.0
        {
            let col = if px > spot.x_pt + spot.w_pt * 0.55 {
                spot.col.saturating_add(1)
            } else {
                spot.col
            };
            return Some(ClickJumpResult {
                line: spot.line,
                col,
                x_pt: spot.x_pt,
                y_pt: spot.y_pt,
                w_pt: spot.w_pt,
                h_pt: spot.h_pt,
            });
        }

        // 2. Check non-text elements (images, diagrams, rules, callout/code boxes):
        // pick the smallest-area element bounding box containing the click (4pt pad for thin rules).
        let mut best_elem: Option<(f32, &GlyphSpot)> = None;
        for elem in page_elems {
            let pad = 4.0;
            if px >= elem.x_pt - pad
                && px <= elem.x_pt + elem.w_pt + pad
                && py >= elem.y_pt - pad
                && py <= elem.y_pt + elem.h_pt + pad
            {
                let area = elem.w_pt.max(1.0) * elem.h_pt.max(1.0);
                if best_elem.is_none_or(|(b_area, _)| area < b_area) {
                    best_elem = Some((area, elem));
                }
            }
        }
        if let Some((_, elem)) = best_elem {
            return Some(ClickJumpResult {
                line: elem.line,
                col: elem.col,
                x_pt: elem.x_pt,
                y_pt: elem.y_pt,
                w_pt: elem.w_pt,
                h_pt: elem.h_pt,
            });
        }

        // 3. Fallback: nearest glyph within 45pt
        let (score, spot) = best_glyph?;
        if score > 45.0 * 45.0 {
            return None;
        }
        let col = if px > spot.x_pt + spot.w_pt * 0.55 {
            spot.col.saturating_add(1)
        } else {
            spot.col
        };

        Some(ClickJumpResult {
            line: spot.line,
            col,
            x_pt: spot.x_pt,
            y_pt: spot.y_pt,
            w_pt: spot.w_pt,
            h_pt: spot.h_pt,
        })
    }

    /// Resolve an editor cursor `(line: 1-based, col: 1-based)` to an exact PDF coordinate.
    pub fn jump_from_cursor(&self, line: usize, col: usize) -> Option<CursorJumpResult> {
        if self.spots_by_md_byte.is_empty() {
            return None;
        }
        let target_byte = self.source_map.line_col_to_md_byte(line, col) as u32;
        let idx = self
            .spots_by_md_byte
            .partition_point(|s| s.md_byte < target_byte);

        let spot = if idx == 0 {
            &self.spots_by_md_byte[0]
        } else if idx >= self.spots_by_md_byte.len() {
            &self.spots_by_md_byte[self.spots_by_md_byte.len() - 1]
        } else {
            let prev = &self.spots_by_md_byte[idx - 1];
            let next = &self.spots_by_md_byte[idx];
            if target_byte.saturating_sub(prev.md_byte) <= next.md_byte.saturating_sub(target_byte) {
                prev
            } else {
                next
            }
        };

        let page_h = self
            .page_heights_pt
            .get(spot.page.saturating_sub(1) as usize)
            .copied()
            .unwrap_or(842.0)
            .max(1.0);

        Some(CursorJumpResult {
            page: spot.page,
            x_pt: spot.x_pt,
            y_pt: spot.y_pt,
            w_pt: spot.w_pt,
            h_pt: spot.h_pt,
            y_ratio: (spot.y_pt / page_h).clamp(0.0, 1.0),
        })
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_frame_items(
    frame: &Frame,
    ox: f64,
    oy: f64,
    page: u16,
    source: &Source,
    source_map: &SourceMap,
    glyphs_out: &mut Vec<GlyphSpot>,
    elements_out: &mut Vec<GlyphSpot>,
) {
    for (pos, item) in frame.items() {
        let ix = ox + pos.x.to_pt();
        let iy = oy + pos.y.to_pt();
        match item {
            FrameItem::Group(group) => {
                let gx = ix + group.transform.tx.to_pt();
                let gy = iy + group.transform.ty.to_pt();
                collect_frame_items(&group.frame, gx, gy, page, source, source_map, glyphs_out, elements_out);
            }
            FrameItem::Text(text_item) => {
                let size_pt = text_item.size.to_pt();
                let top_y = (iy - size_pt * 0.85).max(0.0) as f32;
                let h_pt = (size_pt * 1.15).max(4.0) as f32;
                let mut cursor_x = ix;

                for glyph in &text_item.glyphs {
                    let adv = glyph.x_advance.at(text_item.size).to_pt();
                    let gx = (cursor_x + glyph.x_offset.at(text_item.size).to_pt()) as f32;
                    let gw = adv.abs().max(2.0) as f32;
                    cursor_x += adv;

                    let (span, offset) = glyph.span;
                    let span_range = source.find(span).map(|node| node.range());
                    if let Some(span_range) = span_range {
                        let typst_byte = span_range.start + (offset as usize);
                        if let Some(md_byte) = source_map.typst_byte_to_md_byte(typst_byte) {
                            let (line, col) = source_map.md_byte_to_line_col(md_byte);
                            glyphs_out.push(GlyphSpot {
                                page,
                                x_pt: gx,
                                y_pt: top_y,
                                w_pt: gw,
                                h_pt,
                                md_byte: md_byte as u32,
                                line,
                                col,
                            });
                        }
                    }
                }
            }
            FrameItem::Image(_, size, span) => {
                let w = size.x.to_pt().abs() as f32;
                let h = size.y.to_pt().abs() as f32;
                if w > 1.0 && h > 1.0
                    && let Some(span_range) = source.find(*span).map(|node| node.range())
                    && let Some(md_byte) = source_map.typst_byte_to_segment_md_start(span_range.start)
                {
                    let (line, col) = source_map.md_byte_to_line_col(md_byte);
                    elements_out.push(GlyphSpot {
                        page,
                        x_pt: ix as f32,
                        y_pt: iy as f32,
                        w_pt: w,
                        h_pt: h,
                        md_byte: md_byte as u32,
                        line,
                        col,
                    });
                }
            }
            FrameItem::Shape(shape, span) => {
                let r = shape.bbox(false);
                let sz = r.size();
                let w = sz.x.to_pt().abs() as f32;
                let h = sz.y.to_pt().abs() as f32;
                if (w > 2.0 || h > 2.0)
                    && let Some(span_range) = source.find(*span).map(|node| node.range())
                    && let Some(md_byte) = source_map.typst_byte_to_segment_md_start(span_range.start)
                {
                    let (line, col) = source_map.md_byte_to_line_col(md_byte);
                    elements_out.push(GlyphSpot {
                        page,
                        x_pt: (ix + r.min.x.to_pt()) as f32,
                        y_pt: (iy + r.min.y.to_pt()) as f32,
                        w_pt: w.max(4.0),
                        h_pt: h.max(4.0),
                        md_byte: md_byte as u32,
                        line,
                        col,
                    });
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::{self, DocumentSource, PublishOptions};

    #[test]
    fn test_char_level_roundtrip_prose_code_and_table() {
        let md = "\
---
title: Sync Test
theme: compact
---

# First Heading

Alpha line one of paragraph.
Beta line two of paragraph.

```rust
fn hello_world() {
    let x = 42;
}
```

| ColA | ColB |
| --- | --- |
| ValOne | ValTwo |
";
        let src = DocumentSource::memory_with_meta(md, Some("sync.md"), None);
        let opts = PublishOptions::default();
        let (_pages, sync_opt) = engine::publish_to_pages_with_sync(&src, &opts).expect("compile ok");
        let idx = sync_opt.expect("sync index present");
        assert!(idx.glyph_count() > 50);

        // 1. Heading on line 6: "# First Heading" -> "Heading" starts at col 9
        let h_pos = idx.jump_from_cursor(6, 9).expect("heading cursor");
        let h_back = idx.jump_from_click(h_pos.page as usize, h_pos.x_pt as f64 + 0.5, h_pos.y_pt as f64 + 2.0).expect("heading click");
        assert_eq!(h_back.line, 6);
        assert!((h_back.col as i32 - 9).abs() <= 2, "heading col: {}", h_back.col);

        // 2. Second line of multi-line paragraph (line 9: "Beta line two of paragraph.")
        let p2_pos = idx.jump_from_cursor(9, 11).expect("para line 2 cursor");
        let p2_back = idx.jump_from_click(p2_pos.page as usize, p2_pos.x_pt as f64 + 0.5, p2_pos.y_pt as f64 + 2.0).expect("para line 2 click");
        assert_eq!(p2_back.line, 9);
        assert!((p2_back.col as i32 - 11).abs() <= 2, "para line 2 col: {}", p2_back.col);

        // 3. Inside fenced code block (line 13: "    let x = 42;")
        let c_pos = idx.jump_from_cursor(13, 9).expect("code line cursor");
        let c_back = idx.jump_from_click(c_pos.page as usize, c_pos.x_pt as f64 + 0.5, c_pos.y_pt as f64 + 2.0).expect("code line click");
        assert_eq!(c_back.line, 13);
        assert!((c_back.col as i32 - 9).abs() <= 2, "code line col: {}", c_back.col);

        // 4. Inside table cell (line 19: "| ValOne | ValTwo |")
        let t_pos = idx.jump_from_cursor(19, 12).expect("table cell cursor");
        let t_back = idx.jump_from_click(t_pos.page as usize, t_pos.x_pt as f64 + 0.5, t_pos.y_pt as f64 + 2.0).expect("table cell click");
        assert_eq!(t_back.line, 19);
    }

    #[test]
    fn test_char_level_sync_survives_toc_tables_and_callouts() {
        let md = "\
---
title: Callout & Table Sync
toc: true
theme: compact
---

# Section One

Table: Summary table. {#tbl:sum}

| Name | Value |
| --- | --- |
| Alpha | 100 |
| Beta | 200 |

Paragraph right after table.

> [!NOTE]
> GitHub note callout body text here.

::: warning
Pandoc div callout body text here.
:::

# Section Two

Final paragraph after all callouts.
";
        let src = DocumentSource::memory_with_meta(md, Some("ct.md"), None);
        let opts = PublishOptions::default();
        let (_pages, sync_opt) = engine::publish_to_pages_with_sync(&src, &opts).expect("compile ok");
        let idx = sync_opt.expect("sync index present");

        // Table row (line 13: "| Alpha | 100 |")
        let r1 = idx.jump_from_cursor(13, 5).expect("table row");
        let r1_back = idx.jump_from_click(r1.page as usize, r1.x_pt as f64 + 0.5, r1.y_pt as f64 + 2.0).unwrap();
        assert_eq!(r1_back.line, 13);

        // Paragraph after table (line 16: "Paragraph right after table.")
        let p_after = idx.jump_from_cursor(16, 12).expect("after table");
        let p_back = idx.jump_from_click(p_after.page as usize, p_after.x_pt as f64 + 0.5, p_after.y_pt as f64 + 2.0).unwrap();
        assert_eq!(p_back.line, 16);
        assert!((p_back.col as i32 - 12).abs() <= 2, "after-table col: {}", p_back.col);

        // Inside GitHub callout (line 19: "> GitHub note callout body text here.")
        let gh = idx.jump_from_cursor(19, 15).expect("github callout");
        let gh_back = idx.jump_from_click(gh.page as usize, gh.x_pt as f64 + 0.5, gh.y_pt as f64 + 2.0).unwrap();
        assert_eq!(gh_back.line, 19);
        assert!((gh_back.col as i32 - 15).abs() <= 2, "github callout col: {}", gh_back.col);

        // Inside Pandoc div callout (line 22: "Pandoc div callout body text here.")
        let pd = idx.jump_from_cursor(22, 12).expect("pandoc callout");
        let pd_back = idx.jump_from_click(pd.page as usize, pd.x_pt as f64 + 0.5, pd.y_pt as f64 + 2.0).unwrap();
        assert_eq!(pd_back.line, 22);
        assert!((pd_back.col as i32 - 12).abs() <= 2, "pandoc callout col: {}", pd_back.col);

        // Final paragraph after all callouts (line 27: "Final paragraph after all callouts.")
        let fin = idx.jump_from_cursor(27, 10).expect("final paragraph");
        let fin_back = idx.jump_from_click(fin.page as usize, fin.x_pt as f64 + 0.5, fin.y_pt as f64 + 2.0).unwrap();
        assert_eq!(fin_back.line, 27);
        assert!((fin_back.col as i32 - 10).abs() <= 2, "final para col: {}", fin_back.col);
    }

    #[test]
    fn test_non_text_elements_rule_and_image_clickable() {
        let tmp = std::env::temp_dir().join(format!("botox-sync-img-{}.svg", std::process::id()));
        std::fs::write(
            &tmp,
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100"><rect width="200" height="100" fill="blue"/></svg>"#,
        )
        .unwrap();
        let img_name = tmp.file_name().unwrap().to_string_lossy().to_string();
        let md = format!(
            "---\ntitle: Elements\ntheme: compact\n---\n\nAbove rule.\n\n---\n\nBelow rule.\n\n![Blue Box]({img_name})\n"
        );
        let tmp_dir = std::env::temp_dir();
        let src = DocumentSource::memory_with_meta(&md, Some("el.md"), Some(&tmp_dir));
        let opts = PublishOptions::default();
        let (_pages, sync_opt) = engine::publish_to_pages_with_sync(&src, &opts).expect("compile ok");
        let _ = std::fs::remove_file(&tmp);
        let idx = sync_opt.expect("sync index present");

        // Horizontal rule is on line 8 (`---`)
        let rule_pos = idx.jump_from_cursor(8, 1).expect("rule cursor");
        let rule_back = idx
            .jump_from_click(rule_pos.page as usize, (rule_pos.x_pt + rule_pos.w_pt / 2.0) as f64, rule_pos.y_pt as f64)
            .expect("rule click");
        assert_eq!(rule_back.line, 8, "clicking horizontal rule should jump to line 8");

        // Image is on line 12 (`![Blue Box](...)`)
        let img_pos = idx.jump_from_cursor(12, 1).expect("image cursor");
        let img_back = idx
            .jump_from_click(
                img_pos.page as usize,
                (img_pos.x_pt + img_pos.w_pt / 2.0) as f64,
                (img_pos.y_pt + img_pos.h_pt / 2.0) as f64,
            )
            .expect("image center click");
        assert_eq!(img_back.line, 12, "clicking center of image should jump to line 12");
    }
}
