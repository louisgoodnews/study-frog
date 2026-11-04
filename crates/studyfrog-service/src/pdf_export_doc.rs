//! Conversion of vault documents (`Doc`) into Markdown blocks with resolved wikilinks and images.

use std::collections::HashMap;
use std::path::Path;

use studyfrog_core::uid::Uid;
use studyfrog_md::blocks::{Block, Link, Span, Style, parse_blocks};
use studyfrog_md::doc::{Doc, doc_meta};
use studyfrog_md::ulid::format_ulid;
use studyfrog_store::overview::resolve_title;

use crate::document::load_document;
use crate::pdf_export_types::PdfExportError;
use crate::session::VaultSession;

fn is_safe_image_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.starts_with('\\')
        && !path.contains('\0')
        && !path.contains(':')
        && path
            .split('/')
            .all(|p| !p.is_empty() && p != "." && p != ".." && !p.chars().any(char::is_control))
}

fn resolve_image_file(root: &Path, img_path: &str) -> Option<String> {
    if !is_safe_image_path(img_path) {
        return None;
    }
    let direct = root.join(img_path);
    if direct.is_file() {
        return Some(img_path.to_string());
    }
    if !img_path.starts_with("assets/") {
        let in_assets = root.join("assets").join(img_path);
        if in_assets.is_file() {
            return Some(format!("assets/{img_path}"));
        }
    }
    None
}

fn resolve_span(
    span: &mut Span,
    session: &VaultSession,
    in_export_titles: &HashMap<String, String>,
    warnings: &mut Vec<String>,
) {
    if let Some(ref link) = span.link.clone() {
        match link {
            Link::Wiki(target) => {
                let target_key = target.trim().to_ascii_lowercase();
                if let Some(canonical) = in_export_titles.get(&target_key) {
                    span.text = canonical.clone();
                    span.link = Some(Link::Wiki(canonical.clone()));
                } else if let Ok(Some(uid)) = resolve_title(&session.conn, target) {
                    let id_str = format_ulid(uid);
                    if let Some(canonical) = in_export_titles.get(&id_str) {
                        span.text = canonical.clone();
                        span.link = Some(Link::Wiki(canonical.clone()));
                    } else {
                        let sql = "SELECT title FROM item WHERE uid = ?1";
                        if let Ok(title) = session
                            .conn
                            .query_row(sql, [id_str], |r| r.get::<_, String>(0))
                        {
                            span.text = title;
                        }
                        span.link = None;
                    }
                } else {
                    span.link = None;
                }
            }
            Link::Image(path) => {
                if let Some(found_rel) = resolve_image_file(&session.root, path) {
                    span.link = Some(Link::Image(found_rel));
                } else {
                    let alt = if span.text.is_empty() {
                        path.as_str()
                    } else {
                        span.text.as_str()
                    };
                    warnings.push(format!("Missing picture: {path} ({alt})"));
                    span.text = format!("[Missing picture: {alt}]");
                    span.style.italic = true;
                    span.link = None;
                }
            }
            Link::Url(_) => {}
        }
    }
}

fn resolve_spans_in_block(
    block: &mut Block,
    session: &VaultSession,
    in_export_titles: &HashMap<String, String>,
    warnings: &mut Vec<String>,
) {
    match block {
        Block::Heading { spans, .. } | Block::Paragraph(spans) => {
            for span in spans {
                resolve_span(span, session, in_export_titles, warnings);
            }
        }
        Block::Quote(children) => {
            for child in children {
                resolve_spans_in_block(child, session, in_export_titles, warnings);
            }
        }
        Block::List { items, .. } => {
            for item in items {
                for b in &mut item.blocks {
                    resolve_spans_in_block(b, session, in_export_titles, warnings);
                }
            }
        }
        Block::Table { header, rows, .. } => {
            for cell in header {
                for span in cell {
                    resolve_span(span, session, in_export_titles, warnings);
                }
            }
            for row in rows {
                for cell in row {
                    for span in cell {
                        resolve_span(span, session, in_export_titles, warnings);
                    }
                }
            }
        }
        Block::Code { .. } | Block::Rule | Block::Toc => {}
    }
}

fn doc_content_blocks(doc: &Doc, show_answers: bool) -> Vec<Block> {
    match doc {
        Doc::Note(d) => parse_blocks(&d.body),
        Doc::Stack(d) => parse_blocks(&d.body),
        Doc::Diary(d) => parse_blocks(&d.body),
        Doc::Card(d) => {
            let mut blocks = parse_blocks(&d.front);
            if show_answers && !d.back.trim().is_empty() {
                blocks.push(Block::Rule);
                blocks.extend(parse_blocks(&d.back));
            }
            blocks
        }
        Doc::Question(d) => {
            let mut blocks = parse_blocks(&d.prompt);
            if show_answers && !d.reference.trim().is_empty() {
                blocks.push(Block::Rule);
                blocks.extend(parse_blocks(&d.reference));
            }
            blocks
        }
    }
}

/// Convert a loaded vault document into laid-out blocks with links and pictures resolved.
pub fn convert_doc_to_blocks(
    session: &VaultSession,
    uid: Uid,
    in_export_titles: &HashMap<String, String>,
    show_answers: bool,
    warnings: &mut Vec<String>,
) -> Result<Vec<Block>, PdfExportError> {
    let loaded = load_document(session, uid).map_err(|e| {
        PdfExportError::Io(format!("could not load item {}: {e}", format_ulid(uid)))
    })?;
    let title = doc_meta(&loaded.doc).title.clone();

    let mut item_blocks = Vec::new();
    item_blocks.push(Block::Heading {
        level: 1,
        spans: vec![Span {
            text: title,
            style: Style {
                bold: true,
                ..Default::default()
            },
            link: None,
        }],
    });

    let mut body = doc_content_blocks(&loaded.doc, show_answers);
    for b in &mut body {
        resolve_spans_in_block(b, session, in_export_titles, warnings);
    }
    item_blocks.extend(body);

    Ok(item_blocks)
}
