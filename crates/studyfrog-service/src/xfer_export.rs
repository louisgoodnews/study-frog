//! Export: the library or one stack as JSON, CSV or Markdown (`docs/specs/interchange.md`).

use std::fs;
use std::path::Path;

use studyfrog_core::plan::ItemKind;
use studyfrog_md::doc::{Doc, doc_meta};
use studyfrog_md::extra::extra_value;
use studyfrog_store::item_meta::load_item_meta;
use studyfrog_store::overview::load_overview;
use studyfrog_store::reviews::load_reviews;

use crate::document::load_document;
use crate::session::{ServiceError, VaultSession};
use crate::xfer_model::{Bundle, XAverage, XItem, XReview, XStack, XStackMemory};
use crate::xfer_zip::write_zip;

const FILE_EXTS: [&str; 5] = ["png", "jpg", "jpeg", "gif", "pdf"];
const MAX_FILES: usize = 2_000;

type SchedData<'a> = (&'a [XReview], Vec<XStackMemory>, Option<XAverage>);

fn in_scope(folder: &str, scope: Option<&str>) -> bool {
    scope.is_none_or(|s| folder == s || folder.starts_with(&format!("{s}/")))
}

fn item_of(
    doc: &Doc,
    (stack, also_in): (&str, Vec<String>),
    scheduling: Option<SchedData<'_>>,
) -> XItem {
    let m = doc_meta(doc);
    let (kind, text, answer) = match doc {
        Doc::Card(d) => (ItemKind::Card, d.front.clone(), d.back.clone()),
        Doc::Question(d) => (ItemKind::Question, d.prompt.clone(), d.reference.clone()),
        Doc::Note(d) => (ItemKind::Note, d.body.clone(), String::new()),
        Doc::Stack(d) => (ItemKind::Note, d.body.clone(), String::new()),
        Doc::Diary(d) => (ItemKind::Note, d.body.clone(), String::new()),
    };
    let (reviews, stacks, average) = match scheduling {
        Some((r, s, a)) => (r.to_vec(), s, a),
        None => (Vec::new(), Vec::new(), None),
    };
    XItem {
        id: Some(m.uid),
        kind,
        title: m.title.clone(),
        text,
        answer,
        tags: m.tags.clone(),
        stack: stack.to_owned(),
        also_in,
        status: extra_value(doc, "status"),
        priority: extra_value(doc, "priority"),
        difficulty: extra_value(doc, "difficulty"),
        reviews,
        stacks,
        average,
        extra: Vec::new(),
    }
}

fn files_under(root: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>)>) {
    let Ok(read) = fs::read_dir(dir) else { return };
    for entry in read.flatten() {
        let path = entry.path();
        if out.len() >= MAX_FILES || entry.file_name().to_string_lossy().starts_with('.') {
            continue;
        }
        if path.is_dir() {
            files_under(root, &path, out);
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| FILE_EXTS.contains(&e.to_lowercase().as_str()))
        {
            let rel = path
                .strip_prefix(root)
                .map(|p| p.to_string_lossy().replace('\\', "/"));
            if let (Ok(rel), Ok(bytes)) = (rel, fs::read(&path)) {
                out.push((rel, bytes));
            }
        }
    }
}

