#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The Anki importer: decks, basic and cloze notes, tags, pictures, the review log, and a corpus of broken packages.

use std::fs;

use rusqlite::Connection;
use studyfrog_core::plan::ItemKind;
use studyfrog_core::review::Rating;
use studyfrog_service::create::IdSource;
use studyfrog_service::session::open_vault;
use studyfrog_service::xfer_anki::{cloze, from_apkg, html_to_text};
use studyfrog_service::xfer_import::{Policy, apply_import};
use studyfrog_service::xfer_zip::write_zip;
use studyfrog_store::schedule::load_mem_state;

const SQL: &str = "
CREATE TABLE col (models TEXT, decks TEXT);
CREATE TABLE notes (id INTEGER, mid INTEGER, tags TEXT, flds TEXT);
CREATE TABLE cards (id INTEGER, nid INTEGER, did INTEGER);
CREATE TABLE revlog (id INTEGER, cid INTEGER, ease INTEGER);
INSERT INTO col VALUES ('{\"1\":{\"type\":0},\"2\":{\"type\":1}}', '{\"10\":{\"name\":\"Languages::French\"},\"11\":{\"name\":\"Maths\"}}');
INSERT INTO notes VALUES (100, 1, ' verbs  irregular ', 'Capital of <b>France</b>?&nbsp;<br>Think' || char(31) || 'Paris <img src=\"Pic.PNG\">');
INSERT INTO notes VALUES (101, 2, '', 'The {{c1::sun}} is a {{c2::star::kind}}' || char(31) || '');
INSERT INTO notes VALUES (102, 1, 'x', '2+2' || char(31) || '4');
INSERT INTO cards VALUES (1000, 100, 10);
INSERT INTO cards VALUES (1001, 101, 10);
INSERT INTO cards VALUES (1002, 102, 11);
INSERT INTO revlog VALUES (1700000000000, 1000, 3);
INSERT INTO revlog VALUES (1700000000000 + 259200000, 1000, 4);
INSERT INTO revlog VALUES (1700000000000, 1002, 1);
";

