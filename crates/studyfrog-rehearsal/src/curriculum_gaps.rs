//! Offline fill-in-the-gaps question synthesis from curriculum chunks (STORY-0207). Pure.

use crate::curriculum_build::GeneratedQuestion;
use crate::exam_format::ExamItemFormat;

/// Build one fill-in-the-gaps question by masking 1-2 key words from `source`.
pub fn make_gap_question(topic: &str, source: &str) -> Option<GeneratedQuestion> {
    let clean = source.trim().replace('\n', " ");
    let words: Vec<&str> = clean.split_whitespace().collect();
    let candidates: Vec<usize> = words
        .iter()
        .enumerate()
        .filter_map(|(i, w)| {
            let stripped = w.trim_matches(|c: char| !c.is_alphanumeric());
            (stripped.chars().count() >= 5 && !stripped.chars().all(char::is_numeric)).then_some(i)
        })
        .collect();
    if candidates.is_empty() {
        return None;
    }
    let chosen: Vec<usize> = if candidates.len() >= 2 {
        vec![candidates[0], candidates[candidates.len() - 1]]
    } else {
        vec![candidates[0]]
    };
    let mut gap_answers = Vec::with_capacity(chosen.len());
    let mut masked_words = Vec::with_capacity(words.len());
    for (i, w) in words.iter().enumerate() {
        if let Some(pos) = chosen.iter().position(|&c| c == i) {
            let clean_w = w.trim_matches(|c: char| !c.is_alphanumeric());
            let suffix: String = w
                .chars()
                .rev()
                .take_while(|c| !c.is_alphanumeric())
                .collect::<String>()
                .chars()
                .rev()
                .collect();
            gap_answers.push(clean_w.to_owned());
            masked_words.push(format!("[{}]{suffix}", pos + 1));
        } else {
            masked_words.push((*w).to_owned());
        }
    }
    let prompt: String = format!("Fill in the missing term(s): {}", masked_words.join(" "))
        .chars()
        .take(380)
        .collect();
    if prompt.chars().count() < 10 {
        return None;
    }
    let hints = vec![None; gap_answers.len()];
    Some(GeneratedQuestion {
        format: ExamItemFormat::FillInGaps,
        prompt,
        sample_answer: clean,
        rubric: gap_answers.clone(),
        points: (gap_answers.len() * 2).min(4) as u8,
        topic: topic.to_owned(),
        options: Vec::new(),
        correct_indices: Vec::new(),
        gap_hints: hints,
        gap_answers,
    })
}

/// Extract numeric `[k]` blank placeholders in left-to-right order.
fn gap_markers(prompt: &str) -> Vec<usize> {
    let mut out = Vec::new();
    let mut rest = prompt;
    while let Some(open) = rest.find('[') {
        rest = &rest[open + 1..];
        let Some(close) = rest.find(']') else {
            break;
        };
        let inside = &rest[..close];
        if !inside.is_empty() && inside.bytes().all(|b| b.is_ascii_digit()) {
            out.push(inside.parse::<usize>().unwrap_or(usize::MAX));
            rest = &rest[close + 1..];
        } else if let Some(inner) = inside.find('[') {
            rest = &rest[inner..];
        } else {
            rest = &rest[close + 1..];
        }
    }
    out
}

/// Whether `prompt` contains `[1]`..`[expected]` in order with no extra or missing numbered blanks.
pub(crate) fn valid_gap_markers(prompt: &str, expected: usize) -> bool {
    expected > 0 && gap_markers(prompt).into_iter().eq(1..=expected)
}
