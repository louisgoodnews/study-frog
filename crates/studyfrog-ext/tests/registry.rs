#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The registry as plain data.

use proptest::prelude::*;
use studyfrog_ext::registry::{
    find_card_type, importable_extensions, importer_for_file, register_card_type,
    register_importer, registry_new,
};
use studyfrog_ext::types::{CardType, Faces, ImportedCard, Importer};

fn no_cards(_: &str) -> Result<Vec<ImportedCard>, String> {
    Ok(Vec::new())
}

fn no_faces(_: &str) -> Result<Vec<Faces>, String> {
    Ok(Vec::new())
}

const fn importer(id: &'static str, extensions: &'static [&'static str]) -> Importer {
    Importer {
        id,
        name: "Test",
        extensions,
        import: no_cards,
    }
}

const fn card_type(id: &'static str) -> CardType {
    CardType {
        id,
        name: "Test",
        hint: "type something",
        expand: no_faces,
    }
}

#[test]
fn importers_are_found_by_file_extension_and_the_first_registered_wins() {
    let mut r = registry_new();
    register_importer(&mut r, importer("tsv", &["tsv", "txt"])).unwrap();
    register_importer(&mut r, importer("other", &["txt", "csv"])).unwrap();
    assert_eq!(
        importer_for_file(&r, "deck.TSV").unwrap().id,
        "tsv",
        "case is ignored"
    );
    assert_eq!(
        importer_for_file(&r, "notes.txt").unwrap().id,
        "tsv",
        "first registered wins"
    );
    assert_eq!(importer_for_file(&r, "x.csv").unwrap().id, "other");
    assert!(
        importer_for_file(&r, "archive.tar.gz").is_none()
            && importer_for_file(&r, "noext").is_none()
    );
    assert_eq!(importable_extensions(&r), ["tsv", "txt", "csv"]);
}

#[test]
fn bad_or_repeated_registrations_are_refused_with_a_reason() {
    let mut r = registry_new();
    register_importer(&mut r, importer("tsv", &["tsv"])).unwrap();
    for (bad, needle) in [
        (importer("tsv", &["x"]), "already registered"),
        (importer("Bad Id", &["x"]), "lowercase"),
        (importer("-x", &["x"]), "lowercase"),
        (importer("", &["x"]), "lowercase"),
        (importer("none", &[]), "extensions"),
        (importer("dot", &[".tsv"]), "without a dot"),
    ] {
        assert!(
            register_importer(&mut r, bad).unwrap_err().contains(needle),
            "{needle}"
        );
    }
    assert_eq!(r.importers.len(), 1, "refused ones are not added");

    register_card_type(&mut r, card_type("cloze")).unwrap();
    assert!(
        register_card_type(&mut r, card_type("cloze"))
            .unwrap_err()
            .contains("already")
    );
    assert!(register_card_type(&mut r, card_type("Cloze!")).is_err());
    assert_eq!(find_card_type(&r, "cloze").unwrap().hint, "type something");
    assert!(find_card_type(&r, "nope").is_none());
}

proptest! {
    #[test]
    fn registering_never_loses_or_reorders_what_was_accepted(ids in prop::collection::vec("[a-z][a-z0-9-]{0,6}", 0..12)) {
        let mut r = registry_new();
        let mut accepted: Vec<String> = Vec::new();
        for id in &ids {
            let leaked: &'static str = Box::leak(id.clone().into_boxed_str());
            if register_card_type(&mut r, card_type(leaked)).is_ok() {
                accepted.push(id.clone());
            }
        }
        let have: Vec<String> = r.card_types.iter().map(|c| c.id.to_owned()).collect();
        prop_assert_eq!(have.clone(), accepted);
        let mut unique = have.clone();
        unique.sort();
        unique.dedup();
        prop_assert_eq!(unique.len(), have.len(), "ids are unique");
    }
}
