#![allow(dead_code, clippy::unwrap_used, clippy::expect_used)]
//! Realistic and negative import fixture builders using existing workspace dependencies.

use std::fs;
use std::path::{Path, PathBuf};

use lopdf::{Dictionary, Document, Object, Stream, StringFormat, dictionary};
use rusqlite::Connection;
use serde_json::json;
use studyfrog_service::xfer_zip::write_zip;

fn write_bytes(dir: &Path, name: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&path, bytes).unwrap();
    path
}

/// Write a CSV fixture (`headered == true` includes `stack,kind,title,front,back,tags`).
pub fn write_csv_fixture(dir: &Path, name: &str, headered: bool) -> PathBuf {
    let body = if headered {
        "stack,kind,title,front,back,tags\nfrench,card,,the cat,le chat,animals;nouns\nfrench,question,Plural,\"What is the plural of \"\"cheval\"\"?\",chevaux,grammar\n"
    } else {
        "the cat,le chat\nthe dog,le chien\n"
    };
    write_bytes(dir, name, body.as_bytes())
}

/// Write a TSV fixture (`headered == true` includes `stack\tkind\tfront\tback\ttags`).
pub fn write_tsv_fixture(dir: &Path, name: &str, headered: bool) -> PathBuf {
    let body = if headered {
        "stack\tkind\tfront\tback\ttags\nspanish\tcard\thola\thello\tgreetings\nspanish\tcard\tgracias\tthank you\tgreetings\n"
    } else {
        "hola\thello\ngracias\tthank you\n"
    };
    write_bytes(dir, name, body.as_bytes())
}

/// Write a standalone Markdown note fixture with frontmatter, tags, and wikilinks.
pub fn write_md_note_fixture(dir: &Path, name: &str) -> PathBuf {
    let body = "---\ntitle: Cell Membrane Structure\ntags: [biology, histology]\n---\nThe phospholipid bilayer regulates transport across [[Organelles]] #membrane.\n";
    write_bytes(dir, name, body.as_bytes())
}

/// Write a standalone StudyFrog Markdown card fixture (`???` separator).
pub fn write_md_card_fixture(dir: &Path, name: &str) -> PathBuf {
    let body = "---\nid: 0000000000000000000000000A\nkind: card\ntitle: Ohm Law\ntags: [physics]\n---\nState Ohm's law.\n???\nV = I * R\n";
    write_bytes(dir, name, body.as_bytes())
}

/// Write an Obsidian vault folder (`.obsidian/app.json`, root note, nested stack note).
pub fn write_obsidian_vault_fixture(dir: &Path, name: &str) -> PathBuf {
    let root = dir.join(name);
    write_bytes(&root, ".obsidian/app.json", b"{}");
    write_bytes(
        &root,
        "Inbox Idea.md",
        b"Quick #idea about [[Mitosis]] without frontmatter.\n",
    );
    write_bytes(
        &root,
        "Biology/Cells.md",
        b"---\ntitle: Cell Notes\ntags: [bio, exam]\ncssclass: wide\n---\nSee [[Mitosis]] and #biology.\n",
    );
    root
}

/// Write a StudyFrog export v1 JSON fixture.
pub fn write_json_fixture(dir: &Path, name: &str) -> PathBuf {
    let doc = json!({
        "format": "studyfrog-export",
        "version": 1,
        "exported": "2026-01-01T00:00:00Z",
        "scheduling": false,
        "stacks": [{"folder": "french", "title": "French", "description": ""}],
        "items": [{"kind": "card", "title": "Cat", "stack": "french", "text": "the cat", "answer": "le chat", "tags": ["animals"]}]
    });
    write_bytes(
        dir,
        name,
        serde_json::to_string_pretty(&doc).unwrap().as_bytes(),
    )
}

/// Write a StudyFrog export v1 TOML fixture.
pub fn write_toml_fixture(dir: &Path, name: &str) -> PathBuf {
    let body = r#"format = "studyfrog-export"
version = 1
exported = "2026-01-01T00:00:00Z"
scheduling = false

[[stacks]]
folder = "french"
title = "French"
description = ""

[[items]]
kind = "card"
title = "Cat"
stack = "french"
text = "the cat"
answer = "le chat"
tags = ["animals"]
"#;
    write_bytes(dir, name, body.as_bytes())
}

/// Write a StudyFrog export v1 RON fixture.
pub fn write_ron_fixture(dir: &Path, name: &str) -> PathBuf {
    let body = r#"(
    format: "studyfrog-export",
    version: 1,
    exported: "2026-01-01T00:00:00Z",
    scheduling: false,
    stacks: [(folder: "french", title: "French", description: "")],
    items: [(kind: "card", title: "Cat", stack: "french", text: "the cat", answer: "le chat", tags: ["animals"])],
)
"#;
    write_bytes(dir, name, body.as_bytes())
}

/// Write a StudyFrog export `.zip` containing `export.json` and a media entry.
pub fn write_export_zip_fixture(dir: &Path, name: &str) -> PathBuf {
    let json_path = write_json_fixture(dir, ".tmp-export.json");
    let json_bytes = fs::read(&json_path).unwrap();
    let _ = fs::remove_file(&json_path);
    let zip_bytes = write_zip(&[
        ("export.json".to_owned(), json_bytes),
        ("files/french/pic.png".to_owned(), vec![137, 80, 78, 71]),
    ])
    .unwrap();
    write_bytes(dir, name, &zip_bytes)
}

