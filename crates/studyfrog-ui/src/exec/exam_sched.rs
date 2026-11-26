//! Worker executor for Exam Scheduler effects (EPIC-076).

use studyfrog_calendar::exam_plan::{ExamPlan, PrepWindow};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_service::create::fresh_id_source;
use studyfrog_service::exam_scheduler::{
    delete_vault_exam_plan, list_vault_exam_plans, preview_exam_plan, rebalance_and_save_exam_plan,
    save_and_apply_exam_plan,
};
use studyfrog_service::session::current_settings;
use studyfrog_service::timezone::local_today;

use crate::exec::Worker;
use crate::msg::Msg;
use crate::planner::PlanMsg;
use crate::planner::exam_sched::{ExamSchedEffect, ExamSchedMsg};

fn wrap(m: ExamSchedMsg) -> Msg {
    Msg::Plan(PlanMsg::ExamSched(m))
}

fn daily_capacity(w: &Worker) -> usize {
    w.session
        .as_ref()
        .map_or(20, |s| current_settings(s).daily_review_limit.max(10))
}

/// Execute an `ExamSchedEffect` against the open vault.
pub fn run(w: &mut Worker, effect: ExamSchedEffect, now_ns: i64) -> Msg {
    let now = UnixMs(now_ns / 1_000_000);
    let today = local_today(now);
    let cap = daily_capacity(w);

    match effect {
        ExamSchedEffect::LoadPlans => {
            let res = match &w.session {
                Some(s) => list_vault_exam_plans(s).map_err(|e| e.to_string()),
                None => Ok(Vec::new()),
            };
            wrap(ExamSchedMsg::PlansLoaded(res))
        }
        ExamSchedEffect::Preview {
            targets,
            exam_date,
            custom_weeks,
        } => {
            let Some(s) = &w.session else {
                return wrap(ExamSchedMsg::PreviewReady(Err("no vault is open".into())));
            };
            let window = match custom_weeks {
                Some(w) => PrepWindow::UserWeeks(w),
                None => PrepWindow::Proposed {
                    weeks: 1,
                    reasons: Vec::new(),
                },
            };
            let dummy = ExamPlan {
                id: Uid(0),
                title: "Preview".into(),
                exam_date,
                window,
                targets,
                blocks: Vec::new(),
                created_ms: 0,
                updated_ms: 0,
            };
            let res = preview_exam_plan(s, &dummy, today, cap, now)
                .map(|(built, out)| {
                    let prop = match built.window {
                        PrepWindow::Proposed { weeks, reasons } => {
                            Some(studyfrog_calendar::exam_window::ProposedWindow { weeks, reasons })
                        }
                        PrepWindow::UserWeeks(_) => None,
                    };
                    (prop, out)
                })
                .map_err(|e| e.to_string());
            wrap(ExamSchedMsg::PreviewReady(res))
        }
        ExamSchedEffect::SavePlan(mut plan) => {
            let Some(s) = &mut w.session else {
                return wrap(ExamSchedMsg::Saved(Err("no vault is open".into())));
            };
            if plan.id == Uid(0) {
                let ids = match fresh_id_source(u64::try_from(now.0).unwrap_or(0)) {
                    Ok(ids) => ids,
                    Err(e) => return wrap(ExamSchedMsg::Saved(Err(e.to_string()))),
                };
                plan.id = studyfrog_md::ulid::compose_ulid(ids.now_ms, ids.random);
            }
            let res = save_and_apply_exam_plan(s, &plan, today, cap, now)
                .and_then(|_| list_vault_exam_plans(s))
                .map_err(|e| e.to_string());
            wrap(ExamSchedMsg::Saved(res))
        }
        ExamSchedEffect::RebalancePlan(id) => {
            let Some(s) = &mut w.session else {
                return wrap(ExamSchedMsg::Saved(Err("no vault is open".into())));
            };
            let res = rebalance_and_save_exam_plan(s, id, today, cap, now)
                .and_then(|_| list_vault_exam_plans(s))
                .map_err(|e| e.to_string());
            wrap(ExamSchedMsg::Saved(res))
        }
        ExamSchedEffect::DeletePlan(id) => {
            let Some(s) = &w.session else {
                return wrap(ExamSchedMsg::Saved(Err("no vault is open".into())));
            };
            let res = delete_vault_exam_plan(s, id)
                .and_then(|_| list_vault_exam_plans(s))
                .map_err(|e| e.to_string());
            wrap(ExamSchedMsg::Saved(res))
        }
        ExamSchedEffect::StartExamRun(uids) => match &w.session {
            Some(s) => crate::exec_run::drill(s, &uids, now_ns),
            None => wrap(ExamSchedMsg::Saved(Err("no vault is open".into()))),
        },
    }
}
