//! RFC 4180 delimited text parser with delimiter sniffing and header mapping.

use studyfrog_ext::types::ImportedCard;

struct RawRow {
    line: usize,
    fields: Vec<String>,
}

fn sniff_delimiter(text: &str, preferred: Option<char>) -> char {
    const SEPS: [(&str, char); 6] = [
        ("separator:tab", '\t'),
        ("separator:\\t", '\t'),
        ("separator:comma", ','),
        ("separator:,", ','),
        ("separator:semicolon", ';'),
        ("separator:;", ';'),
    ];
    for line in text.lines().map(str::trim).filter(|l| !l.is_empty()) {
        if let Some(c) = line.strip_prefix('#') {
            let l = c.to_ascii_lowercase();
            if let Some((_, ch)) = SEPS.iter().find(|(p, _)| l.contains(p)) {
                return *ch;
            }
            continue;
        }
        let (mut counts, mut in_q) = ([0usize; 3], false);
        for c in line.chars() {
            if c == '"' {
                in_q = !in_q;
            } else if !in_q {
                for (i, d) in ['\t', ',', ';'].into_iter().enumerate() {
                    counts[i] += usize::from(c == d);
                }
            }
        }
        if let Some((_, ch)) = counts
            .into_iter()
            .zip(['\t', ',', ';'])
            .find(|(n, _)| *n > 0)
        {
            return ch;
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
        Vec::new()
    } else if raw.contains(';') {
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

    let flush = |rows: &mut Vec<RawRow>, row: &mut Vec<String>, field: &mut String, quoted, sl| {
        if !row.is_empty() || !field.trim().is_empty() || quoted {
            row.push(std::mem::take(field));
            rows.push(RawRow {
                line: sl,
                fields: std::mem::take(row),
            });
        }
        field.clear();
    };
    while let Some(c) = chars.next() {
        let is_nl = c == '\n'
            || (c == '\r' && {
                chars.next_if_eq(&'\n');
                true
            });
        if in_comment {
            if is_nl {
                (line, in_comment, at_start, start_line) = (line + 1, false, true, line + 1);
            }
        } else if !in_q && at_start && c == '#' {
            in_comment = true;
        } else if in_q {
            if c == '"' && chars.next_if_eq(&'"').is_some() {
                field.push('"');
            } else if c == '"' {
                in_q = false;
            } else if is_nl {
                line += 1;
                field.push('\n');
            } else {
                field.push(c);
            }
        } else if c == '"' && field.trim().is_empty() && !quoted {
            (in_q, quoted) = (true, true);
            field.clear();
        } else if c == delimiter {
            row.push(std::mem::take(&mut field));
            (quoted, at_start) = (false, false);
        } else if is_nl {
            line += 1;
            flush(&mut rows, &mut row, &mut field, quoted, start_line);
            (at_start, quoted, start_line) = (true, false, line);
        } else {
            field.push(c);
            at_start = false;
        }
    }
    if in_q {
        return Err(format!("line {start_line}: unclosed quote"));
    }
    flush(&mut rows, &mut row, &mut field, quoted, start_line);
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

fn normalize_row(fields: &[String], m: &Mapping, cols: usize, delim: char) -> Vec<String> {
    if delim != ',' || cols == 0 || fields.len() <= cols || m.back != m.front + 1 || m.back >= cols
    {
        return fields.to_vec();
    }
    let trail = cols - (m.back + 1);
    let mid = &fields[m.front..fields.len() - trail];
    let split = mid[..mid.len() - 1]
        .iter()
        .position(|s| !s.contains('\n') && s.trim_end().ends_with('?'))
        .unwrap_or(0);
    let mut out = fields[..m.front].to_vec();
    out.push(mid[..=split].join(","));
    out.push(mid[split + 1..].join(","));
    out.extend_from_slice(&fields[fields.len() - trail..]);
    out
}

fn merge_rows(rows: &[RawRow], m: &Mapping, cols: usize, delim: char) -> Vec<RawRow> {
    let mut out: Vec<RawRow> = Vec::with_capacity(rows.len());
    for r in rows {
        if delim == ','
            && cols >= 3
            && let Some(prev) = out.last_mut()
            && (prev.fields.len() < cols || (m.front > 0 && r.fields.len() <= m.front))
            && let Some(first) = r.fields.first()
        {
            if let Some(last) = prev.fields.last_mut() {
                last.push('\n');
                last.push_str(first);
            }
            prev.fields.extend_from_slice(&r.fields[1..]);
        } else {
            out.push(RawRow {
                line: r.line,
                fields: r.fields.clone(),
            });
        }
    }
    out
}

/// Parse delimited text (CSV or TSV) into flashcards with RFC 4180 rules.
pub fn parse_delimited(text: &str, preferred: Option<char>) -> Result<Vec<ImportedCard>, String> {
    let delim = sniff_delimiter(text, preferred);
    let rows = parse_rows(text, delim)?;
    let Some(first) = rows.first() else {
        return Ok(Vec::new());
    };
    let head = detect_header(&first.fields).map_err(|e| format!("line {}: {e}", first.line))?;
    let (start_idx, cols) = head.as_ref().map_or((0, 0), |_| (1, first.fields.len()));
    let m = head.unwrap_or(Mapping {
        front: 0,
        back: 1,
        title: None,
        tags: Some(2),
    });
    merge_rows(&rows[start_idx..], &m, cols, delim)
        .iter()
        .map(|r| {
            let f = normalize_row(&r.fields, &m, cols, delim);
            if f.len() <= m.front.max(m.back) {
                return Err(format!(
                    "line {}: a card needs at least a front and a back",
                    r.line
                ));
            }
            let (front, back) = (clean(&f[m.front]), clean(&f[m.back]));
            for (val, side) in [(&front, "front"), (&back, "back")] {
                if val.is_empty() {
                    return Err(format!(
                        "line {}: the {side} of a card cannot be empty",
                        r.line
                    ));
                }
            }
            let get = |idx: Option<usize>| idx.and_then(|i| f.get(i)).map(String::as_str);
            Ok(ImportedCard {
                front,
                back,
                title: get(m.title).map_or_else(String::new, clean),
                tags: get(m.tags).map_or_else(Vec::new, parse_tags),
            })
        })
        .collect()
}
