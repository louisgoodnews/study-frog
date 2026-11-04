//! Rewriting markdown links and image references when files or folders move or rename.

use studyfrog_md::codeblock::{FenceState, advance_fence};

use crate::folder_ops_path::normalize_segments;

/// Split target into (path_part, extra_part) preserving `#anchor` and `"title"`.
fn split_target_extras(target: &str) -> (&str, &str) {
    let mut end = target.len();
    if let Some(idx) = target.find(['"', '\'']) {
        end = idx;
    }
    let before_quote = target[..end].trim_end();
    if let Some(hash_idx) = before_quote.find('#') {
        (&before_quote[..hash_idx], &target[hash_idx..])
    } else {
        (before_quote, &target[before_quote.len()..])
    }
}

/// Match target against moved file mappings, returning the rewritten target if matched.
fn match_and_replace_target(
    base_dir: &str,
    target: &str,
    mappings: &[(String, String)],
) -> Option<String> {
    let trimmed = target.trim();
    if trimmed.is_empty() {
        return None;
    }
    let (path_part, extra) = split_target_extras(trimmed);
    for (old_path, new_path) in mappings {
        if path_part == old_path {
            return Some(format!("{new_path}{extra}"));
        }
        if path_part.starts_with(&format!("{old_path}/")) {
            let sub = &path_part[old_path.len() + 1..];
            return Some(format!("{new_path}/{sub}{extra}"));
        }
    }
    if let Some(norm) = normalize_segments(base_dir, path_part) {
        for (old_path, new_path) in mappings {
            if norm == *old_path {
                return Some(format!("{new_path}{extra}"));
            }
            if norm.starts_with(&format!("{old_path}/")) {
                let sub = &norm[old_path.len() + 1..];
                return Some(format!("{new_path}/{sub}{extra}"));
            }
        }
    }
    None
}

/// Find end of a code span started at `i`.
fn code_span_end(bytes: &[u8], i: usize) -> Option<usize> {
    let open = bytes[i..].iter().take_while(|b| **b == b'`').count();
    let mut j = i + open;
    while j < bytes.len() {
        let run = bytes[j..].iter().take_while(|b| **b == b'`').count();
        if run == open {
            return Some(j + run);
        }
        j += run.max(1);
    }
    None
}

/// Rewrite link targets on a single markdown line outside code spans.
fn rewrite_line(base_dir: &str, line: &str, mappings: &[(String, String)]) -> (String, bool) {
    let bytes = line.as_bytes();
    let mut out = String::with_capacity(line.len());
    let mut i = 0;
    let mut changed = false;

    while i < bytes.len() {
        if bytes[i] == b'\\' && i + 1 < bytes.len() {
            let next_ch = line[i + 1..].chars().next().unwrap_or(' ');
            out.push('\\');
            out.push(next_ch);
            i += 1 + next_ch.len_utf8();
            continue;
        }
        if bytes[i] == b'`'
            && let Some(end) = code_span_end(bytes, i)
        {
            out.push_str(&line[i..end]);
            i = end;
            continue;
        }
        if bytes[i..].starts_with(b"![[")
            && let Some(end_idx) = line[i + 3..].find("]]")
        {
            let inner = &line[i + 3..i + 3 + end_idx];
            if let Some(replacement) = match_and_replace_target(base_dir, inner, mappings) {
                out.push_str("![[");
                out.push_str(&replacement);
                out.push_str("]]");
                i = i + 3 + end_idx + 2;
                changed = true;
                continue;
            }
        }
        if bytes[i..].starts_with(b"[[")
            && let Some(end_idx) = line[i + 2..].find("]]")
        {
            out.push_str(&line[i..i + 2 + end_idx + 2]);
            i = i + 2 + end_idx + 2;
            continue;
        }
        let is_img = bytes[i] == b'!' && i + 1 < bytes.len() && bytes[i + 1] == b'[';
        let start_bracket = if is_img { i + 1 } else { i };
        if bytes[start_bracket] == b'['
            && let Some((close_b, close_p)) = find_bracket_paren(line, start_bracket)
        {
            let target = &line[close_b + 2..close_p];
            if let Some(replacement) = match_and_replace_target(base_dir, target, mappings) {
                out.push_str(&line[i..close_b + 2]);
                out.push_str(&replacement);
                out.push(')');
                i = close_p + 1;
                changed = true;
                continue;
            }
        }
        let ch = line[i..].chars().next().unwrap_or(' ');
        out.push(ch);
        i += ch.len_utf8();
    }
    (out, changed)
}

/// Find matching `]` followed immediately by `(` and its matching `)`.
fn find_bracket_paren(line: &str, open_b: usize) -> Option<(usize, usize)> {
    let bytes = line.as_bytes();
    let mut depth = 0;
    let mut close_b = None;
    let mut j = open_b;
    while j < bytes.len() {
        if bytes[j] == b'[' {
            depth += 1;
        } else if bytes[j] == b']' {
            depth -= 1;
            if depth == 0 {
                close_b = Some(j);
                break;
            }
        }
        j += 1;
    }
    let b_idx = close_b?;
    if b_idx + 1 >= bytes.len() || bytes[b_idx + 1] != b'(' {
        return None;
    }
    let mut p_depth = 0;
    let mut k = b_idx + 1;
    while k < bytes.len() {
        if bytes[k] == b'(' {
            p_depth += 1;
        } else if bytes[k] == b')' {
            p_depth -= 1;
            if p_depth == 0 {
                return Some((b_idx, k));
            }
        }
        k += 1;
    }
    None
}

/// Rewrite all links and picture references in `text` pointing to `mappings`.
pub fn rewrite_markdown_links(
    base_dir: &str,
    text: &str,
    mappings: &[(String, String)],
) -> (String, bool) {
    if mappings.is_empty() {
        return (text.to_owned(), false);
    }
    let mut out_lines = Vec::new();
    let mut state: FenceState = None;
    let mut any_changed = false;

    for line in text.lines() {
        let (next, in_code) = advance_fence(state, line);
        state = next;
        if in_code {
            out_lines.push(line.to_owned());
        } else {
            let (new_line, ch) = rewrite_line(base_dir, line, mappings);
            if ch {
                any_changed = true;
            }
            out_lines.push(new_line);
        }
    }
    let mut result = out_lines.join("\n");
    if text.ends_with('\n') {
        result.push('\n');
    }
    (result, any_changed)
}
