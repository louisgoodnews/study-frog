//! Setting `request_retention` in the text of `studyfrog.toml` (under `[srs]`) or `_stack.toml` (top level)
//! without touching anything else: comments, the order of keys and the other tables stay (STORY-0106). Pure.

use crate::values::float_text;

const KEY: &str = "request_retention";

fn is_key_line(line: &str) -> bool {
    line.trim_start()
        .strip_prefix(KEY)
        .is_some_and(|rest| rest.trim_start().starts_with('='))
}

fn is_table(line: &str) -> bool {
    line.trim_start().starts_with('[')
}

/// The lines `[from, to)` of the section: the top level (`table` `None`) or the table named `table`.
fn section(lines: &[&str], table: Option<&str>) -> Option<(usize, usize)> {
    let from = match table {
        None => 0,
        Some(t) => {
            let header = format!("[{t}]");
            lines.iter().position(|l| l.trim() == header)? + 1
        }
    };
    let to = lines[from..]
        .iter()
        .position(|l| is_table(l))
        .map_or(lines.len(), |i| from + i);
    Some((from, to))
}

/// The text with `request_retention = r` in the top level (`table` `None`, as in `_stack.toml`) or in the table
/// `table` (`Some("srs")`, as in `studyfrog.toml`): the line replaced when there is one, else added at the start of
/// the table (at the end of the top level), or a new table at the end of the file.
pub fn with_request_retention(text: &str, r: f32, table: Option<&str>) -> String {
    let line = format!("{KEY} = {}", float_text(r));
    let lines: Vec<&str> = text.lines().collect();
    let mut out: Vec<String> = lines.iter().map(|l| (*l).to_owned()).collect();
    match section(&lines, table) {
        Some((from, to)) => match lines[from..to].iter().position(|l| is_key_line(l)) {
            Some(i) => out[from + i] = line,
            None if table.is_some() => out.insert(from, line),
            None => out.insert(to, line),
        },
        None => {
            if !out.is_empty() {
                out.push(String::new());
            }
            out.push(format!("[{}]", table.unwrap_or_default()));
            out.push(line);
        }
    }
    let mut joined = out.join("\n");
    joined.push('\n');
    joined
}
