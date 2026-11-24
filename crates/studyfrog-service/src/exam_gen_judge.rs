//! Format-aware validation and cloze normalization of raw SLM exam questions (STORY-0207).

use studyfrog_agent::exam_gen::RawQuestion;
use studyfrog_md::cloze::cloze_spans;
use studyfrog_rehearsal::curriculum::Chunk;
use studyfrog_rehearsal::curriculum_build::{GeneratedQuestion, accept};
use studyfrog_rehearsal::exam_format::{ExamItemFormat, parse_format_wire_name};

/// Validate and normalize one [`RawQuestion`] from a model reply into a [`GeneratedQuestion`].
pub fn judge(
    raw: RawQuestion,
    chunk: &Chunk,
    chunks: &[Chunk],
    accepted: &mut Vec<GeneratedQuestion>,
    rejected: &mut Vec<String>,
) {
    let Some(format) = parse_format_wire_name(&raw.kind) else {
        rejected.push(format!("{}: unknown kind {:?}", chunk.topic, raw.kind));
        return;
    };
    let q = match format {
        ExamItemFormat::FillInGaps => build_gap_question(raw, &chunk.topic),
        ExamItemFormat::SingleSelect | ExamItemFormat::MultipleSelect => {
            build_select_question(format, raw, &chunk.topic)
        }
        ExamItemFormat::OpenQuestion
        | ExamItemFormat::Calculation
        | ExamItemFormat::CaseScenario => GeneratedQuestion {
            format,
            prompt: raw.prompt,
            sample_answer: raw.sample_answer,
            rubric: raw.rubric,
            points: u8::try_from(raw.points).unwrap_or(0),
            topic: chunk.topic.clone(),
            options: Vec::new(),
            correct_indices: Vec::new(),
            gap_hints: Vec::new(),
            gap_answers: Vec::new(),
        },
    };
    match accept(&q, chunks) {
        Ok(()) => accepted.push(q),
        Err(e) => rejected.push(format!("{}: {e}", chunk.topic)),
    }
}

fn build_gap_question(raw: RawQuestion, topic: &str) -> GeneratedQuestion {
    let spans = cloze_spans(&raw.prompt);
    let (prompt, gap_answers, gap_hints) = if !spans.is_empty() {
        let mut masked = String::with_capacity(raw.prompt.len());
        let mut last = 0usize;
        let mut answers = Vec::with_capacity(spans.len());
        let mut hints = Vec::with_capacity(spans.len());
        for (i, span) in spans.iter().enumerate() {
            masked.push_str(&raw.prompt[last..span.range.0]);
            masked.push_str(&format!("[{}]", i + 1));
            last = span.range.1;
            answers.push(span.answer.trim().to_owned());
            hints.push(span.hint.clone());
        }
        masked.push_str(&raw.prompt[last..]);
        (masked, answers, hints)
    } else {
        let answers: Vec<String> = raw.gaps.into_iter().map(|g| g.trim().to_owned()).collect();
        let hints = vec![None; answers.len()];
        (raw.prompt, answers, hints)
    };
    let sample_answer = if raw.sample_answer.trim().is_empty() {
        gap_answers.join("; ")
    } else {
        raw.sample_answer
    };
    let rubric = if raw.rubric.is_empty() {
        gap_answers.clone()
    } else {
        raw.rubric
    };
    GeneratedQuestion {
        format: ExamItemFormat::FillInGaps,
        prompt,
        sample_answer,
        rubric,
        points: u8::try_from(raw.points).unwrap_or(0),
        topic: topic.to_owned(),
        options: Vec::new(),
        correct_indices: Vec::new(),
        gap_hints,
        gap_answers,
    }
}

fn build_select_question(
    format: ExamItemFormat,
    raw: RawQuestion,
    topic: &str,
) -> GeneratedQuestion {
    let mut correct = raw.correct_indices;
    if correct.is_empty()
        && let Some(idx) = raw.correct_index
    {
        correct.push(idx);
    }
    correct.sort_unstable();
    correct.dedup();
    let correct_texts: Vec<String> = correct
        .iter()
        .filter_map(|&i| raw.options.get(i).cloned())
        .collect();
    let sample_answer = if raw.sample_answer.trim().is_empty() {
        correct_texts.join("; ")
    } else {
        raw.sample_answer
    };
    let rubric = if raw.rubric.is_empty() {
        correct_texts
    } else {
        raw.rubric
    };
    GeneratedQuestion {
        format,
        prompt: raw.prompt,
        sample_answer,
        rubric,
        points: u8::try_from(raw.points).unwrap_or(0),
        topic: topic.to_owned(),
        options: raw.options,
        correct_indices: correct,
        gap_hints: Vec::new(),
        gap_answers: Vec::new(),
    }
}
