//! Curriculum chunks for exam generation (STORY-0186): the notes cut into topics, each topic into
//! pieces the model can read at once, with the definitions and formulas found in them. Pure.

/// Longest piece of text sent to the model at once, in characters.
pub const MAX_CHUNK_CHARS: usize = 3_000;
/// Topic used for text before the first heading (and for plain text without headings).
pub const GENERAL: &str = "General";

/// A piece of the course notes, with its topic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chunk {
    /// The heading the piece is under.
    pub topic: String,
    /// The text of the piece.
    pub text: String,
    /// Definitions found in it, one line each ("Term: meaning" or "X ist ...").
    pub definitions: Vec<String>,
    /// Formulas found in it, one line each.
    pub formulas: Vec<String>,
}

/// Cut Markdown notes into chunks: one or more per heading, each at most [`MAX_CHUNK_CHARS`] long,
/// split at blank lines. Headings inside code fences are not headings.
pub fn chunk_markdown(md: &str) -> Vec<Chunk> {
    let mut sections: Vec<(String, Vec<String>)> = vec![(GENERAL.to_owned(), Vec::new())];
    let mut in_fence = false;
    for line in md.lines() {
        let t = line.trim_start();
        if t.starts_with("```") {
            in_fence = !in_fence;
        }
        if !in_fence && t.starts_with('#') {
            let topic = t.trim_start_matches('#').trim().to_owned();
            if !topic.is_empty() {
                sections.push((topic, Vec::new()));
                continue;
            }
        }
        if let Some((_, lines)) = sections.last_mut() {
            lines.push(line.to_owned());
        }
    }
    sections
        .into_iter()
        .filter(|(_, lines)| lines.iter().any(|l| !l.trim().is_empty()))
        .flat_map(|(topic, lines)| {
            pieces(&lines.join("\n"))
                .into_iter()
                .map(move |text| with_terms(&topic, text))
        })
        .collect()
}

/// Split text at blank lines into pieces of at most [`MAX_CHUNK_CHARS`] characters (a single paragraph
/// longer than that is cut at the limit).
fn pieces(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for para in text.split("\n\n").map(str::trim).filter(|p| !p.is_empty()) {
        let para: String = para.chars().take(MAX_CHUNK_CHARS).collect();
        if !cur.is_empty() && cur.chars().count() + para.chars().count() + 2 > MAX_CHUNK_CHARS {
            out.push(std::mem::take(&mut cur));
        }
        if !cur.is_empty() {
            cur.push_str("\n\n");
        }
        cur.push_str(&para);
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn with_terms(topic: &str, text: String) -> Chunk {
    let (definitions, formulas) = terms(&text);
    Chunk {
        topic: topic.to_owned(),
        text,
        definitions,
        formulas,
    }
}

/// The definitions and formulas in a text: a definition is a line "Term: meaning" with a short term, or
/// a line with " ist " or " is " after a short term; a formula is a line with "=" and a digit or a `$`.
pub fn terms(text: &str) -> (Vec<String>, Vec<String>) {
    let mut defs = Vec::new();
    let mut formulas = Vec::new();
    for line in text.lines().map(str::trim).filter(|l| !l.is_empty()) {
        if line.contains('=') && (line.contains('$') || line.chars().any(|c| c.is_ascii_digit())) {
            formulas.push(line.trim_start_matches(['-', '*', ' ']).to_owned());
        } else if is_definition(line) {
            defs.push(line.trim_start_matches(['-', '*', ' ']).to_owned());
        }
    }
    (defs, formulas)
}

fn is_definition(line: &str) -> bool {
    if line.chars().count() < 25 || line.chars().count() > 300 {
        return false;
    }
    if let Some((term, meaning)) = line.split_once(':') {
        let term = term.trim_start_matches(['-', '*', ' ']);
        return term.chars().count() <= 60 && !term.is_empty() && meaning.trim().len() >= 10;
    }
    [" ist ", " is "].iter().any(|verb| {
        line.split_once(verb).is_some_and(|(term, _)| {
            term.split_whitespace().count() <= 4 && !term.trim().is_empty()
        })
    })
}

/// Significant words of a text: lowercase letters and digits, four characters or more, not a common
/// word.
pub fn significant_words(text: &str) -> Vec<String> {
    const COMMON: [&str; 12] = [
        "that", "this", "with", "from", "they", "their", "which", "have", "were", "will", "into",
        "about",
    ];
    text.split(|c: char| !c.is_alphanumeric())
        .map(str::to_lowercase)
        .filter(|w| w.chars().count() >= 4 && !COMMON.contains(&w.as_str()))
        .collect()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn headings_make_the_topics_and_code_fences_do_not() {
        let md = "# Gütekriterien\nObjektivität is the independence of results from the tester, and more.\n\n## Reliabilität\nReliabilität: consistency of measurement over time.\n```\n# not a heading\n```\n";
        let chunks = chunk_markdown(md);
        let topics: Vec<&str> = chunks.iter().map(|c| c.topic.as_str()).collect();
        assert_eq!(topics, ["Gütekriterien", "Reliabilität"]);
        assert!(
            chunks[1]
                .definitions
                .iter()
                .any(|d| d.starts_with("Reliabilität: consistency"))
        );
    }

    #[test]
    fn formulas_are_found() {
        let (_, f) = terms("s_e = s_x * sqrt(1 - r_xx)\nplain sentence here");
        assert_eq!(f, ["s_e = s_x * sqrt(1 - r_xx)"]);
    }

    #[test]
    fn long_topics_are_cut_into_pieces_under_the_limit() {
        let para = "word ".repeat(200);
        let md = format!("# Long\n{para}\n\n{para}\n\n{para}\n");
        let chunks = chunk_markdown(&md);
        assert!(chunks.len() >= 2);
        assert!(
            chunks
                .iter()
                .all(|c| c.text.chars().count() <= MAX_CHUNK_CHARS)
        );
    }
}