/// The bundle of the whole library (`scope` `None`) or of the stack at `scope` with the stacks inside it.
pub fn export_bundle(
    session: &VaultSession,
    scope: Option<&str>,
    scheduling: bool,
) -> Result<Bundle, ServiceError> {
    let store = ServiceError::Store;
    let mut b = Bundle {
        scheduling,
        ..Bundle::default()
    };
    let overview = load_overview(&session.conn).map_err(store)?;
    for s in overview
        .stacks
        .iter()
        .filter(|s| in_scope(&s.folder, scope))
    {
        let description = match load_document(session, s.uid)?.doc {
            Doc::Stack(d) if d.description.trim().is_empty() => d.body.trim().to_owned(),
            Doc::Stack(d) => d.description,
            _ => String::new(),
        };
        b.stacks.push(XStack {
            folder: s.folder.clone(),
            title: s.title.clone(),
            description,
        });
    }
    let per_stack = if scheduling {
        let now = studyfrog_core::time::UnixMs(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_millis() as i64),
        );
        let scores = crate::fam_refs::reference_scores(&session.conn, now).unwrap_or_default();
        crate::fam_refs::views_by_item(&scores, &overview.stacks)
    } else {
        std::collections::HashMap::new()
    };
    let mut metas: Vec<_> = load_item_meta(&session.conn)
        .map_err(ServiceError::Store)?
        .into_iter()
        .collect();
    metas.sort_by(|a, b| a.1.path.cmp(&b.1.path));
    for (uid, m) in metas {
        let Some(folder) = m.stack.as_deref().filter(|f| in_scope(f, scope)) else {
            continue;
        };
        let also_in: Vec<String> = m
            .also_in
            .iter()
            .filter(|f| in_scope(f, scope))
            .cloned()
            .collect();
        let sched = scheduling.then(|| {
            let revs: Vec<XReview> = load_reviews(&session.conn, uid)
                .unwrap_or_default()
                .iter()
                .map(|r| XReview {
                    at_ms: r.at.0,
                    rating: r.rating,
                    elapsed_days: r.elapsed_days,
                })
                .collect();
            let (mems, avg) = match per_stack.get(&uid) {
                Some(v_list) => {
                    let (fam, diff) = crate::fam_refs::overall(v_list);
                    let avg = (fam.is_some() || diff.is_some()).then_some(XAverage {
                        familiarity: fam,
                        difficulty: diff,
                    });
                    let mems = v_list
                        .iter()
                        .map(|v| XStackMemory {
                            stack: v.folder.clone().unwrap_or_else(|| "Inbox".to_owned()),
                            source: if v.primary {
                                "own".into()
                            } else {
                                "referenced".into()
                            },
                            difficulty: v.difficulty,
                            familiarity: v.value,
                            due: v.due_ms,
                        })
                        .collect();
                    (mems, avg)
                }
                None => (Vec::new(), None),
            };
            (revs, mems, avg)
        });
        let sched_param = sched
            .as_ref()
            .map(|(r, s, a)| (r.as_slice(), s.clone(), a.clone()));
        b.items.push(item_of(
            &load_document(session, uid)?.doc,
            (folder, also_in),
            sched_param,
        ));
    }
    let base = scope.map_or_else(|| session.root.clone(), |s| session.root.join(s));
    files_under(&session.root, &base, &mut b.files);
    Ok(b)
}

/// The zip of a JSON export: `export.json` and the files under `files/`.
pub fn export_json_zip(b: &Bundle, exported: &str) -> Result<Vec<u8>, ServiceError> {
    let mut entries = vec![(
        "export.json".to_owned(),
        crate::xfer_json::to_json(b, exported).into_bytes(),
    )];
    entries.extend(
        b.files
            .iter()
            .map(|(p, d)| (format!("files/{p}"), d.clone())),
    );
    write_zip(&entries).map_err(ServiceError::Invalid)
}

/// The zip of a Markdown export: the vault's own files of the scope, with the pictures and PDFs.
pub fn export_markdown_zip(
    session: &VaultSession,
    scope: Option<&str>,
) -> Result<Vec<u8>, ServiceError> {
    let mut entries = Vec::new();
    for (_, m) in load_item_meta(&session.conn).map_err(ServiceError::Store)? {
        if m.stack.as_deref().is_some_and(|f| in_scope(f, scope)) {
            entries.push((
                m.path.clone(),
                fs::read(session.root.join(&m.path)).unwrap_or_default(),
            ));
        }
    }
    for s in load_overview(&session.conn)
        .map_err(ServiceError::Store)?
        .stacks
        .iter()
        .filter(|s| in_scope(&s.folder, scope))
    {
        let path = format!("{}/_stack.md", s.folder);
        entries.push((
            path.clone(),
            fs::read(session.root.join(path)).unwrap_or_default(),
        ));
    }
    entries.sort();
    entries.extend(export_bundle(session, scope, false)?.files);
    write_zip(&entries).map_err(ServiceError::Invalid)
}
