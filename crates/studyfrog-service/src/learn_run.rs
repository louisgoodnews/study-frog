//! Custom learn-mode runs (STORY-0198): start a run of a stored mode, and walk its graph with each rating. A
//! custom run is practice-only in v1 (it writes no reviews and changes no schedule); its steps are logged apart
//! from the built-in modes (`docs/research/learn-mode-proposal.md`).

use std::collections::BTreeMap;

use rusqlite::Connection;
use studyfrog_core::plan::{RunPlan, ScheduleEffect};
use studyfrog_core::review::Rating;
use studyfrog_core::run::{Run, RunMode};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;
use studyfrog_rehearsal::learn_graph::{LearnGraph, NodeKind, check, reason};
use studyfrog_rehearsal::learn_interp::{End, Visit, walk};
use studyfrog_rehearsal::learn_path::{Back, gap, rating_word};
use studyfrog_store::error::StoreError;
use studyfrog_store::learn::{
    StepRow, insert_learn_run, insert_learn_step, item_path, learn_run_of, next_learn_seq,
    set_last_rating,
};

use crate::create::IdSource;
use crate::day::DayWindow;
use crate::learn_mode_store as store;
use crate::rehearse::start_run;
use crate::session::{ServiceError, VaultSession};

/// How many times one item may enter one node in a custom run: the stop rule of a loop.
pub const VISIT_CAP: u32 = 3;

/// Start a practice-only run of the stored mode `file` (its name in `learn_modes`). The graph is checked
/// first; the run keeps a snapshot of the file's text, and its hash.
pub fn start(
    session: &VaultSession,
    file: &str,
    plan: RunPlan,
    now: UnixMs,
    day: DayWindow,
    ids: IdSource,
) -> Result<Run, ServiceError> {
    let text = store::read_text(&session.root, file).map_err(ServiceError::Invalid)?;
    let graph = store::from_toml(&text).map_err(ServiceError::Invalid)?;
    let refusals = check(&graph);
    if !refusals.is_empty() {
        let why: Vec<String> = refusals.iter().map(reason).collect();
        return Err(ServiceError::Invalid(format!(
            "not run: {}",
            why.join("; ")
        )));
    }
    let plan = RunPlan {
        effect: ScheduleEffect::PracticeOnly,
        ..plan
    };
    let run = start_run(session, plan, RunMode::Flip, now, day, ids)?;
    let run_id = format_ulid(run.uid);
    let mode_id = file.trim_end_matches(".toml");
    let version = i64::from(store::stored_version(&session.root, file));
    if let Err(e) = insert_learn_run(
        &session.conn,
        &run_id,
        mode_id,
        version,
        &store::graph_hash(&text),
        &text,
    ) {
        // Nothing has been done in the run yet: take it back, so no built-in history holds a custom run.
        let _ = session
            .conn
            .execute("DELETE FROM run WHERE uid = ?1", [&run_id]);
        return Err(ServiceError::Store(e));
    }
    Ok(run)
}

/// Walk the graph of custom run `run` for `item` after it was rated `rating`, and store the steps taken. Does
/// nothing for a built-in run. Called inside the transaction that stores the rating.
pub fn record_rating(
    conn: &Connection,
    run: Uid,
    item: Uid,
    rating: Rating,
    now: UnixMs,
) -> Result<Option<Back>, StoreError> {
    let run_id = format_ulid(run);
    let Some((_, text)) = learn_run_of(conn, &run_id)? else {
        return Ok(None);
    };
    let graph = store::from_toml(&text).map_err(StoreError::Corrupt)?;
    let item_id = format_ulid(item);
    let path = item_path(conn, &run_id, &item_id)?;
    let mut counts: BTreeMap<String, u32> = BTreeMap::new();
    for (node, _, _) in &path {
        *counts.entry(node.clone()).or_insert(0) += 1;
    }
    let at = match path.last() {
        Some((node, _, _)) => node.clone(),
        None => {
            // The item's first rating: first walk from the start to the rate node it waits at.
            let start = graph
                .nodes
                .iter()
                .find(|n| n.kind == NodeKind::Start)
                .map(|n| n.id.clone())
                .ok_or_else(|| StoreError::Corrupt("the learn mode has no start".to_owned()))?;
            let first = walk(&graph, &start, "", &mut counts, VISIT_CAP);
            append(conn, &graph, &run_id, &item_id, &first.visits, now)?;
            match first.end {
                End::AtRate(node) => node,
                _ => return Ok(Some(Back::Out)),
            }
        }
    };
    let word = rating_word(rating);
    set_last_rating(conn, &run_id, &item_id, word)?;
    let next = walk(&graph, &at, word, &mut counts, VISIT_CAP);
    append(conn, &graph, &run_id, &item_id, &next.visits, now)?;
    // The same rule as the live queue (`learn_path::rate`): back after `gap` items when the walk ends at a rate node.
    Ok(Some(match next.end {
        End::AtRate(_) => Back::After(gap(rating)),
        _ => Back::Out,
    }))
}

fn append(
    conn: &Connection,
    graph: &LearnGraph,
    run_id: &str,
    item_id: &str,
    visits: &[Visit],
    now: UnixMs,
) -> Result<(), StoreError> {
    for v in visits {
        let kind = graph
            .nodes
            .iter()
            .find(|n| n.id == v.node)
            .map_or("unknown", |n| store::kind_name(n.kind));
        let seq = next_learn_seq(conn, run_id)?;
        let row = StepRow {
            run_uid: run_id,
            seq,
            item_uid: item_id,
            node_id: &v.node,
            node_kind: kind,
            edge_when: v.via.as_deref(),
            t_ms: now.0,
        };
        insert_learn_step(conn, &row)?;
    }
    Ok(())
}
