pub fn sanitize_column_pagebreaks(content: &str, columns: usize) -> String {
    if columns > 1 {
        content
            .replace("#pagebreak()", "#colbreak()")
            .replace("#pagebreak(weak: true)", "#colbreak(weak: true)")
            .replace("#pagebreak(weak: false)", "#colbreak(weak: false)")
    } else {
        content
            .replace("#colbreak()", "#pagebreak()")
            .replace("#colbreak(weak: true)", "#pagebreak(weak: true)")
            .replace("#colbreak(weak: false)", "#pagebreak(weak: false)")
    }
}
