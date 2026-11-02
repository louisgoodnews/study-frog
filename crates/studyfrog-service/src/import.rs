//! Putting cards that an extension produced into a stack.

use studyfrog_ext::types::ImportedCard;

use crate::create::IdSource;
use crate::session::{ServiceError, VaultInfo, VaultSession};

/// Most cards one import may create (ids are made from one source and one counter).
pub const MAX_IMPORT: usize = 5_000;

/// What an import did.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ImportReport {
    /// Cards written.
    pub created: usize,
    /// The titles of the cards written, in order.
    pub titles: Vec<String>,
    /// Cards that could not be written, each with the reason; the rest were still written.
    pub skipped: Vec<String>,
}

/// The id source for the `n`th card of an import: same time, random bytes changed by the counter.
pub(crate) fn nth(ids: IdSource, n: usize) -> IdSource {
    let [hi, lo] = u16::try_from(n).unwrap_or(u16::MAX).to_be_bytes();
    let mut random = ids.random;
    random[7] ^= hi;
    random[8] ^= lo;
    IdSource { random, ..ids }
}

/// Write every card into the stack at `folder`, then reindex once. A card that cannot be written
/// (an empty side, an unusable title) is skipped and reported; the others are kept.
pub fn import_cards(
    session: &mut VaultSession,
    folder: &str,
    cards: &[ImportedCard],
    ids: IdSource,
) -> Result<(VaultInfo, ImportReport), ServiceError> {
    if cards.len() > MAX_IMPORT {
        return Err(ServiceError::Invalid(format!(
            "that would create {} cards; one import is limited to {MAX_IMPORT}",
            cards.len()
        )));
    }
    if !session
        .root
        .join(folder)
        .join(studyfrog_store::stackmap::STACK_FILE)
        .is_file()
    {
        let report = ImportReport {
            skipped: (1..=cards.len())
                .map(|i| format!("card {i}: that folder is not a stack"))
                .collect(),
            ..ImportReport::default()
        };
        return Ok((
            crate::session::vault_info(&session.root, &session.conn)?,
            report,
        ));
    }
    let items = cards
        .iter()
        .map(|c| {
            let mut it = crate::xfer_model::item(
                studyfrog_core::plan::ItemKind::Card,
                "",
                &c.front,
                &c.back,
            );
            it.title = c.title.clone();
            it.tags = c.tags.clone();
            it
        })
        .collect();
    let bundle = crate::xfer_model::Bundle {
        items,
        ..Default::default()
    };
    // The same path as every other import; cards that exist already are written again, as this import always did.
    let plan = crate::xfer_plan::plan_import(session, &bundle, Some(folder))?;
    let done = crate::xfer_import::apply_import(
        session,
        &bundle,
        Some(folder),
        (crate::xfer_plan::Policy::KeepBoth, false),
        ids,
    )?;
    let mut report = ImportReport {
        created: done.created,
        ..ImportReport::default()
    };
    for (i, line) in plan.lines.iter().enumerate() {
        match &line.verdict {
            crate::xfer_plan::Verdict::Error(why) => {
                report.skipped.push(format!("card {}: {why}", i + 1))
            }
            _ => report.titles.push(line.title.clone()),
        }
    }
    Ok((done.info, report))
}
