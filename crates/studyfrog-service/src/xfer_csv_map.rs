//! Reading CSV and TSV with a column mapping (TASK-0208): the delimiter, the encoding, a header or none and the
//! meaning of each column are detected and can be set. Every problem of the file is collected, with its row and
//! column, so the user (or an AI) can fix them all at once.

use studyfrog_core::plan::{ItemKind, item_kind_from_name};

use crate::xfer_csv::rows;
use crate::xfer_model::{Bundle, XItem, item};

pub use crate::xfer_csv_types::{
    CsvOptions, CsvRead, Mapping, Problem, Sheet, ai_message, problem_text,
};

/// How many rows the dialog shows.
pub const SHOWN_ROWS: usize = 5;
const DELIMITERS: [char; 4] = [',', ';', '\t', '|'];
const FRONT: [&str; 6] = ["front", "question", "term", "prompt", "q", "word"];
const BACK: [&str; 6] = [
    "back",
    "answer",
    "definition",
    "a",
    "meaning",
    "translation",
];
const TITLE: [&str; 2] = ["title", "name"];
const TAGS: [&str; 2] = ["tags", "tag"];
const STACK: [&str; 4] = ["stack", "deck", "folder", "category"];
const ALSO_IN: [&str; 4] = ["also_in", "also in", "alsoin", "references"];
const KIND: [&str; 2] = ["kind", "type"];

/// The text of the bytes: UTF-8 (a leading mark removed), or Latin-1 when forced or when the bytes are not UTF-8.
pub fn decode(bytes: &[u8], latin1: Option<bool>) -> (String, bool) {
    let bytes = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);
    match (latin1, std::str::from_utf8(bytes)) {
        (Some(false) | None, Ok(s)) => (s.to_owned(), false),
        _ => (bytes.iter().map(|b| char::from(*b)).collect(), true),
    }
}

fn detect_delimiter(text: &str) -> char {
    let first = text.lines().next().unwrap_or_default();
    let count = |d: char| first.matches(d).count();
    DELIMITERS
        .iter()
        .copied()
        .max_by_key(|d| (count(*d), *d == ','))
        .filter(|d| count(*d) > 0)
        .unwrap_or(',')
}

fn named(cell: &str, names: &[&str]) -> bool {
    names.iter().any(|n| cell.trim().eq_ignore_ascii_case(n))
}

fn find(head: &[String], names: &[&str]) -> Option<usize> {
    head.iter().position(|h| named(h, names))
}

fn looks_like_header(first: &[String]) -> bool {
    first.iter().any(|c| {
        [&FRONT[..], &BACK, &TITLE, &TAGS, &STACK, &KIND, &ALSO_IN]
            .iter()
            .any(|names| named(c, names))
    })
}

fn guess_map(head: &[String], header: bool) -> Option<Mapping> {
    let by_name = |names: &[&str]| header.then(|| find(head, names)).flatten();
    let (stack, kind) = (by_name(&STACK), by_name(&KIND));
    let (title, tags) = (by_name(&TITLE), by_name(&TAGS));
    let also_in = by_name(&ALSO_IN);
    let taken: Vec<usize> = [stack, kind, title, tags, also_in]
        .into_iter()
        .flatten()
        .collect();
    let mut free = (0..head.len()).filter(|i| !taken.contains(i));
    let front = by_name(&FRONT).or_else(|| free.next())?;
    let back = by_name(&BACK).or_else(|| free.find(|i| *i != front))?;
    Some(Mapping {
        front,
        back,
        title,
        tags,
        stack,
        kind,
        also_in,
    })
}

fn cell(row: &[String], at: Option<usize>) -> &str {
    at.and_then(|i| row.get(i)).map_or("", |s| s.trim())
}

fn clean_cell(row: &[String], at: Option<usize>) -> String {
    let mut s = crate::content::unescape_newlines(cell(row, at));
    for br in ["<br>", "<br/>", "<br />", "<BR>", "<BR/>", "<BR />"] {
        s = s.replace(br, "\n");
    }
    s.trim().to_owned()
}

fn name_of(names: &[String], at: usize) -> String {
    names
        .get(at)
        .cloned()
        .unwrap_or_else(|| format!("Column {}", at + 1))
}

