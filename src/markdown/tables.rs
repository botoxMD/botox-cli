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
