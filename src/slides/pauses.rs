/// Splits markup by an exact command string (e.g. `#pagebreak()` or `#botox_pause()`),
/// but ONLY when the command appears outside of code fences (``` or ~~~).
pub fn split_outside_code_fences<'a>(markup: &'a str, command: &str) -> Vec<&'a str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut in_code_fence = false;
    let mut fence_char = '`';
    let mut fence_len = 0;

    let mut line_start = 0;
    while line_start < markup.len() {
        let line_end = markup[line_start..].find('\n').map(|idx| line_start + idx).unwrap_or(markup.len());
        let line = &markup[line_start..line_end];
        let trimmed = line.trim_start();

        if !in_code_fence {
            let backticks = trimmed.chars().take_while(|&c| c == '`').count();
            let tildes = trimmed.chars().take_while(|&c| c == '~').count();
            if backticks >= 3 {
                in_code_fence = true;
                fence_char = '`';
                fence_len = backticks;
            } else if tildes >= 3 {
                in_code_fence = true;
                fence_char = '~';
                fence_len = tildes;
            } else if let Some(cmd_idx) = line.find(command) {
                let split_point = line_start + cmd_idx;
                parts.push(&markup[start..split_point]);
                start = split_point + command.len();
            }
        } else {
            let count = trimmed.chars().take_while(|&c| c == fence_char).count();
            if count >= fence_len {
                in_code_fence = false;
            }
        }

        line_start = if line_end < markup.len() { line_end + 1 } else { markup.len() };
    }

    if start <= markup.len() {
        parts.push(&markup[start..]);
    }

    parts
}

pub fn process_slide_pauses(trimmed_slide: &str, out: &mut String) {
    let raw_chunks = split_outside_code_fences(trimmed_slide, "#botox_pause()");
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
