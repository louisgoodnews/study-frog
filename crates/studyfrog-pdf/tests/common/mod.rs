#![allow(dead_code, clippy::unwrap_used, clippy::expect_used)] // shared test helpers
//! Small PDFs made for the tests, so no real file is needed.

use std::sync::Arc;

use lopdf::{Dictionary, Document, Object, Stream, StringFormat, dictionary};

/// A page: lines of text as (text, x, y) in PDF points from the bottom left.
pub type Page = Vec<(&'static str, f32, f32)>;

pub fn bytes(d: Vec<u8>) -> Arc<Vec<u8>> {
    Arc::new(d)
}

/// A PDF whose pages hold the given lines of Helvetica text; `title` goes into the Info dictionary.
pub fn make(pages: &[Page], title: Option<&str>) -> Vec<u8> {
    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();
    let font = doc.add_object(
        dictionary! {"Type" => "Font", "Subtype" => "Type1", "BaseFont" => "Helvetica"},
    );
    let resources = doc.add_object(dictionary! {"Font" => dictionary! {"F1" => font}});
    let mut kids = Vec::new();
    for lines in pages {
        let mut content = String::new();
        for (text, x, y) in lines {
            content.push_str(&format!("BT /F1 14 Tf {x} {y} Td ({text}) Tj ET\n"));
        }
        let stream = doc.add_object(Stream::new(Dictionary::new(), content.into_bytes()));
        let page = doc.add_object(dictionary! {
            "Type" => "Page", "Parent" => pages_id, "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
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
    doc.trailer.set("Root", catalog);
    if let Some(t) = title {
        let info = doc.add_object(dictionary! {"Title" => Object::String(t.as_bytes().to_vec(), StringFormat::Literal), "Author" => Object::String(b"Ada".to_vec(), StringFormat::Literal)});
        doc.trailer.set("Info", info);
    }
    let mut out = Vec::new();
    doc.save_to(&mut out).unwrap();
    out
}

/// A one-page PDF that is only a picture (like a scan): no text at all.
pub fn make_scan() -> Vec<u8> {
    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();
    let image = doc.add_object(Stream::new(
        dictionary! {"Type" => "XObject", "Subtype" => "Image", "Width" => 64, "Height" => 64, "ColorSpace" => "DeviceRGB", "BitsPerComponent" => 8},
        vec![200_u8; 64 * 64 * 3],
    ));
    let resources = doc.add_object(dictionary! {"XObject" => dictionary! {"Im1" => image}});
    let stream = doc.add_object(Stream::new(
        Dictionary::new(),
        b"q 300 0 0 300 50 400 cm /Im1 Do Q".to_vec(),
    ));
    let page = doc.add_object(dictionary! {
        "Type" => "Page", "Parent" => pages_id, "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
        "Contents" => stream, "Resources" => resources,
    });
    doc.objects.insert(
        pages_id,
        Object::Dictionary(
            dictionary! {"Type" => "Pages", "Kids" => vec![Object::Reference(page)], "Count" => 1},
        ),
    );
    let catalog = doc.add_object(dictionary! {"Type" => "Catalog", "Pages" => pages_id});
    doc.trailer.set("Root", catalog);
    let mut out = Vec::new();
    doc.save_to(&mut out).unwrap();
    out
}

/// Three pages with different words, for reordering tests.
pub fn three() -> Vec<u8> {
    make(
        &[
            vec![("Alpha first page", 72.0, 700.0)],
            vec![("Beta second page", 72.0, 700.0)],
            vec![("Gamma third page", 72.0, 700.0)],
        ],
        Some("Three pages"),
    )
}

pub fn rect(a: [i64; 4]) -> Object {
    Object::Array(a.iter().map(|v| Object::Integer(*v)).collect())
}

/// Three pages. Page 1 has: a web link, a link to page 3 by destination array, a link by name, a launch
/// action and a JavaScript action. The named destination `chap` is page 2.
pub fn linked() -> Vec<u8> {
    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();
    let ids: Vec<_> = (0..3).map(|_| doc.new_object_id()).collect();
    let uri = doc.add_object(dictionary! {"Type" => "Annot", "Subtype" => "Link", "Rect" => rect([72, 700, 172, 720]),
        "A" => dictionary! {"S" => "URI", "URI" => Object::String(b"https://example.org/x".to_vec(), StringFormat::Literal)}});
    let goto = doc.add_object(
        dictionary! {"Type" => "Annot", "Subtype" => "Link", "Rect" => rect([72, 600, 172, 620]),
        "A" => dictionary! {"S" => "GoTo", "D" => vec![Object::Reference(ids[2]), "Fit".into()]}},
    );
    let by_name = doc.add_object(
        dictionary! {"Type" => "Annot", "Subtype" => "Link", "Rect" => rect([72, 500, 172, 520]),
        "Dest" => Object::String(b"chap".to_vec(), StringFormat::Literal)},
    );
    let launch = doc.add_object(dictionary! {"Type" => "Annot", "Subtype" => "Link", "Rect" => rect([72, 400, 172, 420]),
        "A" => dictionary! {"S" => "Launch", "F" => Object::String(b"calc".to_vec(), StringFormat::Literal)}});
    let script = doc.add_object(dictionary! {"Type" => "Annot", "Subtype" => "Link", "Rect" => rect([72, 300, 172, 320]),
        "A" => dictionary! {"S" => "JavaScript", "JS" => Object::String(b"app.alert(1)".to_vec(), StringFormat::Literal)}});
    let note = doc.add_object(
        dictionary! {"Type" => "Annot", "Subtype" => "Text", "Rect" => rect([10, 10, 30, 30])},
    );
    let annots: Vec<Object> = [uri, goto, by_name, launch, script, note]
        .into_iter()
        .map(Object::Reference)
        .collect();
    for (i, id) in ids.iter().enumerate() {
        let stream = doc.add_object(Stream::new(Dictionary::new(), Vec::new()));
        let mut page = dictionary! {"Type" => "Page", "Parent" => pages_id, "MediaBox" => rect([0, 0, 612, 792]), "Contents" => stream};
        if i == 0 {
            page.set("Annots", annots.clone());
        }
        doc.objects.insert(*id, Object::Dictionary(page));
    }
    let kids: Vec<Object> = ids.iter().map(|i| Object::Reference(*i)).collect();
    doc.objects.insert(
        pages_id,
        Object::Dictionary(dictionary! {"Type" => "Pages", "Kids" => kids, "Count" => 3}),
    );
    let dests =
        doc.add_object(dictionary! {"chap" => vec![Object::Reference(ids[1]), "Fit".into()]});
    let catalog =
        doc.add_object(dictionary! {"Type" => "Catalog", "Pages" => pages_id, "Dests" => dests});
    doc.trailer.set("Root", catalog);
    let mut out = Vec::new();
    doc.save_to(&mut out).unwrap();
    out
}
