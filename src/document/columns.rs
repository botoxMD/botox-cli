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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_column_pagebreaks_single_column() {
        let input = "Col 1\n#colbreak()\nCol 2\n#colbreak(weak: true)\nCol 3\n#colbreak(weak: false)";
        let output = sanitize_column_pagebreaks(input, 1);
        assert!(!output.contains("#colbreak()"));
        assert!(!output.contains("#colbreak(weak: true)"));
        assert!(!output.contains("#colbreak(weak: false)"));
        assert!(output.contains("#pagebreak()"));
        assert!(output.contains("#pagebreak(weak: true)"));
        assert!(output.contains("#pagebreak(weak: false)"));
    }

    #[test]
    fn test_sanitize_column_pagebreaks_multi_column() {
        let input = "Col 1\n#colbreak()\nCol 2\n#pagebreak()";
        let output_2col = sanitize_column_pagebreaks(input, 2);
        assert_eq!(output_2col, input);

        let output_3col = sanitize_column_pagebreaks(input, 3);
        assert_eq!(output_3col, input);
    }
}
