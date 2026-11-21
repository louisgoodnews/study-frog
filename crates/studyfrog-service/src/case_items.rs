//! Case studies in the vault (STORY-0179): creating a case file in a stack, and reading the valid cases
//! of a stack back. The file format is in [`crate::case_codec`].

use studyfrog_core::plan::ItemKind;
use studyfrog_core::uid::Uid;
use studyfrog_md::render::render_doc;
use studyfrog_rehearsal::case_study::{CaseStudyItem, validate_case_study};
use studyfrog_store::items::list_items;

use crate::case_codec::{case_from_doc, case_to_question};
use crate::create::{
    IdSource, check_title, meta, refresh, render_error, slug, unique_name, write_new,
};
use crate::create_more::Created;
use crate::session::{ServiceError, VaultInfo, VaultSession};

/// Create a case study as a question in `folder` (the vault root when `None`), reindex, and return the
/// refreshed vault with the new file's details.
pub fn create_case_study(
    session: &mut VaultSession,
    folder: Option<&str>,
    title: &str,
    case: &CaseStudyItem,
    ids: IdSource,
) -> Result<(VaultInfo, Created), ServiceError> {
    check_title(title)?;
    validate_case_study(case).map_err(ServiceError::Invalid)?;
    let doc = case_to_question(case, meta(title, &[], ids));
    let doc = crate::item_keys::with_new_key(&session.conn, doc)?;
    let uid = studyfrog_md::doc::doc_meta(&doc).uid;
    let text = render_doc(&doc).map_err(render_error)?;
    let base = match folder {
        Some(f) => session.root.join(f),
        None => session.root.clone(),
    };
    let name = unique_name(&slug(title), ".md", |n| base.join(n).exists());
    write_new(&base.join(&name), &text)?;
    let path = match folder {
        Some(f) => format!("{f}/{name}"),
        None => name,
    };
    let info = refresh(session)?;
    Ok((
        info,
        Created {
            title: title.to_owned(),
            path,
            uid: Some(uid),
        },
    ))
}

/// The valid case studies of `folder` (the vault root when `None`), with their item ids. Plain questions
/// and files that are not well formed cases are skipped.
pub fn load_case_studies(
    session: &VaultSession,
    folder: Option<&str>,
) -> Result<Vec<(Uid, CaseStudyItem)>, ServiceError> {
    let rows = list_items(&session.conn, folder).map_err(ServiceError::Store)?;
    let mut out = Vec::new();
    for row in rows.iter().filter(|r| r.kind == ItemKind::Question) {
        let doc = crate::content::load_doc(session, row.uid)?;
        if let Ok(Some(case)) = case_from_doc(&doc) {
            out.push((row.uid, case));
        }
    }
    Ok(out)
}
