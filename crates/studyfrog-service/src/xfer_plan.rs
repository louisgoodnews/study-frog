//! The dry run of an import: what would be created, what exists already and what fails. Nothing is written.

use std::collections::HashMap;

use studyfrog_core::plan::ItemKind;
use studyfrog_core::uid::Uid;
use studyfrog_md::doc::Doc;
use studyfrog_store::item_meta::load_item_meta;
use studyfrog_store::stackmap::STACK_FILE;

use crate::create::default_card_title;
use crate::document::load_document;
use crate::session::{ServiceError, VaultSession};
use crate::xfer_model::{Bundle, XItem, safe_folder};

/// Most items one import may write.
pub const MAX_ITEMS: usize = 20_000;

/// What to do with an item that already exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Policy {
    /// Leave the existing one.
    Skip,
    /// Overwrite the existing one, keeping its path and id.
    Replace,
    /// Write the new one beside it with a new id.
    KeepBoth,
}

/// What a dry run says about one item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// It would be written.
    Create,
    /// It exists already, at this path.
    Duplicate(String),
    /// It cannot be written, and why.
    Error(String),
}

/// One item of the plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanLine {
    /// The item's title (or the start of its text).
    pub title: String,
    /// The folder it would go into.
    pub folder: String,
    /// What would happen.
    pub verdict: Verdict,
}

/// The dry run.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ImportPlan {
    /// One line per item, in order.
    pub lines: Vec<PlanLine>,
    /// Stacks that would be created.
    pub new_stacks: usize,
    /// What the reader said about the file.
    pub warnings: Vec<String>,
}

/// How many items the plan would create, find already there, and refuse.
pub fn counts(plan: &ImportPlan) -> (usize, usize, usize) {
    let n = |f: fn(&Verdict) -> bool| plan.lines.iter().filter(|l| f(&l.verdict)).count();
    (
        n(|v| matches!(v, Verdict::Create)),
        n(|v| matches!(v, Verdict::Duplicate(_))),
        n(|v| matches!(v, Verdict::Error(_))),
    )
}

fn destination(item: &XItem, target: Option<&str>) -> Result<String, String> {
    let folder = match (target, item.stack.trim()) {
        (Some(t), "") => t.to_owned(),
        (Some(t), s) => format!("{t}/{s}"),
        (None, "") => return Err("it names no stack; choose one to import into".to_owned()),
        (None, s) => s.to_owned(),
    };
    if safe_folder(&folder) {
        Ok(folder)
    } else {
        Err(format!("\"{folder}\" is not a usable stack folder"))
    }
}

pub(crate) fn label(item: &XItem) -> String {
    if item.title.trim().is_empty() {
        default_card_title(&item.text)
    } else {
        item.title.trim().to_owned()
    }
}

/// An item already in the vault: its path and uid.
pub(crate) type Existing = (Uid, String);

pub(crate) struct Known {
    pub(crate) by_id: HashMap<Uid, String>,
    pub(crate) by_text: HashMap<(String, String, String), Existing>,
}

pub(crate) fn known(session: &VaultSession) -> Result<Known, ServiceError> {
    let mut k = Known {
        by_id: HashMap::new(),
        by_text: HashMap::new(),
    };
    for (uid, m) in load_item_meta(&session.conn).map_err(ServiceError::Store)? {
        k.by_id.insert(uid, m.path.clone());
        let Some(folder) = m.stack.clone() else {
            continue;
        };
        if let Ok(loaded) = load_document(session, uid) {
            let (a, b) = match &loaded.doc {
                Doc::Card(d) => (d.front.clone(), d.back.clone()),
                Doc::Question(d) => (d.prompt.clone(), d.reference.clone()),
                _ => continue,
            };
            k.by_text.insert(
                (folder, a.trim().to_owned(), b.trim().to_owned()),
                (uid, m.path),
            );
        }
    }
    Ok(k)
}

pub(crate) fn find(k: &Known, item: &XItem, folder: &str) -> Option<Existing> {
    if let Some(path) = item
        .id
        .and_then(|id| k.by_id.get(&id).map(|p| (id, p.clone())))
    {
        return Some(path);
    }
    (item.kind != ItemKind::Note)
        .then(|| {
            k.by_text
                .get(&(
                    folder.to_owned(),
                    item.text.trim().to_owned(),
                    item.answer.trim().to_owned(),
                ))
                .cloned()
        })
        .flatten()
}

fn missing_stacks(session: &VaultSession, folders: &[String]) -> usize {
    let mut seen: Vec<String> = Vec::new();
    for f in folders {
        let mut upto = String::new();
        for part in f.split('/') {
            upto = if upto.is_empty() {
                part.to_owned()
            } else {
                format!("{upto}/{part}")
            };
            if !session.root.join(&upto).join(STACK_FILE).is_file() && !seen.contains(&upto) {
                seen.push(upto.clone());
            }
        }
    }
    seen.len()
}

/// The dry run: what the import would create, what exists already and what fails. Nothing is written.
pub fn plan_import(
    session: &VaultSession,
    b: &Bundle,
    target: Option<&str>,
) -> Result<ImportPlan, ServiceError> {
    let k = known(session)?;
    let mut plan = ImportPlan {
        warnings: b.warnings.clone(),
        ..ImportPlan::default()
    };
    let mut folders = Vec::new();
    if b.items.len() > MAX_ITEMS {
        return Err(ServiceError::Invalid(format!(
            "the file holds {} items; one import is limited to {MAX_ITEMS}",
            b.items.len()
        )));
    }
    for item in &b.items {
        let (folder, verdict) = match destination(item, target) {
            Err(e) => (String::new(), Verdict::Error(e)),
            Ok(f) if crate::create::check_title(&label(item)).is_err() => (
                f,
                Verdict::Error("its title must be one line without control characters".to_owned()),
            ),
            Ok(f) if item.text.trim().is_empty() => {
                (f, Verdict::Error("it has no text".to_owned()))
            }
            Ok(f) if item.kind != ItemKind::Note && item.answer.trim().is_empty() => {
                (f, Verdict::Error("it has no back".to_owned()))
            }
            Ok(f) => {
                folders.push(f.clone());
                let v = find(&k, item, &f).map_or(Verdict::Create, |(_, p)| Verdict::Duplicate(p));
                (f, v)
            }
        };
        plan.lines.push(PlanLine {
            title: label(item),
            folder,
            verdict,
        });
    }
    plan.new_stacks = missing_stacks(session, &folders);
    Ok(plan)
}
