//! Document Emitter Seam
//!
//! Defines the `DocumentEmitter` trait and shared Typst formatting utilities
//! implemented by the `ArticleEmitter` (for documents) and `SlideEmitter` (for presentation slides).

pub const BOTOX_BODY_START: &str = "// BOTOX_BODY_START\n";

/// Deep interface for target typesetting layout emitters.
pub trait DocumentEmitter {
    /// Emits complete Typst markup with document preambles and layout from the transpiled body.
    fn emit(&self, body_typst: &str) -> String;
}

/// Standard cross-platform sans-serif fallbacks to ensure vector graphics,
/// embedded SVGs, and diagram text render with proportional sans-serif glyph metrics.
pub const SANS_SERIF_FALLBACK_STACK: &str = "\"Liberation Sans\", \"DejaVu Sans\", \"Arial\", \"Helvetica\", \"Nimbus Sans\"";

/// Formats a font name into a Typst font stack with reliable cross-platform fallbacks.
pub fn format_font_stack(font_raw: &str) -> String {
    if font_raw.eq_ignore_ascii_case("Times New Roman") || font_raw.eq_ignore_ascii_case("Times") {
        format!("(\"Times New Roman\", \"Nimbus Roman\", \"Liberation Serif\", \"New Computer Modern\", {SANS_SERIF_FALLBACK_STACK})")
    } else if font_raw.eq_ignore_ascii_case("New Computer Modern") {
        format!("(\"New Computer Modern\", {SANS_SERIF_FALLBACK_STACK})")
    } else if font_raw.eq_ignore_ascii_case("Liberation Sans")
        || font_raw.eq_ignore_ascii_case("Helvetica")
        || font_raw.eq_ignore_ascii_case("Arial")
        || font_raw.eq_ignore_ascii_case("Sans")
    {
        "(\"Liberation Sans\", \"Adwaita Sans\", \"DejaVu Sans\", \"Helvetica Neue\", \"Arial\", \"New Computer Modern\")".to_string()
    } else if font_raw.eq_ignore_ascii_case("Linux Libertine") || font_raw.eq_ignore_ascii_case("Libertinus Serif") {
        format!("(\"Linux Libertine\", \"Libertinus Serif\", \"Nimbus Roman\", \"Times New Roman\", \"New Computer Modern\", {SANS_SERIF_FALLBACK_STACK})")
    } else {
        format!("(\"{font_raw}\", \"New Computer Modern\", {SANS_SERIF_FALLBACK_STACK})")
    }
}

/// Renders a date string into Typst format, resolving `\today` to a dynamic datetime call.
pub fn render_date_value(d: &str) -> String {
    let trimmed = d.trim();
    if trimmed == r"\today" || trimmed.eq_ignore_ascii_case("today") {
        "#datetime.today().display(\"[day] [month repr:long] [year]\")".to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_font_stack_fallbacks() {
        assert!(format_font_stack("Times New Roman").contains("Nimbus Roman"));
        assert!(format_font_stack("Helvetica").contains("Liberation Sans"));
        assert!(format_font_stack("New Computer Modern").contains("New Computer Modern"));
        assert!(format_font_stack("New Computer Modern").contains("Liberation Sans"));
        assert!(format_font_stack("Custom Font").contains("Custom Font"));
        assert!(format_font_stack("Custom Font").contains("Liberation Sans"));
    }

    #[test]
    fn test_render_date_value() {
        assert!(render_date_value(r"\today").contains("datetime.today()"));
        assert!(render_date_value("today").contains("datetime.today()"));
        assert_eq!(render_date_value("2026-11-04"), "2026-11-04");
    }
}
