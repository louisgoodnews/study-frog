//! The goals of a stack: a bullet list the user writes under a `## Goals` heading in the stack's own Markdown file.
//! Pure text functions; the rest of the body is never touched.

const HEADING: &str = "## Goals";

fn is_heading(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with('#') && t.trim_start_matches('#').starts_with(' ')
}

/// The line range of the Goals section: its heading and the lines up to the next heading (fences are respected).
fn section(lines: &[&str]) -> Option<(usize, usize)> {
    let mut fenced = false;
    let mut start = None;
    for (i, l) in lines.iter().enumerate() {
        if l.trim_start().starts_with("```") {
            fenced = !fenced;
            continue;
        }
        if fenced {
            continue;
        }
        match start {
            None if l.trim() == HEADING => start = Some(i),
            Some(s) if is_heading(l) => return Some((s, i)),
            _ => {}
        }
    }
    start.map(|s| (s, lines.len()))
}

/// The goals written in `body`, without the heading; empty when there is no section.
pub fn goals_of(body: &str) -> String {
    let lines: Vec<&str> = body.lines().collect();
    let Some((s, e)) = section(&lines) else {
        return String::new();
    };
    lines[s + 1..e].join("\n").trim().to_owned()
}

/// Whether `text` can be stored as goals: it must not contain a heading line, which would end the section.
pub fn goals_problem(text: &str) -> Option<&'static str> {
    text.lines()
        .any(is_heading)
        .then_some("goals are a list: a line starting with # would start another section")
}

/// `body` with its goals replaced by `goals`; the section is added at the end when missing and removed when `goals` is empty.
pub fn with_goals(body: &str, goals: &str) -> String {
    let lines: Vec<&str> = body.lines().collect();
    let (before, after) = match section(&lines) {
        Some((s, e)) => (&lines[..s], &lines[e..]),
        None => (&lines[..], &lines[lines.len()..]),
    };
    let mut parts: Vec<String> = Vec::new();
    let head = before.join("\n").trim_end().to_owned();
    if !head.is_empty() {
        parts.push(head);
    }
    if !goals.trim().is_empty() {
        parts.push(format!("{HEADING}\n\n{}", goals.trim()));
    }
    let tail = after.join("\n").trim().to_owned();
    if !tail.is_empty() {
        parts.push(tail);
    }
    if parts.is_empty() {
        String::new()
    } else {
        format!("{}\n", parts.join("\n\n"))
    }
}
