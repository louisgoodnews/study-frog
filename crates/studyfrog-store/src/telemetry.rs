//! Persistence of cognitive telemetry (STORY-0176): one row per answer, read back by item and time
//! range. Times are Unix milliseconds.

use rusqlite::{Connection, params};
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::{format_ulid, parse_ulid};

use crate::error::{StoreError, sql_err};

/// One answer's telemetry.
#[derive(Debug, Clone, PartialEq)]
pub struct TelemetryPoint {
    /// The item answered.
    pub item: Uid,
    /// When the answer was submitted, Unix milliseconds.
    pub reviewed_at_ms: i64,
    /// Milliseconds before submission.
    pub latency_ms: u32,
    /// Milliseconds of active reading.
    pub dwell_ms: u32,
    /// SLM semantic score in `[0, 1]`, if any.
    pub semantic_score: Option<f32>,
    /// Words in the question and reference answer (or bytes; a size proxy).
    pub text_complexity: u32,
}

/// Store one answer's telemetry. A semantic score outside `[0, 1]` is refused.
pub fn record_telemetry(conn: &Connection, p: &TelemetryPoint) -> Result<(), StoreError> {
    if p.semantic_score.is_some_and(|s| !(0.0..=1.0).contains(&s)) {
        return Err(StoreError::Corrupt(
            "semantic score must be in [0, 1]".to_owned(),
        ));
    }
    conn.execute(
        "INSERT INTO review_telemetry (item_uid, reviewed_at, latency_ms, dwell_ms, semantic_score, text_complexity)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            format_ulid(p.item),
            p.reviewed_at_ms,
            p.latency_ms,
            p.dwell_ms,
            p.semantic_score.map(f64::from),
            p.text_complexity,
        ],
    )
    .map_err(sql_err)?;
    Ok(())
}

/// Telemetry for the given items within `[from_ms, to_ms]`, oldest first. Pass every item of a
/// stack to get the stack's series.
pub fn load_telemetry_for_items(
    conn: &Connection,
    items: &[Uid],
    from_ms: i64,
    to_ms: i64,
) -> Result<Vec<TelemetryPoint>, StoreError> {
    let mut out = Vec::new();
    let mut stmt = conn
        .prepare(
            "SELECT item_uid, reviewed_at, latency_ms, dwell_ms, semantic_score, text_complexity
             FROM review_telemetry WHERE item_uid = ?1 AND reviewed_at BETWEEN ?2 AND ?3
             ORDER BY reviewed_at",
        )
        .map_err(sql_err)?;
    for item in items {
        let rows = stmt
            .query_map(params![format_ulid(*item), from_ms, to_ms], row)
            .map_err(sql_err)?;
        for r in rows {
            out.push(decode(r.map_err(sql_err)?)?);
        }
    }
    out.sort_by_key(|p| p.reviewed_at_ms);
    Ok(out)
}

/// Every answer's telemetry within `[from_ms, to_ms]`, oldest first.
pub fn load_telemetry_range(
    conn: &Connection,
    from_ms: i64,
    to_ms: i64,
) -> Result<Vec<TelemetryPoint>, StoreError> {
    let mut stmt = conn
        .prepare(
            "SELECT item_uid, reviewed_at, latency_ms, dwell_ms, semantic_score, text_complexity
             FROM review_telemetry WHERE reviewed_at BETWEEN ?1 AND ?2 ORDER BY reviewed_at",
        )
        .map_err(sql_err)?;
    let rows = stmt
        .query_map(params![from_ms, to_ms], row)
        .map_err(sql_err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(sql_err)?;
    rows.into_iter().map(decode).collect()
}

type Raw = (String, i64, u32, u32, Option<f64>, u32);

fn row(r: &rusqlite::Row) -> rusqlite::Result<Raw> {
    Ok((
        r.get(0)?,
        r.get(1)?,
        r.get(2)?,
        r.get(3)?,
        r.get(4)?,
        r.get(5)?,
    ))
}

fn decode(raw: Raw) -> Result<TelemetryPoint, StoreError> {
    let (item, reviewed_at_ms, latency_ms, dwell_ms, semantic, text_complexity) = raw;
    let item = parse_ulid(&item)
        .ok_or_else(|| StoreError::Corrupt(format!("telemetry item id: {item:?}")))?;
    Ok(TelemetryPoint {
        item,
        reviewed_at_ms,
        latency_ms,
        dwell_ms,
        semantic_score: semantic.map(|s| s as f32),
        text_complexity,
    })
}
