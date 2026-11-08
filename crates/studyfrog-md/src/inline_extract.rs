//! Cards and questions written inline in a note: `Term :: Definition` lines (also as list items)
//! give cards, `Q: prompt` followed by `A: answer` gives a card and `??: prompt` followed by
//! `A: answer` a question. Fenced code is skipped; each block remembers the heading it is under.
//! Pure.

/// What an inline block becomes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InlineKind {
    /// A flashcard.
    Card,
    /// A question with a reference answer.
    Question,
}

/// One inline block found in a note.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineBlock {
    /// Card or question.
    pub kind: InlineKind,
    /// The term or prompt.
    pub front: String,
    /// The definition or answer.
    pub back: String,
    /// The nearest heading above it, if any.
    pub heading: Option<String>,
}

fn heading_of(line: &str) -> Option<String> {
    let rest = line.trim_start().strip_prefix('#')?;
    let text = rest.trim_start_matches('#');
    text.starts_with(' ')
        .then(|| text.trim().to_owned())
        .filter(|t| !t.is_empty())
}

fn without_bullet(line: &str) -> &str {
    let t = line.trim();
    ["- ", "* ", "+ "]
        .iter()
        .find_map(|b| t.strip_prefix(b))
        .unwrap_or(t)
        .trim()
}

fn term_line(line: &str) -> Option<(String, String)> {
    let (term, def) = without_bullet(line).split_once(" :: ")?;
    let (term, def) = (term.trim(), def.trim());
    (!term.is_empty() && !def.is_empty()).then(|| (term.to_owned(), def.to_owned()))
}

fn prompt_line(line: &str) -> Option<(InlineKind, String)> {
    let t = without_bullet(line);
    let (kind, rest) = if let Some(r) = t.strip_prefix("??:") {
        (InlineKind::Question, r)
    } else {
        (InlineKind::Card, t.strip_prefix("Q:")?)
    };
    let rest = rest.trim();
    (!rest.is_empty()).then(|| (kind, rest.to_owned()))
}

fn answer_line(line: &str) -> Option<String> {
    let rest = without_bullet(line).strip_prefix("A:")?.trim();
    (!rest.is_empty()).then(|| rest.to_owned())
}

/// Every inline block of a note's Markdown body, in order.
pub fn extract_inline(body: &str) -> Vec<InlineBlock> {
    let mut out = Vec::new();
    let (mut heading, mut fenced) = (None, false);
    let mut prompt: Option<(InlineKind, String)> = None;
    for line in body.lines() {
        if line.trim_start().starts_with("```") || line.trim_start().starts_with("~~~") {
            fenced = !fenced;
            continue;
        }
        if fenced {
            continue;
        }
        if let Some(h) = heading_of(line) {
            heading = Some(h);
            continue;
        }
        let block = |kind, front, back| InlineBlock {
            kind,
            front,
            back,
            heading: heading.clone(),
        };
        if let (Some((kind, front)), Some(back)) = (prompt.clone(), answer_line(line)) {
            out.push(block(kind, front, back));
            prompt = None;
        } else if let Some(p) = prompt_line(line) {
            prompt = Some(p);
        } else if let Some((term, def)) = term_line(line) {
            out.push(block(InlineKind::Card, term, def));
        }
    }
    out
}

/// The wikilink target of a block's place in its note: `Title#Heading`, or the title alone.
pub fn source_target(note_title: &str, heading: Option<&str>) -> String {
    match heading {
        Some(h) => format!("{note_title}#{h}"),
        None => note_title.to_owned(),
    }
}

/// A stable key of a block (its kind and its front with case and spacing evened out), so a later
/// sync finds the item it made: 16 hex digits of FNV-1a. Changing the definition keeps the key.
pub fn extract_key(block: &InlineBlock) -> String {
    let kind = match block.kind {
        InlineKind::Card => "card",
        InlineKind::Question => "question",
    };
    let front: Vec<String> = block
        .front
        .split_whitespace()
        .map(str::to_lowercase)
        .collect();
    let text = format!("{kind}\u{1f}{}", front.join(" "));
    let hash = text.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    });
    format!("{hash:016x}")
}
