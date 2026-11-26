//! Application service for the Exam Scheduler (EPIC-076): target resolution, window estimation,
//! schedule generation, re-balancing, and persistence.

use std::collections::HashSet;

use studyfrog_calendar::exam_plan::{ExamPlan, ExamTarget, PrepWindow, validate_exam_plan};
use studyfrog_calendar::exam_schedule::{
    ExamSchedItem, ExamScheduleOutput, build_exam_schedule, rebalance_exam_schedule,
};
use studyfrog_calendar::exam_window::{ExamPrepStats, ProposedWindow, propose_prep_weeks};
use studyfrog_core::civil::Date;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_store::exam_plan::{
    delete_exam_plan, list_exam_plans, load_exam_plan, save_exam_plan,
};
use studyfrog_store::schedule::load_mem_state;

use crate::library_rows::{LibraryItem, load_library_rows};
use crate::session::{ServiceError, VaultSession};

fn matches_targets(item: &LibraryItem, targets: &[ExamTarget]) -> bool {
    targets.iter().any(|t| match t {
        ExamTarget::Item(uid) => *uid == item.uid,
        ExamTarget::Stack(folder) => item.row.stacks.iter().any(|s| s == folder),
    })
}

/// Resolve all unique vault items attached to `targets` into `ExamSchedItem` entries.
pub fn resolve_target_items(
    session: &VaultSession,
    targets: &[ExamTarget],
    now: UnixMs,
) -> Result<Vec<ExamSchedItem>, ServiceError> {
    if targets.is_empty() {
        return Ok(Vec::new());
    }
    let rows = load_library_rows(session, now)?;
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for item in rows {
        if !matches_targets(&item, targets) || !seen.insert(item.uid) {
            continue;
        }
        let stability_days = load_mem_state(&session.conn, item.uid)
            .ok()
            .flatten()
            .map_or(0.0, |m| m.state.stability);
        out.push(ExamSchedItem {
            uid: item.uid,
            is_new: item.row.reviews == 0,
            familiarity: item.row.familiarity.unwrap_or(0.0),
            stability_days,
        });
    }
    Ok(out)
}

/// Compute `ExamPrepStats` and propose an evidence-based preparation window in weeks.
pub fn estimate_exam_window(
    session: &VaultSession,
    targets: &[ExamTarget],
    today: Date,
    exam_date: Date,
    daily_capacity: usize,
    now: UnixMs,
) -> Result<ProposedWindow, ServiceError> {
    let items = resolve_target_items(session, targets, now)?;
    let total_items = items.len();
    if total_items == 0 {
        let stats = ExamPrepStats {
            total_items: 0,
            new_items: 0,
            avg_familiarity: 0.0,
            avg_stability_days: 0.0,
            recent_accuracy: 1.0,
            lapse_rate: 0.0,
            daily_capacity,
        };
        return Ok(propose_prep_weeks(&stats, today, exam_date));
    }
    let new_items = items.iter().filter(|i| i.is_new).count();
    let studied: Vec<&ExamSchedItem> = items.iter().filter(|i| !i.is_new).collect();
    let avg_familiarity = if studied.is_empty() {
        0.0
    } else {
        studied.iter().map(|i| i.familiarity).sum::<f32>() / studied.len() as f32
    };
    let avg_stability_days = if studied.is_empty() {
        0.0
    } else {
        studied.iter().map(|i| i.stability_days).sum::<f32>() / studied.len() as f32
    };
    let weak_count = studied.iter().filter(|i| i.familiarity < 0.45).count();
    let lapse_rate = weak_count as f32 / total_items as f32;
    let recent_accuracy = (0.55 + 0.40 * avg_familiarity).clamp(0.45, 0.98);

    let stats = ExamPrepStats {
        total_items,
        new_items,
        avg_familiarity,
        avg_stability_days,
        recent_accuracy,
        lapse_rate,
        daily_capacity,
    };
    Ok(propose_prep_weeks(&stats, today, exam_date))
}

/// Preview or build an `ExamPlan`'s schedule blocks without writing to SQLite.
pub fn preview_exam_plan(
    session: &VaultSession,
    plan: &ExamPlan,
    today: Date,
    daily_capacity: usize,
    now: UnixMs,
) -> Result<(ExamPlan, ExamScheduleOutput), ServiceError> {
    let items = resolve_target_items(session, &plan.targets, now)?;
    let window = match &plan.window {
        PrepWindow::UserWeeks(w) => PrepWindow::UserWeeks(*w),
        PrepWindow::Proposed { .. } => {
            let prop = estimate_exam_window(
                session,
                &plan.targets,
                today,
                plan.exam_date,
                daily_capacity,
                now,
            )?;
            PrepWindow::Proposed {
                weeks: prop.weeks,
                reasons: prop.reasons,
            }
        }
    };
    let weeks = studyfrog_calendar::exam_plan::prep_window_weeks(&window);
    let out = build_exam_schedule(&items, today, plan.exam_date, weeks, daily_capacity);
    let built = ExamPlan {
        window,
        blocks: out.blocks.clone(),
        ..plan.clone()
    };
    Ok((built, out))
}

/// Validate, generate schedule blocks, and persist an `ExamPlan` in SQLite.
pub fn save_and_apply_exam_plan(
    session: &mut VaultSession,
    plan: &ExamPlan,
    today: Date,
    daily_capacity: usize,
    now: UnixMs,
) -> Result<(ExamPlan, ExamScheduleOutput), ServiceError> {
    let (mut built, out) = preview_exam_plan(session, plan, today, daily_capacity, now)?;
    validate_exam_plan(&built, today).map_err(ServiceError::Invalid)?;
    if built.created_ms == 0 {
        built.created_ms = now.0;
    }
    built.updated_ms = now.0;
    save_exam_plan(&mut session.conn, &built).map_err(ServiceError::Store)?;
    Ok((built, out))
}

/// Re-balance an existing `ExamPlan` from `today` forward and persist the updated blocks.
pub fn rebalance_and_save_exam_plan(
    session: &mut VaultSession,
    id: Uid,
    today: Date,
    daily_capacity: usize,
    now: UnixMs,
) -> Result<(ExamPlan, ExamScheduleOutput), ServiceError> {
    let mut plan = load_exam_plan(&session.conn, id)
        .map_err(ServiceError::Store)?
        .ok_or_else(|| ServiceError::Invalid("Exam plan not found.".to_owned()))?;
    let items = resolve_target_items(session, &plan.targets, now)?;
    let weeks = studyfrog_calendar::exam_plan::prep_window_weeks(&plan.window);
    let out = rebalance_exam_schedule(
        &plan.blocks,
        &items,
        today,
        plan.exam_date,
        weeks,
        daily_capacity,
    );
    plan.blocks = out.blocks.clone();
    plan.updated_ms = now.0;
    save_exam_plan(&mut session.conn, &plan).map_err(ServiceError::Store)?;
    Ok((plan, out))
}

/// List all persisted exam plans in the open vault.
pub fn list_vault_exam_plans(session: &VaultSession) -> Result<Vec<ExamPlan>, ServiceError> {
    list_exam_plans(&session.conn).map_err(ServiceError::Store)
}

/// Delete a persisted exam plan by `id`.
pub fn delete_vault_exam_plan(session: &VaultSession, id: Uid) -> Result<bool, ServiceError> {
    delete_exam_plan(&session.conn, id).map_err(ServiceError::Store)
}
