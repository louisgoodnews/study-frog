//! CSV and TSV: `stack,kind,title,front,back,tags` for cards and questions (`docs/specs/interchange.md`). Writing, and
//! the splitting into cells; reading with a column mapping is `xfer_csv_map`.

use studyfrog_core::plan::ItemKind;

use crate::xfer_model::Bundle;

const HEADER: [&str; 6] = ["stack", "kind", "title", "front", "back", "tags"];

fn quote(s: &str) -> String {
    if s.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_owned()
    }
}

/// The CSV text of the cards and questions of a bundle. The `also_in` column (the other stacks, folders separated
/// by semicolons) is there only when some item has references, so other files keep the six columns.
pub fn to_csv(b: &Bundle) -> String {
    let items: Vec<_> = b
        .items
        .iter()
        .filter(|i| i.kind != ItemKind::Note)
        .collect();
    let refs = items.iter().any(|i| !i.also_in.is_empty());
    let mut head = HEADER.to_vec();
    if refs {
        head.push("also_in");
    }
    let mut out = format!("{}\n", head.join(","));
    for i in items {
        let kind = crate::xfer_json::kind_word(i.kind);
        let (tags, also) = (i.tags.join(";"), i.also_in.join(";"));
        let mut row = vec![i.stack.as_str(), kind, &i.title, &i.text, &i.answer, &tags];
        if refs {
            row.push(&also);
        }
        out.push_str(&row.iter().map(|c| quote(c)).collect::<Vec<_>>().join(","));
        out.push('\n');
    }
    out
}

/// Split text into rows of cells with `delim`, honouring quotes (a doubled quote is a quote; a quoted cell may hold line breaks).
pub(crate) fn rows(text: &str, delim: char) -> Vec<Vec<String>> {
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

/// Read CSV or TSV text with everything detected (see [`crate::xfer_csv_map`] to choose the columns).
pub fn from_csv(source: &str) -> Result<Bundle, String> {
    crate::xfer_csv_map::read_csv(source.as_bytes(), &Default::default()).map(|r| r.bundle)
}
