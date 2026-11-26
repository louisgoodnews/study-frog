//! Exam Scheduler UI state, messages, effects, and pure update logic (EPIC-076, STORY-0248).
//!
//! Complies with Invariant P1: zero inherent impl blocks; all logic in free functions.

use studyfrog_calendar::exam_plan::{ExamPlan, ExamTarget, PrepWindow};
use studyfrog_calendar::exam_schedule::ExamScheduleOutput;
use studyfrog_calendar::exam_window::ProposedWindow;
use studyfrog_core::civil::{Date, add_days, format_date, parse_date};
use studyfrog_core::uid::Uid;

use crate::effect::Effect;
use crate::planner::Planner;
use crate::state::State;

/// Form state for creating or editing an exam schedule plan.
#[derive(Debug, Clone, PartialEq)]
pub struct ExamSchedForm {
    /// Existing plan ID when editing, or `None` for a new exam plan.
    pub editing: Option<Uid>,
    /// Exam title.
    pub title: String,
    /// Target exam date (`YYYY-MM-DD`).
    pub exam_date: String,
    /// `true` = StudyFrog-proposed week window; `false` = user-set weeks.
    pub use_proposed: bool,
    /// User-set preparation window in weeks (when `use_proposed == false`).
    pub custom_weeks: String,
    /// Latest StudyFrog-proposed window and factor reasons.
    pub proposed: Option<ProposedWindow>,
    /// Attached stacks and individual items.
    pub targets: Vec<ExamTarget>,
    /// Live preview of generated daily blocks and warnings.
    pub preview: Option<ExamScheduleOutput>,
    /// Validation or persistence error message.
    pub error: Option<String>,
}

/// Messages emitted by the Exam Scheduler UI and worker.
#[derive(Debug, Clone, PartialEq)]
pub enum ExamSchedMsg {
    /// Open form for a new exam plan.
    OpenNew,
    /// Open form for an existing exam plan.
    OpenEdit(Uid),
    /// Close the Exam Scheduler form without saving.
    Close,
    /// Edit exam title.
    SetTitle(String),
    /// Edit exam target date (`YYYY-MM-DD`).
    SetDate(String),
    /// Toggle between StudyFrog-proposed (`true`) and Custom weeks (`false`).
    SetUseProposed(bool),
    /// Edit custom preparation window in weeks.
    SetCustomWeeks(String),
    /// Attach or detach a stack folder.
    ToggleStack(String),
    /// Attach or detach an individual item.
    ToggleItem(Uid),
    /// Recompute proposed window and workload preview on the worker.
    RefreshPreview,
    /// Apply and persist the exam plan and generated blocks.
    Apply,
    /// Re-balance an existing persisted exam plan from today forward.
    Rebalance(Uid),
    /// Delete a persisted exam plan.
    Delete(Uid),
    /// Worker reply with all persisted exam plans.
    PlansLoaded(Result<Vec<ExamPlan>, String>),
    /// Worker reply with computed proposed window and preview blocks.
    PreviewReady(Result<(Option<ProposedWindow>, ExamScheduleOutput), String>),
    /// Worker reply after saving, re-balancing, or deleting an exam plan.
    Saved(Result<Vec<ExamPlan>, String>),
    /// Launch a one-click rehearsal run for a scheduled exam block's items.
    RehearseBlock(Vec<Uid>),
}

/// Side effects requested by the Exam Scheduler.
#[derive(Debug, Clone, PartialEq)]
pub enum ExamSchedEffect {
    /// Load all persisted exam plans from SQLite.
    LoadPlans,
    /// Estimate proposed window (when requested) and preview daily schedule blocks.
    Preview {
        /// Attached stacks and items.
        targets: Vec<ExamTarget>,
        /// Target exam date.
        exam_date: Date,
        /// `None` = use StudyFrog proposal; `Some(w)` = user-set weeks.
        custom_weeks: Option<u16>,
    },
    /// Validate, generate schedule blocks, and persist an exam plan.
    SavePlan(ExamPlan),
    /// Re-balance an existing exam plan from today forward.
    RebalancePlan(Uid),
    /// Delete an exam plan by ID.
    DeletePlan(Uid),
    /// Start a practice rehearsal drill for `item_uids`.
    StartExamRun(Vec<Uid>),
}

