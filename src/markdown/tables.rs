use super::pandoc::parse_heading_attributes;

pub fn parse_caption_and_label(s: &str) -> (String, Option<String>) {
    let rest = if let Some(stripped) = s.strip_prefix("Table:") {
        stripped.trim()
    } else if let Some(stripped) = s.strip_prefix(':') {
        stripped.trim()
    } else {
        s.trim()
    };
    let (caption, _, id) = parse_heading_attributes(rest);
    (caption, id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_caption_and_label() {
        let (cap1, id1) = parse_caption_and_label("Table: Execution Times Across Threads {#tbl:threads}");
        assert_eq!(cap1, "Execution Times Across Threads");
        assert_eq!(id1.as_deref(), Some("tbl-threads"));

        let (cap2, id2) = parse_caption_and_label(": Comparison Matrix {#tbl:matrix}");
        assert_eq!(cap2, "Comparison Matrix");
        assert_eq!(id2.as_deref(), Some("tbl-matrix"));

        let (cap3, id3) = parse_caption_and_label("Summary of Experimental Results {#tbl:results}");
        assert_eq!(cap3, "Summary of Experimental Results");
        assert_eq!(id3.as_deref(), Some("tbl-results"));

        let (cap4, id4) = parse_caption_and_label("Table: Descriptive Table Without Label");
        assert_eq!(cap4, "Descriptive Table Without Label");
        assert_eq!(id4, None);
    }
}
