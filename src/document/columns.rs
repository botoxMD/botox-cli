pub fn sanitize_column_pagebreaks(content: &str, columns: usize) -> String {
    if columns == 1 {
        // In single-column documents, column breaks have no second column to advance to,
        // so they gracefully map to physical page breaks.
        content
            .replace("#colbreak()", "#pagebreak()")
            .replace("#colbreak(weak: true)", "#pagebreak(weak: true)")
            .replace("#colbreak(weak: false)", "#pagebreak(weak: false)")
    } else {
        // In multi-column documents using page-level columns, both #pagebreak()
        // and #colbreak() are natively supported by Typst layout.
        content.to_string()
    }
}
