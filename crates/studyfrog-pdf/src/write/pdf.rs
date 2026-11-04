//! Document serialization from PagedDocument into PDF bytes using lopdf.

use lopdf::{Dictionary, Document, Object, ObjectId, Stream, StringFormat, dictionary};
use studyfrog_md::blocks::Link;

use super::geometry::content_box;
use super::layout::LayoutOptions;
use super::metrics::measure_str;
use super::palette::{COLOR_BORDER_SUBTLE, COLOR_MUTED, rgb_to_floats};
use super::pdf_fonts::register_fonts;
use super::pdf_outline::build_pdf_outlines;
use super::pdf_stream::build_page_content_stream;
use super::primitives::{LayoutItem, LayoutPage, PagedDocument};
use super::to_unicode::{bytes_to_pdf_hex, encode_text_winansi};
use crate::error::PdfError;

#[allow(clippy::too_many_arguments)]
fn emit_header_footer(
    page: &LayoutPage,
    page_idx: usize,
    total_pages: usize,
    doc_title: Option<&str>,
    options: &LayoutOptions,
) -> Vec<u8> {
    let mut s = String::new();
    let (left, _, printable_w, _) = content_box(&options.geometry);
    let right = left + printable_w;
    let (mr, mg, mb) = rgb_to_floats(COLOR_MUTED);
    let (br, bg, bb) = rgb_to_floats(COLOR_BORDER_SUBTLE);

    if total_pages > 1 {
        if let Some(title) = doc_title {
            let enc = encode_text_winansi(title);
            let hex = bytes_to_pdf_hex(&enc);
            let y_text = page.height - 28.0;
            s.push_str(&format!(
                "BT /F1 8.00 Tf {mr:.3} {mg:.3} {mb:.3} rg 1 0 0 1 {left:.2} {y_text:.2} Tm {hex} Tj ET\n"
            ));
            let y_line = page.height - 34.0;
            s.push_str(&format!(
                "q 0.50 w {br:.3} {bg:.3} {bb:.3} RG {left:.2} {y_line:.2} m {right:.2} {y_line:.2} l S Q\n"
            ));
        }

        let y_foot_line = 38.0;
        s.push_str(&format!(
            "q 0.50 w {br:.3} {bg:.3} {bb:.3} RG {left:.2} {y_foot_line:.2} m {right:.2} {y_foot_line:.2} l S Q\n"
        ));

        let left_label = options.vault_name.as_deref().unwrap_or("StudyFrog");
        let enc_left = encode_text_winansi(left_label);
        let hex_left = bytes_to_pdf_hex(&enc_left);
        s.push_str(&format!(
            "BT /F1 8.00 Tf {mr:.3} {mg:.3} {mb:.3} rg 1 0 0 1 {left:.2} 26.00 Tm {hex_left} Tj ET\n"
        ));

        if options.page_numbers {
            let page_str = format!("Page {} of {}", page_idx + 1, total_pages);
            let page_w = measure_str(&page_str, 8.0, false, false);
            let enc_page = encode_text_winansi(&page_str);
            let hex_page = bytes_to_pdf_hex(&enc_page);
            let page_x = right - page_w;
            s.push_str(&format!(
                "BT /F1 8.00 Tf {mr:.3} {mg:.3} {mb:.3} rg 1 0 0 1 {page_x:.2} 26.00 Tm {hex_page} Tj ET\n"
            ));
        }
    }

    s.into_bytes()
}

