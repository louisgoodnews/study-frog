//! Offline questions for a mock exam, made from the notes' own definitions, formulas and text (STORY-0186), so
//! an exam can be made with no model. Every rubric point is taken from the notes, so the keyword check can test
//! it (BUG-0038); a point with no word to match is shown as not checked. Pure.

use crate::curriculum::Chunk;
use crate::curriculum_build::{GeneratedQuestion, Kind, Plan};

/// Longest key point taken from a note, in characters.
const KEY_POINT_CHARS: usize = 160;

/// Offline questions from the notes' own definitions, formulas and text, up to the wanted number of each
/// kind. The answers are the notes' own lines, and the questions say so.
pub fn templates(chunks: &[Chunk], wanted: Plan) -> Vec<GeneratedQuestion> {
    let pool = pool(chunks);
    let mut out: Vec<GeneratedQuestion> = pool.recall.into_iter().take(wanted.recall).collect();
    out.extend(pool.case.into_iter().take(wanted.case));
    out.extend(pool.calc.into_iter().take(wanted.calc));
    out
}

/// How many questions of each kind the notes can supply offline: one recall question per definition, one
/// case per paragraph, one calculation per formula. A part wanted above this is short (BUG-0055).
pub fn material(chunks: &[Chunk]) -> Plan {
    let p = pool(chunks);
    Plan {
        recall: p.recall.len(),
        case: p.case.len(),
        calc: p.calc.len(),
    }
}

/// Every offline question the notes can supply, by kind, in the notes' order.
struct Pool {
    recall: Vec<GeneratedQuestion>,
    case: Vec<GeneratedQuestion>,
    calc: Vec<GeneratedQuestion>,
}

fn pool(chunks: &[Chunk]) -> Pool {
    let mut recall = Vec::new();
    let mut calc = Vec::new();
    let mut case = Vec::new();
    for c in chunks {
        for d in &c.definitions {
            recall.push(GeneratedQuestion {
                kind: Kind::Recall,
                prompt: format!("Explain the following in your own words: {}", term_of(d)),
                sample_answer: d.clone(),
                rubric: vec![meaning_of(d)],
                points: 2,
                topic: c.topic.clone(),
            });
        }
        calc.extend(c.formulas.iter().map(|f| calculation(f, &c.topic)));
        for para in paragraphs(&c.text) {
            let n = case.len() + 1;
            case.push(GeneratedQuestion {
                kind: Kind::Case,
                prompt: format!(
                    "Case {n}: a colleague applies the ideas of {} in a new study. Which points must the study respect?",
                    c.topic
                ),
                sample_answer: para.chars().take(240).collect(),
                rubric: vec![key_point(&para, &c.topic)],
                points: 4,
                topic: c.topic.clone(),
            });
        }
    }
    Pool { recall, case, calc }
}

/// The non-empty paragraphs of a chunk's text.
fn paragraphs(text: &str) -> Vec<String> {
    text.split("\n\n")
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .map(str::to_owned)
        .collect()
}

/// A calculation from a formula line: the quantity it names (the left side) and the expression it gives
/// (the right side). The question asks for the formula without printing it.
fn calculation(formula: &str, topic: &str) -> GeneratedQuestion {
    let parts: Vec<&str> = formula
        .split('=')
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .collect();
    let name = parts.first().copied().unwrap_or(formula);
    let mut rubric = vec![name.to_owned()];
    if let Some(expression) = parts.get(1) {
        rubric.push((*expression).to_owned());
    }
    GeneratedQuestion {
        kind: Kind::Calculation,
        prompt: format!("Give the formula for {name} and explain its symbols."),
        sample_answer: formula.to_owned(),
        rubric,
        points: 4,
        topic: topic.to_owned(),
    }
}

/// The first sentence of a note's text, cut to a length a rubric point can hold; the topic when there is none.
fn key_point(text: &str, topic: &str) -> String {
    text.split(['.', '\n'])
        .map(str::trim)
        .find(|s| !s.is_empty())
        .map_or_else(
            || topic.to_owned(),
            |s| s.chars().take(KEY_POINT_CHARS).collect(),
        )
}

/// The meaning of a definition line: the text after its term, without the final full stop. The whole line
/// when there is no meaning after the term.
fn meaning_of(line: &str) -> String {
    let rest = match line.split_once(':') {
        Some((_, meaning)) => meaning,
        None => line
            .split_once(" ist ")
            .or_else(|| line.split_once(" is "))
            .map_or(line, |(_, meaning)| meaning),
    };
    let meaning = rest.trim().trim_end_matches('.').trim();
    if meaning.is_empty() {
        line.to_owned()
    } else {
        meaning.to_owned()
    }
}

/// The term of a definition line: the text before the colon, or the first words before " ist "/" is ".
fn term_of(line: &str) -> String {
    match line.split_once(':') {
        Some((t, _)) => t.trim().to_owned(),
        None => line
            .split(" ist ")
            .chain(line.split(" is "))
            .next()
            .unwrap_or(line)
            .trim()
            .to_owned(),
    }
}
