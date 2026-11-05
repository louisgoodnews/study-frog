//! RFC 4180 delimited text parser with delimiter sniffing and header mapping.

use studyfrog_ext::types::ImportedCard;

struct RawRow {
    line: usize,
    fields: Vec<String>,
}

fn sniff_delimiter(text: &str, preferred: Option<char>) -> char {
    for line in text.lines().map(str::trim).filter(|l| !l.is_empty()) {
        if let Some(c) = line.strip_prefix('#') {
            let l = c.to_ascii_lowercase();
            if l.contains("separator:tab") || l.contains("separator:\\t") {
                return '\t';
            } else if l.contains("separator:comma") || l.contains("separator:,") {
                return ',';
            } else if l.contains("separator:semicolon") || l.contains("separator:;") {
                return ';';
            }
            continue;
        }
        let (mut tabs, mut commas, mut semis, mut in_q) = (0usize, 0usize, 0usize, false);
        for c in line.chars() {
            if c == '"' {
                in_q = !in_q;
            } else if !in_q {
                match c {
                    '\t' => tabs += 1,
                    ',' => commas += 1,
                    ';' => semis += 1,
                    _ => {}
                }
            }
        }
        if tabs > 0 {
            return '\t';
        }
        if commas > 0 {
            return ',';
        }
        if semis > 0 {
            return ';';
        }
        break;
    }
    preferred.unwrap_or('\t')
}

fn clean(field: &str) -> String {
    let mut s = field.trim().to_owned();
    for br in ["<br>", "<br/>", "<br />", "<BR>", "<BR/>", "<BR />"] {
        s = s.replace(br, "\n");
    }
    s.trim().to_owned()
}

fn parse_tags(raw: &str) -> Vec<String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Vec::new();
    }
    if raw.contains(';') {
        raw.split(';')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect()
    } else {
        raw.split_whitespace().map(str::to_owned).collect()
    }
}

fn parse_rows(text: &str, delimiter: char) -> Result<Vec<RawRow>, String> {
    let (mut rows, mut chars) = (Vec::new(), text.chars().peekable());
    let (mut line, mut start_line) = (1usize, 1usize);
    let (mut in_q, mut in_comment, mut quoted) = (false, false, false);
    let (mut field, mut row, mut at_start) = (String::new(), Vec::new(), true);

    while let Some(c) = chars.next() {
        if in_comment {
            if c == '\n' || (c == '\r' && chars.next_if_eq(&'\n').is_some()) {
                line += 1;
                in_comment = false;
                at_start = true;
                start_line = line;
            }
            continue;
        }
        if !in_q && at_start && c == '#' {
            in_comment = true;
            continue;
        }
        if in_q {
            if c == '"' {
                if chars.next_if_eq(&'"').is_some() {
                    field.push('"');
                } else {
                    in_q = false;
                }
            } else if c == '\n' || (c == '\r' && chars.next_if_eq(&'\n').is_some()) {
                line += 1;
                field.push('\n');
            } else {
                field.push(c);
            }
        } else if c == '"' && field.trim().is_empty() && !quoted {
            in_q = true;
            quoted = true;
            field.clear();
        } else if c == delimiter {
            row.push(field);
            field = String::new();
            quoted = false;
            at_start = false;
        } else if c == '\n' || c == '\r' {
            if c == '\r' {
                chars.next_if_eq(&'\n');
            }
            line += 1;
            at_start = true;
            if !row.is_empty() || !field.trim().is_empty() || quoted {
                row.push(field);
                rows.push(RawRow {
                    line: start_line,
                    fields: row,
                });
                row = Vec::new();
            }
            field = String::new();
            quoted = false;
            start_line = line;
        } else {
            field.push(c);
            at_start = false;
        }
    }
    if in_q {
        return Err(format!("line {start_line}: unclosed quote"));
    }
    if !row.is_empty() || !field.trim().is_empty() || quoted {
        row.push(field);
        rows.push(RawRow {
            line: start_line,
            fields: row,
        });
    }
    Ok(rows)
}

struct Mapping {
    front: usize,
    back: usize,
    title: Option<usize>,
    tags: Option<usize>,
}

fn detect_header(row: &[String]) -> Result<Option<Mapping>, String> {
    const FRONT: &[&str] = &["front", "question", "prompt", "term", "q"];
    const BACK: &[&str] = &["back", "answer", "definition", "a"];
    const PROMPT: &[&str] = &["stack", "deck", "folder", "kind", "type"];
    let find = |names: &[&str]| {
        row.iter()
            .position(|c| names.iter().any(|n| c.trim().eq_ignore_ascii_case(n)))
    };
    match (find(FRONT), find(BACK)) {
        (Some(f), Some(b)) => Ok(Some(Mapping {
            front: f,
            back: b,
            title: find(&["title", "name"]),
            tags: find(&["tags", "tag"]),
        })),
        _ if row
            .iter()
            .any(|c| PROMPT.iter().any(|p| c.trim().eq_ignore_ascii_case(p))) =>
        {
            Err("header must contain front and back columns".into())
        }
        _ => Ok(None),
    }
}

/// Parse delimited text (CSV or TSV) into flashcards with RFC 4180 rules.
pub fn parse_delimited(text: &str, preferred: Option<char>) -> Result<Vec<ImportedCard>, String> {
    let rows = parse_rows(text, sniff_delimiter(text, preferred))?;
    if rows.is_empty() {
        return Ok(Vec::new());
    }
    let (m, start_idx) =
        match detect_header(&rows[0].fields).map_err(|e| format!("line {}: {e}", rows[0].line))? {
            Some(m) => (m, 1),
            None => (
                Mapping {
                    front: 0,
                    back: 1,
                    title: None,
                    tags: Some(2),
                },
                0,
            ),
        };
    let mut cards = Vec::with_capacity(rows.len().saturating_sub(start_idx));
    for r in &rows[start_idx..] {
        if r.fields.len() <= m.front || r.fields.len() <= m.back {
            return Err(format!(
                "line {}: a card needs at least a front and a back",
                r.line
            ));
        }
        let (front, back) = (clean(&r.fields[m.front]), clean(&r.fields[m.back]));
        if front.is_empty() {
            return Err(format!(
                "line {}: the front of a card cannot be empty",
                r.line
            ));
        }
        if back.is_empty() {
            return Err(format!(
                "line {}: the back of a card cannot be empty",
                r.line
            ));
        }
        let title = m
            .title
            .and_then(|i| r.fields.get(i))
            .map(|t| clean(t))
            .unwrap_or_default();
        let tags = m
            .tags
            .and_then(|i| r.fields.get(i))
            .map(|t| parse_tags(t))
            .unwrap_or_default();
        cards.push(ImportedCard {
            front,
            back,
            title,
            tags,
        });
    }
    Ok(cards)
}
