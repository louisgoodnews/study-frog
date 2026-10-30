//! Tab-separated text into cards.

use studyfrog_ext::types::{ImportedCard, Importer};

/// The importer record.
pub fn importer() -> Importer {
    Importer {
        id: "tab-separated",
        name: "Tab-separated text (front, back, tags)",
        extensions: &["tsv", "txt"],
        import: import_tsv,
    }
}

fn clean(field: &str) -> String {
    field
        .replace("<br>", "\n")
        .replace("<br/>", "\n")
        .replace("<br />", "\n")
        .trim()
        .to_owned()
}

/// One card per line. Lines starting with `#` (such as `#separator:tab`) and empty lines are
/// skipped; a line needs at least a front and a back; a third column holds space-separated tags.
pub fn import_tsv(text: &str) -> Result<Vec<ImportedCard>, String> {
    let mut cards = Vec::new();
    for (n, line) in text.lines().enumerate() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        let (front, back) = match fields.as_slice() {
            [f, b, ..] if !clean(f).is_empty() && !clean(b).is_empty() => (clean(f), clean(b)),
            _ => {
                return Err(format!(
                    "line {}: a card needs a front and a back separated by a tab",
                    n + 1
                ));
            }
        };
        let tags = fields.get(2).map_or_else(Vec::new, |t| {
            t.split_whitespace().map(str::to_owned).collect()
        });
        cards.push(ImportedCard {
            front,
            back,
            title: String::new(),
            tags,
        });
    }
    Ok(cards)
}
