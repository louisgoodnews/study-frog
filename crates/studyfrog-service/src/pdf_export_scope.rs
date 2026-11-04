//! Resolves an export target into an ordered, deduplicated sequence of item UIDs and metadata.

use std::collections::HashSet;

use rusqlite::params;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::{format_ulid, parse_ulid};

use crate::pdf_export_types::{ExportTarget, PdfExportError};
use crate::session::VaultSession;

/// The resolved scope of an export: document title, optional description, and ordered item UIDs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetScope {
    /// Document or stack title.
    pub title: String,
    /// Stack description (from `_stack.md` if available).
    pub description: Option<String>,
    /// Distinct item UIDs in deterministic order.
    pub uids: Vec<Uid>,
}

fn err_sql(e: rusqlite::Error) -> PdfExportError {
    PdfExportError::Store(e.to_string())
}

fn stack_info(
    session: &VaultSession,
    folder: &str,
) -> Result<(String, Option<String>), PdfExportError> {
    if folder.is_empty() {
        let root_name = session
            .root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Vault");
        return Ok((format!("{root_name} Export"), None));
    }
    let sql = "SELECT title, COALESCE(body, '') FROM item WHERE kind = 'stack' AND stack_path = ?1";
    let mut stmt = session.conn.prepare(sql).map_err(err_sql)?;
    let mut rows = stmt.query(params![folder]).map_err(err_sql)?;
    if let Some(row) = rows.next().map_err(err_sql)? {
        let title: String = row.get(0).map_err(err_sql)?;
        let body: String = row.get(1).map_err(err_sql)?;
        let desc = if body.trim().is_empty() {
            None
        } else {
            Some(body.trim().to_string())
        };
        Ok((title, desc))
    } else {
        let title = folder.rsplit('/').next().unwrap_or(folder);
        Ok((title.to_string(), None))
    }
}

fn collect_stack_uids(session: &VaultSession, folder: &str) -> Result<Vec<Uid>, PdfExportError> {
    let (primary_sql, is_root) = if folder.is_empty() {
        (
            "SELECT uid, path FROM item WHERE kind <> 'stack' ORDER BY path, uid",
            true,
        )
    } else {
        (
            "SELECT uid, path FROM item WHERE kind <> 'stack' AND (stack_path = ?1 OR stack_path LIKE ?2) ORDER BY path, uid",
            false,
        )
    };

    let mut primary_uids = Vec::new();
    let prefix = format!("{folder}/%");
    let mut stmt = session.conn.prepare(primary_sql).map_err(err_sql)?;
    let mut rows = if is_root {
        stmt.query([]).map_err(err_sql)?
    } else {
        stmt.query(params![folder, prefix]).map_err(err_sql)?
    };

    while let Some(row) = rows.next().map_err(err_sql)? {
        let id_str: String = row.get(0).map_err(err_sql)?;
        if let Some(uid) = parse_ulid(&id_str) {
            primary_uids.push(uid);
        }
    }

    let mut referenced_uids = Vec::new();
    let ref_sql = if is_root {
        "SELECT DISTINCT m.item_uid, i.path FROM membership m JOIN item i ON i.uid = m.item_uid WHERE m.is_primary = 0 ORDER BY i.path, m.item_uid"
    } else {
        "SELECT DISTINCT m.item_uid, i.path FROM membership m JOIN item i ON i.uid = m.item_uid
         WHERE m.is_primary = 0 AND m.stack_uid IN (
             SELECT uid FROM stack_identity WHERE folder = ?1 OR folder LIKE ?2
         ) ORDER BY i.path, m.item_uid"
    };

    let mut ref_stmt = session.conn.prepare(ref_sql).map_err(err_sql)?;
    let mut ref_rows = if is_root {
        ref_stmt.query([]).map_err(err_sql)?
    } else {
        ref_stmt.query(params![folder, prefix]).map_err(err_sql)?
    };

    while let Some(row) = ref_rows.next().map_err(err_sql)? {
        let id_str: String = row.get(0).map_err(err_sql)?;
        if let Some(uid) = parse_ulid(&id_str) {
            referenced_uids.push(uid);
        }
    }

    let mut seen = HashSet::new();
    let mut combined = Vec::new();
    for uid in primary_uids.into_iter().chain(referenced_uids) {
        if seen.insert(uid) {
            combined.push(uid);
        }
    }

    Ok(combined)
}

/// Resolve any [`ExportTarget`] into its complete ordered items and metadata.
pub fn resolve_target_scope(
    session: &VaultSession,
    target: &ExportTarget,
) -> Result<TargetScope, PdfExportError> {
    match target {
        ExportTarget::SingleItem(uid) => {
            let sql = "SELECT title FROM item WHERE uid = ?1";
            let mut stmt = session.conn.prepare(sql).map_err(err_sql)?;
            let id_str = format_ulid(*uid);
            let mut rows = stmt.query(params![id_str]).map_err(err_sql)?;
            if let Some(row) = rows.next().map_err(err_sql)? {
                let title: String = row.get(0).map_err(err_sql)?;
                Ok(TargetScope {
                    title,
                    description: None,
                    uids: vec![*uid],
                })
            } else {
                Err(PdfExportError::NotFound(format!("item {id_str}")))
            }
        }
        ExportTarget::Selection(uids) => {
            let mut seen = HashSet::new();
            let mut deduped = Vec::new();
            for uid in uids {
                if seen.insert(*uid) {
                    let sql = "SELECT 1 FROM item WHERE uid = ?1";
                    let mut stmt = session.conn.prepare(sql).map_err(err_sql)?;
                    let id_str = format_ulid(*uid);
                    if stmt.exists(params![id_str]).map_err(err_sql)? {
                        deduped.push(*uid);
                    }
                }
            }
            if deduped.is_empty() {
                return Err(PdfExportError::EmptyExport);
            }
            let title = if deduped.len() == 1 {
                let sql = "SELECT title FROM item WHERE uid = ?1";
                let mut stmt = session.conn.prepare(sql).map_err(err_sql)?;
                let id_str = format_ulid(deduped[0]);
                stmt.query_row(params![id_str], |r| r.get(0))
                    .unwrap_or_else(|_| "Selection".to_string())
            } else {
                "Selected Notes".to_string()
            };
            Ok(TargetScope {
                title,
                description: None,
                uids: deduped,
            })
        }
        ExportTarget::Stack(folder) => {
            let (title, description) = stack_info(session, folder)?;
            let uids = collect_stack_uids(session, folder)?;
            if uids.is_empty() {
                return Err(PdfExportError::EmptyExport);
            }
            Ok(TargetScope {
                title,
                description,
                uids,
            })
        }
    }
}
