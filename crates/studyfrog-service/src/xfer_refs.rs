//! After an import: the items that name other stacks (`also_in`) get a `referenced_by` with those stacks' ids
//! (ADR 0011). Stacks that do not exist yet are created, like the stack of an item; a reference to the item's own
//! stack is left out.

use studyfrog_md::parse::parse_doc;
use studyfrog_md::refs::{references_of, with_references};
use studyfrog_md::render::render_doc;
use studyfrog_md::ulid::format_ulid;
use studyfrog_store::overview::load_overview;

use crate::create::{IdSource, refresh, render_error, write_new};
use crate::session::{ServiceError, VaultSession};
use crate::xfer_import::ensure_stacks;
use crate::xfer_model::Bundle;

/// An item that was written and the stacks it names besides its own.
pub(crate) type Named = (String, Vec<String>);

/// Write `referenced_by` into the files in `written` (path, folders). Returns notes for the preview's report.
pub(crate) fn link_references(
    session: &mut VaultSession,
    (written, b): (&[Named], &Bundle),
    ids: IdSource,
    stacks_made: &mut usize,
) -> Result<Vec<String>, ServiceError> {
    for folder in written.iter().flat_map(|(_, f)| f) {
        ensure_stacks(session, folder, b, ids, stacks_made)?;
    }
    refresh(session)?;
    let known = load_overview(&session.conn)
        .map_err(ServiceError::Store)?
        .stacks;
    let mut notes = Vec::new();
    for (path, folders) in written {
        let file = session.root.join(path);
        let text = std::fs::read_to_string(&file).map_err(|e| ServiceError::Io(e.to_string()))?;
        let Ok(doc) = parse_doc(&text) else { continue };
        let mut refs = references_of(&doc);
        for folder in folders {
            let own = path.starts_with(&format!("{folder}/"));
            let Some(stack) = known.iter().find(|s| s.folder == *folder) else {
                notes.push(format!("{path}: the stack {folder} could not be made"));
                continue;
            };
            let id = format_ulid(stack.uid);
            if !own && !refs.contains(&id) {
                refs.push(id);
            }
        }
        let out = render_doc(&with_references(&doc, &refs)).map_err(render_error)?;
        write_new(&file, &out)?;
    }
    Ok(notes)
}
