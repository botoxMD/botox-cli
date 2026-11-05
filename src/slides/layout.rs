//! Slide Deck Layout Coordinator
//!
//! Handles slide deck segmentation (splitting slides outside code fences),
//! incremental pause reveals, and title-slide hero formatting.

use crate::slides::pauses::{process_slide_pauses, split_outside_code_fences};

/// Layout coordinator for presentation slide decks.
#[derive(Debug, Clone)]
pub struct SlideDeckLayout<'a> {
    pub heading_color: &'a str,
    pub muted_color: &'a str,
}

impl<'a> SlideDeckLayout<'a> {
    pub fn new(heading_color: &'a str, muted_color: &'a str) -> Self {
        Self {
            heading_color,
            muted_color,
        }
    }

    /// Tests whether a slide is a top-level title slide (has H1 `= ` and no H2 `== `).
    pub fn is_title_slide(&self, slide_text: &str) -> bool {
        let has_h1 = slide_text.lines().any(|l| l.starts_with("= "));
        let has_h2 = slide_text.lines().any(|l| l.starts_with("== "));
        has_h1 && !has_h2
    }

    /// Renders the complete body of slides by segmenting slides outside code fences,
    /// placing title slides, stepping the slide counter, and expanding pause points.
    pub fn layout_slides(&self, slides_typst: &str) -> String {
        let mut out = String::new();
        let raw_slides = split_outside_code_fences(slides_typst, "#pagebreak()");
        let mut first_emitted = false;

        for (i, slide) in raw_slides.iter().enumerate() {
            let trimmed = slide.trim();
            if trimmed.is_empty() {
                continue;
            }

            let is_first_logical_slide = !first_emitted;
            first_emitted = true;

            if !is_first_logical_slide {
                out.push_str("\n#pagebreak()\n#botox_slide_counter.step()\n\n");
            }

            if i == 0 && self.is_title_slide(trimmed) && !trimmed.contains("#botox_pause()") {
                out.push_str(&format!(
                    "#place(center + horizon)[\n  #align(center)[\n    #show heading.where(level: 1): it => block(below: 0.6em)[#text(size: 2.1em, weight: \"bold\", fill: rgb(\"{}\"))[#it.body]]\n    #show heading.where(level: 3): it => block(below: 0.4em)[#text(size: 1.15em, style: \"italic\", fill: rgb(\"{}\"))[#it.body]]\n    {trimmed}\n  ]\n]\n",
                    self.heading_color, self.muted_color
                ));
            } else if trimmed.contains("#botox_pause()") {
                process_slide_pauses(trimmed, &mut out);
            } else {
                out.push_str(trimmed);
                out.push('\n');
            }
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slide_deck_layout_title_slide_detection() {
        let layout = SlideDeckLayout::new("#000000", "#666666");
        assert!(layout.is_title_slide("= Main Title\nSubtitle info"));
        assert!(!layout.is_title_slide("= Section Title\n== Subtitle or Sub-bullet"));
        assert!(!layout.is_title_slide("== Only H2"));
    }

    #[test]
    fn test_slide_deck_layout_renders_pauses_and_pages() {
        let layout = SlideDeckLayout::new("#000000", "#666666");
        let content = "= Slide 1\n#pagebreak()\n== Slide 2\nPoint A\n#botox_pause()\nPoint B";
        let out = layout.layout_slides(content);
        assert!(out.contains("Slide 1"));
        assert!(out.contains("#pagebreak()"));
        assert!(out.contains("Point A"));
        assert!(out.contains("Point B"));
    }
}