/// Create a fresh `ExamSchedForm` defaulting to 3 weeks ahead and Proposed mode.
pub fn form_new(today: Date) -> ExamSchedForm {
    let default_date = add_days(today, 21);
    ExamSchedForm {
        editing: None,
        title: String::new(),
        exam_date: format_date(default_date),
        use_proposed: true,
        custom_weeks: "3".to_owned(),
        proposed: None,
        targets: Vec::new(),
        preview: None,
        error: None,
    }
}

/// Populate an `ExamSchedForm` from an existing persisted `ExamPlan`.
pub fn form_from(plan: &ExamPlan) -> ExamSchedForm {
    let (use_proposed, custom_weeks, proposed) = match &plan.window {
        PrepWindow::UserWeeks(w) => (false, w.to_string(), None),
        PrepWindow::Proposed { weeks, reasons } => (
            true,
            weeks.to_string(),
            Some(ProposedWindow {
                weeks: *weeks,
                reasons: reasons.clone(),
            }),
        ),
    };
    ExamSchedForm {
        editing: Some(plan.id),
        title: plan.title.clone(),
        exam_date: format_date(plan.exam_date),
        use_proposed,
        custom_weeks,
        proposed,
        targets: plan.targets.clone(),
        preview: Some(ExamScheduleOutput {
            blocks: plan.blocks.clone(),
            warnings: Vec::new(),
        }),
        error: None,
    }
}

fn preview_effect(f: &ExamSchedForm) -> Option<Effect> {
    let exam_date = parse_date(f.exam_date.trim())?;
    let custom_weeks = if f.use_proposed {
        None
    } else {
        Some(
            f.custom_weeks
                .trim()
                .parse::<u16>()
                .ok()
                .filter(|&w| w > 0)?,
        )
    };
    Some(Effect::ExamSched(ExamSchedEffect::Preview {
        targets: f.targets.clone(),
        exam_date,
        custom_weeks,
    }))
}

fn toggle_target(targets: &mut Vec<ExamTarget>, target: ExamTarget) {
    if let Some(pos) = targets.iter().position(|t| *t == target) {
        targets.remove(pos);
    } else {
        targets.push(target);
    }
}

fn build_plan_from_form(f: &ExamSchedForm, today: Date) -> Result<ExamPlan, String> {
    if f.title.trim().is_empty() {
        return Err("Enter an exam title.".to_owned());
    }
    let exam_date = parse_date(f.exam_date.trim())
        .ok_or_else(|| "Date must look like 2026-12-20.".to_owned())?;
    let window = if f.use_proposed {
        let prop = f.proposed.clone().unwrap_or(ProposedWindow {
            weeks: 2,
            reasons: Vec::new(),
        });
        PrepWindow::Proposed {
            weeks: prop.weeks,
            reasons: prop.reasons,
        }
    } else {
        let w = f
            .custom_weeks
            .trim()
            .parse::<u16>()
            .ok()
            .filter(|&w| w > 0)
            .ok_or_else(|| "Preparation weeks must be a positive whole number.".to_owned())?;
        PrepWindow::UserWeeks(w)
    };
    let plan = ExamPlan {
        id: f.editing.unwrap_or(Uid(0)),
        title: f.title.trim().to_owned(),
        exam_date,
        window,
        targets: f.targets.clone(),
        blocks: Vec::new(),
        created_ms: 0,
        updated_ms: 0,
    };
    studyfrog_calendar::exam_plan::validate_exam_plan(&plan, today)?;
    Ok(plan)
}

fn set_form(state: State, form: Option<ExamSchedForm>, fx: Vec<Effect>) -> (State, Vec<Effect>) {
    let planner = Planner {
        exam_form: form,
        ..state.planner
    };
    (State { planner, ..state }, fx)
}

