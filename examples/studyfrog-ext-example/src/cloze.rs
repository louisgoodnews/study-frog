//! Cloze deletions: one card per numbered gap.

use studyfrog_ext::types::{CardType, Faces};

/// The card type record.
pub fn card_type() -> CardType {
    CardType {
        id: "cloze",
        name: "Cloze (fill in the gaps)",
        hint: "Write a sentence and mark gaps like {{c1::answer}} or {{c1::answer::hint}}.",
        expand: expand_cloze,
    }
}

/// A piece of the source: plain text, or a gap with its number, answer and optional hint.
enum Piece<'a> {
    Text(&'a str),
    Gap {
        n: u32,
        answer: &'a str,
        hint: Option<&'a str>,
    },
}

fn parse_gap(inner: &str) -> Result<Piece<'_>, String> {
    let rest = inner
        .strip_prefix('c')
        .ok_or("a gap starts with c and a number, like {{c1::answer}}")?;
    let (num, body) = rest
        .split_once("::")
        .ok_or("a gap needs :: after its number")?;
    let n: u32 = num
        .parse()
        .map_err(|_| format!("\"c{num}\" is not a gap number"))?;
    if n == 0 {
        return Err("gap numbers start at 1".to_owned());
    }
    let (answer, hint) = match body.split_once("::") {
        Some((a, h)) => (a, Some(h)),
        None => (body, None),
    };
    if answer.trim().is_empty() {
        return Err(format!("gap c{n} has no answer"));
    }
    Ok(Piece::Gap { n, answer, hint })
}

fn pieces(source: &str) -> Result<Vec<Piece<'_>>, String> {
    let (mut out, mut rest) = (Vec::new(), source);
    while let Some(start) = rest.find("{{") {
        out.push(Piece::Text(&rest[..start]));
        let after = &rest[start + 2..];
        let end = after
            .find("}}")
            .ok_or("a gap is opened with {{ but never closed with }}")?;
        out.push(parse_gap(&after[..end])?);
        rest = &after[end + 2..];
    }
    out.push(Piece::Text(rest));
    Ok(out)
}

fn render(pieces: &[Piece<'_>], active: u32, reveal: bool) -> String {
    let mut s = String::new();
    for p in pieces {
        match p {
            Piece::Text(t) => s.push_str(t),
            Piece::Gap { n, answer, .. } if *n != active => s.push_str(answer),
            Piece::Gap { answer, .. } if reveal => s.push_str(&format!("**{answer}**")),
            Piece::Gap { hint, .. } => s.push_str(&format!("[{}]", hint.unwrap_or("..."))),
        }
    }
    s
}

/// One card per distinct gap number, in ascending order. The front hides that gap (showing its hint,
/// or `...`) and shows the others; the back shows the gap's answer in bold.
pub fn expand_cloze(source: &str) -> Result<Vec<Faces>, String> {
    let pieces = pieces(source.trim())?;
    let mut numbers: Vec<u32> = pieces
        .iter()
        .filter_map(|p| {
            if let Piece::Gap { n, .. } = p {
                Some(*n)
            } else {
                None
            }
        })
        .collect();
    numbers.sort_unstable();
    numbers.dedup();
    if numbers.is_empty() {
        return Err("no gaps found: mark one like {{c1::answer}}".to_owned());
    }
    Ok(numbers
        .into_iter()
        .map(|n| Faces {
            front: render(&pieces, n, false),
            back: render(&pieces, n, true),
        })
        .collect())
}
