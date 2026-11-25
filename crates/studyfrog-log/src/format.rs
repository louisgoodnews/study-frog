//! The one-line text form of a record.

use std::fmt::Write as _;

use crate::record::Record;
use crate::severity::name;

/// Formats a record as `<unix_ms> <LEVEL> <target> <message> key=value ...`.
/// A value that is empty or contains whitespace, `=`, a quote, a backslash or a control character
/// is wrapped in quotes, with `"` and `\` escaped. Control characters are written as escapes so
/// that each record stays on one line.
pub fn format_line(record: &Record) -> String {
    let mut line = String::new();
    let _ = write!(line, "{} {} ", record.unix_ms, name(record.level));
    push_text(&mut line, &record.target, false);
    line.push(' ');
    push_text(&mut line, &record.message, false);
    for (key, value) in &record.fields {
        line.push(' ');
        line.push_str(key);
        line.push('=');
        if needs_quotes(value) {
            line.push('"');
            push_text(&mut line, value, true);
            line.push('"');
        } else {
            line.push_str(value);
        }
    }
    line
}

/// Appends `text`, escaping control characters. Inside quotes, `"` and `\` are escaped too.
fn push_text(out: &mut String, text: &str, quoted: bool) {
    for ch in text.chars() {
        match ch {
            '\\' | '"' if quoted => {
                out.push('\\');
                out.push(ch);
            }
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            control if control.is_control() => {
                let _ = write!(out, "\\u{{{:x}}}", u32::from(control));
            }
            plain => out.push(plain),
        }
    }
}

fn needs_quotes(value: &str) -> bool {
    value.is_empty()
        || value
            .chars()
            .any(|c| c.is_whitespace() || c.is_control() || matches!(c, '"' | '=' | '\\'))
}
