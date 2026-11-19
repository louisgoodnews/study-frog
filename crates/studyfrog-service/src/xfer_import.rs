//! Import: writing a bundle into the vault after the dry run (`docs/specs/interchange.md`).

use studyfrog_core::uid::Uid;
use studyfrog_md::doc::{Doc, StackDoc};
use studyfrog_md::render::render_doc;
use studyfrog_store::stackmap::STACK_FILE;

use crate::create::{IdSource, meta, refresh, render_error, slug, unique_name, write_new};
use crate::import::nth;
use crate::session::{ServiceError, VaultInfo, VaultSession};
use crate::xfer_model::{Bundle, XItem};
pub use crate::xfer_plan::{
    ImportPlan, ImportTarget, MAX_ITEMS, PlanLine, Policy, Verdict, counts, plan_import,
    plan_import_to,
};
use crate::xfer_plan::{find, known, label};
use crate::xfer_target::{doc_of, ensure_new_stack, insert_reviews, target_from_opt};

pub(crate) fn ensure_stacks(
    session: &VaultSession,
    folder: &str,
    b: &Bundle,
    ids: IdSource,
    n: &mut usize,
) -> Result<(), ServiceError> {
    if folder.is_empty() {
        return Ok(());
    }
    let mut upto = String::new();
    for part in folder.split('/') {
        upto = if upto.is_empty() {
            part.to_owned()
        } else {
            format!("{upto}/{part}")
        };
        let file = session.root.join(&upto).join(STACK_FILE);
        if file.is_file() {
            continue;
        }
        let info = b.stacks.iter().find(|s| s.folder == upto);
        let title = info.map_or_else(|| part.to_owned(), |s| s.title.clone());
        *n += 1;
        let doc = Doc::Stack(StackDoc {
            meta: meta(&title, &[], nth(ids, 50_000 + *n)),
            description: info.map_or_else(String::new, |s| s.description.clone()),
            body: String::new(),
            extra: Vec::new(),
        });
        let doc = crate::item_keys::with_new_key(&session.conn, doc)?;
        write_new(&file, &render_doc(&doc).map_err(render_error)?)?;
    }
    Ok(())
}

/// What an import wrote.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportDone {
    /// Items written as new.
    pub created: usize,
    /// Existing items overwritten.
    pub replaced: usize,
    /// Items left out (duplicates skipped, errors).
    pub skipped: usize,
    /// Things worth telling the user.
    pub notes: Vec<String>,
    /// The vault afterwards.
    pub info: VaultInfo,
}

/// Write the bundle to `target`: new items are created, duplicates follow `policy`, errors are skipped.
pub fn apply_import_to(
    session: &mut VaultSession,
    b: &Bundle,
    target: &ImportTarget,
    (policy, with_schedule): (Policy, bool),
    ids: IdSource,
) -> Result<ImportDone, ServiceError> {
    let plan = plan_import_to(session, b, target)?;
    let k = known(session)?;
    let (mut created, mut replaced, mut skipped, mut stacks) = (0, 0, 0, 0);
    let mut notes = plan.warnings.clone();
    let mut named: Vec<crate::xfer_refs::Named> = Vec::new();
    let mut histories: Vec<(Uid, &XItem)> = Vec::new();
    let will_write = plan.lines.iter().any(|l| {
        !matches!(l.verdict, Verdict::Error(_))
            && !matches!((&l.verdict, policy), (Verdict::Duplicate(_), Policy::Skip))
    });
    if let ImportTarget::NewStack(name) = target
        && will_write
    {
        ensure_new_stack(session, name, nth(ids, 50_000))?;
        stacks += 1;
    }
    for (n, (item, line)) in b.items.iter().zip(&plan.lines).enumerate() {
        let (uid, path) = match (&line.verdict, policy) {
            (Verdict::Error(e), _) => {
                notes.push(format!("{}: {e}", line.title));
                skipped += 1;
                continue;
            }
            (Verdict::Duplicate(_), Policy::Skip) => {
                skipped += 1;
                continue;
            }
            (Verdict::Duplicate(p), Policy::Replace) => (
                find(&k, item, &line.folder).map_or(Uid(0), |e| e.0),
                Some(p.clone()),
            ),
            (Verdict::Duplicate(_), Policy::KeepBoth) => (fresh_uid(ids, n), None),
            (Verdict::Create, _) => (item.id.unwrap_or_else(|| fresh_uid(ids, n)), None),
        };
        ensure_stacks(session, &line.folder, b, ids, &mut stacks)?;
        let doc = crate::item_keys::with_new_key(
            &session.conn,
            doc_of(item, uid, nth(ids, n), &mut notes),
        )?;
        let text = render_doc(&doc).map_err(render_error)?;
        let file = match &path {
            Some(p) => p.clone(),
            None if line.folder.is_empty() => unique_name(&slug(&label(item)), ".md", |c| {
                session.root.join(c).exists()
            }),
            None => {
                let dir = session.root.join(&line.folder);
                format!(
                    "{}/{}",
                    line.folder,
                    unique_name(&slug(&label(item)), ".md", |c| dir.join(c).exists())
                )
            }
        };
        write_new(&session.root.join(&file), &text)?;
        if !item.also_in.is_empty() {
            named.push((file.clone(), item.also_in.clone()));
        }
        if path.is_some() {
            replaced += 1;
        } else {
            created += 1;
            if with_schedule && !item.reviews.is_empty() {
                histories.push((uid, item));
            }
        }
    }
    crate::xfer_files::copy_files(session, b);
    notes.extend(crate::xfer_refs::link_references(
        session,
        (&named, b),
        ids,
        &mut stacks,
    )?);
    let info = refresh(session)?;
    // After the reindex, so each history lands in the stack the item now lives in.
    for (uid, item) in histories {
        insert_reviews(session, uid, item);
    }
    Ok(ImportDone {
        created,
        replaced,
        skipped,
        notes,
        info,
    })
}

/// Write the bundle: new items are created, duplicates are handled by `policy`, and nothing is written for items with errors.
pub fn apply_import(
    session: &mut VaultSession,
    b: &Bundle,
    target: Option<&str>,
    (policy, with_schedule): (Policy, bool),
    ids: IdSource,
) -> Result<ImportDone, ServiceError> {
    apply_import_to(
        session,
        b,
        &target_from_opt(target),
        (policy, with_schedule),
        ids,
    )
}

fn fresh_uid(ids: IdSource, n: usize) -> Uid {
    studyfrog_md::ulid::compose_ulid(ids.now_ms, nth(ids, n).random)
}
