//! Standard font registration and font selection for PDF generation.

use lopdf::{Dictionary, Document, ObjectId, Stream, dictionary};

use super::to_unicode::generate_to_unicode_cmap;

/// Select the font resource name for a given styling combination.
pub fn select_font(bold: bool, italic: bool, code: bool) -> &'static str {
    if code {
        if bold { "F6" } else { "F5" }
    } else {
        match (bold, italic) {
            (true, true) => "F4",
            (true, false) => "F2",
            (false, true) => "F3",
            (false, false) => "F1",
        }
    }
}

/// Register standard Helvetica and Courier font dictionaries with /ToUnicode CMaps in the document.
pub fn register_fonts(doc: &mut Document) -> ObjectId {
    let cmap_stream = doc.add_object(Stream::new(Dictionary::new(), generate_to_unicode_cmap()));

    let f1 = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica",
        "Encoding" => "WinAnsiEncoding",
        "ToUnicode" => cmap_stream,
    });
    let f2 = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica-Bold",
        "Encoding" => "WinAnsiEncoding",
        "ToUnicode" => cmap_stream,
    });
    let f3 = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica-Oblique",
        "Encoding" => "WinAnsiEncoding",
        "ToUnicode" => cmap_stream,
    });
    let f4 = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica-BoldOblique",
        "Encoding" => "WinAnsiEncoding",
        "ToUnicode" => cmap_stream,
    });
    let f5 = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Courier",
        "Encoding" => "WinAnsiEncoding",
        "ToUnicode" => cmap_stream,
    });
    let f6 = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Courier-Bold",
        "Encoding" => "WinAnsiEncoding",
        "ToUnicode" => cmap_stream,
    });

    doc.add_object(dictionary! {
        "Font" => dictionary! {
            "F1" => f1,
            "F2" => f2,
            "F3" => f3,
            "F4" => f4,
            "F5" => f5,
            "F6" => f6,
        }
    })
}
