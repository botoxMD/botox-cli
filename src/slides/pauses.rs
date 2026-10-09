pub fn process_slide_pauses(trimmed_slide: &str, out: &mut String) {
    let raw_chunks: Vec<&str> = trimmed_slide.split("#botox_pause()").collect();
    let mut valid_chunks: Vec<&str> = raw_chunks
        .into_iter()
        .map(|c| c.trim())
        .filter(|c| !c.is_empty())
        .collect();
    if valid_chunks.is_empty() {
        valid_chunks.push(trimmed_slide);
    }
    let total_steps = valid_chunks.len();
    let mut accumulated = String::new();
    for (step_idx, chunk) in valid_chunks.iter().enumerate() {
        if step_idx > 0 {
            out.push_str("\n#pagebreak()\n\n");
        }
        if !accumulated.is_empty() {
            accumulated.push_str("\n\n");
        }
        accumulated.push_str(chunk);
        out.push_str(&accumulated);

        if step_idx + 1 < total_steps {
            out.push_str("\n#place(top + left)[#text(size: 0.001pt, fill: rgb(0, 0, 0, 0))[botox-pause-step]]\n");
        }
    }
    out.push('\n');
}