fn row_item(
    row: &[String],
    m: &Mapping,
    n: usize,
    names: &[String],
) -> Result<XItem, Vec<Problem>> {
    let mut bad = Vec::new();
    let mut want = |at: usize, expected: &str, found: &str| {
        bad.push(Problem {
            row: n,
            column: name_of(names, at),
            expected: expected.to_owned(),
            found: found.to_owned(),
        });
    };
    let kind_text = cell(row, m.kind);
    let kind = if kind_text.is_empty() {
        Some(ItemKind::Card)
    } else {
        item_kind_from_name(kind_text)
    };
    if kind.is_none() {
        want(m.kind.unwrap_or(0), "card, question or note", kind_text);
    }
    let kind = kind.unwrap_or(ItemKind::Card);
    let (front, back) = (
        clean_cell(row, Some(m.front)),
        clean_cell(row, Some(m.back)),
    );
    if front.is_empty() {
        want(m.front, "the text of the front", "");
    }
    if back.is_empty() && kind != ItemKind::Note {
        want(m.back, "the text of the back", "");
    }
    if !bad.is_empty() {
        return Err(bad);
    }
    let mut it = item(kind, cell(row, m.stack), &front, &back);
    it.title = cell(row, m.title).to_owned();
    it.tags = cell(row, m.tags)
        .split(';')
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(str::to_owned)
        .collect();
    it.also_in = cell(row, m.also_in)
        .split(';')
        .map(str::trim)
        .filter(|f| !f.is_empty())
        .map(str::to_owned)
        .collect();
    Ok(it)
}

fn normalize_row(row: &[String], m: &Mapping, cols: usize, delim: char) -> Vec<String> {
    if delim != ',' || cols == 0 || row.len() <= cols || m.back != m.front + 1 || m.back >= cols {
        return row.to_vec();
    }
    let trail = cols - (m.back + 1);
    let mid = &row[m.front..row.len() - trail];
    let split = mid[..mid.len() - 1]
        .iter()
        .position(|s| !s.contains('\n') && s.trim_end().ends_with('?'))
        .unwrap_or(0);
    let mut out = row[..m.front].to_vec();
    out.push(mid[..=split].join(","));
    out.push(mid[split + 1..].join(","));
    out.extend_from_slice(&row[row.len() - trail..]);
    out
}

fn merge_rows(rows: &[Vec<String>], m: &Mapping, cols: usize, delim: char) -> Vec<Vec<String>> {
    let mut out: Vec<Vec<String>> = Vec::with_capacity(rows.len());
    for r in rows {
        if delim == ','
            && cols >= 3
            && let Some(prev) = out.last_mut()
            && (prev.len() < cols || (m.front > 0 && r.len() <= m.front))
            && let Some(first) = r.first()
        {
            if let Some(last) = prev.last_mut() {
                last.push('\n');
                last.push_str(first);
            }
            prev.extend_from_slice(&r[1..]);
        } else {
            out.push(r.clone());
        }
    }
    out
}

/// Read a CSV or TSV file with the options; whatever is not chosen is detected. `Err` only when no reading is
/// possible at all (an empty file, or fewer than two columns).
pub fn read_csv(bytes: &[u8], o: &CsvOptions) -> Result<CsvRead, String> {
    let (text, latin1) = decode(bytes, o.latin1);
    let delimiter = o.delimiter.unwrap_or_else(|| detect_delimiter(&text));
    let table = rows(&text, delimiter);
    let Some(first) = table.first() else {
        return Err("the file is empty".to_owned());
    };
    let header = o.header.unwrap_or_else(|| looks_like_header(first));
    let names: Vec<String> = if header {
        first.iter().map(|c| c.trim().to_owned()).collect()
    } else {
        (1..=first.len()).map(|i| format!("Column {i}")).collect()
    };
    let map = o.map.or_else(|| guess_map(first, header)).ok_or_else(|| {
        "it needs at least two columns: the front and the back of each card".to_owned()
    })?;
    let cols = if header { names.len() } else { 0 };
    let body: Vec<Vec<String>> = merge_rows(&table[usize::from(header)..], &map, cols, delimiter)
        .iter()
        .map(|r| normalize_row(r, &map, cols, delimiter))
        .collect();
    let mut bundle = Bundle::default();
    let mut problems = Vec::new();
    for (i, row) in body.iter().enumerate() {
        match row_item(row, &map, i + 1 + usize::from(header), &names) {
            Ok(it) => bundle.items.push(it),
            Err(p) => problems.extend(p),
        }
    }
    bundle.warnings = problems.iter().map(problem_text).collect();
    let first_rows = body.into_iter().take(SHOWN_ROWS).collect();
    let sheet = Sheet {
        delimiter,
        latin1,
        header,
        names,
        first_rows,
        map,
    };
    Ok(CsvRead {
        bundle,
        sheet,
        problems,
    })
}
