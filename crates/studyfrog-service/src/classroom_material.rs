//! Lecture material from the vault (STORY-0209, TASK-0290): the text of a session's sources (a stack's
//! notes, one note, or a lecture PDF), cut into pieces the teacher explains one at a time.

use studyfrog_core::classroom::{SourceKind, SourceRef};
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::curriculum::Chunk;
use studyfrog_store::overview::load_overview;
use studyfrog_store::pdf_index::list_pdf_rows;

use crate::content::load_item_content;
use crate::exam_save::{notes_chunks, slide_chunks};
use crate::session::{ServiceError, VaultSession};

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
const SECOND_SEED: u64 = 0x9e37_79b9_7f4a_7c15;

/// The id a lecture PDF is listed under in a session. A PDF has no id of its own in the vault, so the
/// id is a 128-bit fingerprint of its vault-relative path: the same path always gives the same id.
/// It is FNV-1a run twice with two seeds; it is not cryptographic, and a vault will not hold two
/// paths that collide. A crate was not used because a few lines cover this and the id must be stable.
pub fn pdf_source_uid(path: &str) -> Uid {
    let hi = fnv1a(path.as_bytes(), FNV_OFFSET);
    let lo = fnv1a(path.as_bytes(), FNV_OFFSET ^ SECOND_SEED);
    Uid((u128::from(hi) << 64) | u128::from(lo))
}

fn fnv1a(bytes: &[u8], seed: u64) -> u64 {
    bytes
        .iter()
        .fold(seed, |h, b| (h ^ u64::from(*b)).wrapping_mul(FNV_PRIME))
}

/// The text pieces of every source, in the order the sources were added. Empty pieces are dropped.
pub fn lecture_material(
    vault: &VaultSession,
    sources: &[SourceRef],
) -> Result<Vec<String>, ServiceError> {
    let mut out = Vec::new();
    for source in sources {
        out.extend(source_material(vault, source)?);
    }
    Ok(out)
}

/// The text pieces of one source.
pub fn source_material(
    vault: &VaultSession,
    source: &SourceRef,
) -> Result<Vec<String>, ServiceError> {
    match source.kind {
        SourceKind::Stack => stack_material(vault, source.uid),
        SourceKind::Note => note_material(vault, source.uid),
        SourceKind::Pdf => pdf_material(vault, source.uid),
    }
}

fn stack_material(vault: &VaultSession, uid: Uid) -> Result<Vec<String>, ServiceError> {
    let overview = load_overview(&vault.conn).map_err(ServiceError::Store)?;
    let folder = overview
        .stacks
        .iter()
        .find(|s| s.uid == uid)
        .map(|s| s.folder.clone())
        .ok_or_else(|| ServiceError::Invalid("that stack is not in the vault".to_owned()))?;
    Ok(chunk_texts(notes_chunks(vault, Some(&folder))?))
}

fn note_material(vault: &VaultSession, uid: Uid) -> Result<Vec<String>, ServiceError> {
    let c = load_item_content(vault, uid)?;
    let body = if c.answer.trim().is_empty() {
        c.prompt
    } else {
        c.answer
    };
    if body.trim().is_empty() {
        return Ok(Vec::new());
    }
    Ok(vec![format!("{}\n{}", c.title, body)])
}

fn pdf_material(vault: &VaultSession, uid: Uid) -> Result<Vec<String>, ServiceError> {
    let rows = list_pdf_rows(&vault.conn).map_err(ServiceError::Store)?;
    let path = rows
        .into_iter()
        .map(|r| r.path)
        .find(|p| pdf_source_uid(p) == uid)
        .ok_or_else(|| ServiceError::Invalid("that PDF is not in the vault index".to_owned()))?;
    Ok(chunk_texts(slide_chunks(vault, &path)?))
}

/// Each chunk as one piece: its topic line, then its text. Chunks with no text are dropped.
fn chunk_texts(chunks: Vec<Chunk>) -> Vec<String> {
    chunks
        .into_iter()
        .filter(|c| !c.text.trim().is_empty())
        .map(|c| format!("{}\n{}", c.topic, c.text))
        .collect()
}
