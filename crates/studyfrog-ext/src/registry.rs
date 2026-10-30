//! Registering and finding extensions.

use crate::types::{CardType, Importer, Registry};

/// An empty registry.
pub fn registry_new() -> Registry {
    Registry::default()
}

fn id_problem(id: &str) -> Option<&'static str> {
    let ok = !id.is_empty()
        && !id.starts_with('-')
        && !id.ends_with('-')
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    (!ok).then_some("an id is lowercase letters, digits and `-`")
}

/// Add an importer. Refused when its id is malformed or already taken, or it reads no extension.
pub fn register_importer(r: &mut Registry, importer: Importer) -> Result<(), String> {
    if let Some(why) = id_problem(importer.id) {
        return Err(format!("importer \"{}\": {why}", importer.id));
    }
    if r.importers.iter().any(|i| i.id == importer.id) {
        return Err(format!(
            "importer \"{}\" is already registered",
            importer.id
        ));
    }
    if importer.extensions.is_empty()
        || importer
            .extensions
            .iter()
            .any(|e| e.is_empty() || e.contains('.'))
    {
        return Err(format!(
            "importer \"{}\" needs file extensions without a dot",
            importer.id
        ));
    }
    r.importers.push(importer);
    Ok(())
}

/// Add a card type. Refused when its id is malformed or already taken.
pub fn register_card_type(r: &mut Registry, card_type: CardType) -> Result<(), String> {
    if let Some(why) = id_problem(card_type.id) {
        return Err(format!("card type \"{}\": {why}", card_type.id));
    }
    if r.card_types.iter().any(|c| c.id == card_type.id) {
        return Err(format!(
            "card type \"{}\" is already registered",
            card_type.id
        ));
    }
    r.card_types.push(card_type);
    Ok(())
}

/// The importer that reads files named `file_name` (by extension, case-insensitive); the first one
/// registered wins.
pub fn importer_for_file<'a>(r: &'a Registry, file_name: &str) -> Option<&'a Importer> {
    let ext = file_name.rsplit_once('.')?.1.to_ascii_lowercase();
    r.importers
        .iter()
        .find(|i| i.extensions.iter().any(|e| *e == ext))
}

/// The card type with this id.
pub fn find_card_type<'a>(r: &'a Registry, id: &str) -> Option<&'a CardType> {
    r.card_types.iter().find(|c| c.id == id)
}

/// Every file extension some importer reads, without repeats.
pub fn importable_extensions(r: &Registry) -> Vec<&'static str> {
    let mut all: Vec<&'static str> = Vec::new();
    for e in r
        .importers
        .iter()
        .flat_map(|i| i.extensions.iter().copied())
    {
        if !all.contains(&e) {
            all.push(e);
        }
    }
    all
}
