pub fn sanitize_column_pagebreaks(content: &str, columns: usize) -> String {
    if columns > 1 {
        // Inside Typst's `columns.with(...)` container, `#pagebreak()` is strictly forbidden
        // by Typst and causes a fatal compilation error:
        // "pagebreaks are not allowed inside of containers (hint: try using a `#colbreak()` instead)".
        // Sanitizing `#pagebreak()` to `#colbreak()` ensures valid compilation in multi-column layouts.
        content
            .replace("#pagebreak()", "#colbreak()")
            .replace("#pagebreak(weak: true)", "#colbreak(weak: true)")
            .replace("#pagebreak(weak: false)", "#colbreak(weak: false)")
    } else {
        // In single-column documents, column breaks have no second column to advance to,
        // so they gracefully map to physical page breaks.
        content
            .replace("#colbreak()", "#pagebreak()")
            .replace("#colbreak(weak: true)", "#pagebreak(weak: true)")
            .replace("#colbreak(weak: false)", "#pagebreak(weak: false)")
    }
}