fn apkg(label: &str, extra: &[(&str, Vec<u8>)]) -> Vec<u8> {
    let path = std::env::temp_dir().join(format!(
        "studyfrog-apkg-{label}-{}.sqlite",
        std::process::id()
    ));
    let _ = fs::remove_file(&path);
    Connection::open(&path).unwrap().execute_batch(SQL).unwrap();
    let mut files = vec![
        ("collection.anki2".to_owned(), fs::read(&path).unwrap()),
        ("media".to_owned(), br#"{"0":"Pic.PNG"}"#.to_vec()),
        ("0".to_owned(), vec![137, 80, 78, 71]),
    ];
    files.retain(|(n, _)| !extra.iter().any(|(e, _)| e == n));
    files.extend(extra.iter().map(|(n, d)| ((*n).to_owned(), d.clone())));
    let _ = fs::remove_file(&path);
    write_zip(&files).unwrap()
}

#[test]
fn text_helpers_turn_html_and_clozes_into_markdown_text() {
    let media =
        std::collections::HashMap::from([("a.png".to_owned(), "images/anki/a.png".to_owned())]);
    assert_eq!(
        html_to_text("a<br>b <i>c</i>&amp;&lt;<img src=\"A.png\">", &media),
        "a\nb c&<![](images/anki/a.png)"
    );
    assert_eq!(html_to_text("<img src=\"missing.png\">x", &media), "x");
    assert_eq!(
        cloze("The {{c1::sun}} is a {{c2::star::kind}}", Some(1)),
        "The [...] is a star"
    );
    assert_eq!(
        cloze("The {{c1::sun}} is a {{c2::star::kind}}", None),
        "The sun is a star"
    );
    assert_eq!(cloze("broken {{c1::x", Some(1)), "broken {{c1::x");
}

#[test]
fn decks_become_stacks_notes_become_cards_and_history_becomes_reviews() {
    let b = from_apkg(&apkg("ok", &[])).unwrap();
    let folders: Vec<&str> = b.stacks.iter().map(|s| s.folder.as_str()).collect();
    assert_eq!(folders, ["Languages/French", "Maths"]);
    assert_eq!(b.items.len(), 4, "one basic, two cloze numbers, one basic");
    let first = &b.items[0];
    assert_eq!(
        (first.kind, first.stack.as_str()),
        (ItemKind::Card, "Languages/French")
    );
    assert_eq!(first.text, "Capital of France?\nThink");
    assert_eq!(first.answer, "Paris ![](images/anki/Pic.PNG)");
    assert_eq!(first.tags, ["verbs", "irregular"]);
    assert_eq!(b.files[0].0, "images/anki/Pic.PNG");
    assert_eq!(
        (b.items[1].text.as_str(), b.items[1].answer.as_str()),
        ("The [...] is a star", "The sun is a star")
    );
    assert_eq!(b.items[2].text, "The sun is a [...]");
    let ratings: Vec<Rating> = first.reviews.iter().map(|r| r.rating).collect();
    assert_eq!(ratings, [Rating::Good, Rating::Easy]);
    assert_eq!(first.reviews[1].elapsed_days, 3.0);
    assert_eq!(b.items[3].reviews[0].rating, Rating::Again);
}

#[test]
fn the_import_writes_cards_with_pictures_and_rebuilds_the_schedule_only_when_asked() {
    let b = from_apkg(&apkg("write", &[])).unwrap();
    let dir = std::env::temp_dir().join(format!("studyfrog-anki-vault-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let (mut s, _) = open_vault(dir.to_str().unwrap()).unwrap();
    let ids = IdSource {
        now_ms: 1_800_000_000_000,
        random: [3; 10],
    };
    let done = apply_import(&mut s, &b, None, (Policy::Skip, true), ids).unwrap();
    assert_eq!(done.created, 4, "{:?}", done.notes);
    assert!(dir.join("images/anki/Pic.PNG").is_file());
    let metas = studyfrog_store::item_meta::load_item_meta(&s.conn).unwrap();
    let with_state = metas
        .keys()
        .filter(|u| load_mem_state(&s.conn, **u).unwrap().is_some())
        .count();
    assert_eq!(
        with_state, 2,
        "the two notes with a review log have a schedule"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn broken_and_hostile_packages_are_errors_and_never_crash_or_hang() {
    let junk: Vec<Vec<u8>> = vec![
        Vec::new(),
        b"PK".to_vec(),
        vec![0; 1000],
        write_zip(&[]).unwrap(),
        write_zip(&[("collection.anki2".into(), b"not sqlite".to_vec())]).unwrap(),
        write_zip(&[("collection.anki2".into(), Vec::new())]).unwrap(),
        write_zip(&[("collection.anki2".into(), b"SQLite format 3\0".repeat(500))]).unwrap(),
        write_zip(&[("collection.anki21b".into(), vec![1, 2, 3])]).unwrap(),
        write_zip(&[("../../etc/passwd".into(), vec![1])]).unwrap(),
    ];
    for (i, bytes) in junk.iter().enumerate() {
        assert!(from_apkg(bytes).is_err(), "case {i}");
    }
    let truncated = apkg("trunc", &[]);
    for cut in [10, 100, truncated.len() / 2, truncated.len() - 5] {
        let _ = from_apkg(&truncated[..cut]);
    }
    let mut flipped = apkg("flip", &[]);
    for i in (0..flipped.len()).step_by(97) {
        flipped[i] ^= 0xff;
    }
    let _ = from_apkg(&flipped);
    let bad_media = from_apkg(&apkg(
        "media",
        &[("media", b"{\"0\": \"../x.png\", \"1\": 5}".to_vec())],
    ))
    .unwrap();
    assert!(
        bad_media.files.iter().all(|(p, _)| !p.contains("..")),
        "a media name cannot leave the folder"
    );
}
