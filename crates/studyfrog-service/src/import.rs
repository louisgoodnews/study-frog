//! Putting cards that an extension produced into a stack.

use studyfrog_ext::types::ImportedCard;

use crate::create::{IdSource, NewCard, default_card_title, refresh, write_card};
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
fn nth(ids: IdSource, n: usize) -> IdSource {
    let [hi, lo] = u16::try_from(n).unwrap_or(u16::MAX).to_be_bytes();
    let mut random = ids.random;
    random[7] ^= hi;
    random[8] ^= lo;
    IdSource { random, ..ids }
}

/// The title a written card got: the given one, else one made from its front.
fn card_title(c: &ImportedCard) -> String {
    match c.title.trim() {
        "" => default_card_title(&c.front),
        given => given.to_owned(),
    }
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
    let mut report = ImportReport::default();
    for (i, c) in cards.iter().enumerate() {
        let card = NewCard {
            title: &c.title,
            front: &c.front,
            back: &c.back,
            tags: &c.tags,
        };
        match write_card(session, folder, &card, nth(ids, i)) {
            Ok(()) => {
                report.created += 1;
                report.titles.push(card_title(c));
            }
            Err(ServiceError::Invalid(why)) => {
                report.skipped.push(format!("card {}: {why}", i + 1))
            }
            Err(other) => return Err(other),
        }
    }
    Ok((refresh(session)?, report))
}
