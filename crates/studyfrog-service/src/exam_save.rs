//! Reading a folder's notes for exam generation, rendering a generated exam, and saving it as a note
//! (STORY-0186). The generation itself is in [`crate::exam_gen`].

use studyfrog_core::plan::ItemKind;
use studyfrog_rehearsal::curriculum::{Chunk, chunk_markdown};
use studyfrog_rehearsal::curriculum_build::AnswerKey;
use studyfrog_rehearsal::mock_exam::{MockExam, total_points};
use studyfrog_store::overview::list_items;

use crate::content::load_item_content;
use crate::create::fresh_id_source;
use crate::create_more::{NewItem, NewKind, create_item};
use crate::exam_gen::chunks_of_pdf;
use crate::pdfs::pdf_abs_path;
use crate::session::{ServiceError, VaultSession};

/// The notes of a folder (the vault root when `None`), cut into chunks: each item's title is a topic.
pub fn notes_chunks(
    session: &VaultSession,
    folder: Option<&str>,
) -> Result<Vec<Chunk>, ServiceError> {
    let rows = list_items(&session.conn, folder).map_err(ServiceError::Store)?;
    let mut md = String::new();
    for row in rows
        .iter()
        .filter(|r| matches!(r.kind, ItemKind::Note | ItemKind::Card | ItemKind::Question))
    {
        let c = load_item_content(session, row.uid)?;
        md.push_str(&format!("# {}\n{}\n{}\n\n", c.title, c.prompt, c.answer));
    }
    Ok(chunk_markdown(&md))
}

/// The text of a lecture PDF in the vault (its slides), cut into chunks. `rel` is its vault-relative path;
/// anything outside the vault, or not a PDF, is refused.
pub fn slide_chunks(session: &VaultSession, rel: &str) -> Result<Vec<Chunk>, ServiceError> {
    let clean = !rel.starts_with('/') && !rel.split('/').any(|p| p == ".." || p.is_empty());
    if !clean || !rel.to_lowercase().ends_with(".pdf") {
        return Err(ServiceError::Invalid(
            "only PDF files inside the vault can be used".to_owned(),
        ));
    }
    let bytes = std::fs::read(pdf_abs_path(session, rel))
        .map_err(|e| ServiceError::Io(format!("cannot read {rel}: {e}")))?;
    chunks_of_pdf(&bytes).map_err(ServiceError::Invalid)
}

/// The exam as Markdown: its sections and questions, then the reference answers and rubrics.
pub fn exam_markdown(exam: &MockExam, keys: &[AnswerKey]) -> String {
    let mut out = format!(
        "# {}\n\nTime: {} minutes. Points: {}.\n",
        exam.title,
        exam.time_limit_min,
        total_points(exam)
    );
    let mut n = 0;
    for section in &exam.sections {
        out.push_str(&format!("\n## {}\n\n", section.name));
        for q in &section.questions {
            n += 1;
            out.push_str(&format!(
                "{n}. ({} points, {}) {}\n",
                q.points.0, q.topic, q.prompt
            ));
        }
    }
    out.push_str("\n## Reference answers\n\n");
    for (i, k) in keys.iter().enumerate() {
        out.push_str(&format!(
            "{}. {}\n   Points to make: {}\n",
            i + 1,
            k.sample_answer,
            k.rubric.join("; ")
        ));
    }
    out
}

/// Save the exam as a note in `folder` (the vault root when `None`); returns its vault path.
pub fn save_exam(
    session: &mut VaultSession,
    folder: Option<&str>,
    title: &str,
    exam: &MockExam,
    keys: &[AnswerKey],
    now_ms: u64,
) -> Result<String, ServiceError> {
    let text = exam_markdown(exam, keys);
    let item = NewItem {
        kind: NewKind::Note,
        title,
        text: &text,
        answer: "",
        tags: &[],
        parent: folder,
        source: None,
        priority: None,
        difficulty: None,
        also_in: None,
    };
    let (_, created) = create_item(session, &item, fresh_id_source(now_ms)?)?;
    Ok(created.path)
}
