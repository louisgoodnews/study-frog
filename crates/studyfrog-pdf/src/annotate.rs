//! Writing highlights and notes into a copy of a PDF, as the standard annotation types other readers show.

use lopdf::{Dictionary, Object, ObjectId, dictionary};

use crate::PdfBytes;
use crate::edit::{Properties, write_properties};
use crate::error::PdfError;
use crate::limits::{FILE_TIME_LIMIT_SECS, check_size, guarded_for};
use crate::render::PageRect;

/// What kind of mark.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkKind {
    /// Text marked with a colour.
    Highlight,
    /// A note attached to a point.
    Note,
}

/// One annotation to write.
#[derive(Debug, Clone, PartialEq)]
pub struct Mark {
    /// The page (0-based).
    pub page: usize,
    /// Highlight or note.
    pub kind: MarkKind,
    /// The areas it covers, in points from the top left of the page.
    pub rects: Vec<PageRect>,
    /// Colour as red, green and blue between 0 and 1.
    pub color: [f32; 3],
    /// The text of the note (or of the passage, for a highlight).
    pub contents: String,
}

/// A written copy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkedCopy {
    /// The new file.
    pub bytes: Vec<u8>,
    /// Marks on pages that are rotated, which are left out because their places would be wrong.
    pub skipped: usize,
}

fn bad(e: impl std::fmt::Display) -> PdfError {
    PdfError::Invalid(e.to_string())
}

fn numbers(o: &Object) -> Option<[f32; 4]> {
    let a = o.as_array().ok()?;
    let n: Vec<f32> = a
        .iter()
        .filter_map(|v| {
            v.as_float()
                .ok()
                .or_else(|| v.as_i64().ok().map(|i| i as f32))
        })
        .collect();
    (n.len() == 4).then(|| [n[0], n[1], n[2], n[3]])
}

/// A key of a page, looking up through the page tree (MediaBox and Rotate are inherited).
fn inherited(doc: &lopdf::Document, mut id: ObjectId, key: &[u8]) -> Option<Object> {
    for _ in 0..32 {
        let d = doc.get_dictionary(id).ok()?;
        if let Ok(v) = d.get(key) {
            return Some(match v {
                Object::Reference(r) => doc.get_object(*r).ok()?.clone(),
                other => other.clone(),
            });
        }
        id = d.get(b"Parent").ok()?.as_reference().ok()?;
    }
    None
}

fn annotation(mark: &Mark, top: f32, left: f32) -> Dictionary {
    let flip = |r: &PageRect| {
        [
            left + r.x,
            top - r.y - r.height,
            left + r.x + r.width,
            top - r.y,
        ]
    };
    let boxes: Vec<[f32; 4]> = mark.rects.iter().map(flip).collect();
    let rect = boxes
        .iter()
        .fold([f32::MAX, f32::MAX, f32::MIN, f32::MIN], |a, b| {
            [
                a[0].min(b[0]),
                a[1].min(b[1]),
                a[2].max(b[2]),
                a[3].max(b[3]),
            ]
        });
    let color = Object::Array(mark.color.iter().map(|c| Object::Real(*c)).collect());
    let contents = crate::edit::pdf_text_object(&mark.contents);
    match mark.kind {
        MarkKind::Highlight => {
            let quads: Vec<Object> = boxes
                .iter()
                .flat_map(|b| [b[0], b[3], b[2], b[3], b[0], b[1], b[2], b[1]])
                .map(Object::Real)
                .collect();
            dictionary! {
                "Type" => "Annot", "Subtype" => "Highlight", "F" => 4,
                "Rect" => Object::Array(rect.iter().map(|v| Object::Real(*v)).collect()),
                "QuadPoints" => Object::Array(quads), "C" => color, "Contents" => contents,
            }
        }
        MarkKind::Note => {
            let at = [rect[0], rect[3] - 20.0, rect[0] + 20.0, rect[3]];
            dictionary! {
                "Type" => "Annot", "Subtype" => "Text", "Name" => "Note", "F" => 4,
                "Rect" => Object::Array(at.iter().map(|v| Object::Real(*v)).collect()),
                "C" => color, "Contents" => contents,
            }
        }
    }
}

fn attach(doc: &mut lopdf::Document, page: ObjectId, annot: ObjectId) -> Result<(), PdfError> {
    let existing = doc
        .get_dictionary(page)
        .map_err(bad)?
        .get(b"Annots")
        .ok()
        .cloned();
    let mut list = match existing {
        Some(Object::Array(a)) => a,
        Some(Object::Reference(r)) => doc
            .get_object(r)
            .ok()
            .and_then(|o| o.as_array().ok().cloned())
            .unwrap_or_default(),
        _ => Vec::new(),
    };
    list.push(Object::Reference(annot));
    doc.get_dictionary_mut(page)
        .map_err(bad)?
        .set("Annots", Object::Array(list));
    Ok(())
}

/// A copy of the PDF with `marks` written as annotations and, if given, new properties.
pub fn pdf_with_marks(
    bytes: &PdfBytes,
    marks: &[Mark],
    props: Option<&Properties>,
) -> Result<MarkedCopy, PdfError> {
    check_size(bytes)?;
    let (bytes, marks, props) = (bytes.clone(), marks.to_vec(), props.cloned());
    guarded_for(FILE_TIME_LIMIT_SECS, move || {
        let mut doc = lopdf::Document::load_mem(&bytes).map_err(bad)?;
        let pages = doc.get_pages();
        let mut skipped = 0;
        for mark in &marks {
            let id = *pages
                .get(&(mark.page as u32 + 1))
                .ok_or(PdfError::NoSuchPage(mark.page))?;
            let rotated = inherited(&doc, id, b"Rotate")
                .and_then(|o| o.as_i64().ok())
                .is_some_and(|r| r % 360 != 0);
            let media = inherited(&doc, id, b"MediaBox").as_ref().and_then(numbers);
            let Some([left, _, _, top]) = media.filter(|_| !rotated) else {
                skipped += 1;
                continue;
            };
            let annot = doc.add_object(annotation(mark, top, left));
            attach(&mut doc, id, annot)?;
        }
        if let Some(p) = &props {
            write_properties(&mut doc, p)?;
        }
        let mut out = Vec::new();
        doc.save_to(&mut out).map_err(bad)?;
        Ok(MarkedCopy {
            bytes: out,
            skipped,
        })
    })
}
