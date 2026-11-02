//! The JSON interchange format, version 1 (`docs/specs/interchange.md`).

use serde_json::{Value, json};
use studyfrog_core::plan::{ItemKind, item_kind_from_name, item_kind_name};
use studyfrog_core::review::{Rating, rating_from_value, rating_value};
use studyfrog_md::ulid::{format_ulid, parse_ulid};

use crate::xfer_model::{Bundle, XItem, XReview, XStack};

/// The format name and version written.
pub const FORMAT: &str = "studyfrog-export";
/// The version this code writes and reads.
pub const VERSION: u64 = 1;

const RATINGS: [(&str, u8); 4] = [("again", 1), ("hard", 2), ("good", 3), ("easy", 4)];

fn rating_word(r: Rating) -> &'static str {
    RATINGS
        .iter()
        .find(|(_, v)| *v == rating_value(r))
        .map_or("good", |(w, _)| w)
}

fn item_json(i: &XItem, scheduling: bool) -> Value {
    let mut map = serde_json::Map::new();
    for (k, v) in [
        ("kind", item_kind_name(i.kind)),
        ("title", &i.title),
        ("stack", &i.stack),
        ("text", &i.text),
        ("answer", &i.answer),
    ] {
        map.insert(k.into(), json!(v));
    }
    map.insert("tags".into(), json!(i.tags));
    if let Some(id) = i.id {
        map.insert("id".into(), json!(format_ulid(id)));
    }
    for (key, value) in [
        ("status", &i.status),
        ("priority", &i.priority),
        ("difficulty", &i.difficulty),
    ] {
        if let Some(w) = value {
            map.insert(key.into(), json!(w));
        }
    }
    if scheduling {
        let reviews: Vec<Value> = i
            .reviews
            .iter()
            .map(|r| json!({"at": r.at_ms, "rating": rating_word(r.rating), "elapsed_days": r.elapsed_days}))
            .collect();
        map.insert("reviews".into(), Value::Array(reviews));
    }
    Value::Object(map)
}

/// The JSON text of a bundle.
pub fn to_json(b: &Bundle, exported: &str) -> String {
    let stacks: Vec<Value> = b
        .stacks
        .iter()
        .map(|s| json!({"folder": s.folder, "title": s.title, "description": s.description}))
        .collect();
    let items: Vec<Value> = b.items.iter().map(|i| item_json(i, b.scheduling)).collect();
    let doc = json!({"format": FORMAT, "version": VERSION, "exported": exported, "scheduling": b.scheduling, "stacks": stacks, "items": items});
    serde_json::to_string_pretty(&doc).unwrap_or_default()
}

fn text(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

fn optional(v: &Value, key: &str) -> Option<String> {
    v.get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
}

fn review_of(v: &Value) -> Option<XReview> {
    let word = v.get("rating")?.as_str()?;
    let value = RATINGS.iter().find(|(w, _)| *w == word)?.1;
    Some(XReview {
        at_ms: v.get("at")?.as_i64()?,
        rating: rating_from_value(value)?,
        elapsed_days: v.get("elapsed_days").and_then(Value::as_f64).unwrap_or(0.0) as f32,
    })
}

fn item_of(v: &Value, n: usize) -> Result<XItem, String> {
    let kind = item_kind_from_name(&text(v, "kind"))
        .ok_or_else(|| format!("item {n}: the kind must be note, card or question"))?;
    let id = match v.get("id").and_then(Value::as_str) {
        Some(s) => Some(parse_ulid(s).ok_or_else(|| format!("item {n}: \"{s}\" is not an id"))?),
        None => None,
    };
    let tags = v
        .get("tags")
        .and_then(Value::as_array)
        .map_or_else(Vec::new, |a| {
            a.iter()
                .filter_map(|t| t.as_str().map(str::to_owned))
                .collect()
        });
    let reviews = v
        .get("reviews")
        .and_then(Value::as_array)
        .map_or_else(Vec::new, |a| a.iter().filter_map(review_of).collect());
    Ok(XItem {
        id,
        kind,
        title: text(v, "title"),
        text: text(v, "text"),
        answer: text(v, "answer"),
        tags,
        stack: text(v, "stack"),
        status: optional(v, "status"),
        priority: optional(v, "priority"),
        difficulty: optional(v, "difficulty"),
        reviews,
        extra: Vec::new(),
    })
}

/// Read a JSON export. Unknown keys are ignored; an unknown version is refused.
pub fn from_json(source: &str) -> Result<Bundle, String> {
    let v: Value = serde_json::from_str(source).map_err(|e| format!("not valid JSON: {e}"))?;
    if v.get("format").and_then(Value::as_str) != Some(FORMAT) {
        return Err(
            "this is not a StudyFrog export (its \"format\" is missing or different)".to_owned(),
        );
    }
    match v.get("version").and_then(Value::as_u64) {
        Some(VERSION) => {}
        other => {
            return Err(format!(
                "this export has version {other:?}; this app reads version {VERSION}"
            ));
        }
    }
    let stacks = v
        .get("stacks")
        .and_then(Value::as_array)
        .map_or_else(Vec::new, |a| {
            a.iter()
                .map(|s| XStack {
                    folder: text(s, "folder"),
                    title: text(s, "title"),
                    description: text(s, "description"),
                })
                .collect()
        });
    let mut items = Vec::new();
    for (n, i) in v
        .get("items")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
    {
        items.push(item_of(i, n + 1)?);
    }
    let scheduling = v
        .get("scheduling")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    Ok(Bundle {
        stacks,
        items,
        scheduling,
        ..Bundle::default()
    })
}

/// The kind of an item as the CSV and the preview name it.
pub fn kind_word(k: ItemKind) -> &'static str {
    item_kind_name(k)
}
