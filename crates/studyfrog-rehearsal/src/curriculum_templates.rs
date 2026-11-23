//! Offline questions for a mock exam across all 6 item formats (STORY-0186, STORY-0207). Pure.

use crate::curriculum::Chunk;
use crate::curriculum_build::GeneratedQuestion;
use crate::curriculum_choices::{multiple_select_pool, single_select_pool};
use crate::curriculum_gaps::make_gap_question;
use crate::exam_format::{ExamItemFormat, FormatPlan, plan_count, set_plan_count};

const KEY_POINT_CHARS: usize = 160;

/// Offline questions from the notes' definitions, formulas, text, and synthesized gaps/choices.
pub fn templates(chunks: &[Chunk], wanted: FormatPlan) -> Vec<GeneratedQuestion> {
    let p = pool(chunks);
    let mut out = Vec::new();
    out.extend(
        p.open
            .into_iter()
            .take(plan_count(&wanted, ExamItemFormat::OpenQuestion)),
    );
    out.extend(
        p.gaps
            .into_iter()
            .take(plan_count(&wanted, ExamItemFormat::FillInGaps)),
    );
    out.extend(
        p.single
            .into_iter()
            .take(plan_count(&wanted, ExamItemFormat::SingleSelect)),
    );
    out.extend(
        p.multiple
            .into_iter()
            .take(plan_count(&wanted, ExamItemFormat::MultipleSelect)),
    );
    out.extend(
        p.calc
            .into_iter()
            .take(plan_count(&wanted, ExamItemFormat::Calculation)),
    );
    out.extend(
        p.case
            .into_iter()
            .take(plan_count(&wanted, ExamItemFormat::CaseScenario)),
    );
    out
}

/// How many questions of each format the notes can supply offline (BUG-0055).
pub fn material(chunks: &[Chunk]) -> FormatPlan {
    let p = pool(chunks);
    let mut plan = FormatPlan::default();
    set_plan_count(&mut plan, ExamItemFormat::OpenQuestion, p.open.len());
    set_plan_count(&mut plan, ExamItemFormat::FillInGaps, p.gaps.len());
    set_plan_count(&mut plan, ExamItemFormat::SingleSelect, p.single.len());
    set_plan_count(&mut plan, ExamItemFormat::MultipleSelect, p.multiple.len());
    set_plan_count(&mut plan, ExamItemFormat::Calculation, p.calc.len());
    set_plan_count(&mut plan, ExamItemFormat::CaseScenario, p.case.len());
    plan
}

struct Pool {
    open: Vec<GeneratedQuestion>,
    gaps: Vec<GeneratedQuestion>,
    single: Vec<GeneratedQuestion>,
    multiple: Vec<GeneratedQuestion>,
    calc: Vec<GeneratedQuestion>,
    case: Vec<GeneratedQuestion>,
}

fn pool(chunks: &[Chunk]) -> Pool {
    let mut open = Vec::new();
    let mut calc = Vec::new();
    let mut case = Vec::new();
    let mut gaps = Vec::new();
    for c in chunks {
        for d in &c.definitions {
            open.push(GeneratedQuestion {
                format: ExamItemFormat::OpenQuestion,
                prompt: format!("Explain the following in your own words: {}", term_of(d)),
                sample_answer: d.clone(),
                rubric: vec![meaning_of(d)],
                points: 2,
                topic: c.topic.clone(),
                options: Vec::new(),
                correct_indices: Vec::new(),
                gap_hints: Vec::new(),
                gap_answers: Vec::new(),
            });
            if let Some(gq) = make_gap_question(&c.topic, d) {
                gaps.push(gq);
            }
        }
        calc.extend(c.formulas.iter().map(|f| calculation(f, &c.topic)));
        for para in paragraphs(&c.text) {
            let n = case.len() + 1;
            case.push(GeneratedQuestion {
                format: ExamItemFormat::CaseScenario,
                prompt: format!(
                    "Case {n}: a colleague applies the ideas of {} in a new study. Which points must the study respect?",
                    c.topic
                ),
                sample_answer: para.chars().take(240).collect(),
                rubric: vec![key_point(&para, &c.topic)],
                points: 4,
                topic: c.topic.clone(),
                options: Vec::new(),
                correct_indices: Vec::new(),
                gap_hints: Vec::new(),
                gap_answers: Vec::new(),
            });
            if c.definitions.is_empty()
                && let Some(gq) = make_gap_question(&c.topic, &key_point(&para, &c.topic))
            {
                gaps.push(gq);
            }
        }
    }
    Pool {
        open,
        gaps,
        single: single_select_pool(chunks),
        multiple: multiple_select_pool(chunks),
        calc,
        case,
    }
}

fn paragraphs(text: &str) -> Vec<String> {
    text.split("\n\n")
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .map(str::to_owned)
        .collect()
}

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
        format: ExamItemFormat::Calculation,
        prompt: format!("Give the formula for {name} and explain its symbols."),
        sample_answer: formula.to_owned(),
        rubric,
        points: 4,
        topic: topic.to_owned(),
        options: Vec::new(),
        correct_indices: Vec::new(),
        gap_hints: Vec::new(),
        gap_answers: Vec::new(),
    }
}

fn key_point(text: &str, topic: &str) -> String {
    text.split(['.', '\n'])
        .map(str::trim)
        .find(|s| !s.is_empty())
        .map_or_else(
            || topic.to_owned(),
            |s| s.chars().take(KEY_POINT_CHARS).collect(),
        )
}

/// The meaning of a definition line: the text after its term, without the final full stop.
pub fn meaning_of(line: &str) -> String {
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
pub fn term_of(line: &str) -> String {
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