/// Write a zipped Obsidian vault fixture.
pub fn write_obsidian_zip_fixture(dir: &Path, name: &str) -> PathBuf {
    let md = b"---\ntitle: Cell Notes\ntags: [bio]\n---\nSee [[Mitosis]].\n".to_vec();
    let zip_bytes = write_zip(&[
        (".obsidian/app.json".to_owned(), b"{}".to_vec()),
        ("Biology/Cells.md".to_owned(), md),
    ])
    .unwrap();
    write_bytes(dir, name, &zip_bytes)
}

/// Write a valid PDF 1.4 fixture with Helvetica font, text streams, and `Info` metadata.
pub fn write_pdf_fixture(dir: &Path, name: &str, title: &str, pages: &[&str]) -> PathBuf {
    let mut doc = Document::with_version("1.4");
    let pages_id = doc.new_object_id();
    let font = doc.add_object(
        dictionary! {"Type" => "Font", "Subtype" => "Type1", "BaseFont" => "Helvetica"},
    );
    let resources = doc.add_object(dictionary! {"Font" => dictionary! {"F1" => font}});
    let mut kids = Vec::new();
    for line in pages {
        let content = format!("BT /F1 14 Tf 72 700 Td ({line}) Tj ET\n");
        let stream = doc.add_object(Stream::new(Dictionary::new(), content.into_bytes()));
        let page = doc.add_object(dictionary! {
            "Type" => "Page", "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
            "Contents" => stream, "Resources" => resources,
        });
        kids.push(Object::Reference(page));
    }
    let count = kids.len() as i64;
    doc.objects.insert(
        pages_id,
        Object::Dictionary(dictionary! {"Type" => "Pages", "Kids" => kids, "Count" => count}),
    );
    let catalog = doc.add_object(dictionary! {"Type" => "Catalog", "Pages" => pages_id});
    let info = doc.add_object(dictionary! {
        "Title" => Object::String(title.as_bytes().to_vec(), StringFormat::Literal),
    });
    doc.trailer.set("Root", catalog);
    doc.trailer.set("Info", info);
    let mut out = Vec::new();
    doc.save_to(&mut out).unwrap();
    write_bytes(dir, name, &out)
}

/// Write a valid `.apkg` fixture containing `collection.anki2`, `media`, and an image asset.
pub fn write_apkg_fixture(dir: &Path, name: &str) -> PathBuf {
    let db_path = dir.join(format!("{name}.anki2.sqlite"));
    let _ = fs::remove_file(&db_path);
    let sql = r#"
        CREATE TABLE col (models TEXT, decks TEXT);
        CREATE TABLE notes (id INTEGER, mid INTEGER, tags TEXT, flds TEXT);
        CREATE TABLE cards (id INTEGER, nid INTEGER, did INTEGER);
        CREATE TABLE revlog (id INTEGER, cid INTEGER, ease INTEGER);
        INSERT INTO col VALUES ('{"1":{"type":0},"2":{"type":1}}', '{"10":{"name":"Languages::French"}}');
        INSERT INTO notes VALUES (100, 1, ' verbs ', 'Capital of France?' || char(31) || 'Paris <img src="Pic.PNG">');
        INSERT INTO notes VALUES (101, 2, 'astro', 'The {{c1::sun}} is a star' || char(31) || '');
        INSERT INTO cards VALUES (1000, 100, 10), (1001, 101, 10);
        INSERT INTO revlog VALUES (1700000000000, 1000, 3);
    "#;
    Connection::open(&db_path)
        .unwrap()
        .execute_batch(sql)
        .unwrap();
    let db_bytes = fs::read(&db_path).unwrap();
    let _ = fs::remove_file(&db_path);
    let apkg_bytes = write_zip(&[
        ("collection.anki2".to_owned(), db_bytes),
        ("media".to_owned(), br#"{"0":"Pic.PNG"}"#.to_vec()),
        ("0".to_owned(), vec![137, 80, 78, 71]),
    ])
    .unwrap();
    write_bytes(dir, name, &apkg_bytes)
}

/// Write an empty 0-byte negative fixture.
pub fn write_empty_file(dir: &Path, name: &str) -> PathBuf {
    write_bytes(dir, name, b"")
}

/// Write a corrupt zip negative fixture.
pub fn write_corrupt_zip(dir: &Path, name: &str) -> PathBuf {
    write_bytes(dir, name, b"PK\x03\x04corrupt-zip-payload")
}

/// Write a malformed PDF negative fixture (invalid body after `%PDF-1.4`).
pub fn write_bad_pdf(dir: &Path, name: &str) -> PathBuf {
    write_bytes(dir, name, b"%PDF-1.4\nnot-a-valid-xref-or-trailer\n%%EOF\n")
}

/// Write an invalid UTF-8 negative fixture.
pub fn write_invalid_utf8_file(dir: &Path, name: &str) -> PathBuf {
    write_bytes(dir, name, &[0xff, 0xfe, 0x00, 0x80])
}
