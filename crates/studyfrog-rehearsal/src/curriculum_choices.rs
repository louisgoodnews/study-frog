//! Offline single-select and multiple-select question synthesizers from curriculum chunks (STORY-0207). Pure.

use std::collections::HashSet;

use crate::curriculum::Chunk;
use crate::curriculum_build::GeneratedQuestion;
use crate::curriculum_templates::{meaning_of, term_of};
use crate::exam_format::ExamItemFormat;

#[derive(Clone)]
struct Statement {
    topic: String,
    term: Option<String>,
    text: String,
}

fn collect_statements(chunks: &[Chunk]) -> Vec<Statement> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for c in chunks {
        for d in &c.definitions {
            let m = meaning_of(d);
            let norm = m.trim().to_lowercase();
            if !norm.is_empty() && seen.insert(norm) {
                out.push(Statement {
                    topic: c.topic.clone(),
                    term: Some(term_of(d)),
                    text: m,
                });
            }
        }
        for s in c.text.split(['.', '\n']) {
            let t = s.trim();
            let len = t.chars().count();
            if (20..=160).contains(&len) && !t.contains('=') {
                let norm = t.to_lowercase();
                if seen.insert(norm) {
                    out.push(Statement {
                        topic: c.topic.clone(),
                        term: None,
                        text: t.to_owned(),
                    });
                }
            }
        }
    }
    out
}

fn clip_prompt(s: &str) -> String {
    s.chars().take(380).collect()
}

/// Build deterministic offline single-select items from chunk definitions and statements.
pub fn single_select_pool(chunks: &[Chunk]) -> Vec<GeneratedQuestion> {
    let stmts = collect_statements(chunks);
    if stmts.len() < 2 {
        return Vec::new();
    }
    let mut out = Vec::new();
    for (idx, target) in stmts.iter().enumerate() {
        let mut distractors: Vec<String> = stmts
            .iter()
            .enumerate()
            .filter(|(j, s)| *j != idx && !s.text.eq_ignore_ascii_case(&target.text))
            .take(3)
            .map(|(_, s)| s.text.clone())
            .collect();
        if distractors.is_empty() {
            continue;
        }
        let total_opts = distractors.len() + 1;
        let correct_pos = (idx * 3 + 1) % total_opts;
        let mut options = Vec::with_capacity(total_opts);
        options.append(&mut distractors);
        options.insert(correct_pos, target.text.clone());
        let prompt = match &target.term {
            Some(term) => clip_prompt(&format!("Which statement accurately describes {term}?")),
            None => clip_prompt(&format!(
                "Which statement is directly supported under {}?",
                target.topic
            )),
        };
        out.push(GeneratedQuestion {
            format: ExamItemFormat::SingleSelect,
            prompt,
            sample_answer: target.text.clone(),
            rubric: vec![target.text.clone()],
            points: 2,
            topic: target.topic.clone(),
            options,
            correct_indices: vec![correct_pos],
            gap_hints: Vec::new(),
            gap_answers: Vec::new(),
        });
    }
    out
}

/// Build deterministic offline multiple-select items from chunk definitions and statements.
pub fn multiple_select_pool(chunks: &[Chunk]) -> Vec<GeneratedQuestion> {
    let stmts = collect_statements(chunks);
    if stmts.len() < 2 {
        return Vec::new();
    }
    let mut out = Vec::new();
    let multi_topic = chunks
        .iter()
        .filter(|c| stmts.iter().any(|s| s.topic == c.topic))
        .count()
        >= 2;
    if multi_topic {
        for (c_idx, chunk) in chunks.iter().enumerate() {
            let positives: Vec<&Statement> = stmts
                .iter()
                .filter(|s| s.topic == chunk.topic)
                .take(2)
                .collect();
            let distractors: Vec<&Statement> = stmts
                .iter()
                .filter(|s| s.topic != chunk.topic)
                .take(2)
                .collect();
            if positives.is_empty() || distractors.is_empty() {
                continue;
            }
            out.push(build_multi_item(
                &chunk.topic,
                &format!("Select all statements that apply to {}.", chunk.topic),
                &positives,
                &distractors,
                c_idx,
            ));
        }
        if !out.is_empty() {
            return out;
        }
    }
    for (idx, target) in stmts.iter().enumerate() {
        let Some(term) = &target.term else { continue };
        let distractors: Vec<&Statement> = stmts
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != idx)
            .map(|(_, s)| s)
            .take(3)
            .collect();
        if distractors.is_empty() {
            continue;
        }
        out.push(build_multi_item(
            &target.topic,
            &format!("Select all statements that apply to {term}."),
            &[target],
            &distractors,
            idx,
        ));
    }
    out
}

fn build_multi_item(
    topic: &str,
    prompt: &str,
    positives: &[&Statement],
    distractors: &[&Statement],
    seed: usize,
) -> GeneratedQuestion {
    let mut tagged: Vec<(bool, String)> = positives
        .iter()
        .map(|s| (true, s.text.clone()))
        .chain(distractors.iter().map(|s| (false, s.text.clone())))
        .collect();
    let len = tagged.len();
    if len > 1 {
        tagged.rotate_left((seed + 1) % len);
    }
    let mut options = Vec::with_capacity(len);
    let mut correct_indices = Vec::new();
    let mut correct_texts = Vec::new();
    for (i, (is_pos, text)) in tagged.into_iter().enumerate() {
        if is_pos {
            correct_indices.push(i);
            correct_texts.push(text.clone());
        }
        options.push(text);
    }
    let sample = correct_texts.join("; ");
    GeneratedQuestion {
        format: ExamItemFormat::MultipleSelect,
        prompt: clip_prompt(prompt),
        sample_answer: sample,
        rubric: correct_texts,
        points: 3,
        topic: topic.to_owned(),
        options,
        correct_indices,
        gap_hints: Vec::new(),
        gap_answers: Vec::new(),
    }
}
