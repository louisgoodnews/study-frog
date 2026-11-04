//! Converting between internal `Bundle` and standard interchange `ExportDoc` (TASK-0220).

use studyfrog_core::plan::{item_kind_from_name, item_kind_name};
use studyfrog_core::review::{Rating, rating_from_value, rating_value};
use studyfrog_md::ulid::{format_ulid, parse_ulid};

use crate::xfer_doc::{
    ExportAverage, ExportDoc, ExportItem, ExportReview, ExportStack, ExportStackMemory, FORMAT,
    VERSION,
};
use crate::xfer_model::{Bundle, XAverage, XItem, XReview, XStack, XStackMemory};

const RATINGS: [(&str, u8); 4] = [("again", 1), ("hard", 2), ("good", 3), ("easy", 4)];

fn rating_word(r: Rating) -> &'static str {
    RATINGS
        .iter()
        .find(|(_, v)| *v == rating_value(r))
        .map_or("good", |(w, _)| w)
}

fn rating_from_word(w: &str) -> Option<Rating> {
    RATINGS
        .iter()
        .find(|(word, _)| *word == w)
        .and_then(|(_, v)| rating_from_value(*v))
}

fn item_to_export(i: &XItem, scheduling: bool) -> ExportItem {
    let (average, stacks, reviews) = if scheduling {
        let avg = i.average.as_ref().map(|a| ExportAverage {
            familiarity: a.familiarity,
            difficulty: a.difficulty,
        });
        let mems = i
            .stacks
            .iter()
            .map(|s| ExportStackMemory {
                stack: s.stack.clone(),
                source: s.source.clone(),
                difficulty: s.difficulty,
                familiarity: s.familiarity,
                due: s.due,
            })
            .collect();
        let revs = i
            .reviews
            .iter()
            .map(|r| ExportReview {
                at: r.at_ms,
                rating: rating_word(r.rating).to_owned(),
                elapsed_days: r.elapsed_days,
            })
            .collect();
        (avg, mems, revs)
    } else {
        (None, Vec::new(), Vec::new())
    };

    ExportItem {
        kind: item_kind_name(i.kind).to_owned(),
        title: i.title.clone(),
        stack: i.stack.clone(),
        text: i.text.clone(),
        answer: i.answer.clone(),
        tags: i.tags.clone(),
        also_in: i.also_in.clone(),
        id: i.id.map(format_ulid),
        status: i.status.clone(),
        priority: i.priority.clone(),
        difficulty: i.difficulty.clone(),
        average,
        stacks,
        reviews,
    }
}

/// Build an `ExportDoc` from a `Bundle`.
pub fn export_doc_from_bundle(b: &Bundle, exported: &str) -> ExportDoc {
    let stacks = b
        .stacks
        .iter()
        .map(|s| ExportStack {
            folder: s.folder.clone(),
            title: s.title.clone(),
            description: s.description.clone(),
        })
        .collect();

    let items = b
        .items
        .iter()
        .map(|i| item_to_export(i, b.scheduling))
        .collect();

    ExportDoc {
        format: FORMAT.to_owned(),
        version: VERSION,
        exported: exported.to_owned(),
        scheduling: b.scheduling,
        stacks,
        items,
    }
}

fn item_from_export(n: usize, i: ExportItem) -> Result<XItem, String> {
    let kind = item_kind_from_name(&i.kind)
        .ok_or_else(|| format!("item {}: the kind must be note, card or question", n + 1))?;
    let id = match i.id.as_deref() {
        Some(s) => {
            Some(parse_ulid(s).ok_or_else(|| format!("item {}: \"{s}\" is not an id", n + 1))?)
        }
        None => None,
    };
    let reviews = i
        .reviews
        .into_iter()
        .filter_map(|r| {
            rating_from_word(&r.rating).map(|rating| XReview {
                at_ms: r.at,
                rating,
                elapsed_days: r.elapsed_days,
            })
        })
        .collect();
    let stacks = i
        .stacks
        .into_iter()
        .map(|s| XStackMemory {
            stack: s.stack,
            source: s.source,
            difficulty: s.difficulty,
            familiarity: s.familiarity,
            due: s.due,
        })
        .collect();
    let average = i.average.map(|a| XAverage {
        familiarity: a.familiarity,
        difficulty: a.difficulty,
    });

    Ok(XItem {
        id,
        kind,
        title: i.title,
        text: i.text,
        answer: i.answer,
        tags: i.tags,
        stack: i.stack,
        also_in: i.also_in,
        status: i.status,
        priority: i.priority,
        difficulty: i.difficulty,
        reviews,
        stacks,
        average,
        extra: Vec::new(),
    })
}

/// Reconstruct a `Bundle` from an `ExportDoc`.
pub fn bundle_from_export_doc(doc: ExportDoc) -> Result<Bundle, String> {
    if doc.format != FORMAT {
        return Err(
            "this is not a StudyFrog export (its \"format\" is missing or different)".to_owned(),
        );
    }
    if doc.version != VERSION {
        return Err(format!(
            "this export has version {}; this app reads version {VERSION}",
            doc.version
        ));
    }

    let stacks = doc
        .stacks
        .into_iter()
        .map(|s| XStack {
            folder: s.folder,
            title: s.title,
            description: s.description,
        })
        .collect();

    let mut items = Vec::new();
    for (n, i) in doc.items.into_iter().enumerate() {
        items.push(item_from_export(n, i)?);
    }

    Ok(Bundle {
        stacks,
        items,
        scheduling: doc.scheduling,
        ..Bundle::default()
    })
}