fn collect_page_links(
    page: &LayoutPage,
    page_ids: &[ObjectId],
    doc_layout: &PagedDocument,
    doc: &mut Document,
) -> Vec<Object> {
    let mut annots = Vec::new();
    let page_h = page.height;

    for item in &page.items {
        if let LayoutItem::Text(line) = item {
            let mut cur_x = line.x;
            for run in &line.runs {
                if let Some(ref link) = run.link {
                    let y_bottom = page_h - (line.y + line.height);
                    let y_top = page_h - line.y;
                    let rect = vec![
                        (cur_x as i64).into(),
                        (y_bottom as i64).into(),
                        ((cur_x + run.width) as i64).into(),
                        (y_top as i64).into(),
                    ];

                    match link {
                        Link::Url(url) => {
                            let annot = doc.add_object(dictionary! {
                                "Type" => "Annot",
                                "Subtype" => "Link",
                                "Rect" => rect,
                                "Border" => vec![0.into(), 0.into(), 0.into()],
                                "A" => dictionary! {
                                    "S" => "URI",
                                    "URI" => Object::String(url.as_bytes().to_vec(), StringFormat::Literal),
                                },
                            });
                            annots.push(Object::Reference(annot));
                        }
                        Link::Wiki(target) => {
                            if let Some(dest) = doc_layout.headings.iter().find(|h| {
                                h.title.eq_ignore_ascii_case(target)
                                    && h.page_index < page_ids.len()
                            }) {
                                let target_page_id = page_ids[dest.page_index];
                                let target_h = doc_layout
                                    .pages
                                    .get(dest.page_index)
                                    .map_or(841.89, |p| p.height);
                                let target_top = (target_h - dest.y).max(0.0);
                                let annot = doc.add_object(dictionary! {
                                    "Type" => "Annot",
                                    "Subtype" => "Link",
                                    "Rect" => rect,
                                    "Border" => vec![0.into(), 0.into(), 0.into()],
                                    "A" => dictionary! {
                                        "S" => "GoTo",
                                        "D" => vec![
                                            Object::Reference(target_page_id),
                                            "XYZ".into(),
                                            54.into(),
                                            (target_top as i64).into(),
                                            Object::Null,
                                        ],
                                    },
                                });
                                annots.push(Object::Reference(annot));
                            }
                        }
                        Link::Image(_) => {}
                    }
                }
                cur_x += run.width;
            }
        }
    }
    annots
}

/// Serialize a [`PagedDocument`] into valid PDF bytes using `lopdf`.
pub fn write_pdf(doc_layout: &PagedDocument, options: &LayoutOptions) -> Result<Vec<u8>, PdfError> {
    let mut doc = Document::with_version("1.7");
    let font_res_id = register_fonts(&mut doc);
    let pages_id = doc.new_object_id();

    let total_pages = doc_layout.pages.len();
    let page_ids: Vec<ObjectId> = (0..total_pages).map(|_| doc.new_object_id()).collect();
    let page_heights: Vec<f32> = doc_layout.pages.iter().map(|p| p.height).collect();

    for (idx, page) in doc_layout.pages.iter().enumerate() {
        let mut content_bytes =
            emit_header_footer(page, idx, total_pages, doc_layout.title.as_deref(), options);
        content_bytes.extend(build_page_content_stream(page));

        let stream_id = doc.add_object(Stream::new(Dictionary::new(), content_bytes));
        let annots = collect_page_links(page, &page_ids, doc_layout, &mut doc);

        let mut page_dict = dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), (page.width as i64).into(), (page.height as i64).into()],
            "Contents" => stream_id,
            "Resources" => font_res_id,
        };
        if !annots.is_empty() {
            page_dict.set("Annots", annots);
        }
        doc.objects
            .insert(page_ids[idx], Object::Dictionary(page_dict));
    }

    let kids: Vec<Object> = page_ids.iter().map(|id| Object::Reference(*id)).collect();
    doc.objects.insert(
        pages_id,
        Object::Dictionary(dictionary! {
            "Type" => "Pages",
            "Kids" => kids,
            "Count" => total_pages as i64,
        }),
    );

    let mut catalog_dict = dictionary! {
        "Type" => "Catalog",
        "Pages" => pages_id,
    };

    if let Some(outlines_id) =
        build_pdf_outlines(&doc_layout.headings, &page_ids, &page_heights, &mut doc)
    {
        catalog_dict.set("Outlines", outlines_id);
    }

    let catalog_id = doc.add_object(catalog_dict);
    doc.trailer.set("Root", catalog_id);

    let title_str = doc_layout.title.as_deref().unwrap_or("StudyFrog Document");
    let author_str = options.author.as_deref().unwrap_or("StudyFrog");
    let info_id = doc.add_object(dictionary! {
        "Title" => Object::String(encode_text_winansi(title_str), StringFormat::Literal),
        "Author" => Object::String(encode_text_winansi(author_str), StringFormat::Literal),
        "Creator" => Object::String(b"StudyFrog".to_vec(), StringFormat::Literal),
        "CreationDate" => Object::String(b"D:20261104120000Z".to_vec(), StringFormat::Literal),
    });
    doc.trailer.set("Info", info_id);

    let mut out = Vec::new();
    doc.save_to(&mut out)
        .map_err(|e| PdfError::Invalid(format!("failed to serialize PDF: {e}")))?;
    Ok(out)
}
