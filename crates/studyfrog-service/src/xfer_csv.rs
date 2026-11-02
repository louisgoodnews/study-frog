//! CSV and TSV: `stack,kind,title,front,back,tags` for cards and questions (`docs/specs/interchange.md`).

use studyfrog_core::plan::{ItemKind, item_kind_from_name};

use crate::xfer_model::{Bundle, XItem, item};

const HEADER: [&str; 6] = ["stack", "kind", "title", "front", "back", "tags"];

fn quote(s: &str) -> String {
    if s.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_owned()
    }
}

/// The CSV text of the cards and questions of a bundle.
pub fn to_csv(b: &Bundle) -> String {
    let mut out = format!("{}\n", HEADER.join(","));
    for i in b.items.iter().filter(|i| i.kind != ItemKind::Note) {
        let kind = crate::xfer_json::kind_word(i.kind);
        let tags = i.tags.join(";");
        let row = [i.stack.as_str(), kind, &i.title, &i.text, &i.answer, &tags];
        out.push_str(&row.iter().map(|c| quote(c)).collect::<Vec<_>>().join(","));
        out.push('\n');
    }
    out
}

/// Split text into rows of cells with `delim`, honouring quotes (a doubled quote is a quote; a quoted cell may hold line breaks).
fn rows(text: &str, delim: char) -> Vec<Vec<String>> {
    let (mut out, mut row, mut cell, mut quoted) = (Vec::new(), Vec::new(), String::new(), false);
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match (c, quoted) {
            ('"', true) if chars.peek() == Some(&'"') => {
                cell.push('"');
                chars.next();
            }
            ('"', _) => quoted = !quoted,
            (d, false) if d == delim => row.push(std::mem::take(&mut cell)),
            ('\r', false) => {}
            ('\n', false) => {
                row.push(std::mem::take(&mut cell));
                out.push(std::mem::take(&mut row));
            }
            (c, _) => cell.push(c),
        }
    }
    if !cell.is_empty() || !row.is_empty() {
        row.push(cell);
        out.push(row);
    }
    out.into_iter()
        .filter(|r| r.iter().any(|c| !c.is_empty()))
        .collect()
}

/// Read CSV or TSV (the delimiter is the one the header uses).
pub fn from_csv(source: &str) -> Result<Bundle, String> {
    let first = source.lines().next().unwrap_or_default();
    let delim = if first.contains('\t') && !first.contains(',') {
        '\t'
    } else {
        ','
    };
    let table = rows(source, delim);
    let Some((head, body)) = table.split_first() else {
        return Err("the file is empty".to_owned());
    };
    let col = |name: &str| {
        head.iter()
            .position(|h| h.trim().eq_ignore_ascii_case(name))
    };
    let (Some(front), Some(back)) = (col("front"), col("back")) else {
        return Err("the first row must name the columns, at least front and back".to_owned());
    };
    let (stack, kind, title, tags) = (col("stack"), col("kind"), col("title"), col("tags"));
    let mut b = Bundle::default();
    for (n, r) in body.iter().enumerate() {
        let get = |c: Option<usize>| c.and_then(|i| r.get(i)).map_or("", |s| s.trim());
        let k = if get(kind).is_empty() {
            Some(ItemKind::Card)
        } else {
            item_kind_from_name(get(kind)).filter(|k| *k != ItemKind::Note)
        };
        let Some(k) = k else {
            b.warnings
                .push(format!("row {}: the kind must be card or question", n + 2));
            continue;
        };
        let mut it: XItem = item(
            k,
            get(stack),
            r.get(front).map_or("", String::as_str),
            r.get(back).map_or("", String::as_str),
        );
        it.title = get(title).to_owned();
        it.tags = get(tags)
            .split(';')
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .map(str::to_owned)
            .collect();
        b.items.push(it);
    }
    Ok(b)
}
