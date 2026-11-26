//! Exam Scheduler update handler (EPIC-076, STORY-0248).

use studyfrog_calendar::exam_plan::ExamTarget;

use crate::effect::{Effect, EffectPlanner};
use crate::planner::Planner;
use crate::planner::exam_sched::{
    ExamSchedEffect, ExamSchedForm, ExamSchedMsg, build_plan_from_form, form_from, form_new,
    preview_effect, toggle_target,
};
use crate::state::State;

fn set_form(state: State, form: Option<ExamSchedForm>, fx: Vec<Effect>) -> (State, Vec<Effect>) {
    let planner = Planner {
        exam_form: form,
        ..state.planner
    };
    (State { planner, ..state }, fx)
}

fn fx(e: ExamSchedEffect) -> Vec<Effect> {
    vec![Effect::Planner(EffectPlanner::ExamSched(e))]
}

/// Pure update handler for `ExamSchedMsg`.
pub fn update(state: State, msg: ExamSchedMsg) -> (State, Vec<Effect>) {
    let today = state.planner.today;
    match msg {
        ExamSchedMsg::OpenNew => set_form(state, Some(form_new(today)), Vec::new()),
        ExamSchedMsg::OpenEdit(id) => {
            let form = state
                .planner
                .exam_plans
                .iter()
                .find(|e| e.id == id)
                .map(form_from);
            let fxs = form.as_ref().and_then(preview_effect).into_iter().collect();
            set_form(state, form, fxs)
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
        ExamSchedMsg::PlansLoaded(Err(e))
        | ExamSchedMsg::Saved(Err(e))
        | ExamSchedMsg::PreviewReady(Err(e)) => {
            let mut form = state.planner.exam_form.clone();
            if let Some(ref mut f) = form {
                f.error = Some(e);
            }
            set_form(state, form, Vec::new())
        }
        ExamSchedMsg::PreviewReady(Ok((prop, out))) => {
            let mut form = state.planner.exam_form.clone();
            if let Some(ref mut f) = form {
                if let Some(pr) = prop {
                    f.proposed = Some(pr);
                }
                f.preview = Some(out);
                f.error = None;
            }
            set_form(state, form, Vec::new())
        }
        ExamSchedMsg::Rebalance(id) => (state, fx(ExamSchedEffect::RebalancePlan(id))),
        ExamSchedMsg::Delete(id) => (state, fx(ExamSchedEffect::DeletePlan(id))),
        ExamSchedMsg::RehearseBlock(uids) => (state, fx(ExamSchedEffect::StartExamRun(uids))),
        ExamSchedMsg::Apply => match state.planner.exam_form.as_ref() {
            Some(f) => match build_plan_from_form(f, today) {
                Ok(plan) => (state, fx(ExamSchedEffect::SavePlan(plan))),
                Err(e) => {
                    let mut form = state.planner.exam_form.clone();
                    if let Some(ref mut fm) = form {
                        fm.error = Some(e);
                    }
                    set_form(state, form, Vec::new())
                }
            },
            None => (state, Vec::new()),
        },
        other => edit_form(state, other),
    }
}

fn edit_form(state: State, msg: ExamSchedMsg) -> (State, Vec<Effect>) {
    let mut form = state.planner.exam_form.clone();
    let mut want_preview = false;
    if let Some(ref mut f) = form {
        f.error = None;
        match msg {
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
    let fxs = if want_preview {
        form.as_ref().and_then(preview_effect).into_iter().collect()
    } else {
        Vec::new()
    };
    set_form(state, form, fxs)
}