/// Pure update handler for `ExamSchedMsg`.
pub fn update(state: State, msg: ExamSchedMsg) -> (State, Vec<Effect>) {
    let p = &state.planner;
    match msg {
        ExamSchedMsg::OpenNew => set_form(state, Some(form_new(p.today)), Vec::new()),
        ExamSchedMsg::OpenEdit(id) => {
            let form = p.exam_plans.iter().find(|e| e.id == id).map(form_from);
            let fx = form.as_ref().and_then(preview_effect).into_iter().collect();
            set_form(state, form, fx)
        }
        ExamSchedMsg::Close => set_form(state, None, Vec::new()),
        ExamSchedMsg::PlansLoaded(Ok(plans)) | ExamSchedMsg::Saved(Ok(plans)) => {
            let planner = Planner {
                exam_plans: plans,
                exam_form: None,
                ..state.planner
            };
            (State { planner, ..state }, Vec::new())
        }
        ExamSchedMsg::PlansLoaded(Err(e)) | ExamSchedMsg::Saved(Err(e)) => {
            let mut form = state.planner.exam_form.clone();
            if let Some(ref mut f) = form {
                f.error = Some(e);
            }
            set_form(state, form, Vec::new())
        }
        ExamSchedMsg::PreviewReady(Ok((prop, out))) => {
            let mut form = state.planner.exam_form.clone();
            if let Some(ref mut f) = form {
                if let Some(p) = prop {
                    f.proposed = Some(p);
                }
                f.preview = Some(out);
                f.error = None;
            }
            set_form(state, form, Vec::new())
        }
        ExamSchedMsg::PreviewReady(Err(e)) => {
            let mut form = state.planner.exam_form.clone();
            if let Some(ref mut f) = form {
                f.error = Some(e);
            }
            set_form(state, form, Vec::new())
        }
        ExamSchedMsg::Rebalance(id) => (
            state,
            vec![Effect::ExamSched(ExamSchedEffect::RebalancePlan(id))],
        ),
        ExamSchedMsg::Delete(id) => (
            state,
            vec![Effect::ExamSched(ExamSchedEffect::DeletePlan(id))],
        ),
        ExamSchedMsg::RehearseBlock(uids) => (
            state,
            vec![Effect::ExamSched(ExamSchedEffect::StartExamRun(uids))],
        ),
        ExamSchedMsg::Apply => match p.exam_form.as_ref() {
            Some(f) => match build_plan_from_form(f, p.today) {
                Ok(plan) => (
                    state,
                    vec![Effect::ExamSched(ExamSchedEffect::SavePlan(plan))],
                ),
                Err(e) => {
                    let mut form = p.exam_form.clone();
                    if let Some(ref mut fm) = form {
                        fm.error = Some(e);
                    }
                    set_form(state, form, Vec::new())
                }
            },
            None => (state, Vec::new()),
        },
        other => {
            let mut form = p.exam_form.clone();
            let mut want_preview = false;
            if let Some(ref mut f) = form {
                f.error = None;
                match other {
                    ExamSchedMsg::SetTitle(t) => f.title = t,
                    ExamSchedMsg::SetDate(d) => {
                        f.exam_date = d;
                        want_preview = true;
                    }
                    ExamSchedMsg::SetUseProposed(u) => {
                        f.use_proposed = u;
                        want_preview = true;
                    }
                    ExamSchedMsg::SetCustomWeeks(w) => {
                        f.custom_weeks = w;
                        want_preview = true;
                    }
                    ExamSchedMsg::ToggleStack(s) => {
                        toggle_target(&mut f.targets, ExamTarget::Stack(s));
                        want_preview = true;
                    }
                    ExamSchedMsg::ToggleItem(u) => {
                        toggle_target(&mut f.targets, ExamTarget::Item(u));
                        want_preview = true;
                    }
                    ExamSchedMsg::RefreshPreview => want_preview = true,
                    _ => {}
                }
            }
            let fx = if want_preview {
                form.as_ref().and_then(preview_effect).into_iter().collect()
            } else {
                Vec::new()
            };
            set_form(state, form, fx)
        }
    }
}
