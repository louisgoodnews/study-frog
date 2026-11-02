//! Anki's text: HTML to Markdown text, and clozes.

use std::collections::HashMap;

fn entities(s: &str) -> String {
    [
        ("&nbsp;", " "),
        ("&lt;", "<"),
        ("&gt;", ">"),
        ("&quot;", "\""),
        ("&#39;", "'"),
        ("&amp;", "&"),
    ]
    .iter()
    .fold(s.to_owned(), |t, (a, b)| t.replace(a, b))
}

/// Anki's HTML as Markdown text: line breaks kept, pictures as image links, every other tag removed.
pub fn html_to_text(html: &str, media: &HashMap<String, String>) -> String {
    let (mut out, mut tag) = (String::new(), String::new());
    let mut in_tag = false;
    for c in html.chars() {
        match (c, in_tag) {
            ('<', false) => {
                in_tag = true;
                tag.clear();
            }
            ('>', true) => {
                in_tag = false;
                let lower = tag.to_lowercase();
                if lower.starts_with("br") || lower.starts_with("/div") || lower.starts_with("/p") {
                    out.push('\n');
                } else if lower.starts_with("img") {
                    let src = lower
                        .find("src=")
                        .map(|i| &tag[i + 4..])
                        .map(|r| r.trim_start_matches(['"', '\'']));
                    let name = src
                        .and_then(|r| r.split(['"', '\'', ' ', '>']).next())
                        .unwrap_or_default();
                    if let Some(path) = media.get(&name.to_lowercase()) {
                        out.push_str(&format!("![]({path})"));
                    }
                }
            }
            (c, true) => tag.push(c),
            (c, false) => out.push(c),
        }
    }
    let text = entities(&out);
    text.lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_owned()
}

/// A cloze text for the cloze number `n`: that number's blanks hidden as `[...]`, the others shown. `None` shows all.
pub fn cloze(text: &str, hide: Option<u32>) -> String {
    let (mut out, mut rest) = (String::new(), text);
    while let Some(i) = rest.find("{{c") {
        out.push_str(&rest[..i]);
        let after = &rest[i + 3..];
        let Some(end) = after.find("}}") else {
            out.push_str(&rest[i..]);
            return out;
        };
        let inner = &after[..end];
        let (num, body) = inner
            .split_once("::")
            .map_or((0, inner), |(n, b)| (n.parse().unwrap_or(0), b));
        let answer = body.split("::").next().unwrap_or_default();
        out.push_str(&if Some(num) == hide {
            "[...]".to_owned()
        } else {
            answer.to_owned()
        });
        rest = &after[end + 2..];
    }
    out + rest
}

pub(crate) fn numbers(text: &str) -> Vec<u32> {
    let mut found: Vec<u32> = text
        .split("{{c")
        .skip(1)
        .filter_map(|p| p.split("::").next()?.parse().ok())
        .collect();
    found.sort_unstable();
    found.dedup();
    found
}
