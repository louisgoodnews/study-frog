//! Pure export planning: resolves targets, document options, and complete block sequences.

use std::collections::HashMap;

use rusqlite::params;
use studyfrog_core::uid::Uid;
use studyfrog_md::blocks::{Block, Span, Style};
use studyfrog_md::ulid::format_ulid;

use crate::pdf_export_doc::convert_doc_to_blocks;
use crate::pdf_export_scope::resolve_target_scope;
use crate::pdf_export_types::{ExportTarget, PdfExportError, PdfExportOptions};
use crate::session::VaultSession;

/// A fully prepared export plan containing laid-out Markdown blocks and options.
#[derive(Debug, Clone, PartialEq)]
pub struct PdfExportPlan {
    /// Overall document title.
    pub title: String,
    /// Stack description (if available).
    pub description: Option<String>,
    /// Distinct item UIDs to be exported.
    pub items: Vec<Uid>,
    /// Markdown blocks ready for layout and PDF writing.
    pub blocks: Vec<Block>,
    /// Document layout options.
    pub options: PdfExportOptions,
    /// Non-fatal warnings accumulated during planning.
    pub warnings: Vec<String>,
}

fn build_title_lookup(
    session: &VaultSession,
    uids: &[Uid],
) -> Result<HashMap<String, String>, PdfExportError> {
    let mut map = HashMap::new();
    let sql = "SELECT title FROM item WHERE uid = ?1";
    let mut stmt = session
        .conn
        .prepare(sql)
        .map_err(|e| PdfExportError::Store(e.to_string()))?;
    for uid in uids {
        let id_str = format_ulid(*uid);
        if let Ok(title) = stmt.query_row(params![id_str], |r| r.get::<_, String>(0)) {
            map.insert(title.to_ascii_lowercase(), title.clone());
            map.insert(id_str, title);
        }
    }
    Ok(map)
}

fn title_page_blocks(
    title: &str,
    description: Option<&str>,
    item_count: usize,
    vault_name: Option<&str>,
) -> Vec<Block> {
    let mut blocks = Vec::new();
    blocks.push(Block::Heading {
        level: 1,
        spans: vec![Span {
            text: title.to_string(),
            style: Style {
                bold: true,
                ..Default::default()
            },
            link: None,
        }],
    });
    if let Some(desc) = description {
        blocks.push(Block::Paragraph(vec![Span {
            text: desc.to_string(),
            style: Style {
                italic: true,
                ..Default::default()
            },
            link: None,
        }]));
    }
    let vault_str = vault_name.unwrap_or("StudyFrog");
    let meta_str = format!("Total items: {item_count}  ·  Vault: {vault_str}");
    blocks.push(Block::Paragraph(vec![Span {
        text: meta_str,
        style: Style::default(),
        link: None,
    }]));
    blocks.push(Block::Rule);
    blocks
}

/// Create a complete, pure plan for exporting items to PDF.
pub fn plan_pdf_export(
    session: &VaultSession,
    target: &ExportTarget,
    options: &PdfExportOptions,
) -> Result<PdfExportPlan, PdfExportError> {
    let scope = resolve_target_scope(session, target)?;
    let title_lookup = build_title_lookup(session, &scope.uids)?;
    let mut warnings = Vec::new();
    let mut blocks = Vec::new();

    if options.title_page {
        let vault_name = session.root.file_name().and_then(|n| n.to_str());
        blocks.extend(title_page_blocks(
            &scope.title,
            scope.description.as_deref(),
            scope.uids.len(),
            vault_name,
        ));
    }

    if options.table_of_contents {
        blocks.push(Block::Toc);
    }

    for uid in &scope.uids {
        let item_blocks = convert_doc_to_blocks(
            session,
            *uid,
            &title_lookup,
            options.show_answers,
            &mut warnings,
        )?;
        blocks.extend(item_blocks);
    }

    Ok(PdfExportPlan {
        title: scope.title,
        description: scope.description,
        items: scope.uids,
        blocks,
        options: options.clone(),
        warnings,
    })
}
