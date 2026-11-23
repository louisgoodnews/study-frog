//! Generating a mock exam from course notes (STORY-0186, STORY-0207): topic chunks, optional SLM
//! prompting across selected formats, grounding checks, template fallback, and assembly.

use studyfrog_agent::exam_gen::{
    FormatCounts, chunk_system_for_formats, chunk_user, parse_questions,
};
use studyfrog_rehearsal::curriculum::{Chunk, chunk_markdown};
use studyfrog_rehearsal::curriculum_build::{
    AnswerKey, FormatPlan, FormatShort, GeneratedQuestion, accept, assemble, shortfalls, templates,
};
use studyfrog_rehearsal::exam_format::{
    ALL_EXAM_FORMATS, ExamFormatMix, ExamItemFormat, format_index, legacy_plan, plan_total,
    resolve_format_plan,
};
use studyfrog_rehearsal::mock_exam::MockExam;

use crate::exam_gen_judge::judge;

/// The model as the generator asks it: system instructions and user message in; reply out.
pub type Ask<'a> = dyn FnMut(&str, &str) -> Result<String, String> + 'a;

/// The result: the exam, its reference answers, and what the stages did with each question.
#[derive(Debug, Clone, PartialEq)]
pub struct Generated {
    /// The exam, ready to run.
    pub exam: MockExam,
    /// The reference answer and scoring of each question, in the exam's order.
    pub keys: Vec<AnswerKey>,
    /// Questions the model wrote that the notes support.
    pub from_model: usize,
    /// Questions made from templates (no model, a failed chunk, or too few from the model).
    pub from_templates: usize,
    /// Why questions were rejected or a chunk failed, one line each.
    pub rejected: Vec<String>,
    /// The parts that have fewer questions than the plan asks for, because the notes lack material (BUG-0055).
    pub short: Vec<FormatShort>,
}

/// Make an exam using the legacy 40/30/30 plan across OpenQuestion, CaseScenario, and Calculation.
pub fn generate(
    chunks: &[Chunk],
    total: usize,
    title: &str,
    ask: Option<&mut Ask<'_>>,
) -> Generated {
    generate_with_plan(chunks, legacy_plan(total), title, ask)
}

/// Validate `mix`, resolve exact per-format counts, and generate an exam from `chunks`.
pub fn generate_with_mix(
    chunks: &[Chunk],
    mix: &ExamFormatMix,
    title: &str,
    ask: Option<&mut Ask<'_>>,
) -> Result<Generated, String> {
    let want = resolve_format_plan(mix)?;
    Ok(generate_with_plan(chunks, want, title, ask))
}

/// Generate an exam matching `want` per-format counts from `chunks`.
pub fn generate_with_plan(
    chunks: &[Chunk],
    want: FormatPlan,
    title: &str,
    mut ask: Option<&mut Ask<'_>>,
) -> Generated {
    let _ = accept; // re-checked inside exam_gen_judge::judge
    let mut accepted: Vec<GeneratedQuestion> = Vec::new();
    let mut rejected = Vec::new();
    if let Some(ask) = ask.as_mut() {
        for (chunk, quota) in chunks.iter().zip(per_chunk(want, chunks.len())) {
            if plan_total(&quota) == 0 {
                continue;
            }
            let system = chunk_system_for_formats(to_agent_counts(quota));
            let user = chunk_user(&chunk.topic, &chunk.text);
            match ask(&system, &user).and_then(|reply| parse_questions(&reply)) {
                Ok(raws) => {
                    for raw in raws {
                        judge(raw, chunk, chunks, &mut accepted, &mut rejected);
                    }
                }
                Err(e) => rejected.push(format!("{}: {e}", chunk.topic)),
            }
        }
    }
    let from_model = accepted.len();
    let mut chosen = balance(accepted, want);
    let missing = missing_plan(want, &chosen);
    let fills = templates(chunks, missing);
    let from_templates = fills.len();
    chosen.extend(fills);
    let made = counts_of(&chosen);
    let short = shortfalls(want, made);
    let (exam, keys) = assemble(title, &chosen);
    Generated {
        exam,
        keys,
        from_model,
        from_templates,
        rejected,
        short,
    }
}

fn to_agent_counts(p: FormatPlan) -> FormatCounts {
    FormatCounts {
        open_question: p.counts[format_index(ExamItemFormat::OpenQuestion)],
        fill_in_gaps: p.counts[format_index(ExamItemFormat::FillInGaps)],
        single_select: p.counts[format_index(ExamItemFormat::SingleSelect)],
        multiple_select: p.counts[format_index(ExamItemFormat::MultipleSelect)],
        calculation: p.counts[format_index(ExamItemFormat::Calculation)],
        case_scenario: p.counts[format_index(ExamItemFormat::CaseScenario)],
    }
}

fn per_chunk(want: FormatPlan, n: usize) -> Vec<FormatPlan> {
    if n == 0 {
        return Vec::new();
    }
    let share = |total: usize, i: usize| total / n + usize::from(i < total % n);
    (0..n)
        .map(|i| {
            let counts = std::array::from_fn(|slot| share(want.counts[slot], i));
            FormatPlan { counts }
        })
        .collect()
}

fn balance(qs: Vec<GeneratedQuestion>, want: FormatPlan) -> Vec<GeneratedQuestion> {
    let mut rem = want.counts;
    qs.into_iter()
        .filter(|q| {
            let idx = format_index(q.format);
            if rem[idx] > 0 {
                rem[idx] -= 1;
                true
            } else {
                false
            }
        })
        .collect()
}

fn counts_of(qs: &[GeneratedQuestion]) -> FormatPlan {
    let mut counts = [0; 6];
    for q in qs {
        counts[format_index(q.format)] += 1;
    }
    FormatPlan { counts }
}

fn missing_plan(want: FormatPlan, chosen: &[GeneratedQuestion]) -> FormatPlan {
    let have = counts_of(chosen);
    let mut counts = [0; 6];
    for fmt in ALL_EXAM_FORMATS {
        let i = format_index(fmt);
        counts[i] = want.counts[i].saturating_sub(have.counts[i]);
    }
    FormatPlan { counts }
}

/// The notes of a Markdown file as chunks.
pub fn chunks_of_markdown(md: &str) -> Vec<Chunk> {
    chunk_markdown(md)
}

/// The text of a lecture PDF as chunks.
pub fn chunks_of_pdf(bytes: &[u8]) -> Result<Vec<Chunk>, String> {
    let pages = studyfrog_pdf::text::pdf_all_text(&std::sync::Arc::new(bytes.to_vec()))
        .map_err(|e| studyfrog_pdf::error::describe_pdf_error(&e))?;
    let text = pages
        .iter()
        .map(|p| p.text.as_str())
        .collect::<Vec<_>>()
        .join("\n\n");
    Ok(chunk_markdown(&text))
}
