//! Obsidian front matter, tags and the syntax StudyFrog does not render.

use std::collections::BTreeMap;

const RESERVED: [&str; 8] = [
    "id", "kind", "title", "tags", "aliases", "created", "updated", "status",
];

/// The front matter (`key: value` lines between two `---` lines) and the body that follows it.
pub(crate) fn split_front(text: &str) -> (Vec<String>, &str) {
    let Some(rest) = text
        .strip_prefix("---\n")
        .or_else(|| text.strip_prefix("---\r\n"))
    else {
        return (Vec::new(), text);
    };
    let mut at = 0;
    for line in rest.split_inclusive('\n') {
        if line.trim_end() == "---" {
            return (
                rest[..at].lines().map(str::to_owned).collect(),
                &rest[at + line.len()..],
            );
        }
        at += line.len();
    }
    (Vec::new(), text)
}

fn list_of(value: &str) -> Vec<String> {
    value
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(',')
        .map(|t| {
            t.trim()
                .trim_matches(['"', '\''])
                .trim_start_matches('#')
                .to_owned()
        })
        .filter(|t| !t.is_empty())
        .collect()
}

#[derive(Default)]
pub(crate) struct Front {
    pub(crate) title: Option<String>,
    pub(crate) tags: Vec<String>,
    pub(crate) extra: Vec<(String, String)>,
    pub(crate) dropped: Vec<String>,
}

pub(crate) fn read_front(lines: &[String]) -> Front {
    let mut f = Front::default();
    let mut key: Option<String> = None;
    for line in lines {
        if let Some(item) = line
            .trim_start()
            .strip_prefix("- ")
            .filter(|_| line.starts_with(' ') || line.starts_with('-'))
        {
            if key.as_deref() == Some("tags") {
                f.tags.extend(list_of(item));
            } else if let Some(k) = key.as_ref().filter(|k| !f.dropped.contains(k)) {
                f.dropped.push(k.clone());
            }
            continue;
        }
        let Some((k, v)) = line.split_once(':') else {
            continue;
        };
        let (k, v) = (k.trim().to_owned(), v.trim());
        key = Some(k.clone());
        match (k.as_str(), v.is_empty()) {
            ("title", false) => f.title = Some(v.trim_matches(['"', '\'']).to_owned()),
            ("tags", false) => f.tags.extend(list_of(v)),
            (name, false)
                if !RESERVED.contains(&name)
                    && !name.contains(' ')
                    && !v.starts_with(['[', '{']) =>
            {
                f.extra.push((k.clone(), v.to_owned()))
            }
            (name, _) if RESERVED.contains(&name) && name != "tags" && name != "title" => {
                f.dropped.push(k.clone())
            }
            (_, true) | (_, false) => {}
        }
    }
    f
}

pub(crate) fn inline_tags(body: &str) -> Vec<String> {
    let (mut tags, mut fenced) = (Vec::new(), false);
    for line in body.lines() {
        if line.trim_start().starts_with("```") {
            fenced = !fenced;
        }
        if fenced
            || line.trim_start().starts_with('#')
                && line
                    .trim_start()
                    .chars()
                    .nth(1)
                    .is_some_and(|c| c == ' ' || c == '#')
        {
            continue;
        }
        for word in line.split_whitespace() {
            let t = word
                .strip_prefix('#')
                .unwrap_or_default()
                .trim_end_matches(['.', ',', ';', ')']);
            if t.chars().next().is_some_and(char::is_alphabetic)
                && t.chars()
                    .all(|c| c.is_alphanumeric() || matches!(c, '-' | '_' | '/'))
                && !tags.contains(&t.to_owned())
            {
                tags.push(t.to_owned());
            }
        }
    }
    tags
}

pub(crate) fn count_unsupported(body: &str) -> BTreeMap<&'static str, usize> {
    let mut n = BTreeMap::new();
    let mut add = |k: &'static str, c: usize| {
        if c > 0 {
            *n.entry(k).or_insert(0) += c;
        }
    };
    add(
        "callouts",
        body.lines()
            .filter(|l| l.trim_start().starts_with("> [!"))
            .count(),
    );
    add("dataview blocks", body.matches("```dataview").count());
    add("comments", body.matches("%%").count() / 2);
    add("highlights", body.matches("==").count() / 2);
    add(
        "block references",
        body.lines()
            .filter(|l| {
                l.trim_end().rsplit_once(" ^").is_some_and(|(_, id)| {
                    !id.is_empty() && id.chars().all(|c| c.is_alphanumeric() || c == '-')
                })
            })
            .count(),
    );
    add("math blocks", body.matches("$$").count() / 2);
    n
}
