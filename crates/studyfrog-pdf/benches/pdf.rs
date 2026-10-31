#![allow(missing_docs, clippy::expect_used, clippy::unwrap_used)] // criterion_group! generates an undocumented fn; benches may panic

//! A 500-page PDF: reading its structure, extracting all its text, drawing one page and rearranging it
//! (TASK-0122). The file is generated once, with about 300 words per page.

use std::hint::black_box;
use std::sync::Arc;

use criterion::{Criterion, criterion_group, criterion_main};
use lopdf::{Dictionary, Document, Object, Stream, dictionary};
use studyfrog_pdf::PdfBytes;
use studyfrog_pdf::doc::pdf_info;
use studyfrog_pdf::edit::{PageSpec, pdf_build};
use studyfrog_pdf::render::pdf_render_page;
use studyfrog_pdf::text::{pdf_all_text, pdf_page_text};

const PAGES: usize = 500;

fn big_pdf() -> PdfBytes {
    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();
    let font = doc.add_object(
        dictionary! {"Type" => "Font", "Subtype" => "Type1", "BaseFont" => "Helvetica"},
    );
    let resources = doc.add_object(dictionary! {"Font" => dictionary! {"F1" => font}});
    let mut kids = Vec::new();
    for p in 0..PAGES {
        let mut content = String::new();
        for line in 0..40 {
            let y = 760 - line * 18;
            content.push_str(&format!(
                "BT /F1 11 Tf 72 {y} Td (Page {p} line {line}: retrieval practice beats rereading and spacing slows forgetting) Tj ET\n"
            ));
        }
        let stream = doc.add_object(Stream::new(Dictionary::new(), content.into_bytes()));
        let page = doc.add_object(dictionary! {
            "Type" => "Page", "Parent" => pages_id, "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
            "Contents" => stream, "Resources" => resources,
        });
        kids.push(Object::Reference(page));
    }
    doc.objects.insert(
        pages_id,
        Object::Dictionary(
            dictionary! {"Type" => "Pages", "Kids" => kids, "Count" => PAGES as i64},
        ),
    );
    let catalog = doc.add_object(dictionary! {"Type" => "Catalog", "Pages" => pages_id});
    doc.trailer.set("Root", catalog);
    let mut out = Vec::new();
    doc.save_to(&mut out).expect("save");
    Arc::new(out)
}

fn benches(c: &mut Criterion) {
    let pdf = big_pdf();
    let mut group = c.benchmark_group("pdf_500_pages");
    group.sample_size(10);
    group.bench_function("info", |b| b.iter(|| black_box(pdf_info(&pdf).unwrap())));
    group.bench_function("all_text", |b| {
        b.iter(|| black_box(pdf_all_text(&pdf).unwrap()))
    });
    group.bench_function("page_text_with_words", |b| {
        b.iter(|| black_box(pdf_page_text(&pdf, 250).unwrap()))
    });
    group.bench_function("render_one_page_at_1x", |b| {
        b.iter(|| black_box(pdf_render_page(&pdf, 250, 1.0).unwrap()))
    });
    let plan: Vec<PageSpec> = (0..PAGES)
        .rev()
        .map(|page| PageSpec {
            source: 0,
            page,
            rotate: 0,
        })
        .collect();
    group.bench_function("reverse_all_pages", |b| {
        b.iter(|| black_box(pdf_build(std::slice::from_ref(&pdf), &plan).unwrap()));
    });
    group.finish();
}

criterion_group!(pdf, benches);
criterion_main!(pdf);
