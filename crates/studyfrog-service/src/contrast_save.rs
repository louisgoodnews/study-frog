//! Keeping a contrastive drill's distinction: as a hint on each item or as one comparison note that links
//! them. Both replace what an earlier save of the same items wrote, and both can be undone.

use std::fs;

use studyfrog_core::uid::Uid;
use studyfrog_md::doc::{Doc, NoteDoc};
use studyfrog_md::extra::{extra_value, with_extra};
use studyfrog_md::parse::parse_doc;
use studyfrog_md::render::render_doc;
use studyfrog_md::ulid::format_ulid;

use crate::bulk::Snapshot;
use crate::contrast::{
    ContrastTarget, HINT, answer_mut, check_size, hint_line, load_contrast, titles, with_hint,
};
use crate::create::{IdSource, meta, refresh, render_error, slug, unique_name, write_new};
use crate::document::load_document;
use crate::session::{ServiceError, VaultInfo, VaultSession};

/// The files a save touched: path and the text before (`None`: new).
type Touched = Vec<(String, Option<String>)>;

/// The front-matter key of a comparison note: the ids of the items it compares, sorted.
pub const COMPARES_FIELD: &str = "compares";

/// What a save did.
#[derive(Debug, Clone, PartialEq)]
pub struct ContrastSaved {
    /// The vault after reindexing.
    pub info: VaultInfo,
    /// The files written (vault-relative).
    pub written: Vec<String>,
    /// How to take it back.
    pub undo: Snapshot,
}

fn io(path: &str, e: &std::io::Error) -> ServiceError {
    ServiceError::Io(format!("{path}: {e}"))
}

fn hints(s: &VaultSession, uids: &[Uid], text: &str) -> Result<Touched, ServiceError> {
    let loaded = uids
        .iter()
        .map(|u| load_document(s, *u))
        .collect::<Result<Vec<_>, _>>()?;
    let names = titles(&loaded.iter().map(|l| l.doc.clone()).collect::<Vec<_>>());
    let mut undo = Vec::new();
    for (i, mut l) in loaded.into_iter().enumerate() {
        let others: Vec<String> = names
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .map(|(_, t)| t.clone())
            .collect();
        let old = fs::read_to_string(s.root.join(&l.path)).map_err(|e| io(&l.path, &e))?;
        let Some(answer) = answer_mut(&mut l.doc) else {
            return Err(ServiceError::Invalid(format!(
                "{} cannot take a hint",
                l.path
            )));
        };
        *answer = with_hint(answer, &hint_line(&others, text));
        write_new(
            &s.root.join(&l.path),
            &render_doc(&l.doc).map_err(render_error)?,
        )?;
        undo.push((l.path, Some(old)));
    }
    Ok(undo)
}

fn note_body(s: &VaultSession, uids: &[Uid], text: &str) -> Result<String, ServiceError> {
    let items = load_contrast(s, uids)?;
    let links: Vec<String> = items.iter().map(|i| format!("[[{}]]", i.title)).collect();
    let one_line = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut body = format!(
        "Telling apart {}.\n\n{HINT}: {one_line}\n",
        links.join(", ")
    );
    for i in &items {
        let answer = crate::contrast::answer_core(&i.answer);
        body.push_str(&format!(
            "\n## [[{}]]\n\n**Asks:** {}\n\n**Answers:** {}\n",
            i.title, i.prompt, answer
        ));
    }
    Ok(body)
}

/// The comparison note of the folder that compares exactly `key`, if there is one.
fn existing_note(s: &VaultSession, folder: &str, key: &str) -> Option<(String, Doc)> {
    let dir = fs::read_dir(s.root.join(folder)).ok()?;
    dir.filter_map(Result::ok).find_map(|e| {
        let name = e.file_name().to_string_lossy().into_owned();
        let doc = parse_doc(&fs::read_to_string(e.path()).ok()?).ok()?;
        let rel = if folder.is_empty() {
            name
        } else {
            format!("{folder}/{name}")
        };
        (extra_value(&doc, COMPARES_FIELD).as_deref() == Some(key)).then_some((rel, doc))
    })
}

fn note(
    s: &VaultSession,
    uids: &[Uid],
    text: &str,
    ids: IdSource,
) -> Result<Touched, ServiceError> {
    let first = load_document(s, uids[0])?;
    let folder = first
        .path
        .rsplit_once('/')
        .map_or("", |(f, _)| f)
        .to_owned();
    let mut sorted: Vec<String> = uids.iter().map(|u| format_ulid(*u)).collect();
    sorted.sort();
    let key = format!("\"{}\"", sorted.join(" "));
    let body = note_body(s, uids, text)?;
    if let Some((rel, mut doc)) = existing_note(s, &folder, key.trim_matches('"')) {
        let old = fs::read_to_string(s.root.join(&rel)).map_err(|e| io(&rel, &e))?;
        if let Doc::Note(n) = &mut doc {
            n.body = body;
        }
        write_new(&s.root.join(&rel), &render_doc(&doc).map_err(render_error)?)?;
        return Ok(vec![(rel, Some(old))]);
    }
    let docs = uids
        .iter()
        .map(|u| load_document(s, *u).map(|l| l.doc))
        .collect::<Result<Vec<_>, _>>()?;
    let title = format!("Comparison: {}", titles(&docs).join(" vs "));
    let doc = Doc::Note(NoteDoc {
        meta: meta(&title, &[], ids),
        rehearse_title: false,
        body,
        extra: Vec::new(),
    });
    let doc = with_extra(&doc, COMPARES_FIELD, Some(&key));
    let dir = s.root.join(&folder);
    let name = unique_name(&slug(&title), ".md", |n| dir.join(n).exists());
    write_new(&dir.join(&name), &render_doc(&doc).map_err(render_error)?)?;
    let rel = if folder.is_empty() {
        name
    } else {
        format!("{folder}/{name}")
    };
    Ok(vec![(rel, None)])
}

/// Keep `text` as the distinction of the items `uids` (2 to 4), then reindex once.
pub fn save_contrast(
    s: &mut VaultSession,
    uids: &[Uid],
    text: &str,
    target: ContrastTarget,
    ids: IdSource,
) -> Result<ContrastSaved, ServiceError> {
    check_size(uids.len())?;
    if text.trim().is_empty() {
        return Err(ServiceError::Invalid(
            "write the distinction first".to_owned(),
        ));
    }
    let undo = match target {
        ContrastTarget::Hints => hints(s, uids, text)?,
        ContrastTarget::Note => note(s, uids, text, ids)?,
    };
    Ok(ContrastSaved {
        info: refresh(s)?,
        written: undo.iter().map(|(p, _)| p.clone()).collect(),
        undo: Snapshot(undo),
    })
}
