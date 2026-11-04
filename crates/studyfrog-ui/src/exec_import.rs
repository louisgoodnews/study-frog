//! Performing the import effects: read the file, run the extension, write the cards.

use std::path::Path;

use studyfrog_ext::registry::{find_card_type, importable_extensions, importer_for_file};
use studyfrog_ext::types::{Faces, ImportedCard};
use studyfrog_service::create::{IdSource, fresh_id_source};
use studyfrog_service::create_more::{NewItem, NewKind};
use studyfrog_service::import::import_cards;
use studyfrog_service::session::{ServiceError, VaultInfo, VaultSession};
use studyfrog_service::track::{DueDate, create_tracked, track_cards};

use crate::exec::Worker;
use crate::import_ui::ImportOutcome;
use crate::modal::{CreateOutcome, ModalMsg};
use crate::msg::Msg;

fn ids(now_ns: i64) -> Result<IdSource, ServiceError> {
    fresh_id_source(u64::try_from(now_ns / 1_000_000).unwrap_or(0))
}

/// Ask for a file an importer can read.
pub fn pick_file(w: &Worker) -> Msg {
    let extensions = importable_extensions(&w.registry);
    Msg::ImportFilePicked(
        w.file_picker
            .and_then(|pick| pick(&extensions))
            .map(|p| p.display().to_string()),
    )
}

/// Write cards into a stack and report; `Msg::Imported` either way.
fn write(w: &mut Worker, folder: &str, cards: &[ImportedCard], now_ns: i64) -> Msg {
    let Some(session) = &mut w.session else {
        return Msg::Imported(Err("no vault is open".to_owned()));
    };
    let result = ids(now_ns)
        .and_then(|ids| {
            let (info, report) = import_cards(session, folder, cards, ids)?;
            track_cards(session, &info, (folder, &report.titles), ids);
            Ok((info, report))
        })
        .map(|(info, report)| ImportOutcome { info, report })
        .map_err(|e| e.to_string());
    Msg::Imported(result)
}

/// Read `path`, convert it with the importer for its extension, and write the cards.
pub fn import_file(w: &mut Worker, folder: &str, path: &str, now_ns: i64) -> Msg {
    let name = Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path);
    let Some(importer) = importer_for_file(&w.registry, name).copied() else {
        let known = importable_extensions(&w.registry).join(", ");
        return Msg::Imported(Err(format!(
            "no importer reads that kind of file (known: {known})"
        )));
    };
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => return Msg::Imported(Err(format!("{path}: {e}"))),
    };
    match (importer.import)(&text) {
        Ok(cards) => write(w, folder, &cards, now_ns),
        Err(e) => Msg::Imported(Err(format!("{}: {e}", importer.name))),
    }
}

/// Expand one source text with a card type and write the cards.
pub fn from_card_type(
    w: &mut Worker,
    folder: &str,
    type_id: &str,
    source: &str,
    now_ns: i64,
) -> Msg {
    let Some(card_type) = find_card_type(&w.registry, type_id).copied() else {
        return Msg::Imported(Err(format!("card type \"{type_id}\" is not available")));
    };
    match (card_type.expand)(source) {
        Ok(faces) => {
            let cards: Vec<ImportedCard> = faces
                .into_iter()
                .map(|Faces { front, back }| ImportedCard {
                    front,
                    back,
                    ..ImportedCard::default()
                })
                .collect();
            write(w, folder, &cards, now_ns)
        }
        Err(e) => Msg::Imported(Err(format!("{}: {e}", card_type.name))),
    }
}

type ItemOpts<'a> = (
    &'a [String],
    Option<&'a str>,
    Option<&'a str>,
    Option<&'a DueDate>,
);

/// Create one item from the create dialog.
pub fn create_item(
    w: &mut Worker,
    kind: NewKind,
    [title, text, answer]: [&str; 3],
    (tags, parent, also_in, due): ItemOpts<'_>,
    (priority, difficulty): (
        Option<studyfrog_core::priority::Priority>,
        Option<studyfrog_core::event::Difficulty>,
    ),
    now_ns: i64,
) -> Msg {
    let Some(session) = &mut w.session else {
        return Msg::Modal(ModalMsg::Created(Err("no vault is open".to_owned())));
    };
    let item = NewItem {
        source: None,
        kind,
        title,
        text,
        answer,
        tags,
        parent,
        priority,
        difficulty,
        also_in,
    };
    let result = ids(now_ns)
        .and_then(|ids| create_tracked(session, &item, due, ids))
        .map(|t| {
            Box::new(CreateOutcome {
                info: t.info,
                created: t.created,
                warnings: t.warnings,
            })
        })
        .map_err(|e| e.to_string());
    Msg::Modal(ModalMsg::Created(result))
}

/// Run a creating use case with fresh ids and report the refreshed vault.
pub fn create(
    w: &mut Worker,
    now_ns: i64,
    f: impl FnOnce(&mut VaultSession, IdSource) -> Result<VaultInfo, ServiceError>,
) -> Msg {
    let Some(session) = &mut w.session else {
        return Msg::Created(Err("no vault is open".to_owned()));
    };
    let result = fresh_id_source(u64::try_from(now_ns / 1_000_000).unwrap_or(0))
        .and_then(|ids| f(session, ids));
    Msg::Created(result.map_err(|e| e.to_string()))
}

/// Ask for a picture and copy it into the vault.
pub fn add_image(w: &mut Worker, target: crate::image_ui::ImageTarget) -> Msg {
    let picked = w
        .file_picker
        .and_then(|pick| pick(&crate::images::IMAGE_EXTENSIONS));
    let result = match (&w.session, picked) {
        (_, None) => Ok(None),
        (None, _) => Err("no vault is open".to_owned()),
        (Some(session), Some(path)) => studyfrog_service::images::add_image(session, &path)
            .map(Some)
            .map_err(|e| e.to_string()),
    };
    Msg::ImageAdded(target, result)
}
